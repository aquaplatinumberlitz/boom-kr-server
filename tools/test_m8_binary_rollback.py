"""Rehearse schema-5 -> current schema -> previous release with real Linux binaries.

The current schema comes from the current kr-admin (6 at M8, 7 since F4).

With --package, also boot both kr-service binaries using held package fixtures
read-only. Every DB and config is generated in an isolated temporary directory.
"""
import argparse
import json
from pathlib import Path
import secrets
import socket
import subprocess
import tempfile
import time

from rust_package_runtime import verify


def execute(binary, *args, request=None, failure=None):
    result = subprocess.run([binary, *map(str, args)], input=request, text=True,
                            capture_output=True, timeout=20, check=False)
    if failure is not None:
        assert result.returncode and failure in result.stderr, result.stderr
        return
    assert result.returncode == 0, result.stderr
    return json.loads(result.stdout)


def operator(binary, db, op, **fields):
    request = json.dumps(dict(actor='rollback-test', reason='isolated migration acceptance',
                              command=dict(op=op, **fields)))
    return execute(binary, db, request=request)


def schema(path):
    import sqlite3
    with sqlite3.connect(path.as_uri() + '?mode=ro', uri=True) as db:
        return db.execute('PRAGMA user_version').fetchone()[0]


def loopback_config(package, state, with_backup):
    config = json.loads((package / 'config.template.json').read_text())
    config.update(bind='127.0.0.1', network_scope='private_lab')
    two = config['diagnostic_two_lobby']
    config['allowed_peers'] = [f'127.0.0.{i}' for i in range(1, len(two['actors']) + 1)]
    for actor, peer in zip(two['actors'], config['allowed_peers']):
        actor['peer'] = peer
    sockets = [socket.socket() for _ in range(3)]
    with socket.socket(type=socket.SOCK_DGRAM) as udp:
        udp.bind(('127.0.0.1', 0))
        address = f'127.0.0.1:{udp.getsockname()[1]}'
    try:
        for sock in sockets:
            sock.bind(('127.0.0.1', 0))
        ports = [sock.getsockname()[1] for sock in sockets]
    finally:
        for sock in sockets:
            sock.close()
    config.update(ports=ports)
    two.update(login_port=ports[0], channel_port=ports[1], wcp_ports=[ports[2]], login_candidate_ports=[])
    config['authenticated_relay'].update(bind=address, advertise=address)
    admission = two['account_admission']
    admission.update(database=str(state / 'accounts.sqlite'), launch_socket=str(state / 'launch.sock'))
    admission.pop('prelogin_tls', None)
    admission.pop('public_launcher', None)
    if with_backup:
        admission['backup'] = dict(interval_seconds=60, keep=2)
    path = state / 'config.json'
    path.write_text(json.dumps(config))
    return path


def service(binary, package, state, *, failure=None, with_backup=False, bad_config=False):
    config = loopback_config(package, state, with_backup)
    if bad_config:
        invalid = json.loads(config.read_text())
        invalid['ports'] = []
        config.write_text(json.dumps(invalid))
    out = state / f'run-{time.monotonic_ns()}'
    with subprocess.Popen([binary, '--config', str(config), '--out', str(out)],
                          cwd=package, stdout=subprocess.DEVNULL, stderr=subprocess.PIPE, text=True) as proc:
        try:
            if failure:
                error = proc.communicate(timeout=12)[1]
                assert proc.returncode and failure in error, error
                return
            deadline = time.monotonic() + 12
            while not (out / 'service-state.json').exists():
                assert proc.poll() is None, proc.stderr.read() if proc.poll() is not None else ''
                assert time.monotonic() < deadline, 'Readiness deadline'
                time.sleep(.025)
            assert json.loads((out / 'service-state.json').read_text())['ready']
            proc.terminate()
            error = proc.communicate(timeout=15)[1]
            assert proc.returncode == 0, error
            assert json.loads((out / 'service-state.json').read_text())['stopped']
        finally:
            if proc.poll() is None:
                proc.kill()
                proc.wait()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--previous', required=True, type=Path, help='Directory with schema-5 kr-admin/kr-service')
    parser.add_argument('--current', required=True, type=Path, help='Directory with current-schema kr-admin/kr-service')
    parser.add_argument('--package', type=Path, help='Pinned Linux fixture package for kr-service process checks')
    parser.add_argument('--manifest-sha256')
    args = parser.parse_args()
    if (args.package is None) != (args.manifest_sha256 is None):
        parser.error('--package requires --manifest-sha256')
    previous, current = args.previous.resolve(), args.current.resolve()
    package = args.package.resolve() if args.package else None
    if package:
        verify(package, args.manifest_sha256)

    with tempfile.TemporaryDirectory(prefix='kr-m8-') as tmp:
        state = Path(tmp)
        db = state / 'accounts.sqlite'
        older, newer = str(previous / 'kr-admin'), str(current / 'kr-admin')
        password = secrets.token_urlsafe(24)
        probe = state / 'probe'
        probe.mkdir(mode=0o700)
        target = operator(newer, probe / 'accounts.sqlite', 'init')['schema']
        assert isinstance(target, int) and target >= 6, target
        snapshots = f'accounts.sqlite.before-v5-to-v{target}-*.sqlite'
        assert operator(older, db, 'init')['schema'] == 5
        account = operator(older, db, 'create', login='m8rollback', name='Rollback',
                           password=password)['player']
        if package:
            service(str(previous / 'kr-service'), package, state)
            service(str(current / 'kr-service'), package, state, with_backup=True)
        else:
            assert execute(newer, 'migrate', db)['schema'] == target
        assert schema(db) == target
        pinned, = state.glob(snapshots)
        assert execute(newer, 'verify-backup', pinned)['schema'] == 5

        # Migration must not run twice and a failed upgrade config must never
        # become the rollback config.
        execute(newer, 'migrate', db)
        assert list(state.glob(snapshots)) == [pinned]
        if package:
            service(str(current / 'kr-service'), package, state, bad_config=True,
                    failure='unsupported diagnostic configuration')
            service(str(previous / 'kr-service'), package, state, failure='Schema')
        else:
            execute(older, db, request=json.dumps(dict(actor='test', reason='schema fence',
                                                       command=dict(op='init'))), failure='Schema')
        operator(newer, db, 'create', login='afterupgrade', name='Discarded',
                 password=secrets.token_urlsafe(24))
        restored = state / 'rollback'
        restored.mkdir(mode=0o700)
        t0 = time.monotonic()
        rollback = restored / 'accounts.sqlite'
        assert execute(newer, 'restore', pinned, rollback)['schema'] == 5
        if package:
            service(str(previous / 'kr-service'), package, restored)
        else:
            operator(older, rollback, 'begin-instance')
        assert operator(older, rollback, 'login', login='m8rollback', password=password,
                        ttl_seconds=60)['player'] == account
        elapsed = time.monotonic() - t0
        execute(older, rollback, request=json.dumps(dict(actor='test', reason='RPO fence',
                                                        command=dict(op='lookup', login='afterupgrade'))),
                failure='InvalidCredentials')
        assert schema(rollback) == 5 and schema(db) == target
    print(f'PASS: schema 5 → {target}; old binary refuses schema {target}; schema-5 restore and old login; '
          + ('both service binaries reached readiness and stopped' if package else 'operator binaries only'))
    print(f'Local restore + {"previous service boot/stop + " if package else ""}old login: {elapsed:.3f}s')


if __name__ == '__main__':
    main()
