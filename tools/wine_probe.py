#!/usr/bin/env python3
"""Run one copied client under Wine in private network/mount/PID namespaces."""
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
import time


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--game-dir', type=Path, required=True)
    parser.add_argument('--prefix', type=Path, required=True)
    parser.add_argument('--out', type=Path, required=True)
    parser.add_argument('--exe', default='CA.exe')
    parser.add_argument('--seconds', type=int, default=15)
    parser.add_argument('--debug', action='store_true', help='Use winedbg and record first exception backtrace')
    parser.add_argument('--commands', type=Path, help='Custom winedbg command file')
    parser.add_argument('--trace-api', action='store_true', help='Record Wine relay calls for this bounded run')
    parser.add_argument('--map-host', action='append', default=[], help='Resolve this DNS name to loopback inside the namespace')
    parser.add_argument('--listen', type=int, action='append', default=[], help='Local TCP accept/capture port; sends no reply')
    parser.add_argument('--bootstrap-local', action='store_true', help='Map observed KR bootstrap names and listen on candidate local ports3808..3998 step10')
    parser.add_argument('--inside', action='store_true', help=argparse.SUPPRESS)
    args = parser.parse_args()
    if args.bootstrap_local:
        args.map_host = sorted(set(args.map_host+['lslist.crazyarcade.com','happy-login-ca.nexon.com']))
        args.listen = sorted(set(args.listen+list(range(3808,4000,10))))
    if not 1 <= args.seconds <= 120:
        parser.error('--seconds must be 1..120')
    args.game_dir = args.game_dir.resolve(strict=True)
    args.prefix = args.prefix.resolve()
    args.out = args.out.resolve()
    executable = (args.game_dir / args.exe).resolve(strict=True)
    if executable.parent != args.game_dir:
        parser.error('--exe must name a file directly inside --game-dir')
    local = Path(__file__).resolve().parent.parent / 'local'
    if any(not path.is_relative_to(local) for path in [args.game_dir,args.prefix,args.out]):
        parser.error('game, prefix and output must be copies under this repo local/')
    if args.prefix == args.out or args.prefix.is_relative_to(args.game_dir) or args.out.is_relative_to(args.game_dir):
        parser.error('prefix/output must be outside game directory')
    if not args.inside:
        # UID mapping gives mount privileges only in this disposable user namespace.
        command = ['unshare', '--user', '--map-root-user', '--net', '--mount',
                   '--pid', '--fork', '--mount-proc', '--kill-child=KILL',
                   sys.executable, str(Path(__file__).resolve()), '--inside',
                   '--game-dir', str(args.game_dir), '--prefix', str(args.prefix),
                   '--out', str(args.out), '--exe', args.exe, '--seconds', str(args.seconds)]
        if args.debug:
            command.append('--debug')
        if args.commands:
            command.extend(['--commands', str(args.commands.resolve(strict=True))])
        if args.trace_api:
            command.append('--trace-api')
        for name in args.map_host:
            command.extend(['--map-host', name])
        for port in args.listen:
            command.extend(['--listen', str(port)])
        return subprocess.call(command)
    uid_map=Path('/proc/self/uid_map').read_text().split()
    if os.getpid()!=1 or os.getuid()!=0 or len(uid_map)!=3 or uid_map[0]!='0' or uid_map[2]!='1':
        raise RuntimeError('--inside requires the private user/PID namespace created by this tool')
    args.out.mkdir(parents=True, exist_ok=False)
    subprocess.run(['mount', '--make-rprivate', '/'], check=True)
    subprocess.run(['mount', '-t', 'tmpfs', '-o', 'size=128m', 'tmpfs', '/tmp'], check=True)
    # NSS resolve can use AF_UNIX on the host even with a private network namespace.
    # Hide host service sockets and bind files-only host resolution for this run.
    subprocess.run(['mount', '-t', 'tmpfs', '-o', 'size=16m', 'tmpfs', '/run'], check=True)
    nss = args.out / 'nsswitch.conf'
    text = Path('/etc/nsswitch.conf').read_text()
    nss.write_text('\n'.join('hosts: files' if line.startswith('hosts:') else line for line in text.splitlines())+'\n')
    subprocess.run(['mount', '--bind', str(nss), '/etc/nsswitch.conf'], check=True)
    subprocess.run(['ip', 'link', 'set', 'lo', 'up'], check=True)
    interfaces = socket.if_nameindex()
    if [name for _, name in interfaces] != ['lo']:
        raise RuntimeError('network namespace must contain only loopback')
    for name in args.map_host:
        if not name or any(c not in 'abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789-.' for c in name):
            parser.error('invalid DNS name')
    if any(not 1 <= port <= 65535 for port in args.listen):
        parser.error('invalid TCP port')
    if args.map_host:
        hosts = args.out / 'hosts'
        hosts.write_text('127.0.0.1 localhost ' + ' '.join(args.map_host) + '\n::1 localhost\n')
        subprocess.run(['mount', '--bind', str(hosts), '/etc/hosts'], check=True)
    env = os.environ.copy()
    for key in ('DISPLAY', 'WAYLAND_DISPLAY', 'XAUTHORITY', 'DBUS_SESSION_BUS_ADDRESS',
                'DBUS_STARTER_ADDRESS', 'SESSION_MANAGER', 'WINEDLLOVERRIDES',
                'WINELOADER', 'WINESERVER'):
        env.pop(key, None)
    runtime = args.out / 'runtime'
    runtime.mkdir(mode=0o700)
    env.update(WINEPREFIX=str(args.prefix), WINEARCH='win64', WINEDEBUG='+seh,+winsock,+timestamp,+pid',
               LIBGL_ALWAYS_SOFTWARE='1', MESA_LOADER_DRIVER_OVERRIDE='llvmpipe',
               GALLIUM_DRIVER='llvmpipe', XDG_RUNTIME_DIR=str(runtime), TMPDIR='/tmp', LANG='C.UTF-8')
    record = {'schema': 'kre-wine-probe-r1', 'exe': str(executable),
              'sha256': hashlib.sha256(executable.read_bytes()).hexdigest(),
              'wine': subprocess.check_output(['wine', '--version'], env=env, text=True).strip(),
              'net_namespace': os.readlink('/proc/self/ns/net'), 'interfaces': interfaces,
              'ipv4_routes': Path('/proc/net/route').read_text(),
              'start_utc': time.strftime('%Y-%m-%dT%H:%M:%SZ', time.gmtime()),
              'start_monotonic_ns': time.monotonic_ns(), 'observations': []}
    record['host_map'] = {name: '127.0.0.1' for name in args.map_host}
    record['host_resolution'] = {}
    for name in args.map_host:
        result = subprocess.run(['getent','hosts',name],text=True,stdout=subprocess.PIPE,stderr=subprocess.STDOUT,timeout=3)
        record['host_resolution'][name] = {'returncode':result.returncode,'output':result.stdout}
        if result.returncode or any(line.split()[0]!='127.0.0.1' for line in result.stdout.splitlines()):
            raise RuntimeError('loopback name resolution verification failed')
    record['tcp_events'] = []
    stop = threading.Event()
    listeners = []
    def accept_loop(listener, port):
        with listener:
            while not stop.is_set():
                try:
                    connection, peer = listener.accept()
                except socket.timeout:
                    continue
                event = {'port':port, 'peer':list(peer), 'accept_monotonic_ns':time.monotonic_ns(), 'chunks':[]}
                record['tcp_events'].append(event)
                with connection:
                    connection.settimeout(0.2)
                    size=0
                    while not stop.is_set() and size < 65536:
                        try:
                            data=connection.recv(min(4096,65536-size))
                        except socket.timeout:
                            continue
                        except OSError as error:
                            event['socket_error']=str(error)
                            break
                        if not data:
                            event['peer_eof']=True
                            break
                        event['chunks'].append({'monotonic_ns':time.monotonic_ns(),'hex':data.hex()})
                        size+=len(data)
                    if size==65536:
                        event['byte_limit_reached']=True
    for port in sorted(set(args.listen)):
        listener=socket.socket()
        listener.setsockopt(socket.SOL_SOCKET,socket.SO_REUSEADDR,1)
        listener.bind(('127.0.0.1',port))
        listener.listen(4)
        listener.settimeout(0.2)
        thread=threading.Thread(target=accept_loop,args=(listener,port),daemon=True)
        thread.start()
        listeners.append(thread)
    children, streams = [], []
    game_log = args.game_dir / 'CALog.log'
    if game_log.exists():
        (args.out / 'CALog-before.txt').write_bytes(game_log.read_bytes())
        game_log.unlink()

    def spawn(command, name, cwd=None, pass_fds=(), input_file=None):
        stream = (args.out / (name + '.log')).open('w')
        streams.append(stream)
        process = subprocess.Popen(command, env=env, cwd=cwd, stdout=stream,
                                   stderr=subprocess.STDOUT, start_new_session=True,
                                   pass_fds=pass_fds, stdin=input_file)
        children.append(process)
        return process

    try:
        read_fd, write_fd = os.pipe()
        spawn(['Xvfb', '-displayfd', str(write_fd), '-screen', '0', '1024x768x24',
               '-nolisten', 'tcp'], 'xvfb', pass_fds=(write_fd,))
        os.close(write_fd)
        with selectors.DefaultSelector() as selector:
            selector.register(read_fd, selectors.EVENT_READ)
            if not selector.select(8):
                raise RuntimeError('Xvfb display timeout')
            number = os.read(read_fd, 32).decode().strip()
        os.close(read_fd)
        if not number.isdigit():
            raise RuntimeError('invalid Xvfb display')
        env['DISPLAY'] = ':' + number
        boot = spawn(['wineboot', '-u'], 'wineboot')
        record['wineboot_returncode'] = boot.wait(timeout=35)
        if record['wineboot_returncode']:
            raise RuntimeError('wineboot failed')
        if args.trace_api:
            env['WINEDEBUG'] += ',+relay'
        if args.debug:
            commands = args.out / 'debug-commands.txt'
            commands.write_text(args.commands.read_text() if args.commands else 'cont\nbt\ninfo regs\nx /24x $esp\nx /8x 0x226d000\nquit\n')
            with commands.open() as input_file:
                client = spawn(['wine', 'winedbg', str(executable)], 'client', cwd=args.game_dir, input_file=input_file)
        else:
            client = spawn(['wine', str(executable)], 'client', cwd=args.game_dir)
        record['launcher_pid'] = client.pid
        started = time.monotonic()
        while True:
            elapsed = time.monotonic() - started
            row = {'elapsed_s': round(elapsed, 3), 'launcher_returncode': client.poll()}
            windows = subprocess.run(['xwininfo', '-root', '-tree'], env=env, text=True,
                                     stdout=subprocess.PIPE, stderr=subprocess.STDOUT, timeout=3)
            (args.out / f'windows-{len(record["observations"]):02d}.txt').write_text(windows.stdout)
            processes = subprocess.run(['ps', '-eo', 'pid,ppid,etimes,comm,args'], text=True,
                                       stdout=subprocess.PIPE, timeout=3)
            (args.out / f'processes-{len(record["observations"]):02d}.txt').write_text(processes.stdout)
            subprocess.run(['import', '-window', 'root', str(args.out / f'screen-{len(record["observations"]):02d}.png')],
                           env=env, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, timeout=5)
            record['observations'].append(row)
            if elapsed >= args.seconds or client.poll() is not None:
                break
            time.sleep(min(3, max(0, args.seconds-elapsed)))
        record['launcher_returncode'] = client.poll()
    except Exception as error:
        record['error'] = str(error)
    finally:
        stop.set()
        for thread in listeners:
            thread.join(timeout=0.5)
        if game_log.exists():
            (args.out / 'CALog-run.txt').write_bytes(game_log.read_bytes())
        try:
            subprocess.run(['wineserver', '-k'], env=env, stdout=subprocess.DEVNULL,
                           stderr=subprocess.DEVNULL, timeout=4, check=False)
        except subprocess.TimeoutExpired:
            record['cleanup_wineserver_timeout'] = True
        for process in reversed(children):
            try:
                if process.poll() is None:
                    os.killpg(process.pid, signal.SIGTERM)
                process.wait(timeout=2)
            except ProcessLookupError:
                pass
            except subprocess.TimeoutExpired:
                try:
                    os.killpg(process.pid, signal.SIGKILL)
                except ProcessLookupError:
                    pass
                process.wait(timeout=2)
        for stream in streams:
            stream.close()
        record['end_monotonic_ns'] = time.monotonic_ns()
        (args.out / 'run.json').write_text(json.dumps(record, indent=2) + '\n')
        print(json.dumps(record), flush=True)
    return 1 if record.get('error') else 0


if __name__ == '__main__':
    raise SystemExit(main())
