"""Dựng payload người chơi từ input đã pin, không sửa pack hoặc output L1 nguồn.

Recipe local khai base, pack_stages theo thứ tự nối và ba languages. Không tự
đổi offset .idx: một stage mới phải chứa nguyên byte pack stage trước ở đầu.
"""
import argparse
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
    manifest = dict(schema='kre-player-package-v1', packs=packs, common=common, languages=languages,
                    limits='Payload được dựng và kiểm hash; chưa chứng nhận cài sạch, client song song hoặc native Internet.')
    if art is not None:
        manifest['launcher_art'] = art
    path = out / 'manifest.json'
    path.write_text(json.dumps(manifest, ensure_ascii=False, indent=2) + '\n')
    return dict(manifest_sha256=digest(path), payload_bytes=sum(row['bytes'] for row in payload.values()),
                languages=list(languages), packs=len(packs))


def windows_runtime(recipe, output, manifest_sha256):
    """Stage only pinned first-party programs, public certificates and launch settings."""
    if set(recipe) != {'launcher', 'companion', 'certificate', 'prelogin_certificate',
                       'server', 'prelogin_server', 'ports'}:
        raise ValueError('Recipe Windows có trường thiếu hoặc không hỗ trợ')
    if digest(output / 'manifest.json') != manifest_sha256:
        raise ValueError('Manifest game không khớp pin')
    for key in ('server', 'prelogin_server'):
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
        'account.der': checked_pin(recipe['certificate']),
        'prelogin.der': checked_pin(recipe['prelogin_certificate']),
        'Tao-loi-tat.ps1': ROOT / 'tools/player_shortcut.ps1',
    }
    for name, source in files.items():
        if name.endswith('.exe'):
            windows_exe(source)
        if name.endswith('.der') and not 0 < source.stat().st_size <= 8192:
            raise ValueError('Chứng thư công khai vượt giới hạn')
    for name, source in files.items():
        with source.open('rb') as src, (output / name).open('xb') as dst:
            shutil.copyfileobj(src, dst)
    config = dict(server=recipe['server'], prelogin_server=recipe['prelogin_server'],
                  ports=ports, certificate='account.der', prelogin_certificate='prelogin.der',
                  companion='kr-prelogin.exe', bundle='.', manifest_sha256=manifest_sha256,
                  installation='game', state='state', udp_backend='portable')
    with (output / 'launcher.json').open('x') as file:
        json.dump(config, file, indent=2)
        file.write('\n')
    (output / 'MO-GAME.txt').write_text(
        'Giải nén vào thư mục có quyền ghi trên ổ NTFS. Mở kr-player.exe.\n'
        'Có thể chạy Tao-loi-tat.ps1 sau khi giải nén để tạo lối tắt Boom Play.\n'
        'Chọn ngôn ngữ, chọn Online, bấm CHƠI, rồi đăng nhập hoặc chơi thử trong game.\n'
        'Đừng mở CA.exe trực tiếp: game cần đường TLS do launcher mở.\n',
        encoding='utf-8-sig',
    )
    return {
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
