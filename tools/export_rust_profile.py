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


def expand_four_startup(config, derived_dir, peers):
    """Explicit CP200 four-role startup profile. Generated input is not an oracle.

    Reuse the measured local zero-profile builders already in this repository;
    keep the two-player exporter unchanged and disable unported four Battle.
    """
    import copy
    import ipaddress
    from server import four_player_profile as four
    from server import probe_codec as codec
    config = copy.deepcopy(config)
    two = config.get('diagnostic_two_lobby', {})
    if len(two.get('actors', [])) != 2 or 'fixed_room' not in two:
        raise ValueError('four startup requires a complete two-role fixed-room input')
    if len(peers) != 4 or len(set(peers)) != 4:
        raise ValueError('four distinct diagnostic peers required')
    for address in [config['bind'], *peers]:
        ip = ipaddress.IPv4Address(address)
        if not (ip.is_loopback or any(ip in ipaddress.IPv4Network(net) for net in
                ['10.0.0.0/8', '172.16.0.0/12', '192.168.0.0/16'])):
            raise ValueError('four startup endpoints must be loopback or RFC1918')
    derived_dir = Path(derived_dir).resolve()
    if not derived_dir.is_relative_to(Path(__file__).resolve().parents[1] / 'local'):
        raise ValueError('derived wire inputs must stay under this repository local/')

    def pinned(row):
        path = Path(row['fixture']['path'])
        raw = path.read_bytes()
        if len(raw) != row['fixture']['bytes'] or hashlib.sha256(raw).hexdigest() != row['fixture']['sha256']:
            raise ValueError('four startup source fixture pin mismatch')
        return raw

    room = two['fixed_room']
    common = two['actors'][0]['profile']
    player_template = room['packets']['player_B']
    player_raw = pinned(player_template)
    player = codec.frame_body(player_raw, player_template['sequence'])
    auth_template = common['packets']['auth_A']
    auth = codec.frame_body(pinned(auth_template), auth_template['sequence'])
    derived_dir.mkdir(parents=True, exist_ok=False)

    def pin(name, data):
        path = derived_dir / (name + '.bin')
        path.write_bytes(data)
        return dict(path=str(path), bytes=len(data), sha256=hashlib.sha256(data).hexdigest())

    def packet(name, template, body, sequence=None):
        seq = template['sequence'] if sequence is None else sequence
        raw = codec.encode(pinned(template)[:37], template['opcode'], body, seq)
        return dict(fixture=pin(name, raw), sequence=seq, opcode=template['opcode'])

    actors = []
    for role, ip in zip(four.ROLES, peers):
        member = four.MEMBERS[role]
        profile = copy.deepcopy(common)
        body = four.player_body(player, member, chief=role == 'A')
        header = packet(role + '-player106', player_template, body, 58)
        profile['own_header'] = header['fixture']
        profile['packets']['auth_A'] = packet(role + '-auth', auth_template, four.auth_body(auth, member))
        profile['packets']['own291'] = dict(fixture=pin(role + '-own291', four.own_body(body)), sequence=None, opcode=291)
        room['packets']['player_' + role] = header
        actors.append(dict(peer=ip, identity=member['identity'], name=member['name'], profile=profile))
    start = room['packets']['start']
    room['packets']['start'] = packet('start4', start, four.start_body(codec.frame_body(pinned(start), start['sequence'])))
    peer = room['packets']['peer']
    endpoints = {role: (int(ipaddress.IPv4Address(ip)), 32001 + index)
                 for index, (role, ip) in enumerate(zip(four.ROLES, peers))}
    # Rust preserves this measured prefix and builds endpoints from actual99;
    # these bounded template tuples do not choose a client's runtime UDP port.
    room['packets']['peer'] = packet('peer4', peer, four.peer_body(codec.frame_body(pinned(peer), peer['sequence']), 12346, 'A', endpoints))
    room.pop('battle', None)
    room.pop('terrain', None)
    two['actors'] = actors
    config['allowed_peers'] = list(peers)
    config['max_connections'] = max(24, config['max_connections'])
    source = Path(four.__file__)
    (derived_dir / 'provenance.json').write_text(json.dumps(dict(
        scope='Four diagnostic startup/room/loading only; four Battle/terrain policies disabled, no native acceptance from generation.',
        builder_path=str(source), builder_sha256=hashlib.sha256(source.read_bytes()).hexdigest(),
        contract='docs/PROTOCOL.md#four-session-native',
        limits='Operator-IP identities, fixed alternating teams/zero profiles/known ASCII5-7 names; not authentication or general capacity.'), indent=2) + '\n')
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
    p.add_argument('--four-startup-derived-dir', type=Path, help='Explicit startup-only four-role expansion; creates a new directory under local/')
    p.add_argument('--four-startup-peer-c')
    p.add_argument('--four-startup-peer-d')
    p.add_argument('--four-owner-bombs', action='store_true', help='Opt-in measured four-owner capacity1/2/1/1, power1, fuse6; no terrain/life/results')
    p.add_argument('--four-life-reports', action='store_true', help='Opt-in four 17/19/21 reports and three-recipient170 only; requires four-owner-bombs, no results')
    p.add_argument('--four-results', action='store_true', help='Opt-in fixed four 2v2 drained results/reset; requires four-life-reports')
    p.add_argument('--four-abort-room', action='store_true', help='Opt-in scoped active four-member EOF unranked return; requires four-results')
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
    four = [a.four_startup_derived_dir, a.four_startup_peer_c, a.four_startup_peer_d]
    if any(four):
        if not all(four) or not all(two) or not all(room):
            p.error('four startup requires two-lobby/fixed-room inputs plus derived directory and C/D peers')
        config = expand_four_startup(config, a.four_startup_derived_dir,
            [a.diagnostic_peer_a, a.diagnostic_peer_b, a.four_startup_peer_c, a.four_startup_peer_d])
    if a.four_owner_bombs:
        if not all(four):
            p.error('four-owner bombs requires the explicit four-startup profile')
        config['diagnostic_two_lobby']['fixed_room']['four_owner_bombs'] = True
    if a.four_life_reports:
        if not a.four_owner_bombs:
            p.error('four-life reports requires four-owner-bombs')
        config['diagnostic_two_lobby']['fixed_room']['four_life_reports'] = True
    if a.four_results:
        if not a.four_life_reports:
            p.error('four-results requires four-life-reports')
        config['diagnostic_two_lobby']['fixed_room']['four_results'] = True
    if a.four_abort_room:
        if not a.four_results:
            p.error('four-abort-room requires four-results')
        config['diagnostic_two_lobby']['fixed_room']['four_abort_room'] = True
    with a.out.open('x') as f:
        f.write(json.dumps(config, indent=2) + '\n')


if __name__ == '__main__':
    main()
