import unittest
from pathlib import Path


class NativeHarnessPathTests(unittest.TestCase):
    def test_codex_harness_layout_resolves_repo_and_input_sibling(self):
        bootstrap = Path('/repo/local/research/codex/m8-room-final-2p-control-r1/bootstrap.py')
        repo_root = bootstrap.resolve().parents[4]
        input_script = bootstrap.resolve().parent.parent / 'm8-room-native-waiting-control-r1/input.py'
        self.assertEqual(repo_root, Path('/repo'))
        self.assertEqual(input_script.parent.name, 'm8-room-native-waiting-control-r1')


if __name__ == '__main__':
    unittest.main()
