"""Channel config and transport regressions; native compatibility is separate evidence."""
from contextlib import redirect_stderr
from io import StringIO
from pathlib import Path
import copy
import socket
import tempfile
from types import SimpleNamespace
import unittest
from unittest.mock import patch

from server import probe_codec as codec
from server.probe_service import ProbeService, channel_endpoint_config, main

# CP015 redirect fixture, seq52, independently checked by native KR readers.
REDIRECT = bytes.fromhex('690000000007000000ffffffff0e000000020000000200000002000000'
                         '0000000001000000f34da399cdb515efe2c2dc3af6fdac')


def profile(raw=REDIRECT, sequence=52):
    names = ['rank_Awin', 'promote_B', 'auth_C', 'roomlist_C', 'join_C',
             'player_C', 'slot_C', 'guest_C', 'ready_C']
    frames = dict.fromkeys(names, b'') | {'redirect': raw}
    return SimpleNamespace(frames=frames, raw=frames.__getitem__,
                           data={'fixtures': {'redirect': {'sequence': sequence}},
                                 'items': [], 'initial_key': 12345})


class ChannelEndpointTests(unittest.TestCase):
    def service(self, fixture=None, **kwargs):
        temp = tempfile.TemporaryDirectory()
        self.addCleanup(temp.cleanup)
        service = ProbeService(fixture or profile(), Path(temp.name), {}, **kwargs)
        self.addCleanup(service.selector.close)
        return service

    def receive_redirect(self, service):
        client, server = socket.socketpair()
        self.addCleanup(client.close)
        self.addCleanup(server.close)
        client.settimeout(1)
        server.settimeout(1)
        event = {'global_ordinal': 1, 'reply_ordinal': 3, 'role': 'A', 'chunks': []}
        service.connections[1] = {'socket': server, 'event': event,
                                  'buffer': bytearray(), 'sequence': 40}
        client.sendall(codec.encode(REDIRECT[:37], 5, bytes(9), 40))
        service.receive(1)
        received = bytearray()
        while len(received) < 52:
            received.extend(client.recv(52 - len(received)))
        self.assertEqual(service.connections[1]['sequence'], 43)
        self.assertIn('redirect_response_sent', event)
        return bytes(received)

    def test_default_wire_bytes_and_profile_are_unchanged(self):
        fixture = profile()
        before = copy.deepcopy(fixture.data), dict(fixture.frames)
        service = self.service(fixture)
        self.assertIsNone(service.redirect_frame)
        self.assertEqual(self.receive_redirect(service), REDIRECT)
        self.assertEqual((fixture.data, fixture.frames), before)

    def test_explicit_pair_changes_only_six_typed_body_bytes_on_actual_send(self):
        fixture = profile()
        service = self.service(fixture, channel_ip='10.77.0.1', channel_port=23858)
        raw = self.receive_redirect(service)
        original, body = codec.frame_body(REDIRECT, 52), codec.frame_body(raw, 52)
        self.assertEqual(raw[:37], REDIRECT[:37])
        self.assertEqual(body[:6], original[:6])
        self.assertEqual(body[12:], original[12:])
        self.assertEqual(codec.read_fields(body[6:12], [4, 2]), [0x0A4D0001, 23858])
        self.assertEqual(service.state['network']['channel_endpoint'],
                         {'ip': '10.77.0.1', 'port': 23858})
        self.assertEqual(fixture.frames['redirect'], REDIRECT)
        same = self.service(channel_ip='127.0.0.1', channel_port=3858)
        self.assertEqual(self.receive_redirect(same), REDIRECT)
        legacy = self.service(profile(sequence=None), channel_ip='10.77.0.1', channel_port=3858)
        self.assertEqual(codec.read_fields(codec.frame_body(self.receive_redirect(legacy), 52)[6:12],
                                          [4, 2]), [0x0A4D0001, 3858])

    def test_invalid_pair_rejected_before_selector(self):
        cases = [('127.0.0.1', None), (None, 3858), ('127.0.0.1', 0),
                 ('127.0.0.1', 65536), ('127.0.0.1', True), ('127.0.0.1', '3858'),
                 ('::1', 3858), ('0.0.0.0', 3858), ('239.1.1.1', 3858),
                 ('255.255.255.255', 3858), ('host', 3858), (123, 3858)]
        for ip, port in cases:
            with self.subTest(ip=ip, port=port), patch('server.probe_service.selectors.DefaultSelector') as selector:
                with self.assertRaises(ValueError):
                    self.service(channel_ip=ip, channel_port=port)
                selector.assert_not_called()
        self.assertEqual(channel_endpoint_config('127.0.0.1', 65535)['port'], 65535)

    def test_unsupported_variant_rejected_before_selector_without_mutating_profile(self):
        body = codec.frame_body(REDIRECT, 52)
        variants = [profile(sequence=55), profile(sequence=True), profile(raw=REDIRECT[:-1]),
                    profile(raw=bytes([0]) + REDIRECT[1:]),
                    profile(raw=codec.encode(REDIRECT[:37], 8, body, 52)),
                    profile(raw=codec.encode(REDIRECT[:37], 7, codec.pack_fields([0], [1])+body[1:], 52)),
                    profile(raw=codec.encode(REDIRECT[:37], 7, body[:1]+codec.pack_fields([11], [1])+body[2:], 52)),
                    profile(raw=codec.encode(REDIRECT[:37], 7, body[:2]+codec.pack_fields([1], [4])+body[6:], 52)),
                    profile(raw=codec.encode(REDIRECT[:37], 7, body[:12]+codec.pack_fields([1], [2]), 52)),
                    profile(raw=REDIRECT[:-1]+bytes([REDIRECT[-1]^1]))]
        for fixture in variants:
            before = copy.deepcopy(fixture.data), dict(fixture.frames)
            with self.subTest(raw=fixture.frames['redirect'].hex()), patch('server.probe_service.selectors.DefaultSelector') as selector:
                with self.assertRaises(ValueError):
                    self.service(fixture, channel_ip='10.77.0.1', channel_port=3858)
                selector.assert_not_called()
                self.assertEqual((fixture.data, fixture.frames), before)
        # Profiles unsupported for override remain usable on the old default path.
        self.assertIsNone(self.service(profile(sequence=55)).redirect_frame)

    def test_cli_invalid_pair_does_not_read_profile_or_create_output(self):
        with tempfile.TemporaryDirectory() as directory:
            out = Path(directory) / 'missing'
            for options in [['--channel-ip', '10.77.0.1'], ['--channel-port', '3858'],
                            ['--channel-ip', '0.0.0.0', '--channel-port', '3858']]:
                with patch('sys.argv', ['probe_service', '--profile', 'unused', '--out', str(out),
                                        '--port', '0', *options]), patch('server.probe_service.FixtureProfile') as load, redirect_stderr(StringIO()):
                    with self.assertRaises(SystemExit) as error:
                        main()
                    self.assertEqual(error.exception.code, 2)
                    load.assert_not_called()
                    self.assertFalse(out.exists())

    def test_cli_unsupported_profile_fails_before_output_or_listener(self):
        with tempfile.TemporaryDirectory() as directory:
            out = Path(directory) / 'missing'
            fixture = profile(sequence=55)
            with patch('sys.argv', ['probe_service', '--profile', 'unused', '--out', str(out),
                                    '--port', '0', '--channel-ip', '10.77.0.1', '--channel-port', '3858']), \
                 patch('server.probe_service.FixtureProfile', return_value=fixture), \
                 patch('server.probe_service.selectors.DefaultSelector') as selector, \
                 patch('server.probe_service.socket.socket') as listener:
                with self.assertRaisesRegex(ValueError, 'unsupported channel redirect'):
                    main()
                selector.assert_not_called()
                listener.assert_not_called()
                self.assertFalse(out.exists())


if __name__ == '__main__':
    unittest.main()
