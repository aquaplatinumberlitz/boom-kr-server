"""Bounded CP016/028/076 title echo; this is not room authority or a general codec."""
import struct

from server import probe_codec as codec


MAX_TITLE = 32  # Emulator policy, not a recovered universal client limit.


def _decode_bytes(data):
    result = bytearray()
    offset = 0
    while offset < len(data):
        width = 4 if len(data) - offset >= 4 else 2 if len(data) - offset >= 2 else 1
        value = codec.read_fields(data[offset:offset + width], [width])[0]
        result.extend(value.to_bytes(width, 'little'))
        offset += width
    return bytes(result)


def _encode_bytes(data):
    result = bytearray()
    offset = 0
    while offset < len(data):
        width = 4 if len(data) - offset >= 4 else 2 if len(data) - offset >= 2 else 1
        result.extend(codec.pack_fields([int.from_bytes(data[offset:offset + width], 'little')], [width]))
        offset += width
    return bytes(result)


def _validate_title(title):
    if not 1 <= len(title) <= MAX_TITLE or any(byte < 32 or byte == 127 for byte in title):
        raise ValueError('unsupported room title bytes')
    try:
        title.decode('cp949', errors='strict')
    except UnicodeDecodeError as error:
        raise ValueError('unsupported room title encoding') from error


def create_title(body):
    """Two typed strings, twelve opaque u32 and four opaque u8; empty password only."""
    if not 56 <= len(body) <= 56 + MAX_TITLE:
        raise ValueError('unsupported create81 size')
    size = codec.read_fields(body[:2], [2])[0]
    if not 1 <= size <= MAX_TITLE or size + 56 != len(body):
        raise ValueError('unsupported create81 title length')
    title = _decode_bytes(body[2:2 + size])
    if codec.read_fields(body[2 + size:4 + size], [2]) != [0]:
        raise ValueError('room title echo requires empty password')
    codec.read_fields(body[4 + size:], [4] * 12 + [1] * 4)
    _validate_title(title)
    return title


def replace_listing_title(body, title):
    """Only the measured single uncompressed row; session/room ID/tail stay exact."""
    _validate_title(title)
    if len(body) < 2:
        raise ValueError('short room91 body')
    count = codec.read_fields(body[:2], [2])[0]
    if count == 0 and len(body) == 2:
        return body
    if count != 1 or not 16 <= len(body) <= 512:
        raise ValueError('unsupported room91 row count or size')
    size = codec.read_fields(body[2:6], [4])[0]
    if size != len(body) - 6:
        raise ValueError('room91 blob length mismatch')
    stream = _decode_bytes(body[6:])
    if stream[0] != 0x18 or struct.unpack_from('<I', stream, 1)[0] != len(stream) - 5:
        raise ValueError('unsupported room91 stream')
    if stream[5] != 0:
        raise ValueError('room91 compressed row unsupported')
    row = stream[6:]
    if len(row) < 32:
        raise ValueError('short room91 row')
    previous_size = struct.unpack_from('<I', row, 6)[0]
    if not 1 <= previous_size <= MAX_TITLE or len(row) != 32 + previous_size:
        raise ValueError('unsupported room91 title/tail layout')
    _validate_title(row[10:10 + previous_size])
    updated = row[:6] + struct.pack('<I', len(title)) + title + row[10 + previous_size:]
    stream = b'\x18' + struct.pack('<I', len(updated) + 1) + b'\0' + updated
    return body[:2] + codec.pack_fields([len(stream)], [4]) + _encode_bytes(stream)
