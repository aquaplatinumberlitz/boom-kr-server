/* Ordinary input for an explicitly selected foreground Windows/Wine client. */
#define WIN32_LEAN_AND_MEAN
#include <windows.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include "native_unicode_input.h"

static int number(const char *s, unsigned long min, unsigned long max, unsigned long *out) {
    char *end;
    unsigned long value;
    if (!s[0] || strspn(s, "0123456789") != strlen(s)) return 0;
    value = strtoul(s, &end, 10);
    if (*end || value < min || value > max) return 0;
    *out = value;
    return 1;
}

static HWND foreground(DWORD pid) {
    HWND window = GetForegroundWindow();
    DWORD actual = 0;
    if (window) GetWindowThreadProcessId(window, &actual);
    return actual == pid ? window : NULL;
}

static BOOL CALLBACK list_windows(HWND window, LPARAM ignored) {
    DWORD pid;
    char title[256];
    (void)ignored;
    if (IsWindowVisible(window) && GetWindowTextA(window, title, sizeof title)) {
        GetWindowThreadProcessId(window, &pid);
        printf("%lu %s\n", (unsigned long)pid, title);
    }
    return TRUE;
}

static int hold(INPUT down, DWORD up, DWORD ms, HWND window, DWORD pid) {
    if (foreground(pid) != window || SendInput(1, &down, sizeof down) != 1) return 1;
    Sleep(ms);
    if (down.type == INPUT_MOUSE) down.mi.dwFlags = up;
    else down.ki.dwFlags |= up;
    if (SendInput(1, &down, sizeof down) != 1) return 1;
    return foreground(pid) == window ? 0 : 1;
}

static int hex(char c) {
    if ('0' <= c && c <= '9') return c - '0';
    if ('a' <= c && c <= 'f') return c - 'a' + 10;
    return -1;
}

static int unicode(const char *text, DWORD ms, HWND window, DWORD pid) {
    size_t n = strlen(text);
    if (!n || n % 4 || n > 256) return 2;
    for (size_t i = 0; i < n; ++i) if (hex(text[i]) < 0) return 2;
    for (size_t i = 0; i < n; i += 4) {
        WORD value = (WORD)((hex(text[i+2]) << 12) | (hex(text[i+3]) << 8)
                          | (hex(text[i]) << 4) | hex(text[i+1]));
        if (foreground(pid) != window) return 1;
        if (!kre_unicode_unit(value)) return 1;
        Sleep(ms);
    }
    return foreground(pid) == window ? 0 : 1;
}

int main(int argc, char **argv) {
    unsigned long pid, a, b, ms;
    INPUT down = {0};
    if (argc == 2 && !strcmp(argv[1], "windows")) return EnumWindows(list_windows, 0) ? 0 : 1;
    if (argc < 3 || !number(argv[1], 1, MAXDWORD, &pid)) return 2;
    HWND window = foreground((DWORD)pid);
    if (!window) {
        fputs("Selected client is not the foreground window.\n", stderr);
        return 1;
    }
    if (argc == 3 && !strcmp(argv[2], "check")) return 0;
    if (argc == 6 && !strcmp(argv[2], "click") &&
            number(argv[3], 0, 16383, &a) && number(argv[4], 0, 16383, &b) &&
            number(argv[5], 1, 2000, &ms)) {
        RECT rc;
        POINT point = {(LONG)a, (LONG)b};
        if (!GetClientRect(window, &rc) || !PtInRect(&rc, point) ||
                !ClientToScreen(window, &point) || !SetCursorPos(point.x, point.y)) return 1;
        if (GetAncestor(WindowFromPoint(point), GA_ROOT) != GetAncestor(window, GA_ROOT)) return 1;
        down.type = INPUT_MOUSE;
        down.mi.dwFlags = MOUSEEVENTF_LEFTDOWN;
        return hold(down, MOUSEEVENTF_LEFTUP, (DWORD)ms, window, (DWORD)pid);
    }
    if (argc == 5 && !strcmp(argv[2], "key") &&
            number(argv[3], 1, 254, &a) && number(argv[4], 1, 2000, &ms)) {
        down.type = INPUT_KEYBOARD;
        down.ki.wVk = (WORD)a;
        if ((VK_PRIOR <= a && a <= VK_DOWN) || a == VK_INSERT || a == VK_DELETE)
            down.ki.dwFlags = KEYEVENTF_EXTENDEDKEY;
        return hold(down, KEYEVENTF_KEYUP, (DWORD)ms, window, (DWORD)pid);
    }
    if (argc == 5 && !strcmp(argv[2], "unicode") && number(argv[4], 0, 60, &ms))
        return unicode(argv[3], (DWORD)ms, window, (DWORD)pid);
    fputs("Expected PID check|click X Y HOLD_MS|key VK HOLD_MS|unicode UTF16LE_HEX DELAY_MS.\n", stderr);
    return 2;
}
