#!/usr/bin/env python3
"""Plan (dry run, default) or apply cleanup of host-local raw data under the retention directive.

Classes (docs/evidence/owner-directives/local-retention.json):
  keep_pinned        files the service/TCP suites/tools read (configs `kr-rust-*`, their fixture pins,
                     exact local paths named in rust/ server/ tools/ .github/), run manifests, local tools
  keep               checkpoint-referenced knowledge: small text/JSON/scripts (<= 1 MiB)
  keep_compressed    other checkpoint-referenced raw (captures, logs, traces, screenshots) and IDA
                     databases (annotations are knowledge); kept in place,
                     the btrfs mount already compresses transparently (zstd), so paths/hashes never change
  disposable_copy    per-run game directory / Wine prefix copies (recreatable from template + recipe)
  disposable_dump    memory/core dumps (derived facts are kept elsewhere)
  disposable_intermediate     unreferenced file inside a checkpoint-referenced run
  disposable_unreferenced_run run/file with no checkpoint, docs or code reference (incl. aborted runs)
  keep_recent        every file of a unit (run directory) holding any file whose mtime or ctime is within the
                     last N days (`--keep-recent-days`, default 7, 0 disables): in-progress work that no
                     checkpoint references yet; never deleted or compressed. Pinned files stay keep_pinned.
                     The plan records the guard and `--apply` re-checks it per unit at apply time.

Dry run writes local/retention/plan-<UTC>.json (actions + per-unit report) and a short Markdown summary.
`--apply` needs `--plan FILE --plan-sha256 HEX` of an owner-approved plan; it deletes only listed paths,
re-checks each one, and appends checkpoint-referenced deletions to docs/evidence/checkpoints/local-pruned.json
so the gate reports them as PRUNED (hashes stay in records/run manifests). It never touches pinned files.
"""
import argparse
import collections
import datetime
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import sys

if str(Path(__file__).resolve().parent) not in sys.path:
    sys.path.insert(0, str(Path(__file__).resolve().parent))
import checkpoint_evidence as lean
import checkpoint_store as store

SCHEMA = 'kre-local-retention-plan-v1'
UNIT_PARENTS = ('local/research/codex', 'local/research/l10n-utf8-codex', 'local/research', 'local/runs', 'local')
ALWAYS_KEEP = ('local/evidence-manifests', 'local/tools', 'local/retention', 'local/cache')
COPY_PART = re.compile(r'(?:game|prefix|wineprefix|client-copy|game-copy)(?:[-_][A-Za-z0-9._-]*)?|drive_c|dosdevices')
DUMP_NAME = re.compile(r'(?:\.(?:dmp|mdmp|core|mem|memdump)$|^core(?:\.[0-9]+)?$|memdump|minidump)', re.I)
IMAGE = {'.png', '.bmp', '.jpg', '.jpeg', '.gif', '.ppm', '.xwd', '.webp'}
TEXT = {'.json', '.jsonl', '.md', '.txt', '.py', '.c', '.h', '.rs', '.toml', '.yml', '.yaml', '.csv', '.gdb',
        '.sh', '.ps1', '.ini', '.cfg', '.xml', '.sql', '.patch', '.diff'}
LOCAL_REF = re.compile(r'local/[A-Za-z0-9_.@+-][A-Za-z0-9_./@+-]*')
BRACE_REF = re.compile(r'(local/[A-Za-z0-9_./@+-]*)\{([A-Za-z0-9_.,@+-]+)\}([A-Za-z0-9_./@+-]*)')
IDA_DB = {'.i64', '.idb', '.id0', '.id1', '.id2', '.nam', '.til'}
SMALL = 1 << 20
DEFAULT_KEEP_RECENT_DAYS = 7
DAY = 86400.0
CONFIG_LIMIT = 65536


def unit_depth(relative):
    if relative.startswith(('local/research/codex/', 'local/research/l10n-utf8-codex/')):
        return 4
    if relative.startswith(('local/research/', 'local/runs/')):
        return 3
    return 2


def unit_of(relative):
    """Retention unit: run directory (or top-level file) such as local/research/codex/<run>."""
    return '/'.join(relative.split('/')[:unit_depth(relative)])


def normalize(root, value):
    """Repository-relative local/ path for relative, worktree-absolute or main-checkout-absolute pins."""
    if not isinstance(value, str) or not value:
        return None
    if value.startswith('local/'):
        return value.rstrip('/')
    path = Path(value)
    if path.is_absolute():
        marker = '/local/'
        if marker in value:
            return 'local/' + value.split(marker, 1)[1].rstrip('/')
    return None


def checkpoint_references(root, warn):
    """path -> set(CP) for every host-local evidence row (repository rows and run manifests)."""
    refs = collections.defaultdict(set)
    hashes = {}
    index = store.load(root)
    records = store.records(index, root)
    for position in range(len(records)):
        record = records[position]
        rows = list(store.evidence(record, root))
        for pointer in lean.pointers(record):
            try:
                rows.extend(lean.read_run(pointer, root))
            except (FileNotFoundError, ValueError) as error:
                warn(f'{record["id"]}: run manifest {pointer["path"]}: {error}')
        for row in rows:
            path = normalize(root, row.get('path')) if isinstance(row, dict) else None
            if path:
                refs[path].add(record['id'])
                hashes[path] = row.get('sha256')
    # Audits link host-local files in read_files; the gate requires them, so they are references too.
    for audit_path in sorted((root / 'docs/evidence/checkpoints/audits').glob('AUD*.json')):
        try:
            audit = json.loads(audit_path.read_text(encoding='utf-8'))
        except (OSError, ValueError) as error:
            warn(f'{audit_path.name}: {error}')
            continue
        for name in audit.get('read_files', []) if isinstance(audit, dict) else []:
            path = normalize(root, name) if isinstance(name, str) else None
            if path:
                refs[path].add(audit_path.stem)
    return refs, hashes


def tracked_references(root):
    """Exact local paths named by tracked files: code/tools are pins, docs are references."""
    files = subprocess.run(['git', '-C', str(root), 'ls-files', '-z'], capture_output=True, check=True).stdout
    code, docs = set(), collections.defaultdict(set)
    for name in files.decode().split('\0'):
        if not name or name.startswith('docs/evidence/checkpoints/'):
            continue
        path = root / name
        try:
            if path.stat().st_size > 8 << 20:
                continue
            text = path.read_text(errors='ignore')
        except OSError:
            continue
        found = [head + item + tail for head, items, tail in BRACE_REF.findall(text) for item in items.split(',')]
        braced = {head for head, _, _ in BRACE_REF.findall(text)}
        found += [m for m in LOCAL_REF.findall(text) if m not in braced]
        for match in found:
            match = match.rstrip('.,;:)')
            if name.startswith(('rust/', 'server/', 'tools/', '.github/')):
                code.add(match)
            else:
                docs[match].add(name)
    return code, docs


def config_pins(root, json_files):
    """Small `kr-rust-*` JSON configs/profiles and every nested {path, bytes, sha256} they pin."""
    configs, pins = set(), set()

    def walk(value):
        if isinstance(value, dict):
            if {'path', 'sha256'} <= set(value) and isinstance(value.get('path'), str):
                target = normalize(root, value['path'])
                if target:
                    pins.add(target)
            for item in value.values():
                walk(item)
        elif isinstance(value, list):
            for item in value:
                walk(item)
    for relative in json_files:
        try:
            raw = (root / relative).read_bytes()
        except OSError:
            continue
        if b'"kr-rust-' not in raw:
            continue
        try:
            data = json.loads(raw)
        except ValueError:
            continue
        if isinstance(data, dict) and str(data.get('schema', '')).startswith('kr-rust-'):
            configs.add(relative)
            walk(data)
    return configs, pins


def walk_local(root):
    """Yield (relative path, size, is_symlink, latest activity = max(mtime, ctime)) for every non-directory entry."""
    base = root / 'local'
    for directory, dirs, files in os.walk(base, followlinks=False):
        dirs.sort()
        rel_dir = Path(directory).relative_to(root).as_posix()
        for name in sorted(files) + [d for d in dirs if os.path.islink(os.path.join(directory, d))]:
            path = os.path.join(directory, name)
            try:
                info = os.lstat(path)
            except OSError:
                continue
            yield f'{rel_dir}/{name}', info.st_size, os.path.islink(path), max(info.st_mtime, info.st_ctime)


def unit_activity(root, unit):
    """Latest mtime/ctime (epoch seconds) of any entry under a unit, or None when the unit is absent."""
    base = root / unit
    if not os.path.lexists(base):
        return None
    paths = [str(base)]
    if base.is_dir() and not base.is_symlink():
        paths = [os.path.join(directory, name)
                 for directory, dirs, files in os.walk(base, followlinks=False)
                 for name in list(files) + [d for d in dirs if os.path.islink(os.path.join(directory, d))]]
    latest = None
    for path in paths:
        try:
            info = os.lstat(path)
        except OSError:
            continue
        latest = max(latest or 0.0, info.st_mtime, info.st_ctime)
    return latest


def is_recent(latest, now, days):
    return bool(days) and days > 0 and latest is not None and now - latest < days * DAY


def sha256_file(path):
    digest = hashlib.sha256()
    with open(path, 'rb') as stream:
        for block in iter(lambda: stream.read(1 << 20), b''):
            digest.update(block)
    return digest.hexdigest()


def path_closure(paths):
    """Set of every path in `paths` plus all its ancestor directories (down to two components)."""
    closure = set()
    for path in paths:
        parts = path.split('/')
        closure.update('/'.join(parts[:end]) for end in range(1, len(parts) + 1))
    return closure


def covered(path, prefixes):
    parts = path.split('/')
    return any('/'.join(parts[:end]) in prefixes for end in range(len(parts), 1, -1))


def classify(relative, size, refs, pinned, pinned_dirs, run_referenced):
    parts = relative.split('/')
    name = parts[-1]
    suffix = Path(name).suffix.lower()
    if relative in pinned or covered(relative, pinned_dirs):
        return 'keep_pinned'
    if any(COPY_PART.fullmatch(part) for part in parts[2:-1]):
        return 'disposable_copy'
    if DUMP_NAME.search(name):
        return 'disposable_dump'
    if relative in refs:
        return 'keep' if suffix in TEXT and size <= SMALL else 'keep_compressed'
    if suffix in IDA_DB:
        return 'keep_compressed'  # annotated IDA databases are RE knowledge, not recreatable
    if not run_referenced:
        return 'disposable_unreferenced_run'
    if suffix in ('.py', '.c', '.h', '.rs', '.sh', '.ps1', '.gdb'):
        return 'keep'  # small tooling that produced referenced data
    return 'disposable_intermediate'


def collapse(file_class, sizes):
    """file -> action target: the shallowest directory inside its unit whose files all share the
    file's disposable class, else the file itself."""
    totals = collections.Counter()
    per_class = collections.Counter()
    for path, klass in file_class.items():
        parts = path.split('/')
        for end in range(2, len(parts)):
            prefix = '/'.join(parts[:end])
            totals[prefix] += 1
            per_class[(klass, prefix)] += 1
    targets = {}
    for path, klass in file_class.items():
        if not klass.startswith('disposable'):
            continue
        parts = path.split('/')
        floor = unit_of(path).count('/') + 1
        target = path
        for end in range(floor, len(parts)):
            prefix = '/'.join(parts[:end])
            if per_class[(klass, prefix)] == totals[prefix]:
                target = prefix
                break
        targets[path] = target
    return targets


def btrfs_exclusive(root, targets):
    """Exclusive bytes per target via `btrfs filesystem du -s --raw` (lower bound of reclaim)."""
    result = {}
    targets = list(targets)
    for start in range(0, len(targets), 200):
        batch = targets[start:start + 200]
        out = subprocess.run(['btrfs', 'filesystem', 'du', '-s', '--raw', *batch], cwd=root,
                             capture_output=True, text=True)
        for line in out.stdout.splitlines()[1:]:
            fields = line.split(None, 3)
            if len(fields) == 4 and fields[1].isdigit():
                result[fields[3]] = int(fields[1])
    return result


def plan(root, use_btrfs, hash_images, out_dir, keep_recent_days=DEFAULT_KEEP_RECENT_DAYS, now=None):
    now_ts = datetime.datetime.now(datetime.timezone.utc).timestamp() if now is None else now
    notes = []
    refs, hashes = checkpoint_references(root, notes.append)
    code_refs, doc_refs = tracked_references(root)
    entries, json_files = [], []
    activity = {}
    for relative, size, link, touched in walk_local(root):
        entries.append((relative, size, link))
        unit = unit_of(relative)
        activity[unit] = max(activity.get(unit, 0.0), touched)
        if relative.endswith('.json') and size <= CONFIG_LIMIT and not link:
            json_files.append(relative)
    configs, fixture_pins = config_pins(root, json_files)
    existing = {relative for relative, _, _ in entries}
    pinned = (configs | fixture_pins | {p for p in code_refs if p in existing})
    # A directory named by code/tools (e.g. a --baseline run) is pinned only at run depth or below;
    # bare parents such as local/research/codex are mentioned as output roots, not inputs.
    pinned_dirs = set(ALWAYS_KEEP) | {p.rstrip('/') for p in code_refs
                                      if p.rstrip('/') not in existing and (root / p).is_dir()
                                      and p.rstrip('/').count('/') + 1 >= unit_depth(p.rstrip('/') + '/x')}
    referenced_units = collections.defaultdict(set)
    for path, cps in refs.items():
        referenced_units[unit_of(path)].update(cps)
    doc_units = collections.defaultdict(set)
    for path, names in doc_refs.items():
        doc_units[unit_of(path)].update(names)
    pinned_units = {unit_of(p) for p in pinned}
    recent_units = {u for u, touched in activity.items() if is_recent(touched, now_ts, keep_recent_days)}
    file_class, units = {}, collections.defaultdict(lambda: collections.Counter())
    unit_bytes = collections.defaultdict(lambda: collections.Counter())
    images = collections.defaultdict(list)
    for relative, size, link in entries:
        unit = unit_of(relative)
        run_ref = unit in referenced_units or unit in doc_units or unit in pinned_units
        klass = 'keep' if link else classify(relative, size, refs, pinned, pinned_dirs, run_ref)
        if unit in recent_units and klass != 'keep_pinned':
            klass = 'keep_recent'
        file_class[relative] = klass
        if Path(relative).suffix.lower() in IMAGE and klass == 'keep_compressed':
            images[(unit, size)].append(relative)
    # Byte-identical kept images are not deleted (they carry checkpoint pins); they are reported as
    # reflink-dedupe candidates (e.g. duperemove), which frees space without changing paths or hashes.
    dedupe = collections.Counter()
    if hash_images:
        for (unit, size), paths in images.items():
            if len(paths) < 2:
                continue
            seen = set()
            for relative in sorted(paths):
                digest = hashes.get(relative) or sha256_file(root / relative)
                if digest in seen:
                    dedupe['files'] += 1
                    dedupe['apparent_bytes'] += size
                seen.add(digest)
    by_class = collections.defaultdict(set)
    totals = collections.defaultdict(lambda: collections.Counter())
    for relative, size, link in entries:
        klass = file_class[relative]
        unit = unit_of(relative)
        units[unit][klass] += 1
        unit_bytes[unit][klass] += size
        totals[klass]['files'] += 1
        totals[klass]['apparent_bytes'] += size
        totals[klass]['checkpoint_referenced_files'] += relative in refs
        if klass.startswith('disposable'):
            by_class[klass].add(relative)
    sizes = {relative: size for relative, size, _ in entries}
    targets = collapse(file_class, sizes)
    grouped = collections.defaultdict(list)
    for path, target in targets.items():
        grouped[(target, file_class[path])].append(path)
    actions = sorted(grouped)
    exclusive = btrfs_exclusive(root, [a for a, _ in actions]) if use_btrfs else {}
    unit_exclusive = btrfs_exclusive(root, sorted(units)) if use_btrfs else {}
    action_rows = []
    for target, klass in actions:
        files = grouped[(target, klass)]
        cps = sorted(set().union(*(refs.get(p, set()) for p in files)))
        apparent = sum(sizes[p] for p in files)
        action_rows.append({'path': target, 'class': klass, 'action': 'delete', 'files': len(files),
                            'apparent_bytes': apparent, 'exclusive_bytes': exclusive.get(target),
                            'checkpoint_refs': cps[:20], 'checkpoint_ref_count': len(cps)})
        totals[klass]['exclusive_bytes'] += exclusive.get(target) or 0
    unit_rows = []
    for unit in sorted(units):
        if unit in recent_units and use_btrfs:
            totals['keep_recent']['exclusive_bytes'] += unit_exclusive.get(unit) or 0
        unit_rows.append({'unit': unit, 'classes': dict(units[unit]), 'bytes': dict(unit_bytes[unit]),
                          'exclusive_bytes': unit_exclusive.get(unit),
                          'latest_activity_utc': datetime.datetime.fromtimestamp(
                              activity[unit], datetime.timezone.utc).isoformat(timespec='seconds'),
                          'recent': unit in recent_units,
                          'referenced_by': sorted(referenced_units.get(unit, ())),
                          'doc_references': sorted(doc_units.get(unit, ()))[:10],
                          'action': ('keep' if not any(k.startswith('disposable') for k in units[unit]) else
                                     'delete' if all(k.startswith('disposable') for k in units[unit]) else
                                     'partial')})
    now = datetime.datetime.fromtimestamp(now_ts, datetime.timezone.utc)
    report = {'schema': SCHEMA, 'created_utc': now.isoformat(timespec='seconds'), 'root': str(root),
              'keep_recent_days': keep_recent_days,
              'btrfs_exclusive': use_btrfs, 'image_duplicates_hashed': hash_images,
              'totals': {k: dict(v) for k, v in sorted(totals.items())},
              'pinned': {'configs': sorted(configs), 'fixture_pins': sorted(fixture_pins & existing),
                         'missing_fixture_pins': sorted(fixture_pins - existing),
                         'code_references': sorted(p for p in code_refs if p in existing),
                         'directories': sorted(pinned_dirs)},
              'keep_recent_units': [{'unit': u['unit'], 'files': sum(u['classes'].values()),
                                     'apparent_bytes': sum(u['bytes'].values()),
                                     'exclusive_bytes': u['exclusive_bytes'],
                                     'latest_activity_utc': u['latest_activity_utc']}
                                    for u in unit_rows if u['recent']],
              'dedupe_candidates': dict(dedupe), 'notes': notes[:200], 'actions': action_rows, 'units': unit_rows}
    out_dir = root / out_dir
    out_dir.mkdir(parents=True, exist_ok=True)
    stamp = now.strftime('%Y%m%dT%H%M%SZ')
    plan_path = out_dir / f'plan-{stamp}.json'
    plan_path.write_text(json.dumps(report, ensure_ascii=False, indent=1) + '\n')
    digest = sha256_file(plan_path)
    summary = out_dir / f'plan-{stamp}.md'
    summary.write_text(markdown(report, plan_path.relative_to(root).as_posix(), digest))
    return plan_path, digest, summary, report


def human(value):
    value = float(value or 0)
    for unit in ('B', 'KiB', 'MiB', 'GiB', 'TiB'):
        if value < 1024 or unit == 'TiB':
            return f'{value:.1f} {unit}'
        value /= 1024


def markdown(report, plan_path, digest):
    lines = [f'# Kế hoạch dọn local/ (dry run) {report["created_utc"]}', '',
             f'Plan: `{plan_path}` SHA-256 `{digest}`. Chưa xóa gì. Áp dụng chỉ khi chủ dự án duyệt đúng hash này:',
             '', f'`python3 tools/local_retention.py --apply --plan {plan_path} --plan-sha256 {digest}`', '',
             '| Lớp | File | Byte biểu kiến | Byte exclusive (btrfs, cận dưới) | File có CP tham chiếu |',
             '|---|---:|---:|---:|---:|']
    for klass, row in report['totals'].items():
        lines.append(f'| {klass} | {row.get("files", 0)} | {human(row.get("apparent_bytes"))} | '
                     f'{human(row.get("exclusive_bytes")) if report["btrfs_exclusive"] and klass.startswith(("disposable", "keep_recent")) else "-"} | '
                     f'{row.get("checkpoint_referenced_files", 0)} |')
    pinned = report['pinned']
    lines += ['', f'Guard keep_recent: unit có file mtime/ctime trong {report["keep_recent_days"]} ngày gần nhất '
              f'(0 = tắt) không bị dọn; apply kiểm lại. {len(report["keep_recent_units"])} unit.']
    for row in report['keep_recent_units']:
        lines.append(f'- `{row["unit"]}`: {row["files"]} file, {human(row["apparent_bytes"])} biểu kiến, '
                     f'{human(row["exclusive_bytes"]) if row["exclusive_bytes"] is not None else "-"} exclusive, '
                     f'hoạt động cuối {row["latest_activity_utc"]}')
    lines += ['', f'Pinned: {len(pinned["configs"])} config `kr-rust-*`, {len(pinned["fixture_pins"])} fixture pin, '
              f'{len(pinned["code_references"])} đường dẫn do code/tools nhắc; thiếu {len(pinned["missing_fixture_pins"])} fixture pin.',
              '', f'Ảnh giữ lại trùng byte trong cùng run (không xóa, có thể dedupe reflink): '
              f'{report["dedupe_candidates"].get("files", 0)} file, {human(report["dedupe_candidates"].get("apparent_bytes"))}.',
              '', '## Unit thu hồi nhiều nhất', '', '| Unit | Hành động | Exclusive | Byte biểu kiến disposable | CP |',
              '|---|---|---:|---:|---|']
    ranked = sorted(report['units'], key=lambda u: -sum(v for k, v in u['bytes'].items() if k.startswith('disposable')))
    for unit in ranked[:25]:
        disposable = sum(v for k, v in unit['bytes'].items() if k.startswith('disposable'))
        lines.append(f'| `{unit["unit"]}` | {unit["action"]} | {human(unit["exclusive_bytes"]) if unit["exclusive_bytes"] is not None else "-"} | '
                     f'{human(disposable)} | {", ".join(unit["referenced_by"][:4])}{"…" if len(unit["referenced_by"]) > 4 else ""} |')
    lines += ['', 'Exclusive của từng đường dẫn chỉ tính extent không chia sẻ với file nào khác, nên là cận dưới: '
              'xóa cùng lúc nhiều bản sao reflink có thể thu hồi thêm phần đang chia sẻ giữa chúng.', '']
    return '\n'.join(lines)


def apply(root, plan_file, expected, now=None):
    plan_path = Path(plan_file)
    if not plan_path.is_absolute():
        plan_path = root / plan_path
    if sha256_file(plan_path) != expected:
        raise ValueError('plan SHA-256 does not match the approved hash')
    report = json.loads(plan_path.read_bytes())
    if report.get('schema') != SCHEMA:
        raise ValueError('not a retention plan')
    if 'keep_recent_days' not in report:
        raise ValueError('plan has no keep_recent_days guard; regenerate it with the current tool')
    guard_days = report['keep_recent_days']
    now_ts = datetime.datetime.now(datetime.timezone.utc).timestamp() if now is None else now
    registry_path = root / lean.PRUNED
    registry = (json.loads(registry_path.read_bytes()) if registry_path.exists() else
                {'schema': lean.PRUNED_SCHEMA,
                 'authorization': 'docs/evidence/owner-directives/local-retention.json', 'entries': []})
    lean.load_pruned(root)  # existing registry must be valid
    known = {entry['path'] for entry in registry['entries']}
    refs, _ = checkpoint_references(root, lambda message: None)
    keep = set(report['pinned']['configs']) | set(report['pinned']['fixture_pins']) | set(report['pinned']['code_references'])
    # O(1) per-action checks: every path and each of its ancestor directories, so
    # `target in keep_covered` == "some pinned/referenced path equals or lies under target".
    keep_covered, ref_covered = path_closure(keep), path_closure(refs)
    unit_touched = {}  # unit -> latest activity, scanned once per unit (no deletion can lower it for a kept unit)
    today = datetime.date.today().isoformat()
    deleted, skipped, skipped_recent = 0, [], []
    for action in report['actions']:
        target, klass = action['path'], action['class']
        path = root / target
        if (action.get('action') != 'delete' or not klass.startswith('disposable') or not target.startswith('local/')
                or '..' in target.split('/') or covered(target, set(ALWAYS_KEEP))
                or target in keep_covered):
            skipped.append({'path': target, 'reason': 'not an allowed delete action'})
            continue
        if not os.path.lexists(path):
            skipped.append({'path': target, 'reason': 'already absent'})
            continue
        unit = unit_of(target)
        if unit not in unit_touched:
            unit_touched[unit] = unit_activity(root, unit)
        touched = unit_touched[unit]
        if is_recent(touched, now_ts, guard_days):
            row = {'path': target, 'reason': f'unit {unit} has activity within {guard_days} days at apply time',
                   'latest_activity_utc': datetime.datetime.fromtimestamp(
                       touched, datetime.timezone.utc).isoformat(timespec='seconds')}
            skipped.append(row)
            skipped_recent.append(row)
            continue
        referenced = target in ref_covered
        if path.is_dir() and not path.is_symlink():
            shutil.rmtree(path)
        else:
            path.unlink()
        deleted += 1
        if referenced and target not in known:
            registry['entries'].append({'path': target, 'class': klass,
                                        'reason': f'retention plan {plan_path.name}: {klass}',
                                        'date': today, 'plan_sha256': expected})
            known.add(target)
    registry['entries'].sort(key=lambda entry: entry['path'])
    registry_path.write_bytes(store.encoded(registry))
    return {'deleted': deleted, 'skipped': skipped[:50], 'skipped_count': len(skipped),
            'skipped_recent_count': len(skipped_recent), 'skipped_recent': skipped_recent[:50],
            'pruned_entries': len(registry['entries']), 'registry': lean.PRUNED}


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument('--root', type=Path, default=store.ROOT)
    parser.add_argument('--out-dir', default='local/retention')
    parser.add_argument('--btrfs', action='store_true', help='measure exclusive bytes with btrfs filesystem du')
    parser.add_argument('--no-image-hash', action='store_true', help='skip counting byte-identical kept images')
    parser.add_argument('--keep-recent-days', type=float, default=DEFAULT_KEEP_RECENT_DAYS, metavar='N',
                        help='units with a file touched (mtime/ctime) within N days are keep_recent; 0 disables')
    parser.add_argument('--apply', action='store_true', help='delete per an approved plan (needs --plan/--plan-sha256)')
    parser.add_argument('--plan')
    parser.add_argument('--plan-sha256')
    args = parser.parse_args(argv)
    root = args.root.resolve()
    if args.apply:
        if not args.plan or not args.plan_sha256 or not lean.SHA.fullmatch(args.plan_sha256):
            parser.error('--apply requires --plan FILE and --plan-sha256 of the owner-approved plan')
        print(json.dumps(apply(root, args.plan, args.plan_sha256), ensure_ascii=False, indent=2))
        return 0
    plan_path, digest, summary, report = plan(root, args.btrfs, not args.no_image_hash, args.out_dir, args.keep_recent_days)
    print(json.dumps({'plan': str(plan_path), 'plan_sha256': digest, 'summary': str(summary),
                      'totals': report['totals']}, ensure_ascii=False, indent=2))
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
