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


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--python-profile', type=Path, required=True)
    p.add_argument('--login-error', type=Path, required=True)
    p.add_argument('--port', type=int, action='append', required=True)
    p.add_argument('--out', type=Path, required=True)
    a = p.parse_args()
    if not 1 <= len(a.port) <= 32 or len(set(a.port)) != len(a.port) or any(not 1 <= n <= 65535 for n in a.port):
        p.error('one to32 distinct nonzero ports required')
    config = export(a.python_profile, a.login_error, ports=a.port)
    with a.out.open('x') as f:
        f.write(json.dumps(config, indent=2) + '\n')


if __name__ == '__main__':
    main()
