"""Departure transport failures are separate from native protocol evidence."""
import errno
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


if __name__ == '__main__':
    unittest.main()
