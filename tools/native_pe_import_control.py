"""Create a local PE32 import-loader control, without changing the source.

The repaired Wine diagnostic image declares imports through OriginalFirstThunk
but stores zero in every FirstThunk slot. This experiment copies only declared
lookup values to their corresponding IAT slots. An explicit second control can
reset the inherited bootstrap data flag already reset by the Wine recipe.
Neither control is a native acceptance claim or a distributable game binary.
"""
import argparse
import hashlib
import json
from pathlib import Path
import struct

import pefile

ROOT = Path(__file__).resolve().parents[1]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--source', type=Path, required=True)
    parser.add_argument('--source-sha256', required=True)
    parser.add_argument('--out', type=Path, required=True)
    parser.add_argument('--reset-bootstrap-global', action='store_true',
                        help='Control for inherited 1A966FC flag, already reset in the measured Wine recipe')
    args = parser.parse_args()
    local = (ROOT / 'local').resolve(strict=True)
    source = args.source.resolve(strict=True)
    out = args.out.resolve()
    if any(path == local or not path.is_relative_to(local) for path in [source, out]):
        parser.error('Input/output phải ở local/')
    if out.exists():
        parser.error('Output phải mới')
    before = source.read_bytes()
    if hashlib.sha256(before).hexdigest() != args.source_sha256:
        parser.error('Source identity mismatch')
    pe = pefile.PE(data=before)
    if pe.OPTIONAL_HEADER.Magic != 0x10B or pe.FILE_HEADER.Machine != 0x14C:
        parser.error('Control chỉ nhận PE32 x86')
    image = bytearray(before)
    changes = []
    offsets = set()
    for descriptor in pe.DIRECTORY_ENTRY_IMPORT:
        lookup = descriptor.struct.OriginalFirstThunk
        if not lookup or not descriptor.struct.FirstThunk:
            parser.error('Explicit lookup and IAT are required')
        for index, imp in enumerate(descriptor.imports):
            rva = descriptor.struct.FirstThunk + index * 4
            if imp.address != pe.OPTIONAL_HEADER.ImageBase + rva:
                parser.error('Non-contiguous import declaration')
            offset = pe.get_offset_from_rva(rva)
            old = struct.unpack_from('<I', before, offset)[0]
            value = pe.get_dword_at_rva(lookup + index * 4)
            if offset in offsets or old != 0 or not value:
                parser.error('Control requires unique, zero IAT slots and nonzero lookups')
            offsets.add(offset)
            struct.pack_into('<I', image, offset, value)
            changes.append(dict(file_offset=offset, iat_va=hex(imp.address), old=old,
                                new=value, dll=descriptor.dll.decode('ascii'),
                                name=imp.name.decode('ascii') if imp.name else None,
                                ordinal=imp.ordinal))
        if pe.get_dword_at_rva(lookup + len(descriptor.imports) * 4) != 0:
            parser.error('Import lookup terminator missing')
    if not changes:
        parser.error('No imports declared')
    # Re-reading this sparse repaired IAT with pefile's eager parser hits its
    # global 8192-import safety cap: each one-symbol descriptor also scans the
    # adjacent IAT entries. Check the unchanged declarations directly instead;
    # do not treat the capped parser inventory as native loader evidence.
    allowed_bytes = {offset + index for offset in offsets for index in range(4)}
    if any(a != b and index not in allowed_bytes
           for index, (a, b) in enumerate(zip(before, image))):
        raise ValueError('Control changed bytes outside the declared IAT')
    for descriptor in pe.DIRECTORY_ENTRY_IMPORT:
        for index, imp in enumerate(descriptor.imports):
            name_rva = pe.get_dword_at_rva(descriptor.struct.OriginalFirstThunk + index * 4)
            offset = pe.get_offset_from_rva(imp.address - pe.OPTIONAL_HEADER.ImageBase)
            if struct.unpack_from('<I', image, offset)[0] != name_rva:
                raise ValueError('Lookup-to-IAT mapping changed')
    inherited_reset = None
    if args.reset_bootstrap_global:
        if args.source_sha256 != '37655a36d11b94dbe18bcfdcaaf68ec92c9271743d8e3fdf919492e8f0edaa84':
            parser.error('Bootstrap reset locator requires the measured vi source identity')
        offset = pe.get_offset_from_rva(0x1A966FC - pe.OPTIONAL_HEADER.ImageBase)
        if struct.unpack_from('<I', before, offset)[0] != 1:
            parser.error('Inherited bootstrap flag is not the expected 1')
        struct.pack_into('<I', image, offset, 0)
        inherited_reset = dict(va='0x1a966fc', file_offset=offset, old=1, new=0,
                               provenance='Inherited copied-image state reset in CP012/G1 Wine recipe; '
                                          'file initialization control requires separate native verification')
    out.mkdir(parents=False, mode=0o700)
    target = out / 'CA-windows-import-control.exe'
    target.write_bytes(image)
    report = dict(schema='kre-native-pe-import-control-v1', source_sha256=args.source_sha256,
                  result_sha256=hashlib.sha256(image).hexdigest(), bytes=len(image),
                  descriptor_count=len(pe.DIRECTORY_ENTRY_IMPORT), changes=changes,
                  inherited_bootstrap_reset=inherited_reset,
                  changed_bytes=sum(a != b for a, b in zip(before, image)),
                  parser_limit='Sparse adjacent IAT makes eager pefile reparse exceed its '
                               '8192-import cap. Declaration bytes/INT remain identical; '
                               'this is not native Windows loader verification.',
                  limits='Local PE loader experiment only. No hook/code/language patch; '
                         'the optional inherited bootstrap data reset is declared separately. '
                         'Windows startup and gameplay must be measured separately. '
                         'Rollback uses the unchanged source bytes.')
    (out / 'import-control.json').write_text(json.dumps(report, indent=2) + '\n')
    print('Local import-loader control created; native verification pending.')


if __name__ == '__main__':
    main()
