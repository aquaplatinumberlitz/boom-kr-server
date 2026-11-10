"""Real-process SQLite/admin acceptance. Backend design, not a KR wire oracle."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import secrets
import sqlite3
import subprocess
import time


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binary', type=Path, required=True)
    parser.add_argument('--out', type=Path, required=True)
    args = parser.parse_args()
    binary = args.binary.resolve()
    args.out.mkdir(parents=True, exist_ok=False, mode=0o700)
    os.chmod(args.out, 0o700)
    root = args.out.resolve()
    store = root / 'store'
    store.mkdir(mode=0o700)
    db = store / 'accounts.sqlite'
    raw = root / 'raw'
    raw.mkdir(mode=0o700)
    source_paths = [Path('rust/Cargo.toml'), Path('rust/Cargo.lock'),
                    Path('rust/crates/kr-service/Cargo.toml'),
                    Path('rust/crates/kr-service/src/accounts.rs'),
                    Path('rust/crates/kr-service/src/accounts/tests.rs'),
                    Path('rust/crates/kr-service/src/bin/kr-admin.rs'),
                    Path('rust/crates/kr-service/migrations/0001_accounts.sql'),
                    Path(__file__), binary]
    frozen = root / 'frozen'
    frozen.mkdir(mode=0o700)
    pins = []
    for i, path in enumerate(source_paths):
        retained = frozen / f'{i:02d}-{path.name}'
        retained.write_bytes(path.read_bytes())
        pins.append(dict(path=str(path), retained_path=str(retained),
                         bytes=path.stat().st_size, sha256=sha(path)))
    (root / 'preflight.json').write_text(json.dumps(dict(files=pins, limits='No game/native run; trusted local CLI only.'), indent=2)+'\n')
    password, other = secrets.token_urlsafe(24), secrets.token_urlsafe(24)
    rows, children = [], []

    def invoke(case, command=None, error=None, payload=None, argv=None):
        if payload is None:
            payload = json.dumps(dict(actor='acceptance-admin', reason='isolated synthetic backend acceptance', command=command)).encode()
        start = time.monotonic()
        proc = subprocess.Popen([str(binary), str(db), *(argv or [])], stdin=subprocess.PIPE,
                                stdout=subprocess.PIPE, stderr=subprocess.PIPE)
        children.append(proc.pid)
        try:
            stdout, stderr = proc.communicate(payload, timeout=8)
        finally:
            if proc.poll() is None:
                proc.kill()
                proc.communicate(timeout=2)
        prefix = raw / f'{len(rows)+1:02d}-{case}'
        prefix.with_suffix('.stdout').write_bytes(stdout)
        prefix.with_suffix('.stderr').write_bytes(stderr)
        assert password.encode() not in stdout+stderr and other.encode() not in stdout+stderr
        if error:
            assert proc.returncode == 1 and not stdout and stderr == f'kr-admin: {error}\n'.encode(), (case, proc.returncode)
            result = None
        else:
            assert proc.returncode == 0 and not stderr, (case, proc.returncode, stderr)
            result = json.loads(stdout)
        rows.append(dict(case=case, returncode=proc.returncode, duration_seconds=time.monotonic()-start,
                         input_sha256=hashlib.sha256(payload).hexdigest(), expected_error=error,
                         stdout_path=str(prefix.with_suffix('.stdout')), stderr_path=str(prefix.with_suffix('.stderr'))))
        return result

    invoke('init', dict(op='init'))
    alice = invoke('create-A', dict(op='create', login='Alice', name='Alice', password=password))['player']
    bob = invoke('create-B', dict(op='create', login='Bobby', name='Bobby', password=other))['player']
    assert alice['player_id'] != bob['player_id'] and alice['wire_id'] != bob['wire_id']
    invoke('no-instance', dict(op='login', login='alice', password=password, ttl_seconds=60), error='InstanceRequired')
    assert invoke('begin-instance', dict(op='begin-instance'))['epoch'] == 1
    invoke('wrong-password', dict(op='login', login='alice', password=other, ttl_seconds=60), error='InvalidCredentials')
    invoke('unknown-user', dict(op='login', login='nobody', password=other, ttl_seconds=60), error='InvalidCredentials')
    invoke('duplicate-user', dict(op='create', login='ALICE', name='Alias', password=other), error='Conflict')
    a = invoke('login-A', dict(op='login', login='ALICE', password=password, ttl_seconds=60))
    b = invoke('login-B', dict(op='login', login='bobby', password=other, ttl_seconds=60))
    assert a['token'] != b['token'] and len(a['token']) == 64
    invoke('one-active', dict(op='login', login='alice', password=password, ttl_seconds=60), error='ActiveSession')
    assert invoke('authorize-own', dict(op='authorize', token=a['token'], player_id=alice['player_id']))['player'] == alice
    invoke('wrong-owner', dict(op='authorize', token=a['token'], player_id=bob['player_id']), error='InvalidSession')
    renamed = invoke('rename', dict(op='rename', account_id=alice['account_id'], name='NewName', expected_version=1))['player']
    assert renamed['player_id'] == alice['player_id'] and renamed['wire_id'] == alice['wire_id'] and renamed['profile_version'] == 2
    invoke('stale-profile', dict(op='rename', account_id=alice['account_id'], name='StaleName', expected_version=1), error='Conflict')
    assert invoke('reopen-persistent', dict(op='lookup', login='alice'))['player'] == renamed
    invoke('disable', dict(op='disable', account_id=alice['account_id']))
    invoke('disabled-token', dict(op='authorize', token=a['token'], player_id=alice['player_id']), error='InvalidSession')
    invoke('disabled-login', dict(op='login', login='alice', password=password, ttl_seconds=60), error='InvalidCredentials')
    invoke('enable', dict(op='enable', account_id=alice['account_id']))
    a2 = invoke('login-A2', dict(op='login', login='alice', password=password, ttl_seconds=60))
    invoke('password-reset', dict(op='password-reset', account_id=alice['account_id'], password=other))
    invoke('reset-token', dict(op='authorize', token=a2['token'], player_id=alice['player_id']), error='InvalidSession')
    invoke('reset-old-password', dict(op='login', login='alice', password=password, ttl_seconds=60), error='InvalidCredentials')
    a3 = invoke('new-password', dict(op='login', login='alice', password=other, ttl_seconds=60))
    invoke('explicit-revoke', dict(op='revoke', account_id=alice['account_id']))
    invoke('revoked-token', dict(op='authorize', token=a3['token'], player_id=alice['player_id']), error='InvalidSession')
    assert invoke('new-instance', dict(op='begin-instance'))['epoch'] == 2
    invoke('old-instance-B', dict(op='authorize', token=b['token'], player_id=bob['player_id']), error='InvalidSession')
    invoke('malformed-json', payload=b'{', error='InvalidInput')
    invoke('unknown-field', payload=b'{"actor":"a","reason":"r","command":{"op":"init","extra":1}}', error='InvalidInput')
    invoke('oversize', payload=b' '*4097, error='InvalidInput')
    invoke('no-secrets-argv', dict(op='init'), argv=['forbidden-extra-argument'], error='InvalidInput')
    with sqlite3.connect(db) as conn:
        assert conn.execute('PRAGMA integrity_check').fetchone() == ('ok',)
        assert conn.execute('PRAGMA foreign_key_check').fetchall() == []
        assert conn.execute('SELECT count(*) FROM sessions WHERE revoked_at IS NULL').fetchone() == (0,)
        assert conn.execute('SELECT count(*) FROM accounts').fetchone() == (2,)
        audit_count = conn.execute('SELECT count(*) FROM admin_audit').fetchone()[0]
        assert audit_count == 9
    conn.close()
    for path in store.iterdir():
        data = path.read_bytes()
        for secret in [password, other, a['token'], b['token'], a2['token'], a3['token']]:
            assert secret.encode() not in data, path
        assert path.stat().st_mode & 0o077 == 0
    assert all(sha(Path(row['path'])) == row['sha256'] for row in pins)
    for pid in children:
        assert not Path(f'/proc/{pid}').exists()
    summary = dict(result='pass', cases=rows, accounts=2, audit_rows=audit_count,
                   binary_sha256=sha(binary), source_frozen=True, cleanup=dict(remaining_pids=[]),
                   limits='Company backend/real SQLite/admin CLI design acceptance only. No native KR account bridge, transport lease, gameplay, idle TTL, network rate control, full backup/restore or production claim. Raw token outputs are synthetic local secrets under a private directory, not published.')
    (root / 'verification.json').write_text(json.dumps(summary, indent=2)+'\n')
    files = [p for p in root.rglob('*') if p.is_file()]
    (root / 'raw-seal.json').write_text(json.dumps(dict(source_frozen=True, cleanup=summary['cleanup'], files=[dict(path=str(p), bytes=p.stat().st_size, sha256=sha(p)) for p in sorted(files)]), indent=2)+'\n')
    print(f'PASS {len(rows)} CLI processes; 2 persistent accounts; {audit_count} audited mutations; cleanup0; no KR authentication claim')


if __name__ == '__main__':
    main()
