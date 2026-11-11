"""Check local Markdown targets, fragments and current-document size budgets."""
import re
from collections import Counter
from pathlib import Path
from urllib.parse import unquote

ROOT = Path(__file__).resolve().parents[1]
LINE_LIMITS = {
    'docs/PLAN.md': (800, 'docs/HISTORY.md (diễn biến) hoặc docs/PLAN_ARCHIVE.md (nhánh/gate lịch sử)'),
    'README.md': (150, 'docs/HISTORY.md (mốc cũ); PLAN giữ thứ tự, docs/protocol/ giữ contract'),
    'docs/PROTOCOL.md': (600, 'docs/protocol/; chỉ giữ mục lục và stub anchor tại PROTOCOL.md'),
    'docs/RESEARCH.md': (300, 'docs/HISTORY.md; chỉ giữ câu hỏi mở và khoảng trống theo cơ chế'),
}


def without_code(text):
    return re.sub(r"(?ms)^```[^\n]*\n.*?^```[ \t]*$", "", text)


def markdown_anchors(text):
    text = without_code(text)
    anchors = set(re.findall(r'<a id="([^"]+)"', text))
    duplicates = Counter()
    for heading in re.findall(r'^#{1,6} (.+)$', text, re.M):
        heading = re.sub(r'!?\[([^\]]+)\]\([^)]+\)', r'\1', heading)
        heading = re.sub(r'<[^>]+>', '', heading)
        slug = re.sub(r'[^\w\- ]', '', heading.lower()).replace(' ', '-')
        suffix = duplicates[slug]
        duplicates[slug] += 1
        anchors.add(slug + (f'-{suffix}' if suffix else ''))
    return anchors


def size_errors(relative, text):
    limit = LINE_LIMITS.get(relative)
    if limit and len(text.splitlines()) > limit[0]:
        return [f'{relative}: {len(text.splitlines())} lines exceeds {limit[0]}; chuyển phần cũ sang {limit[1]}']
    return []


def link_errors(path, root, anchor_cache):
    errors, count = [], 0
    for match in re.finditer(r"\]\((<[^>]+>|[^\s)]+)(?:\s+\"[^\"]*\")?\)", without_code(path.read_text())):
        target = match.group(1).strip('<>')
        if re.match(r'^[a-zA-Z][\w+.-]*:', target):
            continue
        name, _, fragment = target.partition('#')
        name = unquote(name.split('?', 1)[0])
        resolved = (path.parent / name).resolve() if name else path.resolve()
        count += 1
        if not resolved.is_relative_to(root.resolve()) or not resolved.is_file():
            errors.append(f'{path.relative_to(root)}: invalid local link {target}')
        elif fragment and resolved.suffix == '.md':
            if resolved not in anchor_cache:
                anchor_cache[resolved] = markdown_anchors(resolved.read_text())
            if unquote(fragment) not in anchor_cache[resolved]:
                errors.append(f'{path.relative_to(root)}: invalid local fragment {target}')
    return errors, count


def main():
    errors, count = [], 0
    files = [ROOT / "README.md", ROOT / "AGENTS.md"]
    files += sorted((ROOT / "docs").rglob("*.md"))
    files += sorted((ROOT / "tools").glob("*.md"))
    files += sorted((ROOT / "server").glob("*.md"))
    anchor_cache = {}
    for path in files:
        errors.extend(size_errors(str(path.relative_to(ROOT)), path.read_text()))
        found, links = link_errors(path, ROOT, anchor_cache)
        errors.extend(found)
        count += links
    if errors:
        raise SystemExit("\n".join(errors))
    from feature_catalog import load, validate, render, previous_catalog, OUTPUT
    catalog = load()
    errors = validate(catalog, previous=previous_catalog())
    if (ROOT / OUTPUT).read_text() != render(catalog):
        errors.append("COVERAGE stale: run python3 tools/feature_catalog.py --generate")
    # Offline dated snapshot only: current declarations, never semantic acceptance/history rewrite.
    from check_issue_links import validate as issue_validate, SNAPSHOT, REVIEWS
    import json
    from checkpoint_store import load as load_checkpoints, current_view
    issue_errors, issue_warnings = issue_validate(
        json.loads((ROOT / SNAPSHOT).read_text()), json.loads((ROOT / REVIEWS).read_text()),
        catalog, current_view(load_checkpoints(ROOT), ROOT),
        (ROOT / 'docs/PLAN.md').read_text())
    errors.extend(issue_errors)
    for warning in issue_warnings:
        print('WARNING:', warning)
    if errors:
        raise SystemExit("\n".join(errors))
    print("Feature catalog declarations/links/anchors/dependencies/generated view OK; semantics/gates not certified.")
    print(f"Docs OK: {len(files)} files, {count} internal links including Markdown fragments; size budgets checked.")


if __name__ == "__main__":
    main()
