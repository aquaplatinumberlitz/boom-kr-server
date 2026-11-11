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



def expand_eight_startup(config, derived_dir, peers):
    """CP208/209 fixed eight inputs from known two-profile fields.

    Uses unchanged four profile primitives, never imports current eight WIP.
    Producer output is not a native compatibility oracle.
    """
    import copy
    import ipaddress
    import struct
    from server import four_player_profile as four, probe_codec as codec
    from server.room_title import _decode_bytes, _encode_bytes
    config = copy.deepcopy(config)
    two = config.get('diagnostic_two_lobby', {})
    if len(two.get('actors', [])) != 2 or 'fixed_room' not in two:
        raise ValueError('eight startup requires two-role fixed-room input')
    if len(peers) != 8 or len(set(peers)) != 8:
        raise ValueError('eight distinct diagnostic peers required')
    for address in [config['bind'], *peers]:
        ip = ipaddress.IPv4Address(address)
        if not (ip.is_loopback or any(ip in ipaddress.IPv4Network(net) for net in
                ['10.0.0.0/8', '172.16.0.0/12', '192.168.0.0/16'])):
            raise ValueError('eight startup endpoints must be loopback or RFC1918')
    derived_dir = Path(derived_dir).resolve()
    if not derived_dir.is_relative_to(Path(__file__).resolve().parents[1] / 'local'):
        raise ValueError('derived inputs must stay under repository local/')
    roles = 'ABCDEFGH'
    members = {role: dict(four.MEMBERS[role]) if role in four.MEMBERS else
               dict(identity=i+1, slot=i, team=i%2, name='KRPeer'+str(i+1))
               for i, role in enumerate(roles)}
    room = two['fixed_room'];common = two['actors'][0]['profile']
    def pinned(row):
        path = Path(row['fixture']['path']);raw = path.read_bytes()
        if len(raw) != row['fixture']['bytes'] or hashlib.sha256(raw).hexdigest() != row['fixture']['sha256']:
            raise ValueError('eight startup source fixture pin mismatch')
        return raw
    player_template = room['packets']['player_B'];player = codec.frame_body(pinned(player_template), player_template['sequence'])
    auth_template = common['packets']['auth_A'];auth = codec.frame_body(pinned(auth_template), auth_template['sequence'])
    derived_dir.mkdir(parents=True, exist_ok=False)
    def pin(name, raw):
        path = derived_dir / (name+'.bin');path.write_bytes(raw)
        return dict(path=str(path), bytes=len(raw), sha256=hashlib.sha256(raw).hexdigest())
    def packet(name, template, body, sequence=None):
        seq = template['sequence'] if sequence is None else sequence
        return dict(fixture=pin(name, codec.encode(pinned(template)[:37], template['opcode'], body, seq)), sequence=seq, opcode=template['opcode'])
    actors = []
    for role, ip in zip(roles, peers):
        member = members[role];profile = copy.deepcopy(common);body = four.player_body(player, member, chief=role=='A')
        header = packet(role+'-player106', player_template, body, 58);profile['own_header'] = header['fixture']
        profile['packets']['auth_A'] = packet(role+'-auth', auth_template, four.auth_body(auth, member))
        profile['packets']['own291'] = dict(fixture=pin(role+'-own291', four.own_body(body)), sequence=None, opcode=291)
        room['packets']['player_'+role] = header
        actors.append(dict(peer=ip, identity=member['identity'], name=member['name'], profile=profile))
    start = room['packets']['start'];body = codec.frame_body(pinned(start), start['sequence'])
    if len(body) != 129 or codec.read_fields(body[:1], [1]) != [0]:
        raise ValueError('eight startup requires known two-row121')
    size = codec.read_fields(body[1:5], [4])[0];raw = _decode_bytes(body[5:5+size])
    if size != 106 or raw[:6] != b'\x18'+struct.pack('<I',101)+b'\0':
        raise ValueError('unsupported eight121 envelope')
    inner = raw[6:]
    if len(inner) != 100 or inner[43] != 2 or inner[48] != 0 or inner[74] != 1:
        raise ValueError('unsupported eight121 rows')
    rows = [inner[48:74], inner[74:100]]
    for role in roles[2:]:
        m = members[role];row = bytearray(rows[1]);row[0]=row[2]=m['slot'];struct.pack_into('<H',row,3,m['slot']);row[5]=m['team'];rows.append(bytes(row))
    prefix = bytearray(inner[:48]);prefix[43]=8;inner = prefix+b''.join(rows)
    stream = b'\x18'+struct.pack('<I',len(inner)+1)+b'\0'+inner
    body = body[:1]+codec.pack_fields([len(stream)],[4])+_encode_bytes(stream)+body[5+size:]
    if len(body) != 285:
        raise ValueError('eight121 bounded size')
    room['packets']['start'] = packet('start8', start, body)
    peer = room['packets']['peer'];prefix = codec.frame_body(pinned(peer),peer['sequence'])[:35]
    if len(prefix) != 35:
        raise ValueError('eight peer prefix truncated')
    body = bytearray(prefix);body[15:19]=codec.pack_fields([12346],[4])
    for slot in range(26):
        body += codec.pack_fields([int(slot<8)],[1])
        if slot < 8:
            endpoint = [0]*6 if slot == 0 else [int(ipaddress.IPv4Address(peers[slot])),32001+slot,0,0,0,0]
            body += codec.pack_fields(endpoint,[4,2]*3)
    room['packets']['peer'] = packet('peer8',peer,bytes(body))
    room.pop('battle',None);room.pop('terrain',None)
    for flag in ['four_owner_bombs','four_life_reports','four_results','four_abort_room']:
        room.pop(flag,None)
    two['actors']=actors;config['allowed_peers']=list(peers);config['max_connections']=max(48,config['max_connections'])
    source = Path(four.__file__)
    (derived_dir/'provenance.json').write_text(json.dumps(dict(scope='Eight fixed diagnostic startup input producer; eightBattle separately opt-in, not nativeacceptance.',builder_path=str(source),builder_sha256=hashlib.sha256(source.read_bytes()).hexdigest(),contract='docs/PROTOCOL.md#eight-session-preview',limits='CP208knownfields/zero profiles/fixedalternatingteams. No currenteightWIP import, auth/fullitems/admission9/nativecompatibility inferred.'),indent=2)+'\n')
    return config

def add_ninth_admission_probe(config, derived_dir, peer):
    """Bounded diagnostic visitor fixture, not a ninth room slot or native oracle."""
    import copy
    import ipaddress
    from server import four_player_profile as four, probe_codec as codec
    config = copy.deepcopy(config)
    two = config.get('diagnostic_two_lobby', {})
    room = two.get('fixed_room', {})
    if len(two.get('actors', [])) != 8 or not room.get('eight_battle'):
        raise ValueError('ninth probe requires explicit eight battle input')
    ip = ipaddress.IPv4Address(peer)
    if str(ip) in config['allowed_peers'] or not (ip.is_loopback or any(ip in ipaddress.IPv4Network(net) for net in ['10.0.0.0/8', '172.16.0.0/12', '192.168.0.0/16'])):
        raise ValueError('ninth probe needs a distinct loopback or RFC1918 peer')
    directory = Path(derived_dir).resolve()
    if not directory.is_relative_to(Path(__file__).resolve().parents[1] / 'local'):
        raise ValueError('derived probe input must stay under repository local/')
    def pinned(row):
        spec = row['fixture'];raw = Path(spec['path']).read_bytes()
        if len(raw) != spec['bytes'] or hashlib.sha256(raw).hexdigest() != spec['sha256']:
            raise ValueError('ninth probe source pin mismatch')
        return raw
    template = room['packets']['player_H']
    common = two['actors'][7]['profile']
    member = dict(identity=9, slot=0, team=0, name='KRPeer9')
    body = four.player_body(codec.frame_body(pinned(template), template['sequence']), member)
    auth_pin = common['packets']['auth_A'];auth = four.auth_body(codec.frame_body(pinned(auth_pin), auth_pin['sequence']), member)
    directory.mkdir(parents=True, exist_ok=False)
    def pin(name, raw):
        path = directory / name;path.write_bytes(raw)
        return dict(path=str(path), bytes=len(raw), sha256=hashlib.sha256(raw).hexdigest())
    profile = copy.deepcopy(common)
    # The player106 carrier supplies only a pinned header and zero-profile291;
    # it is never inserted into the room packet/slot table.
    profile['own_header'] = pin('probe-header106.bin', codec.encode(pinned(template)[:37], 106, body, 58))
    profile['packets']['own291'] = dict(fixture=pin('probe-own291.bin', four.own_body(body)), opcode=291, sequence=None)
    profile['packets']['auth_A'] = dict(fixture=pin('probe-auth4.bin', codec.encode(pinned(auth_pin)[:37], 4, auth, auth_pin['sequence'])), opcode=4, sequence=auth_pin['sequence'])
    two['actors'].append(dict(peer=str(ip), identity=9, name='KRPeer9', profile=profile))
    room['ninth_admission_probe'] = True
    config['allowed_peers'].append(str(ip))
    config['max_connections'] = max(48, config['max_connections'])
    (directory/'provenance.json').write_text(json.dumps(dict(source='design', contract='docs/PROTOCOL.md#rust-ninth-admission-wip', builder='server/four_player_profile.py', builder_sha256=hashlib.sha256(Path(four.__file__).read_bytes()).hexdigest(), limits='Identity9 is a diagnostic lobby identity, not an allocated room slot or authentication. Generated input is not a native oracle. Generic87/result2 comes from native CP227; capacity reason is emulator policy.'), indent=2) + '\n')
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
    p.add_argument('--eight-startup-derived-dir', type=Path, help='Known fixed8 producer, separate from four startup inputs')
    p.add_argument('--eight-startup-peer', action='append', default=[])
    p.add_argument('--eight-team-layout', choices=['4v4', '1v7'], help='Fixed CP213 operator layout; requires eight-battle, not interactive team selection')
    p.add_argument('--ninth-admission-probe-peer', help='Distinct diagnostic lobby peer; eight room slots stay unchanged')
    p.add_argument('--ninth-admission-derived-dir', type=Path)
    p.add_argument('--eight-battle', action='store_true', help='Opt-in CP209/210 fixed8 bombs/life/results; no items/terrain/unrankedabort')
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
    if a.eight_startup_derived_dir or a.eight_startup_peer:
        if not a.eight_startup_derived_dir or len(a.eight_startup_peer) != 8 or any(four) or not all(two) or not all(room):
            p.error('eight startup requires two/fixed-room and eight distinct peers, separate from four expansion')
        config = expand_eight_startup(config, a.eight_startup_derived_dir, a.eight_startup_peer)
    if a.eight_battle:
        if not a.eight_startup_derived_dir:
            p.error('eight battle requires eight-startup profile')
        config['diagnostic_two_lobby']['fixed_room']['eight_battle'] = True
    if a.eight_team_layout:
        if not a.eight_battle:
            p.error('eight-team-layout requires eight-battle')
        config['diagnostic_two_lobby']['fixed_room']['eight_team_layout'] = a.eight_team_layout
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
    if a.ninth_admission_probe_peer or a.ninth_admission_derived_dir:
        if not a.ninth_admission_probe_peer or not a.ninth_admission_derived_dir:
            p.error('ninth probe needs both peer and derived directory')
        config = add_ninth_admission_probe(config, a.ninth_admission_derived_dir, a.ninth_admission_probe_peer)
    with a.out.open('x') as f:
        f.write(json.dumps(config, indent=2) + '\n')


if __name__ == '__main__':
    main()
