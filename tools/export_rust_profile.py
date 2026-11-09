"""Export pinned inputs for the Rust M1 diagnostic; do not generate wire bytes."""
import argparse
import hashlib
import json
from pathlib import Path


def export(profile_path, error_path, *, ports, idle_seconds=60, max_connections=16):
    profile = json.loads(Path(profile_path).read_text())
    pins = {}
    for name, alias in [('bootstrap', 'bootstrap'), ('wcp_second', 'second')]:
        row = profile['fixtures'][alias]
        p = Path(row['path']).resolve(strict=True)
        data = p.read_bytes()
        if len(data) != row['bytes'] or hashlib.sha256(data).hexdigest() != row['sha256']:
            raise ValueError(f'fixture pin mismatch: {alias}')
        pins[name] = dict(path=str(p), bytes=len(data), sha256=row['sha256'])
    error = Path(error_path).resolve(strict=True)
    data = error.read_bytes()
    if len(data) != 39 or hashlib.sha256(data).hexdigest() != '9aeae7b7937ad0c54d968297c48fffb4d506087f7826b2e9fdde35d84ed3ac19':
        raise ValueError('expected native-tested CP181 result1 fixture')
    pins['login_error'] = dict(path=str(error), bytes=len(data), sha256=hashlib.sha256(data).hexdigest())
    return dict(schema='kr-rust-diagnostic-v1', diagnostic_login_error_only=True,
                bind='127.0.0.1', allowed_peers=['127.0.0.1'], ports=list(ports),
                max_connections=max_connections, idle_timeout_seconds=idle_seconds,
                write_timeout_ms=2000, **pins)


def export_local_lobby(profile_path, error_path, own_body_path, *, ports, idle_seconds=60):
    """Only the measured fixed KRLocal branch; no account authentication."""
    result = export(profile_path, error_path, ports=ports, idle_seconds=idle_seconds)
    profile = json.loads(Path(profile_path).read_text())
    packets = {}
    sequences = dict(auth_A=43, netinfo=46, flags=49, redirect=52, channel=43,
                     ready=46, roomlist_A=49, channel_list=52)
    for alias, sequence in sequences.items():
        row = profile['fixtures'][alias]
        p = Path(row['path']).resolve(strict=True)
        data = p.read_bytes()
        if len(data) != row['bytes'] or hashlib.sha256(data).hexdigest() != row['sha256']:
            raise ValueError(f'fixture pin mismatch: {alias}')
        packets[alias] = dict(fixture=dict(path=str(p), bytes=len(data), sha256=row['sha256']),
                              sequence=sequence, opcode=int.from_bytes(data[5:9], 'little'))
    own = Path(own_body_path).resolve(strict=True)
    data = own.read_bytes()
    if len(data) != 128:
        raise ValueError('expected measured zero-profile KRLocal291 body128')
    packets['own291'] = dict(fixture=dict(path=str(own), bytes=128,
                                        sha256=hashlib.sha256(data).hexdigest()),
                            sequence=None, opcode=291)
    row = profile['fixtures']['player_A']
    template = Path(row['path']).resolve(strict=True)
    raw = template.read_bytes()
    if len(raw) != row['bytes'] or hashlib.sha256(raw).hexdigest() != row['sha256']:
        raise ValueError('own291 header template pin mismatch')
    result['diagnostic_login_error_only'] = False
    result['diagnostic_local_lobby'] = dict(packets=packets,
        own_header=dict(path=str(template), bytes=len(raw), sha256=row['sha256']))
    return result


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--python-profile', type=Path, required=True)
    p.add_argument('--login-error', type=Path, required=True)
    p.add_argument('--port', type=int, action='append', required=True)
    p.add_argument('--out', type=Path, required=True)
    p.add_argument('--local-lobby-own-profile', type=Path,
                   help='Opt in to fixed KRLocal lobby; existing measured body128 input, not accounts')
    a = p.parse_args()
    if not 1 <= len(a.port) <= 32 or len(set(a.port)) != len(a.port) or any(not 1 <= n <= 65535 for n in a.port):
        p.error('one to32 distinct nonzero ports required')
    config = (export_local_lobby(a.python_profile, a.login_error, a.local_lobby_own_profile, ports=a.port)
              if a.local_lobby_own_profile else export(a.python_profile, a.login_error, ports=a.port))
    with a.out.open('x') as f:
        f.write(json.dumps(config, indent=2) + '\n')


if __name__ == '__main__':
    main()
