"""Departure transport failures are separate from native protocol evidence."""
import errno
import copy
from pathlib import Path
import selectors
import socket
import tempfile
from types import SimpleNamespace
import unittest
from unittest.mock import patch

from server import probe_codec as codec
from server.probe_service import ProbeService


class DepartureSendFailureTests(unittest.TestCase):
    def setUp(self):
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.out = Path(temporary.name)
        aliases = ['rank_Awin', 'promote_B', 'auth_C', 'roomlist_C', 'join_C',
                   'player_C', 'slot_C', 'guest_C', 'ready_C', 'start', 'slot_A']
        profile = SimpleNamespace(frames=dict.fromkeys(aliases),
                                  data={'items': [], 'initial_key': 12345},
                                  raw=lambda _: codec.encode(bytes(37), 110, b'', 73),
                                  body=lambda _: b'')
        self.service = ProbeService(profile, self.out, {})
        self.addCleanup(self.service.selector.close)
        self.remote = {}
        for ordinal, role in enumerate('AB', 1):
            connection, remote = socket.socketpair()
            self.addCleanup(connection.close)
            self.addCleanup(remote.close)
            self.remote[role] = remote
            event = dict(global_ordinal=ordinal, role=role, reply_ordinal=4,
                         create_response_sent={'after_c2s_opcode': 82}, map_response_sent=True)
            self.service.events.append(event)
            self.service.channels[role] = event
            self.service.connections[ordinal] = dict(socket=connection, event=event)
            self.service.selector.register(connection, selectors.EVENT_READ, ordinal)
        self.service.room.begin()
        self.service.room.finish()
        self.service.round = {'index': 1, 'refresh': {}}

    def test_both_remote_closes_before_readiness_delivery_targets_failed_recipient(self):
        self.remote['A'].close()
        self.remote['B'].close()
        # A's EOF is handled first while B's remote close is still unobserved.
        self.service.close_connection(1, 'eof')
        a, b = self.service.events
        self.assertEqual(a['closed']['reason'], 'eof')
        self.assertEqual(b['closed']['reason'], 'departure_send_failure')
        failure, = self.service.state['send_failures']
        self.assertEqual((failure['departed'], failure['recipient'],
                          failure['connection'], failure['opcode']), ('A', 'B', 2, 107))
        self.assertEqual(failure['errno'], errno.EPIPE)
        self.assertEqual(failure['sent_prefix'], [])
        self.assertNotIn('dynamic_s2c', b)
        self.assertNotIn('s2c_next_seq', b)
        self.assertNotIn('departures_sent', b)
        self.assertEqual(list(self.out.glob('*.bin')), [])
        self.assertEqual(self.service.peers(), [])
        self.assertEqual(self.service.state['errors'], [])
        self.service.close_connection(2, 'eof')
        self.assertEqual(len(self.service.state['send_failures']), 1)

    def test_failed_second_delivery_preserves_only_successful_first_frame(self):
        original = self.service.connections[2]['socket']

        class ResetAfterFirst:
            calls = 0

            def fileno(self):
                return original.fileno()

            def close(self):
                original.close()

            def sendall(self, raw):
                self.calls += 1
                if self.calls == 2:
                    raise ConnectionResetError(errno.ECONNRESET, 'controlled reset after107')
                original.sendall(raw)

        self.service.connections[2]['socket'] = ResetAfterFirst()
        self.service.close_connection(1, 'eof')
        b = self.service.events[1]
        failure, = self.service.state['send_failures']
        self.assertEqual((failure['recipient'], failure['opcode']), ('B', 110))
        self.assertEqual([row['opcode'] for row in failure['sent_prefix']], [107])
        self.assertEqual([row['opcode'] for row in b['dynamic_s2c']], [107])
        self.assertEqual(b['s2c_next_seq'], 76)
        self.assertNotIn('departures_sent', b)
        frame, = list(self.out.glob('*.bin'))
        self.assertEqual(self.remote['B'].recv(4096), frame.read_bytes())
        self.assertEqual(b['closed']['reason'], 'departure_send_failure')

    def test_storage_failure_is_not_reclassified_as_peer_disconnect(self):
        with patch.object(self.service, 'send', side_effect=OSError(errno.ENOSPC, 'disk full')):
            with self.assertRaises(OSError) as raised:
                self.service.deliver_departure(self.service.events[1], 'A')
        self.assertEqual(raised.exception.errno, errno.ENOSPC)
        self.assertNotIn('closed', self.service.events[1])
        self.assertNotIn('send_failures', self.service.state)

    def test_cached_batch_skips_recipient_closed_during_other_eof_cleanup(self):
        self.remote['A'].close()
        self.remote['B'].close()
        batch = self.service.selector.select(.5)
        self.assertEqual({key.data for key, _ in batch}, {1, 2})
        # Run the actual cached A key first, closing B as a failed recipient.
        for key, _ in sorted(batch, key=lambda pair: pair[0].data):
            self.service.receive(key.data)
        a, b = self.service.events
        self.assertEqual(a['closed']['reason'], 'eof')
        self.assertEqual(b['closed']['reason'], 'departure_send_failure')
        self.assertEqual(self.service.connections[2]['socket'].fileno(), -1)
        self.assertNotIn('peer_eof', b)
        self.assertEqual(self.service.state['errors'], [])
        self.assertEqual(len(self.service.state['send_failures']), 1)
        before = copy.deepcopy(self.service.state)
        for key, _ in batch:
            self.service.receive(key.data)
        self.assertEqual(self.service.state, before)

    def test_reverse_batch_order_preserves_attribution_and_closes_once(self):
        self.remote['A'].close()
        self.remote['B'].close()
        batch = self.service.selector.select(.5)
        self.assertEqual({key.data for key, _ in batch}, {1, 2})
        for key, _ in sorted(batch, key=lambda pair: pair[0].data, reverse=True):
            self.service.receive(key.data)
        a, b = self.service.events
        self.assertEqual(b['closed']['reason'], 'eof')
        self.assertEqual(a['closed']['reason'], 'departure_send_failure')
        self.assertEqual(self.service.state['send_failures'][0]['recipient'], 'A')
        self.assertEqual(self.service.peers(), [])

    def test_one_eof_does_not_skip_other_live_key_or_drop_its_bytes(self):
        self.remote['A'].close()
        payload = b'positive live read'
        self.remote['B'].sendall(payload)
        self.service.connections[2].update(buffer=bytearray(), sequence=40)
        self.service.events[1]['chunks'] = []
        batch = self.service.selector.select(.5)
        self.assertEqual({key.data for key, _ in batch}, {1, 2})
        for key, _ in sorted(batch, key=lambda pair: pair[0].data):
            self.service.receive(key.data)
        self.assertNotIn('closed', self.service.events[1])
        self.assertEqual(self.service.connections[2]['buffer'], payload)
        self.assertEqual(bytes.fromhex(self.service.events[1]['chunks'][0]['hex']), payload)
        self.assertNotIn('send_failures', self.service.state)
        self.assertEqual(self.service.events[1]['s2c_next_seq'], 79)

    def test_unmarked_broken_descriptor_is_still_an_error(self):
        self.service.connections[2]['socket'].close()
        with self.assertRaises(OSError) as raised:
            self.service.receive(2)
        self.assertEqual(raised.exception.errno, errno.EBADF)
        self.assertNotIn('closed', self.service.events[1])

    def test_run_dispatches_actual_cached_batch_without_spurious_error_row(self):
        self.remote['A'].close()
        self.remote['B'].close()
        batch = sorted(self.service.selector.select(.5), key=lambda pair: pair[0].data)
        self.assertEqual([key.data for key, _ in batch], [1, 2])

        def one_batch(_timeout):
            # Stop after one iteration; dispatch uses the actual returned keys.
            self.service.closed = True
            return batch

        with patch.object(self.service.selector, 'select', side_effect=one_batch):
            self.service.run([])
        self.assertTrue(self.service.state['stopped'])
        self.assertEqual(self.service.state['errors'], [])
        self.assertEqual(len(self.service.state['send_failures']), 1)


if __name__ == '__main__':
    unittest.main()
