"""Consistency tests use synthetic documentation fixtures, not protocol evidence."""
import copy
import hashlib
import importlib.util
import json
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest import mock

spec = importlib.util.spec_from_file_location("checkpoint_checker", Path(__file__).with_name("check_checkpoints.py"))
checker = importlib.util.module_from_spec(spec)
spec.loader.exec_module(checker)


class CheckpointsTest(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        for name in (*checker.CORE_DOCS, "docs/evidence/README.md", "local/sample.txt"):
            path = self.root / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text("synthetic schema test\n")
        data = (self.root / "local/sample.txt").read_bytes()
        self.record = {
            "id": "CP001", "previous": "origin", "lifecycle": "complete",
            "recorded_utc": "2026-10-27T00:00:00Z", "question": "Check schema behavior",
            "scope": "No RE claims", "context_commit": "a" * 40,
            "artifacts": [{"name": "fixture", "sha256": hashlib.sha256(data).hexdigest()}],
            "locators": ["fixture only"], "owner": "author", "reviewer": "independent reviewer",
            "evidence": [{"id": "E1", "kind": "local", "path": "local/sample.txt",
                          "sha256": hashlib.sha256(data).hexdigest(), "bytes": len(data)}],
            "findings": [{"summary": "fixture", "source": "design", "status": "design",
                          "method": "documentation", "limits": "schema only",
                          "canonical": "docs/PROTOCOL.md", "evidence": ["E1"]}],
            "verification": [{"check": "fixture", "result": "pass", "evidence": ["E1"], "limits": "schema only"}],
            "delta": {"added": ["fixture"], "corrected": [], "superseded": []},
            "blockers": [], "next_steps": [{"id": "N1", "task": "T1", "state": "active",
                                          "action": "future work", "owner": "author", "reviewer": "reviewer"}],
            "review": {"status": "pending", "as_of_utc": "2026-10-27T00:00:00Z",
                       "reference": "test PR", "scope": "schema only"},
            "docs_sync": [{"path": p, "state": "updated", "reason": "fixture"} for p in (*checker.CORE_DOCS, "docs/evidence/README.md")],
        }
        self.manifest = {
            "schema": "kre-checkpoints-v1", "last_id": "CP001", "completed_count": 1,
            "audit_interval": 4, "last_periodic_audit": 0, "next_periodic_audit": "CP004",
            "last_manual_audit": None, "checkpoints": [self.record], "audits": [],
            "task_states": [{"id": "T1", "state": "open", "canonical": "docs/PLAN.md", "reference": "test issue"}],
        }

    def errors(self, manifest=None, require_local=True):
        return checker.validate(manifest or self.manifest, self.root, require_local)[0]

    def append(self):
        n = len(self.manifest["checkpoints"]) + 1
        record = copy.deepcopy(self.record)
        record.update(id=checker.cp_id(n), previous=checker.cp_id(n - 1))
        self.manifest["checkpoints"].append(record)
        self.manifest.update(last_id=record["id"], completed_count=n)

    def audit(self, trigger, through):
        audit_id = f'AUD{len(self.manifest["audits"]) + 1:03d}'
        data = {"id": audit_id, "trigger": trigger, "completed_through": through,
                "auditor": "manual reviewer", "read_files": list(checker.CORE_DOCS) + ["docs/evidence/README.md"],
                "findings": [], "remaining": [], "recorded_utc": "2026-10-27T00:00:00Z"}
        path = self.root / f"docs/evidence/{audit_id}.json"
        path.write_text(json.dumps(data))
        self.manifest["audits"].append({"id": audit_id, "path": str(path.relative_to(self.root)),
                                       "sha256": checker.digest(path)})
        self.manifest["last_manual_audit"] = audit_id

    def test_valid_pending_is_explicit_warning(self):
        errors, warnings, stats = checker.validate(self.manifest, self.root, True)
        self.assertEqual(errors, [])
        self.assertTrue(any("REVIEW pending" in w for w in warnings))
        self.assertEqual(stats["evidence_verified"], 1)

    def test_id_counter_gap_and_missing_fields(self):
        self.record["id"] = "CP002"
        del self.record["question"]
        errors = self.errors()
        self.assertTrue(any("noncontiguous" in e for e in errors))
        self.assertTrue(any("missing question" in e for e in errors))
        self.manifest["completed_count"] = 99
        self.assertTrue(any("completed_count" in e for e in self.errors()))

    def test_complete_cannot_have_unsynced_or_missing_doc(self):
        self.record["docs_sync"][0]["state"] = "blocked"
        self.record["docs_sync"][1]["path"] = "docs/missing.md"
        errors = self.errors()
        self.assertTrue(any("DOCS UNSYNCED" in e for e in errors))
        self.assertTrue(any("missing linked file" in e for e in errors))
        self.record["lifecycle"] = "in_progress"
        self.manifest["completed_count"] = 0
        errors, warnings, _ = checker.validate(self.manifest, self.root, True)
        self.assertTrue(any("INCOMPLETE" in w for w in warnings))

    def test_hash_corruption_rejected(self):
        (self.root / "local/sample.txt").write_text("corrupted")
        self.assertTrue(any("size/hash mismatch" in e for e in self.errors()))

    def test_unavailable_local_never_claimed_verified(self):
        (self.root / "local/sample.txt").unlink()
        errors, warnings, stats = checker.validate(self.manifest, self.root, False)
        self.assertEqual(errors, [])
        self.assertTrue(any("UNVERIFIED" in w for w in warnings))
        self.assertEqual(stats["evidence_verified"], 0)
        self.assertEqual(stats["evidence_unavailable"], 1)
        self.assertTrue(any("UNVERIFIED" in e for e in self.errors()))

    def test_path_escape_and_same_author_review_rejected(self):
        self.record["evidence"][0]["path"] = "../outside.txt"
        self.record["reviewer"] = self.record["owner"]
        errors = self.errors()
        self.assertTrue(any("outside repository" in e for e in errors))
        self.assertTrue(any("reviewer must differ" in e for e in errors))

    def test_closed_task_needs_next_step_disposition(self):
        self.manifest["task_states"][0]["state"] = "closed"
        self.assertTrue(any("closed task" in e for e in self.errors()))
        self.append()
        self.manifest["checkpoints"][-1]["next_steps"][0]["state"] = "closed"
        self.assertEqual(self.errors(), [])

    def test_fourth_checkpoint_requires_periodic_not_ad_hoc_audit(self):
        for _ in range(3):
            self.append()
        self.audit("serious_contradiction", "CP004")
        self.assertTrue(any("MANUAL AUDIT DUE" in e for e in self.errors()))
        self.audit("periodic", "CP004")
        self.manifest.update(last_periodic_audit=4, next_periodic_audit="CP008")
        self.assertEqual(self.errors(), [])
        path = self.root / self.manifest["audits"][-1]["path"]
        path.write_text(path.read_text() + " ")
        self.assertTrue(any("audit hash mismatch" in e for e in self.errors()))

    def test_complete_history_immutable_but_review_can_be_updated(self):
        previous = copy.deepcopy(self.manifest)
        self.record["review"].update(status="approved", head="b" * 40)
        self.assertEqual(checker.continuity(previous, self.manifest), [])
        self.record["findings"][0]["summary"] = "rewrite old conclusion"
        self.assertTrue(any("completed record changed" in e for e in checker.continuity(previous, self.manifest)))

    def test_approved_review_cannot_regress_or_change_head(self):
        self.record["review"].update(status="approved", head="b" * 40)
        previous = copy.deepcopy(self.manifest)
        self.record["review"]["status"] = "pending"
        self.assertTrue(any("approved review is immutable" in e for e in checker.continuity(previous, self.manifest)))
        self.record["review"].update(status="approved", head="c" * 40)
        self.assertTrue(any("approved review is immutable" in e for e in checker.continuity(previous, self.manifest)))

    def test_uncommitted_manifest_history_is_explicit(self):
        git = ["git", "-C", str(self.root), "-c", "user.name=Test", "-c", "user.email=test@example.invalid"]
        subprocess.run(git + ["init", "-q"], check=True)
        (self.root / "initial.txt").write_text("initial")
        subprocess.run(git + ["add", "initial.txt"], check=True)
        subprocess.run(git + ["commit", "-qm", "initial"], check=True)
        availability = []
        self.assertEqual(checker.check_history(self.root, checker.MANIFEST, self.manifest, availability), [])
        self.assertTrue(any("no committed manifest history" in e for e in availability))

    def test_missing_git_is_a_clear_availability_error(self):
        with mock.patch.object(checker.subprocess, "run", side_effect=FileNotFoundError("git")):
            with self.assertRaisesRegex(ValueError, "Git unavailable"):
                checker.check_history(self.root, checker.MANIFEST, self.manifest)

    def test_shallow_history_does_not_hide_known_counter_reset(self):
        git = ["git", "-C", str(self.root), "-c", "user.name=Test", "-c", "user.email=test@example.invalid"]
        subprocess.run(git + ["init", "-q"], check=True)
        target = self.root / checker.MANIFEST
        target.parent.mkdir(parents=True)
        for _ in range(2):
            target.write_text(json.dumps(self.manifest))
            subprocess.run(git + ["add", str(checker.MANIFEST)], check=True)
            subprocess.run(git + ["commit", "-qm", "fixture"], check=True)
            self.append()
        clone = self.root / "shallow"
        subprocess.run(["git", "clone", "-q", "--depth", "1", self.root.as_uri(), str(clone)], check=True)
        availability = []
        # Working copy deletes CP002; even with incomplete Git history this is a known violation.
        reduced = copy.deepcopy(self.manifest)
        reduced["checkpoints"] = reduced["checkpoints"][:1]
        errors = checker.check_history(clone, checker.MANIFEST, reduced, availability)
        self.assertTrue(any("shallow Git history" in e for e in availability))
        self.assertTrue(any("counter reset" in e for e in errors))

    def test_eighth_checkpoint_needs_its_own_periodic_audit(self):
        for _ in range(3):
            self.append()
        self.audit("periodic", "CP004")
        self.manifest.update(last_periodic_audit=4, next_periodic_audit="CP008")
        for _ in range(4):
            self.append()
        self.assertTrue(any("CP008: MANUAL AUDIT DUE" in e for e in self.errors()))

    def test_committed_counter_reset_detected_from_git_history(self):
        git = ["git", "-C", str(self.root), "-c", "user.name=Test", "-c", "user.email=test@example.invalid"]
        subprocess.run(git + ["init", "-q"], check=True)
        target = self.root / checker.MANIFEST
        target.parent.mkdir(parents=True)
        original = copy.deepcopy(self.manifest)
        for data in (original, None, original):
            if data is None:
                self.append()
                data = self.manifest
            target.write_text(json.dumps(data))
            subprocess.run(git + ["add", str(checker.MANIFEST)], check=True)
            subprocess.run(git + ["commit", "-qm", "fixture"], check=True)
        errors = checker.check_history(self.root, checker.MANIFEST, original)
        self.assertTrue(any("counter reset" in e for e in errors))


if __name__ == "__main__":
    unittest.main()
