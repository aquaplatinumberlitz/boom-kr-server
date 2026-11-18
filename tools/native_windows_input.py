"""Execute declared ordinary QMP input against one owned Windows candidate.

Actions and captures stay local. No hidden application state writes or native
success inference; compare screenshots and independent wire/account evidence.
"""
import argparse
import io
import json
import os
from pathlib import Path
import socket
import subprocess
import tarfile
import time

ROOT = Path(__file__).resolve().parents[1]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--run', type=Path, required=True)
    parser.add_argument('--actions', type=Path, required=True)
    parser.add_argument('--out', type=Path, required=True)
    args = parser.parse_args()
    local = (ROOT / 'local').resolve()
    run = args.run.resolve(strict=True)
    output = args.out.resolve()
    action_path = args.actions.resolve(strict=True)
    if any(path == local or not path.is_relative_to(local) for path in [run, output, action_path]):
        parser.error('Run/actions/output phải ở local/')
    if output.exists():
        parser.error('Output phải mới')
    config = json.loads((run / 'private-owner-config.json').read_text())
    owner = json.loads((run / 'owner-status.json').read_text())
    if owner.get('ended_epoch'):
        parser.error('Candidate đã kết thúc')
    name = config['container']
    label = subprocess.run(['docker', 'inspect', '--format', '{{index .Config.Labels "boom.task"}}', name],
                           capture_output=True, text=True, check=True, timeout=10).stdout.strip()
    if label != 'm8-native-windows':
        parser.error('Container không thuộc native candidate')
    actions = json.loads(action_path.read_text())
    if not isinstance(actions, list) or len(actions) > 100:
        parser.error('Action list bound')
    output.mkdir(mode=0o700)
    events = []
    previous = Path.cwd()
    try:
        os.chdir(run)
        with socket.socket(socket.AF_UNIX, socket.SOCK_STREAM) as client:
            client.settimeout(30)
            client.connect('qmp.sock')
            stream = client.makefile('rwb', buffering=0)
            json.loads(stream.readline())

            def request(command, arguments=None):
                stream.write((json.dumps(dict(execute=command, arguments=arguments or {})) + '\n').encode())
                while True:
                    value = json.loads(stream.readline())
                    if 'return' in value or 'error' in value:
                        events.append(dict(command=command, arguments=arguments, response=value, monotonic_ns=time.monotonic_ns()))
                        if 'error' in value:
                            raise RuntimeError('QMP request failed; retained response')
                        return value['return']

            def key(value, hold=60):
                if any(char not in 'abcdefghijklmnopqrstuvwxyz0123456789_-' for char in value):
                    raise ValueError('Key name bound')
                if not 1 <= hold <= 500:
                    raise ValueError('Key hold bound')
                keys = value.split('-')
                down = [dict(type='key', data=dict(down=True, key=dict(type='qcode', data=code))) for code in keys]
                up = [dict(type='key', data=dict(down=False, key=dict(type='qcode', data=code))) for code in reversed(keys)]
                try:
                    request('input-send-event', dict(events=down))
                    time.sleep(hold / 1000)
                finally:
                    request('input-send-event', dict(events=up))
                time.sleep(.12)

            request('qmp_capabilities')
            symbols = {' ': 'spc', '\\': 'backslash', '/': 'slash', '.': 'dot', '-': 'minus',
                       '_': 'shift-minus', ':': 'shift-semicolon', '"': 'shift-apostrophe',
                       "'": 'apostrophe', '(': 'shift-9', ')': 'shift-0', '&': 'shift-7',
                       '+': 'shift-equal', '=': 'equal', '$': 'shift-4', ';': 'semicolon',
                       '[': 'bracket_left', ']': 'bracket_right', '|': 'shift-backslash', ',': 'comma'}
            for index, action in enumerate(actions):
                operation = action['op']
                if operation == 'key':
                    key(action['key'], action.get('hold_ms', 60))
                elif operation == 'type':
                    text = action['text']
                    if len(text) > 512 or any(not char.isascii() or not (char.isalnum() or char in symbols) for char in text):
                        raise ValueError('Declared text bound')
                    for char in text:
                        key(symbols.get(char, 'shift-' + char.lower() if char.isupper() else char))
                elif operation == 'click':
                    width, height = action.get('size', [1024, 768])
                    x, y = action['xy']
                    if not (0 <= x < width <= 8192 and 0 <= y < height <= 8192):
                        raise ValueError('Click coordinate bound')
                    settle_ms = action.get('move_settle_ms', 150)
                    if type(settle_ms) is not int or not 0 <= settle_ms <= 1000:
                        raise ValueError('Mouse movement settle bound')
                    # Let the guest/game consume the new pointer position
                    # before delivering a button edge. A single QMP batch can
                    # otherwise let frame-polled UI sample the prior position.
                    request('input-send-event', dict(events=[
                        dict(type='abs', data=dict(axis='x', value=round(x * 32767 / (width - 1)))),
                        dict(type='abs', data=dict(axis='y', value=round(y * 32767 / (height - 1))))]))
                    time.sleep(settle_ms / 1000)
                    try:
                        request('input-send-event', dict(events=[dict(type='btn', data=dict(button='left', down=True))]))
                        time.sleep(.12)
                    finally:
                        request('input-send-event', dict(events=[dict(type='btn', data=dict(button='left', down=False))]))
                elif operation == 'capture':
                    basename = output.name + '-' + str(index) + '.png'
                    request('screendump', dict(filename='/task/' + basename, format='png'))
                    captured = subprocess.run(['docker', 'cp', name + ':/task/' + basename, '-'],
                                              capture_output=True, check=True, timeout=30)
                    with tarfile.open(fileobj=io.BytesIO(captured.stdout)) as archive:
                        members = archive.getmembers()
                        if len(members) != 1 or not members[0].isfile():
                            raise ValueError('Capture archive shape')
                        (output / ('capture-' + str(index) + '.png')).write_bytes(archive.extractfile(members[0]).read())
                else:
                    raise ValueError('Unknown declared action')
                delay = action.get('wait_seconds', 0)
                if not 0 <= delay <= 10:
                    raise ValueError('Action wait bound')
                if delay:
                    time.sleep(delay)
    finally:
        os.chdir(previous)
        (output / 'input-record.json').write_text(json.dumps(dict(actions=actions, events=events,
            limits='Declared ordinary keyboard/tablet input and screenshots; application effect/native success must be verified separately.'), indent=2) + '\n')


if __name__ == '__main__':
    main()
