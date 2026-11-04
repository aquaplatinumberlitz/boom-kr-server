"""Trusted local configuration for the measured map, not client authority."""
import hashlib
import re

from server.terrain_model import TerrainShadow


def load_terrain(spec, frames):
    required = {'map_sha256', 'seed_sha256', 'fixtures', 'width', 'height',
                'kinds', 'maximum_power'}
    if not isinstance(spec, dict) or set(spec) != required:
        raise ValueError('invalid bomb_terrain profile')
    if (spec['width'] != 15 or type(spec['width']) is not int
            or spec['height'] != 13 or type(spec['height']) is not int
            or not isinstance(spec['kinds'], list)
            or len(spec['kinds']) != 195
            or any(type(k) is not int or k not in (0, 1, 2) for k in spec['kinds'])):
        raise ValueError('bomb_terrain only supports the declared 15x13 seed')
    for field in ['map_sha256', 'seed_sha256']:
        if not isinstance(spec[field], str) or not re.fullmatch('[0-9a-f]{64}', spec[field]):
            raise ValueError('invalid bomb_terrain digest')
    if hashlib.sha256(bytes(spec['kinds'])).hexdigest() != spec['seed_sha256']:
        raise ValueError('bomb_terrain seed hash mismatch')
    pins = spec['fixtures']
    if (not isinstance(pins, dict) or set(pins) != {'map', 'start'}
            or any(name not in frames or not isinstance(frames[name], bytes)
                   or hashlib.sha256(frames[name]).hexdigest() != expected
                   for name, expected in pins.items())):
        raise ValueError('bomb_terrain fixture hash mismatch')
    # The configuration records the source-map digest; operators establish its
    # provenance during export. This is not a signature or asset attestation.
    return TerrainShadow(spec['width'], spec['height'], spec['kinds'],
                         map_sha256=spec['map_sha256'],
                         maximum_power=spec['maximum_power'])
