"""Sequence/name binding regressions; native evidence remains under CP183/local."""
from pathlib import Path
import struct
import tempfile
from types import SimpleNamespace
import unittest

from server import own_profile, probe_codec as codec
from server.probe_service import ProbeService
from server.room_title import _decode_bytes, _encode_bytes


def fixtures(enabled=True):
    sequences = {'channel': 43, 'ready': 46, 'roomlist_A': 49, 'roomlist_B': 49,
                 'roomlist_C': 49, 'channel_list': 52, 'create': 55,
                 'slot_A': 61, 'guest_A': 64, 'table': 67, 'map': 70, 'start': 73}
    frames = {}
    metadata = {}
    bodies = {}
    for role, (identity, name) in own_profile.IDENTITIES.items():
        auth = bytearray(130 + len(name))
        auth[4:5] = codec.pack_fields([1], [1])
        auth[25:31] = codec.pack_fields([identity, len(name)], [4, 2])
        auth[31:31 + len(name)] = _encode_bytes(name)
        bodies['auth_' + role] = bytes(auth)
        sequences['auth_' + role] = 43
        # Deliberately retain C's historical wrong room-profile name.
        room_name = b'KRLocal' if role == 'C' else name
        inner = struct.pack('<I', len(room_name)) + room_name + bytes(98)
        stream = b'\x18' + struct.pack('<I', len(inner) + 1) + b'\0' + inner
        bodies['player_' + role] = (bytes(16) + codec.pack_fields([len(stream)], [4])
                                    + _encode_bytes(stream))
        sequences['player_' + role] = 58
    for alias in ['rank_Awin', 'promote_B', 'join_C', 'slot_C', 'guest_C', 'ready_C']:
        sequences[alias] = 73
    opcodes = {'channel': 17, 'ready': 241, 'channel_list': 14, 'create': 86,
               'table': 238, 'map': 117, 'start': 121, 'slot_A': 110, 'guest_A': 106}
    for alias, seq in sequences.items():
        opcode = (106 if alias.startswith('player_') else
                  91 if alias.startswith('roomlist_') else opcodes.get(alias, 4))
        frames[alias] = codec.encode(bytes(37), opcode, bodies.get(alias, b''), seq)
        metadata[alias] = {'sequence': seq}
    data = {'fixtures': metadata, 'items': [], 'initial_key': 12345}
    if enabled:
        data['own_profile'] = dict(own_profile.POLICY)
    return SimpleNamespace(data=data, frames=frames, raw=lambda alias: frames[alias],
                           body=lambda alias: codec.frame_body(frames[alias], metadata[alias]['sequence']))


class OwnProfileTests(unittest.TestCase):
    def service(self, enabled=True):
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        service = ProbeService(fixtures(enabled), Path(temporary.name), {})
        self.addCleanup(service.selector.close)
        event = {'role': 'A', 'global_ordinal': 4, 'reply_ordinal': 4}
        sent = []
        service.connections[4] = {'socket': SimpleNamespace(sendall=sent.append)}
        return service, event, sent

    def dispatch(self, service, event, opcode):
        service.receive_channel(event, codec.encode(bytes(37), opcode, b'', 40), 40, b'')

    def test_c_own_name_binds_to_auth_instead_of_old_room_fixture(self):
        profile = fixtures()
        prepared = own_profile.prepare(profile)
        for role, expected in [('A', b'KRLocal'), ('B', b'KRGuest'), ('C', b'KRNew')]:
            body = prepared[role]
            size = codec.read_fields(body[1:5], [4])[0]
            stream = _decode_bytes(body[5:5 + size])
            inner = stream[6:]
            self.assertEqual(struct.unpack_from('<I', inner)[0], len(expected))
            self.assertEqual(inner[4:4 + len(expected)], expected)
            self.assertEqual(inner[4 + len(expected):], bytes(98))
        self.assertEqual([len(prepared[r]) for r in 'ABC'], [128, 128, 126])

    def test_initial_setup_dynamic_start_and_reentry_share_one_sequence(self):
        service, event, sent = self.service()
        self.dispatch(service, event, 6)
        self.dispatch(service, event, 67)
        self.dispatch(service, event, 81)
        self.dispatch(service, event, 585)
        service.send_setup(event, ['slot_A', 'guest_A', 'table', 'map'], 'loading')
        service.send(event, 'start', 'start')
        event['room_reentry'] = True
        service.send_setup(event, ['create'], 'rejoin')
        self.assertEqual(len(sent), 13)
        self.assertEqual([struct.unpack_from('<I', raw, 5)[0] for raw in sent],
                         [17, 241, 91, 14, 291, 86, 106, 110, 106, 238, 117, 121, 86])
        for seq, raw in zip(range(43, 82, 3), sent):
            codec.frame_body(raw, seq)
            with self.assertRaises(ValueError):
                codec.frame_body(raw, seq - 3)
        self.assertEqual(event['s2c_next_seq'], 82)
        before = len(sent)
        self.dispatch(service, event, 6)
        self.dispatch(service, event, 67)
        self.assertEqual(len(sent), before)

    def test_default_off_preserves_raw_initial_fixtures(self):
        service, event, sent = self.service(False)
        for opcode in [6, 67, 81, 585]:
            self.dispatch(service, event, opcode)
        expected = [service.profile.raw('channel'),
                    b''.join(service.profile.raw(a) for a in ['ready', 'roomlist_A', 'channel_list']),
                    service.profile.raw('create'), service.profile.raw('player_A')]
        self.assertEqual(sent, expected)
        self.assertNotIn('s2c_next_seq', event)

    def test_wrong_stage_and_send_failure_do_not_commit_ready_marker(self):
        service, event, sent = self.service()
        for opcode in [67, 81, 585, 1]:
            self.dispatch(service, event, opcode)
        self.assertEqual(sent, [])
        self.assertEqual(len(service.state['stage_rejections']), 4)
        self.assertEqual([x['reason'] for x in service.state['stage_rejections']],
                         ['missing_channel_response_sent', 'missing_ready_response_sent',
                          'missing_create_response_sent', 'login_opcode_on_channel'])
        self.assertNotIn('ready_response_sent', event)
        self.dispatch(service, event, 6)
        def fail_own(raw):
            if struct.unpack_from('<I', raw, 5)[0] == 291:
                raise OSError('test own-profile transport failure')
            sent.append(raw)
        service.connections[4]['socket'].sendall = fail_own
        with self.assertRaises(OSError):
            self.dispatch(service, event, 67)
        self.assertNotIn('ready_response_sent', event)
        self.assertEqual(event['s2c_next_seq'], 55)

    def test_valid_frames_on_wrong_login_stage_are_logged_without_reply(self):
        service, _, sent = self.service()
        event = {'role': 'A', 'global_ordinal': 3, 'reply_ordinal': 3,
                 'chunks': []}
        wire = b''.join(codec.encode(bytes(37), op, b'', seq)
                        for op, seq in [(67, 40), (9, 43), (5, 46), (528, 49)])
        service.connections[3] = {
            'socket': SimpleNamespace(recv=lambda _: wire), 'event': event,
            'buffer': bytearray(), 'sequence': 40}
        service.receive(3)
        self.assertEqual(sent, [])
        self.assertEqual(service.connections[3]['sequence'], 52)
        self.assertNotIn('auth_response_sent', event)
        self.assertEqual([q['reason'] for q in service.state['stage_rejections']],
                         ['channel_opcode_on_login', 'missing_auth_response_sent',
                          'missing_netinfo_response_sent', 'channel_opcode_on_login'])

    def test_cache_request_is_stage_guarded_and_shares_create_sequence(self):
        service, event, sent = self.service()
        self.dispatch(service, event, 528)
        self.assertEqual(sent, [])
        self.assertEqual(service.state['stage_rejections'][-1]['reason'],
                         'missing_ready_response_sent')
        self.dispatch(service, event, 6)
        self.dispatch(service, event, 67)
        self.dispatch(service, event, 528)
        self.assertEqual(struct.unpack_from('<I', sent[-1], 5)[0], 238)
        self.assertEqual(codec.frame_body(sent[-1], 58), service.profile.body('table'))
        self.assertEqual(event['s2c_next_seq'], 61)
        self.assertEqual(len(event['cache_responses']), 1)
        self.dispatch(service, event, 81)
        self.assertEqual(struct.unpack_from('<I', sent[-1], 5)[0], 86)
        codec.frame_body(sent[-1], 61)
        self.assertEqual(event['s2c_next_seq'], 64)

    def test_cache_request_unknown_body_and_legacy_default_are_unchanged(self):
        service, event, sent = self.service()
        self.dispatch(service, event, 6)
        self.dispatch(service, event, 67)
        before = len(sent)
        service.receive_channel(event, codec.encode(bytes(37), 528, b'x', 40), 40, b'x')
        self.assertEqual(len(sent), before)
        self.assertNotIn('cache_responses', event)
        self.assertEqual(event['s2c_next_seq'], 58)
        self.assertEqual(service.state['stage_rejections'][-1]['reason'],
                         'unsupported_cache_request_body')
        legacy, old_event, old_sent = self.service(False)
        self.dispatch(legacy, old_event, 528)
        self.assertEqual(old_sent, [])

    def test_bad_policy_identity_or_unmeasured_profile_fails_before_listening(self):
        profile = fixtures()
        profile.data['own_profile'] = {'mode': 'guess'}
        with self.assertRaisesRegex(ValueError, 'policy'):
            own_profile.prepare(profile)
        for kind in ['identity', 'profile']:
            profile = fixtures()
            alias = 'auth_C' if kind == 'identity' else 'player_A'
            body = bytearray(profile.body(alias))
            if kind == 'identity':
                body[25:29] = codec.pack_fields([2], [4])
            else:
                stream = bytearray(_decode_bytes(body[20:]))
                stream[-1] = 1
                body[20:] = _encode_bytes(stream)
            seq = profile.data['fixtures'][alias]['sequence']
            profile.frames[alias] = codec.encode(profile.frames[alias][:37],
                                                 4 if kind == 'identity' else 106, body, seq)
            with self.assertRaises(ValueError):
                own_profile.prepare(profile)


if __name__ == '__main__':
    unittest.main()
