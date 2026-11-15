"""Fake debugger checks for the read-only one-shot trace observer."""
import contextlib
import io
from pathlib import Path
import struct
import sys
import types
import unittest
from unittest import mock


class TraceTests(unittest.TestCase):
    def setUp(self):
        self.reg = {'$esp': 0x1000, '$eax': 6}; self.memory = {}; self.breakpoints = []
        fake = types.ModuleType('gdb')
        outer = self
        class Breakpoint:
            def __init__(self, address, internal=False):
                self.address = address; self.enabled = True; outer.breakpoints.append(self)
        fake.Breakpoint = Breakpoint
        fake.parse_and_eval = lambda name: self.reg[name]
        fake.selected_thread = lambda: types.SimpleNamespace(global_num=7)
        fake.selected_inferior = lambda: types.SimpleNamespace(read_memory=lambda address, size: self.memory[address][:size])
        self.output = io.StringIO(); self.module = {'__name__': 'trace_test'}
        with mock.patch.dict(sys.modules, {'gdb': fake}), contextlib.redirect_stdout(self.output):
            exec(compile(Path(__file__).with_name('l10n_utf8_trace_probe.py').read_text(), 'trace', 'exec'), self.module)

    def test_one_shot_and_exact_return_identity(self):
        source, name = next(iter(self.module['SOURCES'].items()))
        self.memory[0x1000] = struct.pack('<IIIII', 0x4567, 0x2000, 32, 0x3000, len(source))
        self.memory[0x3000] = source; self.memory[0x2000] = b'A\0' * 6
        entry = self.breakpoints[0]
        with contextlib.redirect_stdout(self.output): entry.stop()
        self.assertFalse(entry.enabled); ret = self.breakpoints[1]
        with contextlib.redirect_stdout(self.output): ret.stop()
        self.assertTrue(ret.enabled)
        self.reg['$esp'] = 0x1004
        with contextlib.redirect_stdout(self.output): ret.stop()
        self.assertFalse(ret.enabled)
        self.assertIn('"source": "' + name + '"', self.output.getvalue())
        self.assertIn('"return_site": "0x4567"', self.output.getvalue())
        self.assertIn('"kind": "return"', self.output.getvalue())

    def test_unrelated_input_is_ignored(self):
        source = next(iter(self.module['SOURCES']))
        self.memory[0x1000] = struct.pack('<IIIII', 0x4567, 0x2000, 32, 0x3000, len(source))
        self.memory[0x3000] = b'X' * len(source)
        with contextlib.redirect_stdout(self.output): self.breakpoints[0].stop()
        self.assertTrue(self.breakpoints[0].enabled)
        self.assertEqual(len(self.breakpoints), 1)


if __name__ == '__main__': unittest.main()
