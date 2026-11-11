"""Opt-in four-session room/Battle diagnostic with explicit emulator policies.

Four local operator-IP sessions reuse measured KR codec/fixtures. Optional bomb
and report-derived 2v2 result/reset policies have bounded Wine evidence. This is
not account authentication, full four-player gameplay or a hardened Internet service.
"""
import argparse
from functools import wraps
import hashlib
import ipaddress
from pathlib import Path
import selectors
import signal
import socket
import struct
import sys
import time

from server import probe_codec as codec
from server.probe_service import FixtureProfile, ProbeService
from server.room_title import create_title, _decode_bytes
from server.room_team_mode import create_selector, replace_reply_selector
# Production import is resolved when this staged module is promoted.
from server import four_player_profile as fp
from server.four_bomb_policy import FourBombAdmission
from server.four_battle_policy import FourBattleReports, rank_body
from server.four_input_limits import FourInputBudget
import json


class RecipientSendFailure(OSError):
    def __init__(self, ordinal, error):
        super().__init__(str(error))
        self.recipient_ordinal = ordinal
        self.cause_type = type(error).__name__


def recipient_safe(method):
    """Stop a failed delivery at its owner entrypoint, never blame input origin."""
    @wraps(method)
    def wrapped(self, *args, **kwargs):
        try:
            return method(self, *args, **kwargs)
        except RecipientSendFailure as error:
            ordinal = error.recipient_ordinal
            event = self.connections[ordinal]['event']
            self.state.setdefault('delivery_failures', []).append(dict(
                source='design', recipient=event['role'], connection=ordinal,
                input_connection=args[0] if method.__name__=='receive' else None,
                owner_entrypoint=method.__name__, error=str(error),
                error_type=error.cause_type, monotonic_ns=time.monotonic_ns()))
            self.close_connection(ordinal, 'recipient_send_failed: '+str(error))
    return wrapped


def addresses(options, members=None):
    members = fp.MEMBERS if members is None else members
    result = {}
    for value in options:
        role, sep, address = value.partition('=')
        if not sep or role not in members or role in result:
            raise ValueError('expected unique '+''.join(members)+' ROLE=IPv4')
        ip = ipaddress.IPv4Address(address)
        if ip.is_unspecified or ip.is_multicast or int(ip) == 0xFFFFFFFF:
            raise ValueError('unsupported peer address')
        result[role] = str(ip)
    if set(result) != set(members) or len(set(result.values())) != len(members):
        raise ValueError(str(len(members))+' distinct operator source IPs required')
    return result


from server.four_observability import FourObservability
from server.four_identity_store import pin_identities

class FourPlayerService(ProbeService):
    PLAYER_PROFILE = fp
    RANK_BODY = staticmethod(rank_body)
    ROOM_SELECTORS = (0,)
    def __init__(self, profile, out, source_ips, *, bind_ip='127.0.0.1',
                 channel_ip=None, channel_port=None, bomb_profile=None, battle_outcome=False, abort_room=False, identity_store=None):
        self.player_profile = self.PLAYER_PROFILE
        if self.player_profile.ROLES not in ('ABCD','ABCDEFGH'):
            raise ValueError('only fixed four/eight session profiles supported')
        if len(self.player_profile.ROLES)==8 and (bomb_profile is not None or battle_outcome or abort_room or identity_store is not None):
            raise ValueError('eight session preview excludes unmeasured bomb/outcome/abort/identity persistence')
        # Validate/generate every supported input before creating an owner/socket.
        if struct.unpack_from('<I', profile.raw('auth_A'), 5)[0] != 4:
            raise ValueError('four sessions require measured auth4 success fixture')
        source_ips = addresses([role + '=' + ip for role, ip in source_ips.items()],self.player_profile.MEMBERS)
        players = {role: self.player_profile.player_body(profile.body('player_B'), member,
                                        chief=role == 'A')
                   for role, member in self.player_profile.MEMBERS.items()}
        auth = {role: self.player_profile.auth_body(profile.body('auth_A'), member)
                for role, member in self.player_profile.MEMBERS.items()}
        own = {role: self.player_profile.own_body(body) for role, body in players.items()}
        start = self.player_profile.start_body(profile.body('start'))
        joins = {role: self.player_profile.join_body(profile.body('join'), self.player_profile.MEMBERS[role]['slot'])
                 for role in self.player_profile.ROLES[1:]}
        if 'replacement_slot' in profile.data or 'rescue_contact' in profile.data:
            raise ValueError('four sessions exclude replacement/synthetic actors')
        if type(abort_room) is not bool or (abort_room and not battle_outcome):
            raise ValueError('abort-room requires explicit four Battle outcome')
        if abort_room:
            try:
                if codec.read_fields(profile.body('return'), [1,1,1]) != [0,0,0]:
                    raise ValueError('unsupported abort180 variant')
            except (KeyError, ValueError) as error:
                raise ValueError('abort-room requires measured return180(0,0,0)') from error
        super().__init__(profile, out, {}, bind_ip=bind_ip,
                         channel_ip=channel_ip, channel_port=channel_port)
        self.source_ips, self.source_counts = source_ips, dict.fromkeys(self.player_profile.ROLES, 0)
        self.player_bodies, self.auth_bodies = players, auth
        self.own_bodies, self.start_body, self.join_bodies = own, start, joins
        self.role_slots = {r: m['slot'] for r, m in self.player_profile.MEMBERS.items()}
        self.state['schema'] = 'kr-four-session-service-v1'
        self.abort_room_enabled = abort_room
        self.abort_room_job = None
        self.state['abort_room_policy'] = dict(source='design', enabled=abort_room,
            mode='active_member_loss_unranked180_then_setup107110',
            limits='Fixedfourprofile/currentroomabort, no originalwinnerlaw/accountresume/three-memberStart.')
        self.io_budget = FourInputBudget()
        self.state['io_budget'] = dict(source='design', limits=self.io_budget.limits,
                                       connections=self.io_budget.records)
        self.state['four_session_binding'] = dict(source='design',
            role_ips=self.source_ips, counts=self.source_counts,
            members=self.player_profile.MEMBERS, mode='four_operator_IPs_four_transports_each')
        self.state['limits'] = ('Four operator-IP sessions; optional explicit bomb and report-derived '
                                '2v2 result/reset policies, not authenticated life/original authority. '
                                'No four-item/terrain/rescue/needle integration, general resume/replacement, '
                                'NAT discovery, Internet or gate certification.')
        if type(battle_outcome) is not bool or (battle_outcome and bomb_profile is None):
            self.selector.close()
            raise ValueError("four Battle outcome requires explicit bomb profile")
        self.battle_reports = (FourBattleReports(self.room, self.role_slots,
            {r:m["team"] for r,m in self.player_profile.MEMBERS.items()}) if battle_outcome else None)
        self.bomb_admission = None
        self.bomb_jobs = []
        self.bomb_fuse_seconds = None
        if bomb_profile is not None:
            if (not isinstance(bomb_profile, dict)
                    or set(bomb_profile) != {'mode','capacities','powers','fuse_seconds'}
                    or bomb_profile['mode'] != 'four_current_baseline'
                    or type(bomb_profile['fuse_seconds']) is not int
                    or not 1 <= bomb_profile['fuse_seconds'] <= 30):
                self.selector.close()
                raise ValueError('unsupported four-bomb profile')
            try:
                self.bomb_admission = FourBombAdmission(self.room, self.role_slots,
                    bomb_profile['capacities'], bomb_profile['powers'])
            except ValueError:
                self.selector.close()
                raise
            self.bomb_fuse_seconds = bomb_profile['fuse_seconds']
            self.state['four_bomb_policy'] = dict(bomb_profile, source='design',
                limits='Explicit baseline/reservation/fuse policy only; not native authority, '
                       'terrain/drop/authenticated life or full four-player gameplay.')
        self.state['bomb_deadlines'] = []
        self.state['policy_sources'] += [dict(path=str(Path(module.__file__).resolve()),
            sha256=hashlib.sha256(Path(module.__file__).read_bytes()).hexdigest())
            for module in [self.player_profile, sys.modules[FourBombAdmission.__module__],
                           sys.modules[FourBattleReports.__module__],
                           sys.modules[FourInputBudget.__module__],
                           sys.modules[FourObservability.__module__],
                           sys.modules[pin_identities.__module__],
                           __import__(__name__, fromlist=['FourPlayerService'])]]
        for module in [sys.modules[type(self).__module__],
                       sys.modules[self.player_profile.player_body.__module__]]:
            path = str(Path(module.__file__).resolve())
            if not any(row['path']==path for row in self.state['policy_sources']):
                self.state['policy_sources'].append(dict(path=path,
                    sha256=hashlib.sha256(Path(path).read_bytes()).hexdigest()))

        self.operator_identities = {role:dict(member) for role,member in self.player_profile.MEMBERS.items()}
        try:
            self.identity_store = pin_identities(identity_store,self.player_profile.MEMBERS) if identity_store else None
        except BaseException:
            self.selector.close()
            raise
        self.state['identity_store'] = self.identity_store
        self.observability = FourObservability(self)

    def save(self):
        return self.observability.publish(self)

    def run(self, ports):
        # Same selector/receive/tick ownership as ProbeService; four-service
        # health records shutdown before and after closing owned resources.
        listeners = []
        try:
            for port in ports:
                listener = socket.socket()
                listeners.append(listener)
                listener.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
                listener.bind((self.bind_ip, port))
                listener.listen(8)
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
                        self.state['errors'].append(dict(connection=key.data,error=str(error)))
                        if key.data is None:
                            raise
                        self.close_connection(key.data, str(error))
                if not self.closed:
                    self.tick()
                self.save()
        finally:
            self.closed = True
            try:
                self.save()
            finally:
                self.room.finish()
                for entry in self.connections.values():
                    entry['socket'].close()
                for listener in listeners:
                    listener.close()
                self.selector.close()
                self.state['stopped'] = True
                self.save()

    def send_raw(self, event, data):
        reason = self.io_budget.sending(event['global_ordinal'], len(data))
        if reason is not None:
            raise RecipientSendFailure(event['global_ordinal'], OSError(reason))
        try:
            super().send_raw(event, data)
        except OSError as error:
            raise RecipientSendFailure(event['global_ordinal'], error) from error

    def reject(self, event, opcode, sequence, reason):
        self.reject_stage(event, opcode, sequence, reason)

    @recipient_safe
    def accept(self, listener):
        connection, peer = listener.accept()
        role = next((r for r, ip in self.source_ips.items() if ip == peer[0]), None)
        reason = ('unmapped_source' if role is None else
                  'role_transports_exhausted' if self.source_counts[role] >= 4 else None)
        if reason:
            self.state.setdefault('admission_rejections', []).append(dict(
                source='design', peer=list(peer), role=role, reason=reason,
                monotonic_ns=time.monotonic_ns()))
            self.state['admission_rejections_total'] = self.state.get('admission_rejections_total', 0)+1
            del self.state['admission_rejections'][:-256]
            connection.close()
            return
        stage = self.source_counts[role] + 1
        ordinal = len(self.events) + 1
        event = dict(global_ordinal=ordinal, role=role, reply_ordinal=stage,
                     peer=list(peer), port=listener.getsockname()[1],
                     accept_monotonic_ns=time.monotonic_ns(), chunks=[])
        connection.settimeout(2)
        self.events.append(event)
        self.connections[ordinal] = dict(socket=connection, event=event,
                                        buffer=bytearray(), sequence=40)
        self.source_counts[role] = stage
        if stage == 4:
            self.channels[role] = event
        self.send_raw(event, self.profile.raw('bootstrap'))
        event['reply_sent'] = dict(monotonic_ns=time.monotonic_ns())
        self.selector.register(connection, selectors.EVENT_READ, ordinal)

    @recipient_safe
    def receive(self, ordinal):
        state = self.connections[ordinal];event = state['event']
        data = state['socket'].recv(65536)
        if not data:
            self.close_connection(ordinal, 'eof')
            return
        reason = self.io_budget.chunk(ordinal, len(data), time.monotonic())
        if reason is not None:
            self.reject_input_budget(event, reason, data)
            return
        event['chunks'].append(dict(monotonic_ns=time.monotonic_ns(), hex=data.hex()))
        stage = event['reply_ordinal']
        if stage == 2:
            if not event.get('second_response_sent'):
                self.send_raw(event, self.profile.raw('second'))
                event['second_response_sent'] = dict(monotonic_ns=time.monotonic_ns())
            return
        if stage < 3:
            return
        state['buffer'].extend(data)
        while (frame := codec.pop_frame(state['buffer'])) is not None:
            reason = self.io_budget.frame(ordinal, time.monotonic())
            if reason is not None:
                self.reject_input_budget(event, reason, frame)
                return
            seq = state['sequence'];body = codec.frame_body(frame, seq)
            state['sequence'] += 3
            opcode = struct.unpack_from('<I', frame, 5)[0]
            event['c2s_frame_count'] = event.get('c2s_frame_count', 0) + 1
            if stage == 4:
                self.receive_channel(event, frame, seq, body)
                continue
            if opcode == 1 and not event.get('auth_response_sent'):
                template = self.profile.raw('auth_A')
                self.send_raw(event, codec.encode(template[:37], struct.unpack_from('<I', template, 5)[0],
                                                  self.auth_bodies[event['role']], 43))
                event['auth_response_sent'] = dict(monotonic_ns=time.monotonic_ns())
            elif opcode == 9 and event.get('auth_response_sent') and not event.get('netinfo_response_sent'):
                self.send_raw(event, self.profile.raw('netinfo') + self.profile.raw('flags'))
                event['netinfo_response_sent'] = dict(monotonic_ns=time.monotonic_ns())
            elif opcode == 5 and event.get('netinfo_response_sent') and not event.get('redirect_response_sent'):
                self.send_raw(event, self.redirect_frame if self.redirect_frame is not None
                              else self.profile.raw('redirect'))
                event['redirect_response_sent'] = dict(monotonic_ns=time.monotonic_ns())
            else:
                self.reject(event, opcode, seq, 'login_stage_or_opcode')

    def reject_input_budget(self, event, reason, data):
        self.state.setdefault('input_rejections', []).append(dict(
            source='design', connection=event['global_ordinal'], role=event['role'],
            reason=reason, bytes=len(data), sha256=hashlib.sha256(data).hexdigest(),
            monotonic_ns=time.monotonic_ns()))
        self.close_connection(event['global_ordinal'], reason)

    def receive_channel(self, event, frame, sequence, body):
        opcode = struct.unpack_from('<I', frame, 5)[0];role = event['role']
        if opcode == 127 and self.battle_reports is not None:
            self.receive_four_report(event, frame, sequence, body)
            return
        if opcode == 118 and self.bomb_admission is not None:
            self.receive_bomb(event, frame, sequence, body)
            return
        job = self.abort_room_job
        if (opcode == 585 and job and role in job['returns']
                and not job.get('completed_ns') and event in self.peers()):
            job['setup_requests'].setdefault(role, dict(sequence=sequence,
                monotonic_ns=time.monotonic_ns(), frame_sha256=hashlib.sha256(frame).hexdigest()))
            return
        if opcode == 6 and not event.get('channel_response_sent'):
            name = self.player_profile.MEMBERS[role]['name'].encode('ascii')
            size = codec.read_fields(body[2:4], [2])[0] if len(body) >= 4 else -1
            login = next((e for e in self.events if e['role'] == role and
                          e['reply_ordinal'] == 3), {})
            if (not login.get('redirect_response_sent') or len(body) != 36 + len(name)
                    or size != len(name) or _decode_bytes(body[4:4+size]) != name):
                self.reject(event, opcode, sequence, 'channel_identity_or_login_stage')
                return
            event['s2c_next_seq'] = 43
            self.send(event, 'channel', 'lobby17',
                      body=codec.frame_body(self.profile.raw('channel'), 43))
            event['channel_response_sent'] = dict(monotonic_ns=time.monotonic_ns())
            event['channel_identity'] = dict(name=name.decode('ascii'), source='emu_choice')
        elif opcode == 67 and event.get('channel_response_sent') and not event.get('ready_response_sent'):
            self.send(event, 'ready', 'lobby241')
            alias = 'roomlist_A' if role == 'A' else 'roomlist_B'
            self.send(event, alias, 'lobby91')
            self.send(event, 'channel_list', 'channel14',
                      body=codec.frame_body(self.profile.raw('channel_list'), 52))
            self.send(event, 'player_B', 'own291', opcode=291, body=self.own_bodies[role])
            event['ready_response_sent'] = dict(monotonic_ns=time.monotonic_ns())
        elif opcode == 528 and event.get('ready_response_sent') and not body:
            self.send(event, 'table', 'cache238')
        elif opcode in (81, 82) and event.get('ready_response_sent') and not event.get('create_response_sent'):
            host = self.channels.get('A', {})
            if self.room.active or self.state.get('membership_broken'):
                self.reject(event, opcode, sequence, 'room_unavailable')
                return
            if role == 'A' and opcode == 81:
                title, selector = create_title(body), create_selector(body)
                if selector not in self.ROOM_SELECTORS:
                    self.reject(event, opcode, sequence, 'four_session_requires_free0')
                    return
                self.room_title_bytes, self.room_team_selector = title, selector
                self.send(event, 'create', 'create86',
                          body=replace_reply_selector(self.profile.body('create'), 86, selector))
            elif role != 'A' and opcode == 82 and host.get('create_response_sent'):
                fields = codec.read_fields(body, [2, 4, 2, 1, 2])
                if fields != [1, 1, 0, 1, 5]:
                    self.reject(event, opcode, sequence, 'unsupported_join_context')
                    return
                self.send(event, 'join', 'join87', body=replace_reply_selector(
                    self.join_bodies[role], 87, self.room_team_selector))
            else:
                self.reject(event, opcode, sequence, 'host_create_required')
                return
            event['create_response_sent'] = dict(monotonic_ns=time.monotonic_ns())
        elif opcode == 585 and event.get('create_response_sent') and not event.get('player_response_sent'):
            self.send(event, 'player_B', 'own106', opcode=106, body=self.player_bodies[role])
            event['player_response_sent'] = dict(monotonic_ns=time.monotonic_ns())
        elif opcode == 95 and role != 'A' and event.get('map_response_sent') and not self.room.active:
            stored = codec.read_fields(body[:4], [4])[0] if len(body) >= 9 else -1
            if (not 6 <= stored <= 8192 or len(body) != stored + 9
                    or codec.read_fields(body[-5:], [1, 4]) != [1, 0]):
                self.reject(event, opcode, sequence, 'unsupported_ready95_envelope')
                return
            if not event.get('guest_ready_received') and not self.state.get('membership_broken'):
                event['guest_ready_received'] = dict(monotonic_ns=time.monotonic_ns(),
                                                     payload_hex=body.hex(), source='emu_choice')
                self.broadcast('slot_B', 'ready110', opcode=110, body=self.player_profile.slot_body(
                    self.profile.body('player_B'), self.player_profile.MEMBERS[role], ready=True))
        elif opcode == 104 and role == 'A':
            peers = self.peers()
            if (self.room.active or (self.round['index'] and not self.round.get('refresh')) or self.state.get('membership_broken')
                    or {e['role'] for e in peers} != set(self.player_profile.ROLES)
                    or not all(e.get('guest_ready_received') for e in peers if e['role'] != 'A')):
                self.reject(event, opcode, sequence, 'four_members_ready_required')
                return
            self.start_round()
        elif opcode == 99 and self.room.active and event in self.peers() and not event.get('init99'):
            values = codec.read_fields(body, [4, 4, 2, 4, 2, 1, 1, 1, 4])
            if not 1 <= values[2] <= 65535:
                self.reject(event, opcode, sequence, 'invalid_udp_port')
                return
            # Never trust the reported IPv4 as operator routing/authentication.
            event['init99'] = dict(fields=values, seq=sequence,
                monotonic_ns=time.monotonic_ns(), sha256=hashlib.sha256(frame).hexdigest())
        else:
            self.reject(event, opcode, sequence, 'unsupported_four_session_phase_or_opcode')

    def start_round(self):
        key = self.room.begin();now = time.monotonic_ns()
        if self.round['index']:
            self.state['round_history'].append(self.round)
        self.round = dict(index=self.room.generation, key=key, host_role='A',
            roster=[dict(role=r, **self.player_profile.MEMBERS[r]) for r in self.player_profile.ROLES],
            loading=dict(generation=self.room.generation, started_ns=now,
                deadline_ns=now + 30*10**9,
                members=[dict(role=e['role'], connection=e['global_ordinal']) for e in self.peers()]))
        self.state['current_round'] = self.round
        if self.bomb_admission is not None:
            self.bomb_admission.begin()
        if self.battle_reports is not None:
            self.battle_reports.begin()
        for event in self.peers():
            for marker in ['init99','go_response_sent','play_response_sent','peer153_sent']:
                event.pop(marker, None)
            self.send(event, 'floor', 'floor582')
            event['start_response_sent'] = self.send(event, 'start', 'start121', body=self.start_body)

    def receive_bomb(self, event, frame, sequence, body):
        role = event['role']
        report = dict(role=role, generation=self.room.generation, seq=sequence,
                      monotonic_ns=time.monotonic_ns(), body_sha256=hashlib.sha256(body).hexdigest(),
                      frame_sha256=hashlib.sha256(frame).hexdigest())
        self.state['bomb_requests'].append(report)
        if (not self.room.active or event not in self.peers()
                or not event.get('peer153_sent') or self.state.get('membership_broken')):
            report['rejected'] = 'not_current_loaded_member'
            return
        if (self.battle_reports is not None
                and self.battle_reports.generation == self.room.generation
                and self.battle_reports.life.get(role, {}).get('death') is not None):
            report['rejected'] = 'bomb_owner_reported_dead'
            return
        admitted = self.bomb_admission.admit(role, body)
        if 'rejected' in admitted:
            report.update(admitted)
            return
        values = admitted['fields'];ident = admitted['id'];report.update(id=ident,fields=values)
        fields = [0, values[2], 0, values[3], 0, 0, values[4], values[5], values[10],
                  1, 0, ident, 0, 1, 0, 0, 0]
        payload = codec.pack_fields(fields,[1,1,2,4,4,4,1,1,1,4,1,2,2,1,8,1,1])
        sent = self.broadcast('bomb', 'four-confirm159-'+str(ident), body=payload)
        record = dict(round=self.room.generation,id=ident,owner=values[2],source_role=role,
            serial=values[3],row=values[4],column=values[5],power=values[10],
            created_ns=time.monotonic_ns(),broadcast159=sent)
        self.state['round_bombs'].append(record)
        self.bomb_jobs.append((time.monotonic()+self.bomb_fuse_seconds,ident,record))
        event.setdefault('bombs',[]).append(dict(id=ident,serial=values[3],broadcast159=sent))

    def tick_bombs(self):
        if self.bomb_admission is None:
            return
        for due, ident, record in list(self.bomb_jobs):
            if time.monotonic() < due:
                continue
            self.bomb_jobs.remove((due, ident, record))
            expiry = self.bomb_admission.expire(ident)
            if 'rejected' in expiry:
                record['timer_rejected'] = expiry['rejected']
            else:
                payload = codec.pack_fields([1,ident,0,2,0xFFFFFFFF,255],[1,2,1,4,4,1])
                record['broadcast158'] = self.broadcast('blast','four-blast158-'+str(ident),body=payload)
                if self.battle_reports is not None:
                    self.battle_reports.last_blast_ns = max(z['monotonic_ns'] for z in record['broadcast158'])
            self.state['bomb_deadlines'].append(dict(id=ident,generation=record['round'],
                rejected=expiry.get('rejected'),monotonic_ns=time.monotonic_ns()))

    def receive_four_report(self, event, frame, sequence, body):
        report = dict(role=event['role'], round=self.room.generation, seq=sequence,
            monotonic_ns=time.monotonic_ns(), frame_sha256=hashlib.sha256(frame).hexdigest(),
            body_sha256=hashlib.sha256(body).hexdigest(), payload_hex=body.hex())
        self.state['event_reports'].append(report)
        if (event not in self.peers() or not event.get('peer153_sent')
                or self.state.get('membership_broken')):
            report['rejected'] = 'not_current_loaded_member'
            return
        prepared = self.battle_reports.prepare(event['role'], body, report['monotonic_ns'])
        if 'rejected' in prepared:
            report.update(prepared)
            return
        payload = prepared.pop('forward_payload')
        report.update(prepared)
        # No self reflection: each of three current peers receives a fresh frame.
        if report['forward']:
            report['forward170'] = []
            try:
                for peer in self.peers():
                    if peer is event:
                        continue
                    sent = self.send(peer, 'start',
                        f"four-event170-{report['subtype']}-r{self.room.generation}",
                        opcode=170, body=payload)
                    report['forward170'].append(dict(role=peer['role'], **sent))
            except RecipientSendFailure as error:
                report['rejected'] = 'peer_delivery_incomplete'
                report['failed_recipient_connection'] = error.recipient_ordinal
                raise
        self.battle_reports.commit(report)

    def tick_outcome(self, now):
        if self.battle_reports is None:
            return
        if self.room.active and not self.round.get('rank'):
            pending = any(self.room.fence.rejection(q['ticket']) is None
                          for q in self.bomb_admission.pending.values())
            result = self.battle_reports.outcome(now, pending)
            if result is not None:
                body = self.RANK_BODY(result['winner_team'], self.battle_reports.teams)
                # Finish fences all deferred current work before rank emission.
                self.room.finish()
                self.round['rank'] = dict(monotonic_ns=now, **result,
                    broadcast=self.broadcast('rank', 'four-rank175', body=body))
        rank = self.round.get('rank')
        if rank and not self.round.get('return') and now-rank['monotonic_ns']>=5*10**9:
            self.round['return'] = dict(monotonic_ns=now,
                broadcast=self.broadcast('return', 'four-return180'))
        returned = self.round.get('return')
        if returned and not self.round.get('refresh') and now-returned['monotonic_ns']>=3*10**9:
            sent = []
            for role in self.player_profile.ROLES:
                sent += self.broadcast('slot_B', 'four-room110-'+role, opcode=110,
                    body=self.player_profile.slot_body(self.profile.body('player_B'), self.player_profile.MEMBERS[role], chief=role=='A'))
            self.round['refresh'] = dict(monotonic_ns=now, broadcast=sent)
            for peer in self.peers():
                peer.pop('guest_ready_received', None)

    def tick_abort_room(self, now):
        job = self.abort_room_job
        if not job or job.get('completed_ns'):
            return
        peers = self.peers()
        for peer in peers:
            role = peer['role']
            if role not in job['returns']:
                job['returns'][role] = self.send(peer, 'return', 'four-abort180')
        # A post-return585 is a diagnostic setup request, not an authenticated ACK.
        # Avoid replaying old106, which would restore the departed member.
        if not all(peer['role'] in job['setup_requests'] for peer in peers):
            if now-job['created_ns'] >= 15*10**9:
                job['waiting_setup_roles'] = [p['role'] for p in peers
                                               if p['role'] not in job['setup_requests']]
            return
        chief = min((peer['role'] for peer in peers), key=lambda role:self.role_slots[role], default=None)
        for peer in peers:
            role = peer['role']
            removed = job['removed'].setdefault(role, {})
            for departed in job['departed']:
                if departed in removed:
                    continue
                member = self.player_profile.MEMBERS[departed]
                body = codec.pack_fields([member['slot'],member['identity'],0],[1,4,1])
                removed[departed] = self.send(peer,'start','four-remove107-'+departed,opcode=107,body=body)
            refreshed = job['refreshed'].setdefault(role, {})
            for other in peers:
                name = other['role']
                if name not in refreshed:
                    refreshed[name] = self.send(peer,'slot_B','four-abort-slot110-'+name,opcode=110,
                        body=self.player_profile.slot_body(self.profile.body('player_B'),self.player_profile.MEMBERS[name],chief=name==chief))
            peer.pop('guest_ready_received',None)
        job['completed_ns'] = time.monotonic_ns()
        job['chief_role'] = chief

    @recipient_safe
    def tick(self):
        now = time.monotonic_ns()
        self.tick_bombs()
        self.tick_abort_room(now)
        if self.state.get('membership_broken'):
            return
        self.tick_outcome(now)
        for event in self.events:
            if event['reply_ordinal'] != 4 or event.get('closed'):
                continue
            prev = event.get('player_response_sent')
            if prev and not event.get('slot_response_sent') and now - prev['monotonic_ns'] >= 2*10**9:
                self.send(event, 'slot_B', 'own110', opcode=110, body=self.player_profile.slot_body(
                    self.profile.body('player_B'), self.player_profile.MEMBERS[event['role']], chief=event['role']=='A'))
                event['slot_response_sent'] = dict(monotonic_ns=now)
            prev = event.get('slot_response_sent')
            if prev and not event.get('map_response_sent') and now - prev['monotonic_ns'] >= 10**9:
                for other in self.events:
                    if (other is event or other['reply_ordinal'] != 4 or other.get('closed')
                            or not other.get('player_response_sent')):
                        continue
                    self.send(event, 'player_B', 'peer106-'+other['role'], opcode=106,
                        body=self.player_profile.player_body(self.profile.body('player_B'), self.player_profile.MEMBERS[other['role']],
                            chief=other['role']=='A', ready=bool(other.get('guest_ready_received'))))
                    event.setdefault('roster_sent', []).append(other['role'])
                self.send(event, 'table', 'table238');self.send(event, 'map', 'map117')
                event['map_response_sent'] = dict(monotonic_ns=now)
        peers = self.peers()
        # New arrivals update earlier views once, without overwriting known Ready.
        for event in peers:
            for other in peers:
                if other is event or other['role'] in event.get('roster_sent', []):
                    continue
                self.send(event, 'player_B', 'late-peer106-'+other['role'], opcode=106,
                    body=self.player_profile.player_body(self.profile.body('player_B'), self.player_profile.MEMBERS[other['role']],
                        chief=other['role']=='A', ready=bool(other.get('guest_ready_received'))))
                event.setdefault('roster_sent', []).append(other['role'])
        if not self.room.active:
            return
        self.tick_loading(now)
        if not self.room.active or self.round['loading'].get('resolution') != 'barrier_complete':
            return
        members = self.peers('init99')
        if {e['role'] for e in members} != set(self.player_profile.ROLES):
            return
        endpoints = {e['role']: (int(ipaddress.IPv4Address(self.source_ips[e['role']])),
                                e['init99']['fields'][2]) for e in members}
        for event in members:
            if not event.get('go_response_sent'):
                event['go_response_sent'] = self.send(event, 'go', 'go197')
            elif not event.get('play_response_sent') and now-event['go_response_sent']['monotonic_ns'] >= 10**9:
                event['play_response_sent'] = self.send(event, 'play', 'play196')
            elif event.get('play_response_sent') and not event.get('peer153_sent') and now-event['play_response_sent']['monotonic_ns'] >= 10**9:
                event['peer153_sent'] = self.send(event, 'peer', 'peer153',
                    body=self.player_profile.peer_body(self.profile.body('peer'), self.room.key, event['role'], endpoints))

    def close_connection(self, ordinal, reason):
        state = self.connections[ordinal];event = state['event']
        was_active = self.room.active and bool(event.get('peer153_sent'))
        if event.get('closed'):
            return
        event['closed'] = dict(reason=reason, monotonic_ns=time.monotonic_ns())
        try:
            self.selector.unregister(state['socket'])
        except KeyError:
            pass
        state['socket'].close()
        if event['reply_ordinal'] == 4:
            self.room.finish()
            self.state.setdefault('membership_broken', dict(role=event['role'], reason=reason,
                monotonic_ns=time.monotonic_ns(), policy='stop_room_no_synthetic_result'))
            if self.round.get('loading'):
                self.round['loading'].setdefault('resolution', 'membership_changed')
            if self.abort_room_enabled and (was_active or (self.abort_room_job
                    and not self.abort_room_job.get('completed_ns'))):
                if self.abort_room_job is None:
                    self.abort_room_job = dict(generation=self.room.generation,
                        created_ns=time.monotonic_ns(), primary_departed=event['role'],
                        departed=[], returns={}, setup_requests={}, removed={}, refreshed={},
                        source='design', policy='active_member_loss_unranked_room_abort')
                    self.state['current_abort_room'] = self.abort_room_job
                if event['role'] not in self.abort_room_job['departed']:
                    self.abort_room_job['departed'].append(event['role'])
            # No normal rank/winner is inferred from a transport loss.


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--profile', type=Path, required=True)
    parser.add_argument('--out', type=Path, required=True)
    parser.add_argument('--bomb-profile', type=Path, help='Explicit measured/operator four-role capacity/power/fuse JSON; opt-in')
    parser.add_argument('--identity-store', type=Path, help='Opt-in exact fixed four-role identity pin across process restarts; not account auth/resume')
    parser.add_argument('--abort-room', action='store_true', help='Opt-in active four-member loss: unranked180/setup107110; requires --battle-outcome')
    parser.add_argument('--battle-outcome', action='store_true', help='Opt-in report-derived four-member 2v2 life/result/reset policy; requires --bomb-profile')
    parser.add_argument('--source-ip', action='append', required=True, metavar='ROLE=IPv4')
    parser.add_argument('--bind-ip', default='127.0.0.1')
    parser.add_argument('--port', type=int, action='append', required=True)
    parser.add_argument('--channel-ip')
    parser.add_argument('--channel-port', type=int)
    args = parser.parse_args()
    try:
        ips = addresses(args.source_ip)
        if any(not 0 <= port <= 65535 for port in args.port):
            raise ValueError('invalid listener port')
        service = FourPlayerService(FixtureProfile(args.profile), args.out, ips,
            bind_ip=args.bind_ip, channel_ip=args.channel_ip, channel_port=args.channel_port,
            bomb_profile=json.loads(args.bomb_profile.read_text()) if args.bomb_profile else None,
            battle_outcome=args.battle_outcome, abort_room=args.abort_room, identity_store=args.identity_store)
    except ValueError as error:
        parser.error(str(error))
    args.out.mkdir(parents=True, exist_ok=True)
    def stop(_signal, _frame):
        service.closed = True
    signal.signal(signal.SIGTERM, stop);signal.signal(signal.SIGINT, stop)
    service.run(args.port)


if __name__ == '__main__':
    main()
