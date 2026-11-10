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


def export_two_lobby(profile_path, error_path, own_body_a, own_body_b, *, ports,
                     bind, peers, redirect_path, idle_seconds=120):
    """Two pinned lab actors, explicit destination ports; never authentication."""
    import ipaddress
    expected_ports = {3838, 3848, 3858}
    if not expected_ports.issubset(ports) or len(peers) != 2 or len(set(peers)) != 2:
        raise ValueError('two distinct peers and ports3838/3848/3858 required')
    for ip in [bind, *peers]:
        value = ipaddress.IPv4Address(ip)
        if not (value.is_loopback or value in ipaddress.IPv4Network('10.0.0.0/8')
                or value in ipaddress.IPv4Network('172.16.0.0/12')
                or value in ipaddress.IPv4Network('192.168.0.0/16')):
            raise ValueError('diagnostic endpoint must be loopback or RFC1918')
    result = export(profile_path, error_path, ports=ports, idle_seconds=idle_seconds)
    source = json.loads(Path(profile_path).read_text())
    actors = []
    redirect = Path(redirect_path).resolve(strict=True)
    raw_redirect = redirect.read_bytes()
    if len(raw_redirect) != 52 or int.from_bytes(raw_redirect[5:9], 'little') != 7:
        raise ValueError('expected measured redirect7 frame52/sequence52')
    for role, identity, name, own, peer in [('A', 1, 'KRLocal', own_body_a, peers[0]),
                                          ('B', 2, 'KRGuest', own_body_b, peers[1])]:
        cfg = export_local_lobby(profile_path, error_path, own, ports=ports,
                                 idle_seconds=idle_seconds)['diagnostic_local_lobby']
        row = source['fixtures']['auth_' + role]
        p = Path(row['path']).resolve(strict=True)
        raw = p.read_bytes()
        if len(raw) != row['bytes'] or hashlib.sha256(raw).hexdigest() != row['sha256']:
            raise ValueError('auth fixture pin mismatch')
        cfg['packets']['auth_A'] = dict(fixture=dict(path=str(p), bytes=len(raw),
            sha256=row['sha256']), sequence=43, opcode=4)
        cfg['packets']['redirect'] = dict(fixture=dict(path=str(redirect),
            bytes=len(raw_redirect), sha256=hashlib.sha256(raw_redirect).hexdigest()),
            sequence=52, opcode=7)
        actors.append(dict(peer=peer, identity=identity, name=name, profile=cfg))
    result.update(diagnostic_login_error_only=False, bind=bind, allowed_peers=list(peers),
                  diagnostic_two_lobby=dict(wcp_ports=list(ports), login_port=3838, login_candidate_ports=list(ports),
                      channel_port=3858, actors=actors))
    return result


def add_fixed_room(config, profile_path, listing_path, title):
    """Pin measured two-player room templates; builders remain in Rust, not a wire oracle."""
    if 'diagnostic_two_lobby' not in config or not 1 <= len(title) <= 32 or not title.isascii() or not all(32 <= ord(c) <= 126 for c in title):
        raise ValueError('fixed room requires two-actor mode and bounded ASCII title')
    profile = json.loads(Path(profile_path).read_text())
    packets = {}
    for alias in ['create', 'join', 'player_A', 'player_B', 'table', 'map', 'floor', 'start', 'go', 'play', 'peer']:
        row = profile['fixtures'][alias]
        p = Path(row['path']).resolve(strict=True)
        raw = p.read_bytes()
        if len(raw) != row['bytes'] or hashlib.sha256(raw).hexdigest() != row['sha256']:
            raise ValueError('fixed room fixture mismatch: '+alias)
        packets[alias] = dict(fixture=dict(path=str(p), bytes=len(raw), sha256=row['sha256']),
                              sequence=row['sequence'], opcode=int.from_bytes(raw[5:9], 'little'))
    p = Path(listing_path).resolve(strict=True)
    raw = p.read_bytes()
    packets['listing'] = dict(fixture=dict(path=str(p), bytes=len(raw), sha256=hashlib.sha256(raw).hexdigest()), sequence=49, opcode=91)
    config['diagnostic_two_lobby']['fixed_room'] = dict(title=title, packets=packets)
    if profile.get('bomb_terrain') is not None:
        config['diagnostic_two_lobby']['fixed_room']['terrain'] = profile['bomb_terrain']
    if profile.get('result_freshness')=={'mode':'post_blast_untrapped_drained'} and profile.get('death_lifecycle')=={'mode':'current_trap'}:
        battle={}
        for alias,opcode in [('rank',175),('rank_Awin',175),('rank_all_dead',175),('return',180)]:
            row=profile['fixtures'][alias];p=Path(row['path']).resolve(strict=True);raw=p.read_bytes()
            if len(raw)!=row['bytes'] or hashlib.sha256(raw).hexdigest()!=row['sha256']:raise ValueError('Battle fixture pin mismatch: '+alias)
            battle[alias]=dict(fixture=dict(path=str(p),bytes=len(raw),sha256=row['sha256']),sequence=row['sequence'],opcode=opcode)
        config['diagnostic_two_lobby']['fixed_room']['battle']=dict(mode='post_blast_untrapped_drained',packets=battle)
    return config


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--python-profile', type=Path, required=True)
    p.add_argument('--login-error', type=Path, required=True)
    p.add_argument('--port', type=int, action='append', required=True)
    p.add_argument('--out', type=Path, required=True)
    p.add_argument('--local-lobby-own-profile', type=Path,
                   help='Opt in to fixed KRLocal lobby; existing measured body128 input, not accounts')
    p.add_argument('--two-lobby-own-profile-a', type=Path)
    p.add_argument('--two-lobby-own-profile-b', type=Path)
    p.add_argument('--two-lobby-redirect', type=Path)
    p.add_argument('--diagnostic-bind', default='127.0.0.1')
    p.add_argument('--diagnostic-peer-a', default='127.0.0.2')
    p.add_argument('--diagnostic-peer-b', default='127.0.0.3')
    p.add_argument('--fixed-room-python-profile', type=Path)
    p.add_argument('--fixed-room-listing', type=Path)
    p.add_argument('--fixed-room-title')
    a = p.parse_args()
    if not 1 <= len(a.port) <= 32 or len(set(a.port)) != len(a.port) or any(not 1 <= n <= 65535 for n in a.port):
        p.error('one to32 distinct nonzero ports required')
    two = [a.two_lobby_own_profile_a, a.two_lobby_own_profile_b, a.two_lobby_redirect]
    if any(two):
        if not all(two) or a.local_lobby_own_profile:
            p.error('two-lobby requires A/B profiles and redirect, separately from single-lobby')
        config = export_two_lobby(a.python_profile, a.login_error,
            a.two_lobby_own_profile_a, a.two_lobby_own_profile_b, ports=a.port,
            bind=a.diagnostic_bind, peers=[a.diagnostic_peer_a, a.diagnostic_peer_b],
            redirect_path=a.two_lobby_redirect)
    else:
        config = (export_local_lobby(a.python_profile, a.login_error, a.local_lobby_own_profile, ports=a.port)
                  if a.local_lobby_own_profile else export(a.python_profile, a.login_error, ports=a.port))
    room = [a.fixed_room_python_profile, a.fixed_room_listing, a.fixed_room_title]
    if any(room):
        if not all(room) or not all(two):
            p.error('fixed room requires two-lobby and room profile/listing/title')
        config = add_fixed_room(config, *room)
    with a.out.open('x') as f:
        f.write(json.dumps(config, indent=2) + '\n')


if __name__ == '__main__':
    main()
