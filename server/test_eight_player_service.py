"""Eight admission and policy boundaries; synthetic requests, not native proof."""
from pathlib import Path
import socket
import tempfile
import unittest

from server.eight_player_service import EightPlayerService
from server.test_four_player_service import fixtures


class EightSessionTests(unittest.TestCase):
    IPS = {role: '127.0.0.' + str(i+2) for i, role in enumerate('ABCDEFGH')}

    def test_eight_sources_bind_32_transports_reject_ninth_and_exhausted(self):
        with tempfile.TemporaryDirectory() as folder:
            service = EightPlayerService(fixtures(), Path(folder), self.IPS)
            listener = socket.socket(); listener.bind(('127.0.0.1', 0)); listener.listen(32)
            clients = []
            try:
                for role in 'HGFEDCBA' * 4:
                    client = socket.socket(); clients.append(client); client.settimeout(1)
                    client.bind((self.IPS[role], 0)); client.connect(listener.getsockname())
                    service.accept(listener)
                    self.assertEqual(client.recv(4096), service.profile.raw('bootstrap'))
                    self.assertEqual(service.events[-1]['role'], role)
                for address in ['127.0.0.99', self.IPS['H']]:
                    client = socket.socket(); clients.append(client); client.settimeout(1)
                    client.bind((address, 0)); client.connect(listener.getsockname())
                    service.accept(listener); self.assertEqual(client.recv(4096), b'')
                self.assertEqual(len(service.events), 32)
                self.assertEqual(service.source_counts, dict.fromkeys(self.IPS, 4))
                self.assertEqual(set(service.channels), set(self.IPS))
                service.save()
                self.assertEqual(service.state['observability']['schema'], 'kre-eight-context-v1')
                self.assertIsNone(service.bomb_admission)
                self.assertIsNone(service.battle_reports)
            finally:
                for client in clients: client.close()
                for entry in service.connections.values(): entry['socket'].close()
                listener.close(); service.selector.close()

    def test_unmeasured_combat_or_persistence_is_refused(self):
        with tempfile.TemporaryDirectory() as folder:
            for options in [dict(bomb_profile={}), dict(battle_outcome=True),
                            dict(abort_room=True), dict(identity_store=Path(folder)/'ids.json')]:
                with self.subTest(options=options), self.assertRaisesRegex(ValueError, 'eight session preview'):
                    EightPlayerService(fixtures(), Path(folder), self.IPS, **options)

