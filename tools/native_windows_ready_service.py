"""Observe installed input identities and screen before requesting owned service.

Only local private candidate inputs are read. Live guest-media reads are
diagnostic; the owner must still export final receipts after VM shutdown.
"""
import argparse
import io
import json
from pathlib import Path
import re
import subprocess
import sys
import tarfile
import time

from PIL import Image

ROOT = Path(__file__).resolve().parents[1]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--run', type=Path, required=True)
    parser.add_argument('--input-summary', type=Path, required=True)
    args = parser.parse_args()
    local = (ROOT / 'local').resolve(strict=True)
    run = args.run.resolve(strict=True)
    summary = args.input_summary.resolve(strict=True)
    if any(path == local or not path.is_relative_to(local) for path in [run, summary]):
        parser.error('Run/input phải ở local/')
    if (run / 'readiness-controller.json').exists() or (run / 'start-service-requested').exists():
        parser.error('Readiness controller output must be new')
    expected = json.loads(summary.read_text())
    config = json.loads((run / 'private-owner-config.json').read_text())
    record = dict(started_epoch=time.time(), scope='Live installation readiness; no native acceptance inferred')
    try:
        deadline = time.monotonic() + 600
        while time.monotonic() < deadline:
            owner = json.loads((run / 'owner-status.json').read_text())
            if owner.get('ended_epoch') or owner.get('error'):
                raise RuntimeError('Owner ended before readiness')
            result = subprocess.run(['mtype', '-i', str(run / 'seed.img'), '::native-windows-install.json'],
                                    capture_output=True, text=True, timeout=5)
            if result.returncode == 0:
                guest = json.loads(result.stdout.lstrip('\ufeff'))
                if guest['phase'] == 'failed':
                    raise RuntimeError('Installer failed; inspect private guest report')
                if guest['phase'] == 'install_ready':
                    observed = guest['install']
                    if (observed['files_checked'] != expected['files']
                            or observed['exe_sha256'] != expected['exe_sha256']
                            or observed['launcher_sha256'] != expected['launcher_sha256']):
                        raise RuntimeError('Installed input identity mismatch')
                    (run / 'install-ready-diagnostic.json').write_text(json.dumps(guest, indent=2) + '\n')
                    break
            time.sleep(1)
        else:
            raise TimeoutError('Installation receipt deadline')
        for index in range(3):
            name = 'install-ready-capture-' + str(index)
            target = run / name
            actions = run / (name + '.json')
            actions.write_text(json.dumps([dict(op='capture', wait_seconds=1)]) + '\n')
            result = subprocess.run([sys.executable, str(ROOT / 'tools/native_windows_input.py'),
                                     '--run', str(run), '--actions', str(actions), '--out', str(target)],
                                    capture_output=True, text=True, timeout=45)
            (run / (name + '.stderr')).write_text(result.stderr)
            captured = target / 'capture-0.png'
            if result.returncode:
                copied = subprocess.run(['docker', 'cp', config['container'] + ':/task/' + name + '-0.png', '-'],
                                        capture_output=True, timeout=15)
                if copied.returncode:
                    continue
                with tarfile.open(fileobj=io.BytesIO(copied.stdout)) as archive:
                    members = archive.getmembers()
                    if len(members) != 1 or not members[0].isfile():
                        raise ValueError('Capture archive shape')
                    captured = run / (name + '-recovered.png')
                    captured.write_bytes(archive.extractfile(members[0]).read())
            image = Image.open(captured)
            if image.size != (1024, 768):
                raise ValueError('Unexpected readiness display size')
            crop = run / (name + '-ocr.png')
            image.crop((0, 30, 1000, 220)).resize((2000, 380)).save(crop)
            ocr = subprocess.run(['tesseract', str(crop), 'stdout', '--psm', '6'],
                                 capture_output=True, text=True, check=True, timeout=15)
            (run / (name + '-ocr.txt')).write_text(ocr.stdout)
            wanted = 'm8installreadyfiles' + str(expected['files'])
            if wanted in re.sub('[^a-z0-9]', '', ocr.stdout.lower()):
                record.update(readiness_screenshot=str(captured.relative_to(run)),
                              original_capture_exit=result.returncode)
                break
        else:
            raise RuntimeError('Readiness text not observed; service stays stopped')
        (run / 'start-service-requested').touch(exist_ok=False)
        record['service_requested_epoch'] = time.time()
        print('Installed identities and readiness screen verified; bounded service requested.', flush=True)
    except Exception as error:
        record['error'] = str(error)
        (run / 'stop-requested').touch(exist_ok=True)
        raise
    finally:
        record['ended_epoch'] = time.time()
        (run / 'readiness-controller.json').write_text(json.dumps(record, indent=2) + '\n')


if __name__ == '__main__':
    main()
