#!/usr/bin/env python3
"""Check explicit checkpoint records, evidence and Git continuity; never infer RE truth."""
import argparse
import hashlib
import json
from pathlib import Path
import re
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]
MANIFEST = Path("docs/evidence/checkpoints/manifest.json")
CORE_DOCS = ("README.md", "docs/PLAN.md", "docs/PROTOCOL.md", "docs/RESEARCH.md")
SOURCES = {"kr_static", "kr_runtime", "vn_static", "vn_runtime", "emu_choice", "design"}
STATES = {"confirmed", "candidate", "unknown", "design"}
SHA = re.compile(r"[0-9a-f]{64}")
COMMIT = re.compile(r"[0-9a-f]{40}")


def digest(path):
    result = hashlib.sha256()
    with path.open("rb") as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b""):
            result.update(block)
    return result.hexdigest()


def cp_id(number):
    return f"CP{number:03d}"


def local_target(root, text):
    """Only repository-relative targets. Fragments are recorded but not interpreted."""
    if not isinstance(text, str) or not text:
        raise ValueError("empty path")
    path = Path(text.split("#", 1)[0])
    target = (root / path).resolve()
    if path.is_absolute() or not target.is_relative_to(root.resolve()):
        raise ValueError(f"path outside repository: {text}")
    return target


def require_fields(obj, fields, where, errors):
    if not isinstance(obj, dict):
        errors.append(f"{where}: expected object")
        return False
    for field in fields:
        if field not in obj or obj[field] is None or obj[field] == "":
            errors.append(f"{where}: missing {field}")
    return True


def read_json(path):
    return json.loads(path.read_text())


def historical_repository_blob(root, relative, expected_sha, expected_bytes):
    """Recover exact historical bytes; never accept a changed file by path alone."""
    try:
        revisions = subprocess.run(
            ["git", "-c", "core.fsmonitor=false", "log", "--all", "--format=%H", "--", relative],
            cwd=root, capture_output=True, text=True, check=True)
        for revision in revisions.stdout.splitlines():
            result = subprocess.run(["git", "-c", "core.fsmonitor=false", "show", f"{revision}:{relative}"],
                                    cwd=root, capture_output=True)
            if (result.returncode == 0 and len(result.stdout) == expected_bytes
                    and hashlib.sha256(result.stdout).hexdigest() == expected_sha):
                return revision
    except (OSError, subprocess.CalledProcessError):
        pass
    return None


def validate(manifest, root=ROOT, require_local=False):
    errors, warnings = [], []
    checked, unavailable = 0, 0
    root = Path(root).resolve()
    if not require_fields(manifest, (
        "schema", "last_id", "completed_count", "audit_interval", "next_periodic_audit",
        "checkpoints", "audits", "task_states",
    ), "manifest", errors):
        return errors, warnings, {}
    if manifest.get("schema") != "kre-checkpoints-v1":
        errors.append("manifest: unsupported schema")
    records = manifest.get("checkpoints")
    audits = manifest.get("audits")
    tasks = manifest.get("task_states")
    if not isinstance(records, list) or not records:
        errors.append("manifest: checkpoints must be a nonempty list")
        records = []
    if not isinstance(audits, list):
        errors.append("manifest: audits must be a list")
        audits = []
    if not isinstance(tasks, list):
        errors.append("manifest: task_states must be a list")
        tasks = []
    if manifest.get("audit_interval") != 4:
        errors.append("manifest: audit_interval must be 4")

    def file_link(value, where):
        try:
            path = local_target(root, value)
            if not path.is_file():
                errors.append(f"{where}: missing linked file {value}")
            return path
        except (ValueError, TypeError) as error:
            errors.append(f"{where}: {error}")
            return None

    def evidence_file(item, where):
        nonlocal checked, unavailable
        if not require_fields(item, ("id", "kind", "path", "sha256", "bytes"), where, errors):
            return
        kind = item.get("kind")
        if kind not in {"local", "repository", "artifact"}:
            errors.append(f"{where}: invalid evidence kind")
            return
        value = item.get("path")
        try:
            if kind == "artifact":
                path = Path(value)
                if not path.is_absolute():
                    raise ValueError("artifact path must be an explicit absolute host path")
            else:
                path = local_target(root, value)
                is_local = path.is_relative_to(root / "local")
                if is_local != (kind == "local"):
                    raise ValueError("kind/path mismatch")
        except (ValueError, TypeError) as error:
            errors.append(f"{where}: {error}")
            return
        if not isinstance(item.get("sha256"), str) or not SHA.fullmatch(item["sha256"]):
            errors.append(f"{where}: invalid SHA-256")
        if not isinstance(item.get("bytes"), int) or isinstance(item["bytes"], bool) or item["bytes"] < 0:
            errors.append(f"{where}: invalid byte count")
        if not path.is_file():
            unavailable += 1
            message = f"{where}: UNVERIFIED unavailable host evidence {value}"
            (errors if require_local or kind == "repository" else warnings).append(message)
            return
        checked += 1
        if path.stat().st_size != item.get("bytes") or digest(path) != item.get("sha256"):
            # Canonical docs can evolve while old checkpoint bytes remain immutable.
            # Recover the old exact content only for repository documentation. Code,
            # binaries, raw captures and local evidence still require the pinned bytes.
            revision = None
            if kind == "repository" and path.suffix == ".md":
                revision = historical_repository_blob(root, value, item.get("sha256"), item.get("bytes"))
            if revision:
                warnings.append(f"{where}: HISTORICAL DOC VERIFIED {value} at {revision}; current file differs; checkpoint bytes unchanged")
            else:
                errors.append(f"{where}: evidence size/hash mismatch {value}")

    completed, saw_draft = 0, False
    ids, current_steps = [], {}
    for index, record in enumerate(records, 1):
        where = cp_id(index)
        if not require_fields(record, (
            "id", "previous", "lifecycle", "recorded_utc", "question", "scope",
            "context_commit", "artifacts", "locators", "findings", "verification",
            "evidence", "delta", "blockers", "next_steps", "owner", "reviewer",
            "review", "docs_sync",
        ), where, errors):
            continue
        expected_previous = cp_id(index - 1) if index > 1 else "origin"
        if record.get("id") != where or record.get("previous") != expected_previous:
            errors.append(f"{where}: noncontiguous id/previous")
        ids.append(record.get("id"))
        lifecycle = record.get("lifecycle")
        if lifecycle == "complete":
            completed += 1
            if saw_draft:
                errors.append(f"{where}: complete record follows an incomplete record")
        elif lifecycle == "in_progress":
            saw_draft = True
            warnings.append(f"{where}: INCOMPLETE; not a completed checkpoint")
        else:
            errors.append(f"{where}: invalid lifecycle")
        if record.get("owner") == record.get("reviewer") and record.get("review", {}).get("status") != "waived":
            errors.append(f"{where}: reviewer must differ from owner")
        if not isinstance(record.get("context_commit"), str) or not COMMIT.fullmatch(record["context_commit"]):
            errors.append(f"{where}: invalid context commit")
        for artifact in record.get("artifacts", []):
            if not isinstance(artifact, dict) or not SHA.fullmatch(str(artifact.get("sha256", ""))):
                errors.append(f"{where}: invalid artifact identity")
        if not record.get("artifacts") or not record.get("locators"):
            errors.append(f"{where}: artifact identity and locators/run IDs required")
        evidence_ids = set()
        for item in record.get("evidence", []):
            evidence_file(item, where)
            if isinstance(item, dict):
                if item.get("id") in evidence_ids:
                    errors.append(f"{where}: duplicate evidence id")
                evidence_ids.add(item.get("id"))
        if not evidence_ids:
            errors.append(f"{where}: evidence required")
        findings = record.get("findings", [])
        if not findings:
            errors.append(f"{where}: findings required")
        for item in findings:
            if not require_fields(item, ("summary", "source", "status", "method", "limits", "canonical", "evidence"), where, errors):
                continue
            if item.get("source") not in SOURCES or item.get("status") not in STATES:
                errors.append(f"{where}: invalid finding source/status")
            if item.get("method") not in {"static", "emulation", "runtime", "design", "documentation"}:
                errors.append(f"{where}: invalid evidence method")
            file_link(item.get("canonical"), where)
            if not item.get("evidence") or any(e not in evidence_ids for e in item["evidence"]):
                errors.append(f"{where}: finding references unknown/no evidence")
        checks = record.get("verification", [])
        if not checks:
            errors.append(f"{where}: verification records required")
        for item in checks:
            if not require_fields(item, ("check", "result", "evidence", "limits"), where, errors):
                continue
            if item.get("result") not in {"pass", "fail", "observed", "pending", "not_run"}:
                errors.append(f"{where}: invalid verification result")
            if any(e not in evidence_ids for e in item.get("evidence", [])):
                errors.append(f"{where}: verification references unknown evidence")
            if item.get("result") in {"pending", "not_run", "fail"}:
                warnings.append(f"{where}: {item['result'].upper()} {item.get('check')}")
        delta = record.get("delta", {})
        require_fields(delta, ("added", "corrected", "superseded"), where + ".delta", errors)
        review = record.get("review", {})
        require_fields(review, ("status", "as_of_utc", "reference", "scope"), where + ".review", errors)
        if review.get("status") not in {"pending", "approved", "changes_requested", "waived"}:
            errors.append(f"{where}: invalid review status")
        if review.get("status") == "waived":
            authorization = review.get("authorization", {})
            if require_fields(authorization, ("by", "reference", "reason"), where + ".review.authorization", errors):
                if authorization.get("by") != "project_owner":
                    errors.append(f"{where}: review waiver requires declared project_owner authorization")
                if not isinstance(authorization.get("reason"), str) or not authorization["reason"].strip():
                    errors.append(f"{where}: review waiver requires a reason")
                file_link(authorization.get("reference"), where + ".review.authorization")
            warnings.append(f"{where}: REVIEW WAIVED; independent approval waived; see {review.get('reference')}")
        elif review.get("status") != "approved":
            warnings.append(f"{where}: REVIEW {review.get('status')}; see {review.get('reference')}")
        elif not COMMIT.fullmatch(str(review.get("head", ""))):
            errors.append(f"{where}: approved review requires exact head")
        sync_paths = set()
        for item in record.get("docs_sync", []):
            if not require_fields(item, ("path", "state", "reason"), where + ".docs_sync", errors):
                continue
            path = file_link(item.get("path"), where)
            if path:
                sync_paths.add(str(path.relative_to(root)))
            if item.get("state") == "blocked":
                (errors if lifecycle == "complete" else warnings).append(f"{where}: DOCS UNSYNCED {item.get('path')}: {item.get('reason')}")
            elif item.get("state") not in {"updated", "unchanged"}:
                errors.append(f"{where}: invalid docs sync state")
        if not set(CORE_DOCS).issubset(sync_paths) or not any(x.startswith("docs/evidence/") for x in sync_paths):
            errors.append(f"{where}: docs_sync must cover all four core docs and evidence")
        for step in record.get("next_steps", []):
            if not require_fields(step, ("id", "task", "state", "action", "owner", "reviewer"), where + ".next_steps", errors):
                continue
            if step.get("state") not in {"active", "closed", "superseded"}:
                errors.append(f"{where}: invalid next-step state")
            current_steps[step.get("id")] = step

    if manifest.get("last_id") != (ids[-1] if ids else None):
        errors.append("manifest: last_id disagrees with sequence")
    if manifest.get("completed_count") != completed:
        errors.append("manifest: completed_count disagrees with lifecycle")
    periodic = []
    for index, reference in enumerate(audits, 1):
        where = f"AUD{index:03d}"
        if not require_fields(reference, ("id", "path", "sha256"), where, errors):
            continue
        if reference.get("id") != where:
            errors.append(f"{where}: noncontiguous audit ID")
        path = file_link(reference.get("path"), where)
        if not path or not path.is_file():
            continue
        if digest(path) != reference.get("sha256"):
            errors.append(f"{where}: audit hash mismatch")
        try:
            audit = read_json(path)
        except (ValueError, OSError) as error:
            errors.append(f"{where}: invalid audit JSON: {error}")
            continue
        if not require_fields(audit, ("id", "trigger", "completed_through", "auditor", "read_files", "findings", "remaining", "recorded_utc"), where, errors):
            continue
        if audit.get("id") != where or audit.get("completed_through") not in ids[:completed]:
            errors.append(f"{where}: audit must cover an existing complete checkpoint")
        read_files = audit.get("read_files", [])
        if read_files[:4] != list(CORE_DOCS) or not any(str(x).startswith("docs/evidence/") for x in read_files):
            errors.append(f"{where}: record manual reading in core-doc order plus evidence")
        for name in read_files:
            file_link(name, where)
        if audit.get("trigger") == "periodic":
            number = ids.index(audit.get("completed_through")) + 1 if audit.get("completed_through") in ids else 0
            if not number or number % 4:
                errors.append(f"{where}: periodic audit must cover CP004/008/...")
            periodic.append(number)
        elif audit.get("trigger") not in {"initial_reconciliation", "serious_contradiction", "ad_hoc"}:
            errors.append(f"{where}: invalid trigger")
    due = [n for n in range(4, completed + 1, 4) if n not in periodic]
    for number in due:
        errors.append(f"{cp_id(number)}: MANUAL AUDIT DUE; ad-hoc audits do not reset cadence")
    last_periodic = max(periodic, default=0)
    if manifest.get("last_periodic_audit") != last_periodic:
        errors.append("manifest: last_periodic_audit disagrees with audit records")
    if manifest.get("next_periodic_audit") != cp_id(last_periodic + 4):
        errors.append("manifest: next_periodic_audit disagrees with cadence")
    expected_last_audit = audits[-1].get("id") if audits else None
    if manifest.get("last_manual_audit") != expected_last_audit:
        errors.append("manifest: last_manual_audit disagrees with audit sequence")

    seen_tasks = set()
    for task in tasks:
        if not require_fields(task, ("id", "state", "canonical", "reference"), "task", errors):
            continue
        task_id = task.get("id")
        if task_id in seen_tasks:
            errors.append("duplicate task state")
        seen_tasks.add(task_id)
        if task.get("state") not in {"open", "closed"}:
            errors.append(f"{task_id}: invalid task state")
        file_link(task.get("canonical"), str(task_id))
        active = [s["id"] for s in current_steps.values() if s.get("task") == task_id and s.get("state") == "active"]
        if task.get("state") == "closed" and active:
            errors.append(f"{task_id}: closed task still has active next steps {active}")
    for step in current_steps.values():
        if step.get("task") not in seen_tasks:
            errors.append(f"{step.get('id')}: task has no explicit state")
    return errors, warnings, {"completed": completed, "evidence_verified": checked, "evidence_unavailable": unavailable,
                              "next_audit": cp_id(last_periodic + 4)}


def continuity(previous, current):
    """Protect completed content while permitting later, explicit review updates."""
    errors = []
    before, after = previous.get("checkpoints", []), current.get("checkpoints", [])
    if len(after) < len(before) or [x.get("id") for x in before] != [x.get("id") for x in after[:len(before)]]:
        errors.append("checkpoint history deleted, reordered or counter reset")
    for old, new in zip(before, after):
        if old.get("lifecycle") == "complete":
            without_review = lambda x: {k: v for k, v in x.items() if k != "review"}
            if without_review(old) != without_review(new):
                errors.append(f"{old.get('id')}: completed record changed; append a correction checkpoint")
            if old.get("review", {}).get("status") == "approved" and old.get("review") != new.get("review"):
                errors.append(f"{old.get('id')}: approved review is immutable; append a correction checkpoint")
            if old.get("review", {}).get("status") == "waived" and old.get("review") != new.get("review"):
                errors.append(f"{old.get('id')}: waived review is immutable; append a correction checkpoint")
    old_audits, new_audits = previous.get("audits", []), current.get("audits", [])
    if old_audits != new_audits[:len(old_audits)]:
        errors.append("manual audit history deleted or changed")
    return errors


def check_history(root, relative_manifest, current, availability=None):
    def unverified(message):
        if availability is None:
            raise ValueError(message)
        availability.append(message)

    command = ["git", "-C", str(root)]
    try:
        shallow = subprocess.run(command + ["rev-parse", "--is-shallow-repository"], capture_output=True, text=True)
        revisions = subprocess.run(command + ["log", "--first-parent", "--reverse", "--format=%H", "HEAD", "--", str(relative_manifest)],
                                   capture_output=True, text=True)
    except OSError as error:
        raise ValueError(f"Git unavailable: {error}") from error
    if revisions.returncode:
        raise ValueError("Git history unavailable")
    if shallow.returncode == 0 and shallow.stdout.strip() == "true":
        unverified("shallow Git history; earlier counter continuity UNVERIFIED")
    if not revisions.stdout.strip():
        unverified("no committed manifest history; counter rollback UNVERIFIED")
    previous, errors = None, []
    for revision in revisions.stdout.splitlines():
        content = subprocess.run(command + ["show", f"{revision}:{relative_manifest.as_posix()}"], capture_output=True, text=True)
        if content.returncode:
            errors.append(f"{revision[:8]}: checkpoint manifest removed from committed history")
            continue
        try:
            snapshot = json.loads(content.stdout)
        except ValueError:
            errors.append(f"{revision[:8]}: invalid committed checkpoint JSON")
            continue
        if previous is not None:
            errors.extend(f"{revision[:8]}: {e}" for e in continuity(previous, snapshot))
        previous = snapshot
    if previous is not None:
        errors.extend(continuity(previous, current))
    return errors


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--require-local", action="store_true", help="Fail on unavailable host-local evidence; otherwise report UNVERIFIED")
    parser.add_argument("--require-history", action="store_true", help="Fail if Git is unavailable; checked whenever available")
    args = parser.parse_args()
    try:
        manifest = read_json(ROOT / MANIFEST)
        errors, warnings, stats = validate(manifest, ROOT, args.require_local)
        availability = []
        try:
            errors.extend(check_history(ROOT, MANIFEST, manifest, availability))
        except ValueError as error:
            availability.append(str(error) + "; counter rollback UNVERIFIED")
        (errors if args.require_history else warnings).extend(availability)
        stats["history_verified"] = not availability
        links = subprocess.run([sys.executable, str(ROOT / "tools/check_docs.py")], capture_output=True, text=True)
        if links.returncode:
            errors.append(links.stdout + links.stderr)
    except (OSError, ValueError, TypeError, KeyError) as error:
        errors, warnings, stats = [f"invalid manifest: {error}"], [], {}
    for warning in warnings:
        print("WARNING:", warning)
    for error in errors:
        print("ERROR:", error, file=sys.stderr)
    print(json.dumps({"ok": not errors, **stats, "warnings": len(warnings), "errors": len(errors)}, ensure_ascii=False))
    return 1 if errors else 0


if __name__ == "__main__":
    raise SystemExit(main())
