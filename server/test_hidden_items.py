"""Report-derived fixture policy; no claim of native or server authority."""
from pathlib import Path
from types import SimpleNamespace
import tempfile
import unittest

from server import probe_codec as codec
from server.probe_service import ProbeService


class HiddenItemPolicyTests(unittest.TestCase):
    def profile(self, hidden=None):
        aliases = ['rank_Awin', 'promote_B', 'auth_C', 'roomlist_C', 'join_C',
                   'player_C', 'slot_C', 'guest_C', 'ready_C']
        return SimpleNamespace(frames=dict.fromkeys(aliases, b''), data={
            'items': [dict(index=17, row=4, column=3, kind=2)], 'initial_key': 12345,
            'hidden_items': hidden if hidden is not None else [
                dict(index=27, row=2, column=3, kind=0), dict(index=28, row=3, column=2, kind=1)]})

    def setUp(self):
        tmp = tempfile.TemporaryDirectory()
        self.addCleanup(tmp.cleanup)
        self.out = Path(tmp.name)
        self.s = ProbeService(self.profile(), self.out, {})
        self.addCleanup(self.s.selector.close)
        self.a = dict(role='A', reply_ordinal=4, map_response_sent=True)
        self.b = dict(role='B', reply_ordinal=4, map_response_sent=True)
        self.s.events = [self.a, self.b]
        self.s.channels = dict(A=self.a, B=self.b)
        self.sent = []
        def send(event, alias, label, **kwargs):
            result = dict(role=event['role'], alias=alias, label=label, monotonic_ns=1, **kwargs)
            self.sent.append(result)
            return result
        self.s.send = send
        self.s.start_round()
        self.sent.clear()

    def pickup(self, index=27, *, key=None):
        item = {17: (4, 3, 2), 27: (2, 3, 0), 28: (3, 2, 1)}[index]
        body = codec.pack_fields([0, 0, item[0], item[1], index, item[2], 100,
                                  self.s.room.key if key is None else key, 0],
                                 [1, 1, 1, 1, 4, 2, 4, 4, 1])
        self.s.receive_channel(self.a, codec.encode(bytes(37), 124, body, 100), 100, body)
        return self.s.state['item_requests'][-1]

    def report(self, row=2, col=3, *, event=None, slot=0, aux=0xFFFFFFFF):
        body = codec.pack_fields([row, col, slot, aux], [1, 1, 1, 4])
        self.s.receive_channel(self.a if event is None else event,
                               codec.encode(bytes(37), 142, body, 103), 103, body)
        return self.s.state['block_reports'][-1]

    def emitted_blast(self, generation=None):
        self.s.state['round_bombs'].append(dict(round=self.s.room.generation if generation is None else generation,
                                                broadcast158=[dict(monotonic_ns=1)]))

    def test_hidden_pickup_stays_unclaimed_until_current_blast_and_known_report(self):
        self.assertEqual(self.pickup()['rejected'], 'hidden_item_not_reported_revealed')
        self.assertEqual(self.report()['rejected'], 'no_current_blast_sent')
        self.emitted_blast()
        self.assertEqual(self.pickup()['rejected'], 'hidden_item_not_reported_revealed')
        self.assertEqual(self.sent, [])
        self.assertEqual(self.report()['revealed_indices'], [27])
        self.assertNotIn('rejected', self.pickup())
        self.assertEqual(len(self.sent), 2)
        self.assertEqual(self.pickup()['rejected'], 'claim_not_current_or_already_consumed')
        self.assertEqual(len(self.sent), 2)

    def test_wrong_cell_slot_aux_do_not_release_hidden_claims(self):
        self.emitted_blast()
        for kwargs, reason in [(dict(row=9), 'unknown_hidden_cell'),
                               (dict(slot=1), 'unsupported_block_report'),
                               (dict(aux=0), 'unsupported_block_report')]:
            self.assertEqual(self.report(**kwargs)['rejected'], reason)
        self.assertEqual(self.s.round['revealed_hidden'], {})
        self.assertEqual(self.pickup()['rejected'], 'hidden_item_not_reported_revealed')

    def test_other_bound_peer_report_can_release_cell_and_duplicates_are_idempotent(self):
        self.emitted_blast()
        first = self.report(event=self.b, slot=1)
        self.assertEqual(first['revealed_indices'], [27])
        duplicate = self.report()
        self.assertEqual(duplicate['already_revealed'], [27])
        self.assertIs(self.s.round['revealed_hidden']['27'], first)
        self.assertNotIn('rejected', self.pickup())
        self.assertEqual(len(self.sent), 2)

    def test_start_clears_reveal_and_old_blast_cannot_release_new_claim(self):
        self.emitted_blast()
        self.report()
        self.pickup()
        self.s.room.finish()
        self.s.start_round()
        self.sent.clear()
        self.assertEqual(self.s.round['revealed_hidden'], {})
        self.assertEqual(self.report()['rejected'], 'no_current_blast_sent')
        self.assertEqual(self.pickup()['rejected'], 'hidden_item_not_reported_revealed')
        self.assertEqual(self.pickup(key=12345)['rejected'], 'round_key_mismatch')
        self.emitted_blast()
        self.report()
        self.assertNotIn('rejected', self.pickup())

    def test_floor_items_and_other_hidden_cell_remain_independent(self):
        self.assertNotIn('rejected', self.pickup(17))
        self.emitted_blast()
        self.report()
        self.assertEqual(self.pickup(28)['rejected'], 'hidden_item_not_reported_revealed')
        self.report(row=3, col=2)
        self.assertNotIn('rejected', self.pickup(28))

    def test_pending_timer_is_not_emitted_blast(self):
        self.s.state['round_bombs'].append(dict(round=self.s.room.generation))
        self.assertEqual(self.report()['rejected'], 'no_current_blast_sent')
        self.assertEqual(self.s.round['revealed_hidden'], {})

    def test_duplicate_catalog_index_fails_without_silently_replacing_record(self):
        duplicate = [dict(index=27, row=2, column=3, kind=0)] * 2
        with self.assertRaisesRegex(ValueError, 'duplicate hidden item'):
            ProbeService(self.profile(duplicate), self.out, {})

    def test_after_blast_forged_known_cell_report_is_outside_authority_scope(self):
        self.emitted_blast()
        # No recovered key/tick in142 or server collision simulation. A forged
        # report after any current158 passes; this limitation is explicit.
        self.assertEqual(self.report()['revealed_indices'], [27])


if __name__ == '__main__':
    unittest.main()
