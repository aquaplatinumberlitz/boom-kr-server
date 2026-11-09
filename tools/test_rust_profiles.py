"""Prelisten supported-profile/config guards; no native wrong-version error claim."""
import argparse,copy,hashlib,json,os,signal,socket,subprocess,time
from pathlib import Path

def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--binary',type=Path,required=True);p.add_argument('--config',type=Path,required=True);p.add_argument('--out',type=Path,required=True);a=p.parse_args();a.out.mkdir(parents=True,exist_ok=False)
 base=json.loads(a.config.read_text());rows=[]
 for case in ['schema-version','foreign-bootstrap-valid-pin','foreign-WCP-valid-pin','fixture-hash','actor-ID-profile','room-listing-title']:
  root=a.out/case;root.mkdir();cfg=copy.deepcopy(base)
  s=socket.socket();s.bind(('127.0.0.1',0));port=s.getsockname()[1];s.close();cfg.update(bind='127.0.0.1',ports=[port]);cfg['diagnostic_two_lobby'].update(login_port=port,wcp_ports=[port],login_candidate_ports=[],channel_port=port)
  # Endpoint validation must be valid first; reserve distinct ephemeral ports.
  ports={port}
  while len(ports)<3:
   with socket.socket() as s:s.bind(('127.0.0.1',0));ports.add(s.getsockname()[1])
  cfg['ports']=sorted(ports);cfg['diagnostic_two_lobby'].update(login_port=cfg['ports'][0],channel_port=cfg['ports'][1],wcp_ports=cfg['ports'])
  if case=='schema-version':cfg['schema']='unsupported-schema-version'
  elif case.startswith('foreign-'):
   key='bootstrap' if 'bootstrap' in case else 'wcp_second';data=bytearray(Path(cfg[key]['path']).read_bytes());data[-1]^=1;f=root/'different-profile.bin';f.write_bytes(data);cfg[key]=dict(path=str(f.resolve()),bytes=len(data),sha256=hashlib.sha256(data).hexdigest())
  elif case=='fixture-hash':cfg['bootstrap']['sha256']='0'*64
  elif case=='actor-ID-profile':cfg['diagnostic_two_lobby']['actors'][1]['identity']=1;cfg['diagnostic_two_lobby']['actors'][1]['name']='KRLocal'
  else:cfg['diagnostic_two_lobby']['fixed_room']['title']='Different'
  cp=root/'config.json';cp.write_text(json.dumps(cfg,indent=2)+'\n')
  with (root/'stderr.txt').open('x') as output:
   proc=subprocess.Popen([str(a.binary.resolve()),'--config',str(cp),'--out',str(root/'service')],stdout=output,stderr=subprocess.STDOUT,start_new_session=True)
   try:rc=proc.wait(timeout=3)
   finally:
    if proc.poll() is None:os.killpg(proc.pid,signal.SIGKILL);proc.wait(timeout=2)
  assert rc!=0 and not (root/'service/service-state.json').exists()
  for port in cfg['ports']:
   with socket.socket() as s:s.bind(('127.0.0.1',port))
  rows.append(dict(case=case,result='pass',returncode=rc,stderr_path=str(root/'stderr.txt')))
 (a.out/'verification.json').write_text(json.dumps(dict(result='pass',cases=rows,limits='Synthetic operator-config/prelisten validation; foreign-profile bytes not a native malformed/version oracle. No claim about client wrong-version UI, peer authentication or original server policy.'),indent=2)+'\n');print('PASS',len(rows),'prelisten profile/config guards;notnativewrongversionproof')
if __name__=='__main__':main()
