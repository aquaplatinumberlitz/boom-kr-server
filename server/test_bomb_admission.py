"""Measured-profile admission policy; native runtime proof lives in evidence."""
from pathlib import Path
from types import SimpleNamespace
import tempfile
import unittest

from server import probe_codec as codec
from server.probe_service import ProbeService


class BombAdmissionTests(unittest.TestCase):
    def setUp(self):
        tmp = tempfile.TemporaryDirectory()
        self.addCleanup(tmp.cleanup)
        aliases = ['rank_Awin', 'promote_B', 'auth_C', 'roomlist_C', 'join_C',
                   'player_C', 'slot_C', 'guest_C', 'ready_C']
        profile = SimpleNamespace(frames=dict.fromkeys(aliases, b''), data={
            'items': [dict(index=17, row=4, column=3, kind=2)], 'initial_key': 12345})
        self.service = ProbeService(profile, Path(tmp.name), {})
        self.addCleanup(self.service.selector.close)
        self.a = dict(role='A', reply_ordinal=4, map_response_sent=True)
        self.b = dict(role='B', reply_ordinal=4, map_response_sent=True)
        self.service.events = [self.a, self.b]
        self.service.channels = dict(A=self.a, B=self.b)
        self.sent = []
        def send(event, alias, label, **kwargs):
            result = dict(role=event['role'], alias=alias, label=label,
                          monotonic_ns=1, **kwargs)
            self.sent.append(result)
            return result
        self.service.send = send
        self.service.start_round()
        self.sent.clear()

    def bomb(self, *, event=None, key=None, serial=0, row=6, owner=None, flag=0, sequence=100):
        event = self.a if event is None else event
        owner = (0 if event is self.a else 1) if owner is None else owner
        key = self.service.room.key if key is None else key
        values = [key, 0, owner, serial, row, 3, 1, 0, 0, 0, 1, 1, flag]
        body = codec.pack_fields(values, [4, 1, 1, 4, 1, 1, 1, 1, 1, 1, 1, 4, 1])
        frame = codec.encode(bytes(37), 118, body, sequence)
        self.service.receive_channel(event, frame, sequence, body)
        return self.service.state['bomb_requests'][-1]

    def assert_one_bomb(self):
        self.assertEqual(len(self.service.state['round_bombs']), 1)
        self.assertEqual(self.service.room._wire_id, 1)
        self.assertEqual(len(self.service.jobs), 1)
        self.assertEqual(len(self.sent), 2)

    def test_same_body_at_new_sequence_does_not_create_id_timer_or_confirmation(self):
        first = self.bomb()
        duplicate = self.bomb(sequence=103)
        self.assertNotEqual(first['frame_sha256'], duplicate['frame_sha256'])
        self.assertEqual(first['body_sha256'], duplicate['body_sha256'])
        self.assertEqual(duplicate['rejected'], 'duplicate_bomb_serial')
        self.assertEqual(duplicate['prior_id'], 1)
        self.assert_one_bomb()

    def test_old_key_is_rejected_before_reserving_serial_in_new_round(self):
        self.bomb()
        self.service.room.finish()
        self.service.start_round()
        self.sent.clear()
        self.assertNotIn('bomb_serials', self.a)
        for key in [12345, 12346 + 256]:
            self.assertEqual(self.bomb(key=key)['rejected'], 'bomb_round_key_mismatch')
            self.assertNotIn('bomb_serials', self.a)
        self.assertEqual(self.service.room._wire_id, 1)
        self.assertEqual(self.sent, [])
        # The same bound connection can legitimately use serial0 after Start.
        self.assertEqual(self.bomb()['id'], 2)
        self.assertEqual(len(self.sent), 2)
        self.assertEqual(len(self.service.jobs), 2)  # R1 job retained with its old fence ticket.
        self.assertIsNotNone(self.service.room.fence.rejection(self.service.jobs[0][1]))

    def test_conflicting_same_serial_does_not_replace_the_original(self):
        first = self.bomb()
        conflict = self.bomb(row=7)
        self.assertNotEqual(first['body_sha256'], conflict['body_sha256'])
        self.assertEqual(conflict['rejected'], 'conflicting_bomb_serial')
        self.assertEqual(self.a['bomb_serials'][0]['body_sha256'], first['body_sha256'])
        self.assert_one_bomb()

    def test_serial_is_scoped_to_bound_owner_and_new_serials_still_work(self):
        self.assertEqual(self.bomb()['id'], 1)
        self.assertEqual(self.bomb(event=self.b)['id'], 2)
        self.assertEqual(self.bomb(serial=28)['id'], 3)
        self.assertEqual([q['owner'] for q in self.service.state['round_bombs']], [0, 1, 0])
        self.assertEqual(len(self.service.jobs), 3)
        self.assertEqual(len(self.sent), 6)

    def test_serial_is_not_a_monotonic_tick_or_cross_round_nonce(self):
        self.assertEqual(self.bomb(serial=0xFFFFFFFF)['id'], 1)
        self.assertEqual(self.bomb(serial=0)['id'], 2)
        self.assertEqual(self.bomb(serial=0xFFFFFFFF)['rejected'], 'duplicate_bomb_serial')
        self.assertEqual(self.service.room._wire_id, 2)

    def test_invalid_slot_or_flag_does_not_reserve_id_or_serial(self):
        for kwargs in [dict(owner=1), dict(flag=1)]:
            with self.subTest(kwargs=kwargs), self.assertRaisesRegex(ValueError, 'bomb report'):
                self.bomb(**kwargs)
        self.assertNotIn('bomb_serials', self.a)
        self.assertEqual(self.service.room._wire_id, 0)
        self.assertEqual(self.service.jobs, [])
        self.assertEqual(self.sent, [])
        self.assertEqual(self.bomb()['id'], 1)

    def test_current_key_and_unseen_serial_do_not_establish_actor_authority(self):
        # This admission policy has no position/cap/fuse simulator or sender auth.
        # A forged but structurally valid new request remains outside its scope.
        self.assertEqual(self.bomb(serial=900, row=12)['id'], 1)
        self.assert_one_bomb()


if __name__ == '__main__':
    unittest.main()
