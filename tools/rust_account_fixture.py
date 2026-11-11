"""Real SQLite/private launcher for synthetic fixed-room tests; no native oracle."""
import json
import os
from pathlib import Path
import socket
import subprocess
import tempfile
from server import probe_codec as codec
from server.room_title import _encode_bytes


class AccountFixture:
    def __init__(self, admin, out, config):
        self.admin = admin.resolve()
        self.out = out.resolve()
        os.chmod(self.out, 0o700)
        self.database = self.out / 'accounts.sqlite'
        self.control = Path(tempfile.mkdtemp(prefix='kr-basic-'))
        self.socket = self.control / 'launch.sock'
        self.calls = []
        self.launches = []
        self.channel_requests = {}
        self.command(dict(op='init'))
        self.command(dict(op='begin-instance'))
        self.players = []
        self.sessions = []
        for login, name in [('alice', 'KRLocal'), ('bob', 'KRGuest')]:
            player = self.command(dict(op='create', login=login, name=name,
                                       password='account-basic-test-password'))['player']
            grant = self.command(dict(op='login', login=login, password='account-basic-test-password',
                                     ttl_seconds=1200))
            assert grant['player'] == player
            self.players.append(player)
            self.sessions.append(grant['token'])
        config['diagnostic_two_lobby']['account_admission'] = dict(
            database=str(self.database), launch_socket=str(self.socket), launch_ttl_seconds=60)

    def command(self, command):
        request = dict(actor='account-basic-synthetic', reason='isolated ownership regression', command=command)
        result = subprocess.run([str(self.admin), str(self.database)], input=json.dumps(request),
                                text=True, capture_output=True, timeout=10)
        self.calls.append(dict(request=request, returncode=result.returncode,
                               stdout=result.stdout, stderr=result.stderr))
        (self.out/'private-backend.json').write_text(json.dumps(self.calls, indent=2)+'\n')
        assert result.returncode == 0, 'private backend command failed; raw retained'
        return json.loads(result.stdout)

    def connect(self, role, login_port, channel_port, header, hello, client_type):
        peer = '127.0.0.'+str(role+2)
        player = self.players[role]
        request = dict(session=self.sessions[role], player_id=player['player_id'], peer=peer)
        with socket.socket(socket.AF_UNIX) as control:
            control.settimeout(4)
            control.connect(str(self.socket))
            control.sendall(json.dumps(request).encode()+b'\n')
            data = b''
            while b'\n' not in data:
                chunk = control.recv(4096)
                assert chunk
                data += chunk
        launch = json.loads(data)
        assert launch['ok'] and launch['player'] == player and len(launch['frontend']) == 32
        self.launches.append(launch)
        (self.out/'private-launches.json').write_text(json.dumps(self.launches, indent=2)+'\n')
        login = client_type(role, login_port, header, hello)
        try:
            body = (codec.pack_fields([4, 1, 32], [1, 1, 2])
                    +codec.crypt(launch['frontend'].encode(), 0x11223344)+bytes(16)
                    +codec.pack_fields([0, int.from_bytes(socket.inet_aton(peer), 'little'), 0], [4, 4, 1]))
            login.send(1, body)
            _, auth = login.receive(4)
            assert len(auth) == 137 and codec.read_fields(auth[:1], [1]) == [0]
            assert codec.read_fields(auth[25:29], [4]) == [player['wire_id']]
            echo = codec.read_fields(auth[93:97], [4])[0]
            login.send(9, bytes(8))
            login.receive(15)
            login.receive(62)
            login.send(5, bytes(9))
            login.receive(7)
        finally:
            login.close()
        channel = client_type(role, channel_port, header, hello)
        name = player['display_name'].encode()
        body = (codec.pack_fields([1, 255, len(name)], [1, 1, 2])+_encode_bytes(name)
                +codec.pack_fields([player['wire_id'], echo], [4, 4])
                +codec.pack_fields([0]*8, [8, 1, 1, 1, 4, 4, 1, 4]))
        self.channel_requests[role] = body
        channel.send(6, body)
        channel.receive(17)
        channel.send(67, bytes(2))
        for opcode in [241, 91, 14, 291]:
            channel.receive(opcode)
        return channel

    def close(self):
        # Service owns/unlinks its socket. Keep a leftover path as an error.
        self.control.rmdir()
