"""Retain M5 library/real DB tests and replay independently pinned native wire bodies.

Oracle manifest is local evidence, with case IDs, pinned source files, kind/peer
and expected decoder fields. This tool never generates a wire encoder oracle.
No native client is launched and no listener is changed by this acceptance.
"""
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


def pin(path):
    return dict(path=str(path), bytes=path.stat().st_size, sha256=sha(path))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--out', type=Path, required=True)
    parser.add_argument('--oracle-manifest', type=Path, required=True)
    args = parser.parse_args()
    args.out.mkdir(parents=True, exist_ok=False, mode=0o700)
    root = args.out.resolve()
    os.chmod(root, 0o700)
    raw = root / 'raw'
    raw.mkdir(mode=0o700)
    oracle = json.loads(args.oracle_manifest.read_text())
    assert oracle['schema'] == 'kre-account-bridge-wire-oracle-v1'
    assert 1 <= len(oracle['cases']) <= 128
    native_files = []
    inputs = []
    for case in oracle['cases']:
        path = Path(case['body']['path'])
        assert pin(path) == case['body'], 'native body pin mismatch'
        native_files.append(path)
        inputs.append(dict(kind=case['kind'], peer=case['peer'], body_hex=path.read_bytes().hex()))
    command = ['cargo', 'test', '--manifest-path', 'rust/Cargo.toml', '-p', 'kr-service',
               '--lib', '--no-run', '--message-format=json', '--locked']
    build = subprocess.run(command, text=True, capture_output=True)
    (root / 'build.jsonl').write_text(build.stdout)
    (root / 'build.stderr').write_text(build.stderr)
    build.check_returncode()
    artifacts = [json.loads(line) for line in build.stdout.splitlines()]
    binaries = [Path(x['executable']) for x in artifacts if x.get('reason') == 'compiler-artifact'
                and x.get('executable') and x.get('target', {}).get('name') == 'kr_service']
    assert len(binaries) == 1
    binary = binaries[0]
    with (root / 'build-wire.log').open('wb') as log:
        subprocess.run(['cargo', 'build', '--manifest-path', 'rust/Cargo.toml', '-p', 'kr-service',
                        '--example', 'check_account_bridge_wire', '--locked'],
                       stdout=log, stderr=subprocess.STDOUT, check=True)
    wire_binary = Path('rust/target/debug/examples/check_account_bridge_wire')
    source = [Path('rust/Cargo.toml'), Path('rust/Cargo.lock'), Path('rust/rust-toolchain.toml'),
              Path(__file__), binary, wire_binary, args.oracle_manifest, *native_files,
              *sorted(Path('rust/crates').rglob('*.rs')),
              *sorted(Path('rust/crates').rglob('*.sql')),
              *sorted(Path('rust/crates').rglob('Cargo.toml'))]
    source = list(dict.fromkeys(p for p in source if p.is_file()))
    frozen = root / 'frozen'
    frozen.mkdir(mode=0o700)
    pins = []
    for index, path in enumerate(source):
        retained = frozen / f'{index:03d}-{path.name}'
        shutil.copyfile(path, retained)
        pins.append(dict(**pin(path), retained_path=str(retained)))
    recipe = [str(binary.resolve()), 'account_bridge::tests::', '--test-threads=1', '--nocapture']
    (root / 'preflight.json').write_text(json.dumps(dict(files=pins, command=recipe,
        environment=dict(KR_ACCOUNT_BRIDGE_TEST_RAW=str(raw))), indent=2)+'\n')
    started = time.monotonic()
    with (root / 'process.log').open('wb') as log:
        process = subprocess.Popen(recipe, env=dict(os.environ, KR_ACCOUNT_BRIDGE_TEST_RAW=str(raw)),
                                   stdout=log, stderr=subprocess.STDOUT)
        try:
            code = process.wait(timeout=180)
        finally:
            if process.poll() is None:
                process.kill()
                process.wait()
    wire = subprocess.run([str(wire_binary.resolve())], input=json.dumps(inputs),
                          capture_output=True, text=True, timeout=10)
    (root / 'wire-input.json').write_text(json.dumps(inputs, indent=2)+'\n')
    (root / 'wire-output.json').write_text(wire.stdout)
    (root / 'wire-stderr.txt').write_text(wire.stderr)
    wire_ok = wire.returncode == 0
    outputs = json.loads(wire.stdout) if wire_ok else []
    oracle_results = [dict(id=c['id'], match=out == c['expected'])
                      for c, out in zip(oracle['cases'], outputs)]
    wire_ok = wire_ok and len(outputs) == len(oracle['cases']) and all(x['match'] for x in oracle_results)
    unchanged = all(p.stat().st_size == spec['bytes'] and sha(p) == spec['sha256']
                    for p, spec in zip(source, pins))
    databases = []
    for db in sorted(raw.glob('*/accounts.sqlite')):
        with sqlite3.connect(db.as_uri()+'?mode=ro', uri=True) as conn:
            assert conn.execute('PRAGMA integrity_check').fetchone() == ('ok',)
            assert conn.execute('PRAGMA foreign_key_check').fetchall() == []
            assert conn.execute('SELECT count(*) FROM sqlite_schema WHERE type="trigger"').fetchone() == (0,)
            databases.append(dict(path=str(db), accounts=conn.execute('SELECT count(*) FROM accounts').fetchone()[0],
                players=conn.execute('SELECT count(*) FROM players').fetchone()[0],
                sessions=conn.execute('SELECT count(*) FROM sessions').fetchone()[0],
                schema=conn.execute('SELECT version FROM schema_migrations').fetchall()))
        assert b'bridge-test-password' not in db.read_bytes()
    text = (root / 'process.log').read_text(errors='replace')
    ok = code == 0 and unchanged and len(databases) == 6 and '7 passed; 0 failed' in text and wire_ok
    evidence = [pin(p) for p in sorted(root.rglob('*')) if p.is_file()]
    report = dict(result='pass' if ok else 'fail', duration_s=time.monotonic()-started,
        returncode=code, pid=process.pid, remaining_pid=None if process.poll() is not None else process.pid,
        source_frozen=unchanged, binary_sha256=sha(binary), wire_binary_sha256=sha(wire_binary),
        databases=databases, oracle_results=oracle_results, evidence=evidence,
        limits='Library/backend design and offline retained-native parser replay only; not live KR on account bridge, full authentication, serving integration, restart fence, native wrong-owner/expiry/revoke or M5 closure.')
    (root / 'verification.json').write_text(json.dumps(report, indent=2)+'\n')
    assert ok, 'bridge acceptance failed; raw/process log retained'
    print(f'PASS 7 bridge cases/6 retained DBs/{len(oracle_results)} independent native bodies/source frozen/PID gone; no serving/native M5 claim')


if __name__ == '__main__':
    main()
