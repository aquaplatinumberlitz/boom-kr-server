"""Measured zero-profile transformations for the four-session diagnostic.

CP020/026/199 wire branches only. Identities, slots and teams are operator
choices; generated frames are inputs, never a native compatibility oracle.
"""
import struct
from server import probe_codec as codec
from server.room_title import _decode_bytes, _encode_bytes

ROLES = 'ABCD'
MEMBERS = {
    'A': {'identity': 1, 'slot': 0, 'team': 0, 'name': 'KRLocal'},
    'B': {'identity': 2, 'slot': 1, 'team': 1, 'name': 'KRGuest'},
    'C': {'identity': 3, 'slot': 2, 'team': 0, 'name': 'KRNew'},
    'D': {'identity': 4, 'slot': 3, 'team': 1, 'name': 'KRPeer4'},
}

def rename_profile(data, name):
    """Replace both measured 106 names; require the exact zero-profile branch."""
    if not isinstance(name, bytes) or len(name) not in (5, 7) or not name.isascii():
        raise ValueError('unsupported four-player ASCII name')
    if len(data) < 20:
        raise ValueError('truncated player106')
    size = codec.read_fields(data[16:20], [4])[0]
    if not 108 <= size <= 123 or len(data) < 25 + size:
        raise ValueError('unsupported player106 profile size')
    stream = _decode_bytes(data[20:20 + size])
    if stream[:6] != b'\x18' + struct.pack('<I', size - 5) + b'\0':
        raise ValueError('unsupported player106 profile envelope')
    inner = stream[6:]
    previous_size = struct.unpack_from('<I', inner)[0]
    if previous_size not in (5, 7) or inner[4 + previous_size:] != bytes(98):
        raise ValueError('four-player requires measured zero-profile')
    previous_name = inner[4:4 + previous_size]
    tail = data[20 + size:]
    outer_size = codec.read_fields(tail[3:5], [2])[0]
    if outer_size != previous_size or _decode_bytes(tail[5:5 + outer_size]) != previous_name:
        raise ValueError('player106 names disagree')
    inner = struct.pack('<I', len(name)) + name + bytes(98)
    stream = b'\x18' + struct.pack('<I', len(inner) + 1) + b'\0' + inner
    blob = codec.pack_fields([len(stream)], [4]) + _encode_bytes(stream)
    tail = tail[:3] + codec.pack_fields([len(name)], [2]) + _encode_bytes(name) + tail[5 + outer_size:]
    return data[:16] + blob + tail


def player_body(template, member, *, ready=False, chief=False):
    data = bytearray(rename_profile(template, member['name'].encode('ascii')))
    data[0:5] = codec.pack_fields([member['slot'], member['identity']], [1, 4])
    # Preserve the measured character field; change only the known low state.
    state = int.from_bytes(_decode_bytes(data[12:16]), 'little')
    low = (2 if chief else 4 if ready else 0) | (member['team'] << 5)
    data[12:16] = _encode_bytes(struct.pack('<I', (state & 0xFFFF0000) | low))
    return bytes(data)


def slot_body(template106, member, *, ready=False, chief=False):
    player = player_body(template106, member, ready=ready, chief=chief)
    return player[:1] + player[12:16] + codec.pack_fields([0], [1])


def auth_body(template, member):
    if len(template) < 31 or codec.read_fields(template[4:5], [1]) != [1]:
        raise ValueError('four-player requires one-player auth')
    size = codec.read_fields(template[29:31], [2])[0]
    if size not in (5, 7) or len(template) != 130 + size:
        raise ValueError('unsupported auth profile branch')
    name = member['name'].encode('ascii')
    return (template[:25] + codec.pack_fields([member['identity'], len(name)], [4, 2])
            + _encode_bytes(name) + template[31 + size:])


def own_body(player106):
    size = codec.read_fields(player106[16:20], [4])[0]
    return (codec.pack_fields([1], [1]) + player106[16:20 + size]
            + codec.pack_fields([0, 0, 0, 0, 0], [4, 1, 1, 1, 1]))


def join_body(template, slot):
    if len(template) != 7 or codec.read_fields(template[:1], [1]) != [0]:
        raise ValueError('unsupported join87 branch')
    return template[:1] + codec.pack_fields([slot], [1]) + template[2:]


def start_body(template):
    """Reuse the uncompressed CP020 inner, preserving the outer tail byte-for-byte."""
    if len(template) != 129 or codec.read_fields(template[:1], [1]) != [0]:
        raise ValueError('four-player requires measured two-row121')
    size = codec.read_fields(template[1:5], [4])[0]
    raw = _decode_bytes(template[5:5 + size])
    if size != 106 or raw[:6] != b'\x18' + struct.pack('<I', 101) + b'\0':
        raise ValueError('unsupported start121 inner envelope')
    inner = raw[6:]
    if len(inner) != 100 or inner[43] != 2 or inner[48] != 0 or inner[74] != 1:
        raise ValueError('unsupported start121 rows')
    rows = [inner[48:74], inner[74:100]]
    for role in 'CD':
        m = MEMBERS[role];row = bytearray(rows[1]);row[0] = m['slot'];row[2] = m['slot']
        struct.pack_into('<H', row, 3, m['slot']);row[5] = m['team'];rows.append(bytes(row))
    prefix = bytearray(inner[:48]);prefix[43] = 4;inner = bytes(prefix) + b''.join(rows)
    raw = b'\x18' + struct.pack('<I', len(inner) + 1) + b'\0' + inner
    return template[:1] + codec.pack_fields([len(raw)], [4]) + _encode_bytes(raw) + template[5 + size:]


def peer_body(template, key, recipient, endpoints):
    """Four active slots, zero self, explicit primary tuple for each remote."""
    if len(template) < 35 or recipient not in MEMBERS or set(endpoints) != set(ROLES):
        raise ValueError('incomplete four-player peer roster')
    data = bytearray(template[:35]);data[15:19] = codec.pack_fields([key], [4])
    by_slot = {member['slot']: role for role, member in MEMBERS.items()}
    for slot in range(26):
        data.extend(codec.pack_fields([int(slot in by_slot)], [1]))
        if slot in by_slot:
            role = by_slot[slot];ip, port = endpoints[role]
            if not 0 < ip < 0xFFFFFFFF or not 1 <= port <= 65535:
                raise ValueError('invalid configured peer endpoint')
            values = [ip, port, 0, 0, 0, 0] if role != recipient else [0] * 6
            data.extend(codec.pack_fields(values, [4, 2] * 3))
    return bytes(data)
