"""Synthetic loopback TCP room lifecycle/barrier checks; not native KR proof."""
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
sys.path.insert(0,str(Path(__file__).resolve().parents[1]))
from server import probe_codec as c
from server.room_title import _encode_bytes
from tools.test_rust_foundation import exact,free_port

class Client:
    def __init__(self,role,port,header,hello):
        self.sock=socket.socket();self.sock.settimeout(6);self.sock.bind(('127.0.0.'+str(role+2),0));self.sock.connect(('127.0.0.1',port));assert exact(self.sock,15)==hello
        self.header=header;self.tx=40;self.rx=43
    def send(self,op,body):
        self.sock.sendall(c.encode(self.header,op,body,self.tx));self.tx+=3
    def receive(self,expected=None):
        h=exact(self.sock,37);n=int.from_bytes(h[13:17],'little');assert n<=65536;f=h+exact(self.sock,n+1);body=c.frame_body(f,self.rx);self.rx+=3;op=int.from_bytes(h[5:9],'little')
        if expected is not None:assert op==expected,(op,expected)
        return op,body
    def quiet(self):
        self.sock.settimeout(.2)
        try:
            data=self.sock.recv(1)
        except socket.timeout:data=None
        finally:self.sock.settimeout(6)
        assert data is None,'unexpected packet or EOF at withheld barrier'
    def close(self):self.sock.close()


def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--binary',type=Path,required=True);p.add_argument('--config',type=Path,required=True);p.add_argument('--out',type=Path,required=True);p.add_argument('--bomb-checks',action='store_true',help='Synthetic118/159/158 capacity/fuse/EOF ticket checks');p.add_argument('--pickup-checks',action='store_true',help='Floor124/165 policy negatives and one-winner confirmation, synthetic only');a=p.parse_args();a.out.mkdir(parents=True,exist_ok=False)
    cfg=copy.deepcopy(json.loads(a.config.read_text()));ports=set()
    while len(ports)<3:ports.add(free_port())
    login,wcp,channel=sorted(ports);cfg.update(bind='127.0.0.1',allowed_peers=['127.0.0.2','127.0.0.3'],ports=sorted(ports));two=cfg['diagnostic_two_lobby'];two.update(login_port=login,login_candidate_ports=[login,wcp],channel_port=channel,wcp_ports=sorted(ports))
    for i,actor in enumerate(two['actors']):actor['peer']=cfg['allowed_peers'][i]
    cp=a.out/'config.json';cp.write_text(json.dumps(cfg,indent=2)+'\n');hello=Path(cfg['bootstrap']['path']).read_bytes();header=Path(two['actors'][0]['profile']['own_header']['path']).read_bytes()[:37]
    binary_sha_before=hashlib.sha256(a.binary.read_bytes()).hexdigest();(a.out/'binary-preflight.json').write_text(json.dumps(dict(path=str(a.binary.resolve()),sha256=binary_sha_before),indent=2)+'\n')
    output=(a.out/'process.log').open('x');proc=subprocess.Popen([str(a.binary.resolve()),'--config',str(cp),'--out',str(a.out/'service')],stdout=output,stderr=subprocess.STDOUT,start_new_session=True)
    clients=[];cases=[];error=None;observations=[]
    def snapshot(label):
        item=dict(label=label,pid=proc.pid,returncode=proc.poll(),ns=time.monotonic_ns(),threads=[])
        base=Path('/proc')/str(proc.pid)
        for key in ['status','wchan','stat']:
            try:item[key]=(base/key).read_text()
            except OSError as exc:item[key]=repr(exc)
        if (base/'task').exists():
            for thread in (base/'task').iterdir():
                try:item['threads'].append(dict(tid=thread.name,status=(thread/'status').read_text(),wchan=(thread/'wchan').read_text()))
                except OSError:pass
        observations.append(item);(a.out/'pid-observations.json').write_text(json.dumps(observations,indent=2)+'\n')
    def note(case):cases.append(dict(case=case,result='pass'))
    try:
        deadline=time.monotonic()+5
        while time.monotonic()<deadline:
            f=a.out/'service/service-state.json'
            if f.exists() and json.loads(f.read_text())['ready']:break
            assert proc.poll() is None,'prelisten rejection';time.sleep(.01)
        else:raise AssertionError('startup timeout')
        for role in [0,1]:
            s=Client(role,wcp if role==0 else login,header,hello);s.send(1,bytes([0x5e,0x5b])+c.pack_fields([13],[2])+bytes(38));s.receive(4);s.send(9,bytes(8));s.receive(15);s.receive(62);s.send(5,bytes(9));s.receive(7);s.close()
            s=Client(role,channel,header,hello);name=[b'KRLocal',b'KRGuest'][role];s.send(6,bytes(2)+c.pack_fields([7],[2])+_encode_bytes(name)+c.pack_fields([role+1],[4])+bytes(28));s.receive(17);s.send(67,bytes(2))
            for op in [241,91,14,291]:s.receive(op)
            clients.append(s)
        A,B=clients;title=two['fixed_room']['title'].encode('ascii')
        create=c.pack_fields([len(title)],[2])+_encode_bytes(title)+c.pack_fields([0],[2])+c.pack_fields([0,0,5,8191,0,0,0,0xffffffff,0,0,0,0],[4]*12)+c.pack_fields([0]*4,[1]*4)
        A.send(81,create);A.receive(86);B.receive(91);B.send(82,c.pack_fields([1,1,0,1,5],[2,4,2,1,2]));_,body=B.receive(87);assert c.read_fields(body[1:2],[1])==[1];note('create_list_join_context_and_distinct_slot')
        for s in clients:s.send(585,c.pack_fields([0],[4]))
        def setup(s):
            ids=[];ops=[]
            while True:
                op,body=s.receive();ops.append(op)
                if op==106:ids.append(c.read_fields(body[1:5],[4])[0])
                if op==117:break
            assert sorted(ids)==[1,2] and ops.count(110)==1 and ops[-2:]==[238,117],(ids,ops)
        for s in clients:setup(s)
        note('interleaved585_each_view_own_peer_roster_and_map')
        A.send(86,b'');_,body=B.receive(107);assert c.read_fields(body,[1,4,1])==[0,1,0];_,body=B.receive(110);assert c.read_fields(body,[1,4,1])==[1,34,0];A.receive(241);A.receive(91);note('hostBack_removes_only_A_and_promotes_B_with_team_preserved')
        A.send(82,c.pack_fields([1,1,0,1,5],[2,4,2,1,2]));_,body=A.receive(87);assert c.read_fields(body[1:2],[1])==[0];A.send(585,c.pack_fields([0],[4]));setup(A);_,body=B.receive(106);assert c.read_fields(body[1:5],[4])==[1];note('same_channel_rejoin_new_visit_projects_A_guest_B_host')
        A.send(95,c.pack_fields([6],[4])+bytes(6)+c.pack_fields([1,0],[1,4]))
        for s in clients:
            _,body=s.receive(110);assert c.read_fields(body,[1,4,1])==[0,4,0]
        note('guestReady_broadcast_by_current_membership')
        def bomb(slot=0,serial=1,row=3,column=3,key=12346,power=1):
            return c.pack_fields([key,0,slot,serial,row,column,1,0,0,0,power,1,0],[4,1,1,4,1,1,1,1,1,1,1,4,1])
        if a.bomb_checks:
            A.send(118,bomb());A.quiet();B.quiet();note('bomb_before_Start_no_allocation')
        def pickup(slot=0,key=12346,context=0,row=4,column=3,index=17,kind=2):
            return c.pack_fields([context,slot,row,column,index,kind,100,key,0],[1,1,1,1,4,2,4,4,1])
        if a.pickup_checks:
            A.send(124,pickup());A.quiet();B.quiet();note('pickup_before_Start_no_grant')
        B.send(104,bytes(20))
        for s in clients:s.receive(582);s.receive(121)
        if a.bomb_checks:
            A.send(118,bomb());A.quiet();B.quiet();note('bomb_during_loading_no_allocation')
        if a.pickup_checks:
            A.send(124,pickup());A.quiet();B.quiet();note('pickup_during_loading_no_grant')
        A.send(99,c.pack_fields([0,0,32001,0,0,0,0,0,0],[4,4,2,4,2,1,1,1,4]));A.quiet();B.quiet();note('one_loaded_member_does_not_emit_GO')
        B.send(99,c.pack_fields([0,0,32002,0,0,0,0,0,0],[4,4,2,4,2,1,1,1,4]))
        for s in clients:s.receive(197)
        for s in clients:s.receive(196);_,body=s.receive(153);assert len(body)==97
        note('both_ACKs_open_one_barrier_go_play_current_endpoints')
        if a.pickup_checks:
            A.send(123,bytes(10));A.quiet();B.quiet();note('opaque123_current_armed_no_reply_or_effect')
            for label,request in [
                ('wrong_key',pickup(key=12345)),('wrong_slot',pickup(slot=1)),
                ('wrong_context',pickup(context=1)),('wrong_row',pickup(row=3)),
                ('wrong_column',pickup(column=4)),('wrong_kind',pickup(kind=0)),
                ('unknown_index',pickup(index=999)),('hidden_unrevealed',pickup(row=2,column=3,index=27,kind=0))]:
                A.send(124,request);A.quiet();B.quiet();note('pickup_'+label+'_no_grant')
            A.send(124,pickup())
            for s in clients:
                _,body=s.receive(165);assert c.read_fields(body,[1,1,1,1,1,4,2,1])==[1,0,0,4,3,17,2,0]
            note('current_floor124_one165_per_peer_same_winner_index')
            for label,client,request in [('duplicate',A,pickup()),('other_bound_contender',B,pickup(slot=1))]:
                client.send(124,request);A.quiet();B.quiet();note('pickup_'+label+'_no_second_grant')

        if a.bomb_checks:
            for label,request in [('wrong_key',bomb(key=12345)),('wrong_owner',bomb(slot=1)),('outside_row',bomb(row=13)),('outside_column',bomb(column=15)),('unsupported_power',bomb(power=0))]:
                A.send(118,request);A.quiet();B.quiet();note('bomb_'+label+'_no_ID')
            spawn_widths=[1,1,2,4,4,4,1,1,1,4,1,2,2,1,8,1,1]
            def spawn(owner,serial,row,column,index):
                for client in clients:
                    _,data=client.receive(159);fields=c.read_fields(data,spawn_widths)
                    assert fields==[0,owner,0,serial,0,0,row,column,1,1,0,index,0,1,0,0,0]
            A.send(167,bytes(32));A.quiet();B.quiet();A.send(167,bytes(4));A.quiet();B.quiet();note('opaque167_armed_4_and32_no_reply_or_effect')
            A.send(118,bomb());spawn(0,1,3,3,1);note('current_A118_serial1_id1_159_both')
            for label,request in [('duplicate_serial',bomb()),('conflicting_serial',bomb(column=4)),('reserved_capacity',bomb(serial=2,column=4))]:
                A.send(118,request);A.quiet();B.quiet();note('bomb_'+label+'_no_second_ID')
            B.send(118,bomb(slot=1,row=9));spawn(1,1,9,3,2)
            B.send(118,bomb(slot=1,serial=2,row=9,column=5));spawn(1,2,9,5,3);note('B_initial_capacity2_distinct_IDs2_3')
            B.send(118,bomb(slot=1,serial=3,row=9,column=7));A.quiet();B.quiet();note('B_third_reserved_no_ID')
            for index in [1,2,3]:
                for client in clients:
                    _,data=client.receive(158);assert c.read_fields(data,[1,2,1,4,4,1])==[1,index,0,2,0xffffffff,255]
            note('fuse_jobs_deliver_exact_once_each_current_ID_to_both')
            A.send(118,bomb());A.quiet();B.quiet();note('serial_stays_consumed_after_explosion')
            A.send(118,bomb(serial=2,column=4));spawn(0,2,3,4,4);note('released_capacity_next_ID4')
            A.close();B.receive(107);B.receive(110);time.sleep(6.2);B.quiet();note('owner_EOF_fences_pending158_no_late_delivery')
        for s in clients:s.close()
        clients=[];time.sleep(.1);snapshot('before-SIGTERM');os.killpg(proc.pid,signal.SIGTERM)
        try:rc=proc.wait(timeout=5)
        except subprocess.TimeoutExpired:snapshot('SIGTERM-deadline');raise
        assert rc==0;note('owned_EOF_and_SIGTERM_cleanup')
    except Exception as exc:error=repr(exc);raise
    finally:
        for s in clients:s.close()
        if proc.poll() is None:
            os.killpg(proc.pid,signal.SIGTERM)
            try:proc.wait(timeout=5)
            except subprocess.TimeoutExpired:
                snapshot('cleanup-SIGTERM-deadline');os.killpg(proc.pid,signal.SIGKILL)
                try:proc.wait(timeout=2)
                except subprocess.TimeoutExpired:snapshot('SIGKILL-deadline');error=error or 'Owned PID still present after SIGKILL deadline'
        output.close();rows=[dict(path=str(p),bytes=p.stat().st_size,sha256=hashlib.sha256(p.read_bytes()).hexdigest()) for p in sorted(a.out.rglob('*')) if p.is_file()]
        (a.out/'verification.json').write_text(json.dumps(dict(result='pass' if error is None else 'fail',cases=cases,error=error,raw_files=rows,binary_sha256=binary_sha_before,binary_unchanged=hashlib.sha256(a.binary.read_bytes()).hexdigest()==binary_sha_before,remaining_pid=proc.pid if proc.poll() is None else None,limits='SyntheticpolicyTCP inputs incl95/104envelopes, notnativewiresemantic/acceptance;internalgenerationtestsnotrawreplayproof.'),indent=2)+'\n')
    print('PASS',len(cases),'fixedroomTCP/lifecycle/barrier cases; notnativeproof')
if __name__=='__main__':main()
