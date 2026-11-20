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

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import l10n_949x as X  # noqa: E402

SOURCE_SHA256 = '4b781fbcea21e0f6294c729b63be86ab2953d60ad83b0d0c9219359f826107ed'  # CA-r2-stage-social-cmp.exe
WANT = {'MultiByteToWideChar': 'IAT_MB2WC', 'WideCharToMultiByte': 'IAT_WC2MB', 'PeekMessageA': 'IAT_PEEKMESSAGEA',
        'GetProcAddress': 'IAT_GETPROCADDRESS', 'GetModuleHandleA': 'IAT_GETMODULEHANDLEA',
        'ImmGetCompositionStringA': 'IAT_IMMGETCOMPOSITIONSTRINGA', 'ImmGetCompositionStringW': 'IAT_IMMGETCOMPOSITIONSTRINGW',
        'RegisterClassExA': 'IAT_REGISTERCLASSEXA', 'TranslateMessage': 'IAT_TRANSLATEMESSAGE'}


# Vá layout cho bộ font VN (CxUni/CxUniBold 12x16 nâng 2 hàng, Gulim 12x12 gọn) của bản --lang vi. Mỗi mục:
# (VA, byte cũ, byte mới, lý do); byte cũ được kiểm trước khi ghi. Đọc tĩnh trên bản sao IDA (kr-full.exe = PE nguồn).
VI_LAYOUT = [
    # Tab lọc chat sảnh/phòng (sub_D357E0): y chữ = đỉnh tab + 7 ([ebp-154h]). Chữ Việt có đuôi (g, y, dấu nặng)
    # xuống tới hàng viền dưới; +5 đặt mực chữ ở 2..15 trong lòng tab 17 hàng, không chạm viền trên lẫn dưới.
    (0xD35AD5, '83c007', '83c005', 'ChatFilterNotice y +7 -> +5'),
    (0xD35C06, '83c207', '83c205', 'ChatFilterWhole y +7 -> +5'),
    (0xD35CE8, '83c207', '83c205', 'ChatFilterGuild y +7 -> +5'),
    # Dòng chạy chữ (CChatStage, vẽ ở sub_D39490): DrawText kiểu 2 (chữ có viền), họ font 1 (CxUniGulim) ở y=1 trong
    # khung clip cao 12. Gulim 12x12 gọn gộp dấu chồng vào 2 hàng và khung 12 cắt hàng đuôi chữ. Đổi sang họ font 0
    # (CxUni VN 12x16, đủ dấu chồng) và khung clip cao 16 (14 hàng mực + viền 1 điểm mỗi phía).
    # Độ rộng ước tính len*6 của exe (đặt lại vị trí cuộn) vẫn lớn hơn bề rộng CxUni của mọi dòng loại 2.
    (0xD395B6, '6a01', '6a00', 'marquee font family 1 (CxUniGulim) -> 0 (CxUni)'),
    *[(va, '6a0c', '6a10', 'marquee clip h 12 -> 16') for va in (
        0x6E41C7, 0x71C678, 0xD55E1D, 0xDA70E3, 0xDC34D9, 0xE58722, 0xE58F11, 0xE5932C,
        0x106EC22, 0x10E0287, 0x111ACE3, 0x1129E20, 0x11545B3)],
]
# Tab chat phòng (sub_D364D0): y chữ = đỉnh khung (call sub_149BEF0, không cộng hằng nào để sửa tại chỗ), dấu nặng của
# `Hội` chạm viền dưới tab. Lời gọi được chuyển sang stub trong .l10n: gọi hàm cũ rồi trừ 2.
VI_LAYOUT_CALLS = [(0xD365C8, 0x149BEF0, 2, 'PreparePreTab ChatFilterWhole y -2'),
                   (0xD36898, 0x149BEF0, 2, 'PreparePreTab ChatFilterGuild y -2')]


# Hai đường MessageBoxA của cùng hộp lỗi; các caller caption dùng chung giữ nguyên.
VI_LOGIN_DIALOG_SITES = (0x10FA74E, 0x13AE104)
VI_LOGIN_DIALOG_PREIMAGE = bytes.fromhex('68748f7801 68848f7801 6a00 ff154ca66e01')


def vi_login_dialog(base_va):
    caption = 'Lỗi nghiêm trọng'.encode('utf-16le') + b'\0\0'
    text = 'Đăng nhập không hợp lệ!'.encode('utf-16le') + b'\0\0'
    replacement = (struct.pack('<BIBI', 0x68, base_va, 0x68, base_va + len(caption))
                   + bytes.fromhex('6a00 ff1578a66e01'))
    return caption + text, [(site, VI_LOGIN_DIALOG_PREIMAGE, replacement) for site in VI_LOGIN_DIALOG_SITES]


def apply_checked(image, offset_for_va, patches):
    for va, old, new in patches:
        off = offset_for_va(va)
        if len(old) != len(new) or image[off:off + len(old)] != old:
            raise SystemExit('byte vá tại %#x không phải %s' % (va, old.hex()))
        image[off:off + len(new)] = new


def call_stub(stub_va, target, sub):
    """call target; sub eax, imm8; ret (thiscall không đối số, giữ ecx)."""
    return b'\xe8' + struct.pack('<i', target - (stub_va + 5)) + bytes((0x83, 0xE8, sub, 0xC3))


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


def build(src, out, lang=None, catalog=None, trace=False, vi_layout=True):
    import pefile

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
        # Vtable CCtrlEditORG / CCtrlTempChatEditORG (RTTI, IDA trên bản sao), chỉ bản --trace dùng.
        f.write('#define VT_EDITORG 0x17b3b20\n#define VT_TEMPCHATEDITORG 0x17b416c\n'
                '#define VT_EDITIME 0x17b3ee4\n#define VT_TEMPCHATEDITIME 0x17b4294\n#define VT_NAVI 0x170f524\n')
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
    calls = []
    if lang == 'vi' and vi_layout:
        code += b'\0' * (-len(code) % 16)
        for site, target, sub, why in VI_LAYOUT_CALLS:
            stub = va + len(code)
            code += call_stub(stub, target, sub)
            code += b'\xcc' * (-len(code) % 4)
            calls.append((site, target, stub, why))
    login_dialog = []
    if lang == 'vi':
        message_box_slots = {(lib.dll.lower(), imp.name, imp.address)
                             for lib in pe.DIRECTORY_ENTRY_IMPORT for imp in lib.imports}
        if not {(b'user32.dll', b'MessageBoxA', 0x16EA64C),
                (b'user32.dll', b'MessageBoxW', 0x16EA678)} <= message_box_slots:
            raise SystemExit('IAT MessageBox không khớp bản KR đã đo')
        code += b'\0' * (-len(code) % 4)
        dialog, login_dialog = vi_login_dialog(va + len(code))
        code += dialog
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
    apply_checked(image, lambda addr: pe.get_offset_from_rva(addr - base), login_dialog)
    layout = VI_LAYOUT if lang == 'vi' and vi_layout else []
    for va, old, new, _ in layout:
        off = pe.get_offset_from_rva(va - base)
        if image[off:off + len(old) // 2].hex() != old:
            raise SystemExit('byte layout tại %#x không phải %s' % (va, old))
        image[off:off + len(new) // 2] = bytes.fromhex(new)
    for site, target, stub, _ in calls:
        off = pe.get_offset_from_rva(site - base)
        if image[off] != 0xE8 or struct.unpack_from('<i', image, off + 1)[0] != target - (site + 5):
            raise SystemExit('lời gọi tại %#x không tới %#x' % (site, target))
        struct.pack_into('<i', image, off + 1, stub - (site + 5))
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
                             'strings': literal_report, 'operands_patched': len(patches)},
            'vi_login_dialog': [{'va': hex(site), 'old': old.hex(), 'new': new.hex()}
                                for site, old, new in login_dialog],
            'layout': [{'va': hex(va), 'old': old, 'new': new, 'why': why} for va, old, new, why in layout] +
                      [{'va': hex(site), 'call': hex(target), 'stub': hex(stub), 'why': why}
                       for site, target, stub, why in calls]}
    (out / ('patch%s.json' % tag)).write_text(json.dumps(meta, indent=2) + '\n')
    return meta


def main():
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument('--src', type=Path, required=True)
    ap.add_argument('--out', type=Path, required=True)
    ap.add_argument('--lang', choices=('vi', 'en'), help='thay chuỗi cứng của exe bằng bản dịch (949x)')
    ap.add_argument('--trace', action='store_true', help='bản chẩn đoán ghi l10n-trace.log')
    ap.add_argument('--catalog', type=Path, default=Path('/home/admin/boom/repos/boom-re-l10n/l10n/catalog_en.csv'))
    ap.add_argument('--no-vi-layout', action='store_true',
                    help='bản vi không vá layout cho font VN (VI_LAYOUT, VI_LAYOUT_CALLS)')
    a = ap.parse_args()
    meta = build(a.src.resolve(), a.out.resolve(), a.lang, a.catalog.resolve() if a.lang else None, a.trace,
                 not a.no_vi_layout)
    print(json.dumps({k: v for k, v in meta.items() if k not in ('exe_literals', 'layout')}, indent=2))
    if meta['layout']:
        print('layout patches:', len(meta['layout']))
    if a.lang:
        print('exe literals:', len(meta['exe_literals']['strings']), 'strings,', meta['exe_literals']['operands_patched'], 'operands')


if __name__ == '__main__':
    main()
