"""Measured consistency failures only; no acceptance or game semantics in the checker."""
import copy
from datetime import datetime, timedelta, timezone
import unittest
from check_issue_links import validate, REPO


class IssueLinksTests(unittest.TestCase):
    def setUp(self):
        self.url = 'https://github.com/' + REPO + '/issues/1'
        self.snapshot = {'schema': 'kre-github-snapshot-v1', 'repo': REPO,
                         'captured_utc': datetime.now(timezone.utc).isoformat(),
                         'issues': [{'number': 1, 'state': 'OPEN', 'url': self.url,
                                     'body': 'KR-B001 /docs/PLAN.md /docs/PROTOCOL.md /docs/RESEARCH.md'}],
                         'pull_requests': [{'number': 2, 'state': 'MERGED'}]}
        self.review = {'issues': [{'number': 1, 'kre': 'KRE-001', 'classification': 'backlog',
                                  'features': ['KR-B001'], 'known': 'scope', 'remaining': 'gap',
                                  'next_test': 'bounded test', 'acceptance': 'manual', 'reason': 'partial',
                                  'sources': [], 'evidence': [], 'prs': [], 'successors': []}]}
        self.catalog = {'nodes': [{'id': 'KR-B001', 'kind': 'feature', 'issues': [self.url]}]}
        self.manifest = {'task_states': [{'id': 'KRE-001', 'state': 'open', 'reference': self.url}],
                         'checkpoints': [{'next_steps': [{'id': 'NEXT', 'state': 'active', 'task': 'KRE-001'}]}]}
        self.plan = '<!-- active-issue: 1 -->'

    def check(self):
        return validate(self.snapshot, self.review, self.catalog, self.manifest, self.plan)

    def error(self, text):
        self.assertTrue(any(text in q for q in self.check()[0]), self.check()[0])

    def test_valid_declarations_do_not_certify_semantics(self):
        self.assertEqual(self.check()[0], [])
        self.assertEqual(self.review['issues'][0]['classification'], 'backlog')
        self.assertEqual(self.manifest['task_states'][0]['state'], 'open')

    def test_state_mismatch_and_closed_active_assignment(self):
        self.snapshot['issues'][0]['state'] = 'CLOSED'
        self.error('declared backlog')
        self.error('current manifest declaration')
        self.error('PLAN: active issue')
        self.manifest['task_states'][0]['state'] = 'closed'
        self.error('active next-step assigned to closed task')

    def test_missing_issue_pr_or_file_is_reported(self):
        self.catalog['nodes'][0]['issues'].append(self.url.replace('/1', '/999'))
        self.error('nonexistent issue link')
        self.review['issues'][0]['prs'] = [999]
        self.error('missing PR999')
        self.review['issues'][0]['evidence'] = ['docs/does-not-exist.md']
        self.error('missing canonical/evidence link')

    def test_open_issue_must_be_reviewed_and_have_backlinks(self):
        self.snapshot['issues'][0]['body'] = ''
        self.error('body lacks feature backlink')
        self.error('body lacks docs/PROTOCOL.md')
        self.catalog['nodes'][0]['issues'] = []
        self.error('missing catalog backlink')
        self.review['issues'] = []
        self.error('open issue lacks a dated manual disposition')

    def test_supersession_requires_real_successor_and_completion_merged_pr(self):
        self.review['issues'][0]['classification'] = 'superseded'
        self.error('superseded without successor')
        self.review['issues'][0]['successors'] = [999]
        self.error('missing/self successor')
        self.review['issues'][0]['classification'] = 'completed'
        self.review['issues'][0]['prs'] = [2]
        self.snapshot['pull_requests'][0]['state'] = 'OPEN'
        self.error('completion cites unmerged PR')

    def test_old_snapshot_warns_without_mutation_and_bad_date_fails(self):
        self.snapshot['captured_utc'] = (datetime.now(timezone.utc) - timedelta(days=2)).isoformat()
        before = copy.deepcopy(self.snapshot)
        self.assertTrue(self.check()[1])
        self.assertEqual(self.snapshot, before)
        self.snapshot['captured_utc'] = 'unknown'
        self.error('invalid snapshot timestamp')


if __name__ == '__main__':
    unittest.main()
