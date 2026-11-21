#!/usr/bin/env python3
"""Kiểm và chạy gói Linux độc lập với repo; manifest hash lấy từ bản bàn giao."""
import argparse
import fcntl
import hashlib
import json
import os
from pathlib import Path, PurePosixPath
import stat
import subprocess
import time


def digest(path):
    value = hashlib.sha256()
    with Path(path).open('rb') as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b''):
            value.update(block)
    return value.hexdigest()


def member(root, name):
    relative = PurePosixPath(name)
    if (not name or relative.is_absolute() or relative.as_posix() != name
            or any(part in ['.', '..'] for part in relative.parts)):
        raise ValueError('Đường dẫn manifest không hợp lệ')
    path = root
    for part in relative.parts:
        path = path / part
        if path.is_symlink():
            raise ValueError('Gói không cho phép symlink')
    if not path.resolve().is_relative_to(root.resolve()):
        raise ValueError('Đường dẫn vượt gói')
    return path


def verify(root, expected_sha256):
    root = Path(root).resolve(strict=True)
    manifest = member(root, 'manifest.json')
    if digest(manifest) != expected_sha256:
        raise ValueError('Manifest không khớp hash bàn giao')
    data = json.loads(manifest.read_text())
    if data.get('schema') != 'kre-service-package-v1' or not data.get('files'):
        raise ValueError('Schema gói không hợp lệ')
    names = set()
    for row in data['files']:
        path = member(root, row['path'])
        if row['path'] in names or row['path'] == 'manifest.json':
            raise ValueError('Đường dẫn manifest bị lặp')
        names.add(row['path'])
        info = path.stat()
        if (not stat.S_ISREG(info.st_mode) or info.st_size != row['bytes']
                or digest(path) != row['sha256']
                or stat.S_IMODE(info.st_mode) != row['mode']):
            raise ValueError('File gói không khớp manifest')
    actual = set()
    for path in root.rglob('*'):
        if path.is_symlink():
            raise ValueError('Gói không cho phép symlink')
        if path.is_file() and path != manifest:
            actual.add(path.relative_to(root).as_posix())
    if actual != names:
        raise ValueError('Gói chứa file ngoài manifest')
    return data


def private_state(root, state):
    root = Path(root).resolve(strict=True)
    state = Path(state)
    if not state.is_absolute() or state.resolve().is_relative_to(root):
        raise ValueError('State phải ở đường dẫn tuyệt đối riêng ngoài gói')
    state.mkdir(mode=0o700, parents=True, exist_ok=True)
    if state.is_symlink() or state.stat().st_uid != os.getuid() or state.stat().st_mode & 0o077:
        raise ValueError('State phải thuộc người chạy và có quyền 0700')
    return state.resolve()


def prepare(root, state):
    state = private_state(root, state)
    socket = state / 'launch.sock'
    if max(len(os.fsencode(p)) for p in [socket, state / 'accounts.admin.sock']) > 100:
        raise ValueError('Đường dẫn socket cục bộ quá dài')
    config = json.loads((root / 'config.template.json').read_text())
    admission = config['diagnostic_two_lobby']['account_admission']
    admission.update(database=str(state / 'accounts.sqlite'), launch_socket=str(socket))
    for section, key in [('public_launcher', 'launcher.pk8'), ('prelogin_tls', 'prelogin.pk8')]:
        if admission.get(section) is not None:
            admission[section]['private_key'] = str(state / key)
    run = state / 'runs' / str(time.time_ns())
    run.mkdir(mode=0o700, parents=True)
    runtime = run / 'config.json'
    fd = os.open(runtime, os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600)
    with os.fdopen(fd, 'w') as stream:
        json.dump(config, stream, ensure_ascii=False, indent=2)
        stream.write('\n')
    return runtime, run / 'service', state / 'accounts.sqlite'


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('command', choices=['verify', 'init-db', 'restore-db', 'migrate-db', 'run'])
    parser.add_argument('--manifest-sha256', required=True)
    parser.add_argument('--state', type=Path)
    parser.add_argument('--snapshot', type=Path)
    args = parser.parse_args()
    root = Path(__file__).resolve().parent
    try:
        verify(root, args.manifest_sha256)
        if args.command == 'verify':
            print('Gói khớp manifest bàn giao.')
            return
        if args.state is None:
            raise ValueError('Cần --state riêng')
        if (args.snapshot is not None) != (args.command == 'restore-db'):
            raise ValueError('Chỉ restore-db dùng và yêu cầu --snapshot')
        state = private_state(root, args.state)
        database = state / 'accounts.sqlite'
        lock_fd = os.open(state / 'service.lock', os.O_WRONLY | os.O_CREAT | os.O_NOFOLLOW, 0o600)
        fcntl.flock(lock_fd, fcntl.LOCK_EX | fcntl.LOCK_NB)
        admin = str(root / 'bin/kr-admin')
        if args.command == 'restore-db':
            subprocess.run([admin, 'restore', str(args.snapshot.absolute()), str(database)], check=True, cwd=root)
            return
        if args.command == 'migrate-db':
            subprocess.run([admin, 'migrate', str(database)], check=True, cwd=root)
            return
        if args.command == 'init-db':
            if database.exists():
                raise ValueError('DB đã tồn tại; không ghi đè bằng init-db')
            request = dict(actor='package-init', reason='Initial package installation', command=dict(op='init'))
            subprocess.run([admin, str(database)],
                           input=json.dumps(request), text=True, check=True, cwd=root)
            return
        if not database.is_file():
            raise ValueError('Cần init-db hoặc restore DB trước khi chạy')
        config, out, _ = prepare(root, state)
        os.set_inheritable(lock_fd, True)
        os.chdir(root)
        os.execv(str(root / 'bin/kr-service'), ['kr-service', '--config', str(config), '--out', str(out)])
    except (ValueError, OSError, KeyError, subprocess.SubprocessError) as error:
        parser.exit(1, f'Không thể sử dụng gói: {error}\n')


if __name__ == '__main__':
    main()
