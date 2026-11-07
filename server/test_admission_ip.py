"""Real TCP admission controls; synthetic hello is not a native protocol oracle."""
from contextlib import redirect_stderr
from io import StringIO
from pathlib import Path
import socket
import tempfile
from types import SimpleNamespace
import unittest
from unittest.mock import patch

from server.probe_service import (ProbeService, admission_ip_config,
                                  admission_ip_options, main)


class AdmissionIPTests(unittest.TestCase):
    ADDRESSES = {'A': '127.0.0.2', 'B': '127.0.0.3'}

    def service(self, *, admission_ips=None, replacement=False, **kwargs):
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        names = ['rank_Awin', 'promote_B', 'auth_C', 'roomlist_C', 'join_C',
                 'player_C', 'slot_C', 'guest_C', 'ready_C']
        data = {'items': [], 'initial_key': 12345}
        if replacement:
            data['replacement_slot'] = {}
        profile = SimpleNamespace(frames=dict.fromkeys(names, b''), data=data,
                                  body=lambda _: bytes(35), raw=lambda _: b'hello')
        service = ProbeService(profile, Path(temporary.name), {},
                               admission_ips=admission_ips, **kwargs)
        self.addCleanup(service.selector.close)
        self.addCleanup(lambda: [q['socket'].close() for q in service.connections.values()])
        return service

    def listener(self):
        listener = socket.socket()
        self.addCleanup(listener.close)
        listener.bind(('127.0.0.1', 0))
        listener.listen(16)
        return listener

    def connect(self, service, listener, address):
        client = socket.socket()
        self.addCleanup(client.close)
        client.settimeout(1)
        client.bind((address, 0))
        client.connect(listener.getsockname())
        previous = len(service.events)
        service.accept(listener)
        hello = client.recv(32)
        if len(service.events) == previous:
            self.assertEqual(hello, b'')
            return None
        self.assertEqual(hello, b'hello')
        return service.events[-1]

    def test_interleaved_sources_keep_their_four_transport_stages(self):
        for order in ['ABABABAB', 'BBAAAABB', 'AAAABBBB']:
            with self.subTest(order=order):
                service = self.service(admission_ips=self.ADDRESSES)
                listener = self.listener()
                stages = {'A': [], 'B': []}
                for ordinal, role in enumerate(order, 1):
                    event = self.connect(service, listener, self.ADDRESSES[role])
                    self.assertEqual(event['role'], role)
                    self.assertEqual(event['global_ordinal'], ordinal)
                    stages[role].append(event['reply_ordinal'])
                self.assertEqual(stages, {'A': [1, 2, 3, 4], 'B': [1, 2, 3, 4]})
                for role in 'AB':
                    self.assertEqual(service.channels[role]['peer'][0], self.ADDRESSES[role])
                    self.assertEqual(service.channels[role]['reply_ordinal'], 4)
                self.assertEqual(service.state['admission_binding']['counts'], {'A': 4, 'B': 4})

    def test_default_keeps_measured_legacy_accept_policy(self):
        service = self.service()
        listener = self.listener()
        events = [self.connect(service, listener, self.ADDRESSES[role])
                  for role in 'ABABABAB']
        self.assertEqual([e['role'] for e in events], list('AAAABBBB'))
        self.assertEqual([e['reply_ordinal'] for e in events], [1, 2, 3, 4, 1, 2, 3, 4])
        self.assertNotIn('admission_binding', service.state)

    def test_unknown_source_does_not_consume_role_or_channel(self):
        service = self.service(admission_ips=self.ADDRESSES)
        listener = self.listener()
        self.assertIsNone(self.connect(service, listener, '127.0.0.4'))
        self.assertEqual(service.events, [])
        self.assertEqual(service.channels, {})
        self.assertEqual(service.admission_counts, {'A': 0, 'B': 0})
        self.assertEqual(service.state['admission_rejections'][0]['reason'], 'unmapped_source')
        event = self.connect(service, listener, self.ADDRESSES['B'])
        self.assertEqual((event['global_ordinal'], event['role'], event['reply_ordinal']), (1, 'B', 1))

    def test_fifth_transport_is_rejected_without_consuming_other_role(self):
        service = self.service(admission_ips=self.ADDRESSES)
        listener = self.listener()
        for _ in range(4):
            self.connect(service, listener, self.ADDRESSES['A'])
        channel = service.channels['A']
        self.assertIsNone(self.connect(service, listener, self.ADDRESSES['A']))
        self.assertEqual(len(service.events), 4)
        self.assertIs(service.channels['A'], channel)
        self.assertEqual(service.admission_counts, {'A': 4, 'B': 0})
        self.assertEqual(service.state['admission_rejections'][0]['reason'], 'role_transports_exhausted')
        event = self.connect(service, listener, self.ADDRESSES['B'])
        self.assertEqual((event['global_ordinal'], event['role'], event['reply_ordinal']), (5, 'B', 1))

    def test_advertised_nat_addresses_are_independent_of_incoming_binding(self):
        outgoing = {'A': '10.78.2.1', 'B': '10.78.1.1'}
        service = self.service(admission_ips=self.ADDRESSES, peer_ips=outgoing)
        event = self.connect(service, self.listener(), self.ADDRESSES['B'])
        self.assertEqual((event['role'], event['reply_ordinal']), ('B', 1))
        self.assertEqual(service.peer_ips['B'], '10.78.1.1')
        self.assertEqual(service.state['admission_binding']['source'], 'design')

    def test_invalid_or_ambiguous_binding_fails_before_selector(self):
        for addresses in [{}, {'A': '127.0.0.2'}, {'A': '127.0.0.2', 'C': '127.0.0.3'},
                          {'A': '127.0.0.2', 'B': '127.0.0.2'}, [],
                          {'A': '0.0.0.0', 'B': '127.0.0.3'},
                          {'A': '::1', 'B': '127.0.0.3'}]:
            with self.subTest(addresses=addresses), patch('server.probe_service.selectors.DefaultSelector') as selector:
                with self.assertRaises(ValueError):
                    self.service(admission_ips=addresses)
                selector.assert_not_called()
        self.assertIsNone(admission_ip_config(None))

    def test_replacement_combination_is_rejected_before_selector(self):
        with patch('server.probe_service.selectors.DefaultSelector') as selector:
            with self.assertRaisesRegex(ValueError, 'without replacement_slot'):
                self.service(admission_ips=self.ADDRESSES, replacement=True)
            selector.assert_not_called()

    def test_cli_rejects_incomplete_duplicate_and_same_ip_before_profile_or_output(self):
        with tempfile.TemporaryDirectory() as directory:
            out = Path(directory) / 'must-not-exist'
            for options in [['A=127.0.0.2'], ['A=127.0.0.2', 'A=127.0.0.3'],
                            ['A=127.0.0.2', 'B=127.0.0.2']]:
                argv = ['probe_service', '--profile', 'missing.json', '--out', str(out), '--port', '0']
                for value in options:
                    argv += ['--admission-ip', value]
                with patch('sys.argv', argv), patch('server.probe_service.FixtureProfile') as fixture, redirect_stderr(StringIO()):
                    with self.assertRaises(SystemExit) as error:
                        main()
                    self.assertEqual(error.exception.code, 2)
                    fixture.assert_not_called()
                    self.assertFalse(out.exists())
        self.assertEqual(admission_ip_options(['A=127.0.0.2', 'B=127.0.0.3']), self.ADDRESSES)


if __name__ == '__main__':
    unittest.main()
