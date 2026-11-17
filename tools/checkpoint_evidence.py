"""Lean checkpoint evidence storage: host-local file lists live in content-addressed run manifests.

A record stores in the repository only the rows that must stay resolvable without `local/`
(repository/artifact rows, rows cited by the record text, duplicates, single-row groups) in
`evidence`, plus one `evidence_runs` pointer per run directory. Each pointer pins a manifest at
`local/evidence-manifests/<sha256>.jsonl` (path, size, SHA-256, row count, sum of file bytes).

The split is a pure function of the record and its full evidence list (`pack`), so the logical
identity of a record (`logical_digest`) is the same before and after packing and can be computed
from Git history alone. No acceptance, semantic or gate decisions are made here.
"""
import hashlib
import json
import os
from pathlib import Path
import re

import checkpoint_store as store

RUNS_DIR = 'local/evidence-manifests'
RUN_SCHEMA = 'kre-evidence-run-v1'
MIN_RUN_ROWS = 2
RUN_DEPTH = 4
EVIDENCE_KEYS = ('evidence', 'evidence_manifests', 'evidence_runs')
ROW_KEYS = {'id', 'kind', 'path', 'bytes', 'sha256'}
POINTER_KEYS = {'run', 'path', 'sha256', 'bytes', 'count', 'files_bytes'}
SHA = re.compile(r'[0-9a-f]{64}')
TOKEN = re.compile(r'[A-Za-z0-9_-]+')


def canonical(value):
    return json.dumps(value, ensure_ascii=False, sort_keys=True, separators=(',', ':'))


def movable(row):
    """Only well-formed relative host-local rows may leave the repository."""
    if not isinstance(row, dict) or set(row) != ROW_KEYS or row.get('kind') != 'local':
        return False
    path, ident = row.get('path'), row.get('id')
    return (isinstance(ident, str) and bool(ident) and isinstance(path, str)
            and path.startswith('local/') and not any(c in path for c in '\n\r\\')
            and all(part not in ('', '.', '..') for part in path.split('/'))
            and isinstance(row.get('sha256'), str) and bool(SHA.fullmatch(row['sha256']))
            and type(row.get('bytes')) is int and row['bytes'] >= 0)


def run_key(path):
    """Run directory: parent directory truncated to the first RUN_DEPTH components."""
    return '/'.join(path.split('/')[:-1][:RUN_DEPTH])


def _strings(value):
    if isinstance(value, str):
        yield value
    elif isinstance(value, dict):
        for key, item in value.items():
            yield key
            yield from _strings(item)
    elif isinstance(value, list):
        for item in value:
            yield from _strings(item)


def cited_ids(record, ids):
    """Evidence IDs mentioned anywhere outside the evidence lists (findings, verification, text)."""
    body = {k: v for k, v in record.items() if k not in EVIDENCE_KEYS}
    words = set()
    for text in _strings(body):
        words.add(text)
        words.update(TOKEN.findall(text))
    return set(ids) & words


def manifest_bytes(run, rows):
    lines = [canonical({'schema': RUN_SCHEMA, 'run': run})] + [canonical(row) for row in rows]
    return ('\n'.join(lines) + '\n').encode()


def manifest_path(digest):
    return f'{RUNS_DIR}/{digest}.jsonl'


def pack(record, rows):
    """Deterministic split of a full evidence list -> (inline rows, [(pointer, manifest bytes)])."""
    rows = list(rows)
    seen, duplicate = set(), set()
    for row in rows:
        ident = row.get('id') if isinstance(row, dict) else None
        if isinstance(ident, str):
            (duplicate if ident in seen else seen).add(ident)
    cited = cited_ids(record, seen)
    groups = {}
    for row in rows:
        if movable(row) and row['id'] not in cited and row['id'] not in duplicate:
            groups.setdefault(run_key(row['path']), []).append(row)
    keep = {key for key, members in groups.items() if len(members) >= MIN_RUN_ROWS}
    inline = [row for row in rows
              if not (movable(row) and row['id'] not in cited and row['id'] not in duplicate
                      and run_key(row['path']) in keep)]
    runs = []
    for key, members in groups.items():
        if key not in keep:
            continue
        raw = manifest_bytes(key, members)
        digest = hashlib.sha256(raw).hexdigest()
        runs.append(({'run': key, 'path': manifest_path(digest), 'sha256': digest, 'bytes': len(raw),
                      'count': len(members), 'files_bytes': sum(r['bytes'] for r in members)}, raw))
    return inline, runs


def is_packed(record):
    return 'evidence_runs' in record


def check_pointer(pointer):
    if not isinstance(pointer, dict) or set(pointer) != POINTER_KEYS:
        raise ValueError('invalid evidence run pointer')
    if (not isinstance(pointer['run'], str) or not pointer['run'].startswith('local/')
            or not isinstance(pointer['sha256'], str) or not SHA.fullmatch(pointer['sha256'])
            or pointer['path'] != manifest_path(pointer['sha256'])
            or any(type(pointer[k]) is not int or pointer[k] < 0 for k in ('bytes', 'count', 'files_bytes'))
            or pointer['count'] < MIN_RUN_ROWS):
        raise ValueError(f'invalid evidence run pointer: {pointer.get("path")}')
    return pointer


def pointers(record):
    value = record.get('evidence_runs', [])
    if not isinstance(value, list):
        raise ValueError('evidence_runs must be a list')
    if 'evidence_runs' in record and record.get('evidence_manifests'):
        raise ValueError('a packed record cannot also use evidence_manifests')
    return [check_pointer(pointer) for pointer in value]


def parse_manifest(pointer, raw):
    """Validate pinned bytes and every row; return rows. Raises ValueError on any mismatch."""
    if len(raw) != pointer['bytes'] or hashlib.sha256(raw).hexdigest() != pointer['sha256']:
        raise ValueError('evidence run manifest size/hash mismatch')
    lines = raw.decode().split('\n')
    if lines[-1] != '':
        raise ValueError('evidence run manifest must end with a newline')
    header = json.loads(lines[0])
    if header != {'schema': RUN_SCHEMA, 'run': pointer['run']}:
        raise ValueError('evidence run manifest header mismatch')
    rows = [json.loads(line) for line in lines[1:-1]]
    if len(rows) != pointer['count']:
        raise ValueError('evidence run manifest count mismatch')
    for row in rows:
        if not movable(row) or run_key(row['path']) != pointer['run']:
            raise ValueError(f'evidence run manifest row outside run: {row}')
    if sum(row['bytes'] for row in rows) != pointer['files_bytes']:
        raise ValueError('evidence run manifest files_bytes mismatch')
    if manifest_bytes(pointer['run'], rows) != raw:
        raise ValueError('evidence run manifest is not canonical')
    return rows


def manifest_file(root, pointer):
    """Fixed `local/evidence-manifests/<64 hex>.jsonl` name; content is pinned by SHA-256, so a
    worktree whose `local/` is a symlink to the main checkout may read/write it."""
    check_pointer(pointer)
    return Path(root) / pointer['path']


def read_run(pointer, root=store.ROOT):
    """Rows of one run. FileNotFoundError when the host has no manifest; ValueError if corrupt."""
    path = manifest_file(root, pointer)
    if not path.is_file():
        raise FileNotFoundError(pointer['path'])
    return parse_manifest(pointer, path.read_bytes())


def all_rows(record, root=store.ROOT, read=None):
    """Full evidence list: repository rows (inline + pinned evidence_manifests) then run rows."""
    rows = list(store.evidence(record, root, read))
    for pointer in pointers(record):
        rows.extend(read_run(pointer, root))
    return rows


def identity(record, root=store.ROOT, read=None, rows=None):
    """Storage-independent evidence identity. Needs only repository data (never local/).

    `rows` may pass the already expanded repository rows of an unpacked record.
    """
    if is_packed(record):
        runs = pointers(record)
        rows = record.get('evidence', [])
        if not isinstance(rows, list):
            raise ValueError('record evidence must be a list')
        return store.object_digest({'evidence': rows, 'evidence_runs': runs})
    inline, runs = pack(record, store.evidence(record, root, read) if rows is None else rows)
    return store.object_digest({'evidence': inline, 'evidence_runs': [p for p, _ in runs]})


def logical_digest(record, root=store.ROOT, read=None, rows=None):
    """Record identity used for append-only history; review and evidence storage excluded."""
    body = {k: v for k, v in record.items() if k not in ('review',) + EVIDENCE_KEYS}
    body['evidence identity'] = identity(record, root, read, rows)
    return store.object_digest(body)


def write_manifest(root, pointer, raw):
    """Content-addressed, write-once; an existing file must be byte-identical."""
    path = manifest_file(root, pointer)
    if path.exists():
        if path.read_bytes() != raw:
            raise ValueError(f'existing evidence run manifest differs: {pointer["path"]}')
        return False
    path.parent.mkdir(parents=True, exist_ok=True)
    temporary = path.with_name(f'.{path.name}.{os.getpid()}.tmp')
    temporary.write_bytes(raw)
    os.replace(temporary, path)
    return True


def pack_evidence(record, root=store.ROOT, rows=None):
    """Helper for checkpoint writers: return a packed copy of `record` and write its run manifests.

    Call it last, after findings/verification are final (cited IDs stay in the repository).
    `rows` defaults to the record's current full evidence (inline + evidence_manifests + runs).
    """
    rows = all_rows(record, root) if rows is None else list(rows)
    inline, runs = pack(record, rows)
    for pointer, raw in runs:
        write_manifest(root, pointer, raw)
    result = {k: v for k, v in record.items() if k not in EVIDENCE_KEYS}
    result['evidence'] = inline
    result['evidence_runs'] = [pointer for pointer, _ in runs]
    unpacked = {k: v for k, v in record.items() if k not in EVIDENCE_KEYS} | {'evidence': rows}
    if logical_digest(result, root) != logical_digest(unpacked, root):
        raise ValueError(f'{record.get("id")}: packed evidence identity differs')
    return result


def legacy_movable_count(record, root=store.ROOT, read=None):
    """Rows that canonical packing would move out of the repository (0 for a packed record)."""
    if is_packed(record):
        return 0
    _, runs = pack(record, store.evidence(record, root, read))
    return sum(pointer['count'] for pointer, _ in runs)


def canonical_errors(record, rows):
    """Packed record must equal the canonical pack of its own full row list."""
    inline, runs = pack(record, rows)
    if inline != record.get('evidence') or [p for p, _ in runs] != record.get('evidence_runs'):
        return ['evidence runs are not the canonical packing of this record']
    return []


def manifest_files(root=store.ROOT):
    directory = Path(root) / RUNS_DIR
    return sorted(directory.glob('*.jsonl')) if directory.is_dir() else []


PRUNED = 'docs/evidence/checkpoints/local-pruned.json'
PRUNED_SCHEMA = 'kre-local-pruned-v1'
PRUNED_KEYS = {'path', 'class', 'reason', 'date', 'plan_sha256'}
DATE = re.compile(r'[0-9]{4}-[0-9]{2}-[0-9]{2}')


class Pruned:
    """Committed record of host-local evidence deliberately removed under the retention directive.

    A missing local file whose path is at or below a registered entry is reported as PRUNED, not
    UNVERIFIED; its size/SHA stay in records/run manifests for lookup. A file that still exists is
    verified as usual. Missing files without an entry still fail.
    """

    def __init__(self, entries=()):
        self.entries = {entry['path']: entry for entry in entries}
        self.hits = 0

    def match(self, path):
        if not self.entries or not isinstance(path, str):
            return None
        parts = path.split('/')
        for end in range(len(parts), 1, -1):
            entry = self.entries.get('/'.join(parts[:end]))
            if entry:
                return entry
        return None


def check_pruned_entry(entry):
    if (not isinstance(entry, dict) or set(entry) != PRUNED_KEYS
            or not isinstance(entry['path'], str) or not entry['path'].startswith('local/')
            or any(part in ('', '.', '..') for part in entry['path'].split('/'))
            or entry['path'].startswith(RUNS_DIR + '/') or entry['path'] == RUNS_DIR
            or not isinstance(entry['class'], str) or not entry['class'].startswith('disposable')
            or not isinstance(entry['reason'], str) or not entry['reason'].strip()
            or not isinstance(entry['date'], str) or not DATE.fullmatch(entry['date'])
            or not isinstance(entry['plan_sha256'], str) or not SHA.fullmatch(entry['plan_sha256'])):
        raise ValueError(f'invalid pruned entry: {entry}')
    return entry


def load_pruned(root=store.ROOT, read=None):
    """Registry from the working tree (or `read`); absent file means nothing was pruned."""
    try:
        raw = (read or store.file_reader(root))(PRUNED)
    except (FileNotFoundError, ValueError):
        return Pruned()
    data = json.loads(raw)
    if not isinstance(data, dict) or data.get('schema') != PRUNED_SCHEMA or not isinstance(data.get('entries'), list):
        raise ValueError(f'{PRUNED}: invalid schema')
    entries = [check_pruned_entry(entry) for entry in data['entries']]
    if len({e['path'] for e in entries}) != len(entries):
        raise ValueError(f'{PRUNED}: duplicate path')
    return Pruned(entries)
