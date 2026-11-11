"""Synthetic room occupants for the explicit ninth-visitor probe.

These connections are test inputs, never eight native KR clients.
"""
import socket
from pathlib import Path
from server import probe_codec as c
from server.room_title import _encode_bytes
from tools.test_rust_room import Client
from tools.test_rust_foundation import exact


class EndpointClient(Client):
    def __init__(self, peer, destination, port, header, hello):
        self.sock = socket.socket()
        self.sock.settimeout(6)
        self.sock.bind((peer, 0))
        self.sock.connect((destination, port))
        assert exact(self.sock, 15) == hello
        self.header, self.tx, self.rx = header, 40, 43


def connect_actor(config, identity, *, destination=None):
    two = config['diagnostic_two_lobby']
    actor = next(row for row in two['actors'] if row['identity'] == identity)
    destination = destination or config['bind']
    header = Path(actor['profile']['own_header']['path']).read_bytes()[:37]
    hello = Path(config['bootstrap']['path']).read_bytes()
    login = EndpointClient(actor['peer'], destination, two['login_port'], header, hello)
    try:
        login.send(1, bytes([0x5e, 0x5b]) + c.pack_fields([13], [2]) + bytes(38))
        login.receive(4)
        login.send(9, bytes(8))
        login.receive(15)
        login.receive(62)
        login.send(5, bytes(9))
        login.receive(7)
    finally:
        login.close()
    channel = EndpointClient(actor['peer'], destination, two['channel_port'], header, hello)
    try:
        name = actor['name'].encode('ascii')
        channel.send(6, bytes(2) + c.pack_fields([len(name)], [2]) + _encode_bytes(name) + c.pack_fields([identity], [4]) + bytes(28))
        channel.receive(17)
        channel.send(544, bytes(3 + len(name)))
        channel.send(67, bytes(2))
        for op in [241, 91, 14, 291]:
            channel.receive(op)
        return channel
    except BaseException:
        channel.close()
        raise


def join_body():
    return c.pack_fields([1, 1, 0, 1, 5], [2, 4, 2, 1, 2])


def seed_eight(config, *, destination=None):
    """Create eight idle, setup-complete, ready synthetic members; caller owns sockets."""
    clients = []
    try:
        for identity in range(1, 9):
            clients.append(connect_actor(config, identity, destination=destination))
        title = config['diagnostic_two_lobby']['fixed_room']['title'].encode('ascii')
        clients[0].send(81, c.pack_fields([len(title)], [2]) + _encode_bytes(title) + c.pack_fields([0], [2]) + c.pack_fields([0, 1, 5, 8191, 0, 0, 0, 0xffffffff, 0, 0, 0, 0], [4]*12) + c.pack_fields([0]*4, [1]*4))
        clients[0].receive(86)
        for slot, target in enumerate(clients[1:], 1):
            target.receive(91)
            target.send(82, join_body())
            body = target.receive(87)[1]
            assert c.read_fields(body[1:2], [1]) == [slot]
        for target in clients:
            target.send(585, c.pack_fields([0], [4]))
        for slot, target in enumerate(clients):
            ids = []
            while True:
                op, body = target.receive()
                if op == 106:
                    ids.append(c.read_fields(body[1:5], [4])[0])
                if op == 117:
                    break
            assert ids[0] == slot + 1 and sorted(ids) == list(range(1, 9))
        for target in clients[1:]:
            target.send(95, c.pack_fields([6], [4]) + bytes(6) + c.pack_fields([1, 0], [1, 4]))
            for peer in clients:
                peer.receive(110)
        return clients
    except BaseException:
        for target in clients:
            target.close()
        raise
