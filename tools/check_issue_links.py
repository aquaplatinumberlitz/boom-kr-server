"""Read-only GitHub/local link consistency. Never infer acceptance or close a task/gate."""
import argparse
from datetime import datetime, timezone
import json
from pathlib import Path
import re
import subprocess

from feature_catalog import ROOT, link_error

REPO = 'aquaplatinumberlitz/boom-kr-server'
SNAPSHOT = 'docs/evidence/issue-reconciliation/github-snapshot.json'
REVIEWS = 'docs/evidence/issue-reconciliation/review.json'


def capture():
    def gh(kind, fields):
        return json.loads(subprocess.check_output(
            ['gh', kind, 'list', '--repo', REPO, '--state', 'all', '--limit', '1000',
             '--json', fields], text=True))
    issues = gh('issue', 'number,title,state,url,body,updatedAt,closedAt')
    prs = gh('pr', 'number,title,state,url,mergedAt,headRefOid,mergeCommit')
    if len(issues) >= 1000 or len(prs) >= 1000:
        raise ValueError('snapshot cap reached; pagination required, no completeness claim')
    return {'schema': 'kre-github-snapshot-v1', 'repo': REPO,
            'captured_utc': datetime.now(timezone.utc).isoformat(),
            'limits': 'State as of capture only; body/declarations are not semantic acceptance; no GitHub mutation.',
            'issues': issues, 'pull_requests': prs}


def validate(snapshot, reviews, catalog, manifest, plan, root=ROOT):
    errors, warnings = [], []
    def check(ok, message):
        if not ok:
            errors.append(message)
    check(snapshot.get('schema') == 'kre-github-snapshot-v1' and snapshot.get('repo') == REPO,
          'snapshot schema/repo invalid')
    try:
        captured = datetime.fromisoformat(snapshot['captured_utc'])
        if captured.tzinfo is None:
            raise ValueError('timezone absent')
        age = (datetime.now(timezone.utc) - captured).total_seconds()
        check(age >= -300, 'snapshot timestamp is in the future')
        if age > 86400:
            warnings.append('Snapshot older than 24h; use --live to check current GitHub state.')
    except (ValueError, KeyError, TypeError):
        errors.append('invalid snapshot timestamp')
    issues = {q['number']: q for q in snapshot.get('issues', [])}
    prs = {q['number']: q for q in snapshot.get('pull_requests', [])}
    check(len(issues) == len(snapshot.get('issues', [])), 'duplicate issue number')
    features = {n['id']: n for n in catalog['nodes'] if n['kind'] == 'feature'}
    reviewed = {q['number']: q for q in reviews['issues']}
    check(len(reviewed) == len(reviews['issues']), 'duplicate issue review')
    for n, issue in issues.items():
        check(issue['state'] in {'OPEN', 'CLOSED'}, f'#{n}: invalid GitHub state')
        if issue['state'] == 'OPEN':
            check(n in reviewed, f'#{n}: open issue lacks a dated manual disposition')
    for n, review in reviewed.items():
        issue = issues.get(n)
        check(issue is not None, f'#{n}: issue does not exist in snapshot')
        if issue is None:
            continue
        classification = review.get('classification')
        check(classification in {'in_progress', 'backlog', 'blocked', 'completed', 'superseded'},
              f'#{n}: invalid classification')
        expected = 'CLOSED' if classification in {'completed', 'superseded'} else 'OPEN'
        check(issue['state'] == expected, f'#{n}: declared {classification} but GitHub {issue["state"]}')
        for key in ['kre', 'known', 'remaining', 'next_test', 'acceptance', 'reason']:
            check(bool(review.get(key)), f'#{n}: missing manual {key}')
        fids = review.get('features', [])
        check(bool(fids) and all(fid in features for fid in fids), f'#{n}: invalid/missing feature IDs')
        if issue['state'] == 'OPEN':
            for fid in fids:
                check(issue['url'] in features.get(fid, {}).get('issues', []), f'#{n}/{fid}: missing catalog backlink')
                check(fid in issue['body'], f'#{n}/{fid}: GitHub body lacks feature backlink')
            for path in ['docs/PLAN.md', 'docs/PROTOCOL.md', 'docs/RESEARCH.md']:
                check('/' + path in issue['body'], f'#{n}: GitHub body lacks {path} link')
        for target in review.get('sources', []) + review.get('evidence', []):
            check(link_error(target, root) is None, f'#{n}: missing canonical/evidence link {target}')
        for number in review.get('prs', []):
            check(number in prs, f'#{n}: missing PR{number}')
            if classification == 'completed':
                check(prs.get(number, {}).get('state') == 'MERGED', f'#{n}: completion cites unmerged PR{number}')
        if classification == 'superseded':
            check(bool(review.get('successors')), f'#{n}: superseded without successor')
        for successor in review.get('successors', []):
            check(successor in issues and successor != n, f'#{n}: missing/self successor #{successor}')
    for fid, node in features.items():
        check(bool(node.get('issues')), f'{fid}: missing issue navigation')
        for url in node.get('issues', []):
            match = re.fullmatch(r'https://github.com/' + re.escape(REPO) + r'/issues/(\d+)', url)
            check(bool(match) and int(match[1]) in issues, f'{fid}: nonexistent issue link {url}')
    # These are current declarations only. Completed checkpoint records stay historical/immutable.
    for task in manifest.get('task_states', []):
        match = re.search(r'/issues/(\d+)$', task.get('reference', ''))
        if not match:
            continue
        number = int(match[1])
        check(number in issues, f'{task["id"]}: unknown issue #{number}')
        if number in issues:
            check(task['state'].upper() == issues[number]['state'],
                  f'{task["id"]}/#{number}: current manifest declaration != GitHub state')
    for number in map(int, re.findall(r'<!--\s*active-issue:\s*(\d+)\s*-->', plan)):
        check(issues.get(number, {}).get('state') == 'OPEN', f'PLAN: active issue #{number} is closed/missing')
    for cp in manifest.get('checkpoints', [])[-1:]:
        for step in cp.get('next_steps', []):
            if step['state'] != 'active':
                continue
            task = next((t for t in manifest['task_states'] if t['id'] == step['task']), None)
            check(task is not None and task['state'] == 'open', f'{step["id"]}: active next-step assigned to closed task')
    # Check referenced GitHub URLs, not arbitrary external sites or old prose semantics.
    texts = [plan, json.dumps(catalog), json.dumps(reviews)]
    for kind, number in re.findall(r'https://github.com/' + re.escape(REPO) + r'/(issues|pull)/(\d+)', '\n'.join(texts)):
        check(int(number) in (issues if kind == 'issues' else prs), f'nonexistent {kind}/{number}')
    return sorted(set(errors)), warnings


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--snapshot', type=Path, default=ROOT / SNAPSHOT)
    parser.add_argument('--live', action='store_true', help='read current GitHub; never mutate GitHub')
    parser.add_argument('--capture', type=Path, help='save a dated GitHub snapshot locally, no GitHub write')
    args = parser.parse_args()
    snapshot = capture() if args.live or args.capture else json.loads(args.snapshot.read_text())
    if args.capture:
        args.capture.parent.mkdir(parents=True, exist_ok=True)
        with args.capture.open('x') as f:
            json.dump(snapshot, f, ensure_ascii=False, indent=2)
            f.write('\n')
        print(f'Captured {len(snapshot["issues"])} issues / {len(snapshot["pull_requests"])} PRs at {snapshot["captured_utc"]}')
        return
    errors, warnings = validate(snapshot, json.loads((ROOT / REVIEWS).read_text()),
                                json.loads((ROOT / 'docs/features.json').read_text()),
                                json.loads((ROOT / 'docs/evidence/checkpoints/manifest.json').read_text()),
                                (ROOT / 'docs/PLAN.md').read_text())
    for warning in warnings:
        print('WARNING:', warning)
    if errors:
        raise SystemExit('\n'.join(errors))
    print(f'Issue links/states OK as of {snapshot["captured_utc"]}; no acceptance/gate certified; historical checkpoints not rewritten.')


if __name__ == '__main__':
    main()
