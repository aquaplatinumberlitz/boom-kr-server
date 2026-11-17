#!/usr/bin/env python3
"""Private Wine runtime for g1_native_harness; actions come from a pinned matrix."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import selectors
import signal
import socket
import subprocess
import sys
import threading
import shutil
import time


REPO = Path(__file__).resolve().parents[1]
LOCAL = (REPO / "local").resolve()
SUPPORT = LOCAL / "research/codex"

def cleanup_owned_namespace(timeout_seconds=3):
    # Both PID APIs must see this process as namespace init. A one-UID user
    # mapping and private procfs are mandatory; never operate on the host procfs.
    assert os.getpid() == 1 and os.getuid() == 0
    assert int(Path('/proc/self/stat').read_text().split()[0]) == 1
    uid_map = Path('/proc/self/uid_map').read_text().split()
    assert len(uid_map) == 3 and uid_map[0] == '0' and uid_map[2] == '1'
    before = []
    for proc in Path('/proc').iterdir():
        if not proc.name.isdigit() or proc.name == '1':
            continue
        try:
            before.append(dict(pid=int(proc.name), cmdline=(proc/'cmdline').read_bytes().replace(b'\0',b' ').decode(errors='replace')))
        except FileNotFoundError:
            pass
    reaped = []
    def reap():
        while True:
            try:
                pid, status = os.waitpid(-1, os.WNOHANG)
            except ChildProcessError:
                break
            if not pid:
                break
            reaped.append(dict(pid=pid, status=status))
    def remaining():
        reap()
        return [int(p.name) for p in Path('/proc').iterdir() if p.name.isdigit() and p.name != '1']
    sent = []
    for sig in [signal.SIGTERM, signal.SIGKILL]:
        for pid in remaining():
            try:
                os.kill(pid, sig)
                sent.append(dict(pid=pid, signal=int(sig)))
            except ProcessLookupError:
                pass
        end = time.monotonic()+timeout_seconds
        while time.monotonic()<end and remaining():
            time.sleep(.02)
        if not remaining():
            break
    left = remaining()
    assert not left, ('owned namespace descendants remain', left)
    return dict(result='all_owned_namespace_descendants_reaped', before=before, signals=sent, reaped=reaped, remaining=left, limits='Private one-UID/PID1 namespace only; called after ordinary Wine/service/capture/topology cleanup. Not a host-wide process cleanup or durability claim.')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--game-dir', type=Path, required=True)
    parser.add_argument('--prefix', type=Path, required=True)
    parser.add_argument('--out', type=Path, required=True)
    parser.add_argument('--exe', default='CA.exe')
    parser.add_argument('--seconds', type=int, default=15)
    parser.add_argument('--second-response', type=Path, help='Diagnostic response after first C2S on second connection')
    parser.add_argument('--client-arg', action='append', default=[], help='Exact argument for private diagnostic copy')
    parser.add_argument('--debug', action='store_true', help='Use winedbg and record first exception backtrace')
    parser.add_argument('--commands', type=Path, help='Custom winedbg command file')
    parser.add_argument('--trace-api', action='store_true', help='Record Wine relay calls for this bounded run')
    parser.add_argument('--map-host', action='append', default=[], help='Resolve this DNS name to loopback inside the namespace')
    parser.add_argument('--listen', type=int, action='append', default=[], help='Local TCP accept/capture port; silent unless --bootstrap-reply is supplied')
    parser.add_argument('--bootstrap-local', action='store_true', help='Map observed KR bootstrap names and listen on candidate local ports3808..3998 step10')
    parser.add_argument('--bootstrap-reply', type=Path, help='Send this local fixture once on the first bootstrap connection; max65536 bytes')
    parser.add_argument('--inside', action='store_true', help=argparse.SUPPRESS)
    parser.add_argument('--pair-config', type=Path, required=True)
    args = parser.parse_args()
    args.pair_config=args.pair_config.resolve(strict=True)
    config=json.loads(args.pair_config.read_text())
    if args.bootstrap_local:
        args.map_host = sorted(set(args.map_host+['lslist.crazyarcade.com','happy-login-ca.nexon.com']))
        args.listen = sorted(set(args.listen+list(range(3808,4000,10))))
    if not 1 <= args.seconds <= 3600:
        parser.error('--seconds must be 1..3600')
    args.game_dir = args.game_dir.resolve(strict=True)
    args.prefix = args.prefix.resolve()
    args.out = args.out.resolve()
    executable = (args.game_dir / args.exe).resolve(strict=True)
    if executable.parent != args.game_dir:
        parser.error('--exe must name a file directly inside --game-dir')
    local = LOCAL
    if any(not path.is_relative_to(local) for path in [args.game_dir,args.prefix,args.out]):
        parser.error('game, prefix and output must be copies under this repo local/')
    paths=[args.game_dir,args.prefix,args.out]
    if any(left.is_relative_to(right) for i,left in enumerate(paths) for j,right in enumerate(paths) if i!=j):
        parser.error('game, prefix and output must be separate directory trees')
    if args.commands:
        args.commands=args.commands.resolve(strict=True)
        if not args.debug or not args.commands.is_file() or not args.commands.is_relative_to(local):
            parser.error('--commands requires --debug and a file under this repo local/')
    for name in args.map_host:
        if not name or any(c not in 'abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789-.' for c in name):
            parser.error('invalid DNS name')
    if any(not 1 <= port <= 65535 for port in args.listen):
        parser.error('invalid TCP port')
    reply = None
    if args.bootstrap_reply:
        try:
            args.bootstrap_reply = args.bootstrap_reply.resolve(strict=True)
        except OSError as error:
            parser.error(f'invalid bootstrap reply: {error}')
        if not args.bootstrap_local or not args.bootstrap_reply.is_file() or not args.bootstrap_reply.is_relative_to(local):
            parser.error('--bootstrap-reply requires --bootstrap-local and a file under this repo local/')
        with args.bootstrap_reply.open('rb') as source:
            reply = source.read(65537)
        if not 1 <= len(reply) <= 65536:
            parser.error('bootstrap reply must be 1..65536 bytes')
    if args.second_response:
        args.second_response = args.second_response.resolve(strict=True)
        if not args.second_response.is_file() or not args.second_response.is_relative_to(local):
            parser.error('--second-response must be a file under local/')
        next_reply = args.second_response.read_bytes()
        if not 1 <= len(next_reply) <= 65536:
            parser.error('invalid second-response size')
    else:
        next_reply = None
    if not args.inside:
        # UID mapping gives mount privileges only in this disposable user namespace.
        command = ['unshare', '--user', '--map-root-user', '--net', '--mount',
                   '--pid', '--fork', '--mount-proc', '--kill-child=KILL',
                   sys.executable, str(Path(__file__).resolve()), '--inside',
                   '--game-dir', str(args.game_dir), '--prefix', str(args.prefix),
                   '--out', str(args.out), '--exe', args.exe, '--seconds', str(args.seconds)]
        command.extend(['--pair-config',str(args.pair_config)])
        if args.second_response:
            command.extend(['--second-response', str(args.second_response)])
        for client_arg in args.client_arg:
            command.extend(['--client-arg', client_arg])
        if args.debug:
            command.append('--debug')
        if args.commands:
            command.extend(['--commands', str(args.commands.resolve(strict=True))])
        if args.trace_api:
            command.append('--trace-api')
        if args.bootstrap_reply:
            command.extend(['--bootstrap-local', '--bootstrap-reply', str(args.bootstrap_reply)])
        for name in args.map_host:
            command.extend(['--map-host', name])
        for port in args.listen:
            command.extend(['--listen', str(port)])
        return subprocess.call(command)
    uid_map=Path('/proc/self/uid_map').read_text().split()
    if os.getpid()!=1 or os.getuid()!=0 or len(uid_map)!=3 or uid_map[0]!='0' or uid_map[2]!='1':
        raise RuntimeError('--inside requires the private user/PID namespace created by this tool')
    import fcntl
    runtime_lease=(SUPPORT/'.kr-wine-runtime.lock').open('a+')
    fcntl.flock(runtime_lease.fileno(),fcntl.LOCK_EX|fcntl.LOCK_NB)
    args.out.mkdir(parents=True, exist_ok=False)
    subprocess.run(['mount', '--make-rprivate', '/'], check=True)
    raw_export=args.out.with_name(args.out.name+'-raw-export');assert not raw_export.exists()
    subprocess.run(['mount','-t','tmpfs','-o','size=512m,mode=700','kr-evidence-tmpfs',str(args.out)],check=True)
    # Persistent private output; /tmp, /run and network remain isolated.
    subprocess.run(['mount', '-t', 'tmpfs', '-o', 'size=128m', 'tmpfs', '/tmp'], check=True)
    # NSS resolve can use AF_UNIX on the host even with a private network namespace.
    # Hide host service sockets and bind files-only host resolution for this run.
    subprocess.run(['mount', '-t', 'tmpfs', '-o', 'size=16m', 'tmpfs', '/run'], check=True)
    nss = args.out / 'nsswitch.conf'
    text = Path('/etc/nsswitch.conf').read_text()
    nss.write_text('\n'.join(line for line in text.splitlines() if not line.lstrip().startswith('hosts:'))+'\nhosts: files\n')
    subprocess.run(['mount', '--bind', str(nss), '/etc/nsswitch.conf'], check=True)
    subprocess.run(['ip', 'link', 'set', 'lo', 'up'], check=True)
    interfaces = socket.if_nameindex()
    if [name for _, name in interfaces] != ['lo']:
        raise RuntimeError('network namespace must contain only loopback')
    routes=Path('/proc/net/route').read_text()
    if any(line.strip() for line in routes.splitlines()[1:]):
        raise RuntimeError('network namespace must have no IPv4 routes')
    if args.map_host:
        hosts = args.out / 'hosts'
        hosts.write_text('127.0.0.1 localhost\n10.77.0.1 ' + ' '.join(args.map_host) + '\n::1 localhost\n')
        subprocess.run(['mount', '--bind', str(hosts), '/etc/hosts'], check=True)
    env = os.environ.copy()
    for key in ('DISPLAY', 'WAYLAND_DISPLAY', 'XAUTHORITY', 'DBUS_SESSION_BUS_ADDRESS',
                'DBUS_STARTER_ADDRESS', 'SESSION_MANAGER', 'WINEDLLOVERRIDES',
                'WINELOADER', 'WINESERVER'):
        env.pop(key, None)
    runtime = args.out / 'runtime'
    runtime.mkdir(mode=0o700)
    env['ALSA_CONFIG_PATH'] = str(SUPPORT / 'nmco-identity-r1/alsa-null.conf')
    env.update(WINEPREFIX=str(args.prefix), WINEARCH='win64', WINEDEBUG='+seh,+timestamp,+pid',
               LIBGL_ALWAYS_SOFTWARE='1', MESA_LOADER_DRIVER_OVERRIDE='llvmpipe',
               GALLIUM_DRIVER='llvmpipe', XDG_RUNTIME_DIR=str(runtime), TMPDIR='/tmp', LANG='C.UTF-8')
    record = {'schema': 'kre-wine-probe-r1-args', 'client_args':args.client_arg, 'exe': str(executable),
              'harness_sha256': hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
              'sha256': hashlib.sha256(executable.read_bytes()).hexdigest(),
              'wine': subprocess.check_output(['wine', '--version'], env=env, text=True).strip(),
              'net_namespace': os.readlink('/proc/self/ns/net'), 'interfaces': interfaces,
              'ipv4_routes': routes,
              'start_utc': time.strftime('%Y-%m-%dT%H:%M:%SZ', time.gmtime()),
              'start_monotonic_ns': time.monotonic_ns(), 'observations': []}
    record['nsswitch_sha256']=hashlib.sha256(Path('/etc/nsswitch.conf').read_bytes()).hexdigest()
    record['host_resolution_files_only']=[line.strip() for line in Path('/etc/nsswitch.conf').read_text().splitlines() if line.lstrip().startswith('hosts:')]==['hosts: files']
    if not record['host_resolution_files_only']:
        raise RuntimeError('files-only NSS was not installed')
    record['host_map'] = {name: '10.77.0.1' for name in args.map_host}
    record['host_resolution'] = {}
    for name in args.map_host:
        result = subprocess.run(['getent','hosts',name],text=True,stdout=subprocess.PIPE,stderr=subprocess.STDOUT,timeout=3)
        record['host_resolution'][name] = {'returncode':result.returncode,'output':result.stdout}
        if result.returncode or not result.stdout.strip() or any(line.split()[0]!='10.77.0.1' for line in result.stdout.splitlines()):
            raise RuntimeError('loopback name resolution verification failed')
    record['bootstrap_reply'] = {'path':str(args.bootstrap_reply), 'size':len(reply), 'sha256':hashlib.sha256(reply).hexdigest()} if reply else None
    record['second_response'] = {'path':str(args.second_response),'sha256':hashlib.sha256(next_reply).hexdigest(),'size':len(next_reply)} if next_reply else None
    record['tcp_events']=[]
    stop=threading.Event()
    listeners=[]
    listener_sockets=[]
    children, streams = [], []
    sys.path.insert(0,str(args.pair_config.parent))
    from topology import Topology
    from capture_tcp import Capture
    from capture_udp import Capture as UdpCapture
    sys.path.insert(0,str(REPO))
    from server import probe_codec as codec
    topology=None;capture=None;udp_capture=None;states={};service=None
    record['pair_scope']=config['scope'];record['role_args']={role:x['args'] for role,x in config['roles'].items()}
    def spawn(cmd,name,child_env=None,cwd=None,pass_fds=(),input_file=None,role=None):
        ee=child_env if child_env is not None else env
        if role is not None and cmd[0] in ['wine','wineboot','winedbg','wineserver']:cmd=topology.command(role,cmd)
        stream=(args.out/(name+'.log')).open('w');streams.append(stream)
        process=subprocess.Popen(cmd,env=ee,cwd=cwd,stdout=stream,stderr=subprocess.STDOUT,start_new_session=True,pass_fds=pass_fds,stdin=input_file)
        children.append(process);return process
    def channel_frames(event):
        buf=bytearray(b''.join(bytes.fromhex(x['hex']) for x in event['chunks']));ops=[];seq=40
        while (f:=codec.pop_frame(buf)) is not None:
            codec.frame_body(f,seq);ops.append(int.from_bytes(f[5:9],'little'));seq+=3
        return ops
    def request_stop(signum, frame):
        raise RuntimeError('Declared stop signal '+str(signum))
    signal.signal(signal.SIGINT, request_stop)
    signal.signal(signal.SIGTERM, request_stop)
    observer_stop=observer_thread=None
    try:
        topology=Topology(args.out,args.prefix);capture=Capture(args.out);udp_capture=UdpCapture(args.out)
        record['topology_scope']=config['scope']
        # Prepare every isolated Wine prefix before service and account admission.
        for role,c in config['roles'].items():
            game=Path(c['game_dir']);pre=Path(c['prefix']);assert game.is_relative_to(local) and pre.is_relative_to(local) and game.is_dir() and pre.is_dir()
            ee=env.copy();ee['WINEPREFIX']=str(pre);rr=runtime/role;rr.mkdir(mode=0o700);ee['XDG_RUNTIME_DIR']=str(rr)
            ro=args.out/role;ro.mkdir();st={'role':role,'_env':ee,'out':ro,'game':game,'observations':[]};states[role]=st
            log=game/'CALog.log'
            if log.exists():(ro/'CALog-before.txt').write_bytes(log.read_bytes());log.unlink()
            rf,wf=os.pipe();spawn(['Xvfb','-displayfd',str(wf),'-screen','0','1024x768x24','-nolisten','tcp'],'xvfb-'+role,child_env=ee,pass_fds=(wf,));os.close(wf)
            with selectors.DefaultSelector() as sel:
                sel.register(rf,selectors.EVENT_READ);assert sel.select(8),'display timeout';number=os.read(rf,32).decode().strip()
            os.close(rf);assert number.isdigit();ee['DISPLAY']=':'+number
            boot=spawn(['wineboot','-u'],'wineboot-'+role,child_env=ee,role=role);assert boot.wait(timeout=90)==0,'wineboot failed'
            ac=subprocess.run(topology.command(role,['wine','reg','add',r'HKCU\Software\Wine\Drivers','/v','Audio','/t','REG_SZ','/d','','/f']),env=ee,capture_output=True,text=True,timeout=12)
            (ro/'audio-config.json').write_text(json.dumps(dict(returncode=ac.returncode,stdout=ac.stdout,stderr=ac.stderr),indent=2)+'\n');assert ac.returncode==0
            commands=ro/'debug-commands.txt';commands.write_text(Path(config['commands_by_role'][role]).read_text())
        record['preboot_complete_ns']=time.monotonic_ns()
        repo=REPO
        dbdir=args.out/'account-db';dbdir.mkdir(mode=0o700);subprocess.run(['mount','-t','tmpfs','-o','size=32m,mode=0700','tmpfs',str(dbdir)],check=True);record['db_storage_control']=dict(mode='private_tmpfs_32MiB',directory_mode=oct(dbdir.stat().st_mode&0o777),mounted_ns=time.monotonic_ns(),limits='Diagnostic only; no durability claim');shutil.copyfile(Path(config['rust_config']).parent/'previous-accounts.sqlite',dbdir/'accounts.sqlite');os.chmod(dbdir/'accounts.sqlite',0o600)
        api_parent=Path(config['launch_socket']).parent;api_parent.mkdir(mode=0o700,exist_ok=True)
        backend_calls=[]
        def account_admin(command):
            req=dict(actor='M5-native-runtime',reason='Four account profiles for M7a selection in private TMPFS',command=command)
            result=subprocess.run([str(Path(config['rust_config']).parent/'binary/kr-admin'),str(dbdir/'accounts.sqlite')],input=json.dumps(req),capture_output=True,text=True,timeout=10)
            backend_calls.append(dict(request=req,returncode=result.returncode,stdout=result.stdout,stderr=result.stderr))
            record['backend_seed']=backend_calls
            (args.out/'private-backend-seed.json').write_text(json.dumps(backend_calls,indent=2)+'\n')
            assert result.returncode==0,'private backend seed failed';return json.loads(result.stdout)
        account_admin(dict(op='init'))
        cmd=[str(Path(config['rust_config']).parent/'binary/kr-service'),'--config',config['rust_config'],'--out',str(args.out)]
        service=spawn(cmd,'service',cwd=repo);record['external_service']={'pid':service.pid,'argv':cmd,'limits':'Candidate M7e players; acceptance requires independent native/PCAP/DB/raw verification.'}
        for _ in range(100):
            f=args.out/'service-state.json'
            if f.exists() and json.loads(f.read_text()).get('ready'):break
            if service.poll() is not None:raise RuntimeError('service exited before ready')
            time.sleep(.05)
        else:raise RuntimeError('service ready timeout')
        for role,c in config['roles'].items():
            c['player']=account_admin(dict(op='lookup',login=c['backend_login']))['player']
            issued=account_admin(dict(op='login',login=c['backend_login'],password='native-private-test-password',ttl_seconds=1200))
            c['backend_session']=issued['token'];c['player_id']=issued['player']['player_id']
        from importlib.util import spec_from_file_location,module_from_spec
        spec=spec_from_file_location('g1_passive',Path(config['rust_config']).parent/'runtime_observer.py');mod=module_from_spec(spec);spec.loader.exec_module(mod)
        observer_stop,observer_thread=mod.start_observer(args.out,states,service)
        for role,c in config['roles'].items():
            st=states[role];game=st['game'];ee=st['_env'];ro=st['out'];commands=ro/'debug-commands.txt'
            request=dict(session=c['backend_session'],player_id=c['player_id'],peer='10.77.0.99',reported_login_ipv4=c['ip'])
            with socket.socket(socket.AF_UNIX) as api:
                api.settimeout(4);api.connect(config['launch_socket']);api.sendall(json.dumps(request).encode()+b'\n');data=b''
                while b'\n' not in data:
                    part=api.recv(4096);assert part,'launch control EOF';data+=part
            issued=json.loads(data);assert issued.get('ok')and len(issued.get('frontend',''))==32, ('launch result',issued.get('error'),issued.get('reason'),len(issued.get('frontend','')))
            # Runtime-derived offer is stdin-only, never process argv or source/config.
            offer=json.dumps(dict(peer_transport=issued['peer_transport'],native_ip='127.0.0.1',port_budget=128)).encode()
            with __import__('tempfile').TemporaryFile()as inp:
                inp.write(offer);inp.seek(0)
                peer=spawn(topology.command(role,[str(Path(config['rust_config']).parent/'binary/kr-companion')]),'companion-'+role,input_file=inp)
            for attempt in range(100):
                if '"event":"transport_ready"'in(args.out/('companion-'+role+'.log')).read_text():break
                assert peer.poll()is None,'companion authentication failed'
                time.sleep(.05)
            else:raise RuntimeError('companion connect deadline')
            spawn(topology.command(role,['python3',str(Path(config['rust_config']).parent/'loopback_capture.py'),str(args.out/('loopback-'+role+'.pcap'))]),'loopback-capture-'+role)

            c['args']=['multiserveridx:0','token_1:'+issued['frontend']]
            record['role_args'][role]=list(c['args'])
            record.setdefault('account_launches',{})[role]=dict(response=issued,monotonic_ns=time.monotonic_ns(),limits='Runtime-derived private grant from frozen backend session input; no source/config file changed.')
            record.setdefault('actual_launch_inputs',{})[role]=dict(argv_sha256=hashlib.sha256(json.dumps(c['args'],separators=(',',':')).encode()).hexdigest(),token_chars=32)
            with commands.open() as inp:st['_process']=spawn(['winedbg','--gdb',str(game/args.exe),*c['args']],role+'/client',child_env=ee,cwd=game,input_file=inp,role=role)
            st['started_ns']=time.monotonic_ns();st['exe_sha256']=hashlib.sha256((game/args.exe).read_bytes()).hexdigest()
        started=time.monotonic();ready_ns=None;actions={};hold_ns=None
        def snap(label):
            for role,st in states.items():subprocess.run(['import','-window','root',str(st['out']/(label+'.png'))],env=st['_env'],capture_output=True,timeout=5)
        def click(role,x,y,label,double=False):
            st=states[role];ee=st['_env'];ids=subprocess.check_output(['xdotool','search','--onlyvisible','--name','^Crazy Arcade$'],env=ee,text=True).split();assert len(ids)==1,ids;wid=ids[0]
            focus=subprocess.run(['xdotool','windowfocus','--sync',wid],env=ee,capture_output=True,timeout=3)
            # Leave and re-enter the button: the native UI derives hover from
            # mouse motion, including after returning from the result screen.
            cmd=['xdotool','mousemove','--window',wid,'10','10','sleep','.15',
                 'mousemove','--window',wid,str(x),str(y),'sleep','.15']
            if double:
                cmd+=['click','--repeat','2','--delay','150','1']
            else:
                cmd+=['mousedown','1','sleep','.2','mouseup','1']
            inp=subprocess.run(cmd,env=ee,capture_output=True,timeout=4)
            inp.check_returncode()
            actions[label]=dict(role=role,window=wid,x=x,y=y,ns=time.monotonic_ns(),returncode=inp.returncode)
        def press(role,key,seconds):
            st=states[role];wid=subprocess.check_output(['xdotool','search','--onlyvisible','--name','^Crazy Arcade$'],env=st['_env'],text=True).split();assert len(wid)==1
            subprocess.run(['xdotool','windowfocus','--sync',wid[0],'keydown',key],env=st['_env'],check=True,timeout=3);before=time.monotonic_ns()
            try:time.sleep(seconds)
            finally:subprocess.run(['xdotool','keyup',key],env=st['_env'],check=True,timeout=3)
            item=dict(role=role,key=key,before_ns=before,after_ns=time.monotonic_ns());record.setdefault('key_inputs',[]).append(item);return item
        def align_role(role,slot,field,target,label):
            item=dict(role=role,slot=slot,label=label,field=field,target=target,samples=[],inputs=[]);record.setdefault('ray_alignments',[]).append(item)
            for attempt in range(36):
                after=time.monotonic_ns();deadline=time.monotonic()+8;seen=-1;stable=[]
                while time.monotonic()<deadline:
                    p=args.out/('live-'+role+'.json')
                    if p.exists():
                        sample=json.loads(p.read_text());ns=sample['monotonic_ns'];value=sample['actors'][str(slot)][field]
                        if ns>after and ns>seen:
                            seen=ns
                            if stable and stable[-1][1]!=value:stable=[]
                            stable.append((ns,value))
                            if len(stable)>=3:break
                    time.sleep(.02)
                else:raise RuntimeError('fresh position absent '+role+' '+label)
                item['samples'].append(sample)
                if abs(value-target)<=2:item['reached']=True;return
                key=('Up'if value>target else'Down')if field=='0x58'else('Left'if value>target else'Right');item['inputs'].append(press(role,key,.008 if abs(value-target)<15 else .02))
            raise RuntimeError('bounded alignment miss '+role+' '+label)
        def move_role(role,row,col,label):
            slot=0 if role=='A'else 1
            align_role(role,slot,'0x4c',40*col,label+'-column')
            align_role(role,slot,'0x58',40*row+39,label+'-row')
        from g1_native_harness import drive_matrix
        record['actions']=actions
        matrix=json.loads(Path(config['matrix']).read_text())
        drive_matrix(matrix,config,args.out,states,record,topology,click,press,align_role,snap,args.seconds)

    except Exception as error:
        record['error']=repr(error)
        if 'snap' in locals():
            try:snap('matrix-failure')
            except Exception as snapshot_error:record['failure_snapshot_error']=repr(snapshot_error)
    finally:
        if observer_stop is not None:
            observer_stop.set();observer_thread.join(timeout=12)
            if observer_thread.is_alive():record.setdefault('cleanup_errors',[]).append('passive observer did not stop')
        signal.signal(signal.SIGINT, signal.SIG_IGN)
        signal.signal(signal.SIGTERM, signal.SIG_IGN)
        for role,st in states.items():
            try:subprocess.run(['xdotool','keyup','space','Down','Right','Up','Left'],env=st['_env'],capture_output=True,timeout=2)
            except Exception:pass
            log=st['game']/'CALog.log'
            if log.exists():(st['out']/'CALog-run.txt').write_bytes(log.read_bytes())
            try:subprocess.run(topology.command(role,['wineserver','-k']),env=st['_env'],capture_output=True,timeout=4)
            except Exception as error:record.setdefault('cleanup_errors',[]).append(repr(error))
        for proc in reversed(children):
            try:
                if proc.poll() is None:os.killpg(proc.pid,signal.SIGTERM)
                proc.wait(timeout=3)
            except ProcessLookupError:pass
            except subprocess.TimeoutExpired:
                try:os.killpg(proc.pid,signal.SIGKILL)
                except ProcessLookupError:pass
                proc.wait(timeout=2)
        if record.get('db_storage_control'):
            db_export=args.out/'account-db-retained';shutil.copytree(dbdir,db_export)
            record['db_storage_control']['exported_after_children_ns']=time.monotonic_ns()
            subprocess.run(['umount',str(dbdir)],check=True)
            shutil.copytree(db_export,dbdir,dirs_exist_ok=True)
            record['db_storage_control']['unmounted_ns']=time.monotonic_ns()
            record['db_storage_control']['retained_closed_files']=[dict(name=p.name,bytes=p.stat().st_size,sha256=hashlib.sha256(p.read_bytes()).hexdigest())for p in db_export.iterdir()if p.is_file()]
        for stream in streams:stream.close()
        if capture is not None:
            try: capture.stop()
            except BaseException as exc: record.setdefault('cleanup_errors',[]).append('TCP capture stop: '+repr(exc))
            if getattr(capture,'failure',None):record['error']=record.get('error')or('TCPcaptureworkerfailed:'+capture.failure)
        if udp_capture is not None:
            try: udp_capture.stop()
            except BaseException as exc: record.setdefault('cleanup_errors',[]).append('UDP capture stop: '+repr(exc))
            if getattr(udp_capture,'failure',None):record['error']=record.get('error')or('UDPcaptureworkerfailed:'+udp_capture.failure)
        record['roles']={role:dict(role=role,prefix=st['_env']['WINEPREFIX'],display=st['_env'].get('DISPLAY'),game_dir=str(st['game']),started_ns=st.get('started_ns'),exe_sha256=st.get('exe_sha256'),observations=st['observations']) for role,st in states.items()}
        sp=args.out/'service-state.json'
        if sp.exists():
            state=json.loads(sp.read_text());record['tcp_events']=state.get('tcp_events',[]);record['service_errors']=state.get('errors',[]);record['service_stopped']=state.get('stopped',False)
        if topology is not None:
            topology.stop()
            assert os.getpid()==1 and Path('/proc/self/uid_map').read_text().split()[2]=='1'
            record['owned_bind_unmounts']=[]
            for target,owned_source in [('/etc/hosts',args.out/'hosts'),('/etc/nsswitch.conf',args.out/'nsswitch.conf')]:
                if not owned_source.exists():continue
                subprocess.run(['umount',target],check=True,capture_output=True,text=True,timeout=3)
                record['owned_bind_unmounts'].append(target)
        try:
            record['owned_orphan_cleanup']=cleanup_owned_namespace()
        except Exception as cleanup_error:
            record.setdefault('cleanup_errors',[]).append('Owned namespace orphan cleanup: '+repr(cleanup_error))
        record['end_monotonic_ns']=time.monotonic_ns();(args.out/'run.json').write_text(json.dumps(record,indent=2)+'\n')
        # All Wine/service/capture resources are already closed. Preserve raw bytes.
        shutil.copytree(args.out,raw_export)
        export_rows=[]
        for source in sorted(args.out.rglob('*')):
            if source.is_file():
                dest=raw_export/source.relative_to(args.out);size=source.stat().st_size;digest=hashlib.sha256(source.read_bytes()).hexdigest()
                assert dest.stat().st_size==size and hashlib.sha256(dest.read_bytes()).hexdigest()==digest
                export_rows.append(dict(path=str(source),bytes=size,sha256=digest))
        (args.out.parent/'storage-export-before-unmount.json').write_text(json.dumps(dict(result='same_bytes_verified',raw_export=str(raw_export),files=export_rows,limits='Owned runtime cleanup/export complete; unmount not yet certified.'),indent=2)+chr(10))
        for unmount_attempt in range(50):
            unmounted=subprocess.run(['umount',str(args.out)],capture_output=True,text=True,timeout=3)
            if unmounted.returncode==0:break
            if 'target is busy'not in unmounted.stderr:raise RuntimeError('unmount failure '+unmounted.stderr)
            if unmount_attempt==0:
                diag=dict(ns=time.monotonic_ns(),result='observed_output_umount_busy',stderr=unmounted.stderr,mountinfo=Path('/proc/self/mountinfo').read_text(),processes=[])
                for proc in Path('/proc').iterdir():
                    if not proc.name.isdigit():continue
                    try:
                        item=dict(pid=proc.name,cmd=(proc/'cmdline').read_bytes().replace(b'\0',b' ').decode(errors='replace'),cwd=os.readlink(proc/'cwd'),fds=[])
                        for fd in list((proc/'fd').iterdir())[:64]:
                            try:
                                link=os.readlink(fd)
                                if str(args.out)in link:item['fds'].append(dict(fd=fd.name,target=link))
                            except OSError:pass
                        item['wchan']=(proc/'wchan').read_text();diag['processes'].append(item)
                    except OSError:pass
                (args.out.parent/'output-unmount-busy.json').write_text(json.dumps(diag,indent=2)+'\n')

            time.sleep(.1)
        else:raise RuntimeError('bounded unmount busy after same-byte export; retained export locator recorded')
        args.out.rmdir() # original freshly-created empty underlying directory only
        raw_export.rename(args.out)
        (args.out.parent/'storage-export.json').write_text(json.dumps(dict(result='pass',files=export_rows,limits='Samebytesexportafterruntimecleanup/privateTMPFSunmounted;notcrash-durableevidencelog orBtrfsfix.'),indent=2)+'\n')

    return 1 if record.get('error') or record.get('cleanup_errors') else 0

if __name__=='__main__':raise SystemExit(main())
