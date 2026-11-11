"""Consistency negative cases; these tests do not establish client behavior."""
import copy
import unittest

import feature_catalog as catalog


class CatalogTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.baseline = catalog.load()

    def data(self):
        return copy.deepcopy(self.baseline)

    def feature(self, data):
        return next(n for n in data['nodes'] if n['kind'] == 'feature')

    def assert_error(self, data, text, previous=None):
        self.assertTrue(any(text in e for e in catalog.validate(data, previous=previous)), text)

    def test_duplicate_and_removed_ids(self):
        data = self.data()
        data['nodes'].append(copy.deepcopy(data['nodes'][-1]))
        self.assert_error(data, 'duplicate ID')
        data = self.data()
        data['nodes'].pop()
        self.assert_error(data, 'stable ID removed', self.baseline)

    def test_repurposed_id(self):
        data = self.data()
        self.feature(data)['mechanism'] = 'unrelated-mechanism'
        self.assert_error(data, 'stable ID repurposed', self.baseline)

    def test_cycles_and_missing_dependency(self):
        data = self.data()
        self.feature(data)['depends_on'] = [self.feature(data)['id']]
        self.assert_error(data, 'cycle')
        self.feature(data)['depends_on'] = ['KR-F999']
        self.assert_error(data, 'missing/non-feature dependency')
        data = self.data()
        data['nodes'][0]['parent'] = data['nodes'][0]['id']
        self.assert_error(data, 'parent: cycle')

    def test_missing_file_and_anchor(self):
        data = self.data()
        self.feature(data)['refs']['contract'] = 'docs/PROTOCOL.md#invented-anchor'
        self.assert_error(data, 'missing anchor')
        self.feature(data)['refs']['contract'] = 'docs/nonexistent.md#x'
        self.assert_error(data, 'missing/outside file')

    def test_state_evidence_and_environment(self):
        data = self.data()
        f = self.feature(data)
        f['coverage']['presence'] = {'state': 'present', 'refs': []}
        self.assert_error(data, 'positive declaration without refs')
        f['coverage']['presence']['state'] = 'certain'
        self.assert_error(data, 'state invalid')
        f['coverage']['implementation']['delivery'] = 'via_pr'
        f['coverage']['implementation']['refs'] = ['server/probe_service.py']
        self.assert_error(data, 'via_pr requires PR source')
        f['coverage']['runtime'].pop('windows_original')
        self.assert_error(data, 'runtime environments')

    def test_no_auto_gate_closure_and_question_required(self):
        data = self.data()
        f = self.feature(data)
        f['coverage']['gate']['state'] = 'passed'
        f['coverage']['gate'].pop('acceptance', None)
        self.assert_error(data, 'passed gate needs explicit acceptance')
        f['refs']['question'] = 'docs/RESEARCH.md#q-023'
        self.assert_error(data, 'question locator mismatch')

    def test_baseline_and_reproducible_generation(self):
        self.assertEqual(catalog.validate(self.baseline), [])
        self.assertEqual(catalog.render(self.baseline), catalog.render(self.data()))


if __name__ == '__main__':
    unittest.main()
