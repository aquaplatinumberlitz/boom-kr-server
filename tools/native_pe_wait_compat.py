"""Derive a private Windows compatibility copy from pinned diagnostic inputs.

Observed M8 Windows zero-limit control: normal dialog returns1 and sends67
after replacing only a zero wait limit with30000ms. This file trampoline makes
the same finite company policy explicit without a debugger or forced result.
No original executable, language hook, asset, server deadline or auth is changed.
Native acceptance of the resulting executable is still required.
An optional declared offline control omits the unrecovered external cleanup
trap observed by WER during normal stage destruction. It is never implicit.
Further optional controls restore the measured one-argument quit API, handle
only empty offline content, or omit a separately measured final external trap.
Each has its own declaration and native result; none certifies the whole client.
"""
import argparse
import hashlib
import json
from pathlib import Path
import struct

import pefile

ROOT = Path(__file__).resolve().parents[1]
HOOK = 0x110740c
RETURN = HOOK + 6
ORIGINAL = bytes.fromhex('8b4de4ff55e0')
CLEANUP_REPLACEMENT = b'\xc3'
CONTENT_ENTRY = 0x96a480
CONTENT_PREFIX = bytes.fromhex('558bec6aff')
KNOWN = {
    '348d26ed491f82788ab76342118831e0e9f8001a62e239047cd026890b0648a7': 'vi',
    'eb441bb15eac488bdb43fe6fecf0309d97f4063a92132212c14869fb22e7da6b': 'ko',
    '303f1ab1843dbaa74472fa463e27cecf7bf626662504d65187f7ff1c3a0dc426': 'en',
}


def align(value, size):
    if size <= 0 or size & (size - 1):
        raise ValueError('Invalid PE alignment')
    return (value + size - 1) & -size


def trampoline(address, milliseconds=30000):
    if milliseconds != 30000:
        raise ValueError('Only the measured finite30000ms policy is admitted')
    # Restore the stolen ECX load, preserve flags before invoking the original
    # indirect stdcall with the existing three arguments. EBP/other registers
    # and result stay untouched. A nonzero native wait limit is preserved.
    code = bytes.fromhex('8b4de4 9c 6683b96004000000 7509 66c78160040000')
    code += struct.pack('<H', milliseconds)
    code += bytes.fromhex('9d ff55e0')
    code += b'\xe9' + struct.pack('<i', RETURN - (address + len(code) + 5))
    return code


def empty_content_trampoline(address):
    """CP201 empty-only branch, same native constructor and nonempty prologue."""
    code = bytearray(bytes.fromhex('8b4424088378100075008b4c240468'))
    code += struct.pack('<I', 0x18943b8)
    code += b'\xe8' + struct.pack('<i', 0x447420 - (address + len(code) + 5))
    code += bytes.fromhex('8b442404c20800')
    code[9] = len(code) - 10
    code += CONTENT_PREFIX
    code += b'\xe9' + struct.pack('<i', CONTENT_ENTRY + 5 - (address + len(code) + 5))
    return bytes(code)


def quit_descriptor(pe):
    """Locate only the pinned repaired import; avoid pefile's sparse-IAT cap."""
    start = pe.OPTIONAL_HEADER.DATA_DIRECTORY[1].VirtualAddress
    found = []
    descriptors = []
    for index in range(2048):
        address = start + index * 20
        raw = pe.get_data(address, 20)
        if len(raw) != 20:
            raise ValueError('Truncated import descriptors')
        if not any(raw):
            break
        fields = struct.unpack('<IIIII', raw)
        descriptors.append(fields)
        if fields[4] == 0x16ea59c - 0x400000:
            found.append((address, fields))
    else:
        raise ValueError('Import descriptor bound')
    if len(found) != 1:
        raise ValueError('Quit import slot is not unique')
    address, (lookup, _, _, dll, slot) = found[0]
    if not lookup or sum(row[0] == lookup for row in descriptors) != 1:
        raise ValueError('Quit import lookup is missing/shared')
    name = struct.unpack('<I', pe.get_data(lookup, 4))[0]
    if (name & 0x80000000 or pe.get_string_at_rva(dll).lower() != b'win32u.dll'
            or pe.get_string_at_rva(name + 2) != b'NtUserCallOneParam'
            or pe.get_data(lookup + 4, 4) != bytes(4)
            or pe.get_data(slot, 4) != struct.pack('<I', name)):
        raise ValueError('Measured repaired quit import differs')
    native_exit = pe.get_data(0xd75b70 - 0x400000, 0x78)
    if hashlib.sha256(native_exit).hexdigest() != 'f8c64fc9faa76a82137111fa2fc01fef0dc7634409b4c1c1b543e98f8254d7c4':
        raise ValueError('Measured one-argument native exit function differs')
    return address, lookup, slot


def patch_bytes(source, expected_sha256, omit_unavailable_cleanup=False, restore_quit_message=False,
                guard_empty_content=False, omit_unavailable_exit_cleanup=False):
    actual = hashlib.sha256(source).hexdigest()
    if actual != expected_sha256 or actual not in KNOWN:
        raise ValueError('Expected pinned Windows import/bootstrap control identity')
    pe = pefile.PE(data=source, fast_load=True)
    if pe.FILE_HEADER.Machine != 0x14c or pe.OPTIONAL_HEADER.Magic != 0x10b:
        raise ValueError('Expected PE32 x86')
    if pe.OPTIONAL_HEADER.ImageBase != 0x400000 or pe.get_data(HOOK - 0x400000, 6) != ORIGINAL:
        raise ValueError('Measured wait-call bytes differ')
    if pe.OPTIONAL_HEADER.DATA_DIRECTORY[4].Size:
        raise ValueError('Certificate overlay is outside this copy-only recipe')
    quit_fields = quit_descriptor(pe) if restore_quit_message else None
    if guard_empty_content:
        for address, size, expected in [
            (CONTENT_ENTRY, 256, '55a2272d9f099afc3d9fa623d16ebd9044272302ab2ef350724c05323a261fab'),
            (0x447420, 128, '5215f1ad5381fdbb614fdb2d83b8e01c9d77c110941c9f7fea71602583645d77'),
        ]:
            if hashlib.sha256(pe.get_data(address - 0x400000, size)).hexdigest() != expected:
                raise ValueError('CP201 content parser/constructor bytes differ')
        if pe.get_data(0x18943b8 - 0x400000, 1) != b'\0':
            raise ValueError('Native empty content literal differs')
    exit_cleanup_offset = None
    if omit_unavailable_exit_cleanup:
        if (pe.get_data(0x806b30 - 0x400000, 18) != bytes.fromhex('558bec51894dfcff15eca46e01908be55dc3')
                or pe.get_data(0x16ea4ec - 0x400000, 4) != struct.pack('<I', 0x22638d4)
                or pe.get_data(0x22638d4 - 0x400000, 8) != bytes.fromhex('0f0beca46e01cccc')):
            raise ValueError('Measured final external cleanup call/slot/trap differ')
        exit_cleanup_offset = pe.get_offset_from_rva(0x22638d4 - 0x400000)
    cleanup_offset = None
    if omit_unavailable_cleanup:
        # WER r20: natural stage destruction calls1494A0E through16EA064,
        # whose unrecovered external target is this deliberate repair trap.
        # Historical DWK naming remains candidate. This explicit offline copy
        # policy omits that unavailable cleanup; it does not force stage success.
        if (pe.get_data(0x1494a0e - 0x400000, 6) != bytes.fromhex('ff2564a06e01')
                or pe.get_data(0x16ea064 - 0x400000, 4) != struct.pack('<I', 0x2260df8)
                or pe.get_data(0x2260df8 - 0x400000, 8) != bytes.fromhex('0f0b64a06e01cccc')):
            raise ValueError('Measured unavailable cleanup slot/thunk/trap differ')
        cleanup_offset = pe.get_offset_from_rva(0x2260df8 - 0x400000)
    section_header = pe.sections[0].get_file_offset() + pe.FILE_HEADER.NumberOfSections * 40
    if section_header + 40 > min(section.PointerToRawData for section in pe.sections):
        raise ValueError('No unused section-header space')
    if source[section_header:section_header + 40] != bytes(40):
        raise ValueError('Section header space is occupied')
    rva = align(max(section.VirtualAddress + max(section.Misc_VirtualSize, section.SizeOfRawData)
                    for section in pe.sections), pe.OPTIONAL_HEADER.SectionAlignment)
    raw_offset = align(len(source), pe.OPTIONAL_HEADER.FileAlignment)
    code = trampoline(0x400000 + rva)
    payload = code
    quit_names = None
    if quit_fields:
        payload += bytes(align(len(payload), 4) - len(payload))
        dll_rva = rva + len(payload)
        payload += b'user32.dll\0'
        payload += bytes(align(len(payload), 2) - len(payload))
        name_rva = rva + len(payload)
        payload += b'\0\0PostQuitMessage\0'
        quit_names = dll_rva, name_rva
    content_stub = None
    if guard_empty_content:
        payload += bytes(align(len(payload), 16) - len(payload))
        content_stub = 0x400000 + rva + len(payload)
        payload += empty_content_trampoline(content_stub)
    raw_size = align(len(payload), pe.OPTIONAL_HEADER.FileAlignment)
    image = bytearray(source)
    image.extend(bytes(raw_offset - len(image)))
    image.extend(payload + bytes(raw_size - len(payload)))
    image[section_header:section_header + 40] = struct.pack(
        '<8sIIIIIIHHI', b'.m8wait\0', len(payload), rva, raw_size, raw_offset, 0, 0, 0, 0, 0x60000020)
    changed = [(section_header, 40)]
    for owner, field, value, size in [
        (pe.FILE_HEADER, 'NumberOfSections', pe.FILE_HEADER.NumberOfSections + 1, 2),
        (pe.OPTIONAL_HEADER, 'SizeOfImage', align(rva + len(payload), pe.OPTIONAL_HEADER.SectionAlignment), 4),
        (pe.OPTIONAL_HEADER, 'SizeOfCode', pe.OPTIONAL_HEADER.SizeOfCode + raw_size, 4),
        (pe.OPTIONAL_HEADER, 'CheckSum', 0, 4),
    ]:
        offset = owner.get_field_absolute_offset(field)
        image[offset:offset + size] = value.to_bytes(size, 'little')
        changed.append((offset, size))
    hook_offset = pe.get_offset_from_rva(HOOK - 0x400000)
    replacement = b'\xe9' + struct.pack('<i', (0x400000 + rva) - (HOOK + 5)) + b'\x90'
    image[hook_offset:hook_offset + 6] = replacement
    changed.append((hook_offset, 6))
    if cleanup_offset is not None:
        image[cleanup_offset:cleanup_offset + 1] = CLEANUP_REPLACEMENT
        changed.append((cleanup_offset, 1))
    if exit_cleanup_offset is not None:
        image[exit_cleanup_offset:exit_cleanup_offset + 1] = CLEANUP_REPLACEMENT
        changed.append((exit_cleanup_offset, 1))
    if quit_fields:
        descriptor, lookup, slot = quit_fields
        dll_rva, name_rva = quit_names
        for address, value in [(descriptor + 12, dll_rva), (lookup, name_rva), (slot, name_rva)]:
            offset = pe.get_offset_from_rva(address)
            image[offset:offset + 4] = struct.pack('<I', value)
            changed.append((offset, 4))
    if content_stub:
        offset = pe.get_offset_from_rva(CONTENT_ENTRY - 0x400000)
        image[offset:offset + 5] = b'\xe9' + struct.pack('<i', content_stub - CONTENT_ENTRY - 5)
        changed.append((offset, 5))
    allowed = {offset + index for offset, size in changed for index in range(size)}
    if any(before != after and index not in allowed for index, (before, after) in enumerate(zip(source, image))):
        raise ValueError('Changed a byte outside the declared PE header/hook fields')
    result = bytes(image)
    verified = pefile.PE(data=result, fast_load=True)
    if verified.get_data(rva, len(code)) != code or verified.get_data(HOOK - 0x400000, 6) != replacement:
        raise ValueError('PE mapping does not match trampoline bytes')
    if verified.OPTIONAL_HEADER.AddressOfEntryPoint != pe.OPTIONAL_HEADER.AddressOfEntryPoint:
        raise ValueError('Owner language entry was changed')
    if quit_names:
        dll_rva, name_rva = quit_names
        if (verified.get_string_at_rva(dll_rva) != b'user32.dll'
                or verified.get_string_at_rva(name_rva + 2) != b'PostQuitMessage'):
            raise ValueError('Quit import strings do not map')
    report = dict(schema='kre-m8-native-wait-compat-v1', source='design', status='candidate',
                  language=KNOWN[actual], source_sha256=actual,
                  result_sha256=hashlib.sha256(result).hexdigest(), bytes=len(result),
                  hook_va=hex(HOOK), resume_va=hex(RETURN), original_bytes=ORIGINAL.hex(),
                  hook_bytes=replacement.hex(), stub_va=hex(0x400000 + rva), stub_bytes=code.hex(),
                  wait_limit_ms=30000, condition='Only if existing native u16 wait limit is zero',
                  preserved='Original language entry/imports/assets; native call/result, flags and nonzero limits',
                  limits='Private derived executable requires new Windows native acceptance; not original KR behavior or a published binary.')
    if cleanup_offset is not None:
        report['unavailable_cleanup'] = dict(source='design', status='candidate',
            slot_va='0x16ea064', thunk_va='0x1494a0e', target_va='0x2260df8',
            old_bytes='0f0b', new_bytes='c30b',
            policy='Omit unrecovered external cleanup on this offline copy; return without modifying registers or flags',
            limits='Historical UninstallDWK name is candidate. No original-source, language, asset, server authorization or stage-result change.')
    if quit_fields:
        report['quit_import_control'] = dict(source='design', status='candidate',
            slot_va='0x16ea59c', native_exit_va='0xd75b70',
            old='win32u.dll!NtUserCallOneParam', new='user32.dll!PostQuitMessage',
            condition='Pinned native exit calls this slot with one argument0; old repaired internal API has two arguments',
            limits='Explicit private ABI compatibility control; original API identity remains candidate until further evidence. Native message loop/process outcome must be measured; no direct ExitProcess or forced result.')
        report['preserved'] = 'Original language entry/assets, native exit call site and wait-call result/flags; additional import/content/cleanup changes declared separately'
    if content_stub:
        report['empty_content_control'] = dict(source='design', status='candidate',
            entry_va=hex(CONTENT_ENTRY), stub_va=hex(content_stub),
            stub_hex=empty_content_trampoline(content_stub).hex(), reuse='CP201 empty-only content guard',
            condition='Native input MSVC string size0 only',
            preserved='Nonempty input replays original five-byte prologue to96A485; native empty constructor447420 returns empty out, ret8',
            limits='Offline copy policy; not a fake JSON object, error-handler suppression, original behavior or proof that process exits cleanly.')
    if exit_cleanup_offset is not None:
        report['unavailable_final_cleanup'] = dict(source='design', status='candidate',
            slot_va='0x16ea4ec', caller_va='0x806b30', target_va='0x22638d4',
            old_bytes='0f0b', new_bytes='c30b',
            policy='Omit only this unrecovered external cleanup on the private offline copy; no explicit stack arguments, registers/flags unchanged',
            limits='Original external symbol and side effects unknown; preserves ECX but does not emulate the missing library. Requires independent native process/lifecycle evidence.')
    return result, report


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--source', type=Path, required=True)
    parser.add_argument('--source-sha256', required=True)
    parser.add_argument('--out', type=Path, required=True)
    parser.add_argument('--omit-unavailable-dwk-cleanup', action='store_true',
                        help='Explicit offline copy control for the measured16EA064 repair trap')
    parser.add_argument('--restore-quit-message', action='store_true',
                        help='Private one-argument PostQuitMessage import control for the measured native exit slot')
    parser.add_argument('--guard-empty-content', action='store_true',
                        help='Reuse the CP201 empty-only offline content guard after current input/code comparison')
    parser.add_argument('--omit-unavailable-exit-cleanup', action='store_true',
                        help='Explicit offline copy control for measured final16EA4EC repair trap only')
    args = parser.parse_args()
    local = (ROOT / 'local').resolve(strict=True)
    source, output = args.source.resolve(strict=True), args.out.resolve()
    if any(path == local or not path.is_relative_to(local) for path in [source, output]) or output.exists():
        parser.error('Pinned input and new output must be under local/')
    before = source.read_bytes()
    result, report = patch_bytes(before, args.source_sha256, args.omit_unavailable_dwk_cleanup,
                                 args.restore_quit_message, args.guard_empty_content,
                                 args.omit_unavailable_exit_cleanup)
    output.mkdir(mode=0o700)
    (output / 'CA-windows-wait-compat.exe').write_bytes(result)
    (output / 'wait-compat.json').write_text(json.dumps(report, indent=2) + '\n')
    if source.read_bytes() != before:
        raise RuntimeError('Source identity changed during copy operation')
    print(json.dumps(dict(result='private_copy_created', sha256=report['result_sha256'],
                          language=report['language'], native_acceptance='pending')))


if __name__ == '__main__':
    main()
