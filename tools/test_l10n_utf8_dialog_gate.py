import tempfile
import unittest
from pathlib import Path
from PIL import Image
from l10n_utf8_dialog_gate import dialog_candidate


class DialogGateTests(unittest.TestCase):
    def test_known_positive_and_negative_samples(self):
        base = Path(__file__).resolve().parents[1] / 'local/research/l10n-utf8-codex'
        paths = [('control-03', True), ('glyph-03', False), ('glyph-04', False)]
        if not all((base / run / 'native/A/dialog.png').exists() for run, _ in paths):
            self.skipTest('private screenshot samples unavailable')
        for run, expected in paths:
            with self.subTest(run=run):
                self.assertEqual(dialog_candidate(base / run / 'native/A/dialog.png'), expected)

    def test_wrong_dimensions_do_not_match(self):
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / 'small.png'
            Image.new('RGB', (8, 8)).save(path)
            self.assertFalse(dialog_candidate(path))


if __name__ == '__main__': unittest.main()
