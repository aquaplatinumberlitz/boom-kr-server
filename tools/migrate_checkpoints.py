#!/usr/bin/env python3
"""Re-runnable v1→v2 migration and full object comparison; never rewrite complete records."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess

from checkpoint_store import (ROOT, MANIFEST, V1, V2, encoded, object_digest,
                              records, file_reader, target, record_reference)


def migrate(source, root, baseline_commit, report_path):
    """source is an immutable baseline snapshot, not the destination index."""
    root = Path(root).resolve()
    source_raw = Path(source).read_bytes()
    baseline = json.loads(source_raw)
    if baseline.get('schema') != V1:
        raise ValueError('migration source must be the preserved v1 baseline')
    mappings = []
    migrated = {k: v for k, v in baseline.items() if k != 'checkpoints'}
    migrated.update(schema=V2, version=2, checkpoints=[])
    for record in baseline['checkpoints']:
        relative = f'docs/evidence/checkpoints/records/{record["id"]}.json'
        destination = target(root, relative)
        raw = encoded(record)
        destination.parent.mkdir(parents=True, exist_ok=True)
        # An existing record is compared as a whole object AND kept byte-for-byte.
        if destination.exists():
            actual = destination.read_bytes()
            if json.loads(actual) != record:
                raise ValueError(f'refuse to overwrite changed record: {relative}')
            raw = actual
        else:
            with destination.open('xb') as stream:
                stream.write(raw)
        reference = record_reference(record, relative, raw)
        migrated['checkpoints'].append(reference)
        mappings.append({**reference, 'object_sha256': object_digest(record),
                         'evidence_count': len(record['evidence'])})
    destination = root / MANIFEST
    if destination.exists():
        existing = json.loads(destination.read_bytes())
        if existing.get('schema') == V2:
            # Re-running after a new checkpoint must never reset the counter.
            if len(existing['checkpoints']) < len(migrated['checkpoints']):
                raise ValueError('existing v2 index lost migrated records')
            for original, current in zip(baseline['checkpoints'], records(existing, root)):
                if original != current:
                    raise ValueError('existing v2 record differs from baseline')
            for key, value in baseline.items():
                if key not in {'schema', 'checkpoints', 'last_id', 'completed_count',
                               'audits', 'last_periodic_audit', 'next_periodic_audit',
                               'last_manual_audit', 'task_states'} and existing.get(key) != value:
                    raise ValueError(f'existing v2 metadata mismatch: {key}')
            migrated = existing
        elif existing != baseline:
            raise ValueError('working v1 manifest differs from pinned source')
        else:
            destination.write_bytes(encoded(migrated))
    else:
        destination.parent.mkdir(parents=True, exist_ok=True)
        destination.write_bytes(encoded(migrated))
    after = list(records(migrated, root)[:len(baseline['checkpoints'])])
    if after != baseline['checkpoints']:
        raise ValueError('complete checkpoint object comparison failed')
    if migrated['audits'][:len(baseline['audits'])] != baseline['audits']:
        raise ValueError('audit references changed')
    audit_proof = []
    for audit in baseline['audits']:
        raw = target(root, audit['path']).read_bytes()
        if hashlib.sha256(raw).hexdigest() != audit['sha256']:
            raise ValueError('audit file hash mismatch')
        audit_proof.append(dict(audit, bytes=len(raw)))
    report = dict(schema='kre-checkpoint-migration-report-v1', baseline_commit=baseline_commit,
                  source_sha256=hashlib.sha256(source_raw).hexdigest(),
                  before=dict(bytes=len(source_raw), lines=len(source_raw.splitlines()),
                              checkpoint_count=len(baseline['checkpoints']),
                              evidence_rows=sum(len(c['evidence']) for c in baseline['checkpoints'])),
                  migrated_prefix=dict(counter=baseline['last_id'], completed_count=baseline['completed_count'],
                                       last_periodic_audit=baseline['last_periodic_audit'],
                                       next_periodic_audit=baseline['next_periodic_audit']),
                  record_mappings=mappings, audit_files=audit_proof,
                  comparison=dict(all_records_equal=True, all_inline_evidence_equal=True,
                                  audit_references_equal=True, audit_hashes_verified=True),
                  after=dict(index_bytes=(root / MANIFEST).stat().st_size,
                             record_files=len(mappings), record_bytes=sum(x['bytes'] for x in mappings)),
                  limits='Object-equivalent migration of the legacy prefix; no semantic/acceptance/gate decisions. '
                         'Old evidence lists retained unchanged, including duplicates; new checkpoints may use pinned '
                         'external evidence manifests. Historical Git not rewritten. Later appended checkpoints are not baseline data.')
    report_path = target(root, str(report_path))
    report_path.parent.mkdir(parents=True, exist_ok=True)
    raw = encoded(report)
    if report_path.exists():
        prior = json.loads(report_path.read_bytes())
        # Keep initial report immutable; current index may have additional records.
        for key in ('source_sha256', 'baseline_commit', 'record_mappings', 'audit_files', 'comparison', 'migrated_prefix'):
            if prior.get(key) != report[key]:
                raise ValueError(f'migration report mismatch: {key}')
    else:
        report_path.write_bytes(raw)
    return report


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--root', type=Path, default=ROOT)
    parser.add_argument('--source', type=Path, required=True, help='Pinned original v1 JSON; never print it')
    parser.add_argument('--baseline-commit', required=True)
    parser.add_argument('--report', default='docs/evidence/checkpoints/migration-v2.json')
    args = parser.parse_args()
    # Verify commit syntax/existence on a real repo; fixture API tests can call migrate directly.
    subprocess.run(['git', '-C', str(args.root), 'cat-file', '-e', args.baseline_commit + '^{commit}'], check=True)
    committed = subprocess.check_output(['git', '-C', str(args.root), 'show',
                                         args.baseline_commit + ':' + MANIFEST.as_posix()])
    if committed != args.source.read_bytes():
        raise ValueError('source bytes do not match the declared baseline Git manifest')
    report = migrate(args.source, args.root, args.baseline_commit, args.report)
    print(json.dumps({k: report[k] for k in ('baseline_commit', 'before', 'after', 'comparison')}, ensure_ascii=False))


if __name__ == '__main__':
    main()
