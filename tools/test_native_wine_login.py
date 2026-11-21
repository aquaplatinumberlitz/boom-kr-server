"""Dry controls for black startup, stale field content and click ordering."""
import unittest
from unittest.mock import patch

from PIL import Image

from native_wine_login import FIELDS, PANEL, sign_in, wait_for_panel


class WineLogin(unittest.TestCase):
    def setUp(self):
        self.ref = Image.new("RGB", (1024, 768), "black")
        self.ref.paste("orange", PANEL)
        for box in FIELDS:
            self.ref.paste("white", box)
        self.filled = self.ref.copy()
        for x, y, _, _ in FIELDS:
            self.filled.paste("black", (x + 3, y + 5, x + 33, y + 15))

    def test_black_window_does_not_open_gate(self):
        frames = iter([Image.new("RGB", self.ref.size), self.ref, self.ref])
        with patch("native_wine_login.time.sleep"):
            wait_for_panel(lambda: next(frames), self.ref)
        self.assertIsNone(next(frames, None))

    def test_failed_clear_does_not_submit_or_type(self):
        operations = []
        with self.assertRaisesRegex(RuntimeError, "did not clear"):
            sign_in(lambda: self.filled, self.ref, lambda *args: operations.append(args),
                    ("localuser", "localpass"), pause=lambda _: None)
        self.assertFalse(any(op[0] == "type" or op == ("mousemove", "514", "667")
                             for op in operations))

    def test_submit_follows_both_filled_fields(self):
        operations = []
        frames = iter([self.filled, self.ref, self.ref, self.filled,
                       self.filled, self.ref, self.ref, self.filled])
        sign_in(lambda: next(frames), self.ref, lambda *args: operations.append(args),
                ("localuser", "localpass"), pause=lambda _: None)
        typed = [i for i, op in enumerate(operations) if op[0] == "type"]
        self.assertEqual(len(typed), 2)
        self.assertGreater(operations.index(("mousemove", "514", "667")), typed[-1])


if __name__ == "__main__":
    unittest.main()
