"""Emulate isolated KR writer helpers; this does not run CA.exe or establish framing.

Requires an externally supplied unpacked image with the recorded digest and Unicorn.
All bytes are read from that image, never stored in the repository.
"""
import argparse
import hashlib
import importlib.metadata
import json
import struct
from datetime import datetime, timezone
from pathlib import Path

TARGET_SHA = "cf72da112308d9f1ca256df8089594c5f787875310933073bf6bd5ba6f70e63b"
BASE = 0x400000
CODE_PAGE = 0x14A5000
STACK = 0x10000000
BUFFER = 0x20000000
RETURN = 0x30000000


def pe_section_slice(data, address, length):
    if data[:2] != b"MZ":
        raise ValueError("not an MZ image")
    pe = struct.unpack_from("<I", data, 0x3C)[0]
    if data[pe:pe + 4] != b"PE\0\0":
        raise ValueError("not a PE image")
    machine, sections = struct.unpack_from("<HH", data, pe + 4)
    optional_size = struct.unpack_from("<H", data, pe + 20)[0]
    optional = pe + 24
    if machine != 0x14C or struct.unpack_from("<H", data, optional)[0] != 0x10B:
        raise ValueError("requires PE32 x86")
    image_base = struct.unpack_from("<I", data, optional + 28)[0]
    if image_base != BASE:
        raise ValueError("unexpected image base")
    rva = address - image_base
    for index in range(sections):
        at = optional + optional_size + index * 40
        _, virtual_address, raw_size, raw_offset = struct.unpack_from("<IIII", data, at + 8)
        if virtual_address <= rva and rva + length <= virtual_address + raw_size:
            start = raw_offset + rva - virtual_address
            result = data[start:start + length]
            if len(result) != length:
                raise ValueError("truncated section")
            return result
    raise ValueError("address range is not backed by one section")


def verify(binary):
    from unicorn import Uc, UC_ARCH_X86, UC_MODE_32
    from unicorn.x86_const import UC_X86_REG_ESP, UC_X86_REG_EIP

    data = binary.read_bytes()
    digest = hashlib.sha256(data).hexdigest()
    if digest != TARGET_SHA:
        raise ValueError("input digest differs from the recorded unpacked KR image")
    code = pe_section_slice(data, CODE_PAGE, 0x1000)
    emulator = Uc(UC_ARCH_X86, UC_MODE_32)
    for address in (CODE_PAGE, STACK, BUFFER, RETURN):
        emulator.mem_map(address, 0x1000)
    emulator.mem_write(CODE_PAGE, code)
    stack_pointer = STACK + 0x800
    guard_base = BUFFER + 0x100
    destination = guard_base + 2

    def call(address, value, expected):
        emulator.mem_write(guard_base, b"\xCC" * 8)
        emulator.mem_write(stack_pointer, struct.pack("<III", RETURN, destination, value))
        emulator.reg_write(UC_X86_REG_ESP, stack_pointer)
        emulator.emu_start(address, RETURN, count=100)
        if emulator.reg_read(UC_X86_REG_EIP) != RETURN:
            raise AssertionError(f"helper {address:#x} did not return within instruction limit")
        actual = bytes(emulator.mem_read(guard_base, 8))
        wanted = b"\xCC" * 2 + expected + b"\xCC" * (6 - len(expected))
        if actual != wanted:
            raise AssertionError(f"helper {address:#x}, value {value:#x}: output or guard mismatch")

    variants = [
        ("be16_plain", 0x14A5B90, 2, 0, 0xFFFF),
        ("be16_xor", 0x14A5C90, 2, 0xA569, 0xFFFF),
        ("byte_xor", 0x14A5C80, 1, 0x5A, 0xFF),
    ]
    results = []
    for name, address, width, xor_value, mask in variants:
        cases = 0
        for value in range(mask + 1):
            expected = ((value ^ xor_value) & mask).to_bytes(width, "big")
            call(address, value, expected)
            cases += 1
        for value in (0x10000, 0x7FFFFFFF, 0xDEADBEEF, 0xFFFFFFFF):
            call(address, value, ((value ^ xor_value) & mask).to_bytes(width, "big"))
            cases += 1
        results.append({"name": name, "va": hex(address), "cases": cases, "passed": True})
    return {
        "schema": "kre-primitive-verification-r1",
        "captured_utc": datetime.now(timezone.utc).isoformat(),
        "source": "kr_static",
        "method": "controlled emulation of original helper bytes",
        "binary_path": str(binary.resolve()),
        "binary_sha256": digest,
        "image_base": hex(BASE),
        "unicorn_version": importlib.metadata.version("unicorn"),
        "results": results,
        "limitations": [
            "Only isolated helper behavior is checked, not its selection or caller state.",
            "Does not establish header, checksum, crypto, sequence, hello or socket semantics.",
            "Does not execute the client bootstrap or observe a live KR connection.",
        ],
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=Path, required=True)
    parser.add_argument("--out", type=Path, required=True)
    args = parser.parse_args()
    if args.out.exists():
        parser.error("output already exists; choose a new run path")
    if not args.out.parent.is_dir():
        parser.error("output directory does not exist")
    result = verify(args.binary)
    args.out.write_text(json.dumps(result, indent=2) + "\n")
    print(json.dumps({"results": result["results"], "out": str(args.out)}))


if __name__ == "__main__":
    main()
