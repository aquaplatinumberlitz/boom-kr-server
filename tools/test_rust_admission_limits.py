"""Synthetic admission budget checks; held TCP sockets are not native game players."""
import argparse
import copy
import hashlib
import json
import os
from pathlib import Path
import signal
import socket
import subprocess
import sys
import time

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from tools.test_rust_foundation import exact, free_port


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binary', type=Path, required=True)
    parser.add_argument('--config', type=Path, required=True)
    parser.add_argument('--out', type=Path, required=True)
    args = parser.parse_args()
    args.out.mkdir(parents=True, exist_ok=False)
    base = json.loads(args.config.read_text())
    base.pop('diagnostic_two_lobby', None)
    base.pop('diagnostic_local_lobby', None)
    base.update(diagnostic_login_error_only=True, bind='127.0.0.1', idle_timeout_seconds=120,
                max_connections=65, allowed_peers=['127.0.0.'+str(i) for i in range(2, 52)])
    hello = Path(base['bootstrap']['path']).read_bytes()
    pins = [args.binary.resolve(), args.config.resolve(), Path(__file__).resolve(),
            *[Path(base[key]['path']) for key in ['bootstrap', 'wcp_second', 'login_error']],
            *Path('rust/crates').rglob('*.rs'), Path('rust/Cargo.lock')]
    def pin(p):
        return dict(path=str(p.resolve()), bytes=p.stat().st_size, sha256=hashlib.sha256(p.read_bytes()).hexdigest())
    frozen = [pin(p) for p in pins]
    (args.out/'inputs.json').write_text(json.dumps(frozen, indent=2)+'\n')
    cases, sockets = [], []
    proc, log, error = None, None, None
    remaining = []
    def config(name, cfg):
        p = args.out/(name+'.json')
        p.write_text(json.dumps(cfg, indent=2)+'\n')
        return p
    try:
        for name, mutate in [('zero', lambda c: c.update(max_connections=0)),
                             ('overflow', lambda c: c.update(max_connections=1025)),
                             ('no-peers', lambda c: c.update(allowed_peers=[])),
                             ('too-many-peers', lambda c: c.update(allowed_peers=['127.0.0.2']*257))]:
            cfg = copy.deepcopy(base)
            cfg['ports'] = [free_port()]
            mutate(cfg)
            result = subprocess.run([str(args.binary.resolve()), '--config', str(config(name, cfg)),
                                     '--out', str(args.out/name)], capture_output=True, text=True, timeout=5)
            (args.out/(name+'-process.txt')).write_text(result.stdout+result.stderr)
            assert result.returncode != 0 and not (args.out/name).exists()
            with socket.socket() as check:
                check.bind(('127.0.0.1', cfg['ports'][0]))
            cases.append(dict(case=name+'_rejected_before_bind', result='pass'))
        cfg = copy.deepcopy(base)
        cfg['ports'] = [free_port()]
        path = config('positive', cfg)
        log = (args.out/'process.log').open('x')
        proc = subprocess.Popen([str(args.binary.resolve()), '--config', str(path), '--out', str(args.out/'service')],
                                stdout=log, stderr=subprocess.STDOUT, start_new_session=True)
        deadline = time.monotonic()+5
        while time.monotonic() < deadline:
            state = args.out/'service/service-state.json'
            if state.exists() and json.loads(state.read_text())['ready']:
                break
            assert proc.poll() is None
            time.sleep(.01)
        else:
            raise AssertionError('readiness deadline')
        def connect(peer='127.0.0.2'):
            s = socket.socket()
            s.settimeout(3)
            s.bind((peer, 0))
            s.connect(('127.0.0.1', cfg['ports'][0]))
            sockets.append(s)
            return s
        def closed(s):
            try:
                assert s.recv(1) == b''
            except ConnectionResetError:
                pass
        for _ in range(65):
            assert exact(connect(), 15) == hello
        cases.append(dict(case='65_concurrent_held_transports_receive_actual_hello', result='pass'))
        rejected = connect()
        closed(rejected)
        cases.append(dict(case='66th_transport_rejected_at_configured_limit', result='pass'))
        sockets[10].close()
        deadline = time.monotonic()+3
        while time.monotonic() < deadline:
            lines = (args.out/'service/rust-events.jsonl').read_text().splitlines()
            if any(json.loads(line).get('event') == 'closed' for line in lines):
                break
            time.sleep(.01)
        else:
            raise AssertionError('closed transport did not release')
        denied = connect('127.0.0.200')
        closed(denied)
        assert exact(connect(), 15) == hello
        cases.append(dict(case='released_permit_recovers_and_disallowed_peer_does_not_consume_it', result='pass'))
        for s in sockets:
            s.close()
        os.killpg(proc.pid, signal.SIGTERM)
        assert proc.wait(timeout=8) == 0
        events = [json.loads(line) for line in (args.out/'service/rust-events.jsonl').read_text().splitlines()]
        assert sum(e.get('event') == 'accepted' for e in events) == 66
        assert any(e.get('reason') == 'connection_limit' for e in events)
        assert any(e.get('reason') == 'peer_not_allowed' for e in events)
        assert events[-1]['event'] == 'stopped' and events[-1]['error'] is None
        cases.append(dict(case='clean_shutdown_and_rejection_reasons_observed', result='pass'))
        # Independently test that an allowlist larger than the pinned actor set
        # grants network access but cannot select an unmapped diagnostic identity.
        pair = json.loads(args.config.read_text())
        old_ports = list(pair['ports'])
        fresh = set()
        while len(fresh) < len(old_ports):
            fresh.add(free_port())
        mapping = dict(zip(old_ports, sorted(fresh)))
        pair.update(bind='127.0.0.1', allowed_peers=['127.0.0.2','127.0.0.3','127.0.0.9'],
                    ports=sorted(fresh), max_connections=4)
        two = pair['diagnostic_two_lobby']
        two['login_port'] = mapping[two['login_port']]
        two['channel_port'] = mapping[two['channel_port']]
        two['wcp_ports'] = [mapping[p] for p in two['wcp_ports']]
        two['login_candidate_ports'] = [mapping[p] for p in two.get('login_candidate_ports', [])]
        for i, actor in enumerate(two['actors']):
            actor['peer'] = '127.0.0.'+str(i+2)
        log.close()
        log = (args.out/'profile-process.log').open('x')
        proc = subprocess.Popen([str(args.binary.resolve()), '--config', str(config('profile', pair)),
                                 '--out', str(args.out/'profile-service')], stdout=log,
                                stderr=subprocess.STDOUT, start_new_session=True)
        deadline = time.monotonic()+5
        while time.monotonic() < deadline:
            state = args.out/'profile-service/service-state.json'
            if state.exists() and json.loads(state.read_text())['ready']:
                break
            assert proc.poll() is None
            time.sleep(.01)
        else:
            raise AssertionError('profile mode with extra allowlist source did not start')
        cfg = pair
        unknown = connect('127.0.0.9')
        closed(unknown)
        assert exact(connect('127.0.0.2'), 15) == hello
        cases.append(dict(case='three_allowlisted_sources_two_profiles_unmapped_source_gets_no_identity', result='pass'))
        for s in sockets:
            s.close()
        os.killpg(proc.pid, signal.SIGTERM)
        assert proc.wait(timeout=8) == 0
        events = [json.loads(line) for line in (args.out/'profile-service/rust-events.jsonl').read_text().splitlines()]
        assert not any(e.get('event') == 'diagnostic_binding' for e in events)
        assert any(e.get('event') == 'closed' and e.get('reason') == 'unmapped diagnostic actor' for e in events)
        cases.append(dict(case='network_allowlist_does_not_allocate_actor_profile_or_membership', result='pass'))
    except Exception as exc:
        error = repr(exc)
    finally:
        for s in sockets:
            s.close()
        if proc is not None and proc.poll() is None:
            os.killpg(proc.pid, signal.SIGTERM)
            try:
                proc.wait(timeout=8)
            except subprocess.TimeoutExpired:
                os.killpg(proc.pid, signal.SIGKILL)
                proc.wait(timeout=3)
        if log is not None:
            log.close()
        if proc is not None and Path('/proc', str(proc.pid)).exists():
            remaining.append(proc.pid)
        unchanged = all(pin(Path(item['path'])) == item for item in frozen)
        result = dict(result='pass' if error is None and unchanged and not remaining else 'fail', cases=cases,
                      error=error, inputs_unchanged=unchanged, remaining_pid=remaining,
                      limits='Synthetic hello/admission/resource lifecycle only; not65players,50userload,nativeKR,authenticationorUDPproof.')
        (args.out/'verification.json').write_text(json.dumps(result, indent=2)+'\n')
    assert result['result'] == 'pass', result
    print('PASS', len(cases), 'synthetic admission cases; no native or player-load claim')


if __name__ == '__main__':
    main()
