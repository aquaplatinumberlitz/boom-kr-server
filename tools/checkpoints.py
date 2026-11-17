#!/usr/bin/env python3
"""Read selected checkpoint metadata/evidence. PLAN remains the only work order."""
import argparse
from collections import Counter
import json
from pathlib import Path

import checkpoint_evidence as lean
import checkpoint_store as store


def selected(manifest, ident, root):
    for index, entry in enumerate(manifest['checkpoints']):
        if entry.get('id') == ident:
            return store.records(manifest, root)[index]
    raise ValueError(f'unknown checkpoint: {ident}')


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--root', type=Path, default=store.ROOT)
    sub = parser.add_subparsers(dest='command', required=True)
    sub.add_parser('latest', help='small index/counter/PLAN, never all records')
    show = sub.add_parser('show', help='selected record without the raw index by default')
    show.add_argument('id')
    show.add_argument('--full', action='store_true', help='explicit full selected JSON including legacy inline evidence')
    evidence = sub.add_parser('evidence', help='summary, paginated rows or verify all selected evidence')
    evidence.add_argument('id')
    evidence.add_argument('--offset', type=int, default=0)
    evidence.add_argument('--limit', type=int, help='explicit page size 1..200; default summary only')
    evidence.add_argument('--verify', action='store_true', help='verify every selected evidence file, not only the page')
    evidence.add_argument('--require-local', action='store_true')
    args = parser.parse_args(argv)
    if args.command == 'evidence' and (args.offset < 0 or args.limit is not None and not 1 <= args.limit <= 200):
        parser.error('offset must be >=0; limit must be 1..200')
    try:
        manifest = store.load(args.root)
        if args.command == 'latest':
            result = {k: manifest[k] for k in ['schema', 'last_id', 'completed_count',
                                               'last_periodic_audit', 'last_manual_audit', 'next_periodic_audit']}
            result['audit_policy'] = manifest.get('audit_policy', {'mode': 'legacy_periodic'})
            last = manifest['checkpoints'][-1]
            result['record'] = last.get('path') if manifest['schema'] == store.V2 else 'inline legacy; use show ' + last['id']
            result['plan'] = 'docs/PLAN.md#current-order'
        else:
            record = selected(manifest, args.id, args.root)
            if args.command == 'show':
                if args.full:
                    result = record
                else:
                    result = {k: v for k, v in record.items() if k != 'evidence'}
                    result.pop('evidence_runs', None)
                    result['evidence_summary'] = {
                        'inline_rows': len(record.get('evidence', [])),
                        'external_rows_declared': sum(q['count'] for q in record.get('evidence_manifests', [])),
                        'run_rows_declared': sum(q['count'] for q in record.get('evidence_runs', [])),
                        'runs': len(record.get('evidence_runs', [])),
                        'command': f'python3 tools/checkpoints.py evidence {args.id}'}
            else:
                counts, page, total, errors, warnings, verified, unavailable = Counter(), [], 0, [], [], 0, 0
                if args.verify:
                    from check_checkpoints import verify_evidence_item
                    pruned = lean.load_pruned(args.root)

                def rows():
                    nonlocal unavailable
                    yield from store.evidence(record, args.root)
                    for pointer in lean.pointers(record):
                        try:
                            yield from lean.read_run(pointer, args.root)
                        except FileNotFoundError:
                            unavailable += pointer['count']
                            message = (f"{args.id}: UNVERIFIED unavailable host evidence run manifest "
                                       f"{pointer['path']} ({pointer['count']} files under {pointer['run']})")
                            (errors if args.require_local else warnings).append(message)
                        except ValueError as error:
                            errors.append(f"{args.id}: evidence run manifest {pointer['path']}: {error}")
                for i, item in enumerate(rows()):
                    total += 1
                    counts[item.get('kind', 'invalid')] += 1
                    if args.limit and args.offset <= i < args.offset + args.limit:
                        page.append(item)
                    if args.verify:
                        e, w, good, missing = verify_evidence_item(item, args.root, args.require_local, args.id, pruned)
                        errors.extend(e)
                        warnings.extend(w)
                        verified += good
                        unavailable += missing
                result = dict(id=args.id, total=total, kinds=dict(counts),
                              evidence_manifests=record.get('evidence_manifests', []),
                              evidence_runs=[f"{q['run']} ({q['count']} files)"
                                             for q in record.get('evidence_runs', [])])
                if record.get('evidence_runs') and not args.verify and (errors or warnings):
                    result['missing'] = errors + warnings
                if args.limit:
                    result.update(offset=args.offset, rows=page,
                                  next_offset=args.offset + len(page) if args.offset + len(page) < total else None)
                if args.verify:
                    result.update(verification=dict(ok=not errors, verified=verified, unavailable=unavailable,
                                                    pruned=pruned.hits, errors=errors, warnings=warnings))
                    if errors:
                        print(json.dumps(result, ensure_ascii=False, indent=2))
                        return 1
        print(json.dumps(result, ensure_ascii=False, indent=2))
        return 0
    except (OSError, ValueError, TypeError, KeyError) as error:
        parser.exit(1, f'checkpoint read failed: {error}\n')


if __name__ == '__main__':
    raise SystemExit(main())
