"""Kiểm kê lưới và khóa địa hình F2/F3 từ pack KR 00967 đã pin."""

import argparse
from collections import Counter, defaultdict
import hashlib
import ipaddress
import json
from pathlib import Path
import re
import struct

from export_map_profile import FIELDS, decode_map, load_tile_table
from kr_f1e_script_audit import map_tail_offset, parse_trailer
from kr_pack_export import (BULA_MAPS, BULA_TAIL_EVIDENCE, F1_FIXED_ITEMS, F1_HIDDEN_ITEMS, PACK_SHA256,
                            Pack, f1_terrain_rules, validate_bula_tail, verified_packs)


ROOT = Path(__file__).resolve().parents[1]
EVIDENCE_DIR = Path("docs/evidence/content")
CATALOGS = {"F2": EVIDENCE_DIR / "f2-mode-catalog.json", "F3": EVIDENCE_DIR / "f3-pve-catalog.json"}
JSON_OUT = EVIDENCE_DIR / "f2-f3-tile-grid-census.json"
MARKDOWN_OUT = Path("docs/content/f2-f3-tile-grid-census.md")
PINS = {name: PACK_SHA256[name] for name in ("FxMap.idx", "FxMap.idd", "fx.idx", "fx.idd")}
READERS = ("content_census", "export_map_profile", "kr_f1e_script_audit", "kr_pack_export")
BUILD = "KR00967"
SCHEMA = "kre.f2-f3-tile-grid-census.v1"
STANDARD_GRID = "15x13"
PARSE_ERRORS = (ValueError, struct.error)
PRIORITIES = ("terrain_ready", "quick_unlock", "many_tiles", "terrain_constraints", "needs_layout", "custom_grid")
IDENTIFIER = re.compile(r"[A-Za-z0-9_]+")
HOST_PATH = re.compile(r"(?i)(?:/(?:home|Users|root|mnt|tmp|var)/|(?<!\w)[A-Z]:[\\/]|file:/{2,})")
IP_LITERAL = re.compile(r"(?<![\w.:])"
                        r"(?:(?:[0-9]{1,3}\.){3}[0-9]{1,3}|[0-9A-Fa-f:.]*:[0-9A-Fa-f:.]*:[0-9A-Fa-f:.]*)"
                        r"(?![\w.:])")


class CensusError(ValueError):
    """Thông báo lỗi cố định, không chép nội dung đầu vào."""


def sha256(raw):
    return hashlib.sha256(raw).hexdigest()


def json_text(value, **kwargs):
    return json.dumps(value, ensure_ascii=False, sort_keys=True, allow_nan=False, **kwargs)


def canonical_hash(body):
    return sha256(json_text(body, separators=(",", ":")).encode("utf-8"))


def reject_private(body):
    if HOST_PATH.search(body):
        raise CensusError("đầu ra chứa đường dẫn máy")
    for match in IP_LITERAL.finditer(body):
        try:
            ipaddress.ip_address(match[0])
        except ValueError:
            continue
        raise CensusError("đầu ra chứa địa chỉ IP")


def load_inputs(root):
    catalogs, sources = {}, {}
    for scope, path in CATALOGS.items():
        raw = (root / path).read_bytes()
        sources[path.as_posix()] = sha256(raw)
        catalog = json.loads(raw)
        if catalog["build"] != BUILD:
            raise CensusError("catalog sai build")
        families, seen = {}, set()
        for entry in catalog["families"]:
            name, rows = entry["family"], entry["maps"]
            if scope == "F2":
                names = [row["name"] for row in rows]
            else:
                names = rows["exact"] + rows["partial"]
            if (not names or len(names) != entry["map_count"] or len(set(names)) != len(names)
                    or name in families or seen.intersection(names)
                    or not all(IDENTIFIER.fullmatch(key) for key in [name, *names])):
                raise CensusError(f"phạm vi catalog {scope} không hợp lệ")
            families[name] = sorted(names)
            seen.update(names)
        summary = catalog["summary"]
        if (len(families) != summary["family_count"]
                or len(seen) != summary["total_maps" if scope == "F2" else "map_count"]):
            raise CensusError(f"tổng catalog {scope} không khớp")
        catalogs[scope] = families
    for reader in READERS:
        path = f"tools/{reader}.py"
        sources[path] = sha256((root / path).read_bytes())
    sources[BULA_TAIL_EVIDENCE.as_posix()] = sha256((root / BULA_TAIL_EVIDENCE).read_bytes())
    return catalogs, sources


def recover_grid(raw):
    """Chỉ phục hồi phần trước drop: cùng giới hạn header/lưới với parse_map."""
    if len(raw) < 80:
        return "header", None
    version, _, _, width, height, spawns = struct.unpack_from("<6i", raw)
    if version not in (1000, 1001, 1003) or not (0 < width <= 512 and 0 < height <= 512 and 0 <= spawns <= 64):
        return "header", None
    header = dict(grid=[width, height], version=version)
    offset = 80 + spawns * 8
    if offset + 4 > len(raw):
        return "rectangles", header
    count = struct.unpack_from("<i", raw, offset)[0]
    if not 0 <= count <= 64 or offset + 4 + count * 16 > len(raw):
        return "rectangles", header
    offset += 4
    rectangles = list(struct.iter_unpack("<4i", raw[offset:offset + 16 * count]))
    offset += 16 * count
    stride = 2 if version == 1000 else 3
    end = offset + width * height * stride * 4
    if end > len(raw):
        return "cell_buffer", header
    return None, dict(width=width, height=height, version=version, rectangles=rectangles,
                      spawns=list(struct.iter_unpack("<2i", raw[80:80 + 8 * spawns])),
                      cell_fields=FIELDS[:stride], cells=list(struct.iter_unpack(f"<{stride}i", raw[offset:end])),
                      placements=None, drops=None, scripted=None)


def read_profile(raw, *, name=None):
    """Phân biệt lỗi trước lưới với đuôi chưa được decoder chính chấp nhận."""
    if name in BULA_MAPS:
        profile, tail = validate_bula_tail(raw, name)
        return profile, dict(decode="partial", layout="group_record_tail", tail_status="kr_static/confirmed",
                             tail_offset=tail["tail_offset"], tail_bytes=tail["tail_bytes"],
                             tail_sha256=tail["tail_sha256"], tail_groups=tail["group_count"],
                             tail_records=tail["record_count"], script_length_u32=tail["script_length_u32"])
    try:
        return decode_map(raw), {"decode": "exact", "layout": "length_prefixed_tail"}
    except PARSE_ERRORS:
        layout, grid = recover_grid(raw)
    if layout:
        return None, {"decode": "decode_error", "layout": layout, **(grid or {})}
    try:
        offset = map_tail_offset(raw)
        prefix = decode_map(raw[:offset])
    except PARSE_ERRORS:
        return grid, {"decode": "partial", "layout": "floor_rows_unreadable"}
    prefix["scripted"] = None
    tail = {"decode": "partial", "layout": "unrecognized_tail",
            "tail_offset": offset, "tail_bytes": len(raw) - offset}
    try:
        trailer = parse_trailer(raw, offset)
    except PARSE_ERRORS:
        pass
    else:
        tail.update(layout="group_record_tail", tail_groups=len(trailer["groups"]),
                    tail_records=len(trailer["records"]))
        prefix["scripted"] = b"<" in trailer["script"]
    return prefix, tail


class TileAudit:
    def __init__(self, table):
        self.names, self.tiles = table
        self.table = table
        self.results = {}

    def check_code(self, field, code, width, height):
        cache_key = field, code, width, height
        if cache_key not in self.results:
            self.results[cache_key] = self.probe(*cache_key)
        return self.results[cache_key]

    def probe(self, field, code, width, height):
        if not 0 <= code < len(self.names):
            return f"#{code}", "outside_object_table"
        key = self.names[code]
        if key not in self.tiles:
            return key, "missing_tile_info"
        # Một lớp, một ô đặt tile, giữ nguyên kích thước để kiểm footprint.
        # Bỏ spawn và rectangle để tách luật của tile khỏi ràng buộc cả map.
        cell = [-1, -1, -1]
        cell[FIELDS.index(field)] = code
        profile = dict(name="TileProbe", width=width, height=height, cells=[cell],
                       rectangles=[], spawns=[], payload_sha256="")
        try:
            f1_terrain_rules(profile, self.table, f1e=True)
        except ValueError:
            return key, "outside_f1e_rule"
        return None

    def inspect(self, profile, digest):
        width, height = profile["width"], profile["height"]
        unmapped, accepted = {}, []
        for cell in profile["cells"]:
            filtered = list(cell)
            for index, (field, code) in enumerate(zip(profile["cell_fields"], cell)):
                if code != -1 and (issue := self.check_code(field, code, width, height)):
                    key, reason = issue
                    unmapped.setdefault(key, {})[field] = reason
                    filtered[index] = -1
            accepted.append(filtered)

        def floor_keys(rows, allowlist, field):
            if rows is None:
                return None
            unknown = sorted(set(rows) - allowlist.keys())
            for key in unknown:
                if key in self.tiles:
                    unmapped.setdefault(key, {})[field] = "outside_f1_floor_keys"
            return unknown

        placements = profile["placements"]
        drops = profile["drops"]
        constraints = []
        if profile["rectangles"]:
            constraints.append("rectangles")
        spawns = profile["spawns"]
        if (len(set(map(tuple, spawns))) != len(spawns)
                or any(not (0 <= x < width and 0 <= y < height) for x, y in spawns)):
            constraints.append("spawn_coordinates")
            spawns = []
        try:
            f1_terrain_rules(dict(profile, name="MapTerrain", payload_sha256=digest, cells=accepted,
                                  rectangles=[], spawns=spawns), self.table, f1e=True)
        except ValueError:
            constraints.append("combined_terrain")
        missing_placements = floor_keys(None if placements is None else (p["name"] for p in placements),
                                        F1_FIXED_ITEMS, "placement")
        missing_drops = floor_keys(None if drops is None else (name for name, weight in drops if weight > 0),
                                   F1_HIDDEN_ITEMS, "drop")
        return dict(unmapped_tiles=unmapped, unmapped_placements=missing_placements,
                    unmapped_positive_drops=missing_drops, terrain_constraints=constraints)


def scan_map(raw, audit, *, name=None):
    digest = sha256(raw)
    profile, row = read_profile(raw, name=name)
    row["payload_sha256"] = digest
    if profile is not None:
        row.update(grid=[profile["width"], profile["height"]], version=profile["version"],
                   scripted=profile["scripted"], **audit.inspect(profile, digest))
    return row


def remaining_tile_roles(maps):
    roles = defaultdict(set)
    for row in maps.values():
        for key, fields in row.get("unmapped_tiles", {}).items():
            roles[key].update(set(fields).intersection(FIELDS))
    return {key: sorted(fields) for key, fields in roles.items() if fields}


def audit_remaining_tiles(maps, audit, tile_info_sha256):
    """Đối chiếu đủ 18 số với các khóa đã ánh xạ ở cùng lớp trong lưới chuẩn."""
    roles = remaining_tile_roles(maps)
    records, matches = {}, {}
    by_record = defaultdict(list)
    for code, key in enumerate(audit.names):
        if key in audit.tiles:
            by_record[audit.tiles[key]].append(code)

    for key, fields in sorted(roles.items()):
        data = audit.tiles.get(key)
        records[key] = dict(tile_info=data, roles=fields)
        if data is None:
            continue
        for field in fields:
            peers = sorted({audit.names[code] for code in by_record[data]
                            if audit.names[code] != key and field not in roles.get(audit.names[code], ())
                            and audit.check_code(field, code, 15, 13) is None})
            if peers:
                matches.setdefault(key, {})[field] = peers
    return dict(tile_info_sha256=tile_info_sha256, distinct_keys=len(roles),
                record_set_sha256=canonical_hash(records), records=records,
                mapped_exact_matches=matches)


def map_names(catalogs):
    return {name for families in catalogs.values() for names in families.values() for name in names}


def scan_packs(game_dir, catalogs):
    # verified_packs chỉ nêu tên pack đã pin, không đưa đường dẫn do người dùng nhập.
    try:
        return _scan_packs(game_dir, catalogs)
    except ValueError as exc:
        if str(exc).startswith(("Thiếu pack KR 00967:", "Pack không khớp KR 00967 (SHA-256):")):
            raise CensusError(str(exc)) from None
        raise


def _scan_packs(game_dir, catalogs):
    with verified_packs(game_dir, PINS) as files:
        maps = Pack(files["FxMap.idx"], files["FxMap.idd"])
        fx = Pack(files["fx.idx"], files["fx.idd"])
        audit = TileAudit(load_tile_table(fx))
        output = {}
        for name in sorted(map_names(catalogs)):
            try:
                raw = maps.read(f"Map/{name}")
            except (KeyError, ValueError):
                row = {"decode": "decode_error", "layout": "pack_entry"}
            else:
                row = scan_map(raw, audit, name=name)
            output[name] = row
        return output, audit_remaining_tiles(output, audit, sha256(fx.read("TileInfo/TileInfoList")))


def grid_name(row):
    return "x".join(map(str, row["grid"])) if row.get("grid") else "unknown"


def family_summary(family, names, maps):
    rows = [maps[name] for name in names]
    tiles = sorted({key for row in rows for key in row.get("unmapped_tiles", {})})
    grids = Counter(grid_name(row) for row in rows)
    decode = Counter(row["decode"] for row in rows)
    constraints = sorted({issue for row in rows for issue in row.get("terrain_constraints", [])})
    if grids.keys() - {STANDARD_GRID, "unknown"}:
        priority = "custom_grid"
    elif decode.keys() != {"exact"} or grids.keys() != {STANDARD_GRID}:
        priority = "needs_layout"
    elif constraints or any(key.startswith("#") for key in tiles):
        priority = "terrain_constraints"
    elif not tiles:
        priority = "terrain_ready"
    elif len(tiles) <= 3:
        priority = "quick_unlock"
    else:
        priority = "many_tiles"
    return dict(family=family, maps=names, grids=grids, decode_counts=decode, unmapped_tiles=tiles,
                priority=priority, terrain_constraints=constraints,
                unmapped_placements=sorted({key for row in rows for key in row.get("unmapped_placements") or []}),
                unmapped_positive_drops=sorted({key for row in rows for key in row.get("unmapped_positive_drops") or []}))


def build_body(catalogs, sources, maps, *, remaining_tiles=None):
    if maps.keys() != map_names(catalogs):
        raise CensusError("thiếu hoặc thừa map so với hai catalog")
    families, summaries = {}, {}
    for scope, groups in catalogs.items():
        rows = [family_summary(family, names, maps) for family, names in groups.items()]
        rows.sort(key=lambda row: (PRIORITIES.index(row["priority"]), len(row["unmapped_tiles"]), row["family"]))
        families[scope] = rows
        scans = [maps[name] for names in groups.values() for name in names]
        summaries[scope] = dict(maps=len(scans), families=len(rows),
                                decode_counts=Counter(row["decode"] for row in scans),
                                grids=Counter(grid_name(row) for row in scans),
                                unmapped_tile_types=len({key for row in rows for key in row["unmapped_tiles"]}),
                                priorities=Counter(row["priority"] for row in rows))
    body = dict(schema=SCHEMA, build=BUILD, source="kr_static", status="confirmed", ranking_status="candidate",
                input_sha256=sources, pack_sha256=PINS, maps=maps, families=families, summary=summaries)
    if remaining_tiles is not None:
        roles = remaining_tile_roles(maps)
        records = remaining_tiles["records"]
        if (remaining_tiles["distinct_keys"] != len(roles)
                or records.keys() != roles.keys()
                or remaining_tiles["record_set_sha256"] != canonical_hash(records)
                or any(key not in roles or set(fields) - set(roles[key])
                       for key, fields in remaining_tiles["mapped_exact_matches"].items())):
            raise CensusError("đối chiếu TileInfo không khớp các khóa còn thiếu")
        for key, row in records.items():
            data = row["tile_info"]
            if row["roles"] != roles[key] or (data is not None and (
                    len(data) != 18 or any(type(word) is not int for word in data))):
                raise CensusError("bản ghi TileInfo hoặc lớp ô không hợp lệ")
        body["remaining_tile_audit"] = remaining_tiles
    return body


def render_json(body):
    """Mỗi map hoặc họ một dòng để diff giữ đúng phạm vi thay đổi."""
    def render_rows(rows, indent):
        entries = [f'{indent}  {json_text(key)}: {json_text(row)}' for key, row in sorted(rows.items())]
        return "{\n" + ",\n".join(entries) + "\n" + indent + "}" if entries else "{}"

    output = dict(body, body_sha256=canonical_hash(body))
    fields = []
    for key, value in sorted(output.items()):
        if key == "maps":
            rendered = render_rows(value, "  ")
        elif key == "remaining_tile_audit":
            entries = [f"    {json_text(k)}: " + (render_rows(v, "    ") if k == "records" else json_text(v))
                       for k, v in sorted(value.items())]
            rendered = "{\n" + ",\n".join(entries) + "\n  }"
        elif key == "families":
            scopes = []
            for scope, rows in sorted(value.items()):
                items = ",\n".join(f"      {json_text(row)}" for row in rows)
                scopes.append(f"    {json_text(scope)}: [\n{items}\n    ]")
            rendered = "{\n" + ",\n".join(scopes) + "\n  }"
        else:
            rendered = json_text(value, indent=2).replace("\n", "\n  ")
        fields.append(f"  {json_text(key)}: {rendered}")
    return "{\n" + ",\n".join(fields) + "\n}\n"


def render_markdown(body):
    f2, f3 = (body["summary"][scope]["maps"] for scope in CATALOGS)
    overlap = f2 + f3 - len(body["maps"])
    lines = [
        "# Kiểm kê lưới và tile chưa ánh xạ của F2/F3",
        "",
        f"[JSON kiểm kê](../evidence/content/{JSON_OUT.name}) ghi kết quả `kr_static` của build `{BUILD}` "
        f"cho {len(body['maps'])} map riêng biệt; hai catalog có {overlap} map chung.",
        "Nguồn phạm vi là [F2](f2-mode-catalog.md) và [F3](f3-pve-catalog.md). "
        "Mỗi map có bản ghi riêng; mỗi họ giữ hợp các khóa còn thiếu từ mọi map của họ.",
        "",
        "| Catalog | Map | Họ | Giải mã đủ | Đọc một phần | Lỗi giải mã | Lưới khác 15x13 | Loại tile còn thiếu |",
        "| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |",
    ]
    for scope, summary in body["summary"].items():
        counts = summary["decode_counts"]
        custom = sum(n for grid, n in summary["grids"].items() if grid not in (STANDARD_GRID, "unknown"))
        lines.append(f"| {scope} | {summary['maps']} | {summary['families']} | {counts.get('exact', 0)} | "
                     f"{counts.get('partial', 0)} | {counts.get('decode_error', 0)} | {custom} | "
                     f"{summary['unmapped_tile_types']} |")
    lines += ["", "## Lưới và đuôi map", "",
              "| Lưới | F2 | F3 |", "| --- | ---: | ---: |"]
    grids = {grid for s in body["summary"].values() for grid in s["grids"]}
    for grid in sorted(grids, key=lambda g: tuple(map(int, g.split("x"))) if g != "unknown" else (0, 0)):
        f2, f3 = (body["summary"][scope]["grids"].get(grid, 0) for scope in CATALOGS)
        lines.append(f"| `{grid}` | {f2} | {f3} |")
    lines += ["", f"Kiểu đuôi chưa được `decode_map` đọc đủ: {_partial_layouts(body['maps'])}.",
              "Với đuôi khác bố cục thường, công cụ giữ lưới và chỉ giữ các hàng vật phẩm nếu đọc trọn phần đó. "
              "Bộ đọc đuôi nhóm/bản ghi chỉ xác nhận cấu trúc; chưa xác nhận tác dụng các bản ghi.",
              "Cả bảy map `Bula` khớp từng byte với [bằng chứng P228](../protocol/f2-bula-tail-and-mode-wire-re.md). "
              "Có thể xem JSON bằng `python3 tools/kr_pack_export.py --game-dir <thu-muc-pack> --inspect-bula Bula00`. "
              "Census vẫn ghi `partial` theo bộ đọc thường; các adapter phòng chưa nhận `Bula`.",
              "",
              "## Họ ưu tiên theo số tile còn thiếu", "",
              "Chỉ xếp `quick_unlock` khi mọi map trong họ đều giải mã đủ, có lưới 15x13, "
              "có từ một đến ba khóa tile khác nhau và không có lỗi ràng buộc địa hình đã thấy. "
              "Các họ được xếp theo số tile rồi theo tên trong mỗi nhóm ưu tiên.",
              "",
              "| Catalog | Địa hình đủ | Ưu tiên | Trên ba tile | Ràng buộc địa hình | Thiếu bố cục | Lưới khác |",
              "| --- | ---: | ---: | ---: | ---: | ---: | ---: |"]
    for scope, summary in body["summary"].items():
        counts = " | ".join(str(summary["priorities"].get(key, 0)) for key in PRIORITIES)
        lines.append(f"| {scope} | {counts} |")
    lines += ["", "| Catalog | Họ ưu tiên | Map | Tile còn thiếu | Khóa đặt sẵn / drop dương còn thiếu |",
              "| --- | --- | ---: | --- | ---: |"]
    for scope, rows in body["families"].items():
        for row in rows:
            if row["priority"] == "quick_unlock":
                keys = ", ".join(f"`{key}`" for key in row["unmapped_tiles"])
                floor = f"{len(row['unmapped_placements'])} / {len(row['unmapped_positive_drops'])}"
                lines.append(f"| {scope} | `{row['family']}` | {len(row['maps'])} | {keys} | {floor} |")
    lines += [
        "",
        "Mỗi mã ô được nối từ `ObjectTable/ObjectTable` sang `TileInfo/TileInfoList` trong pack `fx`, "
        "rồi kiểm riêng theo lớp nền, vật thể hoặc overlay bằng `f1_terrain_rules(f1e=True)`. "
        "Công cụ cũng thử các lớp đã ánh xạ cùng nhau để phát hiện ràng buộc như footprint chồng nhau.",
        "Khóa đặt sẵn và drop có trọng số dương được đối chiếu riêng với danh sách khóa F1. "
        "Khóa vật phẩm không tính vào số tile; một khóa đặt sẵn hoặc drop trùng tên trong TileInfo được ghi thêm theo lớp tương ứng.",
    ]
    if (audit := body.get("remaining_tile_audit")) is not None:
        lines += [
            "",
            f"Đối chiếu KR 00967 xét đủ 18 số của {audit['distinct_keys']} khóa còn thiếu trong hai catalog. "
            f"Có {len(audit['mapped_exact_matches'])} khóa trùng tuple với khóa đã ánh xạ ở cùng lớp trên lưới 15x13. "
            "JSON giữ các tuple theo khóa và lớp cùng SHA-256 của tập này và của `TileInfo/TileInfoList`.",
            "Nền Baseball có cờ lap; nền, ô tăng tốc và lốp của Kart có cờ hoặc marker riêng. "
            "`Temple08` khác `Temple11` ở cờ +40, còn `Tower00` có marker +36 bằng 999. "
            "Các tile SnowBear và Tornado vẫn chờ xác nhận hành vi riêng. Không mở thêm tile trong lượt kiểm này.",
        ]
    lines += [
        "",
        "Xếp hạng là `candidate` cho công việc về địa hình. "
        "Điều kiện phục vụ như vị trí, xác suất vật phẩm, allowlist script, chế độ phòng và quyền PvE chưa được xác nhận ở đây. "
        "Ví dụ, `MonS2Fire03` và `RPG11_N_02` đọc được với lưới lớn; `Tutorial01` đọc được với lưới 15x13. "
        "Những kết quả đó mô tả dữ liệu pack; các giới hạn phục vụ vẫn theo [probe Wine r1](f2-f3-wine-probe-r1.md).",
        "",
        "Sinh lại từ pack gốc: `python3 tools/kr_f2_f3_tile_census.py --write` (có thể chỉ định `--game-dir`). "
        "Kiểm hash thân JSON, phạm vi hai catalog và định dạng cả hai đầu ra: "
        "`python3 tools/kr_f2_f3_tile_census.py --check`. Lệnh kiểm chỉ đọc tệp trong repo.",
    ]
    return "\n".join(lines) + "\n"


def _partial_layouts(maps):
    counts = Counter(row["layout"] for row in maps.values() if row["decode"] == "partial")
    return ", ".join(f"`{name}` ({count} map)" for name, count in sorted(counts.items())) or "không có"


def default_game_dir():
    return Path.home() / "boom" / "clients" / "kr-00967"


def main(argv=None, *, root=ROOT):
    parser = argparse.ArgumentParser(description=__doc__)
    mode = parser.add_mutually_exclusive_group(required=True)
    mode.add_argument("--write", action="store_true", help="quét pack KR 00967 và sinh hai đầu ra")
    mode.add_argument("--check", action="store_true", help="kiểm hai đầu ra mà không đọc pack")
    parser.add_argument("--game-dir", type=Path, help="thư mục pack, chỉ dùng với --write")
    args = parser.parse_args(argv)
    if args.check and args.game_dir is not None:
        parser.error("--game-dir chỉ dùng với --write")
    try:
        catalogs, sources = load_inputs(root)
        if args.write:
            maps, remaining_tiles = scan_packs(
                args.game_dir if args.game_dir is not None else default_game_dir(), catalogs)
        else:
            saved = (root / JSON_OUT).read_text(encoding="utf-8")
            reject_private(saved)
            stored = json.loads(saved)
            if stored.pop("body_sha256") != canonical_hash(stored):
                raise CensusError("SHA-256 của thân JSON không khớp")
            maps = stored["maps"]
            remaining_tiles = stored["remaining_tile_audit"]
        body = build_body(catalogs, sources, maps, remaining_tiles=remaining_tiles)
        outputs = {JSON_OUT: render_json(body), MARKDOWN_OUT: render_markdown(body)}
        for path, content in outputs.items():
            reject_private(content)
            target = root / path
            if args.write:
                if target.is_symlink() or (target.exists() and (not target.is_file() or target.stat().st_nlink != 1)):
                    raise CensusError(f"{path} phải là tệp thường")
            else:
                actual = target.read_bytes()
                reject_private(actual.decode("utf-8"))
                if actual != content.encode("utf-8"):
                    raise CensusError(f"{path} chưa đồng bộ; chạy lại với --write")
        if args.write:
            for path, content in outputs.items():
                (root / path).write_bytes(content.encode("utf-8"))
    except OSError:
        parser.exit(1, "F2/F3 tile census: không đọc hoặc ghi được tệp đầu vào/đầu ra.\n")
    except CensusError as exc:
        parser.exit(1, f"F2/F3 tile census: {exc}\n")
    except (KeyError, TypeError, ValueError, struct.error):
        parser.exit(1, "F2/F3 tile census: dữ liệu đầu vào không hợp lệ.\n")
    summaries = body["summary"]
    scopes = ", ".join(f"{s}: {row['maps']} map / {row['families']} họ" for s, row in summaries.items())
    counts = Counter(row["decode"] for row in maps.values())
    print(f"F2/F3 tile census: {scopes}; {len(maps)} map riêng biệt "
          f"({counts['exact']} đủ, {counts['partial']} một phần, {counts['decode_error']} lỗi); "
          f"{'đã ghi' if args.write else 'đã kiểm'} 2 tệp.")


if __name__ == "__main__":
    main()
