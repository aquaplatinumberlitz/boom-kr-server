"""Optional host Back fixture export; this does not certify native semantics."""
import hashlib
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

from export_probe_profile import FIXTURES


class HostBackExportTests(unittest.TestCase):
    def test_optin_pins_known_bodies_and_keeps_legacy_profile_unchanged(self):
        root = Path(__file__).resolve().parents[1]
        aliases = {
            'join_A': ('rejoin-r1/reply87-A-session1.bin', 55),
            'player_A_guest': ('rejoin-r1/A106-unready.bin', 58),
            'slot_A_guest': ('rejoin-r1/A110-unready.bin', 61),
            'guest_B_chief': ('rejoin-r1/B106-chief.bin', 58),
            'ready_A': ('rejoin-r1/A110-ready.bin', 100),
        }
        with tempfile.TemporaryDirectory(prefix='test-host-back-', dir=root / 'local') as tmp:
            folder = Path(tmp)
            for relative, _ in [*FIXTURES.values(), *aliases.values()]:
                path = folder / relative
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_bytes(relative.encode())
            command = [sys.executable, str(root / 'tools/export_probe_profile.py'),
                       '--source-root', str(folder)]
            normal, optional = folder / 'normal.json', folder / 'optional.json'
            battle = folder / 'battle.json'
            subprocess.run(command + ['--out', str(normal)], check=True, capture_output=True)
            subprocess.run(command + ['--out', str(optional), '--host-back-rejoin'],
                           check=True, capture_output=True)
            subprocess.run(command + ['--out', str(battle), '--host-battle-exit'],
                           check=True, capture_output=True)
            battle_profile = json.loads(battle.read_text())
            self.assertEqual(battle_profile.pop('battle_exit'), {'mode': 'host_A_202_86'})
            self.assertEqual(battle_profile, json.loads(optional.read_text()))
            before, after = json.loads(normal.read_text()), json.loads(optional.read_text())
            self.assertNotIn('host_back_rejoin', before)
            self.assertEqual(after.pop('host_back_rejoin'), {'mode': 'idle_A_to_B'})
            for alias, (relative, sequence) in aliases.items():
                raw = (folder / relative).read_bytes()
                self.assertEqual(after['fixtures'].pop(alias), {
                    'path': str(folder / relative), 'bytes': len(raw),
                    'sha256': hashlib.sha256(raw).hexdigest(), 'sequence': sequence})
                self.assertEqual(raw, relative.encode())
            self.assertEqual(before, after)


if __name__ == '__main__':
    unittest.main()
