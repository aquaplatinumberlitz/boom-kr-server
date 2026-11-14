/* Standalone i686 Windows API/primitive comparison. Does not load a game. */
#include <windows.h>
#include <stdint.h>
#include <stdio.h>
#include <string.h>
#include "l10n_utf8_core.h"

static int check(const char *name, const uint8_t *data, int length) {
    WCHAR expected[64]; uint16_t actual[64]; uint8_t roundtrip[128];
    size_t units = 999, bytes = 999;
    int n = MultiByteToWideChar(CP_UTF8, MB_ERR_INVALID_CHARS,
                                (const char *)data, length, expected, 64);
    int rc = l10n_utf8_to_utf16(data, (size_t)length, actual, 64, &units);
    int pass = n > 0 && rc == 0 && units == (size_t)n &&
               memcmp(actual, expected, units * sizeof(uint16_t)) == 0;
    if (pass) pass = l10n_utf16_to_utf8(actual, units, roundtrip, sizeof(roundtrip), &bytes) == 0 &&
                     bytes == (size_t)length && memcmp(roundtrip, data, bytes) == 0;
    printf("%s %s units=%lu bytes=%lu\n", name, pass ? "PASS" : "FAIL",
           (unsigned long)units, (unsigned long)bytes);
    return pass;
}

int main(void) {
    static const uint8_t sample[] = {0x41,0xed,0x95,0x9c,0x56,0x69,0xe1,0xbb,0x87,0x74,0xf0,0x9f,0x98,0x80};
    static const uint8_t invalid[] = {0xed,0xa0,0x80};
    uint16_t dst[8] = {0xdead}; size_t length = 999, cut = 999;
    WCHAR wide[8]; int pass = check("mixed", sample, sizeof(sample));
    SetLastError(0);
    int api = MultiByteToWideChar(CP_UTF8, MB_ERR_INVALID_CHARS,
                                 (const char *)invalid, sizeof(invalid), wide, 8);
    int core = l10n_utf8_to_utf16(invalid, sizeof(invalid), dst, 8, &length);
    int valid = api == 0 && core == -1 && dst[0] == 0xdead && length == 999;
    printf("invalid %s winerr=%lu\n", valid ? "PASS" : "FAIL", (unsigned long)GetLastError());
    pass &= valid;
    valid = l10n_utf8_prefix(sample, sizeof(sample), 8, &cut) == 0 && cut == 6;
    printf("boundary %s cut=%lu\n", valid ? "PASS" : "FAIL", (unsigned long)cut);
    pass &= valid;
    length = 999;
    valid = l10n_utf8_to_utf16(sample, sizeof(sample), dst, 1, &length) == -2 &&
            dst[0] == 0xdead && length == 999;
    printf("capacity %s\n", valid ? "PASS" : "FAIL");
    return pass && valid ? 0 : 1;
}
