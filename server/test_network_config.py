"""Operator endpoint configuration regressions, not a native protocol oracle."""
from contextlib import redirect_stderr
from io import StringIO
from pathlib import Path
import tempfile
from types import SimpleNamespace
import unittest
from unittest.mock import patch

from server import probe_codec as codec
from server.probe_service import ProbeService, main, network_config, peer_ip_options


class NetworkConfigTests(unittest.TestCase):
    def service(self, **kwargs):
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        names = ['rank_Awin', 'promote_B', 'auth_C', 'roomlist_C', 'join_C',
                 'player_C', 'slot_C', 'guest_C', 'ready_C']
        profile = SimpleNamespace(frames={n: b'' for n in names},
                                  data={'items': [], 'initial_key': 12345},
                                  body=lambda _: bytes(35))
        service = ProbeService(profile, Path(temporary.name), {}, **kwargs)
        self.addCleanup(service.selector.close)
        return service

    def peer_bodies(self, service, roles='AB', proxy=None):
        service.proxy_ports = proxy or {}
        service.room.begin()
        sent = {}
        for i, role in enumerate(roles):
            # init99 IP fields deliberately disagree with operator configuration.
            service.events.append({'role': role, 'reply_ordinal': 4, 'map_response_sent': True,
                                   'init99': {'fields': [0xFFFFFFFF, 0, 9700 + i]},
                                   'go_response_sent': {'monotonic_ns': 1},
                                   'play_response_sent': {'monotonic_ns': 1}})
        def send(event, alias, label, **kwargs):
            self.assertEqual(alias, 'peer')
            sent[event['role']] = kwargs['body']
            return {'monotonic_ns': 3 * 10**9}
        service.send = send
        with patch('server.probe_service.time.monotonic_ns', return_value=3 * 10**9):
            service.tick()
        return sent

    def test_default_and_explicit_addresses_change_only_remote_ipv4(self):
        default = self.service()
        explicit = self.service(bind_ip='10.77.0.1', peer_ips={'A': '10.77.0.2', 'B': '10.77.0.3'})
        before, after = self.peer_bodies(default), self.peer_bodies(explicit)
        for role, offset, address in [('A', 55, 0x0A4D0003), ('B', 36, 0x0A4D0002)]:
            expected = bytearray(before[role])
            self.assertEqual(len(expected), 97)
            expected[offset:offset + 4] = codec.pack_fields([address], [4])
            self.assertEqual(after[role], expected)
            self.assertEqual(codec.read_fields(after[role][15:19], [4]), [12345])
        self.assertEqual(default.state['network']['peer_ips'], dict.fromkeys('ABC', '127.0.0.1'))
        self.assertEqual(explicit.events[0]['peer153_sent']['remote_port'], 9701)

    def test_replacement_role_and_proxy_port_override_preserve_zero_endpoints(self):
        service = self.service(peer_ips={'B': '10.77.0.3', 'C': '10.77.0.4'})
        body = self.peer_bodies(service, roles='BC', proxy={'C': 14024})['B']
        self.assertEqual(codec.read_fields(body[36:42], [4, 2]), [0x0A4D0004, 14024])
        self.assertEqual(codec.read_fields(body[42:54], [4, 2, 4, 2]), [0, 0, 0, 0])
        self.assertEqual(codec.read_fields(body[55:73], [4, 2, 4, 2, 4, 2]), [0] * 6)
        self.assertEqual(service.events[0]['peer153_sent']['remote_ip'], '10.77.0.4')

    def test_real_listener_uses_configured_bind_and_ephemeral_port(self):
        for address in ['127.0.0.1', '127.0.0.2']:
            with self.subTest(address=address):
                service = self.service(bind_ip=address)
                service.closed = True
                service.run([0])
                self.assertEqual(service.state['listen_addresses'][0][0], address)
                self.assertGreater(service.state['listen_ports'][0], 0)
                self.assertTrue(service.state['stopped'])

    def test_bad_config_rejected_before_selector_creation(self):
        cases = [{'bind_ip': '::1'}, {'bind_ip': 'hostname'}, {'bind_ip': '224.0.0.1'},
                 {'bind_ip': 123}, {'peer_ips': {'D': '127.0.0.1'}},
                 {'peer_ips': {'A': '0.0.0.0'}}, {'peer_ips': {'A': '239.1.2.3'}},
                 {'peer_ips': {'A': '255.255.255.255'}}, {'peer_ips': {'A': '127.1'}},
                 {'peer_ips': {'A': '::ffff:127.0.0.1'}}, {'peer_ips': []}]
        for config in cases:
            with self.subTest(config=config), patch('server.probe_service.selectors.DefaultSelector') as selector:
                with self.assertRaises(ValueError):
                    self.service(**config)
                selector.assert_not_called()
        self.assertEqual(network_config('0.0.0.0')[0], '0.0.0.0')

    def test_cli_invalid_or_duplicate_role_does_not_create_output_or_read_fixtures(self):
        with tempfile.TemporaryDirectory() as directory:
            out = Path(directory) / 'must-not-exist'
            for options in [['--peer-ip', 'A=127.0.0.1', '--peer-ip', 'A=127.0.0.2'],
                            ['--peer-ip', 'D=127.0.0.1'], ['--bind-ip', 'bad']]:
                argv = ['probe_service', '--profile', 'missing.json', '--out', str(out), '--port', '0', *options]
                with patch('sys.argv', argv), patch('server.probe_service.FixtureProfile') as fixture, redirect_stderr(StringIO()):
                    with self.assertRaises(SystemExit) as error:
                        main()
                    self.assertEqual(error.exception.code, 2)
                    fixture.assert_not_called()
                    self.assertFalse(out.exists())
        self.assertEqual(peer_ip_options(['A=127.0.0.2', 'C=10.77.0.4']),
                         {'A': '127.0.0.2', 'C': '10.77.0.4'})


if __name__ == '__main__':
    unittest.main()
