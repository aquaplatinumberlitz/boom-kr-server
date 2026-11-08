"""Policy regressions with synthetic frames; native acceptance is separate."""
from pathlib import Path
import socket
import struct
import tempfile
from types import SimpleNamespace
import unittest

from server import four_player_profile as fp, probe_codec as codec
from server.four_player_service import FourPlayerService, addresses
from server.room_title import _encode_bytes, _decode_bytes


def fixtures():
    names = ['rank_Awin', 'promote_B', 'auth_C', 'roomlist_C', 'join_C',
             'player_C', 'slot_C', 'guest_C', 'ready_C', 'bootstrap', 'second',
             'auth_A', 'player_B', 'join', 'start', 'channel', 'ready',
             'roomlist_A', 'roomlist_B', 'channel_list', 'slot_B', 'table',
             'map', 'floor', 'go', 'play', 'peer', 'netinfo', 'flags', 'redirect']
    bodies = dict.fromkeys(names, b'')
    inner = struct.pack('<I', 7) + b'KRGuest' + bytes(98)
    stream = b'\x18' + struct.pack('<I', len(inner) + 1) + b'\0' + inner
    bodies['player_B'] = (codec.pack_fields([1, 2, 0, 0, 0, 0], [1, 4, 1, 1, 1, 4])
        + _encode_bytes(struct.pack('<I', 32)) + codec.pack_fields([len(stream)], [4])
        + _encode_bytes(stream) + codec.pack_fields([0, 0, 0, 7], [1, 1, 1, 2])
        + _encode_bytes(b'KRGuest') + bytes(32))
    auth = bytearray(137)
    auth[4:5] = codec.pack_fields([1], [1])
    auth[25:31] = codec.pack_fields([1, 7], [4, 2])
    auth[31:38] = _encode_bytes(b'KRLocal')
    bodies['auth_A'] = bytes(auth)
    bodies['join'] = codec.pack_fields([0, 1, 1, 0, 0, 0], [1, 1, 2, 1, 1, 1])
    inner = bytearray(100);inner[43] = 2;inner[74] = 1
    stream = b'\x18' + struct.pack('<I', 101) + b'\0' + inner
    bodies['start'] = (codec.pack_fields([0, len(stream)], [1, 4])
                       + _encode_bytes(stream) + bytes(18))
    bodies['peer'] = bytes(97)
    frames = {name: codec.encode(bytes(37), 106 if name == 'player_B' else 4,
                                 bodies[name], 43) for name in names}
    return SimpleNamespace(data={'items': [], 'initial_key': 12345}, frames=frames,
                           body=lambda name: bodies[name], raw=lambda name: frames[name])


class FourPlayerTests(unittest.TestCase):
    IPS = {role: '127.0.0.' + str(2+i) for i, role in enumerate(fp.ROLES)}

    def service(self):
        tmp = tempfile.TemporaryDirectory();self.addCleanup(tmp.cleanup)
        service = FourPlayerService(fixtures(), Path(tmp.name), self.IPS)
        self.addCleanup(service.selector.close)
        return service

    def members(self, service):
        sent = {r: [] for r in fp.ROLES}
        for i, role in enumerate(fp.ROLES):
            event = dict(role=role, reply_ordinal=4, global_ordinal=i+1,
                         s2c_next_seq=43, map_response_sent={'monotonic_ns': 1})
            service.events.append(event);service.channels[role] = event
            service.connections[i+1] = {'event': event,
                'socket': SimpleNamespace(sendall=sent[role].append, close=lambda: None, fileno=lambda: 12345)}
        return sent

    def dispatch(self, service, role, opcode, body=b''):
        frame = codec.encode(bytes(37), opcode, body, 40)
        service.receive_channel(service.channels[role], frame, 40, body)

    def test_names_bind_auth_own_and_both_roster_fields(self):
        profile = fixtures()
        for role, member in fp.MEMBERS.items():
            body = fp.player_body(profile.body('player_B'), member, chief=role=='A')
            size = codec.read_fields(body[16:20], [4])[0]
            stream = _decode_bytes(body[20:20+size]);n = struct.unpack_from('<I', stream, 6)[0]
            self.assertEqual(stream[10:10+n], member['name'].encode())
            tail = body[20+size:];m = codec.read_fields(tail[3:5], [2])[0]
            self.assertEqual(_decode_bytes(tail[5:5+m]), member['name'].encode())
            self.assertEqual(codec.read_fields(body[:5], [1, 4]), [member['slot'], member['identity']])
            auth = fp.auth_body(profile.body('auth_A'), member)
            self.assertEqual(codec.read_fields(auth[25:29], [4]), [member['identity']])
        # Refuse to silently turn an unknown nonzero profile into the known branch.
        bad = bytearray(profile.body('player_B'));size = codec.read_fields(bad[16:20], [4])[0]
        stream = bytearray(_decode_bytes(bad[20:20+size]));stream[-1] = 1
        bad[20:20+size] = _encode_bytes(stream)
        with self.assertRaisesRegex(ValueError, 'zero-profile'):
            fp.player_body(bytes(bad), fp.MEMBERS['D'])

    def test_invalid_source_mapping_is_rejected(self):
        for options in [[], ['A=127.0.0.2'], ['A=127.0.0.2']*4,
                        [r+'=127.0.0.2' for r in fp.ROLES],
                        ['A=::1','B=127.0.0.3','C=127.0.0.4','D=127.0.0.5']]:
            with self.subTest(options=options), self.assertRaises(ValueError):
                addresses(options)

    def test_real_interleaved_transport_capacity_and_unknown_source(self):
        service = self.service();listener = socket.socket();self.addCleanup(listener.close)
        listener.bind(('127.0.0.1',0));listener.listen(20)
        for role in 'DCBADCBADCBADCBA':
            client = socket.socket();self.addCleanup(client.close)
            client.settimeout(1);client.bind((self.IPS[role],0));client.connect(listener.getsockname())
            service.accept(listener);self.assertEqual(client.recv(4096),service.profile.raw('bootstrap'))
            self.assertEqual(service.events[-1]['role'],role)
        self.assertEqual(service.source_counts,dict.fromkeys(fp.ROLES,4))
        for address in [self.IPS['D'],'127.0.0.9']:
            client = socket.socket();self.addCleanup(client.close);client.settimeout(1)
            client.bind((address,0));client.connect(listener.getsockname());service.accept(listener)
            self.assertEqual(client.recv(4096),b'')
        self.assertEqual(len(service.events),16)
        for connection in service.connections.values():connection['socket'].close()

    def test_login_request1_returns_KR_reply4_header(self):
        service = self.service();listener = socket.socket();self.addCleanup(listener.close)
        listener.bind(('127.0.0.1',0));listener.listen(4)
        for stage in range(1,4):
            client = socket.socket();self.addCleanup(client.close);client.settimeout(1)
            client.bind((self.IPS['D'],0));client.connect(listener.getsockname())
            service.accept(listener);client.recv(4096)
            if stage == 3:
                client.sendall(codec.encode(bytes(37),1,b'',40))
                service.receive(service.events[-1]['global_ordinal'])
                reply = client.recv(4096)
                self.assertEqual(struct.unpack_from('<I',reply,5)[0],4)
                body = codec.frame_body(reply,43)
                self.assertEqual(codec.read_fields(body[25:29],[4]),[4])
        for connection in service.connections.values():connection['socket'].close()

    def test_start_requires_four_members_and_each_guest_ready(self):
        service = self.service();self.members(service)
        self.dispatch(service,'A',104);self.assertFalse(service.room.active)
        for role in 'BC':service.channels[role]['guest_ready_received']={'monotonic_ns':1}
        self.dispatch(service,'A',104);self.assertFalse(service.room.active)
        self.dispatch(service,'D',95,b'')
        self.assertNotIn('guest_ready_received',service.channels['D'])
        service.channels['D']['guest_ready_received']={'monotonic_ns':1}
        self.dispatch(service,'B',104);self.assertFalse(service.room.active)
        self.dispatch(service,'A',104);self.assertTrue(service.room.active)
        self.assertEqual(len(service.round['roster']),4)
        # Duplicate Start must not consume a second key or generation.
        self.dispatch(service,'A',104);self.assertEqual(service.room.generation,1)

    def test_no_go_until_fourth99_and_endpoint_source_is_operator(self):
        service = self.service();self.members(service)
        for role in 'BCD':service.channels[role]['guest_ready_received']={'monotonic_ns':1}
        self.dispatch(service,'A',104)
        for role in 'ABC':
            self.dispatch(service,role,99,codec.pack_fields(
                [0x01020304,0,24000+fp.MEMBERS[role]['slot'],0,0,0,0,0,0],
                [4,4,2,4,2,1,1,1,4]))
        service.tick()
        self.assertFalse(any(e.get('go_response_sent') for e in service.peers()))
        self.dispatch(service,'D',99,codec.pack_fields([0x01020304,0,24003,0,0,0,0,0,0],
                                                     [4,4,2,4,2,1,1,1,4]))
        service.tick();self.assertTrue(all(e.get('go_response_sent') for e in service.peers()))
        for role in fp.ROLES:
            service.channels[role]['go_response_sent']['monotonic_ns']=1
        service.tick()
        for role in fp.ROLES:
            service.channels[role]['play_response_sent']['monotonic_ns']=1
        service.tick()
        for role,e in service.channels.items():
            meta=e['peer153_sent'];body=codec.frame_body((service.out/meta['fixture']).read_bytes(),meta['seq'])
            self.assertEqual(len(body),133)
            for slot,other in enumerate(fp.ROLES):
                offset=36+19*slot
                values=codec.read_fields(body[offset:offset+18],[4,2]*3)
                expected=[int(__import__('ipaddress').IPv4Address(self.IPS[other])),24000+slot,0,0,0,0]
                self.assertEqual(values,[0]*6 if other==role else expected)

    def test_departure_and_unsupported_battle_never_choose_AB_rank(self):
        service = self.service();self.members(service)
        for role in 'BCD':service.channels[role]['guest_ready_received']={'monotonic_ns':1}
        self.dispatch(service,'A',104);self.dispatch(service,'D',118,b'')
        self.assertEqual(service.state['bomb_requests'],[])
        service.close_connection(service.channels['C']['global_ordinal'],'test_eof')
        self.assertFalse(service.room.active);service.tick()
        self.assertNotIn('rank',service.round)
        self.assertEqual(service.state['membership_broken']['role'],'C')


if __name__ == '__main__':
    unittest.main()
