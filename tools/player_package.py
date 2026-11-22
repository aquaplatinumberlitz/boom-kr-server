"""Dựng payload người chơi từ input đã pin, không sửa pack hoặc output L1 nguồn.

Recipe local khai base, pack_stages theo thứ tự nối và ba languages. Không tự
đổi offset .idx: một stage mới phải chứa nguyên byte pack stage trước ở đầu.
"""
import argparse
import re
import hashlib
import ipaddress
import json
from pathlib import Path, PurePosixPath
import shutil
import struct

from content_census import decrypt_idx, parse_idx

ROOT = Path(__file__).resolve().parents[1]
LANGUAGES = ('vi', 'en', 'ko')
DEFAULT_LAUNCHER_ART = ROOT / 'local/assets/launcher/launch.png'
LOCAL_EXTENSIONS = {'.ini', '.ssd', '.fft', '.bik'}
GUIDE_IMAGES = {
    'o1-create-room.png': dict(path='local/assets/guide/o1-create-room.png', bytes=225709,
                               sha256='1f0fadaf574286e27209a26173f66bce3cdca9e5d236c034826a478a8e2fcc5d'),
    'o1-join-code.png': dict(path='local/assets/guide/o1-join-code.png', bytes=163457,
                             sha256='7807840cfa64adf2b6238ee36bf05127447eb0fdb9023ff18e8bcb90e44a7d11'),
    'o1-join-lan.png': dict(path='local/assets/guide/o1-join-lan.png', bytes=160845,
                            sha256='ca4ee7d9ab98fc1764d78b9b3dccb06bb2d0f593ee4f109c1e5a47096be28d94'),
}


def digest(path):
    with Path(path).open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()


def relative(value):
    path = PurePosixPath(value)
    if (not value or path.is_absolute() or '\\' in value or ':' in value
            or any(part in {'', '.', '..'} for part in value.split('/'))):
        raise ValueError('Đường dẫn payload không hợp lệ')
    return path


def pin(source):
    source = Path(source['path']).resolve(strict=True)
    if not source.is_file():
        raise ValueError('Input phải là file thường')
    return source


def checked_pin(row):
    path = pin(row)
    if path.stat().st_size != row['bytes'] or digest(path) != row['sha256']:
        raise ValueError('Input không khớp pin')
    return path


def prefix_equal(prefix, candidate):
    with prefix.open('rb') as left, candidate.open('rb') as right:
        while block := left.read(1024 * 1024):
            if right.read(len(block)) != block:
                raise ValueError('Pack stage mới không giữ nguyên prefix trước; không được tái dùng idx')


def stage_launcher_art(source, out):
    # Pillow is needed only when a source image is present.
    from PIL import Image, ImageOps

    destination = 'payload/launcher/backdrop.jpg'
    target = out / destination
    target.parent.mkdir(mode=0o700, parents=True, exist_ok=True)
    with Image.open(source) as original:
        art = ImageOps.exif_transpose(original).convert('RGB')
        art.thumbnail((960, 720), Image.Resampling.LANCZOS)
        with target.open('xb') as stream:
            art.save(stream, 'JPEG', quality=92, subsampling=0, optimize=True)
    return dict(path=destination, bytes=target.stat().st_size, sha256=digest(target))


def build(recipe, out, launcher_art=DEFAULT_LAUNCHER_ART):
    if set(recipe['languages']) != set(LANGUAGES):
        raise ValueError('Gói phải khai đủ vi/en/ko đã vá')
    base = Path(recipe['base']).resolve(strict=True)
    files = {p.relative_to(base).as_posix(): p for p in base.rglob('*') if p.is_file()}
    if len({name.casefold() for name in files}) != len(files):
        raise ValueError('Input có tên trùng trên Windows')
    out = Path(out)
    out.mkdir(mode=0o700, parents=False, exist_ok=False)
    payload = {}

    def save(source, destination, offset=0):
        relative(destination)
        source = Path(source)
        target = out / destination
        target.parent.mkdir(mode=0o700, parents=True, exist_ok=True)
        with source.open('rb') as src, target.open('xb') as dst:
            src.seek(offset)
            shutil.copyfileobj(src, dst, length=1024 * 1024)
        row = dict(path=destination, bytes=target.stat().st_size, sha256=digest(target))
        payload[destination] = row
        return row

    art = (stage_launcher_art(launcher_art, out)
           if launcher_art is not None and launcher_art.is_file() else None)
    if art is not None:
        payload[art['path']] = art

    packs = []
    originals = {name: p for name, p in files.items() if p.suffix.lower() == '.idd'}
    if not originals:
        raise ValueError('Không tìm thấy pack gốc')
    stages = recipe.get('pack_stages', [])
    if [stage['language'] for stage in stages] not in [[], ['vi'], ['vi', 'en']]:
        raise ValueError('Thứ tự pack phải là KR → vi → en')
    stage_roots = {stage['language']: Path(stage['root']).resolve(strict=True) for stage in stages}
    for name, original in sorted(originals.items()):
        current = original
        parts = [save(original, 'payload/base/' + name)]
        for stage in stages:
            candidate = stage_roots[stage['language']] / name
            if not candidate.exists():
                continue
            prefix_equal(current, candidate)
            if candidate.stat().st_size > current.stat().st_size:
                parts.append(save(candidate, 'payload/tails/' + stage['language'] + '/' + name + '.tail',
                                  current.stat().st_size))
            current = candidate
        packs.append(dict(path=name, parts=parts, bytes=current.stat().st_size, sha256=digest(current)))

    common, localized = [], {}
    for name, source in sorted(files.items()):
        relative(name)
        if source.suffix.lower() in {'.idd', '.idx'} or name.casefold() == 'ca.exe':
            continue
        if source.suffix.lower() in LOCAL_EXTENSIONS:
            localized[name] = source
        else:
            common.append(dict(path=name, payload=save(source, 'payload/common/' + name)))
    languages = {}
    for language in LANGUAGES:
        entry = recipe['languages'][language]
        if entry.get('encoding') != '949x':
            raise ValueError('Mọi client, kể cả ko, phải là bản vá 949x')
        executable = checked_pin(entry['executable'])
        overrides = dict(localized)
        for root_name in ['data', 'fonts']:
            asset_root = Path(entry[root_name]).resolve(strict=True)
            for source in asset_root.rglob('*'):
                if source.is_file() and source.suffix.lower() in LOCAL_EXTENSIONS:
                    overrides[source.relative_to(asset_root).as_posix()] = source
        overrides['nm.bik'] = checked_pin(entry['video'])
        overrides['CA.exe'] = executable
        index_stage = entry['index_stage']
        if index_stage != 'base' and index_stage not in stage_roots:
            raise ValueError('Index stage chưa có pack')
        for pack in packs:
            name = str(PurePosixPath(pack['path']).with_suffix('.idx'))
            source = base / name
            # A stage may replace only some packs; inherit earlier indices.
            if index_stage != 'base':
                for stage in stages:
                    candidate = stage_roots[stage['language']] / name
                    if candidate.exists():
                        source = candidate
                    if stage['language'] == index_stage:
                        break
            for _, offset, size, _ in parse_idx(decrypt_idx(source.read_bytes())):
                if offset + size > pack['bytes']:
                    raise ValueError('Idx trỏ ra ngoài pack chung')
            overrides[name] = source
        rows = []
        for name, source in sorted(overrides.items()):
            relative(name)
            rows.append(dict(path=name, payload=save(source, 'payload/languages/' + language + '/' + name)))
        languages[language] = dict(encoding='949x', images=entry['images'], files=rows)
    guides = {
        f'player/guide.{language}.md': save(ROOT / f'docs/player/guide.{language}.md',
                                           f'player/guide.{language}.md')
        for language in LANGUAGES
    }
    guide_images = {}
    for name, expected in GUIDE_IMAGES.items():
        source = ROOT / expected['path']
        if not source.is_file():
            continue
        if source.stat().st_size != expected['bytes'] or digest(source) != expected['sha256']:
            raise ValueError(f'Ảnh hướng dẫn không khớp pin: {name}')
        guide_images[name] = save(source, f'player/{name}')
    manifest = dict(schema='kre-player-package-v1', packs=packs, common=common, languages=languages,
                    limits='Payload được dựng và kiểm hash; chưa chứng nhận cài sạch, client song song hoặc native Internet.')
    if guide_images:
        manifest['guide_images'] = guide_images
    if art is not None:
        manifest['launcher_art'] = art
    path = out / 'manifest.json'
    path.write_text(json.dumps(manifest, ensure_ascii=False, indent=2) + '\n')
    return dict(manifest_sha256=digest(path), payload_bytes=sum(row['bytes'] for row in payload.values()),
                languages=list(languages), packs=len(packs), guides=guides, guide_images=guide_images)


def windows_runtime(recipe, output, manifest_sha256):
    """Stage only pinned first-party programs, public certificates and launch settings."""
    required = {'launcher', 'companion', 'ports'}
    online = {'certificate', 'prelogin_certificate', 'server', 'prelogin_server'}
    optional = {'server_name', 'release_url', 'friends', 'friends_directory'}
    if not required <= recipe.keys() <= required | optional | online or (recipe.keys() & online not in (set(), online)):
        raise ValueError('Recipe Windows có trường thiếu hoặc không hỗ trợ')
    if not online <= recipe.keys() and 'friends' not in recipe:
        raise ValueError('Cần gói O1 hoặc cấu hình máy chủ')
    release_url = recipe.get('release_url', '')
    if not isinstance(release_url, str):
        raise ValueError('Link phát hành phải là chuỗi')
    release_url = release_url.strip()
    if release_url and (len(release_url) > 2048 or not re.fullmatch(
            r"https://drive\.google\.com/[a-zA-Z0-9\-._~/%?&=+#:@!$',;()*]+", release_url)):
        raise ValueError('Link phát hành phải là HTTPS Google Drive')
    server_name = recipe.get('server_name', '')
    if (not isinstance(server_name, str) or len(server_name) > 64
            or any(not ch.isprintable() for ch in server_name)):
        raise ValueError('Tên máy chủ không hợp lệ')
    if digest(output / 'manifest.json') != manifest_sha256:
        raise ValueError('Manifest game không khớp pin')
    for key in ('server', 'prelogin_server') if online <= recipe.keys() else ():
        host, port = recipe[key].rsplit(':', 1)
        ip = ipaddress.IPv4Address(host)
        if ip.is_unspecified or ip.is_multicast or int(ip) == 0xffffffff or not 1 <= int(port) <= 65535:
            raise ValueError('Endpoint launcher không hợp lệ')
    ports = recipe['ports']
    if (not isinstance(ports, list) or not 1 <= len(ports) <= 32
            or any(type(port) is not int or not 1 <= port <= 65535 for port in ports)
            or len(set(ports)) != len(ports)):
        raise ValueError('Cổng loopback không hợp lệ')

    def windows_exe(path):
        with path.open('rb') as f:
            header = f.read(64)
            if len(header) != 64 or header[:2] != b'MZ':
                raise ValueError('Binary không phải PE x86-64')
            offset, = struct.unpack_from('<I', header, 60)
            f.seek(offset)
            if f.read(6) != b'PE\0\0\x64\x86':
                raise ValueError('Binary không phải PE x86-64')

    files = {
        'kr-player.exe': checked_pin(recipe['launcher']),
        'kr-prelogin.exe': checked_pin(recipe['companion']),
        'Tao-loi-tat.ps1': ROOT / 'tools/player_shortcut.ps1',
    }
    if online <= recipe.keys():
        files.update({'account.der': checked_pin(recipe['certificate']),
                      'prelogin.der': checked_pin(recipe['prelogin_certificate'])})
    if 'friends' in recipe:
        files['Cai-dat-Windows.ps1'] = ROOT / 'tools/player_windows_setup.ps1'
    for name, source in files.items():
        if name.endswith('.exe'):
            windows_exe(source)
        if name.endswith('.der') and not 0 < source.stat().st_size <= 8192:
            raise ValueError('Chứng thư công khai vượt giới hạn')
    for name, source in files.items():
        with source.open('rb') as src, (output / name).open('xb') as dst:
            shutil.copyfileobj(src, dst)
    config = dict(ports=ports, companion='kr-prelogin.exe', bundle='.', manifest_sha256=manifest_sha256,
                  installation='game', state='state', udp_backend='portable')
    if online <= recipe.keys():
        config.update(server=recipe['server'], prelogin_server=recipe['prelogin_server'],
                      certificate='account.der', prelogin_certificate='prelogin.der')
    friends_files = {}
    if 'friends' in recipe:
        spec = recipe['friends']
        if set(spec) != {'package', 'manifest_sha256', 'python_files'}:
            raise ValueError('Recipe O1 yêu cầu gói server và Python embeddable đã pin')
        from rust_package_runtime import friends_pack, verify
        from kr_pack_export import PACK_SHA256
        friends_pack(output, manifest_sha256, PACK_SHA256)
        package = Path(spec['package']).resolve()
        manifest = verify(package, spec['manifest_sha256'])
        if manifest['target'] != 'x86_64-pc-windows-gnu' or manifest.get('pack_scope') not in ('g1', 'f1'):
            raise ValueError('Cần gói server Windows xuất dữ liệu từ pack')
        template = json.loads((package / 'config.template.json').read_bytes())
        two = template['diagnostic_two_lobby']
        expected_ports = set(range(3808, 4000, 10))
        if (set(ports) != expected_ports or set(template['ports']) != expected_ports
                or set(two['wcp_ports']) != expected_ports
                or not expected_ports <= set(two['login_candidate_ports'])
                or (two['login_port'], two['channel_port']) != (3838, 3858)):
            raise ValueError('O1 yêu cầu profile cổng đã đo với client KR 00967, không dùng profile kr-load')
        runtime = {f"friends-server/{row['path']}": package / row['path'] for row in manifest['files']}
        runtime['friends-server/manifest.json'] = package / 'manifest.json'
        for name, row in spec['python_files'].items():
            if not re.fullmatch(r'[a-zA-Z0-9_.-]+', name) or name in ('.', '..'):
                raise ValueError('Tên file Python embeddable không hợp lệ')
            runtime[f'friends-python/{name}'] = checked_pin(row)
        windows_exe(runtime['friends-python/python.exe'])
        for name, source in runtime.items():
            destination = output / name
            destination.parent.mkdir(exist_ok=True, parents=True)
            with source.open('rb') as src, destination.open('xb') as dst:
                shutil.copyfileobj(src, dst)
            friends_files[name] = digest(destination)
        config['friends'] = dict(package='friends-server', manifest_sha256=spec['manifest_sha256'],
                                 python='friends-python/python.exe')
        directory = recipe.get('friends_directory', ['wss://relay.damus.io', 'wss://nostr.mom'])
        if (not isinstance(directory, list) or len(directory) > 4
                or any(not isinstance(url, str) or not re.fullmatch(r'wss://[a-zA-Z0-9./:-]{1,250}', url) for url in directory)):
            raise ValueError('Danh sách relay tra cứu O1 không hợp lệ')
        config['friends_directory'] = directory
    if server_name.strip():
        config['server_name'] = server_name.strip()
    if release_url:
        config['release_url'] = release_url
    with (output / 'launcher.json').open('x') as file:
        json.dump(config, file, indent=2)
        file.write('\n')
    (output / 'MO-GAME.txt').write_text(
        'Giải nén vào thư mục có quyền ghi trên ổ NTFS. Mở kr-player.exe.\n'
        'Có thể chạy Tao-loi-tat.ps1 sau khi giải nén để tạo lối tắt Boom Play.\n'
        + ('Chạy Cai-dat-Windows.ps1 để thêm quy tắc LAN nếu có quyền quản trị.\n'
           'Mở Chơi với bạn bè, nhập tên, tạo hoặc vào phòng theo Hướng dẫn.\n'
           if 'friends' in recipe else
           'Chọn ngôn ngữ, chọn một máy chủ, bấm CHƠI, rồi đăng nhập hoặc chơi thử trong game.\n')
        +
        'Đừng mở CA.exe trực tiếp: game cần đường TLS do launcher mở.\n',
        encoding='utf-8-sig',
    )
    return friends_files | {
        name: digest(output / name)
        for name in (*files, 'launcher.json', 'MO-GAME.txt')
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--recipe', type=Path, required=True)
    parser.add_argument('--out', type=Path, required=True)
    parser.add_argument('--windows-runtime', type=Path, help='Recipe local chứa launcher, companion và certificate đã pin')
    parser.add_argument('--launcher-art', type=Path, default=DEFAULT_LAUNCHER_ART,
                        help='Ảnh nền launcher (mặc định: local/assets/launcher/launch.png)')
    args = parser.parse_args()
    local = (ROOT / 'local').resolve(strict=True)
    output = args.out.resolve()
    if output == local or not output.is_relative_to(local):
        parser.error('Gói và tài sản client phải ở local/')
    summary = build(json.loads(args.recipe.read_text()), output, args.launcher_art)
    if args.windows_runtime:
        summary['windows_runtime'] = windows_runtime(
            json.loads(args.windows_runtime.read_text()), output, summary['manifest_sha256'])
    print(json.dumps(summary))


if __name__ == '__main__':
    main()
