"""Mechanics of the x86 wait-call trampoline; not a native client oracle."""
import struct
import unittest

from unicorn import Uc, UC_ARCH_X86, UC_MODE_32, UC_HOOK_CODE
from unicorn.x86_const import UC_X86_REG_EAX, UC_X86_REG_EBP, UC_X86_REG_ECX, UC_X86_REG_EFLAGS, UC_X86_REG_ESP, UC_X86_REG_EIP

from native_pe_wait_compat import (CLEANUP_REPLACEMENT, CONTENT_ENTRY, HOOK, ORIGINAL, RETURN,
                                   empty_content_trampoline, patch_bytes, trampoline)


class WaitTrampoline(unittest.TestCase):
    def execute(self, code, initial_limit, returned):
        machine = Uc(UC_ARCH_X86, UC_MODE_32)
        for address, length in [(HOOK & ~4095, 4096), (0x2200000, 4096), (0x200000, 4096),
                                (0x250000, 4096), (0x300000, 8192)]:
            machine.mem_map(address, length)
        machine.mem_write(HOOK, code)
        machine.mem_write(0x2200000, trampoline(0x2200000))
        machine.mem_write(0x250000, b'\xb8' + struct.pack('<I', returned) + b'\xc2\x0c\0')
        machine.mem_write(0x200460, struct.pack('<H', initial_limit))
        frame, stack = 0x301000, 0x300f00
        machine.mem_write(frame - 0x1c, struct.pack('<I', 0x200000))
        machine.mem_write(frame - 0x20, struct.pack('<I', 0x250000))
        machine.mem_write(stack, struct.pack('<III', 0, 1, 1))
        for register, value in [(UC_X86_REG_EBP, frame), (UC_X86_REG_ESP, stack),
                                (UC_X86_REG_ECX, 0x1234), (UC_X86_REG_EFLAGS, 0x246)]:
            machine.reg_write(register, value)
        machine.emu_start(HOOK, RETURN, count=100)
        return (tuple(machine.reg_read(register) for register in [UC_X86_REG_EAX, UC_X86_REG_EBP,
                    UC_X86_REG_ESP, UC_X86_REG_ECX, UC_X86_REG_EFLAGS]),
                struct.unpack('<H', machine.mem_read(0x200460, 2))[0])

    def test_zero_only_change_preserves_stdcall_stack_flags_registers_and_arbitrary_result(self):
        hook = b'\xe9' + struct.pack('<i', 0x2200000 - (HOOK + 5)) + b'\x90'
        for limit in [0, 1, 10000, 30000, 65535]:
            for result in [0, 1, 2, 0xffffffff]:
                with self.subTest(limit=limit, result=result):
                    original_state, original_limit = self.execute(ORIGINAL, limit, result)
                    patched_state, patched_limit = self.execute(hook, limit, result)
                    self.assertEqual(patched_state, original_state)
                    self.assertEqual(original_limit, limit)
                    self.assertEqual(patched_limit, limit if limit else 30000)
                    self.assertEqual(patched_state[2], 0x300f00 + 12)

    def test_wrong_identity_and_unmeasured_policy_rejected(self):
        with self.assertRaises(ValueError):
            patch_bytes(b'untrusted input', '0' * 64)
        for milliseconds in [0, 1, 65535, 65536]:
            with self.assertRaises(ValueError):
                trampoline(0x2200000, milliseconds)

    def test_optional_no_argument_cleanup_preserves_inherited_state(self):
        for result in [0, 1, 2, 0xffffffff]:
            machine = Uc(UC_ARCH_X86, UC_MODE_32)
            machine.mem_map(0x100000, 4096)
            machine.mem_map(0x200000, 4096)
            machine.mem_write(0x100000, CLEANUP_REPLACEMENT)
            machine.mem_write(0x200000, struct.pack('<I', 0x100010))
            state = [(UC_X86_REG_EAX, result), (UC_X86_REG_EBP, 0x201000),
                     (UC_X86_REG_ECX, 0x12345678), (UC_X86_REG_EFLAGS, 0x246)]
            for register, value in state + [(UC_X86_REG_ESP, 0x200000)]:
                machine.reg_write(register, value)
            machine.emu_start(0x100000, 0x100010, count=2)
            for register, value in state:
                self.assertEqual(machine.reg_read(register), value)
            self.assertEqual(machine.reg_read(UC_X86_REG_ESP), 0x200004)

    def test_empty_content_uses_native_constructor_and_nonempty_keeps_parser_abi(self):
        # Independent ABI assertions from CP201; the native constructor is a
        # declared model here, not a full application/JSON parsing oracle.
        for size in [0, 1, 5, 7, 109]:
            with self.subTest(size=size):
                machine = Uc(UC_ARCH_X86, UC_MODE_32)
                for address in [CONTENT_ENTRY & ~4095, 0x2300000, 0x447000, 0x1894000]:
                    machine.mem_map(address, 4096)
                machine.mem_map(0x300000, 0x10000)
                machine.mem_write(0x2300000, empty_content_trampoline(0x2300000))
                machine.mem_write(CONTENT_ENTRY, b'\xe9' + struct.pack('<i', 0x2300000 - CONTENT_ENTRY - 5))
                machine.mem_write(0x18943b8, b'\0')
                stack, output, text, returned = 0x308000, 0x301000, 0x302000, 0x303000
                machine.mem_write(stack, struct.pack('<III', returned, output, text))
                machine.mem_write(text, b'X' * 16 + struct.pack('<II', size, 15))
                machine.mem_write(output, b'Z' * 24)
                machine.reg_write(UC_X86_REG_ESP, stack)
                machine.reg_write(UC_X86_REG_EBP, 0x11223344)
                machine.reg_write(UC_X86_REG_ECX, 0x304000)
                calls, stops = [], []

                def observe(cpu, address, length, opaque):
                    if address == 0x447420:
                        sp = cpu.reg_read(UC_X86_REG_ESP)
                        back, literal = struct.unpack('<II', cpu.mem_read(sp, 8))
                        self.assertEqual(cpu.reg_read(UC_X86_REG_ECX), output)
                        self.assertEqual(bytes(cpu.mem_read(literal, 1)), b'\0')
                        calls.append(address)
                        cpu.mem_write(output, bytes(16) + struct.pack('<II', 0, 15))
                        cpu.reg_write(UC_X86_REG_EAX, output)
                        cpu.reg_write(UC_X86_REG_ESP, sp + 8)
                        cpu.reg_write(UC_X86_REG_EIP, back)
                    elif address in [returned, CONTENT_ENTRY + 5]:
                        stops.append(address)
                        cpu.emu_stop()

                machine.hook_add(UC_HOOK_CODE, observe)
                machine.emu_start(CONTENT_ENTRY, 0, count=60)
                if size == 0:
                    self.assertEqual(stops, [returned])
                    self.assertEqual(calls, [0x447420])
                    self.assertEqual(machine.reg_read(UC_X86_REG_ESP), stack + 12)
                    self.assertEqual(machine.reg_read(UC_X86_REG_EBP), 0x11223344)
                    self.assertEqual(machine.reg_read(UC_X86_REG_EAX), output)
                else:
                    self.assertEqual(stops, [CONTENT_ENTRY + 5])
                    self.assertEqual(calls, [])
                    self.assertEqual(machine.reg_read(UC_X86_REG_ESP), stack - 8)
                    self.assertEqual(machine.reg_read(UC_X86_REG_EBP), stack - 4)
                    self.assertEqual(machine.reg_read(UC_X86_REG_ECX), 0x304000)
                    self.assertEqual(bytes(machine.mem_read(output, 24)), b'Z' * 24)


if __name__ == '__main__':
    unittest.main()
