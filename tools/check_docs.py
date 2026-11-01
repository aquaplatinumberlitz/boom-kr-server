"""Check navigable local Markdown file targets without network access."""
import re
from pathlib import Path
from urllib.parse import unquote

ROOT = Path(__file__).resolve().parents[1]


def main():
    errors, count = [], 0
    files = [ROOT / "README.md", ROOT / "AGENTS.md"]
    files += sorted((ROOT / "docs").rglob("*.md"))
    files += sorted((ROOT / "tools").glob("*.md"))
    files += sorted((ROOT / "server").glob("*.md"))
    for path in files:
        text = re.sub(r"(?ms)^```[^\n]*\n.*?^```[ \t]*$", "", path.read_text())
        for match in re.finditer(r"\]\((<[^>]+>|[^\s)]+)(?:\s+\"[^\"]*\")?\)", text):
            target = match.group(1).strip("<>")
            if target.startswith("#") or re.match(r"^[a-zA-Z][\w+.-]*:", target):
                continue
            name = unquote(target.split("#", 1)[0].split("?", 1)[0])
            count += 1
            resolved = (path.parent / name).resolve()
            if not resolved.is_relative_to(ROOT) or not resolved.is_file():
                errors.append(f"{path.relative_to(ROOT)}: invalid local link {target}")
    if errors:
        raise SystemExit("\n".join(errors))
    from feature_catalog import load, validate, render, previous_catalog, OUTPUT
    catalog = load()
    errors = validate(catalog, previous=previous_catalog())
    if (ROOT / OUTPUT).read_text() != render(catalog):
        errors.append("COVERAGE stale: run python3 tools/feature_catalog.py --generate")
    if errors:
        raise SystemExit("\n".join(errors))
    print("Feature catalog declarations/links/anchors/dependencies/generated view OK; semantics/gates not certified.")
    print(f"Docs OK: {len(files)} files, {count} internal file links; fragments not checked.")


if __name__ == "__main__":
    main()
