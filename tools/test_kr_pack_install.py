"""Tạo pack giả và kiểm cài gói Linux trong môi trường cô lập."""
import argparse
import json
import os
from pathlib import Path
import platform
import re
import struct
import subprocess
import sys
import tempfile

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from server import probe_codec as codec
from server.room_title import _encode_bytes as wire_bytes
from kr_pack_export import ITEMS, sha
from rust_map_profile import raw_blob
from rust_package_runtime import digest, verify
from rust_service_package import build

ROOT = Path(__file__).resolve().parents[1]
MAPS = ('SyntheticOne', 'SyntheticTwo')
HEADER = bytes.fromhex('690000000000000000ffffffff000000000200000002000000020000000000000001000000')


def fields(*values):
    return codec.pack_fields([v for v, _ in values], [size for _, size in values])


def string(value):
    raw = value.encode()
    return struct.pack('<I', len(raw)) + raw


def encrypt_index(raw):
    n = len(raw) // 4
    words = struct.unpack_from(f'<{n}I', raw)
    result = struct.pack(f'<{n}I', 0x12345678, *(a ^ b for a, b in zip(words, words[1:])))
    return result + bytes(x ^ y for x, y in zip(raw[n * 4:], struct.pack('<I', words[-1])))


def encrypt_entry(raw, key):
    n = len(raw) // 4
    prior = key
    words = []
    for i, value in enumerate(struct.unpack_from(f'<{n}I', raw)):
        words.append(value ^ prior ^ (i * i * (i + 966) * (i + 2020) & 0xffffffff) ^ 0x9c6c95ce)
        prior = value
    return struct.pack(f'<{n}I', *words) + bytes(x ^ y for x, y in zip(raw[n * 4:], struct.pack('<I', prior)))


def fake_pair(root, stem, entries):
    data = bytearray()
    index = bytearray(struct.pack('<III', 0, 0, len(entries)))
    for name, raw in entries.items():
        key = 0x12340000 + len(data)
        index.extend(string(name) + struct.pack('<IIII', len(data), len(raw), key, 0))
        data.extend(encrypt_entry(raw, key))
    (root / (stem + '.idx')).write_bytes(encrypt_index(index))
    (root / (stem + '.idd')).write_bytes(data)


def fake_game(root):
    root.mkdir()
    items = b''.join(string(ITEMS.get(i, f'Item/Synthetic{i}')) + string('Sprite/Placeholder')
                     + struct.pack('<8i', 1, 0, 0, 0, 0, 0, 0, 0) for i in range(146))
    characters = (string('C00') + struct.pack('<11i', 0, 0, 5, 7, 1, 10, 1, 7, 0, 0, 0)
                  + string('C01') + struct.pack('<11i', 1, 0, 4, 8, 2, 7, 1, 9, 0, 0, 0))
    fake_pair(root, 'FxContents', {
        'Attr/ItemAttr': struct.pack('<3I', 146, 0, 0) + items,
        'Attr/BomberAttr': characters,
        'Attr/MapGroup': struct.pack('<I', 1) + string('Synthetic') + struct.pack('<I', len(MAPS))
        + b''.join(string('Map/' + name) for name in MAPS),
    })
    raw = '<Tiles>' + ''.join(
        f'<Tile Name="{name}" Data="{struct.pack("<18i", *attrib).hex()}" />'
        for name, attrib in [('Ground', [0] * 18), ('Box', [2] + [0] * 17)]) + '</Tiles>'
    xml = raw.encode()
    fake_pair(root, 'fx', {
        'ObjectTable/ObjectTable': struct.pack('<I', 2) + string('Ground') + bytes(4) + string('Box') + bytes(4),
        'TileInfo/TileInfoList': struct.pack('<I', len(xml)) + xml,
    })
    maps, seeds = {}, {}
    for offset, name in enumerate(MAPS):
        # Deliberately different spawn, cell and map ordinal for each map.
        header = [1000, 0, 0, 15, 13, 2] + [0] * 14
        spawns = [1 + offset, 1, 13, 11]
        boxes = [int(i == (26 + offset)) for i in range(195)]
        cells = [v for box in boxes for v in (0, 1 if box else -1)]
        maps['Map/' + name] = struct.pack('<' + 'i' * (len(header) + len(spawns) + len(cells) + 7),
                                         *header, *spawns, 0, *cells, *([0] * 6))
        seeds[name] = sha(bytes(2 * box for box in boxes))
    fake_pair(root, 'FxMap', dict(reversed(list(maps.items()))))
    return {p.name: digest(p) for p in sorted(root.iterdir())}, seeds


def fake_recipe(root):
    root.mkdir()

    def pin(stem, raw):
        p = root / stem
        p.write_bytes(raw)
        return dict(path=str(p), bytes=len(raw), sha256=sha(raw))

    def packet(stem, op, body, seq=43):
        return dict(opcode=op, sequence=seq, fixture=pin(stem, codec.encode(HEADER, op, body, seq)))

    # Existing CP283 protocol template; this is not pack data.
    reference = (ROOT / 'rust/crates/kr-service/src/account_bridge/local/native/fixtures.rs').read_text()
    auth0 = bytes.fromhex(re.search(r'const AUTH: &str = "([0-9a-f]+)"', reference)[1])
    player0 = bytes.fromhex(re.search(r'const PLAYER: &str = "([0-9a-f]+)"', reference)[1])

    def account(index, name):
        name = name.encode()
        auth = bytearray(auth0)
        auth[25:29] = fields((index, 4))
        auth[31:38] = wire_bytes(name)
        zero = string(name.decode()) + bytes(98)
        own = fields((1, 1)) + raw_blob(zero)[0] + fields((0, 4), *((0, 1),) * 4)
        lobby = {alias: packet(f'{alias}-{index}', op, body) for alias, op, body in (
            ('auth_A', 4, auth), ('netinfo', 15, bytes(31)), ('flags', 62, bytes(3)),
            ('redirect', 7, bytes(14)), ('channel', 17, bytes(37)),
            ('ready', 241, raw_blob(bytes(8))[0] + fields((0, 4), (0, 4), (0, 1), (0, 4))),
            ('roomlist_A', 91, fields((0, 2))), ('channel_list', 14, bytes(44)), ('own291', 291, own))}
        player = bytearray(player0)
        player[0:5] = fields((index - 1, 1), (index, 4))
        player[12:16] = fields((2 if index == 1 else 32, 4))
        player[20:135] = raw_blob(zero)[0][4:]
        player[140:147] = wire_bytes(name)
        own_head = packet(f'player-{index}', 106, player, 58)
        return (dict(identity=index, name=name.decode(), peer=f'127.0.0.{1 + index}',
                     profile=dict(packets=lobby, own_header=dict(own_head['fixture']))), own_head)

    a, pa = account(1, 'KRLocal')
    b, pb = account(2, 'KRGuest')
    title = 'PackSmoke'
    metadata = struct.pack('<HB', 1, 0) + bytes(4) + string('KRMap') + string(MAPS[0]) + bytes(2)
    metadata += string('ThemeDefault') + string('') + b'\x01\x00' + bytes(14)
    table, crc = raw_blob(metadata)
    for actor in [a, b]:
        actor['profile']['packets']['ready'] = packet(
            f'ready-correct-{actor["identity"]}', 241,
            raw_blob(bytes(8))[0] + fields((crc, 4), (0, 4), (0, 1), (0, 4)))
    row_tail = bytes([0, 0, 255, 0, 0, 0, 0, 255, 255, 255, 255]) + bytes(9)
    start_rows = bytes(6) + row_tail + bytes([1, 0, 1, 1, 0, 1]) + row_tail
    start = (bytes(16) + struct.pack('<IH', 12345, 0) + string(MAPS[0])
             + bytes(6) + b'\x02' + bytes(4) + start_rows)
    packets = dict(player_A=pa, player_B=pb)
    for alias, op, body in (
        ('create', 86, fields((0, 1), (1, 2), (0, 4), *((0, 1),) * 4)),
        ('join', 87, fields((0, 1), (1, 1), (1, 2), *((0, 1),) * 3)),
        ('listing', 91, fields((1, 2)) + raw_blob(struct.pack('<HI', 1, 1) + string(title) + bytes(22))[0]),
        ('table', 238, table), ('map', 117, fields((0, 2), (0, 2), (0, 4))), ('floor', 582, raw_blob(bytes(6))[0]),
        ('start', 121, fields((0, 1)) + raw_blob(start)[0] + bytes(18)),
        ('go', 197, fields((12345, 4), (0, 1), (120, 4), (30, 4), (1, 1))),
        ('play', 196, fields((0, 1), (12345, 4), (0, 2))),
        ('peer', 153, bytes(97))):
        packets[alias] = packet(alias, op, body)

    # WCP is an emulator response, reconstructed from its documented branches.
    wcp = b''.join(codec.encode(HEADER, op, body, seq) for seq, (op, body) in
                   zip(range(43, 65, 3), [(op, fields((0, 1))) for op in (13, 2, 12, 21, 1)] +
                       [(43, fields((1, 1), (0, 4))), (62, fields((0, 1), (0, 1))), (719, fields((0, 1)))]))
    return dict(schema='kr-rust-diagnostic-v1', diagnostic_login_error_only=False, bind='127.0.0.1',
                allowed_peers=['127.0.0.1', a['peer'], b['peer']], ports=[3857, 3858, 3888],
                max_connections=16, idle_timeout_seconds=30, write_timeout_ms=2000,
                bootstrap=pin('bootstrap', bytes.fromhex('69000c00000003c700000000000000')),
                wcp_second=pin('wcp', wcp), login_error=packet('error', 4, fields((1, 1)))['fixture'],
                diagnostic_two_lobby=dict(
                    actors=[a, b], login_port=3857, channel_port=3888, wcp_ports=[3857, 3858, 3888],
                    login_candidate_ports=[3857, 3858], managed_rooms=dict(
                        max_rooms=2, capacity=2, player_choices=True, player_round=dict(characters=[0, 1])),
                    fixed_room=dict(title=title, packets=packets, endpoint_policy='authenticated_relay',
                                    slot_allocation='first_vacant', item_policy='report_derived_basic'),
                    account_admission=dict(profile_mode='database_zero', launch_ttl_seconds=120,
                                           max_sessions=8, patched_949x_names=True, persist_character=True, persist_matches=True,
                                           prelogin_tls=dict(bind='127.0.0.1:4888', max_connections=8,
                                                             max_unauthenticated=4, max_per_ip=2, prelogin_seconds=10,
                                                             admissions_per_ip_per_minute=10))),
                authenticated_relay=dict(bind='127.0.0.1:4889', advertise='127.0.0.1:4889',
                                         max_peers=8, route_capacity=8, queue_messages=32,
                                         send_budget=64, pending_ms=500, lease_seconds=900))


def test_bundle(root, binaries):
    pins, seeds = fake_game(root / 'game')
    config = fake_recipe(root / 'wire')
    # A pack-backed bundle must build even if an old map fixture is unavailable.
    room = config['diagnostic_two_lobby']['fixed_room']
    room['map_profile'] = dict(path='old-unavailable-profile.json', bytes=1, sha256='0' * 64)
    room['terrain'] = dict(kinds=[2] * 195)
    bundle = root / 'bundle'
    target = f'{platform.machine()}-unknown-linux-gnu'
    build(config, binaries, bundle, target, pack_scope='g1')
    template = json.loads((bundle / 'config.template.json').read_bytes())
    room = template['diagnostic_two_lobby']['fixed_room']
    assert 'map_profile' not in room and 'terrain' not in room
    # CI seals a different exporter policy with the fake pack. The production
    # runner has no environment, CLI or recipe override for pack hashes.
    source = bundle / 'kr_pack_export.py'
    patch = f'\nPACK_SHA256.clear()\nPACK_SHA256.update({pins!r})\nG1 = MAPS = {MAPS!r}\nTERRAIN_SEEDS = {seeds!r}\nfrom functools import partial\nexport = partial(export, maps=MAPS)\n'
    source.chmod(0o600)
    with source.open('a') as file:
        file.write(patch)
    source.chmod(0o400)
    manifest = bundle / 'manifest.json'
    manifest.chmod(0o600)
    value = json.loads(manifest.read_text())
    value['limits'] = 'Synthetic fixture only'
    spec = next(row for row in value['files'] if row['path'] == source.name)
    spec.update(bytes=source.stat().st_size, sha256=digest(source))
    manifest.write_text(json.dumps(value) + '\n')
    manifest.chmod(0o400)
    verify(bundle, digest(manifest))
    return bundle, digest(manifest)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binaries', type=Path, required=True)
    parser.add_argument('--image', default='python:3.12-slim-trixie')
    parser.add_argument('--bwrap', action='store_true', help='Thay Docker bằng bwrap trên máy Linux cục bộ')
    options = parser.parse_args()
    with tempfile.TemporaryDirectory(prefix='kr-pack-install-') as work:
        root = Path(work)
        bundle, expected = test_bundle(root, options.binaries.resolve())
        result = root / 'result'
        result.mkdir(mode=0o700)
        if options.bwrap:
            command = ['bwrap', '--unshare-all', '--die-with-parent', '--ro-bind', '/usr', '/usr',
                       '--symlink', 'usr/lib', '/lib', '--symlink', 'usr/lib', '/lib64',
                       '--symlink', 'usr/bin', '/bin', '--proc', '/proc', '--dev', '/dev',
                       '--tmpfs', '/tmp', '--ro-bind', str(bundle), '/opt/kr',
                       '--ro-bind', str(root / 'game'), '/game', '--bind', str(result), '/result',
                       '--setenv', 'KR_MANIFEST_SHA256', expected, '--setenv', 'KR_FAKE_PACK', '1', '--chdir', '/opt/kr', 'python3', '-']
        else:
            command = ['docker', 'run', '--rm', '-i', '--network=none', '--read-only',
                       '--user', f'{os.getuid()}:{os.getgid()}', '--tmpfs', '/tmp',
                       '--env', f'KR_MANIFEST_SHA256={expected}', '--env', 'KR_FAKE_PACK=1', '--workdir', '/opt/kr',
                       '--volume', f'{bundle}:/opt/kr:ro', '--volume', f'{root / "game"}:/game:ro',
                       '--volume', f'{result}:/result:rw', options.image, 'python3', '-']
        with (ROOT / 'tools/smoke_kr_pack_install.py').open('rb') as script:
            run = subprocess.run(command, stdin=script, timeout=120)
        if run.returncode:
            for path in [result / 'service.log', result / 'load.log',
                         *result.glob('state/runs/*/service/rust-events.jsonl')]:
                if path.exists():
                    print(path.read_text(), file=sys.stderr)
            raise SystemExit(run.returncode)


if __name__ == '__main__':
    main()
