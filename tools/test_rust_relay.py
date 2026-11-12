"""Synthetic account/TCP/UDP relay integration, not native KR certification.

Use retained independently native-tested fixtures. UDP payloads here are opaque
test bytes: success proves routing/lifecycle only, not a KR consumer or gameplay.
"""
import argparse
import copy
import hashlib
import json
from pathlib import Path
import signal
import socket
import subprocess
import sys
import time

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from server import probe_codec as codec
from server.room_title import _encode_bytes
from tools.rust_account_fixture import AccountFixture
from tools.test_rust_life import Client
from tools.test_rust_player_slots import configure


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binary', type=Path, required=True)
    parser.add_argument('--admin', type=Path, required=True)
    parser.add_argument('--config', type=Path, required=True)
    parser.add_argument('--out', type=Path, required=True)
    args = parser.parse_args()
    args.out.mkdir(mode=0o700, parents=True, exist_ok=False)
    cfg = configure(json.loads(args.config.read_text()))
    two = cfg['diagnostic_two_lobby']
    two['fixed_room']['endpoint_policy'] = 'relay'
    # Two token-owned identities share a source IP. Metadata remains per player.
    cfg.update(allowed_peers=['127.0.0.2'], network_scope='controlled_internet')
    Client.peers = ('127.0.0.2', '127.0.0.2')
    for i, actor in enumerate(two['actors']):
        actor.update(peer=Client.peers[i], reported_login_ipv4='127.0.0.'+str(i+2))
    # Bind candidate ports until the complete test configuration is ready.
    held = [socket.socket(socket.AF_INET, socket.SOCK_DGRAM) for _ in range(8)]
    for item in held:
        item.bind(('127.0.0.1', 0))
    cfg['udp_relay'] = dict(bind='127.0.0.1', advertise='127.0.0.1',
        ports=[item.getsockname()[1] for item in held], receive_budget=32,
        send_budget=32, queue_messages=8, queue_bytes=2048,
        rate_per_second=100, pending_ms=500, lease_seconds=120)
    account = AccountFixture(args.admin, args.out, cfg)
    config = args.out/'config.json'
    config.write_text(json.dumps(cfg, indent=2)+'\n')
    hello = Path(cfg['bootstrap']['path']).read_bytes()
    header = Path(two['actors'][0]['profile']['own_header']['path']).read_bytes()[:37]
    binary_hash = hashlib.sha256(args.binary.read_bytes()).hexdigest()
    clients, udp, cases = [], [], []
    proc, error = None, None
    log = (args.out/'process.log').open('x')

    def note(name):
        cases.append(dict(case=name, result='pass'))

    def absent(sock):
        sock.settimeout(.25)
        try:
            sock.recvfrom(512)
        except socket.timeout:
            return
        raise AssertionError('unexpected relay datagram')

    try:
        for item in held:
            item.close()
        for name in ['pool-without-policy', 'policy-without-pool', 'no-account',
                     'zero-port', 'duplicate-port', 'zero-queue', 'bad-ttl']:
            invalid = copy.deepcopy(cfg)
            if name == 'pool-without-policy':
                invalid['diagnostic_two_lobby']['fixed_room']['endpoint_policy'] = 'legacy_direct'
            elif name == 'policy-without-pool':
                invalid.pop('udp_relay')
            elif name == 'no-account':
                invalid['diagnostic_two_lobby'].pop('account_admission')
                invalid['allowed_peers'] = ['127.0.0.2', '127.0.0.3']
                for i, actor in enumerate(invalid['diagnostic_two_lobby']['actors']):
                    actor['peer'] = invalid['allowed_peers'][i]
                    actor.pop('reported_login_ipv4', None)
            elif name == 'zero-port':
                invalid['udp_relay']['ports'][0] = 0
            elif name == 'duplicate-port':
                invalid['udp_relay']['ports'][1] = invalid['udp_relay']['ports'][0]
            elif name == 'zero-queue':
                invalid['udp_relay']['queue_messages'] = 0
            else:
                invalid['udp_relay']['pending_ms'] = 10001
            path = args.out/(name+'.json')
            path.write_text(json.dumps(invalid, indent=2)+'\n')
            rejected = subprocess.run([str(args.binary.resolve()), '--config', str(path.resolve()),
                '--out', str((args.out/name).resolve())], capture_output=True, text=True, timeout=6)
            (args.out/(name+'.log')).write_text(rejected.stdout+rejected.stderr)
            assert rejected.returncode != 0 and not (args.out/name/'service-state.json').exists(), name
            if name == 'no-account':
                assert 'relay requires explicit room policy, network pool and account admission' in rejected.stderr
            note('prebind_reject_'+name)
        proc = subprocess.Popen([str(args.binary.resolve()), '--config', str(config.resolve()),
            '--out', str((args.out/'service').resolve())], stdout=log,
            stderr=subprocess.STDOUT, start_new_session=True)
        deadline = time.monotonic()+5
        while time.monotonic() < deadline:
            state = args.out/'service/service-state.json'
            if state.exists() and json.loads(state.read_text())['ready']:
                break
            assert proc.poll() is None, 'service rejected configuration'
            time.sleep(.01)
        else:
            raise AssertionError('startup timeout')
        for role in range(2):
            clients.append(account.connect(role, two['login_port'], two['channel_port'], header, hello, Client))
            sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
            sock.bind((Client.peers[role], 0))
            sock.settimeout(2)
            udp.append(sock)
        note('shared_source_ip_distinct_account_channel_ownership')
        a, b = clients
        title = two['fixed_room']['title'].encode()
        create = (codec.pack_fields([len(title)], [2])+_encode_bytes(title)
            +codec.pack_fields([0], [2])
            +codec.pack_fields([0,0,5,8191,0,0,0,0xffffffff,0,0,0,0], [4]*12)
            +codec.pack_fields([0]*4, [1]*4))
        a.send(81, create)
        a.receive(86)
        b.receive(91)
        b.send(82, codec.pack_fields([1,1,0,1,5], [2,4,2,1,2]))
        b.receive(87)
        for client in clients:
            client.send(585, codec.pack_fields([0], [4]))
        for client in clients:
            while client.receive()[0] != 117:
                pass
        b.send(95, codec.pack_fields([6], [4])+bytes(6)+codec.pack_fields([1,0], [1,4]))
        for client in clients:
            client.receive(110)
        a.send(104, bytes(20))
        for client in clients:
            client.receive(582)
            client.receive(121)
        for role, client in enumerate(clients):
            client.send(99, codec.pack_fields([0,0,32001+role,0,0,0,0,0,0], [4,4,2,4,2,1,1,1,4]))
        endpoints = []
        for role, client in enumerate(clients):
            client.receive(197)
            client.receive(196)
            _, body = client.receive(153)
            rows = [codec.read_fields(body[35+i*19:54+i*19], [1,4,2,4,2,4,2]) for i in range(2)]
            assert rows[role][:3] == [1,0,0]
            ip, port = rows[1-role][1:3]
            assert port in cfg['udp_relay']['ports'] and port != 32001+1-role
            endpoints.append((socket.inet_ntoa(ip.to_bytes(4, 'big')), port))
        assert endpoints[0] != endpoints[1]
        note('recipient153_advertises_distinct_reserved_ports_not_reported99_ports')
        payloads = [b'opaque-A\0\xff-original', b'opaque-B\xff\0-original']
        # Pending A must wait for B's observed mapping, then both directions work.
        udp[0].sendto(payloads[0], endpoints[0])
        time.sleep(.15)
        udp[1].sendto(payloads[1], endpoints[1])
        for role, sock in enumerate(udp):
            sock.settimeout(2)
            data, source = sock.recvfrom(512)
            assert data == payloads[1-role] and source == endpoints[role], (role, source)
        note('unchanged_payload_reverse_socket_source_and_observed_destination')
        rogue = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
        try:
            rogue.bind(('127.0.0.4', 0))
            rogue.sendto(b'foreign-source', endpoints[0])
            udp[0].sendto(bytes(512), endpoints[0])
            absent(udp[1])
        finally:
            rogue.close()
        rebind = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
        try:
            rebind.bind((Client.peers[0], 0))
            rebind.sendto(b'rebind', endpoints[0])
            absent(udp[1])
        finally:
            rebind.close()
        udp[0].sendto(payloads[0], endpoints[0])
        udp[1].settimeout(2)
        assert udp[1].recvfrom(512) == (payloads[0], endpoints[1])
        note('foreign_ip_oversize_rebinding_denied_then_current_positive')
        # Duplicate opaque UDP is forwarded unchanged, not deduplicated by relay.
        for _ in range(2):
            udp[0].sendto(payloads[0], endpoints[0])
        for _ in range(2):
            assert udp[1].recvfrom(512) == (payloads[0], endpoints[1])
        note('duplicate_payload_preserved_no_gameplay_or_authentication_claim')
        account.command(dict(op='revoke', account_id=account.players[0]['account_id']))
        time.sleep(.4)
        for role, sock in enumerate(udp):
            sock.sendto(b'after-revoke', endpoints[role])
        for sock in udp:
            absent(sock)
        a.sock.settimeout(3)
        try:
            assert a.sock.recv(1) == b''
        except ConnectionResetError:
            pass
        note('external_revoke_invalidates_lease_before_more_forwarding_and_closes_owned_channel')
    except BaseException as exc:
        error = repr(exc)
    finally:
        for client in clients:
            client.close()
        for sock in udp+held:
            sock.close()
        if proc is not None and proc.poll() is None:
            proc.send_signal(signal.SIGTERM)
            try:
                proc.wait(timeout=5)
            except subprocess.TimeoutExpired:
                proc.kill()
                proc.wait(timeout=3)
                error = error or 'SIGTERM deadline'
        log.close()
        try:
            account.close()
        except OSError as exc:
            error = error or repr(exc)
        verification = dict(result='pass' if error is None else 'fail', cases=cases,
            error=error, remaining_pid=proc.pid if proc is not None and proc.poll() is None else None,
            binary_sha256=binary_hash, binary_unchanged=hashlib.sha256(args.binary.read_bytes()).hexdigest()==binary_hash,
            limits='Synthetic TCP/UDP/SQLite only; payloads opaque, no native KR consumer or NAT acceptance. Same-IP first-datagram spoof remains unresolved.')
        (args.out/'verification.json').write_text(json.dumps(verification, indent=2)+'\n')
    assert error is None, error
    print('PASS', len(cases), 'synthetic account/relay cases; no native claim')


if __name__ == '__main__':
    main()
