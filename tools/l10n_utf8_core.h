#ifndef L10N_UTF8_CORE_H
#define L10N_UTF8_CORE_H

#include <stddef.h>
#include <stdint.h>

/* Explicit byte ranges; no implicit NUL, allocation, locale or normalization.
 * Return 0 on success, -1 for invalid UTF-8 or arguments, -2 for no space.
 * src, dst and out_len must not overlap. Destinations are unchanged on failure;
 * out_len is written only on success. */
int l10n_utf8_to_utf16(const uint8_t *src, size_t len, uint16_t *dst,
                        size_t capacity, size_t *out_len);
int l10n_utf16_to_utf8(const uint16_t *src, size_t len, uint8_t *dst,
                        size_t capacity, size_t *out_len);
int l10n_utf8_prefix(const uint8_t *src, size_t len, size_t max_bytes,
                      size_t *out_len);

#endif
