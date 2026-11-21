"""Kiểm gói Windows dưới Wine, dùng thư mục game chỉ đọc và hai client Linux."""
import argparse
import ctypes
import hashlib
import hmac
import json
import os
from pathlib import Path
import signal
import socket
import sqlite3
import subprocess
import sys
import time


def console(root, options):
    """Run the packaged runner in a private Windows console."""
    assert os.name == 'nt'
    kernel = ctypes.WinDLL('kernel32', use_last_error=True)
    # Wine's headless inherited console cannot deliver group events. Allocate
    # a real, private console under Xvfb before creating the target group.
    kernel.FreeConsole()
    if not kernel.AllocConsole():
        raise ctypes.WinError(ctypes.get_last_error())
    root = Path(root)
    signal.signal(signal.SIGINT, lambda *_: None)
    child = subprocess.Popen(options, stdout=sys.stdout, stderr=sys.stderr)
    try:
        deadline = time.monotonic() + 100
        stop = root / 'stop'
        while not stop.exists() and child.poll() is None and time.monotonic() < deadline:
            time.sleep(.1)
        if child.poll() is not None:
            raise RuntimeError(f'Runner exited before console event: {child.returncode}')
        if not stop.exists():
            raise TimeoutError('Console stop was never requested')
        os.kill(0, signal.CTRL_C_EVENT)
        assert (code := child.wait(timeout=12)) == 0, f'Ctrl+C runner exit {code:#x}'
        print('CTRL_C_EVENT: runner exit 0', flush=True)
    finally:
        if child.poll() is None:
            child.terminate()
            child.wait(timeout=5)


def windows_path(path):
    return 'Z:' + str(path.resolve()).replace('/', '\\')


def local_ipc(path, request, *, reflect=False):
    raw = path.read_bytes()
    assert raw[:8] == b'KRIPCv1\0' and len(raw) == 42
    port = int.from_bytes(raw[8:10], 'big')
    key = raw[10:]
    with socket.create_connection(('127.0.0.1', port), timeout=4) as stream:
        nonce = os.urandom(32)
        stream.sendall(nonce)
        with stream.makefile('rb') as incoming:
            challenge = incoming.read(64)
            transcript = nonce + challenge[:32]
            expected = hmac.digest(key, b'kr-local-ipc-v1/server\0' + transcript, 'sha256')
            assert hmac.compare_digest(challenge[32:], expected)
            proof = challenge[32:] if reflect else hmac.digest(
                key, b'kr-local-ipc-v1/client\0' + transcript, 'sha256')
            stream.sendall(proof + json.dumps(request).encode() + b'\n')
            stream.shutdown(socket.SHUT_WR)
            try:
                return incoming.read(4096)
            except ConnectionResetError:
                if reflect:
                    return b''
                raise


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for option in ('bundle', 'python', 'kr-load', 'game-dir', 'out'):
        parser.add_argument('--' + option, required=True, type=Path)
    args = parser.parse_args()
    out, bundle, game = (p.resolve(strict=True) for p in (args.out, args.bundle, args.game_dir))
    assert out.is_relative_to(Path(__file__).resolve().parents[1] / 'local')
    assert Path(os.environ['WINEPREFIX']).resolve().is_relative_to(out.parent)
    assert [row['ifname'] for row in json.loads(subprocess.check_output(['ip', '-j', 'link']))] == ['lo']
    assert os.statvfs(game).f_flag & os.ST_RDONLY
    assert os.statvfs(bundle).f_flag & os.ST_RDONLY

    manifest_hash = hashlib.sha256((bundle / 'manifest.json').read_bytes()).hexdigest()
    python = windows_path(args.python)
    admin = ['wine', str(bundle / 'bin/kr-admin.exe')]
    state = out / 'state'
    db = state / 'accounts.sqlite'
    runner = [python, windows_path(bundle / 'kr-package.py')]
    options = ['--manifest-sha256', manifest_hash, '--state', windows_path(state)]
    game_arg = ['--game-dir', windows_path(game)]

    def run(log, cmd, *, data=None, expected=0, secret=False):
        result = subprocess.run(cmd, capture_output=True, timeout=45, input=data)
        (out / (log + '.log')).write_bytes((b'' if secret else result.stdout) + result.stderr)
        if result.returncode != expected:
            raise RuntimeError(f'{log}: exit {result.returncode}; xem log')
        return result.stdout

    def install(op, *extra, log=None, expected=0):
        return run(log or op, ['wine', *runner, op, *options, *extra], expected=expected)

    def manage(operation, expect=0, **fields):
        data = json.dumps(dict(actor='wine-smoke', reason='Synthetic loopback match',
                               command=dict(op=operation, **fields))).encode()
        return run(operation + fields.get('login', ''), [*admin, windows_path(db)], data=data,
                   expected=expect, secret=operation == 'login')

    run('wine-version', ['wine', '--version'])
    run('python-version', ['wine', python, '-V'])
    shared = out / 'shared'
    shared.mkdir(mode=0o777)
    shared.chmod(0o777)
    run('acl-refused', [*admin, 'private-dir', windows_path(shared)], expected=1)
    install('init-db')
    exported = json.loads(install('export', *game_arg))
    assert not exported['reused']
    assert json.loads(install('export', *game_arg, log='export-reuse'))['reused']
    accounts = [dict(login=name, name=name, password='synthetic-test-password') for name in ('AlphaOne', 'BetaTwo')]
    for account in accounts:
        manage('create', **account)

    config = json.loads((bundle / 'config.template.json').read_bytes())
    two = config['diagnostic_two_lobby']
    tls = two['account_admission']['prelogin_tls']['bind']
    relay = config['authenticated_relay']['advertise']
    assert config['bind'] == '127.0.0.1'
    assert all(item.startswith('127.0.0.1:') for item in (tls, relay))
    replay = out / 'replay.json'
    window = dict(duration_ns=1_000_000_000, packets=[[83_000_000 * i, 24] for i in range(12)])
    replay.write_text(json.dumps(dict(schema='kr-m8-replay-v1', checkpoint='CP287', windows=[window] * 8)))
    load_config = out / 'load.json'
    with os.fdopen(os.open(load_config, os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600), 'w') as stream:
        json.dump(dict(server=tls, relay=relay, certificate=str(state / 'prelogin_tls-identity/certificate.der'),
                       login_port=two['login_port'], channel_port=two['channel_port'], replay=str(replay),
                       rooms=[accounts], ramp_ms=225, active_seconds=2, minimum_run_seconds=0, minimum_rounds=1,
                       delivery_p99_ms_max=500, delivery_loss_percent_max=0), stream)

    with (out / 'service.log').open('xb') as log:
        child = subprocess.Popen(
            ['wine', python, windows_path(Path(__file__)), '--console', windows_path(out),
             *runner, 'run', *options, *game_arg], stdout=log, stderr=subprocess.STDOUT,
        )
        try:
            deadline = time.monotonic() + 30
            while time.monotonic() < deadline:
                ready = list(state.glob('runs/*/service/service-state.json'))
                if ready and json.loads(ready[0].read_bytes()).get('ready'):
                    break
                assert child.poll() is None, 'Server exited before readiness; see service.log'
                time.sleep(.1)
            else:
                raise TimeoutError('Service readiness deadline')
            run('health', [*admin, 'health', windows_path(ready[0].parent)])
            install('run', *game_arg, log='runner-lock', expected=1)
            run('instance-lock', [*admin, 'migrate', windows_path(db)], expected=1)
            # A reflected server proof cannot authorize an otherwise valid request.
            denied = dict(actor='ipc-denied', reason='negative control', command={'op': 'list_rooms'})
            assert not local_ipc(state / 'accounts.admin.sock', denied, reflect=True)
            assert json.loads(manage('list-rooms')) == {'rooms': []}

            events = [json.loads(line) for line in run('load', [str(args.kr_load.resolve()), str(load_config)]).splitlines()]
            assert sum(event['event'] == 'round_returned' for event in events) == 1
            account = accounts[0]
            grant = json.loads(manage('login', login=account['login'], password=account['password'], ttl_seconds=120))
            request = dict(session=grant['token'], player_id=grant['player']['player_id'], peer='127.0.0.1')
            ticket = json.loads(local_ipc(state / 'launch.sock', request))
            assert ticket['ok'] and ticket['player']['player_id'] == request['player_id'] and ticket['frontend']
            snap = state / 'smoke.sqlite'
            run('backup', [*admin, 'backup', windows_path(db), windows_path(snap)])
            run('verify-backup', [*admin, 'verify-backup', windows_path(snap)])
            with sqlite3.connect(f'{snap.as_uri()}?mode=ro', uri=True) as connection:
                assert connection.execute('SELECT count(*) FROM accounts').fetchone() == (2,)
                assert connection.execute("SELECT count(*) FROM admin_audit WHERE actor='ipc-denied'").fetchone() == (0,)
                matches = connection.execute('SELECT state, map_name FROM matches').fetchall()
                assert len(matches) == 1 and matches[0][0] == 'complete', matches
                assert connection.execute('SELECT count(*), sum(reward) FROM match_players').fetchone() == (2, 0)

            (out / 'stop').touch(exist_ok=False)
            assert child.wait(timeout=20) == 0, 'Console shutdown failed; see service.log'
            assert json.loads(ready[0].read_bytes())['stopped']
            assert not (state / 'accounts.admin.sock').exists() and not (state / 'launch.sock').exists()
            run('released-lock', [*admin, 'migrate', windows_path(db)])
        finally:
            if child.poll() is None:
                child.terminate()
                child.wait(timeout=5)
    result = dict(manifest_sha256=manifest_hash, pack_cache=Path(exported['cache'].split('\\')[-1]).name,
                  export=True, cache_reused=True, round_returned=1, map=matches[0][1], players=2,
                  admin_ipc=True, launch_ticket=True, unauthenticated_refused=True, runner_lock=True, instance_lock=True,
                  public_acl_refused=True, backup_verified=True, graceful_stop='CTRL_C_EVENT', exit=0)
    (out / 'verification.json').write_text(json.dumps(result, indent=2) + '\n')
    print(json.dumps(result))


if __name__ == '__main__':
    if sys.argv[1:2] == ['--console']:
        console(sys.argv[2], sys.argv[3:])
    else:
        main()
