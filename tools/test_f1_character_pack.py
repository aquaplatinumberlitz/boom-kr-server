#!/usr/bin/env python3
"""Kiểm dữ liệu nhân vật F1-c từ đúng pack KR 00967, không dùng oracle tự sinh."""
import json
from pathlib import Path
import sys
sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "tools"))
from content_census import parse_bomber_attr
from kr_pack_export import Pack
from kr_pack_export import PACK_SHA256, F1_CHARACTERS, file_sha

GAME = Path("/home/admin/boom/clients/kr-00967")
EXPECTED = {"C00": (0, 0, 5, 7, 1, 10, 1, 7), "C01": (1, 0, 4, 8, 2, 7, 1, 9)}

def main():
    with (GAME / "FxContents.idx").open("rb") as index, (GAME / "FxContents.idd").open("rb") as data:
        assert file_sha(index) == PACK_SHA256["FxContents.idx"]
        assert file_sha(data) == PACK_SHA256["FxContents.idd"]
        rows = parse_bomber_attr(Pack(index, data).read("Attr/BomberAttr"))
    by_key = {row["values"][0]: row for row in rows}
    assert len(rows) == 67 and len(by_key) == 67
    assert {row["values"][0] for row in rows if row["name"].startswith("C")} - F1_CHARACTERS == {4019, 4025, 4026}
    for name, values in EXPECTED.items():
        assert next(row["values"] for row in rows if row["name"] == name)[:8] == list(values)
    for key in F1_CHARACTERS:
        row = by_key[key]["values"]
        assert 1 <= row[2] <= row[3] <= 10
        assert 1 <= row[4] <= row[5] <= 10
        assert 1 <= row[6] <= row[7] <= 10
    print(json.dumps({"rows": len(rows), "served_b_keys": len(F1_CHARACTERS), "unknown_skill_keys": [4019, 4025, 4026]}))

if __name__ == "__main__":
    main()
