#!/usr/bin/env python3
"""Index local synthetic fixtures; select known distinct or overlap spawns."""

import argparse
import hashlib
import json
from pathlib import Path
import struct
import copy
import re

PRESET = Path(__file__).resolve().parents[1] / 'server/profiles/two-client-battle.json'


def load_preset(root, preset=PRESET):
    """Rebase pinned fixture paths; never rehash changed input into trusted metadata."""
    spec = json.loads(preset.read_text())
    if spec.get('schema') != 'kre-probe-preset-v1':
        raise ValueError('unsupported preset schema')
    result = copy.deepcopy(spec['profile'])
    for alias, entry in result['fixtures'].items():
        relative = Path(entry['path'])
        path = (root / relative).resolve(strict=True)
        if relative.is_absolute() or not path.is_relative_to(root):
            raise ValueError('preset fixture escapes source root: ' + alias)
        if (type(entry['bytes']) is not int or not 1 <= entry['bytes'] <= 65536
                or not re.fullmatch('[0-9a-f]{64}', str(entry['sha256']))
                or path.stat().st_size != entry['bytes']
                or hashlib.sha256(path.read_bytes()).hexdigest() != entry['sha256']):
            raise ValueError('preset fixture size/hash mismatch: ' + alias)
        entry['path'] = str(path)
    return result

# Paths are in this repo's ignored codex research tree, not the VN server tree.
FIXTURES = {
    'bootstrap': ('bootstrap-reply-r1.bin', None),
    'second': ('wcp-fields-r1/eight-v2-responses.bin', None),
    'auth_A': ('auth-profile-r1/auth-A-final.bin', 43),
    'auth_B': ('auth-profile-r1/auth-B-final.bin', 43),
    'auth_C': ('auth-profile-r1/auth-C-final.bin', 43),
    'roomlist_C': ('duo-round-r1/room91-session1.bin', None),
    'join_C': ('rejoin-r1/reply87-A-session1.bin', 55),
    'player_C': ('abort-rejoin-r1/C106-slot0-id3.bin', 58),
    'slot_C': ('rejoin-r1/A110-unready.bin', 61),
    'guest_C': ('abort-rejoin-r1/C-guestB106-chief64.bin', 64),
    'ready_C': ('rejoin-r1/A110-ready.bin', 100),
    'netinfo': ('login-runtime-r1/login-netinfo-candidate.bin', None),
    'flags': ('postwait-r1/flags62-status2.bin', None),
    'redirect': ('postwait-r1/redirect7-loopback.bin', None),
    'channel': ('postwait-r1/channel17-zero-arrays.bin', None),
    'ready': ('cache-boundary-r1/compressed241.bin', 46),
    'roomlist_A': ('room-input-r1/room-list91-empty.bin', 49),
    'roomlist_B': ('duo-round-r1/room91-session1.bin', 49),
    'channel_list': ('leave-r1/channel14-kind1.bin', None),
    'create': ('room-reply-r1/reply86.bin', 55),
    'join': ('duo-round-r1/reply87-slot1-session1.bin', 55),
    'player_A': ('start-reply-r1/player106-id1-team0.bin', 58),
    'player_B': ('duo-round-r1/B106-unready.bin', 58),
    'slot_A': ('room-player-r1/slot110-team0.bin', 61),
    'slot_B': ('duo-round-r1/B110-unready.bin', 61),
    'guest_A': ('duo-round-r1/A-guest106-unready.bin', 64),
    'guest_B': ('peer-duo-r1/B-guest106.bin', 64),
    'table': ('cache-boundary-r1/compressed238.bin', 67),
    'map': ('bomb-position-r1/exvillage117.bin', 70),
    'floor': ('item-families-r1/floor582-kind2.bin', 73),
    'start': ('bomb-position-r1/exvillage121.bin', 73),
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
    parser.add_argument('--preset', choices=['two-client-battle'],
                        help='pinned CP159 production baseline; requires exact local fixtures')
    parser.add_argument('--spawn-layout', choices=['distinct', 'overlap'], default='distinct',
                        help='known map spawn0/1, or intentional overlap for contact/race RE')
    parser.add_argument('--item-layout', choices=['floor', 'hidden', 'battle'], default='floor',
                        help='known floor Roller, hidden Bomb/Fire, or both known groups together')
    parser.add_argument('--all-dead-rank', type=Path,
                        help='optional measured KR175 rank2/2 fixture under source-root (base seq91)')
    parser.add_argument('--host-back-rejoin', action='store_true',
                        help='opt-in measured idle A Back/B promotion/A rejoin fixture policy (CP035)')
    parser.add_argument('--host-battle-exit', action='store_true',
                        help='opt-in measured unbombed host A Battle exit (CP129); includes host Back/rejoin fixtures')
    args = parser.parse_args()
    local = Path(__file__).resolve().parents[1] / 'local'
    root = args.source_root.resolve(strict=True)
    out = args.out.resolve()
    if not root.is_relative_to(local) or not out.is_relative_to(local):
        parser.error('profile and fixtures must be under this repo local/')
    if args.preset:
        if (args.spawn_layout != 'distinct' or args.item_layout != 'floor'
                or args.all_dead_rank or args.host_back_rejoin or args.host_battle_exit):
            parser.error('--preset cannot be combined with layout or optional policy flags')
        result = load_preset(root)
        out.parent.mkdir(parents=True, exist_ok=True)
        with out.open('x') as stream:
            json.dump(result, stream, indent=2)
            stream.write('\n')
        print(f'Exported pinned {args.preset}: {len(result["fixtures"])} fixtures; runtime gates not certified')
        return
    result = {'schema': 'kr-probe-profile-v1', 'source': 'emu_choice',
              'limits': 'Synthetic local diagnostic fixtures, not recovered full auth or original server.',
              'items': [{'index': 17, 'row': 4, 'column': 3, 'kind': 2}],
              'initial_key': 12345, 'spawn_layout': args.spawn_layout,
              'item_layout': args.item_layout, 'fixtures': {}}
    fixtures = dict(FIXTURES)
    if args.host_back_rejoin or args.host_battle_exit:
        result['host_back_rejoin'] = {'mode': 'idle_A_to_B'}
        fixtures.update({
            'join_A': ('rejoin-r1/reply87-A-session1.bin', 55),
            'player_A_guest': ('rejoin-r1/A106-unready.bin', 58),
            'slot_A_guest': ('rejoin-r1/A110-unready.bin', 61),
            'guest_B_chief': ('rejoin-r1/B106-chief.bin', 58),
            'ready_A': ('rejoin-r1/A110-ready.bin', 100),
        })
    if args.host_battle_exit:
        result['battle_exit'] = {'mode': 'host_A_202_86'}
    if args.spawn_layout == 'overlap':
        fixtures['start'] = ('collision-r1/overlap121.bin', 73)
    if args.item_layout in ('hidden', 'battle'):
        fixtures['floor'] = (('hidden-item-r1/hidden582.bin' if args.item_layout == 'hidden'
                              else 'basic-battle-r1/basic582.bin'), 73)
        if args.item_layout == 'hidden':
            result['items'] = []
        result['hidden_items'] = [
            {'index': 27, 'row': 2, 'column': 3, 'kind': 0},
            {'index': 28, 'row': 3, 'column': 2, 'kind': 1},
        ]
    if args.all_dead_rank:
        candidate = args.all_dead_rank.resolve(strict=True)
        if not candidate.is_relative_to(root):
            parser.error('--all-dead-rank must be under source-root')
        raw = candidate.read_bytes()
        if len(raw) < 38 or struct.unpack_from('<I', raw, 5)[0] != 175:
            parser.error('--all-dead-rank must contain a KR175 frame')
        fixtures['rank_all_dead'] = (str(candidate.relative_to(root)), 91)
    for name, (relative, sequence) in fixtures.items():
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
    print(f'Indexed {len(fixtures)} fixtures ({args.spawn_layout} spawns): {out}')


if __name__ == '__main__':
    main()
