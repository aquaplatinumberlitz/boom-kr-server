"""Owner/ticket/transport lifetime regressions, not native KR protocol oracles."""
from pathlib import Path
import socket
import tempfile
from types import SimpleNamespace
import unittest
from unittest.mock import patch

from server.probe_service import ProbeService


class EofTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        profile = SimpleNamespace(frames={'rank_Awin': b'', 'promote_B': b''},
                                  data={'items': [], 'initial_key': 12345})
        self.service = ProbeService(profile, Path(self.temporary.name), {'A': 0, 'B': 0})
        self.addCleanup(self.service.selector.close)
        self.sends = []
        def send(event, alias, label, **kwargs):
            self.assertFalse(event.get('closed'))
            meta = {'opcode': kwargs.get('opcode', {'rank': 175, 'rank_Awin': 175,
                    'return': 180, 'promote_B': 110, 'slot_A': 110, 'blast': 158}.get(alias)),
                    'alias': alias, 'label': label, 'role': event['role'],
                    'body': kwargs.get('body')}
            self.sends.append(meta)
            return meta
        self.service.send = send
        for ordinal, role in enumerate('AB', 1):
            connection, other = socket.socketpair()
            self.addCleanup(connection.close)
            self.addCleanup(other.close)
            event = {'global_ordinal': ordinal, 'reply_ordinal': 4, 'role': role,
                     'map_response_sent': True, 'guest_ready_received': True}
            self.service.events.append(event)
            self.service.channels[role] = event
            self.service.connections[ordinal] = {'socket': connection, 'event': event}
            self.service.selector.register(connection, 1, ordinal)
        self.service.room.begin()
        self.service.round = {'index': 1, 'key': self.service.room.key}

    def close_at(self, ordinal, seconds):
        with patch('server.probe_service.time.monotonic_ns', return_value=seconds * 10**9):
            self.service.close_connection(ordinal, 'eof')

    def tick_at(self, seconds):
        with patch('server.probe_service.time.monotonic_ns', return_value=seconds * 10**9), \
             patch('server.probe_service.time.monotonic', return_value=seconds):
            self.service.tick()

    def test_host_eof_invalidates_original_job_then_returns_promotes_only_survivor(self):
        ticket = self.service.room.bomb()
        bomb = {'round': 1, 'id': ticket.wire_id}
        self.service.jobs.append((106, ticket, bomb))
        self.close_at(1, 102)
        self.assertFalse(self.service.room.active)
        self.assertEqual(self.service.room.fence.rejection(ticket), 'round_closed')
        self.assertEqual([e['role'] for e in self.service.peers()], ['B'])
        self.tick_at(107)
        self.assertEqual(bomb['timer_rejected'], 'round_closed')
        self.assertEqual(self.sends, [])
        self.tick_at(114)
        self.tick_at(119)
        self.tick_at(122)
        self.assertEqual([(s['opcode'], s['role']) for s in self.sends],
                         [(175, 'B'), (180, 'B'), (107, 'B'), (110, 'B')])
        self.assertEqual(self.sends[-1]['alias'], 'promote_B')
        self.assertEqual(self.service.jobs, [])
        previous = list(self.sends)
        self.close_at(1, 123)
        self.tick_at(140)
        self.assertEqual(self.sends, previous)
        self.assertEqual(sum(e['op'] == 'abort' for e in self.service.state['policy_events']), 1)

    def test_guest_eof_chooses_other_rank_and_keeps_original_host(self):
        self.close_at(2, 102)
        self.tick_at(114)
        self.tick_at(119)
        self.tick_at(122)
        self.assertEqual([s['role'] for s in self.sends], ['A'] * 4)
        self.assertEqual(self.sends[0]['alias'], 'rank_Awin')
        self.assertEqual(self.sends[-1]['alias'], 'slot_A')

    def test_both_eof_and_non_channel_close_do_not_target_dead_sockets(self):
        self.service.events[0]['reply_ordinal'] = 3
        self.close_at(1, 101)
        self.assertTrue(self.service.room.active)
        self.close_at(2, 102)
        self.tick_at(114)
        self.tick_at(119)
        self.tick_at(122)
        self.assertEqual(self.sends, [])

    def test_missing_lifecycle_profile_fails_before_opening_transports(self):
        profile = SimpleNamespace(frames={}, data={})
        with self.assertRaisesRegex(ValueError, 'missing lifecycle fixtures'):
            ProbeService(profile, Path(self.temporary.name), {'A': 0, 'B': 0})

    def test_existing_result_is_not_replaced_by_an_eof_forfeit(self):
        self.service.room.finish()
        self.service.round['rank'] = {'monotonic_ns': 100 * 10**9, 'policy': 'existing outcome'}
        self.close_at(1, 102)
        self.tick_at(105)
        self.tick_at(108)
        self.assertEqual([s['opcode'] for s in self.sends], [180, 107, 110])
        self.assertEqual(self.service.round['rank']['policy'], 'existing outcome')


if __name__ == '__main__':
    unittest.main()
