#include "l10n_utf8_core.h"

/* Decode one scalar. The caller must provide at least one byte. */
static int scalar(const uint8_t *s, size_t n, uint32_t *cp, size_t *used) {
    uint32_t value;
    size_t count;
    if (s[0] < 0x80) { *cp = s[0]; *used = 1; return 0; }
    if (s[0] >= 0xc2 && s[0] <= 0xdf) { value = s[0] & 0x1f; count = 2; }
    else if (s[0] >= 0xe0 && s[0] <= 0xef) { value = s[0] & 0x0f; count = 3; }
    else if (s[0] >= 0xf0 && s[0] <= 0xf4) { value = s[0] & 0x07; count = 4; }
    else return -1;
    if (n < count) return -1;
    for (size_t i = 1; i < count; ++i) {
        if ((s[i] & 0xc0) != 0x80) return -1;
        value = (value << 6) | (s[i] & 0x3f);
    }
    if ((count == 3 && value < 0x800) || (count == 4 && value < 0x10000) ||
        (value >= 0xd800 && value <= 0xdfff) || value > 0x10ffff) return -1;
    *cp = value; *used = count; return 0;
}

int l10n_utf8_to_utf16(const uint8_t *src, size_t len, uint16_t *dst,
                        size_t capacity, size_t *out_len) {
    size_t pos = 0, needed = 0, count;
    uint32_t cp;
    if ((!src && len) || !out_len || (!dst && capacity)) return -1;
    while (pos < len) {
        if (scalar(src + pos, len - pos, &cp, &count)) return -1;
        if (needed > SIZE_MAX - (cp > 0xffff ? 2u : 1u)) return -1;
        needed += cp > 0xffff ? 2u : 1u;
        pos += count;
    }
    if (needed > capacity) return -2;
    pos = 0; needed = 0;
    while (pos < len) {
        (void)scalar(src + pos, len - pos, &cp, &count);
        if (cp <= 0xffff) dst[needed++] = (uint16_t)cp;
        else {
            cp -= 0x10000;
            dst[needed++] = (uint16_t)(0xd800 + (cp >> 10));
            dst[needed++] = (uint16_t)(0xdc00 + (cp & 0x3ff));
        }
        pos += count;
    }
    *out_len = needed; return 0;
}

static int scalar16(const uint16_t *s, size_t n, uint32_t *cp, size_t *used) {
    uint32_t first = s[0];
    if (first >= 0xd800 && first <= 0xdbff) {
        if (n < 2 || s[1] < 0xdc00 || s[1] > 0xdfff) return -1;
        *cp = 0x10000 + ((first - 0xd800) << 10) + (s[1] - 0xdc00);
        *used = 2;
    } else {
        if (first >= 0xdc00 && first <= 0xdfff) return -1;
        *cp = first; *used = 1;
    }
    return 0;
}

int l10n_utf16_to_utf8(const uint16_t *src, size_t len, uint8_t *dst,
                        size_t capacity, size_t *out_len) {
    size_t pos = 0, needed = 0, count, bytes;
    uint32_t cp;
    if ((!src && len) || !out_len || (!dst && capacity)) return -1;
    while (pos < len) {
        if (scalar16(src + pos, len - pos, &cp, &count)) return -1;
        bytes = cp < 0x80 ? 1 : cp < 0x800 ? 2 : cp < 0x10000 ? 3 : 4;
        if (needed > SIZE_MAX - bytes) return -1;
        needed += bytes; pos += count;
    }
    if (needed > capacity) return -2;
    pos = 0; needed = 0;
    while (pos < len) {
        (void)scalar16(src + pos, len - pos, &cp, &count);
        if (cp < 0x80) dst[needed++] = (uint8_t)cp;
        else if (cp < 0x800) {
            dst[needed++] = (uint8_t)(0xc0 | (cp >> 6));
            dst[needed++] = (uint8_t)(0x80 | (cp & 0x3f));
        } else if (cp < 0x10000) {
            dst[needed++] = (uint8_t)(0xe0 | (cp >> 12));
            dst[needed++] = (uint8_t)(0x80 | ((cp >> 6) & 0x3f));
            dst[needed++] = (uint8_t)(0x80 | (cp & 0x3f));
        } else {
            dst[needed++] = (uint8_t)(0xf0 | (cp >> 18));
            dst[needed++] = (uint8_t)(0x80 | ((cp >> 12) & 0x3f));
            dst[needed++] = (uint8_t)(0x80 | ((cp >> 6) & 0x3f));
            dst[needed++] = (uint8_t)(0x80 | (cp & 0x3f));
        }
        pos += count;
    }
    *out_len = needed; return 0;
}

int l10n_utf8_prefix(const uint8_t *src, size_t len, size_t max_bytes,
                      size_t *out_len) {
    size_t pos = 0, end = 0, count;
    uint32_t cp;
    if ((!src && len) || !out_len) return -1;
    while (pos < len) {
        if (scalar(src + pos, len - pos, &cp, &count)) return -1;
        pos += count;
        if (pos <= max_bytes) end = pos;
    }
    *out_len = end; return 0;
}
