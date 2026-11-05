"""Selective checkpoint storage access. No acceptance, semantic or gate decisions."""
from collections.abc import Sequence
import hashlib
import json
from pathlib import Path
import re

ROOT = Path(__file__).resolve().parents[1]
MANIFEST = Path('docs/evidence/checkpoints/manifest.json')
V1 = 'kre-checkpoints-v1'
V2 = 'kre-checkpoints-v2'
EVIDENCE_SCHEMA = 'kre-evidence-manifest-v1'


def encoded(value):
    return (json.dumps(value, ensure_ascii=False, indent=2) + '\n').encode()


def object_digest(value):
    """JSON object identity, independent of serialization whitespace/key order."""
    return hashlib.sha256(json.dumps(value, ensure_ascii=False, sort_keys=True,
                                     separators=(',', ':')).encode()).hexdigest()


def target(root, relative):
    if not isinstance(relative, str) or not relative or Path(relative).is_absolute():
        raise ValueError(f'invalid repository path: {relative}')
    path = (Path(root) / relative).resolve()
    if not path.is_relative_to(Path(root).resolve()):
        raise ValueError(f'path outside repository: {relative}')
    return path


def file_reader(root):
    return lambda path: target(root, path).read_bytes()


def pinned_json(reference, read):
    if not isinstance(reference, dict):
        raise ValueError('expected pinned reference object')
    path = reference.get('path')
    # Enforce the same path boundary even for Git readers.
    target(ROOT, path)
    digest = reference.get('sha256')
    size = reference.get('bytes')
    if (not isinstance(digest, str) or not re.fullmatch('[0-9a-f]{64}', digest)
            or type(size) is not int or size < 0):
        raise ValueError(f'invalid reference size/hash: {path}')
    raw = read(path)
    if len(raw) != size or hashlib.sha256(raw).hexdigest() != digest:
        raise ValueError(f'reference size/hash mismatch: {path}')
    return json.loads(raw)


class Records(Sequence):
    """At most one record decoded per access; no eager all-history expansion."""
    def __init__(self, manifest, read):
        if manifest.get('schema') not in {V1, V2}:
            raise ValueError('unsupported checkpoint schema')
        if not isinstance(manifest.get('checkpoints'), list):
            raise ValueError('checkpoints must be a list')
        if manifest['schema'] == V2 and manifest.get('version') != 2:
            raise ValueError('unsupported checkpoint index version')
        self.manifest, self.read = manifest, read
        self.rows = manifest['checkpoints']

    def __len__(self):
        return len(self.rows)

    def __getitem__(self, index):
        if isinstance(index, slice):
            return [self[i] for i in range(*index.indices(len(self)))]
        row = self.rows[index]
        if self.manifest['schema'] == V1:
            return row
        record = pinned_json(row, self.read)
        if not isinstance(record, dict):
            raise ValueError(f'{row.get("id")}: record must be an object')
        for key in ('id', 'lifecycle', 'recorded_utc'):
            if row.get(key) != record.get(key):
                raise ValueError(f'{row.get("id")}: navigation metadata mismatch: {key}')
        if not re.fullmatch(r'CP[0-9]{3,}', str(record.get('id', ''))):
            raise ValueError('invalid checkpoint ID')
        expected = f'docs/evidence/checkpoints/records/{record["id"]}.json'
        if row.get('path') != expected:
            raise ValueError(f'{record["id"]}: unexpected record path')
        return record


def records(manifest, root=ROOT, read=None):
    return Records(manifest, read or file_reader(root))


def load(root=ROOT):
    return json.loads((Path(root) / MANIFEST).read_bytes())


def current_view(manifest, root=ROOT):
    """Current issue checks need only task declarations and the latest record."""
    result = dict(manifest)
    result['checkpoints'] = records(manifest, root)[-1:]
    return result


def evidence(record, root=ROOT, read=None):
    """Yield legacy inline rows and pinned run/batch lists, one manifest at a time.

    External manifests cannot nest; identifiers must be unique across the record.
    Hash/count checks happen before any row from each manifest is yielded.
    """
    read = read or file_reader(root)
    rows = record.get('evidence', [])
    if not isinstance(rows, list):
        raise ValueError('record evidence must be a list')
    yield from rows
    references = record.get('evidence_manifests', [])
    if not isinstance(references, list):
        raise ValueError('evidence_manifests must be a list')
    for reference in references:
        if not isinstance(reference, dict):
            raise ValueError('invalid evidence manifest reference')
        path = reference.get('path', '')
        if not path.startswith('docs/evidence/checkpoints/evidence/'):
            raise ValueError(f'evidence manifest outside evidence directory: {path}')
        obj = pinned_json(reference, read)
        if (not isinstance(obj, dict) or obj.get('schema') != EVIDENCE_SCHEMA
                or not isinstance(obj.get('evidence'), list)
                or 'evidence_manifests' in obj):
            raise ValueError(f'invalid/nested evidence manifest: {path}')
        if type(reference.get('count')) is not int or reference['count'] != len(obj['evidence']):
            raise ValueError(f'evidence manifest count mismatch: {path}')
        yield from obj['evidence']


def record_reference(record, relative, raw=None):
    raw = encoded(record) if raw is None else raw
    return {key: record[key] for key in ('id', 'lifecycle', 'recorded_utc')} | {
        'path': relative, 'bytes': len(raw), 'sha256': hashlib.sha256(raw).hexdigest()}
