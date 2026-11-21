#!/usr/bin/env python3
"""Kiểm và chạy gói server độc lập với repo; hash manifest lấy từ bản bàn giao."""
import argparse
from contextlib import contextmanager
import hashlib
import json
import os
from pathlib import Path, PurePosixPath
import stat
import signal
import subprocess
import sys
import struct
import xml.etree.ElementTree as ET
import time


def digest(path):
    value = hashlib.sha256()
    with Path(path).open('rb') as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b''):
            value.update(block)
    return value.hexdigest()


def link_or_reparse(path):
    info = path.lstat()
    return (stat.S_ISLNK(info.st_mode)
            or bool(getattr(info, 'st_file_attributes', 0) & stat.FILE_ATTRIBUTE_REPARSE_POINT))


def executable(root, name, target):
    return root / 'bin' / (name + ('.exe' if target.endswith('-windows-gnu') else ''))


def member(root, name):
    relative = PurePosixPath(name)
    if (not name or ':' in name or '\\' in name or relative.is_absolute() or relative.as_posix() != name
            or any(part in ['.', '..'] for part in relative.parts)):
        raise ValueError('Đường dẫn manifest không hợp lệ')
    path = root
    for part in relative.parts:
        path = path / part
        if link_or_reparse(path):
            raise ValueError('Gói không cho phép symlink hoặc reparse point')
    if not path.resolve().is_relative_to(root.resolve()):
        raise ValueError('Đường dẫn vượt gói')
    return path


def verify(root, expected_sha256, *, check_modes=None):
    root = Path(root).resolve(strict=True)
    manifest = member(root, 'manifest.json')
    if digest(manifest) != expected_sha256:
        raise ValueError('Manifest không khớp hash bàn giao')
    data = json.loads(manifest.read_text(encoding='utf-8'))
    if data.get('schema') != 'kre-service-package-v1' or not data.get('files'):
        raise ValueError('Schema gói không hợp lệ')
    if check_modes is None:
        check_modes = os.name != 'nt'
    names = set()
    for row in data['files']:
        path = member(root, row['path'])
        if row['path'] in names or row['path'] == 'manifest.json':
            raise ValueError('Đường dẫn manifest bị lặp')
        names.add(row['path'])
        info = path.stat()
        if (not stat.S_ISREG(info.st_mode) or info.st_size != row['bytes']
                or digest(path) != row['sha256']
                or (check_modes and stat.S_IMODE(info.st_mode) != row['mode'])):
            raise ValueError('File gói không khớp manifest')
    actual = set()
    for path in root.rglob('*'):
        if link_or_reparse(path):
            raise ValueError('Gói không cho phép symlink hoặc reparse point')
        if path.is_file() and path != manifest:
            actual.add(path.relative_to(root).as_posix())
    if actual != names:
        raise ValueError('Gói chứa file ngoài manifest')
    return data


def private_state(root, state, admin=None):
    root = Path(root).resolve(strict=True)
    state = Path(state)
    if not state.is_absolute() or state.resolve().is_relative_to(root):
        raise ValueError('State phải ở đường dẫn tuyệt đối riêng ngoài gói')
    if os.name == 'nt':
        admin = admin or str(root / 'bin/kr-admin.exe')
        # Create with an explicit inheritable DACL in a trusted existing parent.
        subprocess.run([admin, 'private-dir', str(state)], check=True, stdout=subprocess.DEVNULL)
    else:
        state.mkdir(mode=0o700, parents=True, exist_ok=True)
        if state.is_symlink() or state.stat().st_uid != os.getuid() or state.stat().st_mode & 0o077:
            raise ValueError('State phải thuộc người chạy và có quyền 0700')
    return state.resolve()


@contextmanager
def package_lock(state):
    path = state / 'service.lock'
    if os.name == 'nt':
        import msvcrt
        if path.exists() or path.is_symlink():
            if link_or_reparse(path) or not path.is_file():
                raise ValueError('Khoá runner phải là file thường')
        flags = os.O_RDWR | os.O_CREAT | os.O_BINARY
    else:
        import fcntl
        flags = os.O_WRONLY | os.O_CREAT | os.O_NOFOLLOW
    fd = os.open(path, flags, 0o600)
    try:
        if os.name == 'nt':
            msvcrt.locking(fd, msvcrt.LK_NBLCK, 1)
        else:
            fcntl.flock(fd, fcntl.LOCK_EX | fcntl.LOCK_NB)
        yield fd
    finally:
        os.close(fd)


def package_tls(root, state):
    config = json.loads((root / 'config.template.json').read_text(encoding='utf-8'))
    admission = config['diagnostic_two_lobby']['account_admission']
    return {key: state / (key + '-identity') for key in ('public_launcher', 'prelogin_tls')
            if (admission.get(key) or {}).get('certificate') == '@INSTALL@'}


def init_tls(root, state, target):
    for destination in package_tls(root, state).values():
        if not destination.exists():
            subprocess.run([str(executable(root, 'kr-launch-identity', target)), str(destination)],
                           check=True, cwd=state, stdout=subprocess.DEVNULL)
        if link_or_reparse(destination) or not all((destination / name).is_file()
                                                for name in ('private.pk8', 'certificate.der')):
            raise ValueError('TLS identity chưa đầy đủ; cần provision lại trong state riêng')


def prepare(root, state, cache=None, scope=None, f1_maps=None):
    state = private_state(root, state)
    socket = state / 'launch.sock'
    if os.name != 'nt' and max(len(os.fsencode(p)) for p in [socket, state / 'accounts.admin.sock']) > 100:
        raise ValueError('Đường dẫn socket cục bộ quá dài')
    config = json.loads((root / 'config.template.json').read_text(encoding='utf-8'))
    admission = config['diagnostic_two_lobby']['account_admission']
    admission.update(database=str(state / 'accounts.sqlite'), launch_socket=str(socket))
    for section, key in [('public_launcher', 'launcher.pk8'), ('prelogin_tls', 'prelogin.pk8')]:
        if admission.get(section) is not None:
            admission[section]['private_key'] = str(state / key)
    for section, directory in package_tls(root, state).items():
        cert = directory / 'certificate.der'
        admission[section].update(private_key=str(directory / 'private.pk8'),
                                  certificate=dict(path=str(cert), bytes=cert.stat().st_size,
                                                   sha256=digest(cert)))
    run = state / 'runs' / str(time.time_ns())
    run.mkdir(mode=0o700, parents=True)
    if cache is not None:
        from kr_pack_config import configure
        from kr_pack_export import F1, G1
        wire = run / 'wire'
        wire.mkdir(mode=0o700)
        configure(config, root, cache, wire, G1 if scope == 'g1' else F1 if scope == 'f1' else f1_maps, scope)
    runtime = run / 'config.json'
    fd = os.open(runtime, os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600)
    with os.fdopen(fd, 'w', encoding='utf-8') as stream:
        json.dump(config, stream, ensure_ascii=False, separators=(',', ':'))
        stream.write('\n')
    return runtime, run / 'service', state / 'accounts.sqlite'


def main():
    # The verified package stays immutable even when owned by the runner.
    sys.dont_write_bytecode = True
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('command', choices=['verify', 'init-db', 'init-tls', 'restore-db', 'migrate-db', 'export', 'run'])
    parser.add_argument('--manifest-sha256', required=True)
    parser.add_argument('--state', type=Path)
    parser.add_argument('--snapshot', type=Path)
    parser.add_argument('--game-dir', type=Path, help='Thư mục game KR 00967, chỉ đọc')
    parser.add_argument('--f1-map', action='append', help='Tên map chính xác trong gói thử F1-a')
    args = parser.parse_args()
    root = Path(__file__).resolve().parent
    try:
        manifest = verify(root, args.manifest_sha256)
        target = manifest['target']
        if (os.name == 'nt') != target.endswith('-windows-gnu'):
            raise ValueError('Gói không khớp với hệ điều hành đang chạy')
        # Windows' embeddable Python does not include the script directory.
        # Add it only once every member of the package has been verified.
        if os.name == 'nt':
            sys.path.insert(0, str(root))
        if args.command == 'verify':
            print('Gói khớp manifest bàn giao.')
            return
        if args.state is None:
            raise ValueError('Cần --state riêng')
        if (args.snapshot is not None) != (args.command == 'restore-db'):
            raise ValueError('Chỉ restore-db dùng và yêu cầu --snapshot')
        scope = manifest.get('pack_scope')
        if scope is not None and scope not in ('g1', 'f1', 'f1-a'):
            raise ValueError('Phạm vi pack trong gói không hợp lệ')
        needs_game = scope is not None and args.command in ('export', 'run')
        if (scope == 'f1-a' and needs_game) != bool(args.f1_map):
            raise ValueError('Gói F1-a yêu cầu --f1-map khi export hoặc run')
        if needs_game != (args.game_dir is not None):
            raise ValueError('Gói từ pack yêu cầu --game-dir khi export hoặc run')
        if args.command == 'export' and scope is None:
            raise ValueError('Gói này dùng fixture; cần gói xuất từ pack')
        if args.game_dir is not None and args.state.resolve().is_relative_to(args.game_dir.resolve()):
            raise ValueError('State phải nằm ngoài thư mục game chỉ đọc')
        admin = str(executable(root, 'kr-admin', target))
        state = private_state(root, args.state, admin)
        database = state / 'accounts.sqlite'
        with package_lock(state) as lock_fd:
            if args.command == 'restore-db':
                subprocess.run([admin, 'restore', str(args.snapshot.absolute()), str(database)], check=True, cwd=root)
                return
            if args.command == 'migrate-db':
                subprocess.run([admin, 'migrate', str(database)], check=True, cwd=root)
                return
            if args.command == 'init-tls':
                init_tls(root, state, target)
                return
            if args.command == 'init-db':
                if database.exists():
                    raise ValueError('DB đã tồn tại; không ghi đè bằng init-db')
                init_tls(root, state, target)
                request = dict(actor='package-init', reason='Initial package installation', command=dict(op='init'))
                subprocess.run([admin, str(database)],
                               input=json.dumps(request), text=True, check=True, cwd=root)
                return
            cache = None
            if needs_game:
                from kr_pack_export import MAPS, export
                cache, reused = export(args.game_dir, state / 'pack-cache',
                                       maps=args.f1_map or MAPS, f1=bool(args.f1_map))
                if args.command == 'export':
                    print(json.dumps(dict(cache=str(cache), reused=reused)))
                    return
            if not database.is_file():
                raise ValueError('Cần init-db hoặc restore DB trước khi chạy')
            config, out, _ = prepare(root, state, cache, scope, args.f1_map)
            service = str(executable(root, 'kr-service', target))
            command = [service, '--config', str(config), '--out', str(out)]
            if os.name == 'nt':
                # Keep the runner lock while the child drains the same console event.
                signal.signal(signal.SIGINT, lambda *_: None)
                signal.signal(signal.SIGBREAK, lambda *_: None)
                sys.exit(subprocess.run(command, cwd=root).returncode)
            os.set_inheritable(lock_fd, True)
            os.chdir(root)
            os.execv(service, ['kr-service', *command[1:]])
    except (ValueError, OSError, KeyError, IndexError, struct.error, ET.ParseError, subprocess.SubprocessError) as error:
        parser.exit(1, f'Không thể sử dụng gói: {error}\n')


if __name__ == '__main__':
    main()
