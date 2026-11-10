"""Upgrade private copies of sealed schema1 DBs; independently compare all old rows."""
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


def old_snapshot(db):
    with sqlite3.connect(db.as_uri()+'?mode=ro', uri=True) as conn:
        assert conn.execute('PRAGMA integrity_check').fetchone() == ('ok',)
        tables = dict(accounts='*', players='*', runtime_state='*', admin_audit='*',
            sessions='id,account_id,token_hash,boot_epoch,credential_version,issued_at,expires_at,revoked_at')
        result = {}
        for table, columns in tables.items():
            rows = conn.execute(f'SELECT {columns} FROM {table} ORDER BY id').fetchall()
            # Only hashes/counts leave the private raw DB; PHC/token hashes stay local.
            encoded = json.dumps([[{'blob': v.hex()} if isinstance(v, bytes) else v for v in row] for row in rows], sort_keys=True).encode()
            result[table] = dict(rows=len(rows), sha256=hashlib.sha256(encoded).hexdigest())
        version = conn.execute('PRAGMA user_version').fetchone()[0]
        checksums = conn.execute('SELECT version,sha256 FROM schema_migrations ORDER BY version').fetchall()
        return dict(tables=result, version=version, migrations=checksums)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binary', type=Path, required=True)
    parser.add_argument('--baseline', type=Path, required=True, help='sealed CP243 worker acceptance root')
    parser.add_argument('--out', type=Path, required=True)
    args = parser.parse_args()
    args.out.mkdir(parents=True, exist_ok=False, mode=0o700)
    os.chmod(args.out, 0o700)
    root = args.out.resolve()
    raw = root/'raw'
    raw.mkdir(mode=0o700)
    prior = json.loads((args.baseline/'verification.json').read_text())
    assert prior['result'] == 'pass' and prior['source_frozen'] and len(prior['databases']) == 5
    # Verify the baseline, not just paths chosen by a fresh script.
    for row in prior['evidence']:
        p = Path(row['path'])
        assert p.stat().st_size == row['bytes'] and sha(p) == row['sha256']
    source = [Path(__file__), args.binary, Path('rust/Cargo.toml'), Path('rust/Cargo.lock'), Path('rust/rust-toolchain.toml'),
              *sorted(Path('rust/crates').rglob('*.rs')), *sorted(Path('rust/crates').rglob('*.sql'))]
    source += [Path(x['path']) for x in prior['evidence']]
    frozen = root/'frozen'
    frozen.mkdir(mode=0o700)
    pins = []
    for i, p in enumerate(source):
        retained = frozen/f'{i:03d}-{p.name}'
        shutil.copyfile(p, retained)
        pins.append(dict(path=str(p), retained_path=str(retained), bytes=p.stat().st_size, sha256=sha(p)))
    (root/'preflight.json').write_text(json.dumps(dict(files=pins, limits='Baseline DBs never opened/mutated; copied DBs only. Backend CLI init triggers migration; no KR run.'), indent=2)+'\n')
    rows = []
    children = []
    started = time.monotonic()
    for index, entry in enumerate(prior['databases']):
        folder = raw/f'case{index}'
        folder.mkdir(mode=0o700)
        original = Path(entry['path'])
        for sibling in original.parent.glob('accounts.sqlite*'):
            shutil.copyfile(sibling, folder/sibling.name)
            os.chmod(folder/sibling.name, 0o600)
        db = folder/'accounts.sqlite'
        before = old_snapshot(db)
        assert before['version'] == 1
        assert before['migrations'] == [(1, sha(Path('rust/crates/kr-service/migrations/0001_accounts.sql')))]
        for attempt in range(2):
            with (folder/f'init{attempt}.stdout').open('wb') as out, (folder/f'init{attempt}.stderr').open('wb') as err:
                process = subprocess.Popen([str(args.binary.resolve()), str(db)], stdin=subprocess.PIPE, stdout=out, stderr=err)
                children.append(process.pid)
                try:
                    process.communicate(json.dumps(dict(actor='migration-test', reason='copy-only conservation', command=dict(op='init'))).encode(), timeout=15)
                finally:
                    if process.poll() is None:
                        process.kill()
                        process.wait()
                assert process.returncode == 0
            response = json.loads((folder/f'init{attempt}.stdout').read_text())
            assert response['schema'] == 2
            after = old_snapshot(db)
            assert after['tables'] == before['tables']
            assert after['version'] == 2 and after['migrations'] == [(1, sha(Path('rust/crates/kr-service/migrations/0001_accounts.sql'))), (2, sha(Path('rust/crates/kr-service/migrations/0002_session_idle.sql')))]
            with sqlite3.connect(db.as_uri()+'?mode=ro', uri=True) as conn:
                assert conn.execute('SELECT count(*) FROM sessions WHERE idle_timeout_seconds IS NOT NULL OR last_seen_at IS NOT NULL').fetchone() == (0,)
                assert conn.execute('PRAGMA foreign_key_check').fetchall() == []
        rows.append(dict(baseline=str(original), copy=str(db), before=before, after=after, repeat_open='unchanged', old_idle_columns='all NULL'))
    assert all(p.stat().st_size == row['bytes'] and sha(p) == row['sha256'] for p, row in zip(source, pins))
    evidence = [dict(path=str(p), bytes=p.stat().st_size, sha256=sha(p)) for p in sorted(root.rglob('*')) if p.is_file()]
    (root/'verification.json').write_text(json.dumps(dict(result='pass', duration_s=time.monotonic()-started, cases=rows,
        source_frozen=True, baseline_unchanged=True, binary_sha256=sha(args.binary), children=children, remaining_pids=[], evidence=evidence,
        limits='5 copied historical schema1 DBs/10 CLI init processes; conservation of every old row/field only, no KR/native login/new idle session/production backup restore proof.'), indent=2)+'\n')
    print('PASS 5 sealed-v1 DB copies/10 CLI opens; all old rows equal/schema2 hash/old idleNULL; baseline unchanged')


if __name__ == '__main__':
    main()
