"""Policy regression; actual native/UI evidence is stored separately."""
import unittest
from server import probe_codec as c
from server import room_team_mode as mode
from server import test_room_title as title_tests
from server.probe_service import ProbeService


def create_body(value):
    body = title_tests.create_body(b'Room')
    return body[:12] + c.pack_fields([value], [4]) + body[16:]


def ack86(value=0):
    return c.pack_fields([0, 1, 77, value, 8, 9, 0], [1, 2, 4, 1, 1, 1, 1])


def ack87(value=0):
    return c.pack_fields([0, 1, 1, 8, value, 0], [1, 1, 2, 1, 1, 1])


class TeamModeTests(unittest.TestCase):
    def test_selector_is_only_known_second_u32(self):
        for value in (0, 1):
            self.assertEqual(mode.create_selector(create_body(value)), value)
        for body in [create_body(2), create_body(0)[:-1], create_body(0) + b'x']:
            with self.assertRaises(ValueError):
                mode.create_selector(body)

    def test_only_native_confirmed_ack_byte_changes(self):
        for op, original, offset in [(86, ack86(), 7), (87, ack87(), 5), (180, c.pack_fields([0, 0, 0], [1, 1, 1]), 1)]:
            rewritten = mode.replace_reply_selector(original, op, 1)
            self.assertEqual(rewritten[:offset], original[:offset])
            self.assertEqual(rewritten[offset+1:], original[offset+1:])
            self.assertEqual(c.read_fields(rewritten[offset:offset+1], [1]), [1])
        for args in [(ack86()[:-1], 86, 1), (ack87(), 86, 1), (ack86(2), 86, 1),
                     (ack86(), 86, 2), (c.pack_fields([1], [1])+ack86()[1:], 86, 1)]:
            with self.assertRaises(ValueError):
                mode.replace_reply_selector(*args)


class TeamServiceTests(unittest.TestCase):
    create = title_tests.TitleServiceTests.create

    def setUp(self):
        title_tests.TitleServiceTests.setUp(self)
        self.bodies.update(create=ack86(), join=ack87(), **{'return': c.pack_fields([0, 0, 0], [1, 1, 1])})
        self.profile.data['fixtures']['join'] = {'sequence': 55}
        self.profile.data['fixtures']['return'] = {'sequence': 55}
        self.profile.data['room_team_mode'] = {'mode': 'create81_to_86_87'}
        old_raw = self.profile.raw
        self.profile.raw = lambda name: (c.encode(bytes(37), {'join': 87, 'return': 180}[name], self.bodies[name], 55)
                                         if name in ('join', 'return') else old_raw(name))
        self.service.selector.close()
        self.service = ProbeService(self.profile, self.out, {})
        self.addCleanup(self.service.selector.close)
        self.service.send_raw = lambda event, frame: self.sent.append((event['role'], frame))

    def test_native_create_ack_and_guest_join_ack_get_same_selector(self):
        self.create(create_body(1))
        self.assertEqual(self.service.room_team_selector, 1)
        raw = self.sent[-1][1]
        self.assertEqual(c.frame_body(raw, 55), mode.replace_reply_selector(ack86(), 86, 1))
        body = b'known-join'
        self.service.receive_channel(self.b, c.encode(bytes(37), 82, body, 82), 82, body)
        self.assertEqual(c.frame_body(self.sent[-1][1], 55), mode.replace_reply_selector(ack87(), 87, 1))
        self.assertEqual([q['selector'] for q in self.service.state['room_team_replies']], [1, 1])
        self.assertEqual(self.bodies['create'], ack86())
        self.assertEqual(self.bodies['join'], ack87())

    def test_failed_send_commits_neither_mode_nor_title(self):
        self.service.room_team_selector = 0
        self.service.room_title_bytes = b'Old'
        def fail(*_):
            raise OSError('test failure')
        self.service.send_raw = fail
        with self.assertRaises(OSError):
            self.create(create_body(1))
        self.assertEqual(self.service.room_team_selector, 0)
        self.assertEqual(self.service.room_title_bytes, b'Old')
        self.assertNotIn('room_team_updates', self.service.state)
        self.assertNotIn('create_response_sent', self.a)

    def test_bad_selector_commits_neither_ack_nor_title(self):
        self.create(create_body(2))
        self.assertEqual(self.sent, [])
        self.assertIsNone(self.service.room_team_selector)
        self.assertIsNone(self.service.room_title_bytes)
        self.assertNotIn('create_response_sent', self.a)
        self.assertEqual(len(self.service.state['room_team_rejections']), 1)

    def test_join_without_current_create_is_rejected(self):
        body = b'known-join'
        self.service.receive_channel(self.b, c.encode(bytes(37), 82, body, 82), 82, body)
        self.assertEqual(self.sent, [])
        self.assertNotIn('create_response_sent', self.b)

    def test_reentry_ack_keeps_continuing_sequence(self):
        self.a.update(room_reentry=True, s2c_next_seq=103)
        self.create(create_body(1))
        self.assertEqual(c.frame_body(self.sent[-1][1], 103), mode.replace_reply_selector(ack86(), 86, 1))
        self.assertEqual(self.a['s2c_next_seq'], 106)

    def test_mode_off_keeps_default_ack(self):
        self.service.room_team_mode = False
        self.create(create_body(1))
        self.assertEqual(c.frame_body(self.sent[-1][1], 55), ack86())
        self.assertIsNone(self.service.room_team_selector)

    def test_return180_retains_selector_at_current_sequence(self):
        self.create(create_body(1))
        self.b['s2c_next_seq'] = 109
        self.service.send(self.b, 'return', 'test-return')
        self.assertEqual(c.read_fields(c.frame_body(self.sent[-1][1], 109), [1, 1, 1]), [0, 1, 0])
        self.assertEqual(self.b['s2c_next_seq'], 112)
        for fields in ([1, 0, 0], [0, 0, 1]):
            with self.assertRaises(ValueError):
                mode.replace_reply_selector(c.pack_fields(fields, [1, 1, 1]), 180, 1)

    def test_bad_fixture_fails_before_listening(self):
        self.bodies['join'] = ack87()[:-1]
        with self.assertRaisesRegex(ValueError, 'shape'):
            ProbeService(self.profile, self.out, {})


if __name__ == '__main__':
    unittest.main()
