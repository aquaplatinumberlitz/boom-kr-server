"""WCP/channel port collision cannot consume positional proxy role slots."""
import unittest

from tools.probe_channel_identity import find_fixture_connection


class FixtureConnectionTests(unittest.TestCase):
    def event(self, ordinal, role, reply, source_port, **extra):
        return dict(global_ordinal=ordinal, role=role, reply_ordinal=reply,
                    peer=['127.0.0.1', source_port], port=23858,
                    accept_monotonic_ns=ordinal * 100, **extra)

    def test_wcp_a2_then_real_a4_and_b4_keep_their_declared_roles(self):
        events = [self.event(2, 'A', 2, 50001), self.event(4, 'A', 4, 50002),
                  self.event(8, 'B', 4, 50003)]
        observed = [find_fixture_connection({'tcp_events': events}, ('127.0.0.1', port), 23858)
                    for port in (50001, 50002, 50003)]
        self.assertEqual([(e['role'], e['reply_ordinal']) for e in observed],
                         [('A', 2), ('A', 4), ('B', 4)])

    def test_pending_wrong_endpoint_and_retired_rows_are_not_selected(self):
        old = self.event(2, 'A', 2, 50001, closed={'reason': 'eof'})
        self.assertIsNone(find_fixture_connection({'tcp_events': [old]}, ('127.0.0.1', 50001), 23858))
        new = self.event(4, 'A', 4, 50001)
        self.assertEqual(find_fixture_connection({'tcp_events': [old, new]},
                                                ('127.0.0.1', 50001), 23858)['global_ordinal'], 4)
        self.assertIsNone(find_fixture_connection({'tcp_events': [new]}, ('127.0.0.1', 50002), 23858))
        self.assertIsNone(find_fixture_connection({'tcp_events': [new]}, ('127.0.0.1', 50001), 3838))

    def test_ambiguous_live_rows_fail_instead_of_guessing_role(self):
        events = [self.event(4, 'A', 4, 50001), self.event(8, 'B', 4, 50001)]
        with self.assertRaisesRegex(ValueError, 'ambiguous'):
            find_fixture_connection({'tcp_events': events}, ('127.0.0.1', 50001), 23858)


if __name__ == '__main__':
    unittest.main()
