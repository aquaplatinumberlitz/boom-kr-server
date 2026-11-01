"""Narrow KR mode2/selector3 fixture codec; see PROTOCOL, CP006–047.

Only the measured 37-byte header and typed obfuscation are supported. Header
fields are preserved from a hash-checked fixture, not interpreted as recovered
authentication. Native client checks remain the oracle for compatibility.
"""

import struct

TYPE_XOR = {1: 0x5A, 2: 0xA569, 4: 0x96CA5395, 8: 0x71B18589}


def crc8(sequence: int, payload: bytes) -> int:
    value = sequence & 0xFF
    for byte in payload:
        value ^= byte
        for _ in range(8):
            value = ((value << 1) ^ (0x107 if value & 0x80 else 0)) & 0xFF
    return value


def crypt(payload: bytes, sequence: int, *, decrypt: bool = False) -> bytes:
    count = len(payload) // 4
    key = (sequence ^ 0x53351D9C) & 0xFFFFFFFF
    previous = 0
    result = bytearray()
    for index in range(count):
        if index:
            key = (key - 0x63CAACE3) & 0xFFFFFFFF
        word = int.from_bytes(payload[index * 4:index * 4 + 4], 'little')
        converted = word ^ key ^ previous
        result.extend(converted.to_bytes(4, 'little'))
        previous = converted if decrypt else word
    tail_key = previous if count else key
    for index, byte in enumerate(payload[count * 4:]):
        result.append(byte ^ ((tail_key >> (index * 8)) & 0xFF))
    return bytes(result)


def pack_fields(values, widths) -> bytes:
    if len(values) != len(widths):
        raise ValueError('field count mismatch')
    result = bytearray()
    for value, width in zip(values, widths):
        if type(value) is not int or not 0 <= value < (1 << (width * 8)):
            raise ValueError('field outside unsigned width')
        result.extend((value ^ TYPE_XOR[width]).to_bytes(width, 'big'))
    return bytes(result)


def read_fields(payload: bytes, widths) -> list[int]:
    if len(payload) != sum(widths):
        raise ValueError('unsupported field layout')
    result, offset = [], 0
    for width in widths:
        result.append(int.from_bytes(payload[offset:offset + width], 'big') ^ TYPE_XOR[width])
        offset += width
    return result


def frame_body(frame: bytes, sequence: int) -> bytes:
    if len(frame) < 38 or struct.unpack_from('<I', frame, 13)[0] + 38 != len(frame):
        raise ValueError('frame length mismatch')
    body = crypt(frame[37:-1], sequence, decrypt=True)
    if crc8(sequence, body) != frame[-1]:
        raise ValueError('payload checksum mismatch')
    return body


def encode(header: bytes, opcode: int, body: bytes, sequence: int) -> bytes:
    if len(header) != 37 or not 0 <= opcode <= 0xFFFFFFFF or len(body) > 65536 - 38:
        raise ValueError('unsupported frame')
    result = bytearray(header)
    struct.pack_into('<I', result, 5, opcode)
    struct.pack_into('<I', result, 13, len(body))
    return bytes(result) + crypt(body, sequence) + bytes([crc8(sequence, body)])


def pop_frame(buffer: bytearray) -> bytes | None:
    if len(buffer) < 38:
        return None
    size = struct.unpack_from('<I', buffer, 13)[0] + 38
    if size > 65536:
        raise ValueError('frame exceeds diagnostic bound')
    if len(buffer) < size:
        return None
    result = bytes(buffer[:size])
    del buffer[:size]
    return result
