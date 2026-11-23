"""Đóng gói server kèm dữ liệu JSON đã xuất sẵn; --game-dir ở máy cài chỉ để xuất lại."""
import argparse
import copy
import json
from pathlib import Path
import shutil
import struct
import tempfile
import zipfile

from rust_package_runtime import digest, executable, verify

ROOT = Path(__file__).resolve().parents[1]
# python-3.13.9-embed-amd64.zip from python.org, same runtime as CP388.
WINDOWS_PYTHON_SHA256 = '91d828c2da3a029b41699e918674a0cb379c02cf20dab9c501306885f837402a'
DEPLOY_FILES = ('Dockerfile', 'compose.yaml', 'install.sh', 'boom-server.service', 'Dockerfile.dockerignore')
# Bản export mặc định đã commit theo docs/evidence/owner-directives/pack-data-rights.json.
PACK_DATA = ROOT / 'server' / 'pack-data'


def build(config, binaries, out, target, source_root=None, *, pack_scope=None,
          external_tls=False, self_host=False, windows_python=None, pack_data=None):
    windows = target == 'x86_64-pc-windows-gnu'
    if not windows and target not in {'x86_64-unknown-linux-gnu', 'aarch64-unknown-linux-gnu'}:
        raise ValueError('Target Linux GNU hoặc Windows GNU không được hỗ trợ')
    if self_host and (pack_scope is None or external_tls or windows != (windows_python is not None)):
        raise ValueError('S1 yêu cầu gói từ pack, cấp TLS tại máy cài và Python kèm theo trên Windows')
    if not self_host and windows_python is not None:
        raise ValueError('Chỉ gói S1 dùng Python kèm theo')
    if windows_python is not None and digest(windows_python) != WINDOWS_PYTHON_SHA256:
        raise ValueError('Python nhúng không khớp SHA-256 đã pin')
    if windows and pack_scope is None:
        raise ValueError('Gói Windows yêu cầu dữ liệu từ pack KR 00967')
    if pack_data is not None and pack_scope not in ('g1', 'f1'):
        raise ValueError('Dữ liệu JSON đóng sẵn chỉ dùng với phạm vi g1 hoặc f1')
    if self_host and pack_data is None:
        raise ValueError('Gói S1 yêu cầu dữ liệu JSON đóng sẵn; người dùng cuối không chọn thư mục game')
    if pack_data is not None:
        from kr_pack_export import prebuilt
        pack_data = prebuilt(pack_data)
    source_root = Path.cwd() if source_root is None else Path(source_root)
    out = Path(out)
    out.mkdir(mode=0o700, parents=False, exist_ok=False)
    (out / 'data').mkdir(mode=0o700)
    (out / 'bin').mkdir(mode=0o700)
    rows = {}

    def save(source, relative, expected=None, mode=0o400):
        source = Path(source)
        if not source.is_file():
            raise ValueError('Nguồn gói phải là file thường')
        before = dict(bytes=source.stat().st_size, sha256=digest(source))
        if expected is not None and before != expected:
            raise ValueError('Fixture nguồn không khớp pin')
        destination = out / relative
        if not destination.exists():
            shutil.copyfile(source, destination)
            destination.chmod(mode)
        if destination.stat().st_size != before['bytes'] or digest(destination) != before['sha256']:
            raise ValueError('File thay đổi trong khi đóng gói')
        rows[relative] = dict(path=relative, mode=mode, **before)
        return relative

    def assets(value):
        if isinstance(value, dict):
            if {'path', 'bytes', 'sha256'} <= value.keys():
                if (type(value['bytes']) is not int or not 0 <= value['bytes'] <= 65536
                        or len(value['sha256']) != 64
                        or any(char not in '0123456789abcdef' for char in value['sha256'])):
                    raise ValueError('Pin fixture không hợp lệ')
                source = Path(value['path'])
                if not source.is_absolute():
                    source = source_root / source
                suffix = '.json' if source.suffix == '.json' else '.bin'
                relative = 'data/' + value['sha256'] + suffix
                save(source, relative, dict(bytes=value['bytes'], sha256=value['sha256']))
                value['path'] = relative
            for child in value.values():
                assets(child)
        elif isinstance(value, list):
            for child in value:
                assets(child)

    value = copy.deepcopy(config)
    admission = value['diagnostic_two_lobby']['account_admission']
    if (admission.get('profile_mode') != 'database_zero'
            or not value.get('authenticated_relay')
            or value['diagnostic_two_lobby']['fixed_room'].get('endpoint_policy') != 'authenticated_relay'):
        raise ValueError('Gói yêu cầu account database và relay xác thực hiện hành')
    if self_host and not admission.get('prelogin_tls'):
        raise ValueError('Gói S1 yêu cầu prelogin_tls')
    if pack_scope is not None:
        if pack_scope not in ('g1', 'f1', 'f1-a'):
            raise ValueError('Phạm vi pack phải là g1, f1 hoặc f1-a')
        two = value['diagnostic_two_lobby']
        managed = two['managed_rooms']
        room = two['fixed_room']
        if not managed.get('player_round') or not managed.get('player_choices') or room.get('battle'):
            raise ValueError('Xuất pack yêu cầu managed player round có lựa chọn và không dùng legacy Battle')
        for key in ('terrain', 'bomb_cancellation', 'map_profile'):
            room.pop(key, None)
        room.setdefault('map_choices', [])
        for choice in room['map_choices']:
            for key in ('terrain', 'bomb_cancellation', 'map_profile'):
                choice.pop(key, None)
        managed['player_round'].pop('pack_stats', None)
        if not external_tls:
            for section in ('public_launcher', 'prelogin_tls'):
                if admission.get(section) is not None:
                    admission[section]['certificate'] = '@INSTALL@'
    assets(value)
    admission.update(database='@STATE@/accounts.sqlite', launch_socket='@STATE@/launch.sock')
    # Public TLS pins are assets; matching private keys are provisioned separately.
    for section, key in [('public_launcher', 'launcher.pk8'), ('prelogin_tls', 'prelogin.pk8')]:
        if admission.get(section) is not None:
            admission[section]['private_key'] = '@STATE@/' + key
            admission[section].setdefault('session_ttl_seconds', 28_800)
            admission[section].setdefault('session_idle_seconds', 1_800)
    architecture = None
    binary_names = ['kr-service', 'kr-admin']
    if not windows:
        binary_names += ['kr-companion']
    if pack_scope is not None:
        binary_names += ['kr-launch-identity']
        if not windows:
            binary_names += ['kr-load']
    if self_host:
        binary_names += ['boom-server']
    for name in binary_names:
        relative = executable(Path(), name, target)
        source = Path(binaries) / relative.name
        with source.open('rb') as stream:
            if windows:
                dos = stream.read(64)
                if len(dos) != 64 or dos[:2] != b'MZ':
                    raise ValueError('Gói Windows yêu cầu binary PE x86-64')
                stream.seek(struct.unpack_from('<I', dos, 60)[0])
                # PE signature, IMAGE_FILE_MACHINE_AMD64, and PE32+ magic.
                header = stream.read(26)
                if (len(header) != 26 or header[:6] != b'PE\0\0\x64\x86'
                        or header[24:] != b'\x0b\x02'):
                    raise ValueError('Gói Windows yêu cầu binary PE x86-64')
            else:
                header = stream.read(20)
                if len(header) != 20 or header[:6] != b'\x7fELF\x02\x01':
                    raise ValueError('Gói server yêu cầu binary ELF64 little-endian')
                machine = struct.unpack_from('<H', header, 18)[0]
                if machine not in [62, 183] or architecture not in [None, machine]:
                    raise ValueError('Các binary phải cùng kiến trúc Linux được hỗ trợ')
                architecture = machine
        dest = 'BoomServer.exe' if windows and name == 'boom-server' else relative.as_posix()
        save(source, dest, mode=0o500)
    if (architecture == 62 and not target.startswith('x86_64-')) or (architecture == 183 and not target.startswith('aarch64-')):
        raise ValueError('Target khai báo không khớp ELF machine')
    template = out / 'config.template.json'
    template.write_text(json.dumps(value, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')
    template.chmod(0o400)
    rows[template.name] = dict(path=template.name, bytes=template.stat().st_size, sha256=digest(template), mode=0o400)
    save(Path(__file__).with_name('rust_package_runtime.py'), 'kr-package.py', mode=0o500)
    manifest = dict(schema='kre-service-package-v1', target=target,
                    files=[rows[key] for key in sorted(rows)],
                    limits='Validated input recipe and fixture pins, not a fresh export from pack. Requires compatible Linux ABI and Python; Internet/native/load/restore/soak acceptance remains separate.')
    manifest.update({'pe_machine': 0x8664} if windows else {'elf_machine': architecture})
    if pack_scope is not None:
        support = ('kr_pack_export.py', 'kr_pack_config.py', 'content_census.py',
                   'export_map_profile.py', 'rust_map_profile.py')
        for name in support:
            save(ROOT / 'tools' / name, name)
        (out / 'server').mkdir(mode=0o700)
        for name in ('probe_codec.py', 'room_title.py'):
            save(ROOT / 'server' / name, 'server/' + name)
        manifest.update(pack_scope=pack_scope, files=[rows[key] for key in sorted(rows)],
                        limits='Pack KR 00967 được kiểm và xuất ở máy cài. Bundle chỉ giữ mẫu wire và cấu hình vận hành.')
    if pack_data is not None:
        # Chỉ JSON map/vật phẩm/chỉ số nhân vật; không chứa .idx/.idd/.bik/.exe/.dll của client.
        (out / 'pack-data').mkdir(mode=0o700)
        for source in sorted(pack_data.iterdir()):
            if source.suffix != '.json':
                raise ValueError('Dữ liệu đóng sẵn chỉ được chứa file JSON')
            save(source, 'pack-data/' + source.name)
        export_manifest = json.loads((pack_data / 'manifest.json').read_bytes())
        manifest.update(pack_data='pack-data', pack_data_identity=export_manifest['identity'],
                        files=[rows[key] for key in sorted(rows)],
                        limits='Kèm dữ liệu JSON xuất sẵn từ pack KR 00967 (pack-data/, kiểm hash hai lớp). '
                               'Không kèm file client; --game-dir chỉ để xuất lại khi dev/build.')
    if self_host:
        if windows:
            (out / 'python').mkdir(mode=0o700)
            with zipfile.ZipFile(windows_python) as archive, tempfile.TemporaryDirectory() as directory:
                for item in archive.infolist():
                    if item.is_dir() or not item.filename.isascii() or not all(
                            c.isalnum() or c in '.-_' for c in item.filename):
                        raise ValueError('Tên file Python nhúng không hợp lệ')
                    source = Path(directory) / item.filename
                    with archive.open(item) as origin, source.open('xb') as target_file:
                        shutil.copyfileobj(origin, target_file)
                    save(source, 'python/' + item.filename, mode=0o500 if item.filename.endswith('.exe') else 0o400)
        else:
            (out / 'deploy').mkdir(mode=0o700)
            for name in DEPLOY_FILES:
                save(ROOT / 'server' / 'self-host' / name, 'deploy/' + name,
                     mode=0o500 if name == 'install.sh' else 0o400)
        (out / 'player' / 'images').mkdir(mode=0o700, parents=True)
        for name in ('guide.vi.md', 'guide.en.md', 'guide.ko.md', 'images/boomserver-s1.png'):
            save(ROOT / 'docs' / 'player' / name, 'player/' + name)
        manifest.update(self_host=True, files=[rows[key] for key in sorted(rows)])
    path = out / 'manifest.json'
    path.write_text(json.dumps(manifest, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')
    path.chmod(0o400)
    result = digest(path)
    if self_host:
        sidecar = out / 'manifest.sha256'
        sidecar.write_text(result + '\n', encoding='ascii')
        sidecar.chmod(0o400)
    verify(out, result)
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--base', type=Path, required=True)
    parser.add_argument('--binary-dir', type=Path, required=True)
    parser.add_argument('--target', required=True)
    parser.add_argument('--out', type=Path, required=True)
    parser.add_argument('--pack-scope', choices=['g1', 'f1', 'f1-a'], default='g1')
    parser.add_argument('--external-tls', action='store_true',
                        help='Giữ pin TLS công khai hiện có; cấp private key qua state')
    parser.add_argument('--legacy-fixtures', action='store_true',
                        help='Gói cũ từ fixture đã pin, dành cho tái lập/rollback')
    parser.add_argument('--self-host', action='store_true', help='Bao gồm BoomServer và các file triển khai S1')
    parser.add_argument('--windows-python', type=Path, help='ZIP Python nhúng x64 đã pin của gói Windows S1')
    parser.add_argument('--archive', action='store_true', help='Tạo ZIP hoặc tar.gz bên cạnh thư mục gói S1')
    parser.add_argument('--pack-data', type=Path, default=PACK_DATA,
                        help='Thư mục JSON xuất sẵn (mặc định server/pack-data); tạo lại bằng kr_pack_export.py --prebuilt-out')
    parser.add_argument('--no-pack-data', action='store_true',
                        help='Không kèm JSON xuất sẵn; máy cài phải dùng --game-dir (không dùng cho S1)')
    args = parser.parse_args()
    if args.archive and not args.self_host:
        parser.error('--archive yêu cầu --self-host')
    out = args.out.resolve()
    local = (ROOT / 'local').resolve()
    if out == local or not out.is_relative_to(local):
        parser.error('Cấu hình/fixture chỉ được đóng gói vào local/')
    try:
        value = json.loads(args.base.read_text(encoding='utf-8'))
        result = build(value, args.binary_dir, out, args.target, args.base.resolve().parent,
                       pack_scope=None if args.legacy_fixtures else args.pack_scope, external_tls=args.external_tls,
                       self_host=args.self_host, windows_python=args.windows_python,
                       pack_data=None if args.no_pack_data or args.legacy_fixtures or args.pack_scope == 'f1-a'
                       else args.pack_data)
        if args.archive:
            shutil.make_archive(str(out), 'zip' if args.windows_python else 'gztar',
                                root_dir=out.parent, base_dir=out.name)
    except (ValueError, OSError, KeyError, TypeError) as error:
        parser.exit(1, f'Không thể đóng gói: {error}\n')
    print(json.dumps(dict(result='package_built_and_verified', manifest_sha256=result)))


if __name__ == '__main__':
    main()
