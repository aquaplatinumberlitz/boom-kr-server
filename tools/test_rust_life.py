"""Synthetic Rust report127/170, outcome/reset and connection-cursor controls; not native KR proof."""
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
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--binary',type=Path,required=True);p.add_argument('--config',type=Path,required=True);p.add_argument('--out',type=Path,required=True);p.add_argument('--account-admin',type=Path,help='Opt-in real backend/private-launcher account-owned fixed-room regression');p.add_argument('--idle-seconds',type=int,default=120,help='Read deadline preserved across room timer polls');p.add_argument('--battle-profile',type=Path,required=True,help='Retained Python profile with pinned outcome fixtures');p.add_argument('--terrain-profile',type=Path,help='Use existing pinned Python bomb_terrain spec for syntheticgeometry-reveal checks');p.add_argument('--bomb-checks',action='store_true',help='Synthetic118/159/158 capacity/fuse/EOF ticket checks');p.add_argument('--pickup-checks',action='store_true',help='Floor124/165 policy negatives and one-winner confirmation, synthetic only');a=p.parse_args();a.out.mkdir(parents=True,exist_ok=False)
    cfg=copy.deepcopy(json.loads(a.config.read_text()));ports=set()
    while len(ports)<3:ports.add(free_port())
    login,wcp,channel=sorted(ports);cfg.update(idle_timeout_seconds=a.idle_seconds,bind='127.0.0.1',allowed_peers=['127.0.0.2','127.0.0.3'],ports=sorted(ports));two=cfg['diagnostic_two_lobby'];two.update(login_port=login,login_candidate_ports=[login,wcp],channel_port=channel,wcp_ports=sorted(ports))
    profile=json.loads(a.battle_profile.read_text());battle={}
    for alias,opcode in [('rank',175),('rank_Awin',175),('rank_all_dead',175),('return',180)]:
        row=profile['fixtures'][alias];raw=Path(row['path']).read_bytes();assert len(raw)==row['bytes'] and hashlib.sha256(raw).hexdigest()==row['sha256']
        battle[alias]=dict(fixture={k:row[k] for k in ['path','bytes','sha256']},sequence=row['sequence'],opcode=opcode)
    two['fixed_room']['battle']=dict(mode='post_blast_untrapped_drained',packets=battle)
    if a.terrain_profile:two['fixed_room']['terrain']=json.loads(a.terrain_profile.read_text())['bomb_terrain']
    for i,actor in enumerate(two['actors']):actor['peer']=cfg['allowed_peers'][i]
    account_fixture=None
    if a.account_admin:
        from tools.rust_account_fixture import AccountFixture
        account_fixture=AccountFixture(a.account_admin,a.out,cfg)
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
            if account_fixture is not None:
                clients.append(account_fixture.connect(role,login,channel,header,hello,Client))
                continue
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
        A.send(167,bytes(16));A.quiet();B.quiet();note('opaque167_captured16_current_armed_no_response_or_effect')
        def report(kind,tick,slot=0,x=120):
            body=c.pack_fields([kind,slot,x,159,tick,0],[1,1,2,2,4,4])
            if kind in [17,19]:
                size=14 if kind==17 else 11
                blob=bytes([24])+(size-5).to_bytes(4,'little')+bytes(1)+bytes(size-6)
                if kind==17:blob=blob[:10]+(7).to_bytes(4,'little')
                body+=c.pack_fields([size],[4]);offset=0
                while offset<size:
                    n=4 if size-offset>=4 else 2 if size-offset>=2 else 1
                    body+=c.pack_fields([int.from_bytes(blob[offset:offset+n],'little')],[n]);offset+=n
            return body
        def forwarded(origin,peer,kind,tick,slot):
            payload=report(kind,tick,slot);origin.send(127,payload)
            _,body=peer.receive(170)
            expected=payload[:6]+(c.pack_fields([7],[4]) if kind==17 else payload[14:] if kind==19 else b'')
            assert body==expected;origin.quiet()
            return payload
        def quiet_pair():A.quiet();B.quiet()
        def outcome(alias):
            expected=c.frame_body(Path(profile['fixtures'][alias]['path']).read_bytes(),profile['fixtures'][alias]['sequence'])
            for client in clients:
                client.sock.settimeout(12);_,body=client.receive(175);assert body==expected;client.sock.settimeout(6)
            note('outcome_'+alias+'_both_exact_template')
            for client in clients:client.send(202,c.pack_fields([0,1]+[0]*5,[1,1]+[4]*5))
            quiet_pair();note('closed_result202_bounded_no_reply_no_cancel_return')
            for client in clients:client.send(585,c.pack_fields([0],[4]))
            quiet_pair();note('closed_result_setup585_duplicate_bounded_no_resend_or_membership_change')
            A.send(127,report(17,0x7fffffff));quiet_pair();note('closed_round_high_tick_no_cursor_mutation')
            for client in clients:
                _,body=client.receive(180);assert c.read_fields(body,[1,1,1])==[0,0,0]
                for _ in range(2):client.receive(110)
            time.sleep(.1);note('return180_selector_then_two_slot110_refresh')
        def restart(key):
            A.send(95,c.pack_fields([6],[4])+bytes(6)+c.pack_fields([1,0],[1,4]))
            for client in clients:client.receive(110)
            B.send(104,bytes(20))
            for client in clients:client.receive(582);client.receive(121)
            for i,client in enumerate(clients):client.send(99,c.pack_fields([0,0,32001+i,0,0,0,0,0,0],[4,4,2,4,2,1,1,1,4]))
            for client in clients:
                client.receive(197);client.receive(196);_,peer=client.receive(153)
                assert c.read_fields(peer[15:19],[4])==[key]
            note('Ready_Start_loading_current_key_'+str(key))
        A.send(127,report(19,0x7fffffff));quiet_pair();note('death_without_trap_does_not_poison_cursor')
        A.send(127,report(17,0x7fffffff,slot=1));quiet_pair();note('wrong_bound_slot_does_not_poison_cursor')
        old17=forwarded(A,B,17,10,0);note('current_trap17_exact_peer170_no_selfecho')
        A.send(127,old17);quiet_pair();note('duplicate_body_new_wire_sequence_no_second170')
        A.send(124,pickup());quiet_pair();note('trapped_pickup_ineligible')
        old19=forwarded(A,B,19,11,0);note('current_death19_exact_peer170_after_trap')
        forwarded(A,B,17,12,0);A.send(127,report(21,13));quiet_pair();note('new_trap_does_not_hide_accepted_death_for_result')
        A.send(118,bomb());quiet_pair();note('accepted_dead_owner118_no_ID_or_serial_commit')
        # Mature death/survivor does not finish while a later bomb is pending,
        # and the successful158 invalidates the old survivor witness.
        B.send(127,report(21,1,slot=1));time.sleep(4.5)
        B.send(118,bomb(slot=1,row=9))
        for client in clients:client.receive(159)
        time.sleep(3.6);quiet_pair();note('mature_death_waits_for_pending_bomb')
        for client in clients:client.receive(158)
        quiet_pair();note('last158_requires_fresh_survivor21')
        B.send(127,report(21,2,slot=1));outcome('rank')
        A.send(127,report(39,0x7fffffff));quiet_pair();note('Idle_tail_no_cursor_mutation')
        restart(12347)
        for stale in [old17,old19,report(21,13)]:A.send(127,stale);quiet_pair()
        B.send(127,report(21,1,slot=1));quiet_pair();note('same_connection_old_reports_rejected_after_round_reset')
        A.send(124,pickup(key=12347))
        for client in clients:client.receive(165)
        note('current_key_floor_claim_positive_after_old_life_bodies')
        B.send(127,report(19,0x7fffffff,slot=1));quiet_pair();note('new_round_death_requires_new_trap')
        forwarded(B,A,17,3,1);forwarded(B,A,19,4,1)
        A.send(127,report(21,14));outcome('rank_Awin')
        restart(12348)
        forwarded(A,B,17,15,0);forwarded(B,A,17,5,1)
        forwarded(A,B,19,16,0);forwarded(B,A,19,6,1)
        outcome('rank_all_dead');note('all_dead_uses_separate_tie_policy_latest_death_settle')
        restart(12349)
        forwarded(A,B,17,17,0)
        A.send(127,report(21,18));_,body=B.receive(170);assert body==report(21,18)[:6];A.quiet()
        A.send(127,report(21,19));quiet_pair();note('current_survivor21_peer_clear_once_per_trap_epoch')
        A.send(124,pickup(key=12349));quiet_pair();note('survivor21_does_not_restore_pickup_permission')
        forwarded(A,B,17,20,0);A.send(127,report(21,21));_,body=B.receive(170);assert body==report(21,21)[:6];A.quiet();note('fresh_trap_permits_new_once_peer21')
        B.send(118,bomb(slot=1,row=9,key=12349))
        for client in clients:client.receive(159)
        note('survivor_owned_bomb_pending_before_other_member_EOF')
        A.close();B.sock.settimeout(16);_,body=B.receive(175);B.sock.settimeout(6)
        expected=c.frame_body(Path(profile['fixtures']['rank']['path']).read_bytes(),profile['fixtures']['rank']['sequence']);assert body==expected
        note('EOF_immediate_fence_no158_and_delayed_chosen_forfeit_to_survivor')
        B.send(202,c.pack_fields([0,1]+[0]*5,[1,1]+[4]*5));B.receive(180);B.send(585,c.pack_fields([0],[4]))
        _,body=B.receive(107);assert c.read_fields(body,[1,4,1])==[0,1,0]
        _,body=B.receive(110);assert c.read_fields(body,[1,4,1])==[1,34,0]
        note('forfeit_return180_then_departed_IDclear_and_survivor_chief110')
        B.send(127,report(19,0x7fffffff,slot=1));B.quiet();note('EOF_fences_round_and_late_life_reports')
        if account_fixture is not None:
            old_channel = account_fixture.channel_requests[0]
            old_session = account_fixture.sessions[0]
            account_fixture.command(dict(op='revoke', account_id=account_fixture.players[0]['account_id']))
            grant = account_fixture.command(dict(op='login', login='alice',
                password='account-basic-test-password', ttl_seconds=1200))
            assert grant['token'] != old_session and grant['player'] == account_fixture.players[0]
            account_fixture.sessions[0] = grant['token']
            A = account_fixture.connect(0, login, channel, header, hello, Client)
            clients[0] = A
            A.receive(91)
            note('fresh_backend_frontend_channel_same_player_after_departed_owner_revoke')
            A.send(82,c.pack_fields([1,1,0,1,5],[2,4,2,1,2]));_,body=A.receive(87)
            assert c.read_fields(body[1:2],[1]) == [0]
            A.send(585,c.pack_fields([0],[4]));setup(A)
            _,body=B.receive(106);assert c.read_fields(body[1:5],[4]) == [1]
            note('fresh_owned_channel_rejoins_only_free_slot_survivor_membership_retained')
            rogue = Client(0,channel,header,hello)
            try:
                rogue.send(6,old_channel)
                try:assert rogue.sock.recv(1) == b''
                except ConnectionResetError:pass
            finally:rogue.close()
            quiet_pair()
            note('old_channel_correlator_on_unbound_transport_cannot_erase_fresh_room_owner')
            restart(12350)
            quiet_pair()
            note('fresh_owner_next_round_uses_new_key_no_old_bomb_or_outcome')
            forwarded(A,B,17,1,0);forwarded(A,B,21,2,0)
            note('fresh_connection_current_life_cursor_applies_after_old_channel_denial')
        if a.idle_seconds<=30:
            B.sock.settimeout(a.idle_seconds+3);idle_before=time.monotonic();assert B.sock.recv(1)==b'';assert time.monotonic()-idle_before>=a.idle_seconds-1;note('absolute_read_idle_deadline_survives_periodic_room_poll')
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
        if account_fixture is not None:
            try:account_fixture.close()
            except OSError as exc:error=error or repr(exc)
        output.close();rows=[dict(path=str(p),bytes=p.stat().st_size,sha256=hashlib.sha256(p.read_bytes()).hexdigest()) for p in sorted(a.out.rglob('*')) if p.is_file()]
        (a.out/'verification.json').write_text(json.dumps(dict(result='pass' if error is None else 'fail',cases=cases,error=error,raw_files=rows,binary_sha256=binary_sha_before,binary_unchanged=hashlib.sha256(a.binary.read_bytes()).hexdigest()==binary_sha_before,remaining_pid=proc.pid if proc.poll() is None else None,limits='SyntheticpolicyTCP inputs incl95/104envelopes, notnativewiresemantic/acceptance;internalgenerationtestsnotrawreplayproof.'),indent=2)+'\n')
    if error is not None:raise RuntimeError(error)
    print('PASS',len(cases),'fixedroomTCP/lifecycle/barrier cases; notnativeproof')
if __name__=='__main__':main()
