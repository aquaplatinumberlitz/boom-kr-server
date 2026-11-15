/* Hook 949x cho bản sao client 00967 (CA-r2-*), nạp ở section .l10n.
 *
 * Mọi chuyển mã trong exe đi qua hai slot IAT MultiByteToWideChar/WideCharToMultiByte.
 * Với codepage 949, hook giải/ghi thêm các mã hai byte của bảng mở rộng tĩnh 949x
 * (tools/l10n_949x.py); mọi đoạn khác chuyển nguyên cho API gốc. Codepage khác đi thẳng API gốc.
 * Thêm hook PeekMessageA và ImmGetCompositionStringA/W cho nhập liệu (xem dưới).
 * Biên dịch: tools/l10n_949x_patch.py (clang i386 freestanding, nhị phân phẳng).
 */
#include "l10n_949x_table.h"

typedef unsigned char u8;
typedef unsigned short u16;
typedef int(__attribute__((stdcall)) * mb2wc_t)(unsigned, unsigned, const char *, int, u16 *, int);
typedef int(__attribute__((stdcall)) * wc2mb_t)(unsigned, unsigned, const u16 *, int, char *, int, const char *, int *);

#define SLOT_MB2WC ((void **)IAT_MB2WC)
#define SLOT_WC2MB ((void **)IAT_WC2MB)

mb2wc_t orig_mb2wc;
wc2mb_t orig_wc2mb;
unsigned l10n_stats[4]; /* số lần decode/encode có ký tự mở rộng; đọc được bằng debugger */

static int ext_index(u8 lead, u8 trail) {
    int t;
    if (lead < 0xC7 || lead == 0xFF) return -1;
    if (trail >= 0x41 && trail <= 0x5A) t = trail - 0x41;
    else if (trail >= 0x61 && trail <= 0x7A) t = 26 + trail - 0x61;
    else if (trail >= 0x81 && trail <= 0xA0) t = 52 + trail - 0x81;
    else return -1;
    t += (lead - 0xC7) * 84;
    return t < EXT_COUNT ? t : -1;
}

static int ext_lookup(u16 c) {
    int lo = 0, hi = EXT_COUNT - 1;
    if (c < 0xA0) return -1;
    while (lo <= hi) {
        int mid = (lo + hi) >> 1;
        if (ext_sorted[mid] == c) return ext_sorted_idx[mid];
        if (ext_sorted[mid] < c) lo = mid + 1; else hi = mid - 1;
    }
    return -1;
}

static const u8 trails[84] = {
    0x41,0x42,0x43,0x44,0x45,0x46,0x47,0x48,0x49,0x4A,0x4B,0x4C,0x4D,0x4E,0x4F,0x50,0x51,0x52,0x53,0x54,0x55,0x56,0x57,0x58,0x59,0x5A,
    0x61,0x62,0x63,0x64,0x65,0x66,0x67,0x68,0x69,0x6A,0x6B,0x6C,0x6D,0x6E,0x6F,0x70,0x71,0x72,0x73,0x74,0x75,0x76,0x77,0x78,0x79,0x7A,
    0x81,0x82,0x83,0x84,0x85,0x86,0x87,0x88,0x89,0x8A,0x8B,0x8C,0x8D,0x8E,0x8F,0x90,0x91,0x92,0x93,0x94,0x95,0x96,0x97,0x98,0x99,0x9A,
    0x9B,0x9C,0x9D,0x9E,0x9F,0xA0};

__attribute__((stdcall)) int hook_mb2wc(unsigned cp, unsigned flags, const char *src, int n, u16 *dst, int dn) {
    const u8 *s = (const u8 *)src;
    int len, i, start, out, found = 0;
    if (cp != 949 || !s || n == 0 || dn < 0) return orig_mb2wc(cp, flags, src, n, dst, dn);
    if (n < 0) { for (len = 0; s[len]; len++) {} len++; } else len = n;
    for (i = 0; i < len;) {
        if (s[i] < 0x80) { i++; continue; }
        if (i + 1 < len && ext_index(s[i], s[i + 1]) >= 0) { found = 1; break; }
        i += 2;
    }
    if (!found) return orig_mb2wc(cp, flags, src, n, dst, dn);
    l10n_stats[0]++;
    out = 0; start = 0; i = 0;
    for (;;) {
        int k = -1;
        if (i < len) {
            if (s[i] < 0x80) { i++; continue; }
            if (i + 1 < len) k = ext_index(s[i], s[i + 1]);
            if (k < 0) { i += 2; if (i > len) i = len; continue; }
        }
        if (i > start) {
            int r;
            if (dn && out >= dn) return 0;
            r = orig_mb2wc(cp, flags, src + start, i - start, dn ? dst + out : 0, dn ? dn - out : 0);
            if (r <= 0) return 0;
            out += r;
        }
        if (i >= len) break;
        if (dn) { if (out >= dn) return 0; dst[out] = ext_uni[k]; }
        out++; i += 2; start = i;
    }
    return out;
}

__attribute__((stdcall)) int hook_wc2mb(unsigned cp, unsigned flags, const u16 *src, int n, char *dst, int dn,
                                        const char *def, int *used) {
    int len, i, start, out, any_used = 0, found = 0;
    if (cp != 949 || !src || n == 0 || dn < 0) return orig_wc2mb(cp, flags, src, n, dst, dn, def, used);
    if (n < 0) { for (len = 0; src[len]; len++) {} len++; } else len = n;
    for (i = 0; i < len; i++) if (src[i] >= 0xA0 && ext_lookup(src[i]) >= 0) { found = 1; break; }
    if (!found) return orig_wc2mb(cp, flags, src, n, dst, dn, def, used);
    l10n_stats[1]++;
    out = 0; start = 0; i = 0;
    for (;;) {
        int k = -1;
        if (i < len) { k = src[i] >= 0xA0 ? ext_lookup(src[i]) : -1; if (k < 0) { i++; continue; } }
        if (i > start) {
            int r, u = 0;
            if (dn && out >= dn) return 0;
            r = orig_wc2mb(cp, flags, src + start, i - start, dn ? dst + out : 0, dn ? dn - out : 0, def, used ? &u : 0);
            if (r <= 0) return 0;
            out += r; any_used |= u;
        }
        if (i >= len) break;
        if (dn) {
            if (out + 2 > dn) return 0;
            dst[out] = (char)(0xC7 + k / 84); dst[out + 1] = (char)trails[k % 84];
        }
        out += 2; i++; start = i;
    }
    if (used) *used = any_used;
    return out;
}

/* Nhập liệu: cửa sổ game là ANSI và lấy message bằng PeekMessageA, nên WM_CHAR ngoài bảng mã ACP thành '?'.
 * Hai hệ ô nhập của client (0x15222C8 và 0x1514A29) chỉ nhận chữ ngoài ASCII qua composition IME
 * (WM_IME_COMPOSITION + GCS_RESULTSTR); hệ thứ hai cố ý bỏ cặp DBCS đến bằng WM_CHAR. Hook lấy message bằng
 * PeekMessageW; WM_CHAR ngoài ASCII (Unikey/VK_PACKET hay xdotool) được đổi thành WM_IME_COMPOSITION với
 * GCS_RESULTSTR, và hai hook ImmGetCompositionStringA/W trả đúng ký tự đó (A: mã 949x) cho lần xử lý message này.
 * Composition thật của IME đi nguyên vẹn khi không có ký tự tiêm. */
typedef struct { void *hwnd; unsigned message, wParam; long lParam; unsigned time; long x, y; } msg_t;
typedef int(__attribute__((stdcall)) * peek_t)(msg_t *, void *, unsigned, unsigned, unsigned);
typedef long(__attribute__((stdcall)) * imm_t)(void *, unsigned, void *, unsigned);
typedef void *(__attribute__((stdcall)) * gmh_t)(const char *);
typedef void *(__attribute__((stdcall)) * gpa_t)(void *, const char *);
peek_t orig_peeka, peekw;
imm_t orig_imma, orig_immw;
static u16 inj_char;
static int inj_active;

#ifdef L10N_TRACE
/* Bản chẩn đoán: ghi sự kiện nhập liệu vào l10n-trace.log trong thư mục làm việc của game. */
typedef void *(__attribute__((stdcall)) * cfa_t)(const char *, unsigned, unsigned, void *, unsigned, unsigned, void *);
typedef int(__attribute__((stdcall)) * wf_t)(void *, const void *, unsigned, unsigned *, void *);
static cfa_t tr_create;
static wf_t tr_write;
static void *tr_file;
static void trace(char tag, unsigned a, unsigned b, unsigned c) {
    char line[48];
    unsigned v[3] = {a, b, c}, n = 0, w;
    if (!tr_file) return;
    line[n++] = tag;
    for (int k = 0; k < 3; k++) {
        line[n++] = ' ';
        for (int sh = 28; sh >= 0; sh -= 4) line[n++] = "0123456789abcdef"[(v[k] >> sh) & 15];
    }
    line[n++] = '\n';
    tr_write(tr_file, line, n, &w, 0);
}
#else
#define trace(tag, a, b, c) ((void)0)
#endif

__attribute__((stdcall)) int hook_peeka(msg_t *m, void *hwnd, unsigned fmin, unsigned fmax, unsigned flags) {
    int r;
    if (!peekw) return orig_peeka(m, hwnd, fmin, fmax, flags);
    if (flags & 1) inj_active = 0;
    r = peekw(m, hwnd, fmin, fmax, flags);
    if (r && m->message == 0x102 && m->wParam >= 0x80 && m->wParam <= 0xFFFF &&
        !(m->wParam >= 0xD800 && m->wParam <= 0xDFFF)) {
        l10n_stats[2]++;
        trace('P', flags, m->wParam, (unsigned)m->hwnd);
        if (flags & 1) { inj_char = (u16)m->wParam; inj_active = 1; }
        m->message = 0x10F; /* WM_IME_COMPOSITION */
        m->lParam = 0x800;  /* GCS_RESULTSTR */
    }
    return r;
}

__attribute__((stdcall)) long hook_immw(void *himc, unsigned index, void *buf, unsigned len) {
    trace('W', index, len, inj_active ? inj_char : 0xFFFFFFFF);
    if (!inj_active) return orig_immw(himc, index, buf, len);
    if (index != 0x800) return 0;
    if (len == 0) return 2;
    if (len < 2) return -1;
    *(u16 *)buf = inj_char;
    return 2;
}

__attribute__((stdcall)) long hook_imma(void *himc, unsigned index, void *buf, unsigned len) {
    char b[4];
    int n;
    trace('A', index, len, inj_active ? inj_char : 0xFFFFFFFF);
    if (!inj_active) return orig_imma(himc, index, buf, len);
    if (index != 0x800) return 0;
    n = hook_wc2mb(949, 0, &inj_char, 1, b, 4, 0, 0);
    if (n <= 0) { b[0] = '?'; n = 1; }
    if (len == 0) return n;
    if (len < (unsigned)n) return -1;
    for (int i = 0; i < n; i++) ((char *)buf)[i] = b[i];
    return n;
}

void l10n_init(void) {
    void *user32 = ((gmh_t) * (void **)IAT_GETMODULEHANDLEA)("user32.dll");
    orig_mb2wc = (mb2wc_t)*SLOT_MB2WC;
    orig_wc2mb = (wc2mb_t)*SLOT_WC2MB;
    orig_peeka = (peek_t) * (void **)IAT_PEEKMESSAGEA;
    orig_imma = (imm_t) * (void **)IAT_IMMGETCOMPOSITIONSTRINGA;
    orig_immw = (imm_t) * (void **)IAT_IMMGETCOMPOSITIONSTRINGW;
    peekw = user32 ? (peek_t)((gpa_t) * (void **)IAT_GETPROCADDRESS)(user32, "PeekMessageW") : 0;
    *SLOT_MB2WC = (void *)hook_mb2wc;
    *SLOT_WC2MB = (void *)hook_wc2mb;
    *(void **)IAT_PEEKMESSAGEA = (void *)hook_peeka;
    *(void **)IAT_IMMGETCOMPOSITIONSTRINGA = (void *)hook_imma;
    *(void **)IAT_IMMGETCOMPOSITIONSTRINGW = (void *)hook_immw;
    l10n_stats[3] = 0x39343978; /* "x949": dấu đã cài */
#ifdef L10N_TRACE
    {
        void *k32 = ((gmh_t) * (void **)IAT_GETMODULEHANDLEA)("kernel32.dll");
        gpa_t gpa = (gpa_t) * (void **)IAT_GETPROCADDRESS;
        tr_create = (cfa_t)gpa(k32, "CreateFileA");
        tr_write = (wf_t)gpa(k32, "WriteFile");
        if (tr_create && tr_write) tr_file = tr_create("l10n-trace.log", 0x40000000, 1, 0, 2, 0x80, 0);
        if (tr_file == (void *)-1) tr_file = 0;
        trace('I', 0, 0, 0);
    }
#endif
}

__asm__(".globl l10n_entry\n"
        "l10n_entry:\n"
        "  pushal\n"
        "  call l10n_init\n"
        "  popal\n"
        "  jmp *l10n_oep\n");
void *l10n_oep = (void *)OEP;
