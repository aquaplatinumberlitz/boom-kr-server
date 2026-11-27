"""Sinh kiểm kê F7 từ nguồn trong repo và mô tả chính sách kiểm tra map tự tạo."""

import argparse
from collections import Counter, deque
import hashlib
import json
from pathlib import Path
import re
import stat
import sys


ROOT = Path(__file__).resolve().parents[1]
INPUTS = {
    "census": "docs/content/census.json",
    "classification": "docs/content/classification.json",
    "owner_directive": "docs/evidence/owner-directives/custom-map.json",
}
JSON_PATH = "docs/evidence/content/f7-custom-map-spec.json"
MD_PATH = "docs/content/f7-custom-map-spec.md"
ROADMAP = "docs/PLAN.md#full-game-roadmap"
BUILD = "KR00967"
ALLOWED_GRIDS = ((15, 13), (21, 21), (31, 13))
MAX_BYTES = 8192
MIN_SPAWNS, MAX_SPAWNS = 2, 8
MAX_TILE_ID = 65535
MAP_SCHEMA = "kre-f7-normalized-map-v1"
MAP_FIELDS = {"schema", "width", "height", "tiles", "spawns"}
RESOURCE_GROUP = "fx/CustomMap"
RESOURCE_FAMILIES = ("CustomMap", "CustomTileIndexList", "CustomTileList")


class AuditError(ValueError):
    """A diagnostic that does not expose untrusted input or local paths."""


def require(condition, message):
    if not condition:
        raise AuditError(message)


def sha256(raw):
    return hashlib.sha256(raw).hexdigest()


def is_sha256(value):
    return isinstance(value, str) and re.fullmatch(r"[0-9a-f]{64}", value) is not None


def canonical(value):
    return (json.dumps(value, ensure_ascii=False, sort_keys=True, separators=(",", ":")) + "\n").encode("utf-8")


def unique_object(pairs):
    result = dict(pairs)
    require(len(result) == len(pairs), "khóa JSON bị lặp")
    return result


def integer(value, low, high):
    return type(value) is int and low <= value <= high


def validate_payload(payload, palette, *, player_count=None, expected_sha256=None):
    """Validate a normalized payload and return its pin; this is not a KR decoder."""
    require(isinstance(payload, bytes) and 0 < len(payload) <= MAX_BYTES,
            "payload phải có từ 1 đến 8192 byte")
    try:
        document = json.loads(payload.decode("utf-8"), object_pairs_hook=unique_object)
    except AuditError:
        raise
    except (ValueError, RecursionError):
        raise AuditError("payload không phải JSON UTF-8 hợp lệ") from None
    require(isinstance(document, dict) and document.keys() == MAP_FIELDS and document["schema"] == MAP_SCHEMA,
            "schema hoặc trường của map không được hỗ trợ")
    width, height = document["width"], document["height"]
    require(type(width) is int and type(height) is int and (width, height) in ALLOWED_GRIDS,
            "kích thước lưới không được phép")
    require(isinstance(palette, dict) and palette and all(
        integer(tile, 0, MAX_TILE_ID) and type(walkable) is bool for tile, walkable in palette.items()
    ), "bảng tile tin cậy không hợp lệ")
    tiles = document["tiles"]
    require(isinstance(tiles, list) and len(tiles) == width * height and all(
        integer(tile, 0, MAX_TILE_ID) and tile in palette for tile in tiles
    ), "thiếu ô hoặc có tile ngoài bảng cho phép")
    spawns = document["spawns"]
    require(isinstance(spawns, list) and MIN_SPAWNS <= len(spawns) <= MAX_SPAWNS,
            "số điểm xuất phát phải từ 2 đến 8")
    if player_count is not None:
        require(integer(player_count, MIN_SPAWNS, len(spawns)), "không đủ điểm xuất phát cho phòng")

    positions = set()
    for point in spawns:
        require(isinstance(point, list) and len(point) == 2 and
                integer(point[0], 0, width - 1) and integer(point[1], 0, height - 1),
                "tọa độ xuất phát ngoài lưới hoặc sai kiểu")
        x, y = point
        require((x, y) not in positions and palette[tiles[y * width + x]],
                "điểm xuất phát bị trùng hoặc nằm trên ô chặn")
        positions.add((x, y))

    start = tuple(spawns[0])
    seen, pending = {start}, deque([start])
    while pending and not positions <= seen:
        x, y = pending.popleft()
        for nx, ny in ((x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)):
            if (0 <= nx < width and 0 <= ny < height and (nx, ny) not in seen
                    and palette[tiles[ny * width + nx]]):
                seen.add((nx, ny))
                pending.append((nx, ny))
    require(positions <= seen, "không thể đi bộ giữa mọi điểm xuất phát")

    palette_pin = sha256(canonical(sorted(palette.items())))
    pin = sha256(canonical({"map": document, "palette_sha256": palette_pin}))
    if expected_sha256 is not None:
        require(is_sha256(expected_sha256) and expected_sha256 == pin, "hash map không khớp bản đã duyệt")
    return pin


def validate_map_file(path, palette, **options):
    """Read a bounded, operator-selected normalized file, then validate it."""
    with path.open("rb") as handle:
        return validate_payload(handle.read(MAX_BYTES + 1), palette, **options)


def validate_approved_payload(payload, palette, *, approved_sha256, player_count):
    """Require a server-owned pin and roster size before selection or loading."""
    require(is_sha256(approved_sha256), "thiếu SHA-256 đã được server duyệt")
    require(integer(player_count, MIN_SPAWNS, MAX_SPAWNS), "thiếu số người hợp lệ trong phòng")
    return validate_payload(payload, palette, player_count=player_count, expected_sha256=approved_sha256)


def pack_pins(census):
    pins = {}
    for stem in ("FxMap", "fx"):
        pack = census["packs"][stem]
        pins[f"{stem}.idx"] = {"sha256": pack["idx_sha256"]}
        pins[f"{stem}.idd"] = {"bytes": pack["idd_bytes"]}
    pins["FxMap.idd"]["sha256"] = census["packs"]["FxMap"]["idd_sha256"]
    require(all(is_sha256(pin["sha256"]) for pin in pins.values() if "sha256" in pin)
            and all(integer(pin["bytes"], 1, sys.maxsize) for pin in pins.values() if "bytes" in pin),
            "pin pack trong census không hợp lệ")
    return pins


def verify_pack(root, pins):
    for name, pin in pins.items():
        path = root / name
        try:
            info = path.lstat()
            require(stat.S_ISREG(info.st_mode), f"pack không phải tệp thường: {name}")
            if "bytes" in pin:
                require(info.st_size == pin["bytes"], f"sai kích thước pack: {name}")
            if "sha256" in pin:
                digest = hashlib.sha256()
                with path.open("rb") as handle:
                    for chunk in iter(lambda: handle.read(65536), b""):
                        digest.update(chunk)
                require(digest.hexdigest() == pin["sha256"], f"sai SHA-256 pack: {name}")
        except OSError:
            raise AuditError(f"không đọc được pack: {name}") from None


def audited_inventory(census, classification, directive):
    require(census["schema"] == "kre-content-census-v1" and census["build"] == BUILD
            and classification["schema"] == "kre-content-classification-v1"
            and classification["source"] == "kr_static" and classification["build"] == BUILD
            and is_sha256(census["client_exe_sha256"])
            and classification["client_sha256"] == census["client_exe_sha256"],
            "census và classification không cùng nguồn KR 00967")
    require(directive["schema"] == "kre-owner-directive-v1" and directive["authority"] == "project_owner"
            and directive["plan"] == ROADMAP, "chỉ đạo CustomMap không khớp lộ trình")
    reported_grids = tuple((int(w), int(h)) for w, h in re.findall(r"([0-9]+)×([0-9]+)", directive["fact"]))
    require(reported_grids == ALLOWED_GRIDS and "158 mục" in directive["fact"],
            "kiểm kê trong chỉ đạo CustomMap đã đổi; cần đối chiếu lại")

    groups = [row for row in census["resource_groups"] if (row["pack"], row["name"]) == ("fx", "CustomMap")]
    systems = [row for row in census["systems"] if row["name"] == "CustomMap"]
    count = census["packs"]["fx"]["groups"]["CustomMap"]
    require(count == 158 and len(groups) == len(systems) == 1
            and groups[0]["system"] == "CustomMap" and groups[0]["entries"] == systems[0]["entries"] == count
            and systems[0]["groups"] == [RESOURCE_GROUP], "số mục fx/CustomMap không khớp giữa các bảng")
    tables = [row["name"].split("/", 1)[1] for row in census["data_tables"]
              if row["pack"] == "fx" and row["name"].startswith("CustomMap/")]
    require(all(any(name.startswith(prefix) for name in tables) for prefix in RESOURCE_FAMILIES),
            "thiếu nhóm tên bảng CustomMap trong census")

    maps = []
    for row in sorted(census["maps"], key=lambda entry: entry["name"]):
        if not (row["name"].startswith("CustomMap") or row["family"].startswith("CustomMap")):
            continue
        grid = (row["width"], row["height"])
        require(grid in (*ALLOWED_GRIDS, (15, 27), (31, 27)) and all(type(n) is int for n in grid)
                and row["name"] == f"CustomMap{grid[0]}x{grid[1]}" and row["family"] == f"CustomMap{grid[0]}x"
                and row["parse"] == "exact" and row["scripted"] is True and row["spawns"] == 12,
                "dòng map CustomMap trong census đã đổi; cần đối chiếu lại")
        maps.append({"name": row["name"], "family": row["family"], "grid": list(grid),
                     "census_spawn_count": 12, "scripted": True, "grid_allowed_by_design": grid in ALLOWED_GRIDS})
    require({tuple(row["grid"]) for row in maps} == {*ALLOWED_GRIDS, (15, 27), (31, 27)} and len(maps) == 5,
            "không khớp năm map CustomMap trong census")
    families = Counter(row["family"] for row in maps)
    classified = [row for row in classification["rows"] if row["name"].startswith("CustomMap") or row["wave"] == "F7"]
    expected = {(name, "map") for name in families} | {("CustomMap", "system")}
    require(len(classified) == len(expected) and {(row["name"], row["kind"]) for row in classified} == expected
            and all(row["wave"] == "F7" and row["classification"] == "C" and row["status"] == "candidate"
                    and (row["map_count"] == families[row["name"]] if row["kind"] == "map"
                         else row["groups"] == [RESOURCE_GROUP]) for row in classified),
            "xếp loại F7 không khớp các họ map và nhóm tài nguyên")
    return {
        "source": "kr_static", "status": "confirmed", "scope": "committed_pack_census_metadata_only",
        "resource_group": RESOURCE_GROUP, "resource_entries": count, "resource_name_prefixes": RESOURCE_FAMILIES,
        "maps": maps, "map_count": len(maps), "reported_grids": reported_grids,
        "classification": {"grade": "C", "status": "candidate", "map_families": dict(families)},
    }


def build_spec(root):
    data, inputs = {}, {}
    for name, relative in INPUTS.items():
        raw = (root / relative).read_bytes()
        data[name] = json.loads(raw, object_pairs_hook=unique_object)
        inputs[name] = {"path": relative, "sha256": sha256(raw)}
    census, classification = data["census"], data["classification"]
    require(classification["census_sha256"] == inputs["census"]["sha256"], "SHA-256 census không khớp classification")
    spec = {
        "schema": "kre-f7-custom-map-spec-v1", "feature": "F7", "build": BUILD,
        "client_sha256": census["client_exe_sha256"], "inputs": inputs, "roadmap": ROADMAP,
        "inventory": audited_inventory(census, classification, data["owner_directive"]),
        "pack_pins": pack_pins(census),
        "policy": {
            "source": "design", "status": "design", "enabled_by_default": False, "fail_closed": True,
            "allowed_grids": ALLOWED_GRIDS, "max_file_bytes": MAX_BYTES, "max_payload_bytes": MAX_BYTES,
            "normalized_schema": MAP_SCHEMA, "required_fields": sorted(MAP_FIELDS),
            "compression_allowed": False, "executable_content_allowed": False,
            "tiles": {"layout": "row_major", "id_range": [0, MAX_TILE_ID], "trusted_palette_required": True},
            "spawns": {"min": MIN_SPAWNS, "max": MAX_SPAWNS, "coordinates": "zero_based_xy",
                       "unique": True, "walkable_only": True, "reachability": "one_cardinal_walkable_component",
                       "must_cover_active_roster": True},
            "pinning": {"algorithm": "sha256", "canonicalization": "compact_sorted_keys_utf8_trailing_lf",
                        "binds_full_trusted_palette": True, "preserves_tile_and_spawn_order": True,
                        "require_approved_pin_on_share_select_load": True},
            "authority": "server", "validate_before": ["share", "select", "load"],
        },
        "wire": {"status": "unmeasured", "upload": "unmeasured", "selection": "unmeasured",
                 "enabled": False, "synthetic_ack": False, "product_form": "undecided",
                 "evidence_required": ["kr_static_re", "isolated_wine_client_flow"]},
    }
    spec["spec_sha256"] = sha256(canonical(spec))
    return spec


def render_markdown(spec):
    inventory, policy = spec["inventory"], spec["policy"]
    grids = ", ".join(f"`{w}x{h}`" for w, h in policy["allowed_grids"])
    lines = [
        "# F7 — Kiểm kê tĩnh và chính sách map tự tạo",
        "",
        "Sinh bằng `python3 tools/kr_f7_custom_map_audit.py --write`; kiểm từng byte bằng `--check`.",
        "",
        "## Kiểm kê KR 00967",
        "",
        "Nguồn `kr_static`, trạng thái `confirmed` chỉ cho metadata trong [census](census.json). "
        "[Xếp loại](classification.json) giữ các họ map và hệ thống CustomMap ở lớp C, trạng thái `candidate`.",
        f"Census ghi {inventory['resource_entries']} mục trong `{RESOURCE_GROUP}`. Các tên bảng trong nhóm mang "
        "tiền tố `CustomMap`, `CustomTileIndexList` và `CustomTileList`; chưa giải định dạng các bảng này.",
        "",
        "| Map trong `FxMap` | Lưới | Điểm xuất phát theo census | Lưới thuộc danh sách cho phép đề xuất |",
        "|---|---|---:|---|",
    ]
    for row in inventory["maps"]:
        w, h = row["grid"]
        allowed = "Có" if row["grid_allowed_by_design"] else "Không"
        lines.append(f"| `{row['name']}` | `{w}x{h}` | {row['census_spawn_count']} | {allowed} |")
    lines.extend([
        "",
        "[Chỉ đạo CustomMap](../evidence/owner-directives/custom-map.json) và [lộ trình F7](../PLAN.md#full-game-roadmap) "
        f"nêu năm map và ba lưới {grids}. Census vẫn có năm map nhưng có thêm hai cỡ `15x27` và `31x27`. "
        "Đề xuất giữ đúng ba cỡ cho phép; hai cỡ còn lại chỉ ghi nhận trong kiểm kê.",
        "Cả năm dòng census đều có script và ghi 12 điểm xuất phát. Metadata này không xác nhận khả năng chơi, "
        "vị trí hợp lệ của các điểm xuất phát hay số người của phòng. Các map gốc không mặc nhiên đạt validator dưới đây.",
        "",
        "## Chính sách fail-closed đề xuất",
        "",
        "Nguồn và trạng thái đều là `design`. F7 tắt mặc định; các quy tắc này dành cho dữ liệu đã chuẩn hóa của server.",
        "",
        f"- Chỉ nhận lưới {grids} theo thứ tự rộng × cao. File đầu vào và payload chuẩn hóa phải có từ 1 đến "
        f"{policy['max_file_bytes']} byte; chặn đọc quá giới hạn và chặn payload trước khi parse. "
        "Chỉ nhận JSON UTF-8, không giải nén, không nhận khóa lặp hoặc trường lạ và không thực thi script hay nạp tài nguyên do người chơi cung cấp.",
        f"- Schema `{MAP_SCHEMA}` chỉ có `schema`, `width`, `height`, `tiles`, `spawns`. `tiles` phải có đúng "
        f"`width * height` ô theo thứ tự hàng. Mỗi ID là số nguyên trong `0..{MAX_TILE_ID}` và phải nằm trong "
        "bảng tile tin cậy của server kèm cờ đi bộ được. Chưa có bảng được duyệt cho KR thì từ chối.",
        f"- Cần {MIN_SPAWNS}–{MAX_SPAWNS} điểm xuất phát khác nhau, đủ cho số người đang chơi khi chọn/nạp map. "
        "Mỗi điểm là `[x, y]` nguyên, bắt đầu từ 0, nằm trong lưới và trên ô đi bộ được. Dùng đường đi bốn hướng "
        "qua các ô đi bộ được để kiểm mọi điểm xuất phát cùng một vùng liên thông; không tính đi chéo hoặc phá chướng ngại.",
        "- Tính SHA-256 của bảng tile dưới dạng mảng cặp `[id, walkable]` theo ID tăng dần. Sau đó băm đối tượng "
        '`{"map": <map>, "palette_sha256": <hash>}`. Cả hai bước dùng JSON gọn, khóa sắp xếp, UTF-8 và một LF cuối; '
        "giữ thứ tự tile và điểm xuất phát. Thay map hoặc bảng tile làm đổi pin; pin chỉ định danh dữ liệu, không cấp quyền.",
        "- Server kiểm trước khi duyệt hoặc chia sẻ, giữ bản đã duyệt theo hash bất biến và kiểm lại trước khi chia sẻ, "
        "chọn hoặc nạp. Chỉ dùng pin đã duyệt do server giữ, kiểm quyền của người yêu cầu và kiểm lại đủ chỗ theo "
        "roster hiện hành khi chọn/nạp. Thiếu dữ liệu, thiếu quyền, sai hash hoặc vượt giới hạn thì từ chối; "
        "không chia sẻ, không đổi map của phòng và không gửi ACK thành công giả.",
        "",
        "## Wire và cách đo tiếp",
        "",
        "Upload và chọn map trên wire đều là `unmeasured`: chưa biết opcode, layout, định danh hay bên có quyền quyết định "
        "trong KR. Chưa chọn trình dựng trong client, công cụ web hay file chia sẻ. Giữ các route tắt cho tới khi RE tĩnh "
        "xác định đường xử lý và Wine cô lập với stub/server cục bộ đo luồng client từ mở UI đến upload, lựa chọn, "
        "nạp và bắt đầu ván, gồm phản hồi từ chối và trạng thái phòng. Không dùng schema chuẩn hóa làm layout wire KR.",
        "",
        "## Tái lập",
        "",
        "Mặc định chỉ đọc ba nguồn đã commit. [JSON bằng chứng](../evidence/content/f7-custom-map-spec.json) giữ pin "
        "SHA-256 của từng nguồn, client và spec. Có thể thêm `--pack-root <thư-mục-pack>` khi chạy `--write` hoặc `--check` "
        "để so cả hai index cùng các pin của `FxMap.idd`; riêng `fx.idd` chỉ có kích thước trong census. "
        "Kiểm pack là tùy chọn đọc tại chỗ, không thay đầu ra và không xác nhận client thực thi.",
        "",
        f"SHA-256 của spec chuẩn hóa (bỏ trường `spec_sha256`): `{spec['spec_sha256']}`.",
        "",
    ])
    return "\n".join(lines)


def generate(spec):
    return {
        JSON_PATH: (json.dumps(spec, ensure_ascii=False, sort_keys=True, indent=2) + "\n").encode("utf-8"),
        MD_PATH: render_markdown(spec).encode("utf-8"),
    }


def main(argv=None, *, root=ROOT):
    parser = argparse.ArgumentParser(description=__doc__)
    mode = parser.add_mutually_exclusive_group(required=True)
    mode.add_argument("--write", action="store_true", help="ghi JSON và Markdown")
    mode.add_argument("--check", action="store_true", help="so từng byte với bản sinh lại")
    parser.add_argument("--pack-root", type=Path, help="đối chiếu thêm pack cục bộ với pin trong census")
    args = parser.parse_args(argv)
    try:
        root = root.resolve()
        spec = build_spec(root)
        if args.pack_root is not None:
            verify_pack(args.pack_root, spec["pack_pins"])
        outputs = generate(spec)
        for relative, expected in outputs.items():
            path = root / relative
            require(path.resolve() == path and (not path.exists() or
                    (path.is_file() and path.stat().st_nlink == 1)), f"đích phải là tệp thường trong repo: {relative}")
            if args.check:
                require(path.is_file() and path.read_bytes() == expected,
                        f"{relative} không khớp; chạy lại với --write")
        if args.write:
            for relative, expected in outputs.items():
                (root / relative).write_bytes(expected)
    except AuditError as error:
        print(f"F7 CustomMap: {error}", file=sys.stderr)
        return 1
    except (OSError, ValueError, KeyError, TypeError, AttributeError, RecursionError):
        print("F7 CustomMap: thiếu tệp hoặc định dạng nguồn không hợp lệ", file=sys.stderr)
        return 1
    action = "updated" if args.write else "OK"
    local = "; local pack pins OK" if args.pack_root is not None else ""
    print(f"F7 CustomMap: {action} (158 resource entries; 5 census maps; 3 allowed grids{local})")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
