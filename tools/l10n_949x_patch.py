#!/usr/bin/env python3
"""Tạo bản sao exe CA-r2-* có hook 949x (section .l10n mới, entry point mới).

Chỉ ghi file vào thư mục --out dưới local/. Exe nguồn phải đúng SHA-256 đã đo; địa chỉ IAT
và OEP lấy từ chính PE nguồn và được kiểm trước khi ghi.
"""
import argparse
import hashlib
import json
import struct
import subprocess
import sys
from pathlib import Path

import pefile

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import l10n_949x as X  # noqa: E402

SOURCE_SHA256 = '4b781fbcea21e0f6294c729b63be86ab2953d60ad83b0d0c9219359f826107ed'  # CA-r2-stage-social-cmp.exe
WANT = {'MultiByteToWideChar': 'IAT_MB2WC', 'WideCharToMultiByte': 'IAT_WC2MB', 'PeekMessageA': 'IAT_PEEKMESSAGEA',
        'GetProcAddress': 'IAT_GETPROCADDRESS', 'GetModuleHandleA': 'IAT_GETMODULEHANDLEA',
        'ImmGetCompositionStringA': 'IAT_IMMGETCOMPOSITIONSTRINGA', 'ImmGetCompositionStringW': 'IAT_IMMGETCOMPOSITIONSTRINGW'}


def literal_blob(base_va, catalog, lang):
    """Chuỗi cứng của exe theo ngôn ngữ dựng: trả (bytes, [(operand_va, old_va, new_va)], báo cáo)."""
    import csv
    table = json.loads((HERE / 'l10n_949x_exe_literals.json').read_text())
    rows = {r['id']: r for r in csv.DictReader(catalog.open(encoding='utf-8', newline=''))}
    blob, patches, report = bytearray(), [], []
    for lit in table['literals']:
        row = rows[lit['id']]
        text = row[lang] if lang == 'vi' and row.get('vi_status') in ('translated', 'reused') else (
            row['en'] if lang == 'en' and row.get('en_status') not in ('', 'missing') and row['en'] else '')
        if not text or text == row['ko']:
            continue
        data = X.encode(text, 'replace') + b'\0'
        new_va = base_va + len(blob)
        blob += data + b'\0' * (-len(data) % 4)
        for ref in lit['refs']:
            patches.append((int(ref['operand'], 16), int(lit['va'], 16), new_va))
        report.append({'id': lit['id'], 'bytes': len(data) - 1, 'ko_bytes': lit['ko_cp949_len'], 'refs': len(lit['refs'])})
    return bytes(blob), patches, report


def build(src, out, lang=None, catalog=None, trace=False):
    data = src.read_bytes()
    digest = hashlib.sha256(data).hexdigest()
    if digest != SOURCE_SHA256:
        raise SystemExit('exe nguồn sai hash ' + digest)
    pe = pefile.PE(data=data)
    base = pe.OPTIONAL_HEADER.ImageBase
    oep = base + pe.OPTIONAL_HEADER.AddressOfEntryPoint
    slots = {}
    for entry in pe.DIRECTORY_ENTRY_IMPORT:
        for imp in entry.imports:
            if imp.name and imp.name.decode() in WANT:
                slots[WANT[imp.name.decode()]] = imp.address
    assert len(slots) == len(WANT), slots
    align = pe.OPTIONAL_HEADER.SectionAlignment
    falign = pe.OPTIONAL_HEADER.FileAlignment
    last = pe.sections[-1]
    rva = (last.VirtualAddress + last.Misc_VirtualSize + align - 1) // align * align
    va = base + rva
    work = out / 'build'
    work.mkdir(parents=True, exist_ok=True)
    header = work / 'l10n_949x_table.h'
    X.c_table(header)
    with header.open('a') as f:
        f.write(''.join('#define %s 0x%x\n' % kv for kv in sorted(slots.items())) + '#define OEP 0x%x\n' % oep)
    obj = work / 'hook.o'
    elf = work / 'hook.elf'
    blob = work / 'hook.bin'
    script = work / 'link.ld'
    script.write_text('SECTIONS {\n . = 0x%x;\n .all : { *(.text*) *(.rodata*) *(.data*) *(.bss*) *(COMMON) BYTE(0) }\n'
                      ' /DISCARD/ : { *(.comment) *(.note*) *(.eh_frame*) }\n}\n' % va)
    subprocess.run(['clang', '--target=i386-none-elf', '-O2', '-ffreestanding', '-fno-pic', '-fno-builtin',
                    '-fno-stack-protector', '-fno-asynchronous-unwind-tables', '-mno-sse', '-Wall', '-Werror',
                    *(['-DL10N_TRACE'] if trace else []),
                    '-I', str(work), '-c', str(HERE / 'l10n_949x_hook.c'), '-o', str(obj)], check=True)
    subprocess.run(['ld.lld', '-m', 'elf_i386', '-T', str(script), '-o', str(elf), str(obj)], check=True)
    subprocess.run(['llvm-objcopy', '-O', 'binary', '-j', '.all', str(elf), str(blob)], check=True)
    syms = {}
    for line in subprocess.run(['llvm-nm', str(elf)], check=True, capture_output=True, text=True).stdout.splitlines():
        parts = line.split()
        if len(parts) == 3:
            syms[parts[2]] = int(parts[0], 16)
    code = blob.read_bytes()
    code += b'\0' * (-len(code) % 16)
    literal_report, patches = [], []
    if lang:
        lit, patches, literal_report = literal_blob(va + len(code), catalog, lang)
        code += lit
    vsize = len(code)
    rsize = (vsize + falign - 1) // falign * falign
    # Thêm section header mới.
    off = pe.sections[-1].get_file_offset() + 40
    if off + 40 > pe.OPTIONAL_HEADER.SizeOfHeaders or any(pe.__data__[off:off + 40]):
        raise SystemExit('không còn chỗ cho section header')
    raw_ptr = (len(pe.__data__) + falign - 1) // falign * falign
    sec = struct.pack('<8sIIIIIIHHI', b'.l10n', vsize, rva, rsize, raw_ptr, 0, 0, 0, 0, 0xE0000060)
    pe.set_bytes_at_offset(off, sec)
    pe.FILE_HEADER.NumberOfSections += 1
    pe.OPTIONAL_HEADER.SizeOfImage = (rva + vsize + align - 1) // align * align
    pe.OPTIONAL_HEADER.AddressOfEntryPoint = syms['l10n_entry'] - base
    image = bytearray(pe.write())
    for operand, old, new in patches:
        off = pe.get_offset_from_rva(operand - base)
        if struct.unpack_from('<I', image, off)[0] != old:
            raise SystemExit('operand %#x không trỏ chuỗi %#x' % (operand, old))
        struct.pack_into('<I', image, off, new)
    image += b'\0' * (raw_ptr - len(image))
    image += code + b'\0' * (rsize - vsize)
    tag = ('-' + lang if lang else '') + ('-trace' if trace else '')
    exe = out / ('CA-r2-l10n-949x%s.exe' % tag)
    exe.write_bytes(image)
    check = pefile.PE(str(exe))
    assert check.sections[-1].Name.rstrip(b'\0') == b'.l10n'
    meta = {'source': str(src), 'source_sha256': digest, 'exe': str(exe),
            'exe_sha256': hashlib.sha256(image).hexdigest(), 'section_va': hex(va), 'section_bytes': vsize,
            'oep': hex(oep), 'entry': hex(syms['l10n_entry']), 'iat': {k: hex(v) for k, v in slots.items()},
            'symbols': {k: hex(v) for k, v in syms.items() if k.startswith(('hook_', 'orig_', 'l10n_'))},
            'table': {'format': '949x', 'version': X.VERSION, 'ext_chars': len(X.EXT)},
            'hook_c_sha256': hashlib.sha256((HERE / 'l10n_949x_hook.c').read_bytes()).hexdigest(),
            'exe_literals': {'lang': lang, 'catalog': str(catalog) if catalog else None,
                             'catalog_sha256': hashlib.sha256(catalog.read_bytes()).hexdigest() if catalog else None,
                             'strings': literal_report, 'operands_patched': len(patches)}}
    (out / ('patch%s.json' % tag)).write_text(json.dumps(meta, indent=2) + '\n')
    return meta


def main():
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument('--src', type=Path, required=True)
    ap.add_argument('--out', type=Path, required=True)
    ap.add_argument('--lang', choices=('vi', 'en'), help='thay chuỗi cứng của exe bằng bản dịch (949x)')
    ap.add_argument('--trace', action='store_true', help='bản chẩn đoán ghi l10n-trace.log')
    ap.add_argument('--catalog', type=Path, default=Path('/home/admin/boom/repos/boom-re-l10n/l10n/catalog_en.csv'))
    a = ap.parse_args()
    meta = build(a.src.resolve(), a.out.resolve(), a.lang, a.catalog.resolve() if a.lang else None, a.trace)
    print(json.dumps({k: v for k, v in meta.items() if k != 'exe_literals'}, indent=2))
    if a.lang:
        print('exe literals:', len(meta['exe_literals']['strings']), 'strings,', meta['exe_literals']['operands_patched'], 'operands')


if __name__ == '__main__':
    main()
