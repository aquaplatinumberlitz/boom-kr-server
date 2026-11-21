"""Lượt chạy bên trong container/namespace, chỉ sử dụng gói và pack mount chỉ đọc."""
import json
import os
from pathlib import Path
import signal
import sqlite3
import subprocess
import sys
import time

BUNDLE = Path('/opt/kr')
GAME = Path('/game')
OUTPUT = Path('/result')
STATE = OUTPUT / 'state'


def main():
    manifest_hash = os.environ['KR_MANIFEST_SHA256']
    assert not Path('/home/admin/boom').exists()
    assert os.statvfs(GAME).f_flag & os.ST_RDONLY
    assert os.statvfs(BUNDLE).f_flag & os.ST_RDONLY
    assert not STATE.exists()
    bundle = [sys.executable, str(BUNDLE / 'kr-package.py')]
    options = ['--manifest-sha256', manifest_hash, '--state', str(STATE)]
    game = ['--game-dir', str(GAME)]

    def run(*argv, **kwargs):
        result = subprocess.run(argv, text=True, capture_output=True, timeout=30, **kwargs)
        if result.returncode:
            raise RuntimeError(f'{argv[:3]}: {result.stderr.strip()}')
        return result.stdout

    def install(command, *more):
        return run(*bundle, command, *options, *more)

    run(*bundle, 'verify', '--manifest-sha256', manifest_hash)
    install('init-db')
    for bad, expected in [('missing', 'Thiếu pack KR 00967'), ('different', 'Pack không khớp KR 00967')]:
        directory = OUTPUT / bad
        directory.mkdir()
        if bad == 'different':
            for stem in ('FxMap', 'FxContents', 'fx'):
                for ext in ('idx', 'idd'):
                    (directory / f'{stem}.{ext}').write_bytes(b'wrong version')
        result = subprocess.run([*bundle, 'export', *options, '--game-dir', str(directory)],
                                text=True, capture_output=True, timeout=10)
        assert result.returncode == 1 and expected in result.stderr, result.stderr
        for member in directory.iterdir():
            member.unlink()
        directory.rmdir()
    fresh = json.loads(install('export', *game))
    again = json.loads(install('export', *game))
    assert not fresh['reused'] and again['reused'] and fresh['cache'] == again['cache']
    cache = Path(fresh['cache'])
    # First use rejects a damaged cache. Restore the test mutation, then launch.
    tables = cache / 'tables.json'
    original = tables.read_bytes()
    tables.write_bytes(b'{}')
    invalid = subprocess.run([*bundle, 'export', *options, *game], text=True, capture_output=True, timeout=10)
    assert invalid.returncode == 1 and 'File cache sai hash' in invalid.stderr
    tables.write_bytes(original)
    if os.environ.get('KR_FAKE_PACK'):
        profile = json.loads((cache / 'SyntheticOne.json').read_bytes())
        assert profile['spawns'] == [[1, 1], [13, 11]]
        assert json.loads((cache / 'SyntheticTwo.json').read_bytes())['spawns'][0] == [2, 1]
        rules = json.loads((cache / 'SyntheticOne.rules.json').read_bytes())
        assert rules['terrain']['kinds'][26] == 2 and sum(rules['terrain']['kinds']) == 2
        assert profile['map_code'] == 1
    pkg = json.loads((BUNDLE / 'config.template.json').read_text())
    two = pkg['diagnostic_two_lobby']
    relay = pkg['authenticated_relay']['advertise']
    tls = two['account_admission']['prelogin_tls']['bind']
    # A deterministic synthetic traffic cadence; shape is compatible with
    # kr-load's bounded replay reader. It makes no CP287 measurement claim.
    replay = OUTPUT / 'replay.json'
    window = dict(duration_ns=1_000_000_000, packets=[[83_000_000 * i, 24] for i in range(12)])
    replay.write_text(json.dumps(dict(schema='kr-m8-replay-v1', checkpoint='CP287', windows=[window] * 8)))
    accounts = [dict(login=name, name=name, password='synthetic-test-password') for name in ('AlphaOne', 'BetaTwo')]
    admin = str(BUNDLE / 'bin/kr-admin')
    db = str(STATE / 'accounts.sqlite')
    for account in accounts:
        run(admin, db, input=json.dumps(dict(actor='pack-smoke', reason='Synthetic TLS match',
                                             command=dict(op='create', **account))))
    cert = STATE / 'prelogin_tls-identity/certificate.der'
    load_config = OUTPUT / 'load.json'
    fd = os.open(load_config, os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600)
    with os.fdopen(fd, 'w') as file:
        json.dump(dict(server=tls, relay=relay, certificate=str(cert), login_port=two['login_port'],
                       channel_port=two['channel_port'], replay=str(replay), rooms=[accounts],
                       ramp_ms=225, active_seconds=2, minimum_run_seconds=0, minimum_rounds=1,
                       delivery_p99_ms_max=500, delivery_loss_percent_max=0), file)
    with (OUTPUT / 'service.log').open('x') as log:
        service = subprocess.Popen([*bundle, 'run', *options, *game], stdout=log, stderr=subprocess.STDOUT)
        try:
            deadline = time.monotonic() + 25
            while time.monotonic() < deadline:
                ready = list(STATE.glob('runs/*/service/service-state.json'))
                if ready and json.loads(ready[0].read_text()).get('ready'):
                    break
                assert service.poll() is None, (OUTPUT / 'service.log').read_text()
                time.sleep(.05)
            else:
                raise TimeoutError('Server không sẵn sàng')
            run(admin, 'health', str(ready[0].parent))
            # A successful kr-load exit also checks return to lobby and delivery thresholds.
            with (OUTPUT / 'load.log').open('x') as load_log:
                subprocess.run([str(BUNDLE / 'bin/kr-load'), str(load_config)], stdout=load_log,
                               stderr=subprocess.STDOUT, timeout=60, check=True)
            events = [json.loads(line) for line in (OUTPUT / 'load.log').read_text().splitlines()]
            assert sum(event['event'] == 'round_returned' for event in events) == 1
            snapshot = OUTPUT / 'smoke.sqlite'
            run(admin, 'backup', db, str(snapshot))
            run(admin, 'verify-backup', str(snapshot))
            with sqlite3.connect(snapshot) as connection:
                matches = connection.execute('SELECT state, map_code, map_name FROM matches').fetchall()
                assert len(matches) == 1 and matches[0][0] == 'complete'
                _, map_code, map_name = matches[0]
                if os.environ.get('KR_FAKE_PACK'):
                    assert (map_code, map_name) == (1, 'SyntheticOne')
                assert connection.execute('SELECT count(*), sum(reward) FROM match_players').fetchone() == (2, 0)
                players = connection.execute('SELECT count(*) FROM accounts').fetchone()[0]
                assert players == 2
            service.send_signal(signal.SIGTERM)
            assert service.wait(timeout=12) == 0
            assert json.loads(ready[0].read_text())['stopped']
        finally:
            if service.poll() is None:
                service.kill()
                service.wait(timeout=5)
    result = dict(manifest_sha256=manifest_hash, pack_cache=cache.name, export=True, cache_reused=True,
                  pack_missing_rejected=True, pack_mismatch_rejected=True, cache_tamper_rejected=True,
                  tls_players=players, rounds=1, map_code=map_code, map=map_name, backup_verified=True, graceful_stop=True,
                  scope='Synthetic kr-load in a filesystem and network namespace; no native client.')
    (OUTPUT / 'verification.json').write_text(json.dumps(result, indent=2) + '\n')
    print(json.dumps(result))


if __name__ == '__main__':
    main()
