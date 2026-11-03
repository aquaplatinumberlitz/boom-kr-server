"""Fixture-backed KR diagnostic service, one serialized owner, two live local clients with one replacement identity.

Run as ``python3 -m server.probe_service``. Authentication/roster/map/rank bodies
are explicitly synthetic profile fixtures. Accept order assigns four connections
each for A, B, then replacement C (at most two live members). Not a public login server.
No Wine/UI automation is imported. The caller supplies network isolation.
"""

import argparse
import hashlib
import json
from pathlib import Path
import selectors
import signal
import socket
import struct
import time

from server import probe_codec as codec
from server.item_claims import ItemSpec
from server.probe_room import ProbeRoom


class FixtureProfile:
    def __init__(self, path: Path):
        self.data = json.loads(path.read_text())
        if self.data.get('schema') != 'kr-probe-profile-v1':
            raise ValueError('unsupported profile')
        self.frames = {}
        for name, meta in self.data['fixtures'].items():
            raw = Path(meta['path']).read_bytes()
            if len(raw) != meta['bytes'] or hashlib.sha256(raw).hexdigest() != meta['sha256']:
                raise ValueError(f'fixture hash mismatch: {name}')
            if not 1 <= len(raw) <= 65536:
                raise ValueError('unsupported fixture size')
            self.frames[name] = raw
            if meta['sequence'] is not None:
                try:
                    codec.frame_body(raw, meta['sequence'])
                except ValueError as error:
                    raise ValueError(f'invalid fixture {name}: {error}') from error

    def raw(self, name):
        return self.frames[name]

    def body(self, name):
        sequence = self.data['fixtures'][name]['sequence']
        if sequence is None:
            raise ValueError('fixture does not declare sequence: ' + name)
        return codec.frame_body(self.raw(name), sequence)


ROLE_SLOTS = {'A': 0, 'B': 1, 'C': 0}
ROLE_IDS = {'A': 1, 'B': 2, 'C': 3}


class ProbeService:
    def __init__(self, profile: FixtureProfile, out: Path, proxy_ports: dict):
        missing = {'rank_Awin', 'promote_B', 'auth_C', 'roomlist_C', 'join_C',
                   'player_C', 'slot_C', 'guest_C', 'ready_C'} - profile.frames.keys()
        if missing:
            raise ValueError('profile missing lifecycle fixtures: ' + ', '.join(sorted(missing)))
        self.profile, self.out, self.proxy_ports = profile, out, proxy_ports
        self.hidden_items = {item['index']: ItemSpec(**item)
                             for item in profile.data.get('hidden_items', [])}
        if len(self.hidden_items) != len(profile.data.get('hidden_items', [])):
            raise ValueError('duplicate hidden item index')
        self.room = ProbeRoom([ItemSpec(**item) for item in profile.data['items']]
                              + list(self.hidden_items.values()),
                              profile.data['initial_key'])
        self.selector = selectors.DefaultSelector()
        self.connections, self.events, self.jobs = {}, [], []
        self.channels = {}
        self.host_role = 'A'
        self.closed = False
        self.round = {'index': 0}
        self.state = {'schema': 'kr-probe-service-v1', 'tcp_events': self.events,
                      'round_bombs': [], 'bomb_requests': [], 'block_reports': [],
                      'event_reports': [], 'round_history': [],
                      'item_requests': [], 'policy_events': [], 'errors': [],
                      'source': 'emu_choice', 'profile': profile.data,
                      'limits': 'Fixture auth/roster/rank, loopback-only, two live members/A-B then B-C by accept order; no fullmatch gate.'}
        self.state['policy_sources'] = []
        for filename in ['probe_service.py', 'probe_codec.py', 'probe_room.py', 'round_fence.py',
                         'item_claims.py', 'udp_round_keys.py']:
            path = Path(__file__).with_name(filename)
            self.state['policy_sources'].append({'path': str(path.resolve()),
                                                 'sha256': hashlib.sha256(path.read_bytes()).hexdigest()})

    def peers(self, marker='map_response_sent'):
        return [event for event in self.events if event.get('reply_ordinal') == 4
                and not event.get('closed') and event.get(marker)]

    def send_raw(self, event, raw):
        if event.get('closed'):
            raise RuntimeError('attempt to send on a closed connection')
        self.connections[event['global_ordinal']]['socket'].sendall(raw)

    def send(self, event, name, label, *, body=None, opcode=None):
        template = self.profile.raw(name)
        if body is None:
            body = self.profile.body(name)
        if opcode is None:
            opcode = struct.unpack_from('<I', template, 5)[0]
        sequence = event.get('s2c_next_seq', 73)
        frame = codec.encode(template[:37], opcode, body, sequence)
        self.send_raw(event, frame)
        event['s2c_next_seq'] = sequence + 3
        filename = f"{event['role']}-{label}-{sequence}.bin"
        (self.out / filename).write_bytes(frame)
        meta = {'seq': sequence, 'opcode': opcode, 'fixture': filename,
                'sha256': hashlib.sha256(frame).hexdigest(), 'size': len(frame),
                'monotonic_ns': time.monotonic_ns()}
        event.setdefault('dynamic_s2c', []).append(meta)
        return meta

    def broadcast(self, name, label, **kwargs):
        return [{'role': peer['role'], **self.send(peer, name, label, **kwargs)}
                for peer in self.peers()]

    def start_round(self):
        key = self.room.begin()
        if self.round['index']:
            self.state['round_history'].append(self.round)
        self.round = {'index': self.room.generation, 'key': key, 'host_role': self.host_role,
                      'revealed_hidden': {},
                      'roster': [{'role': e['role'], 'slot': ROLE_SLOTS[e['role']],
                                  'identity': ROLE_IDS[e['role']]} for e in self.peers()]}
        self.state['current_round'] = self.round
        self.state['policy_events'].append({'op': 'begin', 'generation': self.room.generation,
                                            'key': key, 'monotonic_ns': time.monotonic_ns()})
        for event in self.peers():
            markers = ['start_response_sent', 'init99', 'go_response_sent', 'play_response_sent',
                       'begin_response_sent', 'peer153_sent', 'death_event', 'pickup_ineligible',
                       'bomb_serials']
            if event.get('start_response_sent'):
                event.setdefault('rounds', []).append({key: event[key] for key in markers if key in event})
            for marker in markers:
                event.pop(marker, None)
            event['floor582_sent'] = self.send(event, 'floor', 'floor582')
            event['start_response_sent'] = {**self.send(event, 'start', f"start121-r{self.round['index']}"),
                                             'round': self.round['index'], 'after_c2s_opcode': 104,
                                             'host_role': self.host_role, 'broadcast': True}

    def leave_room(self, event, frame, sequence, body):
        """Bounded fixture leave: A alone after B EOF, or idle guest B with A mapped.

        The channel identity, codec sequence and round/key/job history survive.
        Other leave variants are logged as unsupported, without mutating membership.
        """
        now = time.monotonic_ns()
        request = {'role': event['role'], 'seq': sequence, 'monotonic_ns': now,
                   'frame_sha256': hashlib.sha256(frame).hexdigest(),
                   'body_hex': body.hex()}
        self.state.setdefault('room_leave_requests', []).append(request)
        others = [peer for peer in self.events if peer is not event
                  and peer.get('reply_ordinal') == 4 and not peer.get('closed')
                  and peer.get('create_response_sent')]
        guest = self.channels.get('B')
        empty_host = (event['role'] == 'A' and self.host_role == 'A' and not others
                      and guest and guest.get('closed') and guest.get('create_response_sent')
                      and self.round.get('refresh'))
        idle_guest = (event['role'] == 'B' and self.host_role == 'A'
                      and len(others) == 1 and others[0]['role'] == 'A'
                      and others[0].get('map_response_sent'))
        if body:
            request['rejected'] = 'unsupported_leave_body'
        elif not event.get('create_response_sent'):
            request['rejected'] = 'not_room_member'
        elif self.room.active or (self.round.get('index') and not self.round.get('refresh')):
            request['rejected'] = 'round_not_returned'
        elif not (empty_host or idle_guest):
            request['rejected'] = 'unsupported_leave_membership_variant'
        else:
            # Older exports can send fixed templates on initial setup, but lack
            # sequence metadata needed to re-encode them. Fail before releasing
            # the member or emitting refresh, rather than crashing mid-recreate.
            try:
                role = event['role']
                aliases = ['ready', 'roomlist_' + role, 'join' if idle_guest else 'create',
                           'player_' + role, 'slot_' + role, 'table', 'map']
                if idle_guest:
                    aliases += ['guest_A', 'guest_B']
                for alias in aliases:
                    self.profile.body(alias)
            except ValueError as error:
                request['rejected'] = 'unsupported_reentry_profile'
                request['detail'] = str(error)
                return
            markers = ['create_response_sent', 'player_response_sent', 'slot_response_sent',
                       'guest_response_sent', 'map_response_sent', 'guest_ready_received',
                       'floor582_sent', 'start_response_sent', 'init99', 'go_response_sent',
                       'play_response_sent', 'begin_response_sent', 'peer153_sent', 'death_event',
                       'pickup_ineligible', 'bomb_serials', 'pending_departures', 'departures_sent']
            request['previous_membership'] = {key: event.pop(key) for key in markers if key in event}
            event.setdefault('room_visits', []).append(request)
            event['last_room_leave'] = request
            event['room_reentry'] = True
            if idle_guest:
                request['departure'] = self.cleanup_departed('B')
            self.state['policy_events'].append({'op': 'room_leave', 'role': event['role'],
                                               'generation': self.room.generation,
                                               'visit': len(event['room_visits']),
                                               'monotonic_ns': now})

    def send_setup(self, event, aliases, label):
        if event.get('room_reentry'):
            return [self.send(event, alias, label + '-' + alias) for alias in aliases]
        self.send_raw(event, b''.join(self.profile.raw(alias) for alias in aliases))
        return []

    def receive_channel(self, event, frame, sequence, body):
        opcode = struct.unpack_from('<I', frame, 5)[0]
        role = event['role']
        if event.get('closed'):
            return
        if opcode == 86:
            self.leave_room(event, frame, sequence, body)
        elif opcode == 6 and not event.get('channel_response_sent'):
            self.send_raw(event, self.profile.raw('channel'))
            event['channel_response_sent'] = {'monotonic_ns': time.monotonic_ns()}
        elif opcode == 67 and not event.get('ready_response_sent'):
            self.send_raw(event, self.profile.raw('ready') + self.profile.raw(f'roomlist_{role}')
                          + self.profile.raw('channel_list'))
            event['ready_response_sent'] = {'monotonic_ns': time.monotonic_ns()}
        elif opcode in (81, 82) and not event.get('create_response_sent'):
            if (role, opcode) not in [('A', 81), ('B', 82), ('C', 82)]:
                raise ValueError('unsupported fixture role/join')
            previous = event.get('last_room_leave')
            if previous and not previous.get('refresh'):
                self.state.setdefault('room_reentry_rejections', []).append({
                    'role': role, 'seq': sequence, 'reason': 'lobby_refresh_pending',
                    'frame_sha256': hashlib.sha256(frame).hexdigest()})
                return
            if previous and role == 'B':
                host = self.channels.get('A')
                if not host or host.get('closed') or not host.get('map_response_sent'):
                    self.state.setdefault('room_reentry_rejections', []).append({
                        'role': role, 'seq': sequence, 'reason': 'host_membership_changed',
                        'frame_sha256': hashlib.sha256(frame).hexdigest()})
                    return
            alias = {'A': 'create', 'B': 'join', 'C': 'join_C'}[role]
            sent = self.send_setup(event, [alias], 'reentry-create')
            event['create_response_sent'] = {'monotonic_ns': time.monotonic_ns(),
                                             'after_c2s_opcode': opcode}
            if previous:
                previous['recreate'] = {'seq': sequence, 'frame_sha256': hashlib.sha256(frame).hexdigest(),
                                        'monotonic_ns': time.monotonic_ns(), 'reply': sent}

        elif opcode == 585 and event.get('create_response_sent') and not event.get('player_response_sent'):
            self.send_setup(event, [f'player_{role}'], 'reentry-profile')
            event['player_response_sent'] = {'monotonic_ns': time.monotonic_ns()}
            if role == 'C':
                peer = self.channels['B']
                event['replacement_roster_sent'] = self.send(peer, 'player_C', 'replacement106')
            elif role == 'B' and event.get('room_reentry'):
                peer = self.channels['A']
                if not peer.get('closed') and peer.get('map_response_sent'):
                    event['rejoined_peer_roster_sent'] = self.send(peer, 'guest_A', 'rejoin-peer106')
        elif opcode == 95 and role != self.host_role and not self.room.active and not event.get('guest_ready_received'):
            event['guest_ready_received'] = {'monotonic_ns': time.monotonic_ns(), 'payload_hex': body.hex()}
            self.broadcast('ready_C' if role == 'C' else 'ready_slot', 'ready110')
        elif opcode == 104 and role == self.host_role:
            peers = self.peers()
            if self.room.active or len(peers) != 2 or event not in peers:
                return
            if not all(peer.get('guest_ready_received') for peer in peers if peer is not event):
                return
            # Result must have returned/refreshed before another accepted Start.
            if self.round['index'] and not self.round.get('refresh'):
                return
            self.start_round()
        elif opcode == 99 and self.room.active and not event.get('init99'):
            values = codec.read_fields(body, [4, 4, 2, 4, 2, 1, 1, 1, 4])
            if not 1 <= values[2] <= 65535:
                raise ValueError('invalid UDP port')
            event['init99'] = {'fields': values, 'seq': sequence, 'monotonic_ns': time.monotonic_ns(),
                               'sha256': hashlib.sha256(frame).hexdigest()}
        elif opcode == 118 and self.room.active:
            values = codec.read_fields(body, [4, 1, 1, 4, 1, 1, 1, 1, 1, 1, 1, 4, 1])
            if values[-1] != 0 or values[2] != ROLE_SLOTS[role]:
                raise ValueError('unsupported bomb report')
            request = {'round': self.room.generation, 'role': role, 'seq': sequence,
                       'fields': values, 'monotonic_ns': time.monotonic_ns(),
                       'frame_sha256': hashlib.sha256(frame).hexdigest(),
                       'body_sha256': hashlib.sha256(body).hexdigest()}
            self.state['bomb_requests'].append(request)
            # Fixture policy, not authentication: native record+8 matches the
            # current153 key in the measured profile. Serial is scoped to the
            # bound owner and this round; it can repeat after Start.
            if values[0] != self.room.key:
                request['rejected'] = 'bomb_round_key_mismatch'
                return
            serials = event.setdefault('bomb_serials', {})
            prior = serials.get(values[3])
            if prior is not None:
                request['rejected'] = ('duplicate_bomb_serial'
                                       if prior['body_sha256'] == request['body_sha256']
                                       else 'conflicting_bomb_serial')
                request['prior_id'] = prior['id']
                return
            ticket = self.room.bomb()
            request['id'] = ticket.wire_id
            serials[values[3]] = {'generation': self.room.generation, 'id': ticket.wire_id,
                                  'body_sha256': request['body_sha256']}
            fields = [0, values[2], 0, values[3], 0, 0, values[4], values[5], values[10],
                      1, 0, ticket.wire_id, 0, 1, 0, 0, 0]
            payload = codec.pack_fields(fields, [1, 1, 2, 4, 4, 4, 1, 1, 1, 4, 1, 2, 2, 1, 8, 1, 1])
            if self.room.fence.rejection(ticket) is not None:
                return
            broadcast = self.broadcast('bomb', f'dynamic159-r{self.room.generation}-{ticket.wire_id}', body=payload)
            for sent in broadcast:
                sent['fields'] = fields
            own = next(sent for sent in broadcast if sent['role'] == role)
            source = {'id': ticket.wire_id, 'actual118': {'monotonic_ns': time.monotonic_ns(),
                       'seq': sequence, 'sha256': hashlib.sha256(frame).hexdigest(), 'fields': values}, 'reply': own}
            event.setdefault('bombs', []).append(source)
            bomb = {'round': self.room.generation, 'id': ticket.wire_id, 'owner': values[2],
                    'serial': values[3], 'source_role': role, 'source_bomb': source,
                    'created_ns': time.monotonic_ns(), 'broadcast159': broadcast}
            self.state['round_bombs'].append(bomb)
            self.jobs.append((time.monotonic() + 6, ticket, bomb))
        elif opcode == 124 and self.room.active:
            values = codec.read_fields(body, [1, 1, 1, 1, 4, 2, 4, 4, 1])
            context, slot, row, column, index, kind, client_tick, reported_key, flag = values
            if flag != 0:
                raise ValueError('unsupported item variant')
            report = {'round': self.room.generation, 'role': role, 'seq': sequence, 'fields': values,
                      'payload_hex': body.hex(), 'frame_sha256': hashlib.sha256(frame).hexdigest(),
                      # CP055: latched CMainSystem logical tick, not round/server milliseconds.
                      # Log it without inventing a timing/authentication policy.
                      'client_tick': client_tick,
                      'monotonic_ns': time.monotonic_ns(), 'broadcast165': []}
            self.state['item_requests'].append(report)
            # Emulator policy for the measured 124 variant. This is the u32 k
            # observed following the second 153 u32, not an authentication token.
            # Reject before touching the current catalog's one-winner claim.
            if reported_key != self.room.key:
                report['rejected'] = 'round_key_mismatch'
                self.state['policy_events'].append({'op': 'item_reject_key',
                                                    'generation': self.room.generation,
                                                    'reported_key': reported_key,
                                                    'expected_key': self.room.key,
                                                    'slot': slot, 'index': index})
                return
            if index in self.hidden_items and str(index) not in self.round['revealed_hidden']:
                report['rejected'] = 'hidden_item_not_reported_revealed'
                return
            # Conservative fixture policy after an accepted trap/death report.
            # This is report-derived, not native state authority/authentication.
            # This profile has no rescue/self-escape flow; a new Start clears it.
            blocked = event.get('pickup_ineligible')
            if blocked and blocked['generation'] == self.room.generation:
                report['rejected'] = 'actor_report_ineligible'
                report['actor_report'] = dict(blocked)
                self.state['policy_events'].append({'op': 'item_reject_actor',
                                                    'generation': self.room.generation,
                                                    'role': role, 'slot': slot, 'index': index,
                                                    'subtype': blocked['subtype']})
                return
            claim = self.room.claim(ROLE_SLOTS[role], slot, context, ItemSpec(index, row, column, kind))
            if claim is not None and self.room.claims.consume(claim):
                payload = codec.pack_fields([1, context, slot, row, column, index, kind, 0], [1, 1, 1, 1, 1, 4, 2, 1])
                report['broadcast165'] = self.broadcast('start', f'item165-r{self.room.generation}-{index}',
                                                         opcode=165, body=payload)
                self.state['policy_events'].append({'op': 'item_consume', 'generation': claim.generation,
                                                    'index': index, 'slot': slot})
            else:
                report['rejected'] = 'claim_not_current_or_already_consumed'
        elif opcode == 142 and self.room.active:
            row, column, selected_slot, aux = codec.read_fields(body, [1, 1, 1, 4])
            report = {'round': self.room.generation, 'role': role, 'seq': sequence,
                      'fields': [row, column, selected_slot, aux],
                      'monotonic_ns': time.monotonic_ns(),
                      'frame_sha256': hashlib.sha256(frame).hexdigest()}
            self.state['block_reports'].append(report)
            if selected_slot != ROLE_SLOTS[role] or aux != 0xFFFFFFFF:
                report['rejected'] = 'unsupported_block_report'
                return
            matches = [item for item in self.hidden_items.values()
                       if (item.row, item.column) == (row, column)]
            if not matches:
                report['rejected'] = 'unknown_hidden_cell'
                return
            # Report-derived fixture policy only. Sending any current158 is a
            # lower bound, not proof that this particular cell was destroyed.
            if not any(bomb['round'] == self.room.generation and bomb.get('broadcast158')
                       for bomb in self.state['round_bombs']):
                report['rejected'] = 'no_current_blast_sent'
                return
            for item in matches:
                prior = self.round['revealed_hidden'].setdefault(str(item.index), report)
                report.setdefault('revealed_indices' if prior is report else 'already_revealed', []).append(item.index)
        elif opcode == 127 and self.room.active:
            self.receive_event(event, frame, sequence, body)

    def receive_event(self, event, frame, sequence, body):
        if len(body) < 14:
            raise ValueError('unsupported event prefix')
        subtype, slot, x, y, client_tick, actor_field = codec.read_fields(body[:14], [1, 1, 2, 2, 4, 4])
        if slot != ROLE_SLOTS[event['role']]:
            raise ValueError('event slot does not match connection')
        report = {'role': event['role'], 'round': self.room.generation, 'subtype': subtype,
                  'slot': slot, 'seq': sequence, 'monotonic_ns': time.monotonic_ns(),
                  'payload_hex': body.hex(), 'frame_sha256': hashlib.sha256(frame).hexdigest(),
                  'client_tick': client_tick, 'actor_field': actor_field,
                  'body_sha256': hashlib.sha256(body).hexdigest()}
        self.state['event_reports'].append(report)
        if subtype in (17, 19):
            if subtype == 17:
                length = codec.read_fields(body[14:18], [4])[0]
                if length != 14 or len(body) != 32:
                    raise ValueError('unsupported trap blob')
                raw, offset = bytearray(), 0
                while offset < length:
                    width = 4 if length - offset >= 4 else 2 if length - offset >= 2 else 1
                    raw.extend(codec.read_fields(body[18 + offset:18 + offset + width], [width])[0].to_bytes(width, 'little'))
                    offset += width
                if raw[:6] != bytes.fromhex('180900000000'):
                    raise ValueError('unsupported trap envelope')
                payload = body[:6] + codec.pack_fields([int.from_bytes(raw[10:14], 'little')], [4])
            else:
                payload = body[:6] + body[14:]
        # Diagnostic policy: compare only reports on this connection. The native
        # CMainSystem counter continues through Start; do not clear this cursor
        # with round markers or compare it with another peer/124's cached tick.
        # Validate the supported trap envelope before changing the cursor.
        cursor = event.get('report127_cursor')
        if cursor is not None:
            delta = (client_tick - cursor['tick']) & 0xFFFFFFFF
            if delta >= 0x80000000:
                report['rejected'] = 'report_tick_not_forward'
                return
            if delta == 0 and report['body_sha256'] in cursor['bodies']:
                report['rejected'] = 'duplicate_report_body'
                return
        if cursor is None or client_tick != cursor['tick']:
            cursor = {'tick': client_tick, 'bodies': []}
            event['report127_cursor'] = cursor
        cursor['bodies'].append(report['body_sha256'])
        if subtype in (17, 19):
            event['pickup_ineligible'] = {'generation': self.room.generation,
                                          'subtype': subtype, 'report_sequence': sequence,
                                          'report_sha256': report['frame_sha256']}
            other = next((peer for peer in self.peers() if peer is not event), None)
            if other is None:
                return
            report['forward170'] = self.send(other, 'start', f'event170-{subtype}-r{self.room.generation}',
                                             opcode=170, body=payload)
            if subtype == 19:
                event['death_event'] = report

    def close_connection(self, ordinal, reason):
        """One owner serializes close, lifetime invalidation and later recovery."""
        state = self.connections[ordinal]
        event = state['event']
        if event.get('closed'):
            return
        event['closed'] = {'reason': reason, 'monotonic_ns': time.monotonic_ns()}
        try:
            self.selector.unregister(state['socket'])
        except KeyError:
            pass
        state['socket'].close()
        if event['reply_ordinal'] != 4:
            return
        event.pop('guest_ready_received', None)
        event.pop('pending_departures', None)
        if self.room.active:
            # Do this immediately, before rank/return delays or another deadline.
            self.room.finish()
            self.round['abort'] = {'role': event['role'], 'reason': reason,
                                   'monotonic_ns': time.monotonic_ns(),
                                   'generation': self.room.generation}
            self.state['policy_events'].append({'op': 'abort', 'role': event['role'],
                                                'generation': self.room.generation,
                                                'reason': reason})
        elif self.round.get('rank') and not self.round.get('refresh'):
            # A result already chosen remains chosen; still clear the departed
            # slot after return. Never start a second result sequence.
            self.round.setdefault('abort', {'role': event['role'], 'reason': reason,
                                           'monotonic_ns': time.monotonic_ns(),
                                           'generation': self.room.generation})
        elif event.get('create_response_sent'):
            # Join/create acceptance establishes membership before map loading.
            # A peer may already have this slot via106 when setup disconnects.
            self.cleanup_departed(event['role'])

    def deliver_departure(self, peer, departed):
        """Send after recipient setup, so fixture roster cannot restore the slot."""
        body = codec.pack_fields([ROLE_SLOTS[departed], ROLE_IDS[departed], 0], [1, 4, 1])
        sent = [{'role': peer['role'], **self.send(peer, 'start', 'remove107',
                                                 opcode=107, body=body)}]
        # This diagnostic profile supports B taking over A, or A staying chief.
        alias = 'promote_B' if peer['role'] == 'B' else ('slot_C' if peer['role'] == 'C' else 'slot_A')
        sent.append({'role': peer['role'], **self.send(peer, alias, 'survivor110')})
        peer.pop('guest_ready_received', None)
        peer.setdefault('departures_sent', []).append(departed)
        return sent

    def cleanup_departed(self, departed):
        members = [event for event in self.events if event.get('reply_ordinal') == 4
                   and not event.get('closed')
                   and (event.get('create_response_sent') or event.get('map_response_sent'))]
        if departed == self.host_role and members:
            self.host_role = members[0]['role']
        sent, deferred = [], []
        for peer in members:
            if peer.get('map_response_sent'):
                sent.extend(self.deliver_departure(peer, departed))
            else:
                pending = peer.setdefault('pending_departures', [])
                if departed not in pending:
                    pending.append(departed)
                deferred.append(peer['role'])
        self.state['policy_events'].append({'op': 'cleanup_departed', 'role': departed,
                                            'generation': self.room.generation,
                                            'broadcast': sent, 'deferred': deferred})
        return sent

    def tick(self):
        now = time.monotonic_ns()
        for due, ticket, bomb in list(self.jobs):
            if time.monotonic() < due:
                continue
            self.jobs.remove((due, ticket, bomb))
            reason = self.room.fence.rejection(ticket)
            if reason is None and self.room.fence.consume(ticket):
                body = codec.pack_fields([1, bomb['id'], 0, 2, 0xFFFFFFFF, 255], [1, 2, 1, 4, 4, 1])
                bomb['broadcast158'] = self.broadcast('blast', f"blast158-r{bomb['round']}-{bomb['id']}", body=body)
            else:
                bomb['timer_rejected'] = reason
            self.state['policy_events'].append({'op': 'bomb_deadline', 'generation': ticket.generation,
                                                'wire_id': ticket.wire_id, 'rejected': reason})
        self.tick_setup(now)
        abort = self.round.get('abort')
        if abort and not self.round.get('rank') and now - abort['monotonic_ns'] >= 12 * 10**9:
            alias = 'rank' if ROLE_SLOTS[abort['role']] == 0 else 'rank_Awin'
            self.round['rank'] = {'monotonic_ns': now,
                                  'broadcast': self.broadcast(alias, 'forfeit175'),
                                  'policy': 'forfeit on channel close, no synthetic death'}
        if self.room.active:
            peers = self.peers('init99')
            if len(peers) == 2:
                for event in peers:
                    if not event.get('go_response_sent'):
                        event['go_response_sent'] = self.send(event, 'go', 'go197')
                    elif not event.get('play_response_sent') and now - event['go_response_sent']['monotonic_ns'] >= 10**9:
                        event['play_response_sent'] = self.send(event, 'play', 'play196')
                    elif event.get('play_response_sent') and not event.get('peer153_sent') and now - event['play_response_sent']['monotonic_ns'] >= 10**9:
                        other = next(peer for peer in peers if peer['role'] != event['role'])
                        port = self.proxy_ports.get(other['role'], 0) or other['init99']['fields'][2]
                        body = bytearray(self.profile.body('peer')[:35])
                        body[15:19] = codec.pack_fields([self.room.key], [4])
                        for slot in range(26):
                            body.extend(codec.pack_fields([int(slot < 2)], [1]))
                            if slot < 2:
                                remote = slot != ROLE_SLOTS[event['role']]
                                for ip, endpoint in ([(0x7F000001, port), (0, 0), (0, 0)] if remote else [(0, 0)] * 3):
                                    body.extend(codec.pack_fields([ip, endpoint], [4, 2]))
                        event['begin_response_sent'] = self.send(event, 'peer', 'peer153', body=bytes(body))
                        event['peer153_sent'] = {**event['begin_response_sent'], 'remote_port': port}
                        self.state['policy_events'].append({'op': 'peer_key', 'role': event['role'],
                                                            'generation': self.room.generation, 'key': self.room.key})
            death = next((peer['death_event'] for peer in self.peers() if peer.get('death_event')), None)
            remote_reports = self.state['event_reports']
            if death and now - death['monotonic_ns'] >= 8 * 10**9 and any(
                row['role'] != death['role'] and row['role'] in {peer['role'] for peer in self.peers()}
                and row['round'] == self.room.generation and row['subtype'] == 21
                and not row.get('rejected')
                and row['monotonic_ns'] > death['monotonic_ns'] for row in remote_reports
            ):
                self.room.finish()
                alias = 'rank' if death['slot'] == 0 else 'rank_Awin'
                self.round['rank'] = {'monotonic_ns': now, 'broadcast': self.broadcast(alias, f"rank175-r{self.room.generation}")}
                self.state['policy_events'].append({'op': 'finish', 'generation': self.room.generation})
        if self.round.get('rank') and not self.round.get('return') and now - self.round['rank']['monotonic_ns'] >= 5 * 10**9:
            self.round['return'] = {'monotonic_ns': now, 'broadcast': self.broadcast('return', f"return180-r{self.room.generation}")}
        if self.round.get('return') and not self.round.get('refresh') and now - self.round['return']['monotonic_ns'] >= 3 * 10**9:
            if self.round.get('abort'):
                sent = self.cleanup_departed(self.round['abort']['role'])
            else:
                if self.host_role == 'B':
                    sent = self.broadcast('promote_B', f"host110-r{self.room.generation}")
                    sent += self.broadcast('slot_C', f"guest110-r{self.room.generation}")
                else:
                    sent = self.broadcast('slot_A', f"host110-r{self.room.generation}")
                    sent += self.broadcast('slot_B', f"guest110-r{self.room.generation}")
            self.round['refresh'] = {'monotonic_ns': now, 'broadcast': sent}
            for event in self.peers():
                event.pop('guest_ready_received', None)

    def tick_setup(self, now):
        for event in self.events:
            if event.get('reply_ordinal') != 4 or event.get('closed'):
                continue
            role = event['role']
            left = event.get('last_room_leave')
            if (left and not left.get('refresh')
                    and now - left['monotonic_ns'] >= 10**9):
                left['refresh'] = [self.send(event, 'ready', 'return-lobby241'),
                                   self.send(event, 'roomlist_' + role, 'return-lobby91')]
            for previous, marker, delay, aliases in [
                ('player_response_sent', 'slot_response_sent', 2, [f'slot_{role}']),
                ('slot_response_sent', 'guest_response_sent', 1, [f'guest_{role}']),
                ('guest_response_sent', 'map_response_sent', 1, ['table', 'map']),
            ]:
                if event.get(previous) and not event.get(marker) and now - event[previous]['monotonic_ns'] >= delay * 10**9:
                    other = (self.channels.get('A' if role == 'B' else 'B')
                             if event.get('room_reentry') else None)
                    if (event.get('room_reentry') and marker == 'guest_response_sent'
                            and (not other or other.get('closed')
                                 or not other.get('create_response_sent'))):
                        event[marker] = {'monotonic_ns': now, 'skipped_closed_guest': True}
                    else:
                        self.send_setup(event, aliases, 'reentry-setup')
                        event[marker] = {'monotonic_ns': now}
            if event.get('map_response_sent'):
                for departed in event.pop('pending_departures', []):
                    sent = self.deliver_departure(event, departed)
                    self.state['policy_events'].append({'op': 'deferred_cleanup', 'role': departed,
                                                        'recipient': role, 'broadcast': sent,
                                                        'monotonic_ns': time.monotonic_ns()})

    def replacement_allowed(self):
        departed, survivor = self.channels.get('A'), self.channels.get('B')
        if not (departed and departed.get('closed') and survivor and not survivor.get('closed')
                and self.host_role == 'B' and survivor.get('map_response_sent')
                and not survivor.get('pending_departures') and not self.room.active):
            return False
        before_first = self.round['index'] == 0 and 'A' in survivor.get('departures_sent', [])
        return bool(before_first or self.round.get('refresh'))

    def accept(self, listener):
        connection, peer = listener.accept()
        ordinal = len(self.events) + 1
        if ordinal > 12 or (ordinal > 8 and not self.replacement_allowed()):
            self.state.setdefault('admission_rejections', []).append({
                'ordinal': ordinal, 'peer': list(peer), 'monotonic_ns': time.monotonic_ns(),
                'reason': 'capacity' if ordinal > 12 else 'replacement_not_ready'})
            connection.close()
            return
        connection.settimeout(2)
        event = {'global_ordinal': ordinal, 'role': 'ABC'[(ordinal - 1) // 4],
                 'reply_ordinal': (ordinal - 1) % 4 + 1, 'peer': list(peer),
                 'port': listener.getsockname()[1], 'accept_monotonic_ns': time.monotonic_ns(), 'chunks': []}
        self.events.append(event)
        self.connections[ordinal] = {'socket': connection, 'event': event, 'buffer': bytearray(), 'sequence': 40}
        if event['reply_ordinal'] == 4:
            self.channels[event['role']] = event
        self.send_raw(event, self.profile.raw('bootstrap'))
        event['reply_sent'] = {'monotonic_ns': time.monotonic_ns()}
        self.selector.register(connection, selectors.EVENT_READ, ordinal)

    def receive(self, ordinal):
        state = self.connections[ordinal]
        connection, event = state['socket'], state['event']
        data = connection.recv(4096)
        if not data:
            event['peer_eof'] = True
            self.close_connection(ordinal, 'eof')
            return
        event['chunks'].append({'monotonic_ns': time.monotonic_ns(), 'hex': data.hex()})
        if event['reply_ordinal'] == 2:
            if not event.get('second_response_sent'):
                self.send_raw(event, self.profile.raw('second'))
                event['second_response_sent'] = {'monotonic_ns': time.monotonic_ns()}
            return
        if event['reply_ordinal'] < 3:
            return
        state['buffer'].extend(data)
        while (frame := codec.pop_frame(state['buffer'])) is not None:
            sequence = state['sequence']
            body = codec.frame_body(frame, sequence)
            state['sequence'] += 3
            opcode = struct.unpack_from('<I', frame, 5)[0]
            event['c2s_frame_count'] = event.get('c2s_frame_count', 0) + 1
            if event['reply_ordinal'] == 3:
                for op, marker, names in [(1, 'auth_response_sent', [f"auth_{event['role']}"]),
                                          (9, 'netinfo_response_sent', ['netinfo', 'flags']),
                                          (5, 'redirect_response_sent', ['redirect'])]:
                    if opcode == op and not event.get(marker):
                        self.send_raw(event, b''.join(self.profile.raw(name) for name in names))
                        event[marker] = {'monotonic_ns': time.monotonic_ns()}
            else:
                self.receive_channel(event, frame, sequence, body)

    def save(self):
        self.state['snapshot_monotonic_ns'] = time.monotonic_ns()
        temporary = self.out / 'service-state.tmp'
        temporary.write_text(json.dumps(self.state, indent=2) + '\n')
        temporary.replace(self.out / 'service-state.json')

    def run(self, ports):
        listeners = []
        try:
            for port in ports:
                listener = socket.socket()
                listener.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
                listener.bind(('127.0.0.1', port))
                listener.listen(8)
                listeners.append(listener)
                self.selector.register(listener, selectors.EVENT_READ, None)
            self.state['listen_ports'] = [listener.getsockname()[1] for listener in listeners]
            self.state['ready'] = True
            self.save()
            while not self.closed:
                for key, _ in self.selector.select(0.05):
                    try:
                        self.accept(key.fileobj) if key.data is None else self.receive(key.data)
                    except (OSError, ValueError, RuntimeError) as error:
                        self.state['errors'].append({'connection': key.data, 'error': str(error)})
                        if key.data is None:
                            raise
                        self.close_connection(key.data, str(error))
                self.tick()
                self.save()
        finally:
            self.room.finish()
            self.state['stopped'] = True
            self.save()
            for entry in self.connections.values():
                entry['socket'].close()
            for listener in listeners:
                listener.close()
            self.selector.close()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--profile', type=Path, required=True)
    parser.add_argument('--out', type=Path, required=True)
    parser.add_argument('--port', action='append', type=int, required=True)
    parser.add_argument('--proxy-a', type=int, default=0)
    parser.add_argument('--proxy-b', type=int, default=0)
    args = parser.parse_args()
    if any(not 0 <= port <= 65535 for port in [*args.port, args.proxy_a, args.proxy_b]):
        parser.error('invalid port')
    args.out.mkdir(parents=True, exist_ok=True)
    service = ProbeService(FixtureProfile(args.profile), args.out, {'A': args.proxy_a, 'B': args.proxy_b})
    def stop(_signal, _frame):
        service.closed = True
    signal.signal(signal.SIGTERM, stop)
    signal.signal(signal.SIGINT, stop)
    service.run(args.port)


if __name__ == '__main__':
    main()
