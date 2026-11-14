"""Exercise exact matching, stack offsets, and the finite observer budget."""
import contextlib
import io
from pathlib import Path
import struct
import sys
import types
import unittest
from unittest import mock


class DrawTraceTests(unittest.TestCase):
    def setUp(self):
        self.memory = bytearray(0x5000)
        self.breakpoints = []
        outer = self
        fake = types.ModuleType('gdb')
        class Breakpoint:
            def __init__(self, address, internal=False):
                self.address, self.enabled = address, True
                outer.breakpoints.append(self)
        fake.Breakpoint = Breakpoint
        fake.parse_and_eval = lambda name: 0x1000
        fake.selected_thread = lambda: types.SimpleNamespace(global_num=7)
        fake.selected_inferior = lambda: types.SimpleNamespace(read_memory=lambda address, size: self.memory[address:address + size])
        self.output = io.StringIO()
        self.module = {'__name__': 'draw_trace_test'}
        with mock.patch.dict(sys.modules, {'gdb': fake}), contextlib.redirect_stdout(self.output):
            exec(compile(Path(__file__).with_name('l10n_utf8_draw_trace_probe.py').read_text(), 'draw_trace', 'exec'), self.module)

    def put(self, address, value):
        self.memory[address:address + 4] = struct.pack('<I', value)

    def test_width_caller_and_each_candidate(self):
        source = next(iter(self.module['SOURCES']))
        self.memory[0x3000:0x3000 + len(source) + 1] = source + b'\0'
        for index, src_offset, len_offset in ((0, 12, 16), (1, 24, 28), (2, 20, 24)):
            self.put(0x1000, 0x14bae45 if index == 0 else 0x4567)
            self.put(0x1000 + src_offset, 0x3000)
            self.put(0x1000 + len_offset, 0xffffffff)
            self.put(0x1824, 0x9876)
            with contextlib.redirect_stdout(self.output): self.breakpoints[index].stop()
            self.assertFalse(self.breakpoints[index].enabled)
        self.assertEqual(self.output.getvalue().count('"kind": "match"'), 3)
        self.assertIn('"width_caller": "0x9876"', self.output.getvalue())

    def test_unrelated_and_budget(self):
        source = next(iter(self.module['SOURCES']))
        self.put(0x1000, 0x14bae45)
        self.put(0x100c, 0x3000)
        self.put(0x1010, len(source))
        self.memory[0x3000:0x3000 + len(source)] = b'X' * len(source)
        with contextlib.redirect_stdout(self.output): self.breakpoints[0].stop()
        self.assertTrue(all(bp.enabled for bp in self.breakpoints))
        self.module['stops'] = self.module['MAX_STOPS'] - 1
        with contextlib.redirect_stdout(self.output): self.breakpoints[0].stop()
        self.assertTrue(all(not bp.enabled for bp in self.breakpoints))
        self.assertIn('"kind": "budget_exhausted"', self.output.getvalue())
        self.assertNotIn('"kind": "match"', self.output.getvalue())


if __name__ == '__main__': unittest.main()
