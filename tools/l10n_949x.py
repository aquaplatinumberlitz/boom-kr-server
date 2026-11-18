#!/usr/bin/env python3
"""Mã nội bộ 949x cho client 00967 đã vá: cp949 cộng bảng mở rộng tĩnh.

Ký tự có trong cp949 giữ nguyên byte cp949. Ký tự ngoài cp949 thuộc các dải trong
``RANGES`` nhận một mã hai byte cố định ở vùng cp949/UHC không dùng: lead 0xC7..0xFE,
trail 0x41..0x5A, 0x61..0x7A, 0x81..0xA0. Mã thứ ``k`` ứng với ký tự thứ ``k`` khi duyệt
``RANGES`` theo thứ tự; dải mới chỉ được nối vào cuối để mã cũ không đổi.

Lệnh:
  table --c OUT.h             sinh bảng C cho hook trong exe
  convert --src DIR --out DIR chuyển bộ dữ liệu UTF-8 (ssi1 + ClientString) sang 949x
  fonts --src KR --out DIR    vá bản sao font Gulim: mượn hình chữ ngoài 949 từ CxUni (phương án dự phòng)
  vnfonts --vn VN --kr KR --out DIR  bộ font VN 12x16 có dấu chồng, sửa advance chữ Hàn theo font KR, nâng hình CxUni/Bold 2 hàng
  selftest                    kiểm round-trip
"""
import argparse
import json
import re
import shutil
import struct
import sys
import zlib
from pathlib import Path

VERSION = 1
RANGES = [(0x00A0, 0x0250), (0x0300, 0x0370), (0x0370, 0x0400), (0x0400, 0x0500), (0x1E00, 0x1F00),
          (0x2000, 0x2070), (0x20A0, 0x20D0), (0x2100, 0x2150), (0x2150, 0x2190), (0x2190, 0x2200),
          (0x2200, 0x2300), (0x2460, 0x2500), (0x2500, 0x2580), (0x2580, 0x25A0), (0x25A0, 0x2600),
          (0x2600, 0x2700), (0x3000, 0x3040), (0xFF00, 0xFFF0)]
TRAILS = list(range(0x41, 0x5B)) + list(range(0x61, 0x7B)) + list(range(0x81, 0xA1))
LEAD0 = 0xC7


def _in_cp949(ch):
    try:
        ch.encode('cp949')
        return True
    except UnicodeEncodeError:
        return False


EXT = [chr(c) for a, b in RANGES for c in range(a, b) if not _in_cp949(chr(c))]
assert len(EXT) <= (0xFF - LEAD0) * len(TRAILS)
EXT_INDEX = {ch: i for i, ch in enumerate(EXT)}


def code(i):
    return bytes([LEAD0 + i // len(TRAILS), TRAILS[i % len(TRAILS)]])


TRAIL_POS = {t: i for i, t in enumerate(TRAILS)}


def index_of(lead, trail):
    if lead < LEAD0 or lead == 0xFF or trail not in TRAIL_POS:
        return None
    i = (lead - LEAD0) * len(TRAILS) + TRAIL_POS[trail]
    return i if i < len(EXT) else None


def encode(text, errors='strict'):
    out = bytearray()
    for ch in text:
        if ch in EXT_INDEX:
            out += code(EXT_INDEX[ch])
        else:
            try:
                out += ch.encode('cp949')
            except UnicodeEncodeError:
                if errors == 'strict':
                    raise
                out += b'?'
    return bytes(out)


def decode(data):
    out, i = [], 0
    while i < len(data):
        b = data[i]
        if b < 0x80:
            out.append(chr(b)); i += 1; continue
        if i + 1 < len(data):
            k = index_of(b, data[i + 1])
            if k is not None:
                out.append(EXT[k]); i += 2; continue
            out.append(data[i:i + 2].decode('cp949')); i += 2; continue
        raise ValueError('byte lẻ cuối chuỗi')
    return ''.join(out)


# ---- dữ liệu: ssi1 và ClientString ----

def lstr(raw, pos):
    n = struct.unpack_from('<I', raw, pos)[0]
    return raw[pos + 4:pos + 4 + n], pos + 4 + n


def packed(data):
    return struct.pack('<I', len(data)) + data


def u8(b, stats):
    """UTF-8 nghiêm ngặt sang 949x; chuỗi không phải UTF-8 hợp lệ thì lỗi."""
    text = b.decode('utf-8')
    try:
        return encode(text)
    except UnicodeEncodeError as exc:
        bad = stats.setdefault('unmapped', {})
        bad[text[exc.start]] = bad.get(text[exc.start], 0) + 1
        return encode(text, 'replace')


def conv_pair(raw, stats):
    n = struct.unpack_from('<H', raw)[0]
    out, p = bytearray(raw[:2]), 2
    for _ in range(n):
        k, p = lstr(raw, p); v, p = lstr(raw, p)
        out += packed(u8(k, stats)) + packed(u8(v, stats)); stats['strings'] = stats.get('strings', 0) + 2
    if p != len(raw):
        raise ValueError('pair: dư dữ liệu')
    return bytes(out)


def conv_maps(raw, stats):
    n = struct.unpack_from('<H', raw)[0]
    out, p = bytearray(raw[:2]), 2
    for _ in range(n):
        out += raw[p:p + 5]; p += 5
        name, p = lstr(raw, p); out += packed(name)
        disp, p = lstr(raw, p); out += packed(u8(disp, stats)); stats['strings'] = stats.get('strings', 0) + 1
        out += raw[p:p + 2]; p += 2
        for _ in range(2):
            s, p = lstr(raw, p); out += packed(s)
        out += raw[p:p + 16]; p += 16
    if p != len(raw):
        raise ValueError('maps: dư dữ liệu')
    return bytes(out)


def conv_fxmg(raw, stats):
    """csysfxmg.ssd (dòng chạy chữ, mẹo chat): u16 count, count × (u8 kind, lstr)."""
    n = struct.unpack_from('<H', raw)[0]
    out, p = bytearray(raw[:2]), 2
    for _ in range(n):
        out += raw[p:p + 1]; p += 1
        v, p = lstr(raw, p)
        out += packed(u8(v, stats)); stats['strings'] = stats.get('strings', 0) + 1
    if p != len(raw):
        raise ValueError('fxmg: dư dữ liệu')
    return bytes(out)


def conv_stock(raw, stats):
    n = struct.unpack_from('<I', raw)[0]
    out, p = bytearray(raw[:4]), 4
    for _ in range(n):
        ln = struct.unpack_from('<I', raw, p + 5)[0]
        name = raw[p + 9:p + 9 + ln]; q = p + 9 + ln
        icon = struct.unpack_from('<I', raw, q + 12)[0]
        end = p + 129 + ln + icon
        out += raw[p:p + 5] + packed(u8(name, stats)) + raw[q:end]; stats['strings'] = stats.get('strings', 0) + 1
        p = end
    if p != len(raw):
        raise ValueError('stock: dư dữ liệu')
    return bytes(out)


def ssi_read(data):
    if data[:4] != b'ssi1' or data[8] != 0x18 or data[13] != 1:
        raise ValueError('header ssi1 không hỗ trợ')
    raw = zlib.decompress(data[18:])
    if len(raw) != struct.unpack_from('<I', data, 14)[0]:
        raise ValueError('độ dài giải nén sai')
    return raw


def ssi_write(original, raw):
    comp = zlib.compress(raw, 9)
    header = bytearray(original[:18])
    header[9:13] = struct.pack('<I', len(comp) + 5)
    header[14:18] = struct.pack('<I', len(raw))
    body = bytes(header[8:]) + comp
    return b'ssi1' + struct.pack('<I', zlib.adler32(body, 0) & 0xffffffff) + body


def _idf():
    sys.path.insert(0, '/home/admin/boom/repos/boom-re-l10n/tools')
    import idf  # chỉ đọc/giải mã index; không sửa repo đó
    return idf


def encrypt_idx(plain, key):
    count, tail = divmod(len(plain), 4)
    words = struct.unpack_from('<%dI' % count, plain)
    cipher = [words[0] ^ key] + [words[i] ^ words[i - 1] for i in range(1, count)] if count else []
    out = bytearray(struct.pack('<%dI' % count, *cipher))
    last = words[-1] if words else key
    for i in range(tail):
        out.append(plain[4 * count + i] ^ ((last >> (8 * i)) & 255))
    return bytes(out)


def conv_pack(src, out, stats):
    """Đổi entry ClientData/ClientString (XML UTF-8, entry không mã hoá) sang 949x; ghi nối vào .idd."""
    idf = _idf()
    data = (src / 'FxDesktop.idx').read_bytes(); key = int.from_bytes(data[:4], 'little')
    plain = bytearray(idf.decrypt_idx(data, key)); version, entries, end = idf.parse_idx(plain)
    assert version is not None and end == len(plain)
    p, pos = 12, None
    for name, offset, size, ekey, crc in entries:
        n = struct.unpack_from('<i', plain, p)[0]; p += 4 + n
        if name == 'ClientData/ClientString':
            assert ekey == 0 and pos is None
            pos, off, sz = p, offset, size
        p += 16
    blob = (src / 'FxDesktop.idd').read_bytes()[off:off + sz]
    assert int.from_bytes(blob[:4], 'little') == len(blob) - 4
    text = blob[4:].decode('utf-8')
    stats['chars'] = len(text)
    new = packed(encode(text, 'replace'))
    stats['unmapped'] = sorted({ch for ch in text if ch not in EXT_INDEX and not _in_cp949(ch)})
    idd = bytearray((src / 'FxDesktop.idd').read_bytes())
    struct.pack_into('<II', plain, pos, len(idd), len(new))
    idd += new
    (out / 'FxDesktop.idd').write_bytes(idd)
    (out / 'FxDesktop.idx').write_bytes(encrypt_idx(plain, key))


SSI = {'csysstr.ssd': conv_pair, 'csysqstr.ssd': conv_pair, 'fxdynmapinfo.ssd': conv_maps,
       **{f'fxstock{i}.ini': conv_stock for i in range(4)}}
# Chỉ đổi nếu bộ nguồn có file (bản dựng cũ trước khi l10n_build ghi csysfxmg.ssd không có).
OPTIONAL_SSI = {'csysfxmg.ssd': conv_fxmg}


def convert(src, out):
    out.mkdir(parents=True, exist_ok=True)
    report = {'format': '949x', 'version': VERSION, 'ext_chars': len(EXT), 'files': {}}
    for name, fn in SSI.items():
        stats = {}
        original = (src / name).read_bytes()
        raw = fn(ssi_read(original), stats)
        (out / name).write_bytes(ssi_write(original, raw))
        assert ssi_read((out / name).read_bytes()) == raw
        report['files'][name] = stats
    for name, fn in OPTIONAL_SSI.items():
        if not (src / name).exists():
            continue
        stats = {}
        original = (src / name).read_bytes()
        raw = fn(ssi_read(original), stats)
        (out / name).write_bytes(ssi_write(original, raw))
        assert ssi_read((out / name).read_bytes()) == raw
        report['files'][name] = stats
    stats = {}
    conv_pack(src, out, stats)
    report['files']['FxDesktop'] = stats
    # Pack khác do bản dựng ảnh thay: chỉ có ảnh, chép nguyên.
    for idx in sorted(src.glob('*.idx')):
        if idx.stem == 'FxDesktop':
            continue
        for path in (idx, idx.with_suffix('.idd')):
            shutil.copyfile(path, out / path.name)
        report['files'][idx.stem] = {'copied': 'image pack, unchanged'}
    # Video mở đầu thay theo ngôn ngữ (nm.bik của client cùng ngôn ngữ): chép nguyên.
    if (src / 'nm.bik').exists():
        shutil.copyfile(src / 'nm.bik', out / 'nm.bik')
        report['files']['nm.bik'] = {'copied': 'intro video, unchanged'}
    (out / 'convert-report.json').write_text(json.dumps(report, ensure_ascii=False, indent=2) + '\n')
    return report


def _fft_blocks(data):
    n, w, h = struct.unpack_from('<III', data, 8)
    bpg = struct.unpack_from('<I', data, 28)[0]
    obpg = ((w + 2) * (h + 2) + 7) // 8
    if 32 + n * (1 + bpg) + n * (1 + obpg) != len(data):
        raise ValueError('kích thước fft không khớp định dạng đã đọc')
    return n, [(32, 1 + bpg), (32 + n * (1 + bpg), 1 + obpg)]


def merge_font(target, donor, chars):
    """Chép hình chữ của donor vào target ở các mã mà target chỉ có hình hộp thay thế."""
    t, d = bytearray(target), bytes(donor)
    n, blocks = _fft_blocks(t)
    if _fft_blocks(d) != (n, blocks):
        raise ValueError('hai font khác định dạng')
    base, size = blocks[0]
    glyph = lambda buf, c: bytes(buf[base + c * size + 1:base + (c + 1) * size])
    counts = {}
    for c in chars:
        counts[glyph(t, c)] = counts.get(glyph(t, c), 0) + 1
    box = max(counts, key=counts.get)
    copied = []
    for c in chars:
        if c < n and glyph(t, c) == box and glyph(d, c) != box and any(glyph(d, c)):
            for b, sz in blocks:
                t[b + c * sz:b + (c + 1) * sz] = d[b + c * sz:b + (c + 1) * sz]
            copied.append(c)
    return bytes(t), copied


FONT_PAIRS = [('CxUniGulim.fft', 'CxUni.fft'), ('CxUniGulimBold.fft', 'CxUniBold.fft')]


def fonts(src, out):
    """Font Gulim (ô nhập liệu) có hình hộp cho chữ Việt; mượn hình từ CxUni cùng kiểu."""
    out.mkdir(parents=True, exist_ok=True)
    chars = [ord(c) for c in EXT]
    report = {}
    for target, donor in FONT_PAIRS:
        data, copied = merge_font((src / target).read_bytes(), (src / donor).read_bytes(), chars)
        (out / target).write_bytes(data)
        report[target] = {'donor': donor, 'copied': len(copied),
                          'sample': ''.join(chr(c) for c in copied[:40])}
    return report


CJK_RANGES = [(0x1100, 0x1200), (0x3000, 0x3100), (0x3130, 0x3190), (0x3200, 0x3400), (0x4E00, 0xA000),
              (0xAC00, 0xD7A4), (0xF900, 0xFB00), (0xFF00, 0xFFF0)]


def vn_fonts(vn, kr, out):
    """Bộ font VN (12x16, đủ dấu chồng) cho client KR. CxUni/CxUniBold: giữ hình VN, ô Hàn/CJK có advance hẹp hơn
    font KR lấy advance KR. CxUniGulim/CxUniGulimBold (ô nhập, ô người nhận): Gulim 12x12 gọn (compact_gulim)."""
    out.mkdir(parents=True, exist_ok=True)
    report = {}
    for name in ('CxUni.fft', 'CxUniBold.fft', 'CxUniGulim.fft', 'CxUniGulimBold.fft'):
        v, k = bytearray((vn / name).read_bytes()), (kr / name).read_bytes()
        nv, bv = _fft_blocks(v)
        nk, bk = _fft_blocks(k)
        fixed = 0
        for a, b in CJK_RANGES:
            for c in range(a, min(b, nv, nk)):
                for (ov, sv), (ok, sk) in zip(bv, bk):
                    av, ak = v[ov + c * sv], k[ok + c * sk]
                    if ak > av and any(k[ok + c * sk + 1:ok + (c + 1) * sk]):
                        v[ov + c * sv] = ak
                        fixed += 1
        report[name] = {'advance_fixed': fixed}
        if 'Gulim' not in name:
            v, info = raise_glyphs(v, RAISE_ROWS)
            report[name].update(info)
        if 'Gulim' in name:
            # Ô nhập của layout KR chỉ cao 12 điểm: dùng Gulim 12x12 gọn thay cho Gulim VN 15/16 hàng.
            base, _ = merge_font(k, (kr / name.replace('Gulim', '')).read_bytes(), [ord(ch) for ch in EXT])
            v, info = compact_gulim(v, base)
            report[name].update(info)
        (out / name).write_bytes(bytes(v))
    return report


LATIN_RANGES = [(0x20, 0x250), (0x300, 0x370), (0x1E00, 0x1F00), (0x20AB, 0x20AC)]

# Exe vẽ chữ với đỉnh ô chữ ở một y cố định theo layout KR (font 12x12, đường chân chữ Latin ở hàng 9).
# CxUni/CxUniBold VN cao 16 hàng, chừa 4 hàng trên cho dấu chồng nên chân chữ Latin ở hàng 12: chữ Việt
# thấp hơn chữ KR 3 điểm, sát/chạm viền dưới ở ô thấp (tab chat, khung map). Nâng mọi hình chữ lên 2 hàng.
RAISE_ROWS = 2


def raise_glyphs(data, k):
    """Nâng hình chữ của font 12x16 lên k hàng (đỉnh ô vẫn ở y cũ nên không cắt thêm ở mép trên).
    Hình có hàng trống phía trên >= k: dời nguyên. Chữ Latin có dấu (tách được theo NFD) mà dấu chạm hàng < k:
    thân chữ (từ hàng đỉnh của chữ gốc) dời k hàng, các hàng dấu bỏ hàng trống rồi gộp đôi cho vừa chỗ phía trên thân.
    Hình khác (CJK, ...) dời tối đa bằng số hàng trống phía trên. Viền (khối thứ hai) sinh lại bằng giãn 8 hướng."""
    import unicodedata
    b = bytearray(data)
    n, ((og, sg), (oo, so)) = _fft_blocks(b)
    w, h = struct.unpack_from('<II', b, 12)
    assert (w, h) == (12, 16) and (sg - 1) * 8 == w * h
    mask = (1 << w) - 1

    src = bytes(b)  # đọc hình gốc: chữ gốc (A, a) có thể đã được dời trước chữ có dấu

    def rows(c):
        bits = int.from_bytes(src[og + c * sg + 1:og + (c + 1) * sg], 'big')
        return [(bits >> ((h - 1 - r) * w)) & mask for r in range(h)]

    def top(r):
        return next((i for i, x in enumerate(r) if x), None)

    stats = {'shifted': 0, 'partial': 0, 'marks_packed': 0, 'marks_merged': []}
    for c in range(n):
        r = rows(c)
        t = top(r)
        if t is None:
            continue
        if t >= k:
            nr = r[k:] + [0] * k
            stats['shifted'] += 1
        else:
            nfd = unicodedata.normalize('NFD', chr(c)) if not 0xD800 <= c <= 0xDFFF else chr(c)
            accented = len(nfd) > 1 and all(unicodedata.category(m) == 'Mn' for m in nfd[1:])
            bt = top(rows(ord(nfd[0]))) if accented and ord(nfd[0]) < n else None
            if bt is None or bt <= k:
                s = t
                if s == 0:
                    continue
                nr = r[s:] + [0] * s
                stats['partial'] += 1
            else:
                marks = [x for x in r[:bt] if x]
                slots = bt - k
                merged = len(marks) > slots
                while len(marks) > slots:
                    # gộp cặp hàng kề nhau có ít điểm nhất
                    i = min(range(len(marks) - 1), key=lambda j: bin(marks[j] | marks[j + 1]).count('1'))
                    marks[i:i + 2] = [marks[i] | marks[i + 1]]
                pad = slots - len(marks)
                gap = 1 if pad >= 1 else 0
                head = [0] * (pad - gap) + marks + [0] * gap
                nr = head + r[bt:] + [0] * k
                stats['marks_packed'] += 1
                if merged:
                    stats['marks_merged'].append(chr(c))
        assert len(nr) == h
        bits = 0
        for x in nr:
            bits = (bits << w) | x
        b[og + c * sg + 1:og + (c + 1) * sg] = bits.to_bytes(sg - 1, 'big')
        ow, oh = w + 2, h + 2
        dil = [0] * oh
        for i, x in enumerate(nr):
            if x:
                sp = ((x << 2) | (x << 1) | x)
                for d in (0, 1, 2):
                    dil[i + d] |= sp
        out = 0
        for i in range(oh):
            g = (nr[i - 1] << 1) if 1 <= i <= h else 0
            out = (out << ow) | (dil[i] & ~g & ((1 << ow) - 1))
        b[oo + c * so + 1:oo + (c + 1) * so] = (out << ((so - 1) * 8 - ow * oh)).to_bytes(so - 1, 'big')
    stats['marks_merged'] = ''.join(stats['marks_merged'])
    stats['raise_rows'] = k
    return bytes(b), stats


def _grid(b, w, h):
    bits = int.from_bytes(b, 'big')
    total = len(b) * 8
    return [[(bits >> (total - 1 - (r * w + k))) & 1 for k in range(w)] for r in range(h)]


def _pack(g, w, h, size):
    v = 0
    for r in range(h):
        for k in range(w):
            v = (v << 1) | g[r][k]
    return (v << (size * 8 - w * h)).to_bytes(size, 'big')


def compact_gulim(vn, base):
    """Font Gulim 12x12 cho ô nhập của layout KR (ô cao 12 điểm): chữ Hàn/CJK lấy từ base (Gulim KR đã vá);
    chữ Latin/Việt lấy từ Gulim VN (12x15/12x16), giữ 10 hàng thân+chân chữ và gộp vùng dấu phía trên vào 2 hàng,
    nên dấu chồng (Ệ, Ố) vẫn thấy và dấu nặng, đuôi g không bị cắt. Viền (khối thứ hai) sinh lại bằng giãn 8 hướng."""
    v, b = bytes(vn), bytearray(base)
    nv, bv = _fft_blocks(v)
    nb, bb = _fft_blocks(b)
    wv, hv = struct.unpack_from('<II', v, 12)
    wb, hb = struct.unpack_from('<II', b, 12)
    assert (wb, hb) == (12, 12) and wv == 12
    (ov, sv), _ = bv
    (ob, sb), (oo, so) = bb
    cps = [c for a, z in LATIN_RANGES for c in range(a, z) if c < nv]
    # Hàng đáy theo chữ thường dùng (ASCII, Latin-1, chữ Việt); ký tự hiếm thấp hơn thì mất phần chân.
    common = [ord(ch) for ch in 'abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789'] + \
        list(range(0x1EA0, 0x1EFA)) + [0x110, 0x111, 0x1A0, 0x1A1, 0x1AF, 0x1B0]
    bottom = max(r for c in common for r, row in enumerate(_grid(v[ov + c * sv + 1:ov + (c + 1) * sv], wv, hv)) if any(row))
    top_rows = list(range(0, bottom - 9))
    half = (len(top_rows) + 1) // 2
    mapping = [top_rows[:half], top_rows[half:]] + [[r] for r in range(bottom - 9, bottom)] + [list(range(bottom, hv))]
    copied = 0
    for c in cps:
        g = _grid(v[ov + c * sv + 1:ov + (c + 1) * sv], wv, hv)
        if c != 0x20 and not any(any(row) for row in g):
            continue
        ng = [[max([g[r][k] for r in rows] or [0]) for k in range(12)] for rows in mapping]
        adv = v[ov + c * sv]
        b[ob + c * sb] = adv
        b[ob + c * sb + 1:ob + (c + 1) * sb] = _pack(ng, 12, 12, sb - 1)
        dil = [[0] * 14 for _ in range(14)]
        for r in range(12):
            for k in range(12):
                if ng[r][k]:
                    for dr in (-1, 0, 1):
                        for dk in (-1, 0, 1):
                            dil[r + 1 + dr][k + 1 + dk] = 1
        for r in range(12):
            for k in range(12):
                if ng[r][k]:
                    dil[r + 1][k + 1] = 0
        b[oo + c * so] = adv + 2
        b[oo + c * so + 1:oo + (c + 1) * so] = _pack(dil, 14, 14, so - 1)
        copied += 1
    return bytes(b), {'latin_glyphs': copied, 'source_bottom_row': bottom, 'mark_rows_merged': [top_rows[:half], top_rows[half:]]}


def c_table(path):
    def rows(values):
        return ',\n'.join('  ' + ','.join('0x%04x' % v for v in values[i:i + 12]) for i in range(0, len(values), 12))
    order = sorted(range(len(EXT)), key=lambda i: EXT[i])
    path.write_text('/* Sinh bởi tools/l10n_949x.py; 949x v%d. */\n#define EXT_COUNT %d\n'
                    'static const unsigned short ext_uni[EXT_COUNT] = {\n%s\n};\n'
                    'static const unsigned short ext_sorted[EXT_COUNT] = {\n%s\n};\n'
                    'static const unsigned short ext_sorted_idx[EXT_COUNT] = {\n%s\n};\n'
                    % (VERSION, len(EXT), rows([ord(c) for c in EXT]), rows([ord(EXT[i]) for i in order]), rows(order)))


def selftest():
    sample = 'Phòng chơi: Trần Đức Hùng ₫ — 방 제목 크레이지 아케이드 ABC ếộữ'
    b = encode(sample)
    assert decode(b) == sample, (decode(b), sample)
    for i in range(len(EXT)):
        c = code(i)
        try:
            c.decode('cp949')
            raise AssertionError('mã trùng cp949 %s' % c.hex())
        except UnicodeDecodeError:
            pass
        assert index_of(*c) == i
    kinds = [(1, 'Hãy cẩn thận khi nhận tệp'), (6, '넥슨 캐시 ₫ ếộữ')]
    raw = struct.pack('<H', len(kinds)) + b''.join(bytes([k]) + packed(t.encode('utf-8')) for k, t in kinds)
    head = b'ssi1' + bytes(4) + b'\x18' + bytes(4) + b'\x01' + bytes(4)
    out = ssi_read(ssi_write(head, conv_fxmg(raw, {})))
    q, rows = 2, []
    for _ in kinds:
        kind = out[q]; text, q = lstr(out, q + 1); rows.append((kind, decode(text)))
    assert q == len(out) and rows == kinds, rows
    print('selftest ok, %d ký tự mở rộng, mã cuối %s' % (len(EXT), code(len(EXT) - 1).hex()))


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    sub = ap.add_subparsers(dest='cmd', required=True)
    t = sub.add_parser('table'); t.add_argument('--c', type=Path, required=True)
    c = sub.add_parser('convert'); c.add_argument('--src', type=Path, required=True); c.add_argument('--out', type=Path, required=True)
    fo = sub.add_parser('fonts'); fo.add_argument('--src', type=Path, required=True); fo.add_argument('--out', type=Path, required=True)
    vf = sub.add_parser('vnfonts'); vf.add_argument('--vn', type=Path, required=True); vf.add_argument('--kr', type=Path, required=True); vf.add_argument('--out', type=Path, required=True)
    sub.add_parser('selftest')
    a = ap.parse_args()
    if a.cmd == 'table':
        c_table(a.c)
    elif a.cmd == 'fonts':
        print(json.dumps(fonts(a.src, a.out), ensure_ascii=False, indent=2))
    elif a.cmd == 'vnfonts':
        print(json.dumps(vn_fonts(a.vn, a.kr, a.out), ensure_ascii=False, indent=2))
    elif a.cmd == 'convert':
        print(json.dumps(convert(a.src, a.out), ensure_ascii=False, indent=2))
    else:
        selftest()


if __name__ == '__main__':
    main()
