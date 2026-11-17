"""Đóng gói binary/config/fixture đã pin; chỉ ghi artifact riêng dưới local/."""
import argparse
import copy
import json
from pathlib import Path
import shutil
import struct

from rust_package_runtime import digest, verify

ROOT = Path(__file__).resolve().parents[1]


def build(config, binaries, out, target, source_root=None):
    if target not in {'x86_64-unknown-linux-gnu', 'aarch64-unknown-linux-gnu'}:
        raise ValueError('Target Linux GNU không được hỗ trợ')
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
    assets(value)
    admission.update(database='@STATE@/accounts.sqlite', launch_socket='@STATE@/launch.sock')
    architecture = None
    for name in ['kr-service', 'kr-admin', 'kr-companion']:
        source = Path(binaries) / name
        with source.open('rb') as stream:
            header = stream.read(20)
        if len(header) != 20 or header[:6] != b'\x7fELF\x02\x01':
            raise ValueError('Gói server yêu cầu binary ELF64 little-endian')
        machine = struct.unpack_from('<H', header, 18)[0]
        if machine not in [62, 183] or architecture not in [None, machine]:
            raise ValueError('Các binary phải cùng kiến trúc Linux được hỗ trợ')
        architecture = machine
        save(source, 'bin/' + name, mode=0o500)
    if (architecture == 62 and not target.startswith('x86_64-')) or (architecture == 183 and not target.startswith('aarch64-')):
        raise ValueError('Target khai báo không khớp ELF machine')
    template = out / 'config.template.json'
    template.write_text(json.dumps(value, ensure_ascii=False, indent=2) + '\n')
    template.chmod(0o400)
    rows[template.name] = dict(path=template.name, bytes=template.stat().st_size, sha256=digest(template), mode=0o400)
    save(Path(__file__).with_name('rust_package_runtime.py'), 'kr-package.py', mode=0o500)
    manifest = dict(schema='kre-service-package-v1', target=target, elf_machine=architecture,
                    files=[rows[key] for key in sorted(rows)],
                    limits='Validated input recipe and fixture pins, not a fresh export from pack. Requires compatible Linux ABI and Python; Internet/native/load/restore/soak acceptance remains separate.')
    path = out / 'manifest.json'
    path.write_text(json.dumps(manifest, ensure_ascii=False, indent=2) + '\n')
    path.chmod(0o400)
    result = digest(path)
    verify(out, result)
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--base', type=Path, required=True)
    parser.add_argument('--binary-dir', type=Path, required=True)
    parser.add_argument('--target', required=True)
    parser.add_argument('--out', type=Path, required=True)
    args = parser.parse_args()
    out = args.out.resolve()
    local = (ROOT / 'local').resolve()
    if out == local or not out.is_relative_to(local):
        parser.error('Cấu hình/fixture chỉ được đóng gói vào local/')
    try:
        value = json.loads(args.base.read_text())
        result = build(value, args.binary_dir, out, args.target, args.base.resolve().parent)
    except (ValueError, OSError, KeyError, TypeError) as error:
        parser.exit(1, f'Không thể đóng gói: {error}\n')
    print(json.dumps(dict(result='package_built_and_verified', manifest_sha256=result)))


if __name__ == '__main__':
    main()
