"""Navigation/size regressions; no client or contract acceptance claims."""
import tempfile
import unittest
from pathlib import Path

from check_docs import link_errors, markdown_anchors, size_errors


class DocsTests(unittest.TestCase):
    def test_current_doc_budget_has_inclusive_boundary_and_relocation_hint(self):
        self.assertEqual(size_errors('docs/PLAN.md', 'line\n' * 800), [])
        errors = size_errors('docs/PLAN.md', 'line\n' * 801)
        self.assertEqual(len(errors), 1)
        self.assertIn('docs/HISTORY.md', errors[0])
        # Historical content has no current-plan cap.
        self.assertEqual(size_errors('docs/HISTORY.md', 'line\n' * 801), [])

    def test_compatibility_stubs_and_real_heading_fragments_are_checked(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            index = root / 'PROTOCOL.md'
            topic = root / 'topic.md'
            index.write_text('<a id="old"></a> [→](topic.md#packet-153)\n')
            topic.write_text('# Packet 153\n\n[old](PROTOCOL.md#old)\n[self](#packet-153)\n')
            self.assertEqual(link_errors(index, root, {})[0], [])
            self.assertEqual(link_errors(topic, root, {})[0], [])
            topic.write_text('# Packet 153\n[bad](PROTOCOL.md#missing)\n')
            self.assertIn('invalid local fragment', link_errors(topic, root, {})[0][0])
            index.write_text('[escape](../outside.md)\n')
            self.assertIn('invalid local link', link_errors(index, root, {})[0][0])

    def test_fenced_examples_are_not_anchors_and_duplicate_heading_suffixes_work(self):
        anchors = markdown_anchors('# Ready\n# Ready\n```markdown\n# Fake\n```\n')
        self.assertEqual(anchors, {'ready', 'ready-1'})


if __name__ == '__main__':
    unittest.main()
