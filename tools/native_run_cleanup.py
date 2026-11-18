"""Remove explicitly owned disposable M8 files after a run, preserving evidence.

Authorization: docs/evidence/owner-directives/run-cleanup.json. This is not an
old-run retention sweep. Paths and receipts remain private under local/.
"""
import hashlib
import json
import os
from pathlib import Path
import re
import stat
import time

import local_retention as retention


def storage(path=Path('/home')):
    value = os.statvfs(path)
    return dict(observed_epoch=time.time(), available_bytes=value.f_bavail * value.f_frsize)


def require_storage():
    available = storage()['available_bytes']
    if available < 80_000_000_000:
        raise RuntimeError('Below 80 GB free on /home; no new native runtime allowed')
    if available < 150_000_000_000:
        raise RuntimeError('Below 150 GB free on /home; clean own old runs before preparing a new run')


def container_absent(returncode, stderr):
    """Recognize only Docker's explicit missing-object result, never a daemon failure."""
    if isinstance(stderr, bytes):
        stderr = stderr.decode('utf-8', errors='replace')
    return returncode != 0 and 'no such object:' in stderr.casefold()


def write_json(path, value):
    temporary = path.with_name(path.name + '.tmp')
    with temporary.open('x', encoding='utf-8') as stream:
        json.dump(value, stream, ensure_ascii=False, indent=2)
        stream.write('\n')
        stream.flush()
        os.fsync(stream.fileno())
    temporary.replace(path)


def checksum(path):
    digest = hashlib.sha256()
    with path.open('rb') as stream:
        for block in iter(lambda: stream.read(1 << 20), b''):
            digest.update(block)
    return digest.hexdigest()


def reference_guards(root):
    """Read current checkpoint/audit, docs/tool, fixture and pruning guards.

    Only this fresh run's explicit files can be deleted. Reuse the retention
    reference reader; never change immutable run manifests or pruning records.
    """
    warnings = []
    refs, _ = retention.checkpoint_references(root, warnings.append)
    if warnings:
        raise ValueError('Incomplete checkpoint reference inventory; preserve run artifacts')
    code, docs = retention.tracked_references(root)
    protected = set(refs) | code | set(docs)
    registry = root / retention.lean.PRUNED
    directories = set(retention.ALWAYS_KEEP)
    if registry.exists():
        directories.update(retention.lean.load_pruned(root).entries)
    for path in code:
        clean = path.rstrip('/')
        if ((root / clean).is_dir()
                and len(clean.split('/')) >= retention.unit_depth(clean + '/x')):
            directories.add(clean)
    return protected, directories


def cleanup(root, run, *, stopped, exports_complete, inputs=(), receipt_suffix=''):
    """Delete only this run's known VM scratch files, after exports and stop.

    Dumps stay until a separate extraction receipt exists; this function never
    treats WER presence or VM shutdown as successful extraction or acceptance.
    A failed stop/export preserves everything for recovery.
    """
    root = Path(root).resolve(strict=True)
    local = (root / 'local').resolve(strict=True)
    run = Path(run).resolve(strict=True)
    if not run.is_relative_to(local / 'research/codex') or not run.name.startswith('m8-'):
        raise ValueError('Cleanup is restricted to one owned M8 run')
    if receipt_suffix and not re.fullmatch('-recovery-r[1-9][0-9]*', receipt_suffix):
        raise ValueError('Only a numbered recovery receipt suffix is allowed')
    receipt_path = run / ('run-cleanup-result' + receipt_suffix + '.json')
    manifest_path = run / ('run-cleanup-manifest' + receipt_suffix + '.json')
    if receipt_path.exists() or manifest_path.exists():
        raise ValueError('Cleanup receipt already exists; never overwrite prior observations')
    receipt = dict(schema='kre-native-run-cleanup-v1',
                   authorization='docs/evidence/owner-directives/run-cleanup.json',
                   before=storage(), stopped=stopped, exports_complete=exports_complete,
                   deleted=[], preserved=[], limits='Storage cleanup only; no native success inferred')
    candidates = [run / name for name in ('native-system.qcow2', 'seed.img')]
    receipt['dumps_pending_extraction'] = [str(p.relative_to(run)) for p in run.rglob('*.dmp')]
    rows = []
    if stopped and exports_complete:
        try:
            protected, directories = reference_guards(root)
            # Config fixture references are checked only in this run: these
            # new scratch paths cannot be an older run's declared input.
            configs = ['local/' + p.relative_to(local).as_posix()
                       for p in run.rglob('*.json') if p.stat().st_size <= retention.CONFIG_LIMIT]
            _, fixture_pins = retention.config_pins(root, configs)
            protected.update(fixture_pins)
            input_paths = {Path(p).resolve(strict=True) for p in inputs}
            for path in candidates:
                if not os.path.lexists(path):
                    continue
                relative = 'local/' + path.relative_to(local).as_posix()
                info = path.lstat()
                if (not stat.S_ISREG(info.st_mode) or path.resolve() in input_paths
                        or info.st_uid != os.getuid() or info.st_nlink != 1
                        or relative in protected or retention.covered(relative, directories)):
                    receipt['preserved'].append(dict(path=path.name, reason='input/reference/ownership guard'))
                    continue
                rows.append(dict(path=path.name, bytes=info.st_size, sha256=checksum(path),
                                 inode=info.st_ino, mtime_ns=info.st_mtime_ns))
        except Exception as error:
            receipt['guard_error'] = type(error).__name__ + ': ' + str(error)
            rows = []
    else:
        receipt['preserved'].append(dict(path='VM scratch', reason='stop or evidence export not confirmed'))
    manifest = dict(schema='kre-native-run-cleanup-manifest-v1', created_epoch=time.time(),
                    run=run.name, files=rows)
    write_json(manifest_path, manifest)
    receipt['manifest_sha256'] = checksum(manifest_path)
    for row in rows:
        path = run / row['path']
        try:
            current = path.lstat()
            if (not stat.S_ISREG(current.st_mode)
                    or (current.st_ino, current.st_size, current.st_mtime_ns)
                    != (row['inode'], row['bytes'], row['mtime_ns'])):
                receipt['preserved'].append(dict(path=row['path'], reason='changed after manifest'))
                continue
            path.unlink()
            receipt['deleted'].append(row['path'])
        except OSError as error:
            receipt['preserved'].append(dict(path=row['path'], reason=type(error).__name__))
    receipt['deleted_apparent_bytes'] = sum(r['bytes'] for r in rows if r['path'] in receipt['deleted'])
    receipt['after'] = storage()
    write_json(receipt_path, receipt)
    return receipt
