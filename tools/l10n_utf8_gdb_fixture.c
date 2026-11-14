/* Owned i686 fixture: exact cdecl conversion boundary and output canaries. */
#include <windows.h>
#include <stdio.h>
#include <string.h>

__declspec(dllexport) __declspec(noinline)
int __cdecl l10n_test_convert(WCHAR *dst, int cap, const char *src, int length) {
    int count = MultiByteToWideChar(949, 0, src, length, dst, cap);
    if (count > 0 && length > 0 && count < cap) dst[count] = 0;
    return count;
}

int main(void) {
    static const char source[] = "\xc0\xfa\xb6\xfb \xb0\xd4\xc0\xd3\xc7\xcf\xbd\xc7\xba\xd0~";
    struct { unsigned before; WCHAR text[32]; unsigned after; } value;
    memset(&value, 0x55, sizeof(value));
    value.before = 0x12345678; value.after = 0x87654321;
    int count = l10n_test_convert(value.text, 32, source, (int)sizeof(source) - 1);
    printf("count=%d before=%08x after=%08x units=", count, value.before, value.after);
    for (int i = 0; i < count; ++i) printf("%04x", value.text[i]);
    printf(" nul=%04x\n", value.text[count]);
    return count == 9 && value.before == 0x12345678 && value.after == 0x87654321 && value.text[count] == 0 ? 0 : 1;
}
