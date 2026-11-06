"""Known CP147/162 team selector; values/layouts belong to KR00967 only."""
from server import probe_codec as codec
from server.room_title import create_title


def create_selector(body):
    """Reuse the bounded empty-password create81 policy; opaque tail stays opaque."""
    title = create_title(body)
    fields = codec.read_fields(body[4 + len(title):], [4] * 12 + [1] * 4)
    value = fields[1]
    if value not in (0, 1):
        raise ValueError('unsupported create81 team selector')
    return value


def replace_reply_selector(body, opcode, value):
    """86 first u8 after u16/u32; 87 second u8 after slot/u16 (8179B5)."""
    if value not in (0, 1):
        raise ValueError('unsupported room team selector')
    if opcode == 86 and len(body) == 11:
        offset = 7
    elif opcode == 87 and len(body) == 7:
        offset = 5
    elif opcode == 180 and len(body) == 3:
        fields = codec.read_fields(body, [1, 1, 1])
        if fields[0] != 0 or fields[2] != 0:
            raise ValueError('unsupported return180 branch/list variant')
        offset = 1
    else:
        raise ValueError('unsupported room team reply shape')
    if codec.read_fields(body[:1], [1]) != [0]:
        raise ValueError('team selector echo requires successful room reply')
    if codec.read_fields(body[offset:offset + 1], [1])[0] not in (0, 1):
        raise ValueError('unsupported template team selector')
    return body[:offset] + codec.pack_fields([value], [1]) + body[offset + 1:]
