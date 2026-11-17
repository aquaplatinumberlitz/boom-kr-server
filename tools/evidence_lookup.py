#!/usr/bin/env python3
"""Find which checkpoint/run references an evidence file, hash or ID; verify on demand.

Searches repository rows (record `evidence`, legacy pinned `evidence_manifests`) and host-local
run manifests (`evidence_runs` -> local/evidence-manifests/<sha256>.jsonl). With --from-history
COMMIT it reads the index/records/lists stored in Git at that commit (e.g. the pre-migration full
lists named in docs/evidence/checkpoints/migration-evidence-lean.json). `restore` rebuilds missing
run manifests byte-for-byte from those Git lists. Output is one JSON object per line.
No semantic, acceptance or gate decision is made.
"""
import argparse
import json
from pathlib import Path
import subprocess
import sys

if str(Path(__file__).resolve().parent) not in sys.path:
    sys.path.insert(0, str(Path(__file__).resolve().parent))
import checkpoint_evidence as lean
import checkpoint_store as store

REPORT = 'docs/evidence/checkpoints/migration-evidence-lean.json'


def git_reader(root, commit):
    def read(relative):
        store.target(root, relative)
        result = subprocess.run(['git', '-c', 'core.fsmonitor=false', 'show', f'{commit}:{relative}'],
                                cwd=root, capture_output=True)
        if result.returncode:
            raise FileNotFoundError(f'{relative} not in {commit}')
        return result.stdout
    return read


def load_index(root, read):
    return json.loads(read(store.MANIFEST.as_posix()))


def relative_query(root, text):
    """Accept repository-relative paths or absolute paths into this checkout or its local/ tree."""
    path = Path(text)
    if not path.is_absolute():
        return text.rstrip('/') if text not in ('', '/') else ''
    for base, prefix in ((root / 'local', 'local'), (root, '')):
        for candidate in (base, base.resolve()):
            try:
                rest = path.relative_to(candidate).as_posix()
            except ValueError:
                continue
            return f'{prefix}/{rest}'.strip('/') if prefix else rest
    return text


def path_matches(query, path):
    return isinstance(path, str) and (path == query or path.startswith(query + '/') or query == '')


def run_related(query, run):
    return query == '' or query == run or query.startswith(run + '/') or run.startswith(query + '/')


def iter_rows(root, read, cp_filter=None, run_filter=None, warn=print):
    """Yield (record, row, stored_in). run_filter(run) decides which local manifests to open."""
    index = load_index(root, read)
    records = store.records(index, root, read)
    for position, entry in enumerate(index['checkpoints']):
        if cp_filter and entry.get('id') not in cp_filter:
            continue
        record = records[position]
        for row in record.get('evidence', []):
            yield record, row, entry.get('path') if index['schema'] == store.V2 else 'manifest (v1 inline)'
        for reference in record.get('evidence_manifests', []):
            obj = store.pinned_json(reference, read)
            for row in obj['evidence']:
                yield record, row, reference['path']
        for pointer in lean.pointers(record):
            if run_filter is not None and not run_filter(pointer['run']):
                continue
            try:
                rows = lean.read_run(pointer, root)
            except FileNotFoundError:
                warn(json.dumps({'cp': record['id'], 'missing_manifest': pointer['path'], 'run': pointer['run'],
                                 'count': pointer['count'],
                                 'hint': 'python3 tools/evidence_lookup.py restore'}, ensure_ascii=False))
                continue
            except ValueError as error:
                warn(json.dumps({'cp': record['id'], 'invalid_manifest': pointer['path'], 'error': str(error)},
                                ensure_ascii=False))
                continue
            for row in rows:
                yield record, row, f'{pointer["path"]} (run {pointer["run"]})'


def emit(record, row, stored, root, verify, require_local, pruned=None):
    result = {'cp': record['id'], **row, 'stored_in': stored}
    if pruned is not None:
        entry = pruned.match(row.get('path'))
        if entry:
            result['pruned'] = entry
    if verify:
        from check_checkpoints import verify_evidence_item
        before = pruned.hits if pruned is not None else 0
        errors, warnings, checked, missing = verify_evidence_item(row, root, require_local, record['id'], pruned)
        pruned_now = pruned is not None and pruned.hits > before
        result['verify'] = ('error' if errors else 'unavailable' if missing else
                            'pruned' if pruned_now else 'ok')
        if errors or warnings:
            result['messages'] = errors + warnings
    print(json.dumps(result, ensure_ascii=False))
    return result


def summary(root, read, ident, report):
    index = load_index(root, read)
    for position, entry in enumerate(index['checkpoints']):
        if entry.get('id') != ident:
            continue
        record = store.records(index, root, read)[position]
        runs = []
        for pointer in lean.pointers(record):
            present = (Path(root) / pointer['path']).is_file()
            runs.append(f"{pointer['run']}: {pointer['count']} files, {pointer['files_bytes']} bytes, "
                        f"{pointer['path']}{'' if present else ' (MISSING on this host)'}")
        legacy = [{k: r[k] for k in ('path', 'count')} for r in record.get('evidence_manifests', [])]
        migrated = [e for e in report.get('records', []) if e['id'] == ident]
        print(json.dumps({'cp': ident, 'record': entry.get('path'),
                          'repository_rows': len(record.get('evidence', [])),
                          'legacy_evidence_manifests': legacy,
                          'run_rows': sum(p['count'] for p in lean.pointers(record)), 'runs': runs,
                          'migration': migrated[0] if migrated else None}, ensure_ascii=False, indent=2))
        return 0
    raise ValueError(f'unknown checkpoint: {ident}')


def restore(root, commit_override, report):
    """Rebuild missing run manifests from full lists in Git; every byte must match the pointer pin."""
    index = load_index(root, store.file_reader(root))
    records = store.records(index, root)
    sources = {e['id']: e['source_commit'] for e in report.get('records', [])}
    written, missing, unrecoverable = 0, 0, []
    for position, entry in enumerate(index['checkpoints']):
        record = records[position]
        wanted = [p for p in lean.pointers(record) if not (Path(root) / p['path']).is_file()]
        if not wanted:
            continue
        missing += len(wanted)
        commit = commit_override or sources.get(record['id'])
        if not commit:
            unrecoverable.append({'cp': record['id'], 'reason': 'record was created packed; no full list in Git',
                                  'manifests': [p['path'] for p in wanted]})
            continue
        read = git_reader(root, commit)
        old = store.records(load_index(root, read), root, read)
        position_old = next((i for i, e in enumerate(load_index(root, read)['checkpoints'])
                             if e.get('id') == record['id']), None)
        if position_old is None:
            unrecoverable.append({'cp': record['id'], 'reason': f'not in {commit}'})
            continue
        old_record = old[position_old]
        rows = list(lean.all_rows(old_record, root, read)) if lean.is_packed(old_record) else \
            list(store.evidence(old_record, root, read))
        _, runs = lean.pack(record, rows)
        by_sha = {pointer['sha256']: raw for pointer, raw in runs}
        for pointer in wanted:
            raw = by_sha.get(pointer['sha256'])
            if raw is None:
                unrecoverable.append({'cp': record['id'], 'reason': f'no list in {commit} reproduces',
                                      'manifest': pointer['path']})
                continue
            written += lean.write_manifest(root, pointer, raw)
    print(json.dumps({'missing': missing, 'restored': written, 'unrecoverable': unrecoverable},
                     ensure_ascii=False, indent=2))
    return 1 if unrecoverable else 0


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument('--root', type=Path, default=store.ROOT)
    parser.add_argument('--from-history', metavar='COMMIT', help='read index/records/lists from this Git commit')
    parser.add_argument('--cp', action='append', help='limit to checkpoint ID (repeatable)')
    parser.add_argument('--verify', action='store_true', help='verify size/SHA-256 of every match')
    parser.add_argument('--require-local', action='store_true', help='with --verify: missing host file is an error')
    parser.add_argument('--limit', type=int, default=200, help='max matches printed (0 = all)')
    sub = parser.add_subparsers(dest='command', required=True)
    sub.add_parser('path', help='file path or directory prefix (relative or absolute)').add_argument('query')
    sub.add_parser('sha', help='SHA-256 or prefix of at least 8 hex digits').add_argument('query')
    sub.add_parser('id', help='evidence ID (IDs repeat across checkpoints; combine with --cp)').add_argument('query')
    sub.add_parser('cp', help='checkpoint summary: repository rows, runs, manifests, migration entry').add_argument('query')
    sub.add_parser('restore', help='rebuild missing local run manifests from Git lists')
    args = parser.parse_args(argv)
    root = args.root.resolve()
    report_path = root / REPORT
    report = json.loads(report_path.read_bytes()) if report_path.is_file() else {}
    read = git_reader(root, args.from_history) if args.from_history else store.file_reader(root)
    try:
        if args.command == 'cp':
            return summary(root, read, args.query, report)
        if args.command == 'restore':
            return restore(root, args.from_history, report)
        query = args.query
        if args.command == 'path':
            query = relative_query(root, query)
            match = lambda row: path_matches(query, row.get('path'))
            run_filter = lambda run: run_related(query, run)
        elif args.command == 'sha':
            query = query.lower()
            if len(query) < 8:
                parser.error('sha prefix needs at least 8 hex digits')
            match = lambda row: str(row.get('sha256', '')).startswith(query)
            run_filter = None
        else:
            match = lambda row: row.get('id') == query
            run_filter = None
        pruned = lean.load_pruned(root)
        found = 0
        for record, row, stored in iter_rows(root, read, set(args.cp or []), run_filter,
                                             warn=lambda text: print(text, file=sys.stderr)):
            if isinstance(row, dict) and match(row):
                found += 1
                if args.limit and found > args.limit:
                    continue
                emit(record, row, stored, root, args.verify, args.require_local, pruned)
        print(json.dumps({'matches': found, 'printed': min(found, args.limit) if args.limit else found,
                          'source': args.from_history or 'working tree'}), file=sys.stderr)
        return 0 if found else 1
    except (OSError, ValueError, KeyError, TypeError) as error:
        parser.exit(1, f'lookup failed: {error}\n')


if __name__ == '__main__':
    raise SystemExit(main())
