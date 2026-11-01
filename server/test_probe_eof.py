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
        profile = SimpleNamespace(frames={name: b'' for name in ['rank_Awin', 'promote_B', 'auth_C', 'roomlist_C', 'join_C', 'player_C', 'slot_C', 'guest_C', 'ready_C']},
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

    def prepare_setup(self, *, accepted=True, player=True):
        self.service.room.finish()
        self.service.round = {'index': 0}
        guest = self.service.channels['B']
        guest.pop('map_response_sent')
        if accepted:
            guest['create_response_sent'] = {'after_c2s_opcode': 82}
        if player:
            guest['player_response_sent'] = {'monotonic_ns': 1}
        return guest

    def test_joined_guest_eof_before_map_clears_slot_once_without_start_or_rank(self):
        from server import probe_codec as codec
        guest = self.prepare_setup()
        self.close_at(2, 10)
        self.assertEqual([s['opcode'] for s in self.sends], [107, 110])
        self.assertEqual([s['role'] for s in self.sends], ['A', 'A'])
        self.assertEqual(codec.read_fields(self.sends[0]['body'], [1, 4, 1]), [1, 2, 0])
        self.assertEqual(self.sends[1]['alias'], 'slot_A')
        self.assertNotIn('guest_ready_received', guest)
        self.assertEqual(self.service.host_role, 'A')
        # A delayed owner tick cannot continue B setup or create a false phase.
        self.service.send_raw = lambda *_args: self.fail('setup send to closed guest')
        self.tick_at(20)
        self.close_at(2, 21)
        frame = codec.encode(bytes(37), 104, b'', 40)
        self.service.receive_channel(self.service.channels['A'], frame, 40, b'')
        self.assertEqual([s['opcode'] for s in self.sends], [107, 110])
        self.assertEqual(self.service.round, {'index': 0})
        self.assertFalse(self.service.room.active)
        self.assertEqual(self.service.room.keys.issued_count, 1)  # fixture begin only
        self.assertFalse(guest.get('map_response_sent'))

    def test_lobby_guest_eof_does_not_remove_room_slot_without_join_acceptance(self):
        self.prepare_setup(accepted=False, player=False)
        self.close_at(2, 10)
        self.tick_at(20)
        self.assertEqual(self.sends, [])
        self.assertEqual(self.service.state['policy_events'], [])

    def test_joined_guest_eof_before_own_profile_still_cleans_membership(self):
        self.prepare_setup(player=False)
        self.close_at(2, 10)
        self.tick_at(20)
        self.assertEqual([s['opcode'] for s in self.sends], [107, 110])
        self.assertEqual(self.service.state['policy_events'][0]['op'], 'cleanup_departed')

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
