"""Strict text-boundary primitives for UTF-8 patch research, not a client patch.

Offsets and limits are bytes and boundaries are Unicode scalars, not grapheme
clusters. Inputs are explicit-length byte strings; an embedded NUL is data.
Decode failures raise UnicodeDecodeError instead of substituting or guessing a
codepage. Glyph widths must come from a separately verified provider.
"""

from collections.abc import Callable


def boundaries(data: bytes) -> tuple[int, ...]:
    """Return all code point boundaries, rejecting malformed or truncated UTF-8."""
    text = data.decode('utf-8', errors='strict')
    positions = [0]
    for char in text:
        positions.append(positions[-1] + len(char.encode('utf-8')))
    return tuple(positions)


def next_boundary(data: bytes, offset: int) -> int:
    positions = boundaries(data)
    if offset not in positions:
        raise ValueError('offset must be a UTF-8 code point boundary')
    index = positions.index(offset)
    return positions[min(index + 1, len(positions) - 1)]


def previous_boundary(data: bytes, offset: int) -> int:
    positions = boundaries(data)
    if offset not in positions:
        raise ValueError('offset must be a UTF-8 code point boundary')
    index = positions.index(offset)
    return positions[max(index - 1, 0)]


def truncate(data: bytes, max_bytes: int) -> bytes:
    """Return the longest valid prefix within max_bytes; never split a code point."""
    if max_bytes < 0:
        raise ValueError('max_bytes must be nonnegative')
    positions = boundaries(data)
    return data[:max(position for position in positions if position <= max_bytes)]


def to_utf16_units(data: bytes) -> tuple[int, ...]:
    """Convert valid UTF-8 to UTF-16 code units without a BOM or terminator."""
    raw = data.decode('utf-8', errors='strict').encode('utf-16-le', errors='strict')
    return tuple(int.from_bytes(raw[i:i + 2], 'little') for i in range(0, len(raw), 2))


def from_utf16_units(units: tuple[int, ...]) -> bytes:
    """Convert UTF-16 units to UTF-8; reject unpaired surrogates and invalid units."""
    if any(not isinstance(unit, int) or not 0 <= unit <= 0xffff for unit in units):
        raise ValueError('UTF-16 units must be integers from 0 to 65535')
    raw = b''.join(unit.to_bytes(2, 'little') for unit in units)
    return raw.decode('utf-16-le', errors='strict').encode('utf-8')


def measured_width(data: bytes, glyph_advance: Callable[[int], int]) -> int:
    """Sum caller-supplied advances per Unicode scalar; no font metrics are assumed."""
    text = data.decode('utf-8', errors='strict')
    total = 0
    for char in text:
        advance = glyph_advance(ord(char))
        if not isinstance(advance, int) or advance < 0:
            raise ValueError('glyph advances must be nonnegative integers')
        total += advance
    return total
