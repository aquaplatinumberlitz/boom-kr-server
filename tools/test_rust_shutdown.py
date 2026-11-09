"""Repeated real TCP close/SIGTERM races; collect owned PID state on a deadline."""
import argparse,json,os,signal,socket,subprocess,sys,time,traceback
from pathlib import Path
# Run from repo root; install under tools/ after CP234 merge.
sys.path.insert(0,str(Path.cwd()))
from tools.test_rust_room import Client
from tools.test_rust_foundation import free_port
from server import probe_codec as c
from server.room_title import _encode_bytes

def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--binary',type=Path,required=True);p.add_argument('--config',type=Path,required=True);p.add_argument('--out',type=Path,required=True);p.add_argument('--iterations',type=int,default=12);a=p.parse_args();assert 1<=a.iterations<=32;a.out.mkdir(parents=True,exist_ok=False)
 results=[]
 for index in range(a.iterations):
  root=a.out/str(index);root.mkdir();cfg=json.loads(a.config.read_text());ports=set()
  while len(ports)<3:ports.add(free_port())
  login,wcp,channel=sorted(ports);cfg.update(bind='127.0.0.1',allowed_peers=['127.0.0.2','127.0.0.3'],ports=sorted(ports));two=cfg['diagnostic_two_lobby'];two.update(login_port=login,login_candidate_ports=[],channel_port=channel,wcp_ports=sorted(ports))
  for i,actor in enumerate(two['actors']):actor['peer']=cfg['allowed_peers'][i]
  cp=root/'config.json';cp.write_text(json.dumps(cfg,indent=2)+'\n');hello=Path(cfg['bootstrap']['path']).read_bytes();header=Path(two['actors'][0]['profile']['own_header']['path']).read_bytes()[:37];clients=[];error=None;status=[];output=(root/'process.log').open('x');proc=subprocess.Popen([str(a.binary.resolve()),'--config',str(cp),'--out',str(root/'service')],stdout=output,stderr=subprocess.STDOUT,start_new_session=True);started=time.monotonic_ns()
  def snapshot(label):
   row=dict(label=label,ns=time.monotonic_ns(),pid=proc.pid,returncode=proc.poll(),threads=[])
   for file in ['status','wchan','stat']:
    try:row[file]=(Path('/proc')/str(proc.pid)/file).read_text()
    except OSError as exc:row[file]=repr(exc)
   task=Path('/proc')/str(proc.pid)/'task'
   if task.exists():
    for thread in task.iterdir():
     try:row['threads'].append(dict(tid=thread.name,status=(thread/'status').read_text(),wchan=(thread/'wchan').read_text()))
     except OSError:pass
   status.append(row);(root/'pid-observations.json').write_text(json.dumps(status,indent=2)+'\n')
  try:
   deadline=time.monotonic()+5
   while time.monotonic()<deadline:
    f=root/'service/service-state.json'
    if f.exists() and json.loads(f.read_text())['ready']:break
    assert proc.poll() is None;time.sleep(.005)
   else:raise AssertionError('readiness deadline')
   for role in [0,1]:
    s=Client(role,login,header,hello);s.send(1,bytes([0x5e,0x5b])+c.pack_fields([13],[2])+bytes(38));s.receive(4);s.send(9,bytes(8));s.receive(15);s.receive(62);s.send(5,bytes(9));s.receive(7);s.close()
    s=Client(role,channel,header,hello);name=[b'KRLocal',b'KRGuest'][role];s.send(6,bytes(2)+c.pack_fields([7],[2])+_encode_bytes(name)+c.pack_fields([role+1],[4])+bytes(28));s.receive(17);s.send(67,bytes(2))
    for op in [241,91,14,291]:s.receive(op)
    clients.append(s)
   title=b'Shutdown';create=c.pack_fields([len(title)],[2])+_encode_bytes(title)+c.pack_fields([0],[2])+c.pack_fields([0,0,5,8191,0,0,0,0xffffffff,0,0,0,0],[4]*12)+c.pack_fields([0]*4,[1]*4)
   clients[0].send(81,create);clients[0].receive(86);clients[1].receive(91);clients[1].send(82,c.pack_fields([1,1,0,1,5],[2,4,2,1,2]));clients[1].receive(87)
   for s in clients:s.send(585,c.pack_fields([0],[4]))
   # Live recipients may have scheduled sends when close/reset/shutdown races.
   snapshot('before-close');variant=index%3
   if variant==0:
    for s in clients:s.close()
   elif variant==1:
    clients[0].close();time.sleep(.005);clients[1].close()
   else:
    clients[1].close();time.sleep(.005);clients[0].close()
   os.killpg(proc.pid,signal.SIGTERM)
   try:rc=proc.wait(timeout=5)
   except subprocess.TimeoutExpired:snapshot('SIGTERM-deadline');raise
   assert rc==0,rc;state=json.loads((root/'service/service-state.json').read_text());assert state['stopped']
  except Exception as exc:error=repr(exc);(root/'failure.txt').write_text(traceback.format_exc());snapshot('failure')
  finally:
   for s in clients:s.close()
   if proc.poll() is None:
    os.killpg(proc.pid,signal.SIGKILL)
    try:proc.wait(timeout=2)
    except subprocess.TimeoutExpired:snapshot('SIGKILL-deadline')
   output.close();results.append(dict(iteration=index,variant=index%3,error=error,returncode=proc.poll(),elapsed_s=(time.monotonic_ns()-started)/1e9));(a.out/'verification.json').write_text(json.dumps(dict(result='pass' if all(x['error'] is None for x in results) else 'fail',iterations=results,limits='SyntheticTCP/SIGTERMraces;passingrepeatsnotproofpriorunobservedtimeouthasrootcausefix/nativehealth.'),indent=2)+'\n')
  if error:raise AssertionError(error)
 print('PASS',len(results),'roomrecipient-close/SIGTERM kernel races;notnativeproof')
if __name__=='__main__':main()
