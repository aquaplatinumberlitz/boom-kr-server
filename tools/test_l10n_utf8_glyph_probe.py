"""Exercise the exact-match guard with a fake debugger; not a native test."""
import contextlib
import io
from pathlib import Path
import struct
import sys
import types
import unittest
from unittest import mock


class FakeMemory:
    def __init__(self): self.regions = {}; self.writes = []
    def read_memory(self, address, size): return self.regions[address][:size]
    def write_memory(self, address, data): self.writes.append((address, bytes(data)))


class ProbeTests(unittest.TestCase):
    def setUp(self):
        self.memory = FakeMemory(); self.registers = {'$esp': 0x1000, '$eax': 0}; self.commands = []
        fake = types.ModuleType('gdb')
        class Breakpoint:
            def __init__(self, *args, **kwargs): pass
            def delete(self): pass
            def is_valid(self): return True
        fake.Breakpoint = fake.FinishBreakpoint = Breakpoint
        fake.newest_frame = lambda: object()
        fake.selected_inferior = lambda: self.memory
        fake.selected_thread = lambda: types.SimpleNamespace(global_num=7)
        fake.breakpoints = lambda: []
        fake.events = types.SimpleNamespace(exited=types.SimpleNamespace(connect=lambda fn: None))
        fake.parse_and_eval = lambda name: self.registers[name]
        fake.execute = lambda command, to_string=False: self.commands.append(command)
        self.output = io.StringIO()
        with mock.patch.dict(sys.modules, {'gdb': fake}), contextlib.redirect_stdout(self.output):
            self.module = {'__name__': 'glyph_test', 'L10N_GLYPH_MODE': 'replace'}
            code = Path(__file__).with_name('l10n_utf8_glyph_probe.py').read_text()
            exec(compile(code, 'glyph-probe', 'exec'), self.module)

    def test_exact_match_and_output_capacity(self):
        source = next(iter(self.module['SOURCES'])); wide = self.module['WIDE']
        expected = source.decode('cp949').encode('utf-16le')
        self.memory.regions[0x1000] = struct.pack('<I', 0x123456)
        self.memory.regions[0x1004] = struct.pack('<IIII', 0x2000, 10, 0x3000, len(source))
        self.memory.regions[0x3000] = source
        with contextlib.redirect_stdout(self.output):
            self.assertFalse(self.module['Entry']().stop())
            self.registers.update({'$esp': 0x1004, '$eax': len(expected)//2})
            self.memory.regions[0x2000] = expected + b'\0\0'
            self.assertFalse(self.module['Return'](0x123456, 0x1004, 7, 0x2000, 10, False, 'test', expected).stop())
        self.assertIn('"kind": "match"', self.output.getvalue())
        self.assertEqual(self.memory.writes, [(0x2000, wide + b'\x00\x00')])
        self.assertEqual(self.commands, ['set $eax = 9'])

    def test_rejects_short_buffer_and_failed_conversion(self):
        self.registers['$esp'] = 0x1004
        with contextlib.redirect_stdout(self.output):
            self.module['Return'](0x123456, 0x1004, 7, 0x2000, 9, False, 'test', b'A\0').stop()
            self.registers['$eax'] = 0
            self.memory.regions[0x2000] = b'A\0\0\0'
            self.module['Return'](0x123456, 0x1004, 7, 0x2000, 10, False, 'test', b'A\0').stop()
        self.assertEqual(self.memory.writes, [])
        self.assertEqual(self.commands, [])

    def test_nul_terminated_return_length(self):
        self.registers.update({'$esp': 0x1004, '$eax': 2}); self.memory.regions[0x2000] = b'A\0\0\0'
        with contextlib.redirect_stdout(self.output): self.module['Return'](0x123456, 0x1004, 7, 0x2000, 10, True, 'test', b'A\0').stop()
        self.assertEqual(self.commands, ['set $eax = 10'])

    def test_unrelated_input_never_matches(self):
        source = next(iter(self.module['SOURCES']))
        self.memory.regions[0x1000] = struct.pack('<I', 0x123456)
        self.memory.regions[0x1004] = struct.pack('<IIII', 0x2000, 10, 0x3000, len(source))
        self.memory.regions[0x3000] = b'X' * len(source)
        with contextlib.redirect_stdout(self.output): self.module['Entry']().stop()
        self.assertNotIn('"kind": "match"', self.output.getvalue())
        self.assertEqual(self.memory.writes, [])

    def test_each_known_title_matches(self):
        for source, name in self.module['SOURCES'].items():
            self.output.seek(0); self.output.truncate()
            self.memory.regions[0x1000] = struct.pack('<I', 0x123456)
            self.memory.regions[0x1004] = struct.pack('<IIII', 0x2000, 10, 0x3000, len(source))
            self.memory.regions[0x3000] = source
            with contextlib.redirect_stdout(self.output): self.module['Entry']().stop()
            self.assertIn('"input": "' + name + '"', self.output.getvalue())

    def test_wrong_stack_or_thread_never_writes(self):
        self.registers.update({'$esp': 0x1008, '$eax': 1}); self.memory.regions[0x2000] = b'A\0\0\0'
        with contextlib.redirect_stdout(self.output):
            self.module['Return'](0x123456, 0x1004, 7, 0x2000, 10, False, 'test', b'A\0').stop()
            self.registers['$esp'] = 0x1004
            self.module['Return'](0x123456, 0x1004, 8, 0x2000, 10, False, 'test', b'A\0').stop()
        self.assertEqual(self.memory.writes, [])

    def test_pass_through_keeps_memory_and_result(self):
        self.module['MODE'] = 'pass'
        self.registers.update({'$esp': 0x1004, '$eax': 1}); self.memory.regions[0x2000] = b'A\0\0\0'
        with contextlib.redirect_stdout(self.output):
            self.module['Return'](0x123456, 0x1004, 7, 0x2000, 10, False, 'test', b'A\0').stop()
        self.assertEqual(self.memory.writes, [])
        self.assertEqual(self.commands, [])
        self.assertIn('"kind": "pass_through"', self.output.getvalue())


if __name__ == '__main__': unittest.main()
