#!/usr/bin/env python3
"""Index local synthetic fixtures already measured in CP020–047; copy no assets."""

import argparse
import hashlib
import json
from pathlib import Path

# Paths are in this repo's ignored codex research tree, not the VN server tree.
FIXTURES = {
    'bootstrap': ('bootstrap-reply-r1.bin', None),
    'second': ('wcp-fields-r1/eight-v2-responses.bin', None),
    'auth_A': ('start-reply-r1/auth-id1-prefix.bin', None),
    'auth_B': ('peer-duo-r1/auth-id2.bin', None),
    'netinfo': ('login-runtime-r1/login-netinfo-candidate.bin', None),
    'flags': ('postwait-r1/flags62-status2.bin', None),
    'redirect': ('postwait-r1/redirect7-loopback.bin', None),
    'channel': ('postwait-r1/channel17-zero-arrays.bin', None),
    'ready': ('bomb-position-r1/exvillage241.bin', 46),
    'roomlist_A': ('room-input-r1/room-list91-empty.bin', 49),
    'roomlist_B': ('duo-round-r1/room91-session1.bin', None),
    'channel_list': ('leave-r1/channel14-kind1.bin', None),
    'create': ('room-reply-r1/reply86.bin', None),
    'join': ('duo-round-r1/reply87-slot1-session1.bin', None),
    'player_A': ('start-reply-r1/player106-id1-team0.bin', None),
    'player_B': ('duo-round-r1/B106-unready.bin', None),
    'slot_A': ('room-player-r1/slot110-team0.bin', 61),
    'slot_B': ('duo-round-r1/B110-unready.bin', 61),
    'guest_A': ('duo-round-r1/A-guest106-unready.bin', None),
    'guest_B': ('peer-duo-r1/B-guest106.bin', None),
    'table': ('bomb-position-r1/exvillage238.bin', None),
    'map': ('bomb-position-r1/exvillage117.bin', None),
    'floor': ('item-families-r1/floor582-kind2.bin', 73),
    'start': ('collision-r1/overlap121.bin', 73),
    'go': ('round-go-r1/go197-time30.bin', 76),
    'play': ('bomb-position-r1/exvillage196.bin', 79),
    'peer': ('peer-duo-r1/peer153-replace.bin', 82),
    'ready_slot': ('round-repeat-r1/ready110-guest.bin', 100),
    'bomb': ('round-go-r1/reply159.bin', 85),
    'blast': ('bomb-calc-r1/kind0.bin', None),
    'rank': ('round-repeat-r1/rank175-v2.bin', 91),
    'rank_Awin': ('peer-events-r1/rank175-Awin-Blose.bin', 91),
    'promote_B': ('leave-r1/promote110-B.bin', 73),
    'return': ('bomb-calc-r1/result180-full.bin', 91),
}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--source-root', type=Path, required=True)
    parser.add_argument('--out', type=Path, required=True)
    args = parser.parse_args()
    local = Path(__file__).resolve().parents[1] / 'local'
    root = args.source_root.resolve(strict=True)
    out = args.out.resolve()
    if not root.is_relative_to(local) or not out.is_relative_to(local):
        parser.error('profile and fixtures must be under this repo local/')
    result = {'schema': 'kr-probe-profile-v1', 'source': 'emu_choice',
              'limits': 'Synthetic local diagnostic fixtures, not recovered full auth or original server.',
              'items': [{'index': 17, 'row': 4, 'column': 3, 'kind': 2}],
              'initial_key': 12345, 'fixtures': {}}
    for name, (relative, sequence) in FIXTURES.items():
        path = (root / relative).resolve(strict=True)
        if not path.is_relative_to(root):
            raise ValueError('fixture escapes source root')
        raw = path.read_bytes()
        if not 1 <= len(raw) <= 65536:
            raise ValueError(f'unsupported fixture size: {name}')
        result['fixtures'][name] = {'path': str(path), 'bytes': len(raw),
                                    'sha256': hashlib.sha256(raw).hexdigest(), 'sequence': sequence}
    out.parent.mkdir(parents=True, exist_ok=True)
    with out.open('x') as stream:
        json.dump(result, stream, indent=2)
        stream.write('\n')
    print(f'Indexed {len(FIXTURES)} fixtures: {out}')


if __name__ == '__main__':
    main()
