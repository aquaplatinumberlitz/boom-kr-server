#ifndef KRE_NATIVE_UNICODE_INPUT_H
#define KRE_NATIVE_UNICODE_INPUT_H

#include <windows.h>

/* Preserve the UTF-16 code-unit order, including surrogate pairs. */
static inline int kre_unicode_unit(WORD value) {
    INPUT input[2] = {0};
    input[0].type = input[1].type = INPUT_KEYBOARD;
    input[0].ki.wScan = input[1].ki.wScan = value;
    input[0].ki.dwFlags = KEYEVENTF_UNICODE;
    input[1].ki.dwFlags = KEYEVENTF_UNICODE | KEYEVENTF_KEYUP;
    return SendInput(2, input, sizeof(INPUT)) == 2;
}
#endif
