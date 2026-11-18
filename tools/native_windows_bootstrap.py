"""Start the declared seed script only after observing Run and PowerShell UAC.

All input, captures and OCR remain local. This establishes ordinary UI input,
not installation, startup or gameplay acceptance.
"""
import argparse
import json
from pathlib import Path
import re
import subprocess
import sys
import time

from PIL import Image

ROOT = Path(__file__).resolve().parents[1]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--run', type=Path, required=True)
    parser.add_argument('--out', type=Path, required=True)
    choice = parser.add_mutually_exclusive_group()
    choice.add_argument('--resume', action='store_true')
    choice.add_argument('--case')
    parser.add_argument('--diagnostic-launcher', action='store_true')
    parser.add_argument('--socket-probe', action='store_true')
    parser.add_argument('--network-control', action='store_true')
    parser.add_argument('--keep-console', action='store_true', help='Retain PowerShell output for script failure diagnosis')
    parser.add_argument('--windows-credentials', type=Path,
                        help='Optional private local VM login/password; never project account input')
    args = parser.parse_args()
    local = (ROOT / 'local').resolve(strict=True)
    run = args.run.resolve(strict=True)
    out = args.out.resolve()
    if any(path == local or not path.is_relative_to(local) for path in [run, out]):
        parser.error('Run/output phải ở local/')
    if out.exists() or (args.case and not re.fullmatch('[a-z0-9-]{1,32}', args.case)):
        parser.error('Output phải mới; case phải là tên ASCII hữu hạn')
    windows = None
    if args.windows_credentials:
        private = args.windows_credentials.resolve(strict=True)
        if private == local or not private.is_relative_to(local):
            parser.error('Windows credentials phải ở local/')
        windows = json.loads(private.read_text())
        if (not isinstance(windows.get('login'), str) or not windows['login']
                or not isinstance(windows.get('password'), str) or not windows['password']):
            parser.error('Private VM login/password missing')
    out.mkdir(mode=0o700)
    record = dict(scope='Observed Run/UAC input only; native acceptance requires independent receipts',
                  started_epoch=time.time(), steps=[])

    def step(name, actions):
        path = out / (name + '.json')
        path.write_text(json.dumps(actions) + '\n')
        target = out / name
        command = [sys.executable, str(ROOT / 'tools/native_windows_input.py'),
                   '--run', str(run), '--actions', str(path), '--out', str(target)]
        result = subprocess.run(command, capture_output=True, text=True, timeout=55)
        (out / (name + '.stderr')).write_text(result.stderr)
        record['steps'].append(dict(name=name, exit=result.returncode))
        if result.returncode:
            # A screenshot can exist even if QMP's response times out. Preserve
            # the failed producer and request a fresh observation only; never
            # replay its keyboard/click actions or call the original exit 0.
            if 'TimeoutError' in result.stderr and "request('screendump'" in result.stderr:
                fresh = out / (name + '-fresh.json')
                fresh.write_text(json.dumps([dict(op='capture', wait_seconds=1)]) + '\n')
                fresh_out = out / (name + '-fresh')
                recovery = subprocess.run([sys.executable, str(ROOT / 'tools/native_windows_input.py'),
                                           '--run', str(run), '--actions', str(fresh), '--out', str(fresh_out)],
                                          capture_output=True, text=True, timeout=55)
                (out / (name + '-fresh.stderr')).write_text(recovery.stderr)
                record['steps'][-1]['fresh_capture_exit'] = recovery.returncode
                if recovery.returncode == 0:
                    return fresh_out / 'capture-0.png'
            raise RuntimeError('Declared input failed: ' + name)
        return target / ('capture-' + str(len(actions) - 1) + '.png')

    def observed(path, crop, label, mode=6):
        image = Image.open(path)
        if image.size != (1024, 768):
            raise ValueError('Unexpected display size; coordinates not applicable')
        image = image.crop(crop)
        image = image.resize((image.width * 4, image.height * 4))
        target = out / (label + '-ocr.png')
        image.save(target)
        result = subprocess.run(['tesseract', str(target), 'stdout', '--psm', str(mode)],
                                capture_output=True, text=True, timeout=15, check=True)
        (out / (label + '-ocr.txt')).write_text(result.stdout)
        return result.stdout.lower()

    try:
        deadline = time.monotonic() + 45
        while True:
            owner_path = run / 'owner-status.json'
            if owner_path.exists():
                owner = json.loads(owner_path.read_text())
                if owner.get('ended_epoch') or owner.get('error'):
                    raise RuntimeError('Candidate owner ended before bootstrap')
                if owner.get('vm_preflight_epoch') or owner.get('vm_started_epoch'):
                    break
            if time.monotonic() > deadline:
                raise TimeoutError('Candidate VM preflight not observed')
            time.sleep(.5)
        deadline = time.monotonic() + 180
        index = 0
        signin_attempts = 0
        while True:
            if windows:
                image = step('signin-observe-' + str(index), [dict(op='key', key='esc', wait_seconds=1),
                                                              dict(op='capture')])
                text = observed(image, (400, 350, 630, 430), 'signin-' + str(index), mode=7)
                desktop = observed(image, (0, 0, 180, 210), 'desktop-' + str(index), mode=11)
                if windows['login'].lower() in text:
                    if signin_attempts >= 2:
                        raise RuntimeError('Dedicated VM sign-in did not reach desktop')
                    # The observed dedicated account sign-in field. Input records
                    # stay in this private local run, including the VM password.
                    step('signin-' + str(index), [dict(op='click', xy=[468, 468]),
                                                 dict(op='key', key='ctrl-a'),
                                                 dict(op='type', text=windows['password']),
                                                 dict(op='key', key='ret', wait_seconds=3), dict(op='capture')])
                    record['vm_signin_attempted'] = True
                    signin_attempts += 1
                    index += 1
                    continue
                if 'recyclebin' not in re.sub(r'\s+', '', desktop):
                    if time.monotonic() > deadline:
                        raise TimeoutError('Dedicated Windows desktop not observed')
                    step('boot-wait-' + str(index), [dict(op='capture', wait_seconds=2)])
                    index += 1
                    continue
            image = step('open-run-' + str(index), [dict(op='click', xy=[800, 300]),
                                                   dict(op='key', key='meta_l-r', wait_seconds=2),
                                                   dict(op='capture')])
            text = observed(image, (16, 511, 415, 719), 'run-' + str(index))
            if 'open' in text and 'windows will open' in text and 'type the name' in text:
                break
            if time.monotonic() > deadline:
                raise TimeoutError('Run dialog not observed')
            index += 1
        # Keep the complete command visible in the measured Run field. These
        # unambiguous PowerShell parameter prefixes avoid horizontal clipping.
        command = 'powershell -ep bypass -f d:\\run.ps1'
        if args.keep_console:
            command = 'powershell -noe -ep bypass -f d:\\run.ps1'
        if args.resume:
            command += ' -r'
        if args.case:
            command += ' -case ' + args.case
        if args.diagnostic_launcher:
            command += ' -d'
        if args.socket_probe:
            command += ' -s'
        if args.network_control:
            command += ' -n'
        image = step('clear-command', [dict(op='click', xy=[229, 622], wait_seconds=1),
                                        dict(op='key', key='home', hold_ms=200, wait_seconds=1),
                                        dict(op='key', key='shift-end', hold_ms=200, wait_seconds=1),
                                        dict(op='key', key='backspace', hold_ms=200, wait_seconds=1),
                                        dict(op='click', xy=[200, 526], wait_seconds=1), dict(op='capture')])
        # OCR mistakes a blinking caret or I-beam pointer for a letter. Move the
        # pointer to the title and inspect the inner white text area, excluding
        # the initial caret and combo-box border/arrow.
        blank = Image.open(image).crop((90, 616, 380, 629)).convert('L')
        blank.save(out / 'cleared-command-pixels.png')
        if any(pixel < 128 for pixel in blank.tobytes()):
            raise RuntimeError('Run input was not observed empty; no typing attempted')
        image = step('command', [dict(op='type', text=command, wait_seconds=2), dict(op='capture')])
        text = observed(image, (80, 612, 388, 635), 'command', mode=7)
        if re.sub('[^a-z0-9]', '', text) != re.sub('[^a-z0-9]', '', command):
            raise RuntimeError('Declared command not observed in Run input')
        image = step('elevate', [dict(op='key', key='ctrl-shift-ret', hold_ms=200, wait_seconds=3),
                                dict(op='capture')])
        deadline = time.monotonic() + 30
        index = 0
        while True:
            text = observed(image, (284, 227, 740, 617), 'uac-' + str(index))
            if all(value in text for value in ['user account control', 'windows powershell']):
                yes = observed(image, (309, 484, 510, 516), 'yes-' + str(index), mode=7)
                if 'yes' in yes:
                    confirm_xy = [410, 500]
                    break
                yes = observed(image, (309, 560, 510, 592), 'yes-expanded-' + str(index), mode=7)
                if 'yes' in yes:
                    confirm_xy = [410, 576]
                    break
            if time.monotonic() > deadline:
                raise TimeoutError('Expected PowerShell UAC not observed')
            image = step('uac-wait-' + str(index), [dict(op='capture', wait_seconds=2)])
            index += 1
        step('confirm', [dict(op='click', xy=confirm_xy, wait_seconds=3), dict(op='capture')])
        record['ui_sequence_completed'] = True
    except Exception as error:
        record['error'] = str(error)
        raise
    finally:
        record['ended_epoch'] = time.time()
        (out / 'bootstrap.json').write_text(json.dumps(record, indent=2) + '\n')


if __name__ == '__main__':
    main()
