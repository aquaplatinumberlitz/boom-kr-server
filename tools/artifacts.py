"""Verify recorded RE input identities; never modify client, pack or source DB."""
import argparse
import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def verify(manifest):
    rows = json.loads(manifest.read_text())
    failed = 0
    for row in rows:
        path = Path(row["path"])
        if not path.is_file():
            print(f"MISSING {path}")
            failed += 1
            continue
        with path.open("rb") as source:
            digest = hashlib.file_digest(source, "sha256").hexdigest()
        valid = digest == row["sha256"] and path.stat().st_size == row["bytes"]
        print(f"{'OK' if valid else 'MISMATCH'} {path}")
        failed += not valid
    return 1 if failed else 0


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("command", choices=["verify"])
    parser.add_argument("--manifest", type=Path, default=ROOT / "docs/evidence/artifacts.json")
    args = parser.parse_args()
    raise SystemExit(verify(args.manifest))


if __name__ == "__main__":
    main()
