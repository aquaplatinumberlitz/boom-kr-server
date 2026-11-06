"""Fixture-backed KR diagnostic service, one serialized owner, two live local clients with one replacement identity.

Run as ``python3 -m server.probe_service``. Authentication/roster/map/rank bodies
are explicitly synthetic profile fixtures. Accept order assigns four connections
each for A, B, then replacement C (at most two live members). Not a public login server.
No Wine/UI automation is imported. The caller supplies network isolation.
"""

import argparse
import copy
import hashlib
import ipaddress
import json
from pathlib import Path
import selectors
import signal
import socket
import struct
import time

from server import probe_codec as codec
from server.item_claims import ItemSpec
from server.room_title import create_title, replace_listing_title
from server.room_team_mode import create_selector, replace_reply_selector
from server.chain_capacity import ClearCellChains
from server.terrain_model import Bomb
from server.terrain_profile import load_terrain
from server.probe_room import ProbeRoom


def supported_event_blob(body, subtype):
    """Decode the measured uncompressed 17/19 variant, before admission.

    CP023 and current native captures establish these bounded envelopes.
    The inner fields remain opaque; this is not a life-state validator.
    Other serialization variants require their own compatibility evidence.
    """
    size = {17: 14, 19: 11}[subtype]
    name = 'trap' if subtype == 17 else 'death'
    if len(body) != 18 + size or codec.read_fields(body[14:18], [4])[0] != size:
        raise ValueError(f'unsupported {name} blob')
    raw, offset = bytearray(), 0
    while offset < size:
        width = 4 if size - offset >= 4 else 2 if size - offset >= 2 else 1
        raw.extend(codec.read_fields(body[18 + offset:18 + offset + width], [width])[0]
                   .to_bytes(width, 'little'))
        offset += width
    if raw[:6] != b'\x18' + struct.pack('<I', size - 5) + b'\x00':
        raise ValueError(f'unsupported {name} envelope')
    return bytes(raw)


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


def network_config(bind_ip, peer_ips=None):
    """Explicit operator addresses, never inferred from untrusted init99 fields."""
    def ipv4(value, *, peer=False):
        try:
            if not isinstance(value, str):
                raise ValueError('expected an IPv4 string')
            address = ipaddress.IPv4Address(value)
        except ipaddress.AddressValueError as error:
            raise ValueError('invalid IPv4 address: ' + str(value)) from error
        if address.is_multicast or int(address) == 0xFFFFFFFF or (peer and address.is_unspecified):
            raise ValueError('unsupported IPv4 address: ' + value)
        return str(address)

    bind_ip = ipv4(bind_ip)
    if peer_ips is not None and not isinstance(peer_ips, dict):
        raise ValueError('peer addresses must map roles to IPv4 strings')
    peers = {role: '127.0.0.1' for role in ROLE_SLOTS}
    for role, address in (peer_ips or {}).items():
        if role not in ROLE_SLOTS:
            raise ValueError('unsupported peer role: ' + str(role))
        peers[role] = ipv4(address, peer=True)
    return bind_ip, peers


def peer_ip_options(options):
    peers = {}
    for option in options:
        role, separator, address = option.partition('=')
        if not separator or role in peers:
            raise ValueError('expected unique --peer-ip ROLE=IPv4: ' + option)
        peers[role] = address
    return peers


def channel_endpoint_config(ip, port):
    """Explicit channel destination; the caller owns isolation and reachability."""
    if ip is None and port is None:
        return None
    if ip is None or port is None:
        raise ValueError('channel IP and port must be supplied together')
    if type(port) is not int or not 1 <= port <= 65535:
        raise ValueError('invalid channel port')
    _, peers = network_config('127.0.0.1', {'A': ip})
    return {'ip': peers['A'], 'port': port}


def channel_redirect_frame(profile, endpoint):
    """Only CP015/146's measured S2C7 variant, preserving header/sequence/tail.

    None deliberately leaves the fixture path untouched, including legacy profiles.
    This operator choice does not recover original channel selection policy.
    """
    if endpoint is None:
        return None
    meta = profile.data.get('fixtures', {}).get('redirect', {})
    raw = profile.frames.get('redirect', b'')
    # Legacy profiles mark this raw-only fixture sequence=None. Its actual fixed
    # seq52 is established by CP015, and verified against the bytes below.
    if ('sequence' not in meta or (meta['sequence'] is not None
            and (type(meta['sequence']) is not int or meta['sequence'] != 52))
            or len(raw) != 52 or raw[0] != 0x69
            or struct.unpack_from('<I', raw, 5)[0] != 7):
        raise ValueError('unsupported channel redirect fixture')
    body = codec.frame_body(raw, 52)
    if (len(body) != 14 or codec.read_fields(body[:6], [1, 1, 4]) != [1, 10, 0]
            or body[12:] != codec.pack_fields([0], [2])):
        raise ValueError('unsupported channel redirect variant')
    replacement = codec.pack_fields([int(ipaddress.IPv4Address(endpoint['ip'])),
                                     endpoint['port']], [4, 2])
    return codec.encode(raw[:37], 7, body[:6] + replacement + body[12:], 52)


HOST_BACK_ALIASES = ('join_A', 'player_A_guest', 'slot_A_guest', 'guest_B_chief', 'ready_A')
# Bounded CP036 diagnostic roster: A/B team0, shared spawn0; synthetic
# slot2/ID3/team1 has no socket. Hash bodies rather than transport sequences.
RESCUE_PROFILE_BODIES = {
    'start': 'd072338979d92b47bdb5ace25660727f070ed45abc35ec18d6e186ea60df9046',
    'synthetic_opponent': '4351dfd44debaeea207515a367f1bb77237b75fc427df5c95e2e44eed72cbbb0',
    'player_B': '1317d6479b63ffcd8f1f57b46c8a145b9925eb8832405209e628ec47ae9ee8d1',
    'guest_A': '1317d6479b63ffcd8f1f57b46c8a145b9925eb8832405209e628ec47ae9ee8d1',
    'slot_B': '08e98098610ddc0ea0c455e3f8108f3b2517867328bebf795351c42b8151f229',
    'ready_slot': '0dda561bec566d5964c3c443bffa58dfe0d767a5d335fc9d0b7066b822401775',
}


class ProbeService:
    def __init__(self, profile: FixtureProfile, out: Path, proxy_ports: dict, *,
                 bind_ip='127.0.0.1', peer_ips=None, channel_ip=None, channel_port=None):
        self.bind_ip, self.peer_ips = network_config(bind_ip, peer_ips)
        self.channel_endpoint = channel_endpoint_config(channel_ip, channel_port)
        self.redirect_frame = channel_redirect_frame(profile, self.channel_endpoint)
        missing = {'rank_Awin', 'promote_B', 'auth_C', 'roomlist_C', 'join_C',
                   'player_C', 'slot_C', 'guest_C', 'ready_C'} - profile.frames.keys()
        if missing:
            raise ValueError('profile missing lifecycle fixtures: ' + ', '.join(sorted(missing)))
        self.profile, self.out, self.proxy_ports = profile, out, proxy_ports
        self.room_title_mode = False
        self.room_title_bytes = None
        if 'room_title' in profile.data:
            if profile.data['room_title'] != {'mode': 'create81_to_row91', 'max_bytes': 32}:
                raise ValueError('unsupported room_title policy')
            for alias in ('roomlist_A', 'roomlist_B'):
                replace_listing_title(profile.body(alias), b'KRRoom')
            self.room_title_mode = True
        self.room_team_mode = False
        self.room_team_selector = None
        if 'room_team_mode' in profile.data:
            if profile.data['room_team_mode'] != {'mode': 'create81_to_86_87'}:
                raise ValueError('unsupported room_team_mode policy')
            for alias, opcode in (('create', 86), ('join', 87), ('return', 180)):
                raw = profile.raw(alias)
                if struct.unpack_from('<I', raw, 5)[0] != opcode:
                    raise ValueError('unsupported room team fixture opcode')
                replace_reply_selector(profile.body(alias), opcode, 0)
            self.room_team_mode = True
        self.loading_timeout_ns = None
        if 'loading_timeout' in profile.data:
            policy = profile.data['loading_timeout']
            if (not isinstance(policy, dict) or set(policy) != {'mode', 'seconds'}
                    or policy['mode'] != 'close_missing_init99'
                    or type(policy['seconds']) is not int or not 1 <= policy['seconds'] <= 300):
                raise ValueError('unsupported loading_timeout policy')
            self.loading_timeout_ns = policy['seconds'] * 10**9
        self.battle_exit = False
        self.battle_exit_mode = None
        if 'battle_exit' in profile.data:
            if (profile.data['battle_exit'] not in (
                    {'mode': 'guest_B_202_86'}, {'mode': 'guest_B_pending_202_86'},
                    {'mode': 'host_A_202_86'})
                    or 'rescue_contact' in profile.data):
                raise ValueError('unsupported battle_exit policy')
            self.battle_exit = True
            self.battle_exit_mode = profile.data['battle_exit']['mode']
        self.death_lifecycle = False
        if 'death_lifecycle' in profile.data:
            if profile.data['death_lifecycle'] != {'mode': 'current_trap'}:
                raise ValueError('unsupported death_lifecycle policy')
            self.death_lifecycle = True
        self.rescue_contact = False
        if 'rescue_contact' in profile.data:
            if (profile.data['rescue_contact'] != {'mode': 'friendly_AB_synthetic2_aux1'}
                    or 'host_back_rejoin' in profile.data or profile.data['initial_key'] != 12345):
                raise ValueError('unsupported rescue_contact policy')
            for alias, digest in RESCUE_PROFILE_BODIES.items():
                if (alias not in profile.frames
                        or hashlib.sha256(profile.body(alias)).hexdigest() != digest):
                    raise ValueError('unsupported rescue_contact fixture: ' + alias)
            self.rescue_contact = True
        self.needle_escape = False
        if 'needle_escape' in profile.data:
            if profile.data['needle_escape'] != {'mode': 'needle_145_report_pair'}:
                raise ValueError('unsupported needle_escape policy')
            items = profile.data['items'] + profile.data.get('hidden_items', [])
            if sum(item['kind'] == 145 for item in items) != 1:
                raise ValueError('needle_escape requires exactly one catalog needle145')
            self.needle_escape = True
        self.host_back_rejoin = False
        if 'host_back_rejoin' in profile.data:
            if profile.data['host_back_rejoin'] != {'mode': 'idle_A_to_B'}:
                raise ValueError('unsupported host_back_rejoin policy')
            missing = set(HOST_BACK_ALIASES) - profile.frames.keys()
            if missing:
                raise ValueError('profile missing host Back fixtures: ' + ', '.join(sorted(missing)))
            # Dynamic sequences and captured bodies must be available before any membership mutation.
            for alias in HOST_BACK_ALIASES + ('ready', 'roomlist_A', 'table', 'map'):
                profile.body(alias)
            self.host_back_rejoin = True
        if self.battle_exit_mode == 'host_A_202_86' and not self.host_back_rejoin:
            raise ValueError('host Battle exit requires host_back_rejoin fixtures/policy')
        self.result_freshness = False
        self.result_drain = False
        if 'result_freshness' in profile.data:
            policy = profile.data['result_freshness']
            if policy not in ({'mode': 'post_blast_untrapped'},
                              {'mode': 'post_blast_untrapped_drained'}):
                raise ValueError('unsupported result_freshness policy')
            self.result_freshness = True
            self.result_drain = policy['mode'] == 'post_blast_untrapped_drained'
        # Optional measured-profile policy, not stats inferred from a request.
        self.capacity_profile = {}
        if 'bomb_capacity' in profile.data:
            limits = profile.data['bomb_capacity']
            if (not isinstance(limits, dict) or not {'A', 'B'} <= limits.keys()
                    or limits.keys() - ROLE_SLOTS.keys()):
                raise ValueError('bomb_capacity must declare A/B and only supported roles')
            for role, spec in limits.items():
                if (not isinstance(spec, dict) or set(spec) != {'initial', 'maximum'}
                        or any(type(spec[k]) is not int for k in spec)
                        or not 1 <= spec['initial'] <= spec['maximum'] <= 255):
                    raise ValueError('invalid bomb_capacity limits for ' + role)
                self.capacity_profile[role] = dict(spec)
        self.capacity_chains = None
        if 'capacity_chain' in profile.data:
            if not self.capacity_profile:
                raise ValueError('capacity_chain requires bomb_capacity')
            self.capacity_chains = ClearCellChains(profile.data['capacity_chain'], profile.frames)
        self.terrain = None
        if 'bomb_terrain' in profile.data:
            if set(self.capacity_profile) != {'A', 'B'} or self.capacity_chains:
                raise ValueError('bomb_terrain requires capacity and excludes capacity_chain')
            self.terrain = load_terrain(profile.data['bomb_terrain'], profile.frames)
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
                      'network': {'bind_ip': self.bind_ip, 'peer_ips': dict(self.peer_ips),
                                  'channel_endpoint': self.channel_endpoint,
                                  'source': 'design', 'policy': 'explicit_operator_ipv4'},
                      'limits': 'Fixture auth/roster/rank, explicit addresses default loopback, two live members/A-B then B-C by accept order; no fullmatch gate.'}
        self.state['policy_sources'] = []
        for filename in ['probe_service.py', 'probe_codec.py', 'probe_room.py', 'round_fence.py',
                         'item_claims.py', 'udp_round_keys.py', 'chain_capacity.py',
                         'terrain_model.py', 'terrain_profile.py', 'room_title.py', 'room_team_mode.py']:
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
            if (getattr(self, 'room_title_mode', False) and name == 'roomlist_B'
                    and self.room_title_bytes is not None):
                body = replace_listing_title(body, self.room_title_bytes)
            if (getattr(self, 'room_team_mode', False) and name == 'return'
                    and self.room_team_selector is not None):
                body = replace_reply_selector(body, 180, self.room_team_selector)
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
        if self.rescue_contact and (self.host_role != 'A'
                or {peer['role'] for peer in self.peers()} != {'A', 'B'}
                or any(not peer.get('synthetic_opponent_sent') for peer in self.peers())):
            raise ValueError('rescue_contact requires live mapped A/B and A chief')
        if self.needle_escape and {peer['role'] for peer in self.peers()} != {'A', 'B'}:
            raise ValueError('needle_escape requires live mapped A/B')
        if self.capacity_profile and any(peer['role'] not in self.capacity_profile
                                         for peer in self.peers()):
            raise ValueError('bomb_capacity does not cover the current roster')
        key = self.room.begin()
        if self.round['index']:
            self.state['round_history'].append(self.round)
        self.round = {'index': self.room.generation, 'key': key, 'host_role': self.host_role,
                      'revealed_hidden': {},
                      'roster': [{'role': e['role'], 'slot': ROLE_SLOTS[e['role']],
                                  'identity': ROLE_IDS[e['role']]} for e in self.peers()]}
        self.state['current_round'] = self.round
        if self.capacity_profile:
            self.round['bomb_capacity'] = {role: spec['initial']
                                           for role, spec in self.capacity_profile.items()}
        if self.terrain:
            self.terrain.begin()
            self.round['terrain_seed'] = list(self.terrain.flags)
        self.state['policy_events'].append({'op': 'begin', 'generation': self.room.generation,
                                            'key': key, 'monotonic_ns': time.monotonic_ns()})
        for event in self.peers():
            markers = ['start_response_sent', 'init99', 'go_response_sent', 'play_response_sent',
                       'begin_response_sent', 'peer153_sent', 'death_event', 'pickup_ineligible',
                       'bomb_serials', 'result21_forward', 'needle_inventory', 'needle_pending',
                       'battle_exit202']
            if event.get('start_response_sent'):
                event.setdefault('rounds', []).append({key: event[key] for key in markers if key in event})
            for marker in markers:
                event.pop(marker, None)
            event['floor582_sent'] = self.send(event, 'floor', 'floor582')
            event['start_response_sent'] = {**self.send(event, 'start', f"start121-r{self.round['index']}"),
                                             'round': self.round['index'], 'after_c2s_opcode': 104,
                                             'host_role': self.host_role, 'broadcast': True}
        if self.loading_timeout_ns is not None:
            now = time.monotonic_ns()
            self.round['loading'] = {
                'generation': self.room.generation, 'started_ns': now,
                'deadline_ns': now + self.loading_timeout_ns,
                'members': [{'role': e['role'], 'connection': e['global_ordinal']}
                            for e in self.peers()]}

    def tick_loading(self, now):
        """Opt-in local deadline, not a native load failure or an original server rule.

        Pin the starting channels, never a replacement. Current init99 wins when
        already dispatched before this owner tick. Reuse close/abort/return rather
        than inventing init99, death, a cancellation packet or a match result.
        """
        loading = self.round.get('loading')
        if (not loading or loading.get('resolved_ns') is not None
                or not self.room.active or loading['generation'] != self.room.generation):
            return
        members = [self.channels.get(row['role']) for row in loading['members']]
        if any(event is None or event.get('closed')
               or event['global_ordinal'] != row['connection']
               or not event.get('map_response_sent')
               for row, event in zip(loading['members'], members)):
            loading.update(resolved_ns=now, resolution='membership_changed')
            return
        missing = [event for event in members if not event.get('init99')]
        if not missing:
            loading.update(resolved_ns=now, resolution='barrier_complete')
        elif now >= loading['deadline_ns']:
            loading.update(resolved_ns=now, resolution='deadline',
                           closed_connections=[e['global_ordinal'] for e in missing])
            self.state['policy_events'].append({
                'op': 'loading_timeout', 'generation': self.room.generation,
                'monotonic_ns': now, 'deadline_ns': loading['deadline_ns'],
                'connections': loading['closed_connections'], 'source': 'emu_choice'})
            for event in missing:
                self.close_connection(event['global_ordinal'], 'loading_timeout')

    def leave_room(self, event, frame, sequence, body):
        """Bounded fixture leave, with opt-in idle A-to-B promotion/rejoin.

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
        idle_host = (self.host_back_rejoin and event['role'] == 'A' and self.host_role == 'A'
                     and len(others) == 1 and others[0]['role'] == 'B'
                     and others[0].get('map_response_sent') and event.get('map_response_sent'))
        exit_report = event.get('battle_exit202')
        active_guest = (self.battle_exit and self.room.active and idle_guest
                        and self.channels.get('B') is event
                        and event.get('map_response_sent') and exit_report
                        and exit_report['generation'] == self.room.generation
                        and sequence > exit_report['seq']
                        and self.battle_exit_scope_rejection() is None)
        active_host = (self.battle_exit_mode == 'host_A_202_86' and self.room.active
                       and idle_host and self.channels.get('A') is event
                       and exit_report and exit_report['generation'] == self.room.generation
                       and sequence > exit_report['seq']
                       and self.battle_exit_scope_rejection() is None)
        active_exit = active_guest or active_host
        if body:
            request['rejected'] = 'unsupported_leave_body'
        elif not event.get('create_response_sent'):
            request['rejected'] = 'not_room_member'
        elif not active_exit and (self.room.active or (self.round.get('index') and not self.round.get('refresh'))):
            request['rejected'] = 'round_not_returned'
        elif not (empty_host or idle_guest or idle_host):
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
                if idle_host:
                    aliases = ['ready', 'roomlist_B', 'table', 'map', 'promote_B', *HOST_BACK_ALIASES]
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
                       'pickup_ineligible', 'bomb_serials', 'pending_departures', 'departures_sent',
                       'needle_inventory', 'needle_pending', 'battle_exit202']
            request['previous_membership'] = {key: event.pop(key) for key in markers if key in event}
            event.setdefault('room_visits', []).append(request)
            event['last_room_leave'] = request
            event['room_reentry'] = True
            if active_exit:
                self.room.finish()
                self.round['abort'] = {'role': event['role'], 'reason': 'normal_Battle_exit',
                                       'generation': self.room.generation,
                                       'monotonic_ns': now}
                request['variant'] = 'active_host_A' if active_host else 'active_guest_B'
                request['report202_sha256'] = exit_report['frame_sha256']
                if active_host:
                    request['survivor_connection'] = others[0]['global_ordinal']
                # The peer is still in Battle. Reuse measured post-return107/
                # 110 cleanup rather than sending a prepare-only removal now.
            elif idle_guest:
                request['departure'] = self.cleanup_departed('B')
            elif idle_host:
                request['variant'] = 'host_A_to_B'
                request['survivor_connection'] = others[0]['global_ordinal']
                request['departure'] = self.cleanup_departed('A')
            self.state['policy_events'].append({'op': 'room_leave', 'role': event['role'],
                                               'generation': self.room.generation,
                                               'visit': len(event['room_visits']),
                                               'monotonic_ns': now})

    def send_setup(self, event, aliases, label, body_overrides=None):
        body_overrides = body_overrides or {}
        if event.get('room_reentry'):
            return [self.send(event, alias, label + '-' + alias, body=body_overrides.get(alias))
                    for alias in aliases]
        frames = []
        for alias in aliases:
            raw = self.profile.raw(alias)
            if alias in body_overrides:
                raw = codec.encode(raw[:37], struct.unpack_from('<I', raw, 5)[0],
                                   body_overrides[alias], self.profile.data['fixtures'][alias]['sequence'])
            frames.append(raw)
        self.send_raw(event, b''.join(frames))
        for alias, raw in zip(aliases, frames):
            if alias in body_overrides:
                name = f"{event['role']}-setup-{alias}-{event['global_ordinal']}.bin"
                (self.out / name).write_bytes(raw)
                event.setdefault('setup_body_overrides', []).append({
                    'alias': alias, 'fixture': name,
                    'sha256': hashlib.sha256(raw).hexdigest(),
                    'sequence': self.profile.data['fixtures'][alias]['sequence'],
                    'monotonic_ns': time.monotonic_ns()})
        return []

    def receive_channel(self, event, frame, sequence, body):
        opcode = struct.unpack_from('<I', frame, 5)[0]
        role = event['role']
        if event.get('closed'):
            return
        if opcode == 86:
            self.leave_room(event, frame, sequence, body)
        elif opcode == 202 and self.battle_exit:
            self.receive_battle_exit(event, frame, sequence, body)
        elif opcode == 6 and not event.get('channel_response_sent'):
            self.send_raw(event, self.profile.raw('channel'))
            event['channel_response_sent'] = {'monotonic_ns': time.monotonic_ns()}
        elif opcode == 67 and not event.get('ready_response_sent'):
            listing = self.profile.raw(f'roomlist_{role}')
            if (getattr(self, 'room_title_mode', False) and role == 'B'
                    and self.room_title_bytes is not None):
                sequence91 = self.profile.data['fixtures']['roomlist_B']['sequence']
                listing = codec.encode(listing[:37], 91, replace_listing_title(
                    self.profile.body('roomlist_B'), self.room_title_bytes), sequence91)
            self.send_raw(event, self.profile.raw('ready') + listing
                          + self.profile.raw('channel_list'))
            event['ready_response_sent'] = {'monotonic_ns': time.monotonic_ns()}
            if getattr(self, 'room_title_mode', False) and role == 'B':
                path = f"B-title-initial91-{event['global_ordinal']}.bin"
                (self.out / path).write_bytes(listing)
                event['initial_roomlist_title'] = {
                    'fixture': path, 'sha256': hashlib.sha256(listing).hexdigest(),
                    'monotonic_ns': time.monotonic_ns(),
                    'title_sha256': hashlib.sha256(self.room_title_bytes).hexdigest()
                    if self.room_title_bytes is not None else None}
        elif opcode in (81, 82) and not event.get('create_response_sent'):
            previous = event.get('last_room_leave')
            promoted = (self.host_back_rejoin and role == 'A' and previous
                        and previous.get('variant') in ('host_A_to_B', 'active_host_A'))
            if promoted and opcode != 82:
                self.state.setdefault('room_reentry_rejections', []).append({
                    'role': role, 'seq': sequence, 'reason': 'promoted_room_requires_join',
                    'frame_sha256': hashlib.sha256(frame).hexdigest()})
                return
            if (role, opcode) not in [('A', 81), ('B', 82), ('C', 82)] and not promoted:
                raise ValueError('unsupported fixture role/join')
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
            if promoted:
                host = self.channels.get('B')
                if (self.host_role != 'B' or not host or host.get('closed')
                        or not host.get('map_response_sent')
                        or host['global_ordinal'] != previous['survivor_connection']):
                    self.state.setdefault('room_reentry_rejections', []).append({
                        'role': role, 'seq': sequence, 'reason': 'host_membership_changed',
                        'frame_sha256': hashlib.sha256(frame).hexdigest()})
                    return
            alias = 'join_A' if promoted else {'A': 'create', 'B': 'join', 'C': 'join_C'}[role]
            pending_title = None
            if getattr(self, 'room_title_mode', False) and role == 'A' and opcode == 81:
                try:
                    pending_title = create_title(body)
                except ValueError as error:
                    self.state.setdefault('room_title_rejections', []).append({
                        'role': role, 'seq': sequence, 'reason': str(error),
                        'frame_sha256': hashlib.sha256(frame).hexdigest()})
                    return
            pending_selector = None
            reply_bodies = {}
            if getattr(self, 'room_team_mode', False):
                try:
                    if alias not in ('create', 'join'):
                        raise ValueError('room team mode supports current A/B membership only')
                    if alias == 'create' and opcode == 81:
                        pending_selector = create_selector(body)
                        selector = pending_selector
                    elif alias == 'join' and self.room_team_selector is not None:
                        selector = self.room_team_selector
                    else:
                        raise ValueError('room team mode requires current create81')
                    reply_bodies[alias] = replace_reply_selector(
                        self.profile.body(alias), 86 if alias == 'create' else 87, selector)
                except ValueError as error:
                    self.state.setdefault('room_team_rejections', []).append({
                        'role': role, 'seq': sequence, 'reason': str(error),
                        'frame_sha256': hashlib.sha256(frame).hexdigest()})
                    return
            sent = self.send_setup(event, [alias], 'reentry-create', body_overrides=reply_bodies)
            if pending_selector is not None:
                self.room_team_selector = pending_selector
                self.state.setdefault('room_team_updates', []).append({
                    'role': role, 'seq': sequence, 'selector': pending_selector,
                    'frame_sha256': hashlib.sha256(frame).hexdigest(),
                    'monotonic_ns': time.monotonic_ns(), 'source': 'emu_choice',
                    'limits': 'Known A/B create81 to success86/87; not original authority/general room variants.'})
            if reply_bodies:
                self.state.setdefault('room_team_replies', []).append({
                    'role': role, 'alias': alias, 'selector': selector,
                    'body_sha256': hashlib.sha256(reply_bodies[alias]).hexdigest(),
                    'monotonic_ns': time.monotonic_ns()})
            if pending_title is not None:
                self.room_title_bytes = pending_title
                self.state.setdefault('room_title_updates', []).append({
                    'role': role, 'seq': sequence, 'source': 'emu_choice',
                    'frame_sha256': hashlib.sha256(frame).hexdigest(),
                    'title_cp949': pending_title.decode('cp949'),
                    'title_sha256': hashlib.sha256(pending_title).hexdigest(),
                    'monotonic_ns': time.monotonic_ns(),
                    'limits': 'Known A/B title display only; opaque fields/password/full room authority unsupported.'})
            if promoted:
                event['promoted_rejoin'] = True
            event['create_response_sent'] = {'monotonic_ns': time.monotonic_ns(),
                                             'after_c2s_opcode': opcode}
            if previous:
                previous['recreate'] = {'seq': sequence, 'frame_sha256': hashlib.sha256(frame).hexdigest(),
                                        'monotonic_ns': time.monotonic_ns(), 'reply': sent}

        elif opcode == 585 and event.get('create_response_sent') and not event.get('player_response_sent'):
            alias = 'player_A_guest' if event.get('promoted_rejoin') else f'player_{role}'
            self.send_setup(event, [alias], 'reentry-profile')
            event['player_response_sent'] = {'monotonic_ns': time.monotonic_ns()}
            if role == 'C':
                peer = self.channels['B']
                event['replacement_roster_sent'] = self.send(peer, 'player_C', 'replacement106')
            elif role == 'B' and event.get('room_reentry'):
                peer = self.channels['A']
                if not peer.get('closed') and peer.get('map_response_sent'):
                    event['rejoined_peer_roster_sent'] = self.send(peer, 'guest_A', 'rejoin-peer106')
            elif role == 'A' and event.get('promoted_rejoin'):
                peer = self.channels['B']
                if not peer.get('closed') and peer.get('map_response_sent'):
                    event['rejoined_peer_roster_sent'] = self.send(peer, 'player_A_guest', 'rejoin-peer106')
        elif opcode == 95 and role != self.host_role and not self.room.active and not event.get('guest_ready_received'):
            if role == 'A' and (not event.get('promoted_rejoin') or not event.get('map_response_sent')):
                self.state.setdefault('ready_rejections', []).append({
                    'role': role, 'seq': sequence, 'reason': 'not_rejoined_member'})
                return
            event['guest_ready_received'] = {'monotonic_ns': time.monotonic_ns(), 'payload_hex': body.hex()}
            self.broadcast('ready_A' if role == 'A' else 'ready_C' if role == 'C' else 'ready_slot', 'ready110')
        elif opcode == 104 and role == self.host_role:
            peers = self.peers()
            if self.room.active or len(peers) != 2 or event not in peers:
                return
            if not all(peer.get('guest_ready_received') for peer in peers if peer is not event):
                return
            # Result must have returned/refreshed before another accepted Start.
            if self.round['index'] and not self.round.get('refresh'):
                return
            if self.capacity_profile and any(peer['role'] not in self.capacity_profile for peer in peers):
                self.state.setdefault('start_rejections', []).append({
                    'reason': 'bomb_capacity_roster_not_configured',
                    'roles': [peer['role'] for peer in peers], 'seq': sequence,
                    'monotonic_ns': time.monotonic_ns()})
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
            terrain_after_place = None
            if self.terrain:
                if [values[i] for i in (1, 6, 7, 8, 9, 11, 12)] != [0, 1, 0, 0, 0, 1, 0]:
                    request['rejected'] = 'bomb_terrain_variant'
                    return
                # Preview geometry without taking an ID, serial or reservation.
                # This serialized owner is the sole allocator between preview
                # and commit; a successful159 is still required below.
                terrain_after_place = copy.deepcopy(self.terrain)
                try:
                    terrain_after_place.place(Bomb(self.room._wire_id + 1, values[2],
                                                  values[4], values[5], values[10]))
                except ValueError:
                    request['rejected'] = 'bomb_terrain_placement'
                    return
            if self.capacity_profile:
                # Default: own job lifetime. Optional pinned clear-cell policy
                # can release direct secondary reservations after a root blast;
                # neither policy pretends to observe the native client's pool.
                reserved = [bomb['id'] for _, pending, bomb in self.jobs
                            if bomb['round'] == self.room.generation
                            and bomb['owner'] == ROLE_SLOTS[role]
                            and not bomb.get('capacity_release')
                            and self.room.fence.rejection(pending) is None]
                capacity = self.round['bomb_capacity'][role]
                request['capacity_guard'] = {'capacity': capacity, 'reserved_ids': reserved,
                                             'source': 'design'}
                if len(reserved) >= capacity:
                    request['rejected'] = 'bomb_capacity_reserved'
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
            if terrain_after_place:
                self.terrain = terrain_after_place
            for sent in broadcast:
                sent['fields'] = fields
            own = next(sent for sent in broadcast if sent['role'] == role)
            source = {'id': ticket.wire_id, 'actual118': {'monotonic_ns': time.monotonic_ns(),
                       'seq': sequence, 'sha256': hashlib.sha256(frame).hexdigest(), 'fields': values}, 'reply': own}
            event.setdefault('bombs', []).append(source)
            bomb = {'round': self.room.generation, 'id': ticket.wire_id, 'owner': values[2],
                    'serial': values[3], 'source_role': role, 'source_bomb': source,
                    'row': values[4], 'column': values[5], 'power': values[10],
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
            # Start clears this marker. Only the opt-in needle pair below can
            # clear a matching trap; report21 and rescue are separate paths.
            blocked = event.get('pickup_ineligible')
            if blocked and blocked['generation'] == self.room.generation:
                report['rejected'] = 'actor_report_ineligible'
                report['actor_report'] = dict(blocked)
                self.state['policy_events'].append({'op': 'item_reject_actor',
                                                    'generation': self.room.generation,
                                                    'role': role, 'slot': slot, 'index': index,
                                                    'subtype': blocked['subtype']})
                return
            if getattr(self, 'needle_escape', False) and kind == 145:
                peers = self.peers()
                if (len(peers) != 2 or {peer['role'] for peer in peers} != {'A', 'B'}
                        or not any(peer is event for peer in peers)):
                    report['rejected'] = 'needle_not_current_AB_member'
                    return
            claim = self.room.claim(ROLE_SLOTS[role], slot, context, ItemSpec(index, row, column, kind))
            if claim is not None and self.room.claims.consume(claim):
                payload = codec.pack_fields([1, context, slot, row, column, index, kind, 0], [1, 1, 1, 1, 1, 4, 2, 1])
                report['broadcast165'] = self.broadcast('start', f'item165-r{self.room.generation}-{index}',
                                                         opcode=165, body=payload)
                self.state['policy_events'].append({'op': 'item_consume', 'generation': claim.generation,
                                                    'index': index, 'slot': slot})
                if (getattr(self, 'needle_escape', False) and kind == 145
                        and len(report['broadcast165']) == 2):
                    event['needle_inventory'] = {'generation': claim.generation, 'count': 1,
                                                 'pickup_sha256': report['frame_sha256']}
                    self.state['policy_events'].append({
                        'op': 'needle_credit', 'source': 'design', 'generation': claim.generation,
                        'role': role, 'count': 1, 'pickup_sha256': report['frame_sha256']})
                if self.capacity_profile and kind == 0:
                    before = self.round['bomb_capacity'][role]
                    after = min(before + 1, self.capacity_profile[role]['maximum'])
                    self.round['bomb_capacity'][role] = after
                    self.state['policy_events'].append({
                        'op': 'bomb_capacity_pickup', 'source': 'design',
                        'generation': claim.generation, 'role': role, 'index': index,
                        'before': before, 'after': after})
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
        elif opcode == 127:
            if self.room.active:
                self.receive_event(event, frame, sequence, body)
            elif self.result_freshness:
                # Transport observation only: do not run admission, advance the
                # report cursor, forward170, or change a closed round's outcome.
                self.state.setdefault('inactive_reports', []).append({
                    'role': role, 'generation': self.room.generation, 'seq': sequence,
                    'monotonic_ns': time.monotonic_ns(), 'reason': 'round_not_active',
                    'frame_sha256': hashlib.sha256(frame).hexdigest(),
                    'body_sha256': hashlib.sha256(body).hexdigest(),
                    'payload_hex': body.hex(),
                    'prefix': (codec.read_fields(body[:14], [1, 1, 2, 2, 4, 4])
                               if len(body) >= 14 else None),
                    'limits': 'Unadmitted transport envelope; no semantic validation.'})

    def receive_event(self, event, frame, sequence, body):
        if len(body) < 14:
            raise ValueError('unsupported event prefix')
        subtype, slot, x, y, client_tick, actor_field = codec.read_fields(body[:14], [1, 1, 2, 2, 4, 4])
        if slot != ROLE_SLOTS[event['role']]:
            raise ValueError('event slot does not match connection')
        if subtype in (17, 19):
            raw = supported_event_blob(body, subtype)
            payload = (body[:6] + codec.pack_fields([int.from_bytes(raw[10:14], 'little')], [4])
                       if subtype == 17 else body[:6] + body[14:])
        report = {'role': event['role'], 'round': self.room.generation, 'subtype': subtype,
                  'slot': slot, 'seq': sequence, 'monotonic_ns': time.monotonic_ns(),
                  'payload_hex': body.hex(), 'frame_sha256': hashlib.sha256(frame).hexdigest(),
                  'client_tick': client_tick, 'actor_field': actor_field,
                  'body_sha256': hashlib.sha256(body).hexdigest()}
        self.state['event_reports'].append(report)
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
        if self.death_lifecycle and subtype == 19:
            reason = self.death_lifecycle_rejection(event)
            if reason:
                report['rejected'] = reason
                return
            report['death_lifecycle'] = {
                'source': 'design', 'generation': self.room.generation,
                'trap_sha256': event['pickup_ineligible']['report_sha256']}
        needle = None
        rescue = None
        if getattr(self, 'rescue_contact', False) and subtype == 18:
            rescue, reason = self.prepare_rescue_contact(event, report, body)
            if reason:
                report['rejected'] = reason
                return
            other = next(peer for peer in self.peers() if peer is not event)
            report['forward170'] = self.send(
                other, 'start', f'rescue170-18-r{self.room.generation}',
                opcode=170, body=body[:6] + codec.pack_fields([rescue['aux']], [1]))
        if getattr(self, 'needle_escape', False) and subtype in (0, 1):
            needle, reason = self.prepare_needle_escape(event, report, body)
            if reason:
                report['rejected'] = reason
                return
            if subtype == 0:
                # Delivery must succeed before consuming inventory, clearing
                # the trap, or committing this report's freshness cursor.
                other = next(peer for peer in self.peers() if peer is not event)
                report['forward170'] = self.send(
                    other, 'start', f'needle170-1-r{self.room.generation}',
                    opcode=170, body=bytes.fromhex(needle['prefix_hex']))
        if cursor is None or client_tick != cursor['tick']:
            cursor = {'tick': client_tick, 'bodies': []}
            event['report127_cursor'] = cursor
        cursor['bodies'].append(report['body_sha256'])
        if rescue is not None:
            event.pop('pickup_ineligible')
            event.pop('needle_pending', None)
            report['rescue_cleared'] = rescue
            self.state['policy_events'].append({
                'op': 'rescue_contact', 'source': 'design', 'role': event['role'],
                'generation': self.room.generation, **rescue,
                'report_sha256': report['frame_sha256']})
        if needle is not None:
            if subtype == 1:
                event['needle_pending'] = needle
                report['needle_staged'] = dict(needle)
            else:
                event['needle_inventory']['count'] = 0
                event.pop('pickup_ineligible')
                event.pop('needle_pending')
                report['needle_consumed'] = {'generation': self.room.generation,
                                             'report1_sha256': needle['report1_sha256'],
                                             'trap_sha256': needle['trap_sha256'],
                                             'before': 1, 'after': 0}
                self.state['policy_events'].append({
                    'op': 'needle_escape', 'source': 'design', 'role': event['role'],
                    **report['needle_consumed'], 'report0_sha256': report['frame_sha256']})
        if subtype == 21:
            self.forward_result21(event, report, body)
        if subtype in (17, 19):
            event.pop('needle_pending', None)
            event['pickup_ineligible'] = {'generation': self.room.generation,
                                          'subtype': subtype, 'report_sequence': sequence,
                                          'monotonic_ns': report['monotonic_ns'],
                                          'report_sha256': report['frame_sha256']}
            other = next((peer for peer in self.peers() if peer is not event), None)
            if other is None:
                return
            report['forward170'] = self.send(other, 'start', f'event170-{subtype}-r{self.room.generation}',
                                             opcode=170, body=payload)
            if subtype == 19:
                event['death_event'] = report

    def receive_battle_exit(self, event, frame, sequence, body):
        """Stage an opt-in role-specific202 -> 86 pair, not death or auth."""
        report = {'role': event['role'], 'generation': self.room.generation,
                  'seq': sequence, 'frame_sha256': hashlib.sha256(frame).hexdigest(),
                  'monotonic_ns': time.monotonic_ns(), 'body_hex': body.hex()}
        self.state.setdefault('battle_exit_reports', []).append(report)
        if len(body) != 22 or codec.read_fields(body[:2], [1, 1]) != [0, 1]:
            report['rejected'] = 'unsupported_battle_exit_body'
        elif (not self.room.active or self.host_role != 'A'
              or event['role'] != ('A' if self.battle_exit_mode == 'host_A_202_86' else 'B')
              or self.channels.get(event['role']) is not event
              or not any(peer is event for peer in self.peers())
              or {p['role'] for p in self.peers()} != {'A', 'B'}):
            report['rejected'] = 'unsupported_battle_exit_membership'
        elif self.battle_exit_scope_rejection() is not None:
            report['rejected'] = self.battle_exit_scope_rejection()
        elif event.get('battle_exit202'):
            report['rejected'] = 'battle_exit_already_staged'
        else:
            report['opaque_fields'] = codec.read_fields(body[2:], [4] * 5)
            event['battle_exit202'] = report

    def battle_exit_scope_rejection(self):
        """Only the measured unbombed case or one live B-owned pending ticket."""
        bombs = [b for b in self.state['round_bombs'] if b['round'] == self.room.generation]
        if self.battle_exit_mode == 'host_A_202_86':
            if bombs:
                return 'unsupported_battle_exit_after_bomb'
            if any(q['round'] == self.room.generation and q['subtype'] in (17, 19)
                   and not q.get('rejected') for q in self.state['event_reports']):
                return 'unsupported_battle_exit_after_trap'
            return None
        if not bombs:
            return None
        if self.battle_exit_mode != 'guest_B_pending_202_86':
            return 'unsupported_battle_exit_after_bomb'
        if (len(bombs) != 1 or bombs[0]['owner'] != 1
                or bombs[0].get('broadcast158') or bombs[0].get('timer_rejected')
                or not any(bomb is bombs[0] and self.room.fence.rejection(ticket) is None
                           for _, ticket, bomb in self.jobs)):
            return 'unsupported_battle_exit_pending_bomb'
        if any(q['round'] == self.room.generation and q['subtype'] in (17, 19)
               and not q.get('rejected') for q in self.state['event_reports']):
            return 'unsupported_battle_exit_after_trap'
        return None

    def death_lifecycle_rejection(self, event):
        """Opt-in measured Battle compatibility policy, not a life-state oracle.

        Retained native reports support 17 -> 19, with a fresh17 after an
        escape. A forged17 followed by19 is still outside this policy's scope.
        No cursor, inventory, marker or transport mutation on rejection.
        """
        if (self.channels.get(event['role']) is not event
                or not any(peer is event for peer in self.peers())):
            return 'death_member_not_current'
        if event.get('death_event', {}).get('round') == self.room.generation:
            return 'death_already_accepted'
        trap = event.get('pickup_ineligible')
        if (not trap or trap.get('generation') != self.room.generation
                or trap.get('subtype') != 17):
            return 'death_no_current_trap'
        return None

    def prepare_rescue_contact(self, event, report, body):
        """Read-only admission for measured A18/aux1 and B18/aux0 variants.

        This validates a declared diagnostic roster and report history, not
        contact distance, authenticated rescuer identity or original authority.
        Successful peer delivery precedes clearing the matching trap/cursor.
        Needle credit is deliberately untouched.
        """
        peers = self.peers()
        if (not self.room.active or self.host_role != 'A' or len(peers) != 2
                or {peer['role'] for peer in peers} != {'A', 'B'}
                or not any(peer is event for peer in peers)):
            return None, 'rescue_not_supported_AB_member'
        if len(body) != 28:
            return None, 'unsupported_rescue_shape'
        other = next(peer for peer in peers if peer is not event)
        aux = ROLE_SLOTS[other['role']]
        # Exact measured uncompressed envelope, including peer's slot0/1 aux. Do not
        # narrow arbitrary u32 values into an unchecked u8 peer argument.
        tail = codec.pack_fields([10, 0x00000518, aux << 16, 0], [4, 4, 4, 2])
        if body[14:] != tail:
            return None, 'unsupported_rescue_envelope_or_aux'
        if any(peer.get('death_event', {}).get('round') == self.room.generation for peer in peers):
            return None, 'rescue_member_report_dead'
        trap = event.get('pickup_ineligible')
        if not (trap and trap['generation'] == self.room.generation and trap['subtype'] == 17):
            return None, 'rescue_no_current_trap'
        blocked = other.get('pickup_ineligible')
        if blocked and blocked['generation'] == self.room.generation:
            return None, 'rescue_peer_report_ineligible'
        return {'trap_sha256': trap['report_sha256'], 'aux': aux,
                'peer_role': other['role'],
                'limits': 'declared friendly fixture; contact/life not authenticated'}, None

    def prepare_needle_escape(self, event, report, body):
        """Read-only admission for the bounded CP037/038 report1→0 pair.

        Credit and matching reported trap are emulator policy, not authenticated
        inventory/life state. Rescue18 and arbitrary active items are excluded.
        """
        peers = self.peers()
        if (not self.room.active or len(peers) != 2
                or {peer['role'] for peer in peers} != {'A', 'B'}
                or not any(peer is event for peer in peers)):
            return None, 'needle_not_current_AB_member'
        subtype = report['subtype']
        if subtype == 1:
            if len(body) != 14:
                return None, 'unsupported_needle_escape_shape'
        elif (len(body) != 22
              or codec.read_fields(body[14:], [1, 2, 1, 4]) != [1, 145, 0, 0]):
            return None, 'unsupported_needle_use_shape'
        death = event.get('death_event')
        if death and death['round'] == self.room.generation:
            return None, 'needle_actor_report_dead'
        trap = event.get('pickup_ineligible')
        if not (trap and trap['generation'] == self.room.generation and trap['subtype'] == 17):
            return None, 'needle_no_current_trap'
        inventory = event.get('needle_inventory')
        if not (inventory and inventory['generation'] == self.room.generation
                and inventory['count'] == 1):
            return None, 'needle_no_credit'
        if subtype == 1:
            if event.get('needle_pending'):
                return None, 'needle_pair_already_pending'
            return {'generation': self.room.generation, 'trap_sha256': trap['report_sha256'],
                    'prefix_hex': body[:6].hex(), 'tick': report['client_tick'],
                    'report1_sha256': report['frame_sha256'], 'report1_seq': report['seq']}, None
        pending = event.get('needle_pending')
        if not (pending and pending['generation'] == self.room.generation
                and pending['trap_sha256'] == trap['report_sha256']
                and pending['prefix_hex'] == (codec.pack_fields([1], [1]) + body[1:6]).hex()
                and pending['tick'] == report['client_tick']):
            return None, 'needle_pair_mismatch'
        return pending, None

    def forward_result21(self, event, report, body):
        """Optional measured-prefix translation; not report authentication.

        Native E6F370 clears local56 via vf148 before21. Its opposite peer has
        E86CA0 behind170/sub21. Send once per accepted17 epoch in drained mode;
        keep pickup eligibility and accepted death history unchanged.
        """
        if not (self.result_freshness and self.result_drain and self.room.active):
            return
        peers = self.peers()
        if not any(peer is event for peer in peers):
            return
        trap = event.get('pickup_ineligible')
        if not (trap and trap['generation'] == self.room.generation and trap['subtype'] == 17
                and self.result_candidate_current(report)):
            return
        previous = event.get('result21_forward')
        if (previous and previous['generation'] == self.room.generation
                and previous['trap_sha256'] == trap['report_sha256']):
            return
        other = next((peer for peer in peers if peer is not event), None)
        if other is None:
            return
        sent = self.send(other, 'start', f'event170-21-r{self.room.generation}',
                         opcode=170, body=body[:6])
        report['forward170'] = sent
        event['result21_forward'] = {
            'generation': self.room.generation, 'trap_sha256': trap['report_sha256'],
            'report_sha256': report['frame_sha256'], 'send': sent}

    def close_connection(self, ordinal, reason):
        """One owner serializes close, lifetime invalidation and later recovery."""
        state = self.connections[ordinal]
        event = state['event']
        if event.get('closed'):
            return
        event['closed'] = {'reason': reason, 'monotonic_ns': time.monotonic_ns()}
        event.pop('needle_pending', None)
        event.pop('needle_inventory', None)
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
        # This diagnostic profile supports B taking over A, or A staying chief.
        alias = 'promote_B' if peer['role'] == 'B' else ('slot_C' if peer['role'] == 'C' else 'slot_A')
        sent = []
        for name, label, opcode, payload in [('start', 'remove107', 107, body),
                                             (alias, 'survivor110', 110, None)]:
            try:
                sent.append({'role': peer['role'], **self.send(peer, name, label,
                                                             opcode=opcode, body=payload)})
            except (BrokenPipeError, ConnectionResetError, ConnectionAbortedError, TimeoutError) as error:
                # A simultaneous disconnect can precede this peer's read-ready
                # EOF. Attribute the failed delivery to its recipient, not the
                # departing socket whose EOF initiated the cleanup. send() only
                # records a successful frame after sendall returns.
                self.state.setdefault('send_failures', []).append({
                    'phase': 'departure', 'departed': departed, 'recipient': peer['role'],
                    'connection': peer['global_ordinal'], 'opcode': opcode,
                    'errno': error.errno, 'error': str(error),
                    'monotonic_ns': time.monotonic_ns(), 'sent_prefix': list(sent)})
                self.close_connection(peer['global_ordinal'], 'departure_send_failure')
                return sent
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
                terrain_after_blast, terrain_result = None, None
                if self.terrain:
                    terrain_after_blast = copy.deepcopy(self.terrain)
                    terrain_result = terrain_after_blast.blast(bomb['id'])
                body = codec.pack_fields([1, bomb['id'], 0, 2, 0xFFFFFFFF, 255], [1, 2, 1, 4, 4, 1])
                bomb['broadcast158'] = self.broadcast('blast', f"blast158-r{bomb['round']}-{bomb['id']}", body=body)
                if self.result_freshness:
                    # Only a successful complete broadcast invalidates earlier
                    # survivor reports. Even a later secondary's own158 counts;
                    # this is a conservative fixture policy, not native law.
                    self.round['result_last_blast'] = {
                        'id': bomb['id'], 'generation': self.room.generation,
                        'monotonic_ns': max(sent['monotonic_ns']
                                            for sent in bomb['broadcast158'])}
                if terrain_after_blast:
                    # No terrain/release commit for a failed/partial broadcast.
                    # The existing transport exception still surfaces; this is
                    # not whole-service recovery or rollback of the fence.
                    self.terrain = terrain_after_blast
                    bomb['terrain_blast'] = {
                        'source': 'design', 'generation': self.room.generation,
                        'processed': list(terrain_result.processed),
                        'cleared': [list(cell) for cell in terrain_result.cleared],
                        'before_flags': list(terrain_result.before),
                        'after_flags': list(terrain_result.after),
                        'committed_ns': time.monotonic_ns()}
                    self.round['terrain_flags'] = list(self.terrain.flags)
                    for _, other_ticket, other in self.jobs:
                        if (other['id'] in terrain_result.processed
                                and other['round'] == self.room.generation
                                and self.room.fence.rejection(other_ticket) is None):
                            other['capacity_release'] = {
                                'root_id': bomb['id'], 'generation': self.room.generation,
                                'monotonic_ns': time.monotonic_ns(), 'source': 'design',
                                'policy': 'pinned_simple_terrain_cascade'}
                            self.state['policy_events'].append({
                                'op': 'terrain_capacity_release', 'wire_id': other['id'],
                                **other['capacity_release']})
                if self.capacity_chains and not bomb.get('capacity_release'):
                    pending = [other for _, pending_ticket, other in self.jobs
                               if other['round'] == self.room.generation
                               and not other.get('capacity_release')
                               and self.room.fence.rejection(pending_ticket) is None]
                    for other in self.capacity_chains.direct_targets(bomb, pending):
                        other['capacity_release'] = {
                            'root_id': bomb['id'], 'generation': self.room.generation,
                            'monotonic_ns': time.monotonic_ns(), 'source': 'design',
                            'policy': 'pinned_clear_cells_direct_first_hit'}
                        self.state['policy_events'].append({
                            'op': 'chain_capacity_release', 'wire_id': other['id'],
                            **other['capacity_release']})
            else:
                bomb['timer_rejected'] = reason
            self.state['policy_events'].append({'op': 'bomb_deadline', 'generation': ticket.generation,
                                                'wire_id': ticket.wire_id, 'rejected': reason})
        self.tick_setup(now)
        self.tick_loading(now)
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
                        peer_ip = int(ipaddress.IPv4Address(self.peer_ips[other['role']]))
                        body = bytearray(self.profile.body('peer')[:35])
                        body[15:19] = codec.pack_fields([self.room.key], [4])
                        for slot in range(26):
                            active = slot < (3 if self.rescue_contact else 2)
                            body.extend(codec.pack_fields([int(active)], [1]))
                            if active:
                                remote = slot < 2 and slot != ROLE_SLOTS[event['role']]
                                for ip, endpoint in ([(peer_ip, port), (0, 0), (0, 0)] if remote else [(0, 0)] * 3):
                                    body.extend(codec.pack_fields([ip, endpoint], [4, 2]))
                        event['begin_response_sent'] = self.send(event, 'peer', 'peer153', body=bytes(body))
                        event['peer153_sent'] = {**event['begin_response_sent'], 'remote_port': port,
                                                'remote_ip': self.peer_ips[other['role']]}
                        self.state['policy_events'].append({'op': 'peer_key', 'role': event['role'],
                                                            'generation': self.room.generation, 'key': self.room.key})
            death = next((peer['death_event'] for peer in self.peers() if peer.get('death_event')), None)
            remote_reports = self.state['event_reports']
            if not self.finish_all_dead(now) and death and now - death['monotonic_ns'] >= 8 * 10**9 and any(
                row['role'] != death['role'] and row['role'] in {peer['role'] for peer in self.peers()}
                and row['round'] == self.room.generation and row['subtype'] == 21
                and not row.get('rejected')
                and row['monotonic_ns'] > death['monotonic_ns'] for row in remote_reports
                if self.result_candidate_current(row)
            ) and not self.pending_result_jobs():
                self.room.finish()
                alias = 'rank' if death['slot'] == 0 else 'rank_Awin'
                self.round['rank'] = {'monotonic_ns': now, 'broadcast': self.broadcast(alias, f"rank175-r{self.room.generation}")}
                if self.result_freshness:
                    self.round['rank']['policy'] = ('post_blast_untrapped_drained' if self.result_drain
                                                    else 'post_blast_untrapped')
                    self.round['rank']['last_blast'] = dict(self.round.get('result_last_blast', {}))
                self.state['policy_events'].append({'op': 'finish', 'generation': self.room.generation})
        if self.round.get('rank') and not self.round.get('return') and now - self.round['rank']['monotonic_ns'] >= 5 * 10**9:
            self.round['return'] = {'monotonic_ns': now, 'broadcast': self.broadcast('return', f"return180-r{self.room.generation}")}
        if self.round.get('return') and not self.round.get('refresh') and now - self.round['return']['monotonic_ns'] >= 3 * 10**9:
            if self.round.get('abort'):
                sent = self.cleanup_departed(self.round['abort']['role'])
            else:
                if self.host_role == 'B':
                    sent = self.broadcast('promote_B', f"host110-r{self.room.generation}")
                    guest = self.channels.get('A')
                    alias = 'slot_A_guest' if guest and guest.get('promoted_rejoin') and not guest.get('closed') else 'slot_C'
                    sent += self.broadcast(alias, f"guest110-r{self.room.generation}")
                else:
                    sent = self.broadcast('slot_A', f"host110-r{self.room.generation}")
                    sent += self.broadcast('slot_B', f"guest110-r{self.room.generation}")
                sent += self.refresh_rescue_opponent()
            self.round['refresh'] = {'monotonic_ns': now, 'broadcast': sent}
            for event in self.peers():
                event.pop('guest_ready_received', None)

    def refresh_rescue_opponent(self):
        """Restore the declared diagnostic actor's readiness after return.

        Read-only runtime observed slot2 group1 before Start, group0 after
        return, then DCB7DA/room guard3. This is emulator roster policy for
        the fixed opt-in profile, never a fabricated live client's Ready95.
        The native104/Start2/fresh-rescue path still needs runtime verification.
        """
        if not self.rescue_contact:
            return []
        body = codec.pack_fields([2, 0x24, 0], [1, 4, 1])
        return self.broadcast('slot_B', f'synthetic-slot2-ready110-r{self.room.generation}',
                              opcode=110, body=body)

    def pending_result_jobs(self):
        """Experimental fixture policy; original server's finish rule unknown.

        Capacity release after a native chain does not retire the secondary's
        own server timer. Keep that successful158 boundary in this mode too.
        The all-dead branch has no survivor and retains its separate policy.
        """
        if not self.result_freshness or not self.result_drain:
            return []
        return [bomb['id'] for _, ticket, bomb in self.jobs
                if bomb['round'] == self.room.generation
                and self.room.fence.rejection(ticket) is None]

    def result_candidate_current(self, report):
        """Opt-in report freshness; not authoritative location or life state.

        The drained experiment permits later accepted21 after17 for result
        selection only: measured native E6F370 clears56 via vf148 before21.
        It does not clear pickup eligibility or supersede accepted death19.
        The original non-drained mode retains permanent17/19 exclusion.
        """
        if not self.result_freshness:
            return True
        peer = next((peer for peer in self.peers() if peer['role'] == report['role']), None)
        if peer is None:
            return False
        # A subsequent accepted17 must not hide an earlier accepted death19.
        # Pickup markers describe the latest report, not death history.
        death = peer.get('death_event')
        if self.result_drain and death and death['round'] == self.room.generation:
            return False
        blocked = peer.get('pickup_ineligible')
        if blocked and blocked['generation'] == self.room.generation:
            if not self.result_drain or blocked['subtype'] != 17:
                return False
            if report['monotonic_ns'] <= blocked.get('monotonic_ns', report['monotonic_ns']):
                return False
        blast = self.round.get('result_last_blast')
        return not blast or report['monotonic_ns'] > blast['monotonic_ns']

    def finish_all_dead(self, now):
        """Optional A/B fixture policy: both accepted dead, both rank2.

        CP082 observed no report21 when both players died. This is a chosen
        all-eliminated outcome, not a recovered tie enum or original authority.
        Return True while this outcome is pending, so a stray report21 cannot
        choose a winner from the first death during the final settling delay.
        """
        peers = self.peers()
        if len(peers) != 2 or {peer['role'] for peer in peers} != {'A', 'B'}:
            return False
        deaths = [peer.get('death_event') for peer in peers]
        if not all(death and death['round'] == self.room.generation
                   and death['subtype'] == 19 and death['role'] == peer['role']
                   and death['slot'] == ROLE_SLOTS[peer['role']]
                   and not death.get('rejected')
                   for peer, death in zip(peers, deaths)):
            return False
        if 'rank_all_dead' not in self.profile.frames:
            if not self.round.get('all_dead_unsupported'):
                self.round['all_dead_unsupported'] = True
                self.state['policy_events'].append({
                    'op': 'all_dead_result_unsupported', 'generation': self.room.generation,
                    'reason': 'profile_missing_rank_all_dead'})
            return True
        latest = max(death['monotonic_ns'] for death in deaths)
        if now - latest < 8 * 10**9:
            return True
        # Resolve/validate the optional alias before closing the round. send()
        # uses its existing current per-connection sequence, not the base key.
        body = self.profile.body('rank_all_dead')
        self.room.finish()
        self.round['rank'] = {
            'monotonic_ns': now,
            'policy': 'all_live_AB_accepted_dead_rank2',
            'death_sequences': {death['role']: death['seq'] for death in deaths},
            'broadcast': self.broadcast('rank_all_dead',
                                        f'rank175-all-dead-r{self.room.generation}', body=body)}
        self.state['policy_events'].append({
            'op': 'finish_all_dead', 'generation': self.room.generation,
            'source': 'design', 'ranks': [2, 2], 'latest_death_ns': latest})
        return True

    def tick_setup(self, now):
        for event in self.events:
            if event.get('reply_ordinal') != 4 or event.get('closed'):
                continue
            role = event['role']
            left = event.get('last_room_leave')
            if (left and not left.get('refresh')
                    and now - left['monotonic_ns'] >= 10**9):
                left['refresh'] = [self.send(event, 'ready', 'return-lobby241'),
                                   self.send(event, 'roomlist_B' if left.get('variant') in ('host_A_to_B', 'active_host_A') else 'roomlist_' + role, 'return-lobby91')]
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
                        if event.get('promoted_rejoin'):
                            substitutions = {'slot_A': 'slot_A_guest', 'guest_A': 'guest_B_chief'}
                            aliases = [substitutions.get(alias, alias) for alias in aliases]
                        self.send_setup(event, aliases, 'reentry-setup')
                        event[marker] = {'monotonic_ns': now}
            if event.get('map_response_sent'):
                for departed in event.pop('pending_departures', []):
                    sent = self.deliver_departure(event, departed)
                    self.state['policy_events'].append({'op': 'deferred_cleanup', 'role': departed,
                                                        'recipient': role, 'broadcast': sent,
                                                        'monotonic_ns': time.monotonic_ns()})
        self.prepare_rescue_roster()

    def prepare_rescue_roster(self):
        """Declare the CP036 synthetic opponent in the room, before Start.

        Explicit diagnostic actor, not replacement identity C or a third live
        member. The corresponding 153 slot keeps this profile after load.
        """
        peers = self.peers()
        if (not self.rescue_contact or self.room.active or self.host_role != 'A'
                or len(peers) != 2 or {p['role'] for p in peers} != {'A', 'B'}):
            return
        for event in peers:
            if not event.get('synthetic_opponent_sent'):
                event['synthetic_opponent_sent'] = self.send(
                    event, 'synthetic_opponent', 'synthetic-slot2-106', opcode=106)

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
        # select() returns a batch: an earlier peer's EOF cleanup can close
        # this recipient before its cached readiness key is dispatched.
        if event.get('closed'):
            return
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
                        self.send_raw(event, b''.join(
                            self.redirect_frame if name == 'redirect' and self.redirect_frame is not None
                            else self.profile.raw(name) for name in names))
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
                listener.bind((self.bind_ip, port))
                listener.listen(8)
                listeners.append(listener)
                self.selector.register(listener, selectors.EVENT_READ, None)
            self.state['listen_ports'] = [listener.getsockname()[1] for listener in listeners]
            self.state['listen_addresses'] = [listener.getsockname() for listener in listeners]
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
    parser.add_argument('--bind-ip', default='127.0.0.1')
    parser.add_argument('--peer-ip', action='append', default=[], metavar='ROLE=IPv4')
    parser.add_argument('--channel-ip', help='Explicit IPv4 destination in measured S2C7 redirect')
    parser.add_argument('--channel-port', type=int, help='Destination port; requires --channel-ip')
    args = parser.parse_args()
    if any(not 0 <= port <= 65535 for port in [*args.port, args.proxy_a, args.proxy_b]):
        parser.error('invalid port')
    try:
        bind_ip, peer_ips = network_config(args.bind_ip, peer_ip_options(args.peer_ip))
        channel_endpoint_config(args.channel_ip, args.channel_port)
    except ValueError as error:
        parser.error(str(error))
    service = ProbeService(FixtureProfile(args.profile), args.out, {'A': args.proxy_a, 'B': args.proxy_b},
                           bind_ip=bind_ip, peer_ips=peer_ips,
                           channel_ip=args.channel_ip, channel_port=args.channel_port)
    args.out.mkdir(parents=True, exist_ok=True)
    def stop(_signal, _frame):
        service.closed = True
    signal.signal(signal.SIGTERM, stop)
    signal.signal(signal.SIGINT, stop)
    service.run(args.port)


if __name__ == '__main__':
    main()
