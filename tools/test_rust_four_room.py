"""Synthetic4identity room/loading/peer TCP; no native KR acceptance."""
from pathlib import Path
import argparse,copy,json,hashlib,os,socket,subprocess,sys,time,signal,shutil
sys.path.insert(0,str(Path.cwd()));from tools.test_rust_room import Client
from tools.test_rust_foundation import free_port
from server import probe_codec as c
from server.room_title import _encode_bytes
parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--binary',type=Path,required=True);parser.add_argument('--config',type=Path,required=True);parser.add_argument('--out',type=Path,required=True);parser.add_argument('--owner-bombs',action='store_true');parser.add_argument('--life-reports',action='store_true');args=parser.parse_args()
root=args.out;root.mkdir(parents=True,exist_ok=False);cfg=json.loads(args.config.read_text());two=cfg['diagnostic_two_lobby'];assert len(two['actors'])==4
ports=set()
while len(ports)<3:ports.add(free_port())
login,wcp,channel=sorted(ports);cfg.update(bind='127.0.0.1',allowed_peers=['127.0.0.'+str(i+2)for i in range(4)],ports=sorted(ports));two.update(login_port=login,login_candidate_ports=[login,wcp],wcp_ports=sorted(ports),channel_port=channel)
for i,actor in enumerate(two['actors']):actor['peer']=cfg['allowed_peers'][i]
if args.owner_bombs or args.life_reports:two['fixed_room']['four_owner_bombs']=True
if args.life_reports:two['fixed_room']['four_life_reports']=True
config=root/'config.json';config.write_text(json.dumps(cfg,indent=2)+'\n');binary=args.binary;sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest()
files=[binary,Path(__file__),args.config,config,Path('tools/test_rust_room.py'),Path('tools/test_rust_foundation.py'),Path('server/probe_codec.py'),Path('server/room_title.py'),Path('tools/export_rust_profile.py'),Path('server/four_player_profile.py')]+[p for p in Path('rust').rglob('*')if p.is_file()and'target'not in p.parts]
def fixtures(value):
 if isinstance(value,dict):
  if set(['path','bytes','sha256']).issubset(value):
   p=Path(value['path']);assert p.stat().st_size==value['bytes']and sha(p)==value['sha256'];files.append(p)
  for child in value.values():fixtures(child)
 elif isinstance(value,list):
  for child in value:fixtures(child)
fixtures(cfg);files=sorted(set(files));frozen=root/'frozen-inputs';frozen.mkdir();pins=[]
for i,p in enumerate(files):
 dest=frozen/f'{i:03d}-{p.name}';shutil.copyfile(p,dest)
 with dest.open('rb')as handle:os.fsync(handle.fileno())
 pins.append(dict(path=str(p),retained_path=str(dest),bytes=p.stat().st_size,sha256=sha(p)))
(root/'preflight.json').write_text(json.dumps(dict(files=pins,limits='Retainedstaticinputs/binarybeforePopen;no native client'),indent=2)+'\n');rawlog=(root/'process.log').open('w');proc=subprocess.Popen([str(binary.resolve()),'--config',str(config),'--out',str(root/'service')],stdout=rawlog,stderr=subprocess.STDOUT);clients=[];cases=[];error=None;hello=Path(cfg['bootstrap']['path']).read_bytes();header=Path(two['actors'][0]['profile']['own_header']['path']).read_bytes()[:37]
def note(label):cases.append(label)
def pid_snapshot(label):
 item=dict(label=label,pid=proc.pid,returncode=proc.poll(),ns=time.monotonic_ns(),threads=[])
 base=Path('/proc')/str(proc.pid)
 for key in ['status','wchan','stat']:
  try:item[key]=(base/key).read_text()
  except OSError as exc:item[key]=repr(exc)
 if (base/'task').exists():
  for thread in (base/'task').iterdir():
   try:item['threads'].append(dict(tid=thread.name,status=(thread/'status').read_text(),wchan=(thread/'wchan').read_text()))
   except OSError:pass
 observations.append(item);(root/'pid-observations.json').write_text(json.dumps(observations,indent=2)+'\n')
observations=[]
def quiet():
 for client in clients:client.quiet()
try:
 deadline=time.monotonic()+5
 while time.monotonic()<deadline:
  p=root/'service/service-state.json'
  if p.exists()and json.loads(p.read_text())['ready']:break
  assert proc.poll()is None,'startup failure';time.sleep(.01)
 else:raise AssertionError('no readiness')
 for role,actor in enumerate(two['actors']):
  client=Client(role,wcp if role%2==0 else login,header,hello);client.send(1,bytes([0x5e,0x5b])+c.pack_fields([13],[2])+bytes(38));client.receive(4);client.send(9,bytes(8));client.receive(15);client.receive(62);client.send(5,bytes(9));client.receive(7);client.close()
  client=Client(role,channel,header,hello);name=actor['name'].encode();client.send(6,bytes(2)+c.pack_fields([len(name)],[2])+_encode_bytes(name)+c.pack_fields([role+1],[4])+bytes(28));client.receive(17);client.send(544,bytes(3+len(name)));client.send(67,bytes(2))
  for opcode in [241,91,14,291]:client.receive(opcode)
  clients.append(client)
 note('four_distinct_identity_channel_short_name_C_and_profile544_shapes')
 A=clients[0];title=two['fixed_room']['title'].encode();create=c.pack_fields([len(title)],[2])+_encode_bytes(title)+c.pack_fields([0],[2])+c.pack_fields([0,0,5,8191,0,0,0,0xffffffff,0,0,0,0],[4]*12)+c.pack_fields([0]*4,[1]*4)
 A.send(81,create);A.receive(86)
 for index,client in enumerate(clients[1:],1):
  client.receive(91);client.send(82,c.pack_fields([1,1,0,1,5],[2,4,2,1,2]));_,body=client.receive(87);assert c.read_fields(body[1:2],[1])==[index]
 note('four_slots_join_one_current_room')
 for client in clients:client.send(585,c.pack_fields([0],[4]))
 for index,client in enumerate(clients):
  ids=[];ops=[]
  while True:
   opcode,body=client.receive();ops.append(opcode)
   if opcode==106:ids.append(c.read_fields(body[1:5],[4])[0])
   if opcode==117:break
  assert ids[0]==index+1 and sorted(ids)==[1,2,3,4] and ops.count(110)==1 and ops[-2:]==[238,117],(index,ids,ops)
 note('each_view_own_and_three_peers_subject_guarded_four_roster')
 for client in clients[1:]:
  client.send(95,c.pack_fields([6],[4])+bytes(6)+c.pack_fields([1,0],[1,4]))
  for target in clients:target.receive(110)
 note('three_guest_ready_fanout_all_four')
 A.send(104,bytes(20))
 for client in clients:
  client.receive(582);_,body=client.receive(121);assert len(body)==181
 note('four_row_start121')
 for slot,client in enumerate(clients):
  client.send(99,c.pack_fields([0,0,32001+slot,0,0,0,0,0,0],[4,4,2,4,2,1,1,1,4]))
  if slot<3:quiet()
 for slot,client in enumerate(clients):
  client.receive(197);client.receive(196);_,body=client.receive(153);assert len(body)==133;at=35
  for peer in range(26):
   active=c.read_fields(body[at:at+1],[1])[0];at+=1;assert active==int(peer<4)
   if active:
    values=c.read_fields(body[at:at+18],[4,2]*3);at+=18;expected=[0,0,0,0,0,0]if peer==slot else[0x7f000002+peer,32001+peer,0,0,0,0];assert values==expected,(slot,peer,values,expected)
  assert at==len(body)
 note('no_GO_with_three_ACKs_then_current_four_peer_endpoints')
 quiet();note('four_channels_quiet_after_loading')
 if args.owner_bombs:
  def bomb(client,slot,serial,row,col,power=1,key=12346):
   client.send(118,c.pack_fields([key,0,slot,serial,row,col,1,0,0,0,power,1,0],[4,1,1,4,1,1,1,1,1,1,1,4,1]))
  # Reject wrong epoch/slot/power without taking an ID or mutating reservations.
  bomb(clients[0],0,0,3,2,key=12345);quiet()
  bomb(clients[0],1,0,3,2);quiet()
  bomb(clients[0],0,0,3,2,power=2);quiet()
  note('four_bombs_reject_wrong_key_owner_and_power_without_reply')
  for slot,client in enumerate(clients):
   bomb(client,slot,7,3,2+slot)
   for target in clients:
    _,body=target.receive(159);f=c.read_fields(body,[1,1,2,4,4,4,1,1,1,4,1,2,2,1,8,1,1]);assert f[1]==slot and f[3]==7 and f[6:9]==[3,2+slot,1] and f[11]==slot+1
  note('four_owner_same_serial_independent_ids_and_16_spawn_replies')
  # D has one reserved; B can place a second. No serial/ID spent on rejection.
  bomb(clients[3],3,8,4,5);quiet()
  bomb(clients[0],0,7,3,2);quiet()
  bomb(clients[1],1,8,4,3)
  for target in clients:
   _,body=target.receive(159);assert c.read_fields(body,[1,1,2,4,4,4,1,1,1,4,1,2,2,1,8,1,1])[11]==5
  note('D_capacity_one_B_capacity_two_duplicate_no_id')
  for target in clients:
   got=[]
   for _ in range(5):
    target.sock.settimeout(8)
    _,body=target.receive(158);f=c.read_fields(body,[1,2,1,4,4,1]);assert f[0]==1 and f[2:]==[0,2,0xffffffff,255];got.append(f[1])
   assert sorted(got)==[1,2,3,4,5]
  note('five_blasts_all_four_recipients_complete_at_design_fuse')
  for client in clients:client.send(167,bytes(22))
  quiet();note('four_only_measured167_22_opaque_no_effect')
  for slot,client in enumerate(clients):
   for aux in [37,0xffffffff]:client.send(142,c.pack_fields([3,2,slot,aux],[1,1,1,4]))
  quiet();note('measured_four142_aux37_and_minus1_bounded_no_reveal_effect')
  bomb(clients[3],3,8,4,5)
  for target in clients:
   _,body=target.receive(159);assert c.read_fields(body,[1,1,2,4,4,4,1,1,1,4,1,2,2,1,8,1,1])[11]==6
  note('D_rejected_serial_can_retry_after_all_recipients_complete')
 if args.life_reports:
  def report(kind,tick,slot=0):
   body=c.pack_fields([kind,slot,120,159,tick,0],[1,1,2,2,4,4])
   if kind in [17,19]:
    size=14 if kind==17 else 11
    blob=bytes([24])+(size-5).to_bytes(4,'little')+bytes(1)+bytes(size-6)
    if kind==17:blob=blob[:10]+(7).to_bytes(4,'little')
    body+=c.pack_fields([size],[4]);offset=0
    while offset<size:
     n=4 if size-offset>=4 else 2 if size-offset>=2 else 1
     body+=c.pack_fields([int.from_bytes(blob[offset:offset+n],'little')],[n]);offset+=n
   return body
  def forward(slot,kind,tick):
   payload=report(kind,tick,slot);clients[slot].send(127,payload)
   expected=payload[:6]+(c.pack_fields([7],[4])if kind==17 else payload[14:]if kind==19 else b'')
   for peer,target in enumerate(clients):
    if peer!=slot:
     _,body=target.receive(170);assert body==expected
   clients[slot].quiet();return payload
  clients[0].send(127,report(19,0x7fffffff));quiet()
  clients[0].send(127,report(17,0x7fffffff,slot=1));quiet()
  clients[0].send(127,report(39,0x7fffffff));quiet()
  note('four_life_death_without_trap_wrong_slot_unknown_subtype_no_cursor_poison')
  trap=forward(0,17,10);note('current_A17_exact170_all_three_peers_no_selfecho')
  clients[0].send(127,trap);quiet();note('four_duplicate17_no_second170')
  forward(0,21,11);note('clear21_all_three_peers_complete_no_first_peer_invalidation')
  clients[0].send(127,report(21,12));quiet();note('clear21_once_per_trap_after_all_three_complete')
  forward(0,17,13);forward(0,21,14);note('newtrap_clear21_fanout_all_three_again')
  forward(0,19,15);note('Adeath19_exact170_three_peers')
  for kind,tick in [(17,16),(21,17),(19,18)]:clients[0].send(127,report(kind,tick))
  clients[0].send(118,c.pack_fields([12346,0,0,1,3,2,1,0,0,0,1,1,0],[4,1,1,4,1,1,1,1,1,1,1,4,1]))
  quiet();note('four_dead_owner_no_resurrection_forward_or_bomb_ID')
  forward(2,17,1);forward(2,19,2);note('Cdeath19_all_three_peers_independent_cursor')
  clients[1].send(127,report(21,1,slot=1));clients[3].send(127,report(21,1,slot=3))
  time.sleep(8.1);quiet();note('life_only_profile_never_selects_two_player_rank_or_result')
except BaseException as exc:
 error=repr(exc);raise
finally:
 for client in clients:client.close()
 pid_snapshot('before-SIGTERM')
 proc.send_signal(signal.SIGTERM)if proc.poll()is None else None
 try:proc.wait(timeout=5)
 except subprocess.TimeoutExpired:
  pid_snapshot('SIGTERM-deadline')
  proc.kill()
  try:proc.wait(timeout=3)
  except subprocess.TimeoutExpired:
   pid_snapshot('SIGKILL-deadline');error=error or 'Owned PID still present after cleanup deadline'
 rawlog.close();unchanged=all(Path(x['path']).stat().st_size==x['bytes'] and sha(x['path'])==x['sha256']for x in pins)
 report=dict(result='pass'if error is None and proc.returncode==0 and unchanged else'fail',cases=cases,error=error,returncode=proc.returncode,binary_sha256=sha(binary),source_frozen=unchanged,remaining_pid=None if proc.poll()is not None else proc.pid,limits='SyntheticTCP/policy/fixture inputs only, not nativeKR4compatibility/life-results/8/accountauth. Inputgenerator is not oracle.')
 rows=[dict(path=str(p),bytes=p.stat().st_size,sha256=sha(p))for p in sorted(root.rglob('*'))if p.is_file()];report['raw_files']=rows
 (root/'verification.json').write_text(json.dumps(report,indent=2)+'\n');print(report['result'],len(cases),'syntheticfour cases, notnativeproof')
 assert report['result']=='pass',report['error'] or 'service cleanup or source pin failure'
