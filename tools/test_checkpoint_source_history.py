"""Publication-bound source recovery tests; no RE/acceptance certification."""
import hashlib
import json
from pathlib import Path
import subprocess
import tempfile
import unittest

import check_checkpoints as checker
import checkpoint_store as store


class SourceHistoryTests(unittest.TestCase):
    def setUp(self):
        tmp = tempfile.TemporaryDirectory()
        self.addCleanup(tmp.cleanup)
        self.root = Path(tmp.name)
        self.git = ['git', '-C', str(self.root), '-c', 'user.name=test',
                    '-c', 'user.email=test@example.invalid']
        subprocess.run(self.git + ['init', '-q'], check=True)

    def commit(self):
        subprocess.run(self.git + ['add', '.'], check=True)
        subprocess.run(self.git + ['commit', '-qm', 'synthetic publication'], check=True)

    def fixture(self, relative='server/service.py', external=False, broken_reference=False):
        path = self.root / relative
        path.parent.mkdir(parents=True, exist_ok=True)
        raw = b'original fixture bytes\n'
        path.write_bytes(raw)
        item = dict(id='E1', kind='repository', path=relative, bytes=len(raw),
                    sha256=hashlib.sha256(raw).hexdigest())
        record = dict(id='CP001', lifecycle='complete', evidence=[item])
        if external:
            ep = 'docs/evidence/checkpoints/evidence/CP001.json'
            data = store.encoded(dict(schema=store.EVIDENCE_SCHEMA, evidence=[item]))
            p = self.root / ep
            p.parent.mkdir(parents=True, exist_ok=True)
            p.write_bytes(data)
            record.update(evidence=[], evidence_manifests=[dict(
                path=ep, bytes=len(data),
                sha256='f' * 64 if broken_reference else hashlib.sha256(data).hexdigest(), count=1)])
        record_path = self.root / 'docs/evidence/checkpoints/records/CP001.json'
        record_path.parent.mkdir(parents=True, exist_ok=True)
        record_path.write_text(json.dumps(record))
        self.commit()
        return item, path

    def verify(self, item):
        return checker.verify_evidence_item(item, self.root, True, 'CP001')

    def test_inline_pin_recovers_exact_source_at_record_publication(self):
        item, path = self.fixture()
        path.write_text('new working source\n')
        errors, warnings, checked, missing = self.verify(item)
        self.assertEqual((errors, checked, missing), ([], 1, 0))
        self.assertTrue(any('HISTORICAL SOURCE VERIFIED' in w for w in warnings))

    def test_c_tool_source_requires_exact_external_publication(self):
        item, path = self.fixture('tools/hook.c', external=True)
        later = b'later C source\n'
        path.write_bytes(later)
        self.commit()
        path.write_text('current C source\n')
        errors, warnings, checked, missing = self.verify(item)
        self.assertEqual((errors, checked, missing), ([], 1, 0))
        self.assertTrue(any('HISTORICAL SOURCE VERIFIED' in w for w in warnings))
        requested = item | dict(bytes=len(later), sha256=hashlib.sha256(later).hexdigest())
        self.assertTrue(self.verify(requested)[0])
        self.assertTrue(checker.verify_evidence_item(item, self.root, True, 'CP002')[0])

    def test_c_tool_header_needs_exact_manifest_and_binary_stays_strict(self):
        item, path = self.fixture('tools/hook.h', external=True, broken_reference=True)
        path.write_text('current header\n')
        self.assertTrue(self.verify(item)[0])
        for relative in ['local/hook.c', 'tools/../hook.c', 'rust/target/hook.c',
                         'tools/hook.dll', 'tools/hook.exe', 'tools/hook.o', 'server/hook.c']:
            self.assertIsNone(checker.historical_checkpoint_source(
                self.root, relative, item['sha256'], item['bytes'], 'CP001'))

    def test_c_tool_header_recovers_only_its_publication_blob(self):
        item, path = self.fixture('tools/hook.h', external=True)
        path.write_text('current header\n')
        errors, warnings, checked, missing = self.verify(item)
        self.assertEqual((errors, checked, missing), ([], 1, 0))
        self.assertTrue(any('HISTORICAL SOURCE VERIFIED' in w for w in warnings))

    def test_powershell_tool_source_requires_exact_publication(self):
        item, path = self.fixture('tools/native_windows_multilang.ps1', external=True)
        later = b'later PowerShell source\n'
        path.write_bytes(later)
        self.commit()
        path.write_text('current PowerShell source\n')
        errors, warnings, checked, missing = self.verify(item)
        self.assertEqual((errors, checked, missing), ([], 1, 0))
        self.assertTrue(any('HISTORICAL SOURCE VERIFIED' in w for w in warnings))
        later_pin = item | dict(bytes=len(later), sha256=hashlib.sha256(later).hexdigest())
        self.assertTrue(self.verify(later_pin)[0])
        self.assertTrue(self.verify(item | dict(sha256='f' * 64))[0])
        self.assertTrue(checker.verify_evidence_item(item, self.root, True, 'CP002')[0])
        self.assertTrue(self.verify(item | dict(kind='artifact'))[0])

    def test_powershell_tool_source_rejects_broken_manifest(self):
        item, path = self.fixture('tools/native_windows_multilang.ps1', external=True,
                                  broken_reference=True)
        path.write_text('current PowerShell source\n')
        self.assertTrue(self.verify(item)[0])

    def test_powershell_history_does_not_cover_local_data_or_binary(self):
        item, path = self.fixture('local/run.ps1')
        path.write_text('current private raw script\n')
        self.assertTrue(self.verify(item)[0])
        for relative in ['local/run.ps1', 'server/run.ps1', 'docs/evidence/run.ps1',
                         'tools/../run.ps1', 'tools/./run.ps1', 'tools/run.exe',
                         'tools/run.psm1', 'rust/target/run.ps1']:
            self.assertIsNone(checker.historical_checkpoint_source(
                self.root, relative, item['sha256'], item['bytes'], 'CP001'))

    def test_rust_source_requires_its_original_external_publication_and_exact_blob(self):
        item, path = self.fixture('rust/crates/kr-service/src/room.rs', external=True)
        later = b'later Rust source\n'
        path.write_bytes(later)
        self.commit()
        path.write_text('current Rust source\n')
        errors, warnings, checked, missing = self.verify(item)
        self.assertEqual((errors, checked, missing), ([], 1, 0))
        self.assertTrue(any('HISTORICAL SOURCE VERIFIED' in w for w in warnings))
        later_pin = item | dict(bytes=len(later), sha256=hashlib.sha256(later).hexdigest())
        self.assertTrue(self.verify(later_pin)[0])
        self.assertTrue(checker.verify_evidence_item(item, self.root, True, 'CP002')[0])
        self.assertTrue(self.verify(item | dict(kind='artifact'))[0])

    def test_rust_source_with_broken_publication_manifest_stays_invalid(self):
        item, path = self.fixture('rust/crates/kr-wire/src/kr00967/room.rs',
                                  external=True, broken_reference=True)
        path.write_text('new Rust source\n')
        self.assertTrue(self.verify(item)[0])

    def cargo_publication(self, relative):
        item, path = self.fixture(relative, external=True)
        later = b'later Cargo input\n'
        path.write_bytes(later)
        self.commit()
        path.write_text('current Cargo input\n')
        errors, warnings, checked, missing = self.verify(item)
        self.assertEqual((errors, checked, missing), ([], 1, 0))
        self.assertTrue(any('HISTORICAL SOURCE VERIFIED' in w for w in warnings))
        later_pin = item | dict(bytes=len(later), sha256=hashlib.sha256(later).hexdigest())
        self.assertTrue(self.verify(later_pin)[0])
        self.assertTrue(self.verify(item | {'sha256': 'f' * 64})[0])
        self.assertTrue(checker.verify_evidence_item(item, self.root, True, 'CP002')[0])
        self.assertTrue(self.verify(item | dict(kind='artifact'))[0])

    def test_cargo_workspace_manifest_requires_exact_publication(self):
        self.cargo_publication('rust/Cargo.toml')

    def test_cargo_workspace_lock_requires_exact_publication(self):
        self.cargo_publication('rust/Cargo.lock')

    def test_cargo_crate_manifest_requires_exact_publication(self):
        self.cargo_publication('rust/crates/kr-wire/Cargo.toml')

    def test_cargo_broken_publication_manifest_cannot_rescue_pin(self):
        item, path = self.fixture('rust/Cargo.lock', external=True, broken_reference=True)
        path.write_text('new Cargo input\n')
        self.assertTrue(self.verify(item)[0])

    def test_cargo_history_does_not_cover_other_toml_lock_local_or_output_files(self):
        item, path = self.fixture('rust/data/rules.toml')
        path.write_text('new rules fixture\n')
        self.assertTrue(self.verify(item)[0])
        for relative in ['Cargo.toml', 'local/Cargo.toml', 'rust/data/Cargo.toml',
                         'rust/target/Cargo.toml', 'rust/crates/kr-wire/Cargo.lock',
                         'rust/crates/kr-wire/assets/Cargo.toml',
                         'rust/crates/../Cargo.toml', 'rust/crates/./Cargo.toml',
                         'rust/crates/kr-wire/other.toml', 'rust/other.lock']:
            self.assertIsNone(checker.historical_checkpoint_source(
                self.root, relative, item['sha256'], item['bytes'], 'CP001'))

    def test_rust_integration_test_source_requires_exact_publication_not_fixture_or_later_blob(self):
        item, path = self.fixture('rust/crates/kr-wire/tests/native_start_vectors.rs', external=True)
        later = b'later integration test source\n'
        path.write_bytes(later)
        self.commit()
        path.write_text('current integration test source\n')
        errors, warnings, checked, missing = self.verify(item)
        self.assertEqual((errors, checked, missing), ([], 1, 0))
        self.assertTrue(any('HISTORICAL SOURCE VERIFIED' in w for w in warnings))
        later_pin = item | dict(bytes=len(later), sha256=hashlib.sha256(later).hexdigest())
        self.assertTrue(self.verify(later_pin)[0])
        self.assertTrue(checker.verify_evidence_item(item, self.root, True, 'CP002')[0])
        self.assertTrue(self.verify(item | dict(kind='artifact'))[0])
        for relative in ['rust/crates/kr-wire/tests/fixtures/vector.rs',
                         'rust/crates/kr-wire/tests/native.bin',
                         'rust/crates/kr-wire/tests/../fixtures/vector.rs']:
            self.assertIsNone(checker.historical_checkpoint_source(
                self.root, relative, item['sha256'], item['bytes'], 'CP001'))

    def test_rust_integration_test_broken_publication_manifest_cannot_be_rescued(self):
        item, path = self.fixture('rust/crates/kr-wire/tests/native_start_vectors.rs',
                                  external=True, broken_reference=True)
        path.write_text('new test source\n')
        self.assertTrue(self.verify(item)[0])

    def test_rust_fixture_and_build_tree_are_not_source_history(self):
        item, path = self.fixture('rust/target/room.rs')
        path.write_text('changed build artifact\n')
        self.assertTrue(self.verify(item)[0])
        self.assertIsNone(checker.historical_checkpoint_source(
            self.root, 'rust/crates/kr-wire/fixtures/room.rs', item['sha256'], item['bytes'], 'CP001'))
        self.assertIsNone(checker.historical_checkpoint_source(
            self.root, 'rust/crates/kr-wire/src/../fixtures/room.rs', item['sha256'], item['bytes'], 'CP001'))

    def test_docs_workflow_requires_exact_original_external_publication(self):
        item, path = self.fixture('.github/workflows/docs.yml', external=True)
        later = b'later workflow source\n'
        path.write_bytes(later)
        self.commit()
        path.write_text('current workflow source\n')
        errors, warnings, checked, missing = self.verify(item)
        self.assertEqual((errors, checked, missing), ([], 1, 0))
        self.assertTrue(any('HISTORICAL SOURCE VERIFIED' in w for w in warnings))
        later_pin = item | dict(bytes=len(later), sha256=hashlib.sha256(later).hexdigest())
        self.assertTrue(self.verify(later_pin)[0])
        self.assertTrue(checker.verify_evidence_item(item, self.root, True, 'CP002')[0])
        self.assertTrue(self.verify(item | dict(sha256='f' * 64))[0])
        absolute_artifact = item | dict(kind='artifact', path=str(path))
        self.assertTrue(self.verify(absolute_artifact)[0])

    def test_docs_workflow_broken_external_manifest_cannot_be_rescued(self):
        item, path = self.fixture('.github/workflows/docs.yml', external=True,
                                  broken_reference=True)
        path.write_text('changed workflow\n')
        self.assertTrue(self.verify(item)[0])

    def test_undeclared_docs_workflow_cannot_borrow_another_publication(self):
        item, _ = self.fixture('server/service.py')
        path = self.root / '.github/workflows/docs.yml'
        path.parent.mkdir(parents=True)
        path.write_bytes(b'original fixture bytes\n')
        self.commit()
        path.write_text('changed workflow\n')
        self.assertTrue(self.verify(item | dict(path='.github/workflows/docs.yml'))[0])

    def test_other_yaml_paths_do_not_gain_workflow_source_recovery(self):
        for relative in ['.github/workflows/rust.yml', '.github/workflows/docs.yaml',
                         '.github/workflows/../workflows/docs.yml', 'tools/docs.yml',
                         'local/docs.yml']:
            with self.subTest(relative=relative):
                self.assertIsNone(checker.historical_checkpoint_source(
                    self.root, relative, 'f' * 64, 0, 'CP001'))

    def test_generated_census_requires_exact_publication_not_later_matching_json(self):
        item, path = self.fixture('docs/content/census.json', external=True)
        later = b'{"later": true}\n'
        path.write_bytes(later)
        self.commit()
        path.write_text('{"current": true}\n')
        self.assertEqual(self.verify(item)[0], [])
        later_pin = item | dict(bytes=len(later), sha256=hashlib.sha256(later).hexdigest())
        self.assertTrue(self.verify(later_pin)[0])
        self.assertTrue(self.verify(item | {'sha256': 'f' * 64})[0])

    def test_external_pin_requires_original_evidence_manifest_hash(self):
        item, path = self.fixture('tools/tool.py', external=True)
        path.write_text('new source\n')
        self.assertEqual(self.verify(item)[0], [])
        # Verify publication bytes, not an edited working evidence list.
        ep = self.root / 'docs/evidence/checkpoints/evidence/CP001.json'
        ep.write_text('{}')
        self.assertEqual(self.verify(item)[0], [])

    def test_later_matching_source_does_not_repair_a_different_publication_pin(self):
        item, path = self.fixture()
        later = b'later source\n'
        path.write_bytes(later)
        self.commit()
        requested = item | dict(bytes=len(later), sha256=hashlib.sha256(later).hexdigest())
        path.write_text('third version\n')
        self.assertTrue(self.verify(requested)[0])
        self.assertTrue(self.verify(item | {'sha256': 'f' * 64})[0])

    def test_matching_old_source_without_publication_blob_is_not_enough(self):
        item, path = self.fixture()
        path.write_text('new working source\n')
        # A different checkpoint cannot claim CP001's birth/source as its own.
        errors = checker.verify_evidence_item(item, self.root, True, 'CP002')[0]
        self.assertTrue(errors)

    def test_bad_publication_evidence_manifest_hash_cannot_rescue_source(self):
        item, path = self.fixture(external=True, broken_reference=True)
        path.write_text('new working source\n')
        self.assertTrue(self.verify(item)[0])

    def test_binary_local_or_unpublished_source_never_uses_fallback(self):
        item, path = self.fixture('server/client.bin')
        path.write_text('changed binary fixture\n')
        self.assertTrue(self.verify(item)[0])
        self.assertIsNone(checker.historical_checkpoint_source(
            self.root, 'local/source.py', item['sha256'], item['bytes'], 'CP001'))
        source = self.root / 'server/not-published.py'
        source.write_text('current\n')
        self.assertTrue(self.verify(item | {'path': 'server/not-published.py'})[0])

    def test_catalog_metadata_requires_exact_publication_not_later_matches(self):
        item, path = self.fixture('docs/features.json')
        later = b'{"later": true}\n'
        path.write_bytes(later)
        self.commit()
        path.write_text('{"current": true}\n')
        errors, warnings, checked, missing = self.verify(item)
        self.assertEqual((errors, checked, missing), ([], 1, 0))
        self.assertTrue(any('HISTORICAL METADATA DOC VERIFIED' in w for w in warnings))
        requested = item | dict(bytes=len(later), sha256=hashlib.sha256(later).hexdigest())
        self.assertTrue(self.verify(requested)[0])
        self.assertTrue(checker.verify_evidence_item(item, self.root, True, 'CP002')[0])

    def test_record_created_in_merge_retains_exact_publication_boundary(self):
        (self.root / 'base.txt').write_text('base\n')
        self.commit()
        base_branch = subprocess.check_output(
            self.git + ['branch', '--show-current'], text=True).strip()
        subprocess.run(self.git + ['checkout', '-qb', 'side'], check=True)
        (self.root / 'side.txt').write_text('side\n')
        self.commit()
        subprocess.run(self.git + ['checkout', '-q', base_branch], check=True)
        (self.root / 'main.txt').write_text('main\n')
        self.commit()
        subprocess.run(self.git + ['merge', '--no-ff', '--no-commit', 'side'],
                       check=True, capture_output=True)
        # The fixture is first published by the merge, not by either parent.
        item, path = self.fixture('docs/features.json', external=True)
        publication = subprocess.check_output(self.git + ['rev-parse', 'HEAD'], text=True).strip()
        self.assertEqual(len(subprocess.check_output(
            self.git + ['show', '-s', '--format=%P'], text=True).split()), 2)
        later = b'{"later": true}\n'
        path.write_bytes(later)
        self.commit()
        path.write_text('{"current": true}\n')
        errors, warnings, checked, missing = self.verify(item)
        self.assertEqual((errors, checked, missing), ([], 1, 0))
        self.assertTrue(any(publication in w for w in warnings))
        later_pin = item | dict(bytes=len(later), sha256=hashlib.sha256(later).hexdigest())
        self.assertTrue(self.verify(later_pin)[0])

    def test_renamed_record_is_published_at_its_new_path(self):
        item, path = self.fixture('docs/features.json', external=True)
        old = self.root / 'docs/evidence/checkpoints/records/CP001.json'
        new = old.with_name('CP002.json')
        record = json.loads(old.read_text())
        record['id'] = 'CP002'
        old.unlink()
        new.write_text(json.dumps(record))
        self.commit()
        path.write_text('{"current": true}\n')
        errors, warnings, checked, missing = checker.verify_evidence_item(
            item, self.root, True, 'CP002')
        self.assertEqual((errors, checked, missing), ([], 1, 0))
        self.assertTrue(any('HISTORICAL METADATA DOC VERIFIED' in w for w in warnings))

    def test_discarded_side_branch_checkpoint_does_not_replace_selected_publication(self):
        (self.root / 'base.txt').write_text('base\n')
        self.commit()
        branch = subprocess.check_output(self.git + ['branch', '--show-current'], text=True).strip()
        subprocess.run(self.git + ['checkout', '-qb', 'discarded'], check=True)
        side_item, _ = self.fixture('docs/features.json', external=True)
        subprocess.run(self.git + ['checkout', '-q', branch], check=True)
        path = self.root / 'docs/features.json'
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text('selected publication\n')
        selected = side_item | dict(bytes=path.stat().st_size,
                                    sha256=hashlib.sha256(path.read_bytes()).hexdigest())
        ep = self.root / 'docs/evidence/checkpoints/evidence/CP001.json'
        ep.parent.mkdir(parents=True, exist_ok=True)
        data = store.encoded(dict(schema=store.EVIDENCE_SCHEMA, evidence=[selected]))
        ep.write_bytes(data)
        rp = self.root / 'docs/evidence/checkpoints/records/CP001.json'
        rp.parent.mkdir(parents=True, exist_ok=True)
        rp.write_text(json.dumps(dict(id='CP001', lifecycle='complete', evidence=[],
            evidence_manifests=[dict(path=str(ep.relative_to(self.root)),bytes=len(data),
                                    sha256=hashlib.sha256(data).hexdigest(),count=1)])))
        self.commit()
        # The other branch remains reachable, but its competing ID is discarded.
        subprocess.run(self.git + ['merge', '-s', 'ours', '--no-ff', '-m', 'keep selected',
                                   'discarded'],check=True,capture_output=True)
        path.write_text('current metadata\n')
        self.assertEqual(self.verify(selected)[0], [])
        self.assertTrue(self.verify(side_item)[0])

    def test_issue_snapshot_metadata_keeps_external_manifest_authentication(self):
        item, path = self.fixture(
            'docs/evidence/issue-reconciliation/github-snapshot.json', external=True)
        path.write_text('{"current": true}\n')
        self.assertEqual(self.verify(item)[0], [])
        self.assertTrue(self.verify(item | {'sha256': 'f' * 64})[0])

    def test_issue_review_metadata_can_evolve_after_publication(self):
        item, path = self.fixture('docs/evidence/issue-reconciliation/review.json')
        path.write_text('{"current": true}\n')
        self.assertEqual(self.verify(item)[0], [])

    def test_other_json_evidence_cannot_use_metadata_history_fallback(self):
        item, path = self.fixture('docs/evidence/rust-service/runtime.json')
        path.write_text('{"changed": true}\n')
        self.assertTrue(self.verify(item)[0])


if __name__ == '__main__':
    unittest.main()
