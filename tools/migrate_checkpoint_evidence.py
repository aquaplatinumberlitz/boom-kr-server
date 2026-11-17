#!/usr/bin/env python3
"""Deterministic, idempotent move of host-local evidence lists into content-addressed run manifests.

For every record that still stores movable host-local rows in the repository (inline `evidence`
or pinned `evidence_manifests`), write `local/evidence-manifests/<sha256>.jsonl` per run and
replace the record by its canonical packing (`checkpoint_evidence.pack`). The logical identity
(`checkpoint_evidence.logical_digest`) and the full row multiset are compared for every record
before anything is written. Pinned evidence-manifest files no longer referenced by any record are
deleted; their bytes stay in Git history at --source-commit. Git history is never rewritten.
Re-running on a migrated tree changes nothing; new legacy records are migrated and appended to the
report. No semantic, acceptance or gate decision is made.
"""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess
import sys

if str(Path(__file__).resolve().parent) not in sys.path:
    sys.path.insert(0, str(Path(__file__).resolve().parent))
import checkpoint_evidence as lean
import checkpoint_store as store

REPORT = 'docs/evidence/checkpoints/migration-evidence-lean.json'
REPORT_SCHEMA = 'kre-checkpoint-evidence-lean-migration-v1'
EVIDENCE_DIR = 'docs/evidence/checkpoints/evidence/'


def git_bytes(root, commit, relative):
    result = subprocess.run(['git', '-c', 'core.fsmonitor=false', 'show', f'{commit}:{relative}'],
                            cwd=root, capture_output=True)
    return result.stdout if result.returncode == 0 else None


def sha(raw):
    return hashlib.sha256(raw).hexdigest()


def row_multiset(rows):
    return sorted(lean.canonical(row) for row in rows)


def write_report(path, report):
    """Readable but compact: one line per record/file entry."""
    head = {k: v for k, v in report.items() if k not in ('records', 'deleted_evidence_files')}
    text = json.dumps(head, ensure_ascii=False, indent=2)[:-2]
    for key in ('records', 'deleted_evidence_files'):
        items = report[key]
        body = ',\n'.join('    ' + json.dumps(item, ensure_ascii=False, separators=(', ', ': '))
                          for item in items)
        text += f',\n  "{key}": [\n{body}\n  ]' if items else f',\n  "{key}": []'
    path.write_text(text + '\n}\n')


def migrate(root, source_commit, dry_run=False):
    root = Path(root).resolve()
    commit = subprocess.run(['git', 'rev-parse', '--verify', f'{source_commit}^{{commit}}'], cwd=root,
                            capture_output=True, text=True)
    if commit.returncode:
        raise ValueError(f'unknown source commit: {source_commit}')
    source_commit = commit.stdout.strip()
    index_path = root / store.MANIFEST
    index = json.loads(index_path.read_bytes())
    if index.get('schema') != store.V2:
        raise ValueError('checkpoint index must be kre-checkpoints-v2')
    report_path = root / REPORT
    report = (json.loads(report_path.read_bytes()) if report_path.exists() else
              {'schema': REPORT_SCHEMA, 'manifest_dir': lean.RUNS_DIR, 'run_schema': lean.RUN_SCHEMA,
               'limits': ('Storage-only migration. Every record keeps its logical identity '
                          '(checkpoint_evidence.logical_digest) and exact evidence row multiset; '
                          'findings/review/raw are unchanged. Full pre-migration lists stay in Git at '
                          'source_commit (tools/evidence_lookup.py --from-history). No semantic or gate claim.'),
               'records': [], 'deleted_evidence_files': []})
    done = {entry['id'] for entry in report['records']}
    records = store.records(index, root)
    stats = {'records_migrated': 0, 'rows_moved': 0, 'runs': 0, 'manifests_written': 0}
    pending = []
    for position, row in enumerate(index['checkpoints']):
        record = records[position]
        if lean.is_packed(record):
            continue
        rows = list(store.evidence(record, root))
        inline, runs = lean.pack(record, rows)
        if not runs:
            continue
        if record['id'] in done:
            raise ValueError(f'{record["id"]}: reported as migrated but stored unpacked')
        old_raw = (root / row['path']).read_bytes()
        historical = git_bytes(root, source_commit, row['path'])
        if historical != old_raw:
            raise ValueError(f'{record["id"]}: record at {source_commit[:12]} differs from working tree; '
                             'migrate from a source commit that contains this exact record')
        for reference in record.get('evidence_manifests', []):
            pinned = git_bytes(root, source_commit, reference['path'])
            if pinned is None or sha(pinned) != reference['sha256']:
                raise ValueError(f'{record["id"]}: {reference["path"]} not pinned at source commit')
        packed = {k: v for k, v in record.items() if k not in lean.EVIDENCE_KEYS}
        packed['evidence'] = inline
        packed['evidence_runs'] = [pointer for pointer, _ in runs]
        # Equivalence before writing: logical identity and exact row multiset.
        if lean.logical_digest(packed, root) != lean.logical_digest(record, root):
            raise ValueError(f'{record["id"]}: logical identity changed')
        expanded = inline + [r for _, raw in runs for r in
                             (json.loads(line) for line in raw.decode().splitlines()[1:])]
        if row_multiset(expanded) != row_multiset(rows):
            raise ValueError(f'{record["id"]}: evidence rows changed')
        if lean.canonical_errors(packed, expanded):
            raise ValueError(f'{record["id"]}: packing is not canonical')
        pending.append((position, row, record, packed, runs, old_raw))
    for position, row, record, packed, runs, old_raw in pending:
        new_raw = store.encoded(packed)
        if not dry_run:
            for pointer, raw in runs:
                stats['manifests_written'] += lean.write_manifest(root, pointer, raw)
            (root / row['path']).write_bytes(new_raw)
        index['checkpoints'][position] = store.record_reference(packed, row['path'], new_raw)
        moved = sum(pointer['count'] for pointer, _ in runs)
        stats['records_migrated'] += 1
        stats['rows_moved'] += moved
        stats['runs'] += len(runs)
        report['records'].append({
            'id': record['id'], 'source_commit': source_commit,
            'old_sha256': sha(old_raw), 'old_bytes': len(old_raw),
            'old_evidence_manifests': [r['path'] for r in record.get('evidence_manifests', [])],
            'new_sha256': sha(new_raw), 'new_bytes': len(new_raw),
            'logical_sha256': lean.logical_digest(packed, root),
            'repository_rows': len(packed['evidence']), 'run_rows': moved, 'runs': len(runs)})
    # Pinned list files that no record references any more leave the working tree only.
    referenced = set()
    replaced = {position for position, *_ in pending}
    for position in range(len(records)):
        if position not in replaced:
            referenced.update(r['path'] for r in records[position].get('evidence_manifests', []))
    deleted = {entry['path'] for entry in report['deleted_evidence_files']}
    evidence_dir = root / EVIDENCE_DIR
    for path in sorted(evidence_dir.glob('*.json')) if evidence_dir.is_dir() else []:
        relative = path.relative_to(root).as_posix()
        if relative in referenced or relative in deleted:
            continue
        raw = path.read_bytes()
        if git_bytes(root, source_commit, relative) != raw:
            raise ValueError(f'{relative}: not identical at source commit; refusing to delete')
        report['deleted_evidence_files'].append({'path': relative, 'sha256': sha(raw), 'bytes': len(raw),
                                                 'source_commit': source_commit})
        if not dry_run:
            path.unlink()
    stats['evidence_files_deleted'] = len(report['deleted_evidence_files']) - len(deleted)
    report['records'].sort(key=lambda entry: entry['id'])
    report['deleted_evidence_files'].sort(key=lambda entry: entry['path'])
    if not dry_run and (stats['records_migrated'] or stats['evidence_files_deleted']):
        index_path.write_bytes(store.encoded(index))
        write_report(report_path, report)
    return stats


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--root', type=Path, default=store.ROOT)
    parser.add_argument('--source-commit', required=True,
                        help='commit (on main) that contains the exact unmigrated records/lists, e.g. origin/main')
    parser.add_argument('--dry-run', action='store_true')
    args = parser.parse_args(argv)
    try:
        print(json.dumps(migrate(args.root, args.source_commit, args.dry_run), ensure_ascii=False))
    except (OSError, ValueError, KeyError, TypeError) as error:
        parser.exit(1, f'migration failed: {error}\n')
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
