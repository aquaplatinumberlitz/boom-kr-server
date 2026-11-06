"""Opt-in KR fixture policy; a fresh C identity may occupy the vacated B slot.

Pins come from the CP051/174 fixtures, not a recovered account-resume protocol.
Only the known 87/106 slot fields change; profile tails remain opaque.
"""
import hashlib
import struct

from server import probe_codec as codec


PINS = {
    'join_C': (87, '03d914f0c6aed5631f9bac1e8a97988fcd9065f993c6245aeb5c4b84fcd9850a'),
    'player_C': (106, '36f5a1a7d8d1442a50d4ddfa0f7ea9fbc1d19152c1e84826785103711999d694'),
    'slot_C': (110, 'ea6ecfe86ef4b6c764bdfda0a8f592a5299f6fbc5c0aa4cdd471407e60d93db1'),
    'ready_C': (110, 'fb469add1912e24a9b479c0b8fb798cff1d5f828f72b300aceec96512f4459af'),
    'guest_C': (106, '28f2074f78f12beb86d2ff4cea8e2aad96ccdd121017bb70f503ddbcc3516e22'),
    'guest_B': (106, 'e16fdf54a17382e7b34c42e1aa7271b65b6e99b0fdf6f8a0558dc185f0ee85c0'),
    'slot_B': (110, 'cc80bf0f63a8ade28fa65bfb04ff7312169b28aa10b394747e02949c7bdf7b70'),
    'ready_slot': (110, '2a8dcdd4d3db19010352bfaa1bde7c85eb15487b6f58d0a5dea280948f69cf17'),
}


def validate(profile):
    policy = profile.data['replacement_slot']
    if policy not in ({'mode': 'vacant_AB_slot'},
                      {'mode': 'vacant_AB_slot', 'capacity': 'vacated_slot'}):
        raise ValueError('unsupported replacement_slot policy')
    if any(key in profile.data for key in ('rescue_contact', 'needle_escape')):
        raise ValueError('replacement_slot excludes specialized A/B lifecycle policies')
    for alias, (opcode, digest) in PINS.items():
        if (alias not in profile.frames
                or struct.unpack_from('<I', profile.raw(alias), 5)[0] != opcode
                or hashlib.sha256(profile.body(alias)).hexdigest() != digest):
            raise ValueError('unsupported replacement_slot fixture: ' + alias)
    capacity = profile.data.get('bomb_capacity')
    if policy.get('capacity') == 'vacated_slot':
        if not isinstance(capacity, dict) or set(capacity) != {'A', 'B'}:
            raise ValueError('vacated_slot capacity requires exactly A/B bomb_capacity')
    elif capacity and 'C' not in capacity:
        raise ValueError('replacement_slot requires explicit C bomb_capacity')


def slot1_body(profile, alias):
    """Only used after a validated binding to A's room; no global fixture edits."""
    body = profile.body(alias)
    if alias == 'join_C':
        return body[:1] + codec.pack_fields([1], [1]) + body[2:]
    if alias == 'player_C':
        return codec.pack_fields([1], [1]) + body[1:]
    substitute = {'slot_C': 'slot_B', 'ready_C': 'ready_slot', 'guest_C': 'guest_B'}
    return profile.body(substitute[alias]) if alias in substitute else body
