"""Fixed8 real TCP policy checks; generated inputs are not native KR proof."""
from pathlib import Path
import argparse,hashlib,json,os,shutil,signal,subprocess,sys,time
sys.path.insert(0,str(Path.cwd()))
from tools.test_rust_room import Client
from tools.test_rust_foundation import free_port
from server import probe_codec as c
from server.room_title import _encode_bytes,_decode_bytes
p=argparse.ArgumentParser(description=__doc__);p.add_argument('--binary',type=Path,required=True);p.add_argument('--config',type=Path,required=True);p.add_argument('--out',type=Path,required=True);p.add_argument('--selector',type=int,choices=[0,1],default=0);p.add_argument('--manner-negative',action='store_true');a=p.parse_args()
r=a.out;r.mkdir(parents=True,exist_ok=False);cfg=json.loads(a.config.read_text());two=cfg['diagnostic_two_lobby'];assert len(two['actors'])==8 and two['fixed_room']['eight_battle']
layout=two['fixed_room'].get('eight_team_layout','4v4');teams=[0,1,1,1,1,1,1,1]if layout=='1v7'else[i%2 for i in range(8)]
assert not a.manner_negative or (layout=='1v7'and a.selector==1)
ports=set()
while len(ports)<3:ports.add(free_port())
login,wcp,channel=sorted(ports);cfg.update(bind='127.0.0.1',allowed_peers=[f'127.0.0.{i}'for i in range(2,10)],ports=sorted(ports));two.update(login_port=login,login_candidate_ports=[login,wcp],wcp_ports=sorted(ports),channel_port=channel)
for i,actor in enumerate(two['actors']):actor['peer']=cfg['allowed_peers'][i]
config=r/'config.json';config.write_text(json.dumps(cfg,indent=2)+'\n');sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest();binary=a.binary
files=[binary,Path(__file__),a.config,config,Path('tools/test_rust_room.py'),Path('tools/test_rust_foundation.py'),Path('tools/export_rust_profile.py'),Path('server/probe_codec.py'),Path('server/room_title.py'),Path('server/four_player_profile.py')]+[p for p in Path('rust').rglob('*')if p.is_file()and'target'not in p.parts]
def fixtures(v):
 if isinstance(v,dict):
  if {'path','bytes','sha256'}<=v.keys():
   p=Path(v['path']);assert p.stat().st_size==v['bytes']and sha(p)==v['sha256'];files.append(p)
  for child in v.values():fixtures(child)
 elif isinstance(v,list):
  for child in v:fixtures(child)
fixtures(cfg);frozen=r/'frozen-inputs';frozen.mkdir();pins=[]
for i,p in enumerate(sorted(set(files))):
 dst=frozen/f'{i:03d}-{p.name}';shutil.copyfile(p,dst);pins.append(dict(path=str(p),retained_path=str(dst),bytes=p.stat().st_size,sha256=sha(p)))
(r/'preflight.json').write_text(json.dumps(dict(files=pins,scope='Synthetic8 TCP only; frozen before Popen'),indent=2)+'\n')
output=(r/'process.log').open('x');proc=subprocess.Popen([str(binary.resolve()),'--config',str(config),'--out',str(r/'service')],stdout=output,stderr=subprocess.STDOUT);clients=[];cases=[];error=None
hello=Path(cfg['bootstrap']['path']).read_bytes();header=Path(two['actors'][0]['profile']['own_header']['path']).read_bytes()[:37];power=[1,1,1,1,1,1,2,2]
def note(label):cases.append(label)
def quiet():
 for client in clients:client.quiet()
def report(kind,tick,slot):
 body=c.pack_fields([kind,slot,120,159,tick,0],[1,1,2,2,4,4])
 if kind in[17,19]:
  size=14 if kind==17 else 11;blob=bytes([24])+(size-5).to_bytes(4,'little')+bytes(1)+bytes(size-6)
  if kind==17:blob=blob[:10]+(7).to_bytes(4,'little')
  body+=c.pack_fields([size],[4]);at=0
  while at<size:
   n=4 if size-at>=4 else 2 if size-at>=2 else 1;body+=c.pack_fields([int.from_bytes(blob[at:at+n],'little')],[n]);at+=n
 return body

def forward(slot,kind,tick):
 body=report(kind,tick,slot);clients[slot].send(127,body);expected=body[:6]+(c.pack_fields([7],[4])if kind==17 else body[14:]if kind==19 else b'')
 for peer,target in enumerate(clients):
  if peer!=slot:assert target.receive(170)[1]==expected
 clients[slot].quiet();return body

def bomb(slot,serial,row,col,*,key=12346,custom_power=None):
 clients[slot].send(118,c.pack_fields([key,0,slot,serial,row,col,1,0,0,0,power[slot]if custom_power is None else custom_power,1,0],[4,1,1,4,1,1,1,1,1,1,1,4,1]))

def spawn(slot,serial,row,col,identity):
 bomb(slot,serial,row,col)
 for target in clients:
  _,body=target.receive(159);fields=c.read_fields(body,[1,1,2,4,4,4,1,1,1,4,1,2,2,1,8,1,1]);assert fields[1]==slot and fields[3]==serial and fields[6:9]==[row,col,power[slot]]and fields[11]==identity

def go(key):
 for slot,target in enumerate(clients):
  target.send(99,c.pack_fields([0,0,32001+slot,0,0,0,0,0,0],[4,4,2,4,2,1,1,1,4]))
  if slot<7:quiet()
 for slot,target in enumerate(clients):
  target.receive(197);target.receive(196);_,body=target.receive(153);assert len(body)==205 and c.read_fields(body[15:19],[4])==[key];at=35
  for other in range(26):
   active=c.read_fields(body[at:at+1],[1])[0];at+=1;assert active==int(other<8)
   if active:
    values=c.read_fields(body[at:at+18],[4,2]*3);at+=18;expected=[0]*6 if other==slot else [0x7f000002+other,32001+other,0,0,0,0];assert values==expected
  assert at==len(body)
 note('eight_ACK_barrier_all205byte_peer_maps_selfzero_key_'+str(key))

def outcome(winner):
 for target in clients:
  target.sock.settimeout(10);_,body=target.receive(175);target.sock.settimeout(6);assert len(body)==1082;inner=_decode_bytes(body[4:])[6:];assert len(inner)==1072 and inner[9]==255
  assert [inner[16+131*i+5]for i in range(8)]==[1 if team==winner else 2 for team in teams]
 for target in clients:
  assert c.read_fields(target.receive(180)[1],[1,1,1])==[0,a.selector,0];slots=[]
  for _ in range(8):slots.append(c.read_fields(target.receive(110)[1][:1],[1])[0])
  assert sorted(slots)==list(range(8))
 note('rank8_vector_'+str(winner)+'_180_all64refreshes')

def restart(key):
 for target in clients[1:]:
  target.send(95,c.pack_fields([6],[4])+bytes(6)+c.pack_fields([1,0],[1,4]))
  for peer in clients:peer.receive(110)
 clients[0].send(104,bytes(20))
 for target in clients:
  target.receive(582);body=target.receive(121)[1];assert len(body)==285;size=c.read_fields(body[1:5],[4])[0];inner=_decode_bytes(body[5:5+size])[6:];assert[inner[48+26*i+5]for i in range(8)]==teams
 go(key)
def scenario():
 deadline=time.monotonic()+5
 while time.monotonic()<deadline:
  state=r/'service/service-state.json'
  if state.exists()and json.loads(state.read_text())['ready']:break
  assert proc.poll()is None,'startupfailure';time.sleep(.01)
 else:raise AssertionError('service readiness timeout')
 for slot,actor in enumerate(two['actors']):
  client=Client(slot,wcp if slot%2==0 else login,header,hello);client.send(1,bytes([0x5e,0x5b])+c.pack_fields([13],[2])+bytes(38));client.receive(4);client.send(9,bytes(8));client.receive(15);client.receive(62);client.send(5,bytes(9));client.receive(7);client.close()
  client=Client(slot,channel,header,hello);name=actor['name'].encode();client.send(6,bytes(2)+c.pack_fields([len(name)],[2])+_encode_bytes(name)+c.pack_fields([slot+1],[4])+bytes(28));client.receive(17);client.send(544,bytes(3+len(name)));client.send(67,bytes(2))
  for op in[241,91,14,291]:client.receive(op)
  clients.append(client)
 note('eight_distinct_channel_profiles_and_five_character_C_name')
 title=two['fixed_room']['title'].encode();clients[0].send(81,c.pack_fields([len(title)],[2])+_encode_bytes(title)+c.pack_fields([0],[2])+c.pack_fields([0,a.selector,5,8191,0,0,0,0xffffffff,0,0,0,0],[4]*12)+c.pack_fields([0]*4,[1]*4));assert c.read_fields(clients[0].receive(86)[1][7:8],[1])==[a.selector]
 for slot,target in enumerate(clients[1:],1):
  target.receive(91);target.send(82,c.pack_fields([1,1,0,1,5],[2,4,2,1,2]));joined=target.receive(87)[1];assert c.read_fields(joined[1:2],[1])==[slot]and c.read_fields(joined[5:6],[1])==[a.selector]
 for target in clients:target.send(585,c.pack_fields([0],[4]))
 for slot,target in enumerate(clients):
  identities=[]
  while True:
   op,body=target.receive()
   if op==106:identities.append(c.read_fields(body[1:5],[4])[0])
   if op==117:break
  assert identities[0]==slot+1 and sorted(identities)==list(range(1,9))
 note('eight_ownslots_all64roster106_no_duplicate_identity')
 if a.manner_negative:
  for target in clients[1:]:
   target.send(95,c.pack_fields([6],[4])+bytes(6)+c.pack_fields([1,0],[1,4]))
   for peer in clients:peer.receive(110)
  for attempt in range(2):
   clients[0].send(104,bytes(20));quiet()
   events=[json.loads(line)for line in(r/'service/rust-events.jsonl').read_text().splitlines()]
   rejected=[x for x in events if x.get('start_rejection')=='manner_fixed_teams_unbalanced'];assert len(rejected)==attempt+1
   assert not any(x.get('event')=='room_reply'and x.get('opcode')in[121,153,159,175,180]for x in events)
  clients[0].send(86,b'')
  clients[0].receive(241);clients[0].receive(91)
  note('Manner1v7_forged104_twice_no_round_wire_transport_still_live_Back86')
  return
 restart(12346);quiet()
 bomb(0,7,3,2,custom_power=2);bomb(6,7,3,8,custom_power=1);bomb(7,7,3,9,key=12345);quiet();note('wrong_power_or_key_no_spawn_or_ID')
 for slot in range(8):spawn(slot,7,3,2+slot,slot+1)
 note('eight_owner_same_serial_distinct_IDs_64spawns_measured_powers')
 bomb(7,8,4,9);quiet();spawn(1,8,4,3,9);spawn(4,8,4,6,10);note('Hcapacity1_BEcapacity2_no_ID_on_reject')
 for target in clients:
  ids=[];target.sock.settimeout(8)
  for _ in range(10):ids.append(c.read_fields(target.receive(158)[1],[1,2,1,4,4,1])[1])
  target.sock.settimeout(6);assert sorted(ids)==list(range(1,11))
 note('all80blastwrites_complete_before_life_assertions')
 for slot,target in enumerate(clients):
  for aux in[33,37,39,47,0xffffffff]:target.send(142,c.pack_fields([3,2,slot,aux],[1,1,1,4]))
  for size in[4,22,42]:target.send(167,bytes(size))
 quiet();note('only_measured_eight142_aux_and167_shapes_no_effect')
 old=forward(0,17,1);forward(0,21,2);clients[0].send(127,report(21,3,0));quiet();note('clear21_exact_seven_recipients_once')
 for slot in[i for i,t in enumerate(teams)if t==0]:forward(slot,17,4);forward(slot,19,5)
 for slot in[i for i,t in enumerate(teams)if t==1]:clients[slot].send(127,report(21,1,slot))
 outcome(1);restart(12347)
 clients[0].send(127,old);quiet();note('old_life_report_fenced_after_reset_with_cursor_retained')
 for slot in[i for i,t in enumerate(teams)if t==1]:forward(slot,17,6);forward(slot,19,7)
 for slot in[i for i,t in enumerate(teams)if t==0]:clients[slot].send(127,report(21,8,slot))
 outcome(0);restart(12348);forward(0,17,9);forward(0,21,10);quiet();note('Start3_current_life_positive_after_two_eight_outcomes')
 bomb(7,99,3,9,key=12348)
 for target in clients:target.receive(159)
 clients[7].close();del clients[7:]
 for target in clients:
  assert c.read_fields(target.receive(107)[1],[1,4,1])==[7,8,0];target.receive(110)
 time.sleep(6.5);quiet();note('eight_pending_H_EOF_fences_timer_no_rank_or_unproven180')
try:scenario()
except BaseException as exc:error=repr(exc);raise
finally:
 for target in clients:target.close()
 proc.send_signal(signal.SIGTERM)if proc.poll()is None else None
 try:proc.wait(timeout=5)
 except subprocess.TimeoutExpired:
  proc.kill()
  try:proc.wait(timeout=3)
  except subprocess.TimeoutExpired:error=error or 'OwnedPIDstillpresentaftercleanupdeadline'
 output.close();unchanged=all(Path(x['path']).stat().st_size==x['bytes']and sha(x['path'])==x['sha256']for x in pins)
 v=dict(result='pass'if error is None and proc.returncode==0 and unchanged else'fail',cases=cases,error=error,returncode=proc.returncode,source_frozen=unchanged,remaining_pid=None if proc.poll()is not None else proc.pid,binary_sha256=sha(binary),layout=layout,selector=a.selector,limits='Synthetic8 TCP policy/producer inputs only, not nativeKR8/56mesh/fullM4/nativevariants/admission9/auth. No native7survivorRoomclaim afterEOF.')
 v['raw_files']=[dict(path=str(p),bytes=p.stat().st_size,sha256=sha(p))for p in sorted(r.rglob('*'))if p.is_file()];(r/'verification.json').write_text(json.dumps(v,indent=2)+'\n');print(v['result'],len(cases),'synthetic8cases/notnativeproof');assert v['result']=='pass',v['error'] or 'cleanup/sourcefailure'
