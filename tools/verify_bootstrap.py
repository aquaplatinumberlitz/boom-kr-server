"""Check KR native bootstrap length/copy functions with controlled synthetic inputs.

This is not a packet capture or a complete bootstrap/session oracle. The only
replaced function is the buffer allocator; parser and copy bytes run unchanged.
"""
import argparse
import hashlib
import importlib.metadata
import json
import struct
from pathlib import Path

TARGET_SHA = "a08950c833d3acab8281049def7b3dc96b221f107eeda01da3a548e34b91dce5"
BASE, STACK, OBJECT, RETURN = 0x400000, 0x10000000, 0x20000000, 0x30000000
BUFFER, DESTINATION = OBJECT + 0x1000, OBJECT + 0x20000


def verify(binary):
    from unicorn import Uc, UC_ARCH_X86, UC_MODE_32, UC_HOOK_CODE
    from unicorn.x86_const import UC_X86_REG_EAX, UC_X86_REG_ECX, UC_X86_REG_ESP, UC_X86_REG_EIP

    data = binary.read_bytes()
    if hashlib.sha256(data).hexdigest() != TARGET_SHA:
        raise ValueError("requires the recorded modO memory image")
    machine = Uc(UC_ARCH_X86, UC_MODE_32)
    machine.mem_map(BASE, (len(data) + 4095) & ~4095)
    machine.mem_write(BASE, data)
    for address, size in ((STACK, 0x40000), (OBJECT, 0x40000), (RETURN, 0x1000)):
        machine.mem_map(address, size)
    machine.mem_map(0, 0x1000)  # SEH chain used by the original string reader.
    allocations = []

    def allocator(mu, address, size, context):
        # 14F5790(this=object+4, requested): retain an already allocated buffer.
        esp = mu.reg_read(UC_X86_REG_ESP)
        target, requested = struct.unpack("<II", mu.mem_read(esp, 8))
        if mu.reg_read(UC_X86_REG_ECX) != OBJECT + 4 or requested > 65535:
            raise AssertionError("unexpected allocator context")
        allocations.append(requested)
        mu.reg_write(UC_X86_REG_EAX, 1)
        mu.reg_write(UC_X86_REG_ESP, esp + 8)
        mu.reg_write(UC_X86_REG_EIP, target)

    machine.hook_add(UC_HOOK_CODE, allocator, begin=0x14F5790, end=0x14F5790)

    def reset():
        machine.mem_write(OBJECT, bytes(0x100))
        machine.mem_write(OBJECT, struct.pack("<I", 0x17B0D18))
        machine.mem_write(OBJECT + 0x10, struct.pack("<I", DESTINATION))
        machine.mem_write(OBJECT + 0x2C, struct.pack("<I", 2))
        machine.mem_write(DESTINATION - 4, b"\xCC" * (65535 + 8))
        allocations.clear()

    def call(address, arguments, this=OBJECT):
        pointer = STACK + 0x30000
        machine.mem_write(pointer, struct.pack("<" + "I" * (len(arguments) + 1), RETURN, *arguments))
        machine.reg_write(UC_X86_REG_ESP, pointer)
        machine.reg_write(UC_X86_REG_ECX, this)
        machine.emu_start(address, RETURN, count=300000)
        if machine.reg_read(UC_X86_REG_EIP) != RETURN:
            raise AssertionError("native function exceeded instruction budget")
        return machine.reg_read(UC_X86_REG_EAX)

    rows = []
    # Constructor selector2 is overwritten by virtual+8 on the ProcessPatch path.
    # Verify both the standalone constructor case and setter(0,0) case explicitly.
    for selector in (0, 2):
        for prefix in ((b"\0", b"\xAA") if selector == 0 else (b"\0\0", b"\xAA\xBB")):
            for length in (0, 1, 2, 4, 11, 967, 0xC000, 0xFFFE):
                reset()
                call(0x14EEFA0, [0, selector])
                if bytes(machine.mem_read(OBJECT + 0x28, 8)) != struct.pack("<II", 0, selector):
                    raise AssertionError("response mode/selector setter mismatch")
                payload = bytes(index % 256 for index in range(length))
                frame = prefix + struct.pack(">H", length) + payload
                machine.mem_write(BUFFER, frame)
                header_length = len(prefix) + 2
                for available in (0, 1, 2, 3, 4, len(frame)):
                    actual = call(0x14EEB40, [BUFFER, available])
                    expected = 0xFFFFFFFF if available < header_length else len(frame)
                    if actual != expected:
                        raise AssertionError("length parser mismatch")
                    rows.append({"operation": "length", "selector": selector, "prefix": prefix.hex(),
                                 "payload_length": length, "available": available, "returned": actual})
                actual = call(0x14EDC20, [BUFFER, len(frame), 0, 0, 0])
                fields = struct.unpack("<9I", machine.mem_read(OBJECT + 0x14, 36))
                if length == 0:
                    if actual != 0 or fields[8] != 1 or allocations:
                        raise AssertionError(("empty frame rejection mismatch", selector, actual, fields, allocations))
                else:
                    copied = bytes(machine.mem_read(DESTINATION - 4, length + 8))
                    # Selector2 peeks two bytes even for a one-byte body.
                    # This case establishes copying only, not a valid packet.
                    if actual != 1 or fields[3:5] != (length, 0) or fields[8] != 0:
                        raise AssertionError("copy state mismatch")
                    if copied != b"\xCC" * 4 + payload + b"\xCC" * 4 or allocations != [length]:
                        raise AssertionError("payload/guard/allocation mismatch")
                rows.append({"operation": "copy", "selector": selector, "prefix": prefix.hex(),
                             "payload_length": length, "returned": actual,
                             "error": fields[8], "allocations": list(allocations)})
    receive_rows = []
    socket_object = OBJECT + 0x18000
    for prefix in (0, 0x69):
        for header_check in (0, 1):
            for text in (b"", b"hello", b"123456789ABCDEF"):
                reset()
                expected_word = 0xABCD if text else 0
                expected_opaque = 0x89ABCDEF if text else 0
                expected_tail = 0xA5 if text else 0
                payload = b"\0" + struct.pack(">HHI", expected_word, 967, expected_opaque) + struct.pack(">H", len(text)) + text + bytes([expected_tail])
                frame = bytes([prefix]) + struct.pack(">H", len(payload)) + payload
                machine.mem_write(BUFFER, frame)
                machine.mem_write(socket_object, bytes(0x100))
                settings = {0: 0x178D800, 0x1C: 1, 0x40: 0, 0x44: 0,
                            0x50: 0x66, 0x54: 0xC0, 0x58: 0xE7, 0x5C: 40, 0x60: 40,
                            0x64: 3, 0x68: 3, 0x6C: 0x100000, 0x7C: BUFFER,
                            0x98: len(frame), 0xAC: header_check}
                for offset, value in settings.items():
                    machine.mem_write(socket_object + offset, struct.pack("<I", value))
                received = call(0x14EF300, [OBJECT], socket_object)
                guard = call(0x14EDF50, [])
                first_word = call(0x14ED750, []) & 0xFFFF
                version = call(0x14ED750, []) & 0xFFFF
                opaque = call(0x14ED7E0, [])
                string_object = OBJECT + 0x14000
                call(0x14EDF90, [string_object])
                actual_text = bytes(machine.mem_read(string_object, len(text)))
                string_size = struct.unpack("<I", machine.mem_read(string_object + 0x10, 4))[0]
                tail = call(0x14ED6C0, []) & 255
                cursor = struct.unpack("<I", machine.mem_read(OBJECT + 0x24, 4))[0]
                error = struct.unpack("<I", machine.mem_read(OBJECT + 0x30, 4))[0]
                remaining = struct.unpack("<I", machine.mem_read(socket_object + 0x98, 4))[0]
                seed = struct.unpack("<I", machine.mem_read(socket_object + 0x48, 4))[0]
                if (received, first_word, version, opaque, tail, cursor, error, remaining) != (0, expected_word, 967, expected_opaque, expected_tail, len(payload), 0, 0):
                    raise AssertionError("receive/reader state mismatch")
                if actual_text != text or string_size != len(text):
                    raise AssertionError("native string reader mismatch")
                if (guard == 0) != (not header_check or prefix == 0x69):
                    raise AssertionError("prefix seed/guard mismatch")
                receive_rows.append({"prefix": prefix, "header_check_enabled": header_check,
                                     "string_hex": text.hex(), "seed": seed, "guard": guard,
                                     "first_word": first_word, "version": version, "opaque": opaque, "tail": tail,
                                       "final_cursor": cursor, "reader_error": error})
    stream_rows = []
    frame = bytes.fromhex("69000c00000003c700000000000000")
    for available in range(len(frame) + 1):
        reset()
        machine.mem_write(BUFFER, frame)
        machine.mem_write(socket_object, bytes(0x100))
        settings = {0: 0x178D800, 0x1C: 1, 0x6C: 0x100000, 0x7C: BUFFER, 0x98: available}
        for offset, value in settings.items():
            machine.mem_write(socket_object + offset, struct.pack("<I", value))
        actual = call(0x14EF300, [OBJECT], socket_object)
        expected = 0 if available == len(frame) else 2
        if actual != expected or allocations != ([12] if expected == 0 else []):
            raise AssertionError("fragmented receive mismatch")
        stream_rows.append({"available": available, "returned": actual})
    reset()
    machine.mem_write(BUFFER, frame + frame)
    machine.mem_write(socket_object, bytes(0x100))
    for offset, value in {0: 0x178D800, 0x1C: 1, 0x6C: 0x100000, 0x7C: BUFFER, 0x98: 2 * len(frame)}.items():
        machine.mem_write(socket_object + offset, struct.pack("<I", value))
    for remaining in (len(frame), 0):
        actual = call(0x14EF300, [OBJECT], socket_object)
        actual_remaining = struct.unpack("<I", machine.mem_read(socket_object + 0x98, 4))[0]
        if actual != 0 or actual_remaining != remaining:
            raise AssertionError("coalesced receive mismatch")
        stream_rows.append({"coalesced_remaining": remaining, "returned": actual})
    return {"schema": "kre-bootstrap-native-r1", "source": "kr_static", "status": "confirmed",
            "target_sha256": TARGET_SHA, "image_base": hex(BASE),
            "unicorn_version": importlib.metadata.version("unicorn"),
            "scope": "mode0 selectors0/2 length/copy; synthetic object plus native14EEFA0 setter",
            "replacement": "14F5790 allocator succeeds using a preallocated 65535-byte buffer",
            "limitations": ["Synthetic inputs, not original server frames or runtime traffic.",
                            "Receive cases use synthetic socket fields; no live mode, identity, hello or original-server semantics proof.",
                            "Does not test allocator failure or a full ProcessPatch invocation."],
            "cases": rows, "receive_cases": receive_rows, "stream_cases": stream_rows,
            "case_count": len(rows) + len(receive_rows) + len(stream_rows)}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--memory", type=Path, required=True)
    parser.add_argument("--out", type=Path, required=True)
    args = parser.parse_args()
    if args.out.exists() or not args.out.parent.is_dir():
        parser.error("choose a new output file in an existing directory")
    result = verify(args.memory)
    with args.out.open("x") as output:
        output.write(json.dumps(result, indent=2) + "\n")
    print(json.dumps({"passed": result["case_count"], "out": str(args.out)}))


if __name__ == "__main__":
    main()
