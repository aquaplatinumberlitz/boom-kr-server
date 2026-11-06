"""Type one declared command in the existing Windows Run dialog and save input evidence."""
from pathlib import Path
import argparse,json,socket,subprocess,time
p=argparse.ArgumentParser();p.add_argument('--command',required=True);p.add_argument('--record',type=Path,required=True);p.add_argument('--execute',action='store_true');a=p.parse_args()
assert not a.record.exists()
s=socket.create_connection(('127.0.0.1',47292),3);s.settimeout(4);f=s.makefile('rwb',buffering=0);hello=json.loads(f.readline());events=[]
def q(c,args=None):
 ident=len(events)+1;f.write((json.dumps(dict(execute=c,arguments=args or {},id=ident))+'\n').encode())
 while True:
  d=json.loads(f.readline())
  if d.get('id')==ident:events.append(dict(command=c,arguments=args,response=d,ns=time.monotonic_ns()));assert 'error' not in d,d;return
def key(k,delay=.1):
 q('human-monitor-command',{'command-line':'sendkey '+k+' 60'});time.sleep(delay)
q('qmp_capabilities');key('esc',.2);key('esc',.2);key('meta_l-r',.7);key('ctrl-a',.2)
symbols={' ':'spc','\\':'backslash','/':'slash','.':'dot','-':'minus','_':'shift-minus',':':'shift-semicolon','"':'shift-apostrophe'}
for char in a.command:
 assert char.isalnum() or char in symbols,char
 key(symbols.get(char,'shift-'+char.lower() if char.isupper() else char))
image=a.record.with_suffix('.ppm');assert not image.exists()
q('screendump',{'filename':'/shared/boom/repos/boom-kr-server/'+str(image)})
if a.execute:key('ret',1)
s.close();subprocess.run(['docker','exec','WinBoat','chmod','644','/shared/boom/repos/boom-kr-server/'+str(image)],check=True)
a.record.write_text(json.dumps(dict(command=a.command,executed=a.execute,events=events,limits='Ordinary QMP keyboard input; guest output/native observations required for completion.'),indent=2)+'\n');print('Typed command; executed:',a.execute)
