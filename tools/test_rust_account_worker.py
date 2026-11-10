"""Retain real SQLite/KDF worker tests and verify quiescent DBs; no KR wire proof."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import sqlite3
import subprocess
import time


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--out', type=Path, required=True)
    args = parser.parse_args()
    args.out.mkdir(parents=True, exist_ok=False, mode=0o700)
    os.chmod(args.out, 0o700)
    root = args.out.resolve()
    raw = root / 'raw'
    raw.mkdir(mode=0o700)
    command = ['cargo', 'test', '--manifest-path', 'rust/Cargo.toml', '-p', 'kr-service',
               '--lib', '--no-run', '--message-format=json', '--locked']
    build = subprocess.run(command, text=True, capture_output=True)
    (root / 'build.jsonl').write_text(build.stdout)
    (root / 'build.stderr').write_text(build.stderr)
    build.check_returncode()
    artifacts = [json.loads(line) for line in build.stdout.splitlines()]
    binaries = [Path(x['executable']) for x in artifacts
                if x.get('reason') == 'compiler-artifact' and x.get('executable')
                and x.get('target', {}).get('name') == 'kr_service']
    assert len(binaries) == 1, 'ambiguous worker test binary'
    binary = binaries[0]
    source = [Path('rust/Cargo.toml'), Path('rust/Cargo.lock'),
              Path('rust/rust-toolchain.toml'), Path(__file__), binary,
              *sorted(Path('rust/crates').rglob('*.rs')),
              *sorted(Path('rust/crates').rglob('*.sql')),
              *sorted(Path('rust/crates').rglob('Cargo.toml'))]
    source = [p for p in source if p.is_file()]
    frozen = root / 'frozen'
    frozen.mkdir(mode=0o700)
    pins = []
    for index, path in enumerate(source):
        retained = frozen / f'{index:03d}-{path.name}'
        shutil.copyfile(path, retained)
        pins.append(dict(path=str(path), retained_path=str(retained),
                         bytes=path.stat().st_size, sha256=sha(path)))
    recipe = [str(binary.resolve()), 'account_worker::tests::', '--test-threads=1', '--nocapture']
    (root / 'preflight.json').write_text(json.dumps(dict(files=pins, command=recipe,
        environment=dict(KR_ACCOUNT_WORKER_TEST_RAW=str(raw))), indent=2)+'\n')
    env = dict(os.environ, KR_ACCOUNT_WORKER_TEST_RAW=str(raw))
    started = time.monotonic()
    with (root / 'process.log').open('wb') as log:
        process = subprocess.Popen(recipe, env=env, stdout=log, stderr=subprocess.STDOUT)
        try:
            code = process.wait(timeout=180)
        finally:
            if process.poll() is None:
                process.kill()
                process.wait()
    unchanged = all(p.stat().st_size == pin['bytes'] and sha(p) == pin['sha256']
                    for p, pin in zip(source, pins))
    databases = []
    for db in sorted(raw.glob('*/accounts.sqlite')):
        with sqlite3.connect(db.as_uri()+'?mode=ro', uri=True) as conn:
            assert conn.execute('PRAGMA integrity_check').fetchone() == ('ok',)
            assert conn.execute('PRAGMA foreign_key_check').fetchall() == []
            databases.append(dict(path=str(db), accounts=conn.execute('SELECT count(*) FROM accounts').fetchone()[0],
                active=conn.execute('SELECT count(*) FROM sessions WHERE revoked_at IS NULL').fetchone()[0],
                schema=conn.execute('SELECT version FROM schema_migrations').fetchall(),
                audit=conn.execute('SELECT operation,count(*) FROM admin_audit GROUP BY operation ORDER BY operation').fetchall()))
        assert b'worker-test-password' not in db.read_bytes()
    evidence = [dict(path=str(p), bytes=p.stat().st_size, sha256=sha(p))
                for p in sorted(root.rglob('*')) if p.is_file()]
    text = (root / 'process.log').read_text(errors='replace')
    ok = code == 0 and unchanged and len(databases) == 5 and '5 passed; 0 failed' in text
    report = dict(result='pass' if ok else 'fail', duration_s=time.monotonic()-started,
        returncode=code, pid=process.pid, remaining_pid=None if process.poll() is not None else process.pid,
        source_frozen=unchanged, binary_sha256=sha(binary), databases=databases, evidence=evidence,
        limits='Backend design; real SQLite/KDF/dedicated thread on current-thread Tokio, no KR/native wire bridge, idleTTL/rate/load/OSdiskfull/restore or hard OS I/O deadline claim. Expected injected worker panic/audit fault retained in passing test log.')
    (root / 'verification.json').write_text(json.dumps(report, indent=2)+'\n')
    assert ok, 'worker acceptance failed; retained raw and process log'
    print('PASS 5 worker cases/5 retained SQLite DBs/source frozen/process gone; no KR bridge proof')


if __name__ == '__main__':
    main()
