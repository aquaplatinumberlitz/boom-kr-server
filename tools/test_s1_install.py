#!/usr/bin/env python3
"""Build the S1 Docker image and play one two-player round in an offline container."""
import argparse
import json
import os
from pathlib import Path
import secrets
import subprocess
import tempfile
import time

from test_kr_pack_install import test_bundle


def run(*args, **kwargs):
    return subprocess.run(args, text=True, check=True, **kwargs).stdout


def load_inputs(invite, route, replay_path, accounts, *, login=3857, channel=3888, active=2):
    window = dict(duration_ns=1_000_000_000, packets=[[83_000_000 * i, 24] for i in range(12)])
    replay = dict(schema='kr-m8-replay-v1', checkpoint='CP287', windows=[window] * 8)
    config = dict(s1=dict(invite=invite, **route), login_port=login, channel_port=channel,
                  replay=replay_path, rooms=[accounts], ramp_ms=225, active_seconds=active,
                  minimum_run_seconds=0, minimum_rounds=1, delivery_p99_ms_max=500,
                  delivery_loss_percent_max=5)
    return replay, config


def accounts_for_round():
    return [dict(login=name, name=name, password=secrets.token_hex(16))
            for name in ('AlphaOne', 'BetaTwo')]


def account_request(account):
    return dict(actor='s1-smoke', reason='Isolated two-player round', command=dict(op='create', **account))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    source = parser.add_mutually_exclusive_group(required=True)
    source.add_argument('--binaries', type=Path, help='Build a temporary bundle with a synthetic fixture')
    source.add_argument('--bundle', type=Path, help='Use an existing Linux S1 bundle')
    parser.add_argument('--game', type=Path, help='Game path for --bundle')
    parser.add_argument('--public-lookup', action='store_true',
                        help='Use public iroh discovery instead of the isolated loopback route')
    args = parser.parse_args()
    if bool(args.game) != bool(args.bundle):
        parser.error('--bundle and --game must be used together')
    ident = 'kre-s1-smoke-' + secrets.token_hex(4)
    with tempfile.TemporaryDirectory(prefix=ident) as directory:
        root = Path(directory)
        root.chmod(0o755)  # The container user must be able to read the fake game pack.
        if args.binaries:
            bundle, _ = test_bundle(root, args.binaries.resolve(), self_host=True)
            game = root / 'game'
        else:
            bundle = args.bundle.resolve(strict=True)
            game = args.game.resolve(strict=True)
        two = json.loads((bundle / 'config.template.json').read_bytes())['diagnostic_two_lobby']
        docker = ('docker', 'exec', ident)
        try:
            run('docker', 'build', '-t', ident, '-f', str(bundle / 'deploy/Dockerfile'), str(bundle))
            net = ['--network', 'bridge'] if args.public_lookup else ['--network', 'none', '-e', 'BOOM_OFFLINE=1']
            run('docker', 'run', '-d', '--name', ident, *net, '--read-only',
                '--cap-drop=ALL', '--security-opt=no-new-privileges', '--tmpfs', '/tmp',
                '--mount', f'type=bind,source={game},target=/game,readonly',
                ident)
            for _ in range(100):
                log = run('docker', 'logs', ident, stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
                if ('Boom Server ready: true' in log
                        and (not args.public_lookup or 'Boom Server relay connected: true' in log)):
                    break
                time.sleep(.2)
            else:
                raise RuntimeError(log)
            invite = run(*docker, '/opt/boom/bin/boom-server', 'invite', capture_output=True).strip()
            accounts = accounts_for_round()
            for account in accounts:
                run('docker', 'exec', '-i', ident, '/opt/boom/bin/kr-admin', '/state/accounts.sqlite',
                    input=json.dumps(account_request(account)), stdout=subprocess.DEVNULL)
            route = {} if args.public_lookup else dict(hint='127.0.0.1:41600', local_only=True)
            replay, config = load_inputs(invite, route,
                                         '/tmp/replay.json', accounts, login=two['login_port'], channel=two['channel_port'])
            for path, content in [('replay', replay), ('load', config)]:
                run('docker', 'exec', '-i', ident, 'sh', '-c', 'umask 077; cat > "$1"', 'sh',
                    '/tmp/' + path + '.json', input=json.dumps(content))
            results = run(*docker, '/opt/boom/bin/kr-load', '/tmp/load.json', capture_output=True)
            events = [json.loads(line) for line in results.splitlines()]
            assert any(row['event'] == 'load_complete' and row['players'] == 2 for row in events), events
            for row in events:
                if row['event'] in {'round_complete', 'delivery', 'load_complete'}:
                    print(json.dumps(row))
            backup = run(*docker, '/opt/boom/bin/boom-server', 'backup', capture_output=True).strip()
            run(*docker, '/opt/boom/bin/kr-admin', 'verify-backup', backup)
            run('docker', 'stop', '-t', '30', ident)
            assert run('docker', 'inspect', '-f', '{{.State.ExitCode}}', ident, capture_output=True).strip() == '0'
            run('docker', 'start', ident)
            for _ in range(100):
                logs = run('docker', 'logs', ident, stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
                if logs.count('Boom Server ready: true') >= 2:
                    break
                time.sleep(.2)
            else:
                raise RuntimeError(logs)
            assert run(*docker, '/opt/boom/bin/boom-server', 'invite', capture_output=True).strip() == invite
            run('docker', 'stop', '-t', '30', ident)
            route_name = 'public discovery' if args.public_lookup else 'offline loopback'
            print(f'S1 Docker ({route_name}): persistent pinned invite, one two-player round, verified backup, clean shutdown')
        except subprocess.CalledProcessError as error:
            if error.stderr:
                print(error.stderr.rstrip())
            raise
        finally:
            # Isolated synthetic fixtures only. Never stop or prune other containers/images.
            for command in [('docker', 'rm', '-fv', ident), ('docker', 'image', 'rm', ident)]:
                subprocess.run(command, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, check=False)


if __name__ == '__main__':
    os.umask(0o022)
    main()
