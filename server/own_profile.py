"""Opt-in CP183 single-local-player profile291; not account authentication.

Reuse the measured zero-valued KR room-profile blob, binding its name to the
CP058 auth field. C's old room fixture says KRLocal while auth says KRNew.
Only the pinned ASCII/one-player/zero-profile branch is supported.
"""
import struct

from server import probe_codec as codec
from server.room_title import _decode_bytes, _encode_bytes


POLICY = {'mode': 'native_291'}
IDENTITIES = {'A': (1, b'KRLocal'), 'B': (2, b'KRGuest'), 'C': (3, b'KRNew')}


def body(profile, role):
    auth = profile.body('auth_' + role)
    if len(auth) < 31 or codec.read_fields(auth[4:5], [1]) != [1]:
        raise ValueError('own profile requires measured one-player auth')
    identity = codec.read_fields(auth[25:29], [4])[0]
    size = codec.read_fields(auth[29:31], [2])[0]
    if size not in (5, 7) or len(auth) != 130 + size:
        raise ValueError('unsupported own-profile auth name layout')
    name = _decode_bytes(auth[31:31 + size])
    if (identity, name) != IDENTITIES[role]:
        raise ValueError('own profile identity does not match fixture role')
    raw = profile.raw('player_' + role)
    data = profile.body('player_' + role)
    if struct.unpack_from('<I', raw, 5)[0] != 106 or len(data) < 20:
        raise ValueError('unsupported own-profile room fixture')
    size = codec.read_fields(data[16:20], [4])[0]
    if not 108 <= size <= 123 or len(data) < 20 + size:
        raise ValueError('unsupported own-profile blob size')
    stream = _decode_bytes(data[20:20 + size])
    if stream[:6] != b'\x18' + struct.pack('<I', size - 5) + b'\0':
        raise ValueError('own profile requires raw KR profile blob')
    inner = stream[6:]
    previous_size = struct.unpack_from('<I', inner)[0]
    if previous_size not in (5, 7) or inner[4 + previous_size:] != bytes(98):
        raise ValueError('own profile supports measured zero-profile fields only')
    inner = struct.pack('<I', len(name)) + name + inner[4 + previous_size:]
    stream = b'\x18' + struct.pack('<I', len(inner) + 1) + b'\0' + inner
    blob = codec.pack_fields([len(stream)], [4]) + _encode_bytes(stream)
    # active, profile blob, exp, opaque byte, feature106's three bytes.
    return (codec.pack_fields([1], [1]) + blob
            + codec.pack_fields([0, 0, 0, 0, 0], [4, 1, 1, 1, 1]))


def prepare(profile):
    if profile.data.get('own_profile') != POLICY:
        raise ValueError('unsupported own-profile policy')
    # These raw-only legacy fixtures have measured initial sequences. Decode
    # before listening; a changed fixture must not silently acquire a new pin.
    for alias, sequence in [('channel', 43), ('channel_list', 52)]:
        codec.frame_body(profile.raw(alias), sequence)
    return {role: body(profile, role) for role in IDENTITIES}
