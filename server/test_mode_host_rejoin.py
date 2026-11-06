"""Current selector policy plus existing promotion guards; not a native oracle."""
import unittest

from server import probe_codec as codec
from server.probe_service import ProbeService
from server import test_host_back_rejoin as host
from server.test_room_team_mode import ack86, ack87


class ModeHostRejoinTests(unittest.TestCase):
    member = host.HostBackRejoinTests.member
    dispatch = host.HostBackRejoinTests.dispatch
    tick = host.HostBackRejoinTests.tick
    leave = host.HostBackRejoinTests.leave
    rejoin = host.HostBackRejoinTests.rejoin
    test_refresh_pending_create_and_lobby_ready_do_not_restore_member = (
        host.HostBackRejoinTests.test_refresh_pending_create_and_lobby_ready_do_not_restore_member)
    test_changed_survivor_identity_blocks_old_membership_rejoin = (
        host.HostBackRejoinTests.test_changed_survivor_identity_blocks_old_membership_rejoin)
    test_rejoin_is_currentseq_guest_a_chief_b_then_ready_a_start_b = (
        host.HostBackRejoinTests.test_rejoin_is_currentseq_guest_a_chief_b_then_ready_a_start_b)

    def setUp(self):
        host.HostBackRejoinTests.setUp(self)
        send = self.service.send
        self.service.selector.close()
        self.bodies = {'create': ack86(), 'join': ack87(),
                       'join_A': codec.pack_fields([0, 0, 1, 0, 0, 0], [1, 1, 2, 1, 1, 1]),
                       'return': codec.pack_fields([0, 0, 0], [1, 1, 1])}
        self.opcodes = {'create': 86, 'join': 87, 'join_A': 87, 'return': 180}
        self.profile.data['room_team_mode'] = {'mode': 'create81_to_86_87'}
        self.profile.data['fixtures'] = {name: {'sequence': 55} for name in self.profile.frames}
        self.profile.body = lambda name: self.bodies.get(name, b'fixture')
        self.profile.raw = lambda name: codec.encode(bytes(37), self.opcodes.get(name, 0),
                                                      self.profile.body(name), 55)
        self.service = ProbeService(self.profile, self.out, {})
        self.addCleanup(self.service.selector.close)
        self.service.events.extend([self.a, self.b])
        self.service.channels.update(A=self.a, B=self.b)
        self.service.send = send
        self.service.send_raw = lambda *_: self.fail('reentry must use current sequence')
        self.service.room_team_selector = 1

    def test_promoted_join_echoes_selector_preserving_identity_and_opaque_bytes(self):
        for value in (0, 1):
            with self.subTest(selector=value):
                self.service.room_team_selector = value
                # Exercise only one leave per service; later dispatch uses the
                # same current membership proof and a cleared setup marker.
                if not self.a.get('last_room_leave'):
                    self.leave()
                    self.tick(11)
                self.a.pop('create_response_sent', None)
                before = self.a['s2c_next_seq']
                self.dispatch(self.a, 82)
                sent = self.sent[-1]
                self.assertEqual((sent['alias'], sent['seq']), ('join_A', before))
                body = sent['body']
                self.assertEqual(body[:5], self.bodies['join_A'][:5])
                self.assertEqual(body[6:], self.bodies['join_A'][6:])
                self.assertEqual(codec.read_fields(body, [1, 1, 2, 1, 1, 1]), [0, 0, 1, 0, value, 0])
                self.assertEqual(self.a['s2c_next_seq'], before + 3)
                self.assertEqual(self.service.room_team_selector, value)
                self.assertEqual(codec.read_fields(self.bodies['join_A'], [1, 1, 2, 1, 1, 1]), [0, 0, 1, 0, 0, 0])

    def test_missing_current_selector_keeps_promoted_join_rejected(self):
        self.leave()
        self.tick(11)
        self.service.room_team_selector = None
        self.dispatch(self.a, 82)
        self.assertNotIn('create_response_sent', self.a)
        self.assertFalse(any(q['alias'] == 'join_A' for q in self.sent))
        self.assertEqual(self.service.state['room_team_rejections'][-1]['reason'],
                         'room team mode requires current create81')

    def test_wrong_host_or_closed_survivor_keeps_membership_guard(self):
        self.leave()
        self.tick(11)
        for key, value in [('host_role', 'A'), ('closed', True)]:
            with self.subTest(key=key):
                if key == 'host_role':
                    self.service.host_role = value
                else:
                    self.service.host_role = 'B'
                    self.b['closed'] = value
                self.dispatch(self.a, 82)
                self.assertNotIn('create_response_sent', self.a)
                self.assertEqual(self.service.state['room_reentry_rejections'][-1]['reason'],
                                 'host_membership_changed')
        self.assertFalse(any(q['alias'] == 'join_A' for q in self.sent))

    def test_failed_promoted_send_does_not_commit_ack_mode_or_sequence(self):
        self.leave()
        self.tick(11)
        before = self.a['s2c_next_seq']
        self.service.send = ProbeService.send.__get__(self.service)
        def fail(*_):
            raise OSError('failed promoted send')
        self.service.send_raw = fail
        with self.assertRaisesRegex(OSError, 'failed promoted send'):
            self.dispatch(self.a, 82)
        self.assertNotIn('create_response_sent', self.a)
        self.assertNotIn('promoted_rejoin', self.a)
        self.assertNotIn('room_team_replies', self.service.state)
        self.assertEqual(self.a['s2c_next_seq'], before)
        self.assertEqual(self.service.room_team_selector, 1)

    def test_promoted_mode_fixture_fails_preflight_on_shape_branch_or_opcode(self):
        original = self.bodies['join_A']
        for body in (original[:-1], original + b'x', codec.pack_fields([1], [1]) + original[1:]):
            with self.subTest(size=len(body)):
                self.bodies['join_A'] = body
                with self.assertRaises(ValueError):
                    ProbeService(self.profile, self.out, {})
        self.bodies['join_A'] = original
        self.opcodes['join_A'] = 86
        with self.assertRaisesRegex(ValueError, 'promoted room team fixture opcode'):
            ProbeService(self.profile, self.out, {})


if __name__ == '__main__':
    unittest.main()
