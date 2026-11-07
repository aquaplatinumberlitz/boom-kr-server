"""Accept an explicit rules pre-read without relaxing the manual core-doc order."""
import copy
import unittest

from check_checkpoints import CORE_DOCS, audit_core_read_order


class AuditReadOrderTests(unittest.TestCase):
    def test_original_core_first_audit_remains_accepted(self):
        self.assertTrue(audit_core_read_order(list(CORE_DOCS) + ['docs/evidence/a.json']))

    def test_rules_then_core_pass_preserves_immutable_audit_list(self):
        rows = ['AGENTS.md', *CORE_DOCS, 'docs/evidence/a.json']
        before = copy.deepcopy(rows)
        self.assertTrue(audit_core_read_order(rows))
        self.assertEqual(rows, before)

    def test_rule_preread_cannot_hide_missing_or_swapped_core_docs(self):
        self.assertFalse(audit_core_read_order(['AGENTS.md', *CORE_DOCS[:3]]))
        rows = ['AGENTS.md', *CORE_DOCS]
        rows[2], rows[3] = rows[3], rows[2]
        self.assertFalse(audit_core_read_order(rows))

    def test_unrelated_or_repeated_prefix_and_malformed_lists_are_rejected(self):
        for rows in [None, 'README.md', ['AGENTS.md', 'AGENTS.md', *CORE_DOCS],
                     ['unrelated.md', *CORE_DOCS], ['README.md', 'AGENTS.md', *CORE_DOCS[1:]]]:
            with self.subTest(rows=rows):
                self.assertFalse(audit_core_read_order(rows))


if __name__ == '__main__':
    unittest.main()
