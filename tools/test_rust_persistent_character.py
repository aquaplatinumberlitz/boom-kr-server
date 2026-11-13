"""Serving account/room/preference integration over real TCP and SQLite; not native KR."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import signal
import sqlite3
import subprocess
import sys
import time
sys.path.insert(0,str(Path(__file__).resolve().parents[1]))
from server import probe_codec as codec
from server.room_title import _encode_bytes
from tools.rust_account_fixture import AccountFixture
from tools.test_rust_foundation import free_port
from tools.test_rust_room import Client


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    for key in ['binary','admin','config','out']:
        parser.add_argument('--'+key,type=Path,required=True)
    args=parser.parse_args();root=args.out.resolve();root.mkdir(mode=0o700,parents=True,exist_ok=False)
    cfg=json.loads(args.config.read_text());two=cfg['diagnostic_two_lobby'];assert len(two['actors'])==4
    ports=set()
    while len(ports)<3:ports.add(free_port())
    login,wcp,channel=sorted(ports);cfg.update(bind='127.0.0.1',allowed_peers=[f'127.0.0.{n+2}'for n in range(4)],ports=sorted(ports))
    two.update(login_port=login,login_candidate_ports=[login,wcp],wcp_ports=sorted(ports),channel_port=channel,
               managed_rooms=dict(max_rooms=2,capacity=2,player_choices=True,player_round=dict(characters=[0,1])))
    for actor,peer in zip(two['actors'],cfg['allowed_peers']):actor['peer']=peer
    account=AccountFixture(args.admin,root,cfg)
    two['account_admission'].update(profile_mode='database_zero',max_sessions=4,persist_character=True,worker_queue_capacity=4)
    config=root/'config.json';config.write_text(json.dumps(cfg,indent=2)+'\n')
    source_files=[args.binary,args.admin,args.config,Path(__file__),Path('tools/rust_account_fixture.py'),config,*Path('rust/crates').rglob('*.rs'),*Path('rust/crates').rglob('*.sql')]
    def pins(value):
        if isinstance(value,dict):
            if {'path','bytes','sha256'}<=value.keys():
                p=Path(value['path']);assert p.stat().st_size==value['bytes']and hashlib.sha256(p.read_bytes()).hexdigest()==value['sha256'];source_files.append(p)
            for x in value.values():pins(x)
        elif isinstance(value,list):
            for x in value:pins(x)
    pins(cfg);frozen=root/'frozen';frozen.mkdir();inputs=[]
    for i,p in enumerate(dict.fromkeys(source_files)):
        raw=p.read_bytes();dest=frozen/f'{i:03d}-{p.name}';dest.write_bytes(raw);inputs.append(dict(path=str(p.resolve()),bytes=len(raw),sha256=hashlib.sha256(raw).hexdigest(),retained=str(dest)))
    (root/'preflight.json').write_text(json.dumps(dict(files=inputs,net_namespace=os.readlink('/proc/self/ns/net')),indent=2)+'\n')
    hello=Path(cfg['bootstrap']['path']).read_bytes();header=Path(two['actors'][0]['profile']['own_header']['path']).read_bytes()[:37]
    clients=[];cases=[];frames=[];error=None;process=None;log=(root/'process.log').open('x')
    def receive(client,op=None):
        actual,body=client.receive(op);frames.append(dict(role=clients.index(client),opcode=actual,body_hex=body.hex()));return body
    def connect(role):
        c=account.connect(role,login,channel,header,hello,Client);clients.append(c);return c
    def create(c):
        title=b'Persist';body=codec.pack_fields([len(title)],[2])+_encode_bytes(title)+codec.pack_fields([0],[2])+codec.pack_fields([0,0,5,8191,0,0,0,0xffffffff,0,0,0,0],[4]*12)+codec.pack_fields([0]*4,[1]*4)
        c.send(81,body);body=receive(c,86);return codec.read_fields(body[1:3],[2])[0]
    def join(c,key):
        c.send(82,codec.pack_fields([key,key,0,1,5],[2,4,2,1,2]));receive(c,87)
    def setup(c):
        c.send(585,codec.pack_fields([0],[4]));rows={}
        while True:
            op,body=c.receive();frames.append(dict(role=clients.index(c),opcode=op,body_hex=body.hex()))
            if op==106:
                identity=codec.read_fields(body[1:5],[4])[0];state=codec.read_fields(body[12:16],[4])[0];rows[identity]=(state>>16)&8191
            if op==117:return rows
    def closed(c):
        c.sock.settimeout(5)
        try:assert c.sock.recv(1)==b''
        except ConnectionResetError:pass
        c.close();clients.remove(c)
    def dbpref(player):
        with sqlite3.connect(account.database)as db:
            return db.execute('SELECT character_key,revision FROM player_preferences WHERE player_id=?',(player,)).fetchone()
    try:
        process=subprocess.Popen([str(args.binary.resolve()),'--config',str(config),'--out',str(root/'service')],stdout=log,stderr=subprocess.STDOUT,start_new_session=True)
        for _ in range(500):
            p=root/'service/service-state.json'
            if p.exists()and json.loads(p.read_text()).get('ready'):break
            assert process.poll()is None,'startup failure';time.sleep(.01)
        else:raise AssertionError('readiness timeout')
        # DatabaseZero readiness advances epoch; grants must be fresh afterwards.
        for i in range(4):
            account.sessions[i]=account.command(dict(op='login',login=['alice','bob','account_2','account_3'][i],password='account-basic-test-password',ttl_seconds=1200))['token']
        a,b=connect(0),connect(1);a.quiet();b.quiet();key=create(a);receive(b,91);join(b,key)
        ar=setup(a);br=setup(b);assert ar[account.players[0]['wire_id']]==br[account.players[0]['wire_id']]==0
        for c in clients:c.quiet()
        blocked=sqlite3.connect(account.database,isolation_level=None);blocked.execute('BEGIN IMMEDIATE')
        a.send(92,codec.pack_fields([0,1],[1,2]));a.quiet();b.quiet()
        assert dbpref(account.players[0]['player_id'])is None
        cases.append('real writer lock blocks DB commit and neither native-facing client receives110 before release')
        blocked.execute('COMMIT');blocked.close()
        receive(a,110);receive(a,110);receive(b,110);receive(b,110)
        assert dbpref(account.players[0]['player_id'])==(1,1)and dbpref(account.players[1]['player_id'])is None
        cases.append('current account choice persisted before room110; peer account preference untouched')
        # Unsupported character must never reach the persistence API.
        a.send(92,codec.pack_fields([0,4],[1,2]));closed(a);receive(b,107);receive(b,110)
        assert dbpref(account.players[0]['player_id'])==(1,1)
        time.sleep(.1);a=connect(0);join(a,key);restored=setup(a)
        assert restored[account.players[0]['wire_id']]==1
        b.receive(106);a.quiet();b.quiet()
        cases.append('unsupported key rejected without DB write; fresh owned channel restores character from DB before setup106')
        # Hold real writer lock while choice reaches normal persistence worker.
        writer=sqlite3.connect(account.database,isolation_level=None);writer.execute('BEGIN IMMEDIATE')
        a.send(92,codec.pack_fields([0,0],[1,2]));time.sleep(.1)
        writer.execute('UPDATE sessions SET revoked_at=? WHERE account_id=? AND revoked_at IS NULL',(int(time.time()),account.players[0]['account_id']));writer.execute('COMMIT');writer.close()
        closed(a);receive(b,107);receive(b,110)
        assert dbpref(account.players[0]['player_id'])==(1,1)
        b.send(92,codec.pack_fields([0,1],[1,2]));receive(b,110)
        assert dbpref(account.players[1]['player_id'])==(1,1)
        cases.append('revocation committed under DB pressure prevents old pending choice mutation; surviving account current choice persists')
        for c in clients:c.close()
        clients.clear();os.killpg(process.pid,signal.SIGTERM);assert process.wait(timeout=8)==0
        assert not account.socket.exists();account.close();cases.append('owned shutdown/launcher cleanup')
    except Exception as exc:error=repr(exc);raise
    finally:
        for c in clients:c.close()
        if process is not None and process.poll()is None:
            os.killpg(process.pid,signal.SIGTERM)
            try:process.wait(timeout=8)
            except subprocess.TimeoutExpired:os.killpg(process.pid,signal.SIGKILL);process.wait(timeout=3)
        log.close();frozen_ok=all(Path(p['path']).stat().st_size==p['bytes']and hashlib.sha256(Path(p['path']).read_bytes()).hexdigest()==p['sha256']for p in inputs)
        (root/'frames.json').write_text(json.dumps(frames,indent=2)+'\n')
        (root/'result.json').write_text(json.dumps(dict(result='pass'if error is None else'fail',cases=cases,error=error,returncode=None if process is None else process.returncode,source_frozen=frozen_ok,limits='Synthetic TCP/real SQLite/actual owned serving integration only; native KR preference relogin still required.'),indent=2)+'\n');assert frozen_ok


if __name__=='__main__':main()
