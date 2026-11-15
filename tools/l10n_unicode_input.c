/* Gõ chữ Unicode vào cửa sổ đang focus bằng SendInput(KEYEVENTF_UNICODE), như bộ gõ tiếng Việt kiểu Unikey.
 * Chạy trong cùng Wine prefix/wineserver với client thử. Tham số: chuỗi UTF-16LE dạng hex.
 * Dựng: i686-w64-mingw32-gcc -O2 -o l10n_unicode_input.exe tools/l10n_unicode_input.c
 */
#include <windows.h>
#include <stdio.h>

static int hexval(char c) {
    if (c >= '0' && c <= '9') return c - '0';
    if (c >= 'a' && c <= 'f') return c - 'a' + 10;
    if (c >= 'A' && c <= 'F') return c - 'A' + 10;
    return -1;
}

int main(int argc, char **argv) {
    INPUT in[2];
    int i, n, sent = 0, delay = 120;
    if (argc < 2) return 2;
    if (argc > 2) delay = atoi(argv[2]);
    n = (int)strlen(argv[1]);
    if (n % 4) return 2;
    for (i = 0; i < n; i += 4) {
        int a = hexval(argv[1][i]), b = hexval(argv[1][i + 1]), c = hexval(argv[1][i + 2]), d = hexval(argv[1][i + 3]);
        WORD ch;
        if (a < 0 || b < 0 || c < 0 || d < 0) return 2;
        ch = (WORD)((c << 12) | (d << 8) | (a << 4) | b); /* UTF-16LE */
        ZeroMemory(in, sizeof in);
        in[0].type = in[1].type = INPUT_KEYBOARD;
        in[0].ki.wScan = in[1].ki.wScan = ch;
        in[0].ki.dwFlags = KEYEVENTF_UNICODE;
        in[1].ki.dwFlags = KEYEVENTF_UNICODE | KEYEVENTF_KEYUP;
        sent += (int)SendInput(2, in, sizeof(INPUT));
        Sleep(delay);
    }
    printf("sent %d events for %d units, foreground %p\n", sent, n / 4, (void *)GetForegroundWindow());
    return sent == n / 2 ? 0 : 1;
}
