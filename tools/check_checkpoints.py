#!/usr/bin/env python3
"""Check explicit checkpoint records, evidence and Git continuity; never infer RE truth."""
import argparse
import hashlib
import json
from pathlib import Path
import re
import subprocess
import sys
from collections.abc import Sequence

# Also support importlib-based legacy test runners.
if str(Path(__file__).resolve().parent) not in sys.path:
    sys.path.insert(0, str(Path(__file__).resolve().parent))
import checkpoint_store as store

ROOT = Path(__file__).resolve().parents[1]
MANIFEST = Path("docs/evidence/checkpoints/manifest.json")
CORE_DOCS = ("README.md", "docs/PLAN.md", "docs/PROTOCOL.md", "docs/RESEARCH.md")
SOURCES = {"kr_static", "kr_runtime", "vn_static", "vn_runtime", "emu_choice", "design"}
STATES = {"confirmed", "candidate", "unknown", "design"}
SHA = re.compile(r"[0-9a-f]{64}")
COMMIT = re.compile(r"[0-9a-f]{40}")
MUTABLE_METADATA_DOCS = {
    'docs/features.json',
    'docs/evidence/issue-reconciliation/github-snapshot.json',
    'docs/evidence/issue-reconciliation/review.json',
}


def audit_core_read_order(read_files):
    """Rules may be read first; core audit docs must still be contiguous/in order.

    Immutable AUD057 explicitly records AGENTS before the required core pass.
    Do not reorder the retained audit merely to satisfy the checker.
    """
    if not isinstance(read_files, list):
        return False
    offset = 1 if read_files[:1] == ['AGENTS.md'] else 0
    return read_files[offset:offset + 4] == list(CORE_DOCS)


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


def historical_checkpoint_source(root, relative, expected_sha, expected_bytes, checkpoint):
    """Resolve a source pin at its v2 record's publication, not an arbitrary revision.

    Local/raw/artifact files and non-source repository files never take this path.
    A source blob committed later cannot repair a missing publication-time pin.
    Older inline-only records without a matching v2 publication stay strict.
    """
    if (not re.fullmatch(r'CP[0-9]{3,}', checkpoint)
            or Path(relative).parts[0] not in {'server', 'tools'}
            or Path(relative).suffix != '.py'):
        return None
    return historical_checkpoint_publication(
        root, relative, expected_sha, expected_bytes, checkpoint)


def historical_checkpoint_publication(root, relative, expected_sha, expected_bytes, checkpoint):
    """Require an exact declared blob at the first publication of this record."""
    if not re.fullmatch(r'CP[0-9]{3,}', checkpoint):
        return None
    record_path = f'docs/evidence/checkpoints/records/{checkpoint}.json'
    try:
        births = subprocess.run(
            ['git', 'log', '--reverse', '--diff-filter=A', '--format=%H', 'HEAD', '--', record_path],
            cwd=root, capture_output=True, text=True, check=True).stdout.splitlines()
        if not births:
            return None
        revision = births[0]
        def read(path):
            store.target(root, path)
            return subprocess.run(['git', 'show', f'{revision}:{path}'], cwd=root,
                                  capture_output=True, check=True).stdout
        record = json.loads(read(record_path))
        if record.get('id') != checkpoint or record.get('lifecycle') != 'complete':
            return None
        declared = any(row.get('kind') == 'repository' and row.get('path') == relative
                       and row.get('sha256') == expected_sha and row.get('bytes') == expected_bytes
                       for row in store.evidence(record, root=root, read=read))
        if declared:
            raw = read(relative)
            if len(raw) == expected_bytes and hashlib.sha256(raw).hexdigest() == expected_sha:
                return revision
    except (OSError, ValueError, TypeError, subprocess.CalledProcessError):
        pass
    return None


def verify_evidence_item(item, root=ROOT, require_local=False, where="evidence"):
    errors, warnings, checked, unavailable = [], [], 0, 0
    root = Path(root).resolve()
    if not require_fields(item, ("id", "kind", "path", "sha256", "bytes"), where, errors):
        return errors, warnings, checked, unavailable
    kind = item.get("kind")
    if kind not in {"local", "repository", "artifact"}:
        errors.append(f"{where}: invalid evidence kind")
        return errors, warnings, checked, unavailable
    value = item.get("path")
    absolute_local_pin = False
    try:
        if kind == "artifact":
            path = Path(value)
            if not path.is_absolute():
                raise ValueError("artifact path must be an explicit absolute host path")
        else:
            # CP187 published some absolute host-local pins as "repository".
            # Keep the complete record immutable: resolve only paths inside
            # this checkout's local/ tree, report the metadata defect, and
            # verify raw bytes strictly. Never use Git fallback for these pins.
            declared = Path(value)
            if declared.is_absolute() and declared.resolve().is_relative_to(root / 'local'):
                path = declared.resolve()
                absolute_local_pin = True
                warnings.append(f"{where}: LEGACY ABSOLUTE LOCAL PIN {value}; "
                                f"declared kind={kind}, verified as host-local raw; "
                                "record unchanged, no Git fallback")
            else:
                path = local_target(root, value)
            is_local = path.is_relative_to(root / "local")
            if not absolute_local_pin and is_local != (kind == "local"):
                raise ValueError("kind/path mismatch")
    except (ValueError, TypeError) as error:
        errors.append(f"{where}: {error}")
        return errors, warnings, checked, unavailable
    if not isinstance(item.get("sha256"), str) or not SHA.fullmatch(item["sha256"]):
        errors.append(f"{where}: invalid SHA-256")
    if not isinstance(item.get("bytes"), int) or isinstance(item["bytes"], bool) or item["bytes"] < 0:
        errors.append(f"{where}: invalid byte count")
    if not path.is_file():
        unavailable += 1
        message = f"{where}: UNVERIFIED unavailable host evidence {value}"
        (errors if require_local or kind == "repository" else warnings).append(message)
        return errors, warnings, checked, unavailable
    checked += 1
    if path.stat().st_size != item.get("bytes") or digest(path) != item.get("sha256"):
        # Canonical docs can evolve while old checkpoint bytes remain immutable.
        # Docs can use exact history; project Python source additionally needs
        # the same declaration and exact blob at the checkpoint's publication.
        # Binaries, raw captures and local evidence still require current pins.
        revision = None
        historical_kind = 'DOC'
        if not absolute_local_pin and kind == "repository" and path.suffix == ".md":
            revision = historical_repository_blob(root, value, item.get("sha256"), item.get("bytes"))
        elif not absolute_local_pin and kind == 'repository' and value in MUTABLE_METADATA_DOCS:
            revision = historical_checkpoint_publication(
                root, value, item.get('sha256'), item.get('bytes'), where)
            historical_kind = 'METADATA DOC'
        elif not absolute_local_pin and kind == 'repository':
            revision = historical_checkpoint_source(root, value, item.get('sha256'), item.get('bytes'), where)
            historical_kind = 'SOURCE'
        if revision:
            warnings.append(f"{where}: HISTORICAL {historical_kind} VERIFIED {value} at {revision}; current file differs; checkpoint bytes unchanged")
        else:
            errors.append(f"{where}: evidence size/hash mismatch {value}")
    return errors, warnings, checked, unavailable


def documentation_identity(record, evidence):
    """A documentation-only checkpoint has pinned canonical documents, not a binary.

    This is declaration validation only. The common evidence verifier still
    checks every file/hash/history; it must not certify implementation/runtime.
    """
    findings = record.get("findings")
    if record.get("artifacts") != [] or not isinstance(findings, list) or not findings:
        return False
    docs = {
        item.get("path"): item.get("id") for item in evidence
        if isinstance(item, dict) and item.get("kind") == "repository"
        and isinstance(item.get("path"), str) and item["path"].startswith("docs/")
        and item["path"].endswith(".md") and SHA.fullmatch(str(item.get("sha256", "")))
        and isinstance(item.get("bytes"), int) and item["bytes"] > 0
    }
    for finding in findings:
        if not isinstance(finding, dict) or finding.get("source") != "design" \
                or finding.get("method") != "documentation" \
                or finding.get("status") not in {"design", "candidate", "unknown"}:
            return False
        canonical = str(finding.get("canonical", "")).split("#", 1)[0]
        if canonical not in docs or not isinstance(finding.get("evidence"), list) \
                or not set(finding["evidence"]).intersection(docs.values()):
            return False
    return True


def validate(manifest, root=ROOT, require_local=False):
    errors, warnings = [], []
    checked, unavailable = 0, 0
    root = Path(root).resolve()
    if not require_fields(manifest, (
        "schema", "last_id", "completed_count",
        "checkpoints", "audits", "task_states",
    ), "manifest", errors):
        return errors, warnings, {}
    for name in ("audit_interval", "next_periodic_audit"):
        if name not in manifest:
            errors.append(f"manifest: missing {name}")
    if manifest.get("schema") not in {store.V1, store.V2}:
        errors.append("manifest: unsupported schema")
    try:
        records = store.records(manifest, root)
    except (ValueError, OSError, TypeError) as error:
        return [f"manifest: {error}"], warnings, {}
    audits = manifest.get("audits")
    tasks = manifest.get("task_states")
    if not isinstance(records, Sequence) or not records:
        errors.append("manifest: checkpoints must be a nonempty list")
        records = []
    if not isinstance(audits, list):
        errors.append("manifest: audits must be a list")
        audits = []
    if not isinstance(tasks, list):
        errors.append("manifest: task_states must be a list")
        tasks = []

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
        item_errors, item_warnings, good, missing = verify_evidence_item(item, root, require_local, where)
        errors.extend(item_errors)
        warnings.extend(item_warnings)
        checked += good
        unavailable += missing

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
        declared_evidence = list(store.evidence(record, root)) if not record.get("artifacts") else None
        doc_identity = declared_evidence is not None and documentation_identity(record, declared_evidence)
        if (not record.get("artifacts") and not doc_identity) or not record.get("locators"):
            errors.append(f"{where}: artifact identity and locators/run IDs required")
        if doc_identity:
            warnings.append(f"{where}: DOCUMENTATION identity from pinned canonical docs; not implementation/runtime certification")
        evidence_ids = set()
        for item in (declared_evidence if declared_evidence is not None else store.evidence(record, root)):
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
            if item.get("method") not in {"static", "emulation", "runtime", "design", "documentation", "test"}:
                errors.append(f"{where}: invalid evidence method")
            if item.get("method") == "test":
                warnings.append(f"{where}: automated TEST evidence; not native client/runtime certification")
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
    policy = manifest.get("audit_policy")
    grouped = policy is not None
    cutoff = completed
    if grouped:
        if not isinstance(policy, dict) or policy.get("mode") != "feature_group":
            errors.append("manifest: invalid audit_policy")
            policy = {}
        effective = policy.get("effective_after")
        if effective not in ids[:completed]:
            errors.append("manifest: audit policy must follow an existing complete checkpoint")
        else:
            cutoff = ids.index(effective) + 1
        authorization = policy.get("authorization", {})
        if not isinstance(authorization, dict):
            authorization = {}
        name = authorization.get("path")
        if not isinstance(name, str) or not name.startswith("docs/evidence/owner-directives/"):
            errors.append("manifest: audit policy needs a pinned owner directive")
        else:
            path = file_link(name, "audit policy")
            if path and path.is_file():
                if (digest(path) != authorization.get("sha256")
                        or path.stat().st_size != authorization.get("bytes")):
                    errors.append("manifest: audit policy authorization hash/size mismatch")
                try:
                    decision = read_json(path)
                    if (decision.get("authority") != "project_owner"
                            or decision.get("audit_mode") != "feature_group"
                            or decision.get("effective_after") != effective):
                        errors.append("manifest: audit policy authorization disagrees with transition")
                except (ValueError, OSError, AttributeError):
                    errors.append("manifest: invalid audit policy authorization JSON")
        if manifest.get("audit_interval") is not None:
            errors.append("manifest: feature-group audit_interval must be null")
    elif manifest.get("audit_interval") != 4:
        errors.append("manifest: legacy audit_interval must be 4")

    completed_groups = set()
    for record in records:
        groups = record.get("completed_feature_groups", [])
        if (not isinstance(groups, list)
                or any(not isinstance(g, str) or not g.strip() for g in groups)
                or len(set(groups)) != len(groups)):
            errors.append(f"{record.get('id')}: invalid completed_feature_groups")
        elif record.get("lifecycle") == "complete":
            completed_groups.update((record.get("id"), group) for group in groups)
    audited_groups = set()
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
        if not audit_core_read_order(read_files) or not any(str(x).startswith("docs/evidence/") for x in read_files):
            errors.append(f"{where}: record manual reading in core-doc order plus evidence")
        for name in read_files:
            file_link(name, where)
        if audit.get("trigger") == "periodic":
            number = ids.index(audit.get("completed_through")) + 1 if audit.get("completed_through") in ids else 0
            if not number or number % 4:
                errors.append(f"{where}: periodic audit must cover CP004/008/...")
            periodic.append(number)
            if grouped and number > cutoff:
                errors.append(f"{where}: periodic cadence ended at {cp_id(cutoff)}")
        elif audit.get("trigger") == "feature_group":
            groups = audit.get("feature_groups")
            if (not grouped or not isinstance(groups, list) or not groups
                    or any(not isinstance(g, str) or not g.strip() for g in groups)
                    or len(set(groups)) != len(groups)):
                errors.append(f"{where}: feature-group audit needs explicit feature_groups")
            else:
                pairs = {(audit.get("completed_through"), group) for group in groups}
                if not pairs.issubset(completed_groups):
                    errors.append(f"{where}: feature-group audit must match declared checkpoint group completion")
                audited_groups.update(pairs)
        elif audit.get("trigger") not in {"initial_reconciliation", "serious_contradiction", "ad_hoc"}:
            errors.append(f"{where}: invalid trigger")
    for checkpoint, group in sorted(completed_groups - audited_groups):
        errors.append(f"{checkpoint}: FEATURE GROUP AUDIT DUE: {group}")
    due = [n for n in range(4, cutoff + 1, 4) if n not in periodic]
    for number in due:
        errors.append(f"{cp_id(number)}: MANUAL AUDIT DUE; ad-hoc audits do not reset cadence")
    last_periodic = max(periodic, default=0)
    if manifest.get("last_periodic_audit") != last_periodic:
        errors.append("manifest: last_periodic_audit disagrees with audit records")
    next_audit = None if grouped else cp_id(last_periodic + 4)
    if manifest.get("next_periodic_audit") != next_audit:
        errors.append("manifest: next_periodic_audit disagrees with audit policy")
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
                              "next_audit": next_audit,
                              "audit_mode": "feature_group" if grouped else "legacy_periodic"}


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


def history_signature(manifest, root=ROOT, read=None):
    """Logical record identities across inline/ref schemas; never normalize stored records."""
    entries = []
    completed = 0
    for index, record in enumerate(store.records(manifest, root, read), 1):
        if (record.get('id') != cp_id(index)
                or record.get('previous') != (cp_id(index - 1) if index > 1 else 'origin')):
            raise ValueError('checkpoint counter reset/noncontiguous ID/previous in history')
        completed += record.get('lifecycle') == 'complete'
        # Validate external manifests in committed snapshots too. This also
        # detects manifest-only changes without any checkpoint/index mutation.
        for _ in store.evidence(record, root, read):
            pass
        entries.append({"id": record.get("id"), "lifecycle": record.get("lifecycle"),
                        "content": store.object_digest({k: v for k, v in record.items() if k != "review"}),
                        "review": record.get("review")})
    if (manifest.get('last_id') != (entries[-1]['id'] if entries else None)
            or manifest.get('completed_count') != completed):
        raise ValueError('checkpoint counter reset/count mismatch in history')
    return {"checkpoints": entries, "audits": manifest.get("audits", [])}


def signature_continuity(previous, current):
    errors = []
    before, after = previous['checkpoints'], current['checkpoints']
    if len(after) < len(before) or [x['id'] for x in before] != [x['id'] for x in after[:len(before)]]:
        errors.append('checkpoint history deleted, reordered or counter reset')
    for old, new in zip(before, after):
        if old['lifecycle'] != 'complete':
            continue
        if old['content'] != new['content']:
            errors.append(f"{old['id']}: completed record changed; append a correction checkpoint")
        if old.get('review', {}).get('status') in {'approved', 'waived'} and old['review'] != new['review']:
            errors.append(f"{old['id']}: {old['review']['status']} review is immutable; append a correction checkpoint")
    if previous['audits'] != current['audits'][:len(previous['audits'])]:
        errors.append('manual audit history deleted or changed')
    return errors


def check_history(root, relative_manifest, current, availability=None):
    def unverified(message):
        if availability is None:
            raise ValueError(message)
        availability.append(message)

    command = ['git', '-c', 'core.fsmonitor=false', '-C', str(root)]
    try:
        shallow = subprocess.run(command + ['rev-parse', '--is-shallow-repository'], capture_output=True, text=True)
        # Watch the entire checkpoint directory: record/audit/evidence-only
        # tampering must not escape detection when the index does not change.
        revisions = subprocess.run(command + ['log', '--first-parent', '--reverse', '--format=%H',
                                              'HEAD', '--', relative_manifest.parent.as_posix()],
                                   capture_output=True, text=True)
    except OSError as error:
        raise ValueError(f'Git unavailable: {error}') from error
    if revisions.returncode:
        raise ValueError('Git history unavailable')
    if shallow.returncode == 0 and shallow.stdout.strip() == 'true':
        unverified('shallow Git history; earlier counter continuity UNVERIFIED')
    if not revisions.stdout.strip():
        unverified('no committed manifest history; counter rollback UNVERIFIED')
    previous, errors, seen_manifest = None, [], False
    for revision in revisions.stdout.splitlines():
        def read(path):
            target = store.target(root, path)  # validate repository-relative path
            relative = target.relative_to(Path(root).resolve()).as_posix()
            result = subprocess.run(command + ['show', f'{revision}:{relative}'], capture_output=True)
            if result.returncode:
                raise ValueError(f'missing committed file {relative}')
            return result.stdout
        try:
            content = read(relative_manifest.as_posix())
        except ValueError:
            if seen_manifest:
                errors.append(f'{revision[:8]}: checkpoint manifest removed from committed history')
            continue  # documentation may predate the first manifest
        seen_manifest = True
        try:
            snapshot = json.loads(content)
            signature = history_signature(snapshot, root, read)
            # Audit bodies are pinned, even on commits changing only an audit file.
            for audit in snapshot.get('audits', []):
                if hashlib.sha256(read(audit['path'])).hexdigest() != audit['sha256']:
                    raise ValueError(f"audit hash mismatch: {audit['path']}")
        except (ValueError, OSError, KeyError, TypeError) as error:
            errors.append(f'{revision[:8]}: invalid committed checkpoint data: {error}')
            continue
        if previous is not None:
            errors.extend(f'{revision[:8]}: {e}' for e in signature_continuity(previous, signature))
        previous = signature
    if not seen_manifest and revisions.stdout.strip():
        unverified('no committed manifest history; counter rollback UNVERIFIED')
    if previous is not None:
        try:
            current_signature = history_signature(current, root)
            errors.extend(signature_continuity(previous, current_signature))
        except (ValueError, OSError, TypeError, KeyError) as error:
            errors.append(f'current checkpoint history invalid: {error}')
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
