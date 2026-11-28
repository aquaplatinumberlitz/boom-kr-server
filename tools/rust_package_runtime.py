#!/usr/bin/env python3
"""Kiểm và chạy gói server độc lập với repo; hash manifest lấy từ bản bàn giao."""
import argparse
from contextlib import contextmanager
import hashlib
import ipaddress
import json
import os
from pathlib import Path, PurePosixPath
import stat
import signal
import socket
import subprocess
import sys
import struct
import xml.etree.ElementTree as ET
import time


_OPT_IN_ADAPTERS = {'f2-optin': 'f2', 'f3-optin': 'f3'}


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
    if data.get('self_host') is True:
        sidecar = member(root, 'manifest.sha256')
        if sidecar.read_text(encoding='ascii').strip() != expected_sha256:
            raise ValueError('Mã SHA-256 của gói S1 không khớp bàn giao')
        actual.remove('manifest.sha256')
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


def configure_friends(config):
    two = config['diagnostic_two_lobby']
    admission = two['account_admission']
    tls = admission.get('prelogin_tls') or {}
    if tls.get('certificate') != '@INSTALL@' or not admission.get('patched_949x_names'):
        raise ValueError('O1 yêu cầu TLS tự sinh và client 949x')
    config.update(bind='127.0.0.1', allowed_peers=['127.0.0.1'], network_scope='private_lab')
    config.pop('udp_relay', None)
    for actor in two['actors']:
        actor['peer'] = '127.0.0.1'
        actor.pop('reported_login_ipv4', None)
    for key in ('public_launcher', 'backup', 'diagnostic_identity_strings'):
        admission.pop(key, None)
    with (socket.socket(socket.AF_INET, socket.SOCK_DGRAM) as login,
          socket.socket(socket.AF_INET, socket.SOCK_DGRAM) as peer):
        login.bind(('127.0.0.1', 0))
        peer.bind(('127.0.0.1', 0))
        tls.update(bind=f'127.0.0.1:{login.getsockname()[1]}', friends_guest_only=True,
                   prelogin_seconds=180, admissions_per_ip_per_minute=60,
                   max_connections=8, max_unauthenticated=8, max_per_ip=8)
        relay = f'127.0.0.1:{peer.getsockname()[1]}'
        config['authenticated_relay'].update(bind=relay, advertise=relay)


def _concrete_ipv4(host):
    ip = ipaddress.IPv4Address(host)
    if ip.is_unspecified or ip.is_multicast or str(ip) == '255.255.255.255':
        raise ValueError('Địa chỉ IPv4 không hợp lệ')
    return ip


def _validated_policy_path(value, label):
    """Return a resolved JSON-object policy path after fail-closed checks."""
    candidate = Path(value)
    absolute = candidate if candidate.is_absolute() else Path.cwd() / candidate
    current = Path(absolute.anchor)
    try:
        for part in absolute.parts[1:]:
            current /= part
            info = current.lstat()
            if stat.S_ISLNK(info.st_mode) or bool(
                    getattr(info, 'st_file_attributes', 0) & stat.FILE_ATTRIBUTE_REPARSE_POINT):
                raise ValueError(f'{label} không được là symlink hoặc nằm trong parent symlink')
        info = absolute.stat()
        if not stat.S_ISREG(info.st_mode):
            raise ValueError(f'{label} phải là file thường')
        with absolute.open('r', encoding='utf-8') as stream:
            policy = json.load(stream)
        if not isinstance(policy, dict):
            raise ValueError(f'{label} phải là JSON object')
    except (OSError, UnicodeDecodeError, json.JSONDecodeError) as error:
        raise ValueError(f'{label} không hợp lệ') from error
    return absolute.resolve(strict=True)


def configure_economy(config, *, friends=False, s1=False, economy_profile=None,
                      emit_lucci_135=False, project_own291_level=False,
                      web_shop_bind=None, unlock_all=False,
                      f4_extended_policy=None, f3_pve_policy=None, f6_events_policy=None,
                      f5_social_policy=None, f7_custom_map_policy=None):
    """Apply opt-in F4 economy, S->C 135/291, unlock_all, and embedded web shop settings."""
    if (economy_profile is None and not emit_lucci_135 and not project_own291_level
            and web_shop_bind is None and not unlock_all and f4_extended_policy is None
            and f3_pve_policy is None and f6_events_policy is None
            and f5_social_policy is None and f7_custom_map_policy is None):
        return
    admission = config['diagnostic_two_lobby']['account_admission']
    if economy_profile is not None:
        profile = Path(economy_profile)
        if not os.path.lexists(profile) or link_or_reparse(profile) or not profile.is_file():
            raise ValueError('Hồ sơ kinh tế F4 không hợp lệ')
        admission['economy_profile_path'] = str(profile.resolve())
        admission['persist_matches'] = True
        if friends:
            admission['unlock_all_catalog'] = True
    for value, key, label in (
            (f4_extended_policy, 'f4_extended_policy', 'Hồ sơ policy F4 extended'),
            (f3_pve_policy, 'f3_pve_policy', 'Hồ sơ policy F3 PvE'),
            (f6_events_policy, 'f6_events_policy', 'Hồ sơ policy F6 events'),
            (f5_social_policy, 'f5_social_policy', 'Hồ sơ policy F5 social'),
            (f7_custom_map_policy, 'f7_custom_map_policy', 'Hồ sơ policy F7 custom map')):
        if value is not None:
            admission[key] = str(_validated_policy_path(value, label))
    if unlock_all:
        if admission.get('profile_mode') != 'database_zero':
            raise ValueError('unlock_all yêu cầu profile_mode database_zero')
        admission['unlock_all_catalog'] = True
    if emit_lucci_135:
        if admission.get('profile_mode') != 'database_zero':
            raise ValueError('emit_lucci_135 yêu cầu profile_mode database_zero')
        admission['emit_lucci_135'] = True
    if project_own291_level:
        if admission.get('profile_mode') != 'database_zero':
            raise ValueError('project_own291_level yêu cầu profile_mode database_zero')
        admission['project_own291_level'] = True
    if web_shop_bind is not None:
        if admission.get('profile_mode') != 'database_zero':
            raise ValueError('web_shop yêu cầu profile_mode database_zero')
        try:
            host, port_text = web_shop_bind.rsplit(':', 1)
            ip = _concrete_ipv4(host)
            port = int(port_text)
        except ValueError as err:
            raise ValueError('Địa chỉ web_shop không hợp lệ') from err
        if port < 1 or port > 65535:
            raise ValueError('Địa chỉ web_shop không hợp lệ')
        if friends or s1:
            allowed_peers = ['127.0.0.1']
        else:
            peers = []
            for peer in config.get('allowed_peers') or []:
                try:
                    peers.append(str(_concrete_ipv4(peer)))
                except ValueError:
                    pass
            allowed_peers = list(dict.fromkeys(peers)) or ['127.0.0.1']
        admission['web_shop'] = {
            'bind': f'{ip}:{port}',
            'allowed_peers': allowed_peers,
            'max_connections': 16,
            'request_timeout_ms': 2000,
            'max_body_bytes': 8192,
            'session_ttl_seconds': 300,
            'unlock_all': bool(admission.get('unlock_all_catalog', False)),
        }


def opt_in_pack_maps(manifest, args, scope, needs_pack):
    selected = None
    for opt_in_scope, adapter in _OPT_IN_ADAPTERS.items():
        key, flag = f'{adapter}_maps', f'--{adapter}-map'
        cli_maps = getattr(args, key)
        if scope != opt_in_scope:
            if key in manifest or cli_maps is not None:
                raise ValueError(f'{flag} và {key} chỉ dùng với {opt_in_scope}')
            continue
        if cli_maps is not None and not needs_pack:
            raise ValueError(f'{flag} chỉ dùng với export, run, prepare-friends hoặc prepare-s1')
        if key in manifest and cli_maps is not None:
            raise ValueError(f'Chỉ chọn {flag} hoặc {key} trong manifest')
        if key in manifest or cli_maps is not None:
            selected = manifest.get(key, cli_maps)
            if (not isinstance(selected, list) or not selected
                    or any(not isinstance(name, str) or not name.strip() for name in selected)):
                raise ValueError(f'{flag} hoặc {key} phải chứa danh sách tên map không rỗng')
    if scope in _OPT_IN_ADAPTERS and needs_pack and selected is None:
        adapter = _OPT_IN_ADAPTERS[scope]
        raise ValueError(f'{scope} yêu cầu --{adapter}-map hoặc {adapter}_maps trong manifest')
    return selected


def prepare(root, state, cache=None, scope=None, f1_maps=None, *, opt_in_maps=None, friends=False, s1=False,
            economy_profile=None, emit_lucci_135=False, project_own291_level=False,
            web_shop_bind=None, unlock_all=False, f4_extended_policy=None,
            f3_pve_policy=None, f6_events_policy=None, f5_social_policy=None,
            f7_custom_map_policy=None):
    if friends and s1:
        raise ValueError('O1 và S1 dùng hai cấu hình phục vụ riêng')
    if scope in _OPT_IN_ADAPTERS and (cache is None or not opt_in_maps):
        raise ValueError(f'{scope} yêu cầu cache và danh sách map đã chọn')
    state = private_state(root, state)
    launch_socket = state / 'launch.sock'
    if os.name != 'nt' and max(len(os.fsencode(p)) for p in [launch_socket, state / 'accounts.admin.sock']) > 100:
        raise ValueError('Đường dẫn socket cục bộ quá dài')
    config = json.loads((root / 'config.template.json').read_text(encoding='utf-8'))
    admission = config['diagnostic_two_lobby']['account_admission']
    admission.update(database=str(state / 'accounts.sqlite'), launch_socket=str(launch_socket))
    for section, key in [('public_launcher', 'launcher.pk8'), ('prelogin_tls', 'prelogin.pk8')]:
        if admission.get(section) is not None:
            admission[section]['private_key'] = str(state / key)
    if friends:
        configure_friends(config)
    for section, directory in package_tls(root, state).items():
        if friends and section != 'prelogin_tls':
            continue
        cert = directory / 'certificate.der'
        admission[section].update(private_key=str(directory / 'private.pk8'),
                                  certificate=dict(path=str(cert), bytes=cert.stat().st_size,
                                                   sha256=digest(cert)))
    if s1:
        # Reserve all ports together so we never choose the same UDP port twice.
        # Rust verifies the service is ready before accepting the first S1 session.
        with (socket.socket(socket.AF_INET, socket.SOCK_DGRAM) as prelogin,
              socket.socket(socket.AF_INET, socket.SOCK_DGRAM) as relay,
              socket.socket(socket.AF_INET, socket.SOCK_STREAM) as launcher):
            for listener in (prelogin, relay, launcher):
                listener.bind(('127.0.0.1', 0))
            prelogin_addr, relay_addr, launcher_addr = [
                f'127.0.0.1:{listener.getsockname()[1]}' for listener in (prelogin, relay, launcher)
            ]
        config.update(bind='127.0.0.1', network_scope='private_lab')
        for actor in config['diagnostic_two_lobby']['actors']:
            actor.pop('reported_login_ipv4', None)
            actor['peer'] = f"127.0.0.{1 + actor['identity']}"
        config['allowed_peers'] = list(dict.fromkeys([
            '127.0.0.1', *(actor['peer'] for actor in config['diagnostic_two_lobby']['actors'])
        ]))
        primary = admission['prelogin_tls']
        primary.update(bind=prelogin_addr, max_per_ip=primary['max_connections'])
        # Both HTTPS and prelogin use the single leaf pinned in the invite.
        admission['public_launcher'] = {
            'bind': launcher_addr, 'certificate': primary['certificate'], 'private_key': primary['private_key'],
            'max_connections': 32, 'timeout_ms': 2000,
            'session_ttl_seconds': primary['session_ttl_seconds'],
            'session_idle_seconds': primary['session_idle_seconds']
        }
        config['authenticated_relay'].update(bind=relay_addr, advertise=relay_addr)
    configure_economy(config, friends=friends, s1=s1, economy_profile=economy_profile,
                      emit_lucci_135=emit_lucci_135, project_own291_level=project_own291_level,
                      web_shop_bind=web_shop_bind, unlock_all=unlock_all,
                      f4_extended_policy=f4_extended_policy, f3_pve_policy=f3_pve_policy,
                      f6_events_policy=f6_events_policy, f5_social_policy=f5_social_policy,
                      f7_custom_map_policy=f7_custom_map_policy)
    run = state / 'runs' / str(time.time_ns())
    run.mkdir(mode=0o700, parents=True)
    if cache is not None:
        from kr_pack_config import configure
        from kr_pack_export import F1, G1
        wire = run / 'wire'
        wire.mkdir(mode=0o700)
        maps = G1 if scope == 'g1' else F1 if scope == 'f1' else f1_maps
        if scope in _OPT_IN_ADAPTERS:
            maps = opt_in_maps
        configure(config, root, cache, wire, maps, scope, unlock_all=friends)
    runtime = run / 'config.json'
    fd = os.open(runtime, os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600)
    with os.fdopen(fd, 'w', encoding='utf-8') as stream:
        json.dump(config, stream, ensure_ascii=False, separators=(',', ':'))
        stream.write('\n')
    return runtime, run / 'service', state / 'accounts.sqlite'


def bundled_pack_data(root, manifest, scope):
    """JSON xuất sẵn trong gói (pack-data-rights.json), đã qua verify của manifest gói."""
    if manifest.get('pack_data') != 'pack-data' or scope not in ('g1', 'f1'):
        return None
    from kr_pack_export import prebuilt
    return prebuilt(member(root, 'pack-data'))


def friends_pack(player, expected_sha256, pins):
    """Select original package members pinned by the launcher manifest."""
    path = member(player, 'manifest.json')
    if path.stat().st_size > 4 * 1024 * 1024 or digest(path) != expected_sha256:
        raise ValueError('Manifest gói người chơi không khớp')
    value = json.loads(path.read_bytes())
    if value.get('schema') != 'kre-player-package-v1':
        raise ValueError('Cần gói người chơi KR')
    files = {}
    for name, hash_value in pins.items():
        if name.endswith('.idd'):
            rows = (row['parts'][0] for row in value['packs'] if row['path'].casefold() == name.casefold())
        else:
            rows = (row['payload'] for language in value['languages'].values()
                    for row in language['files'] if row['path'].casefold() == name.casefold())
        pin = next((row for row in rows if row['sha256'] == hash_value), None)
        if pin is None:
            raise ValueError(f'Gói người chơi thiếu pack gốc: {name}')
        files[name] = member(player, pin['path'])
    return files

def main():
    # The verified package stays immutable even when owned by the runner.
    sys.dont_write_bytecode = True
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('command', choices=['verify', 'init-db', 'init-tls', 'restore-db', 'migrate-db', 'export', 'run', 'prepare-friends', 'prepare-s1'])
    parser.add_argument('--manifest-sha256', required=True)
    parser.add_argument('--state', type=Path)
    parser.add_argument('--snapshot', type=Path)
    parser.add_argument('--game-dir', type=Path,
                        help='Tuỳ chọn dev/build: xuất lại từ thư mục game KR 00967 thay cho JSON đóng sẵn; '
                             'O1 dùng gốc gói người chơi')
    parser.add_argument('--f1-map', action='append', help='Tên map chính xác trong gói thử F1-a')
    parser.add_argument('--f2-map', dest='f2_maps', action='append',
                        help='Tên map F2 opt-in; lặp lại hoặc dùng danh sách f2_maps trong manifest')
    parser.add_argument('--f3-map', dest='f3_maps', action='append',
                        help='Tên map F3 opt-in; lặp lại hoặc dùng danh sách f3_maps trong manifest')
    parser.add_argument('--player-manifest-sha256', help='Pin manifest gói người chơi, chỉ dùng với O1')
    parser.add_argument('--economy-profile', type=Path, help='File JSON profile kinh tế F4 tuỳ chọn')
    parser.add_argument('--emit-lucci-135', action='store_true', help='Bật phát S->C 135 sau own291 và 175')
    parser.add_argument('--project-own291-level', action='store_true', help='Chiếu level hồ sơ vào S->C 291 (own291)')
    parser.add_argument('--web-shop-bind', help='Địa chỉ IPv4:port mở F4-d HTTP web shop nội bộ')
    parser.add_argument('--unlock-all', action='store_true', help='Bật chế độ F4 O1 unlock_all_catalog không ghi DB')
    parser.add_argument('--f4-extended-policy', type=Path, help='File JSON policy F4 extended tuỳ chọn')
    parser.add_argument('--f3-pve-policy', type=Path, help='File JSON policy F3 PvE tuỳ chọn')
    parser.add_argument('--f6-events-policy', type=Path, help='File JSON policy F6 events tuỳ chọn')
    parser.add_argument('--f5-social-policy', type=Path, help='File JSON policy F5 social tuỳ chọn')
    parser.add_argument('--f7-custom-map-policy', type=Path, help='File JSON policy F7 custom map tuỳ chọn')
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
        scope = manifest.get('pack_scope')
        if scope is not None and scope not in ('g1', 'f1', 'f1-a', *_OPT_IN_ADAPTERS):
            raise ValueError('Phạm vi pack trong gói không hợp lệ')
        needs_pack = scope is not None and args.command in ('export', 'run', 'prepare-friends', 'prepare-s1')
        opt_in_maps = opt_in_pack_maps(manifest, args, scope, needs_pack)
        opt_in_export = ({'maps': opt_in_maps, _OPT_IN_ADAPTERS[scope]: True}
                         if needs_pack and opt_in_maps is not None else {})
        if args.command == 'verify':
            print('Gói khớp manifest bàn giao.')
            return
        if (args.command == 'prepare-friends') != (args.player_manifest_sha256 is not None):
            raise ValueError('Chỉ prepare-friends yêu cầu pin manifest gói người chơi')
        if args.state is None:
            raise ValueError('Cần --state riêng')
        if (args.snapshot is not None) != (args.command == 'restore-db'):
            raise ValueError('Chỉ restore-db dùng và yêu cầu --snapshot')
        if (args.economy_profile is not None or args.emit_lucci_135 or args.project_own291_level
                or args.web_shop_bind is not None or args.unlock_all
                or args.f4_extended_policy is not None or args.f3_pve_policy is not None
                or args.f6_events_policy is not None or args.f5_social_policy is not None
                or args.f7_custom_map_policy is not None) and args.command not in ('run', 'prepare-friends', 'prepare-s1'):
            raise ValueError('Cờ policy chỉ dùng với run, prepare-friends hoặc prepare-s1')
        if scope == 'f1-a' and args.command == 'prepare-friends':
            raise ValueError('Catalog thử F1-a chỉ dùng với export hoặc run, không hỗ trợ O1')
        if (scope == 'f1-a' and args.command in ('export', 'run')) != bool(args.f1_map):
            raise ValueError('Gói F1-a yêu cầu --f1-map khi export hoặc run')
        # Mặc định dùng JSON đóng sẵn trong gói; --game-dir chỉ là đường xuất lại cho dev/build.
        prebuilt = (bundled_pack_data(root, manifest, scope)
                    if needs_pack and args.game_dir is None and args.command != 'prepare-friends' else None)
        if needs_pack and args.game_dir is None and prebuilt is None:
            raise ValueError('Gói này không kèm JSON đóng sẵn cho lệnh đã chọn; cần --game-dir')
        if not needs_pack and args.game_dir is not None:
            raise ValueError('--game-dir chỉ dùng khi export, run hoặc chuẩn bị server từ gói pack')
        if args.command in ('export', 'prepare-friends') and scope is None:
            raise ValueError('Gói này dùng fixture; cần gói xuất từ pack')
        if args.command != 'prepare-friends' and args.game_dir is not None and args.state.resolve().is_relative_to(args.game_dir.resolve()):
            raise ValueError('State phải nằm ngoài thư mục game chỉ đọc')
        admin = str(executable(root, 'kr-admin', target))
        state = private_state(root, args.state, admin)
        database = state / 'accounts.sqlite'
        if args.command == 'prepare-s1':
            # The caller holds the runner lock for its entire lifetime. S1
            # owns its TLS identity and never uses an externally installed key.
            if not manifest.get('self_host') or not needs_pack or 'prelogin_tls' not in package_tls(root, state):
                raise ValueError('Cần gói S1 phát hành cùng cấu hình cấp chứng chỉ khi cài')
            init_tls(root, state, target)
            if prebuilt is not None:
                cache = prebuilt
            else:
                from kr_pack_export import export
                cache, _ = export(args.game_dir, state / 'pack-cache', **opt_in_export)
            runtime, out, _ = prepare(root, state, cache, scope, s1=True, opt_in_maps=opt_in_maps,
                                      economy_profile=args.economy_profile,
                                      emit_lucci_135=args.emit_lucci_135,
                                      project_own291_level=args.project_own291_level,
                                      web_shop_bind=args.web_shop_bind,
                                      unlock_all=args.unlock_all,
                                      f4_extended_policy=args.f4_extended_policy,
                                      f3_pve_policy=args.f3_pve_policy,
                                      f6_events_policy=args.f6_events_policy,
                                      f5_social_policy=args.f5_social_policy,
                                      f7_custom_map_policy=args.f7_custom_map_policy)
            payload = {'runtime': str(runtime), 'out': str(out)}
            ws = json.loads(runtime.read_bytes())['diagnostic_two_lobby']['account_admission'].get('web_shop')
            if ws is not None:
                payload['web_shop_url'] = f"http://{ws['bind']}/"
            print(json.dumps(payload))
            return
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
            cache = prebuilt
            if prebuilt is not None and args.command == 'export':
                print(json.dumps(dict(cache=str(prebuilt), reused=True, prebuilt=True)))
                return
            if needs_pack and prebuilt is None:
                from kr_pack_export import MAPS, PACK_SHA256, export
                if args.command == 'prepare-friends':
                    files = friends_pack(args.game_dir, args.player_manifest_sha256, PACK_SHA256)
                    cache, reused = export(args.game_dir / 'payload', state / 'pack-cache', pack_files=files,
                                           **opt_in_export)
                else:
                    options = opt_in_export or dict(maps=args.f1_map or MAPS, f1=bool(args.f1_map))
                    cache, reused = export(args.game_dir, state / 'pack-cache', **options)
                if args.command == 'export':
                    print(json.dumps(dict(cache=str(cache), reused=reused)))
                    return
            if args.command == 'prepare-friends':
                command = [str(executable(root, 'kr-launch-identity', target)),
                           str(state / 'prelogin_tls-identity')]
                if not (state / 'prelogin_tls-identity').exists():
                    subprocess.run(command, check=True, cwd=state, stdout=subprocess.DEVNULL)
                if not database.exists():
                    request = dict(actor='friends-init', reason='Local friends room', command=dict(op='init'))
                    subprocess.run([admin, str(database)], input=json.dumps(request), text=True, check=True,
                                   cwd=root, stdout=subprocess.DEVNULL)
                config, out, _ = prepare(root, state, cache, scope, friends=True, opt_in_maps=opt_in_maps,
                                         economy_profile=args.economy_profile,
                                         emit_lucci_135=args.emit_lucci_135,
                                         project_own291_level=args.project_own291_level,
                                         web_shop_bind=args.web_shop_bind,
                                         unlock_all=args.unlock_all,
                                         f4_extended_policy=args.f4_extended_policy,
                                         f3_pve_policy=args.f3_pve_policy,
                                         f6_events_policy=args.f6_events_policy,
                                         f5_social_policy=args.f5_social_policy,
                                         f7_custom_map_policy=args.f7_custom_map_policy)
                value = json.loads(config.read_bytes())
                admission = value['diagnostic_two_lobby']['account_admission']
                tls = admission['prelogin_tls']
                payload = dict(config=str(config), out=str(out), address=tls['bind'],
                               certificate=tls['certificate']['path'], ports=value['ports'])
                if admission.get('web_shop') is not None:
                    payload['web_shop_url'] = f"http://{admission['web_shop']['bind']}/"
                print(json.dumps(payload, ensure_ascii=True))
                return
            if not database.is_file():
                raise ValueError('Cần init-db hoặc restore DB trước khi chạy')
            config, out, _ = prepare(root, state, cache, scope, args.f1_map,
                                     opt_in_maps=opt_in_maps,
                                     economy_profile=args.economy_profile,
                                     emit_lucci_135=args.emit_lucci_135,
                                     project_own291_level=args.project_own291_level,
                                     web_shop_bind=args.web_shop_bind,
                                     unlock_all=args.unlock_all,
                                     f4_extended_policy=args.f4_extended_policy,
                                     f3_pve_policy=args.f3_pve_policy,
                                     f6_events_policy=args.f6_events_policy,
                                     f5_social_policy=args.f5_social_policy,
                                     f7_custom_map_policy=args.f7_custom_map_policy)
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
