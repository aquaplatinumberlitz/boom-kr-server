"""Generate the KR00967 step 0 classification without local client assets."""
import argparse
from collections import Counter, defaultdict
import hashlib
import json
from pathlib import Path

from content_census import MODE_ROWS, base_theme, item_keys

ROOT = Path(__file__).resolve().parents[1]
CONTENT = ROOT / "docs/content"
CENSUS = CONTENT / "census.json"
G1 = "../evidence/rust-service/g1-acceptance-matrix.json"
MAPS = {"Forest07", "Patrit14", "Village10", "Camp08", "Tomb12", "Factory07"}
BASIC_ITEMS = {"Item/ItemBomb", "Item/ItemFire", "Item/ItemRoller", "Item/ItemMaxPower"}
ACTION_ITEMS = {"Item/ItemShoe", "Item/ItemPunch", "Item/ItemNiddle"}
F4 = {
    "Cửa hàng", "Level", "Bộ sưu tập item", "Hồ sơ / MyInfo", "Chọn nhân vật",
    "Trang trí avatar", "Trang trí nhân vật", "Trang trí bom", "Trang trí cấp",
    "Cảm xúc / Emoticon", "Trang trí ID",
}
F5 = {"Bang hội", "Bạn bè / Messenger", "Xếp hạng CAJJang", "Chat", "Couple"}
F3 = {"Monster", "Hidden Catch", "RPG", "CAHero", "SandBox", "AIBattle", "Hướng dẫn",
      "IamBoss", "IamWriggle"}
F2 = {"Ladder", "ConfusedFight", "Antarctic", "SnowyChamp", "Ghép trận UXMatching"}
F1 = {"Nhân vật", "Item", "Map", "Ô map", "Trận đấu"}
BASE = {"Sảnh", "Đăng nhập", "Kết quả"}
F6 = {
    "Nhiệm vụ", "Sự kiện theo tiền tố Event", "Exchange", "Sự kiện TakeCareOfEvent",
    "CAPass", "MagicMarket", "HoneyButterChip", "DIYBomb", "MineExplore", "LuckyCard",
    "Pet", "Câu cá", "GrowSprout", "DTT", "Partners", "LineFriends", "MonWars",
    "RetroWriggle", "SelectionBox",
}
F1_C = "Chưa biết quyền chạy script, kiểu ô, item hoặc kích thước chưa có profile."
F1_B = "Thử ánh xạ mã map, spawn, ô và item bằng adapter G1; cần đối chiếu hành vi chưa có."
UNKNOWN = "Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967."
SPECIAL = {
    "Level": ("B", "Dùng hồ sơ M6 và bảng kinh nghiệm tham chiếu; cần chính sách thưởng và kiểm trường hiển thị."),
    "Chat": ("B", "Chat sảnh/phòng đã đo cho L1; whisper, bạn bè và giới hạn lạm dụng chưa đo."),
    "Hồ sơ / MyInfo": ("B", "Có projection tài khoản M5/M6; chưa đầy đủ profile và dữ liệu túi đồ."),
    "Chọn nhân vật": ("B", "Có chọn và lưu cho nhân vật đã cho phép; quyền sở hữu/chỉ số mở rộng còn thiếu."),
    "Nhân vật": ("B", "Có Room/Battle selection và chỉ số nền; đối chiếu key và cap cho từng nhóm."),
    "Item": ("C", "Tách theo nhóm ItemAttr; có lát A/B cho item nền đã đo, phần còn lại chưa biết."),
    "Map": ("C", "Tách theo họ; profile thường dùng adapter G1, script và nhóm đặc biệt chưa biết."),
    "Ô map": ("C", "Đã biết một số movement/ray flags; cần biết ý nghĩa các ô chưa đo."),
    "Trận đấu": ("A", "Lõi RoomRules trận thường đã chạy trong phạm vi G1."),
    "Sảnh": ("A", "Đã có luồng vào sảnh/phòng qua TLS; Square và các màn bên chưa phân loại."),
    "Đăng nhập": ("A", "Đã có companion TLS và admission tài khoản nội bộ, không gồm NGM."),
    "Kết quả": ("A", "Kết quả/journal trận thường đã có, thưởng hiện là 0."),
    "MagicMarket": ("C", "Pack có UI; boomkr-re không tìm thấy công thức trong phạm vi khảo sát."),
    "Cửa hàng": ("C", "Chưa xác định hợp đồng túi đồ và giao dịch; màn nhúng dựng bằng web server."),
    "PCRoomBenefit": (None, "Ngoài phạm vi theo quyết định bỏ ưu đãi Nexon."),
    "CustomMap": ("C", "Chưa xác định đợt; chưa biết upload, validation và quyền điều khiển."),
}
ORDER = ["map", "item", "ship", "character", "mode", "system"]
UNSUPPORTED_WAVE = "chưa xếp đợt"


def generate(data, digest):
    rows = []
    def add(kind, name, wave, code, reason, **evidence):
        rows.append(dict(kind=kind, name=name, wave=wave, classification=code,
                         status="design" if code is None else "candidate", reason=reason, **evidence))

    by_family = defaultdict(list)
    f1_used = set()
    for m in data["maps"]:
        by_family[m["category"], m["family"]].append(m)
    for (category, family), maps in sorted(by_family.items()):
        base = base_theme(family) or family
        f1 = category in {"battle_theme", "battle_variant"} and not family.startswith("Fm_AIBtl")
        wave = ("F1" if f1 else "F2?" if category == "mission" else "F3"
                if category in {"monster", "rpg", "hidden", "bonus"} or
                family.startswith("Fm_AIBtl") or family == "Tutorial" else UNSUPPORTED_WAVE)
        native = sorted(m["name"] for m in maps if m["name"] in MAPS)
        substantial = [m for m in maps if set(m["script_tags"]) - {"mapscript", "StageClearScript"}]
        unusual = any((m["width"], m["height"]) != (15, 13) for m in maps)
        code, reason = ("C", UNKNOWN)
        if f1:
            for m in maps:
                f1_used.update(m["placements"])
                f1_used.update(name for name, weight in m["drops"] if weight > 0)
            code, reason = ("C", F1_C) if substantial or unusual else ("B", F1_B)
            if native and len(native) == len(maps):
                code, reason = "A", "Profile và route có native G1; giữ giới hạn trong ma trận."
        extra = {}
        if f1:
            candidates = sorted((m for m in maps if m["name"] not in MAPS), key=lambda m: (
                m["scripted"] if code != "C" else not m["scripted"],
                (m["width"], m["height"]) != (15, 13), m["name"]))
            extra["probe"] = {"sample": candidates[0]["name"] if candidates else native[0],
                              "status": "pending" if candidates else "prior_g1"}
        add("map", family, wave, code, reason, category=category, base=base,
            map_count=len(maps), script_count=sum(m["scripted"] for m in maps),
            script_unknown=len(substantial), native_g1=native, **extra)

    by_group = defaultdict(list)
    lookup = {}
    ship_keys = {key for ship in data["ships"] for key in item_keys(ship["name"])}
    for i in data["items"]:
        by_group[i["group"]].append(i)
        for key in item_keys(i["name"]) | {i["sprite"].lower()}:
            lookup.setdefault(key, i["name"])
    f1_items = set()
    for name in sorted(f1_used):
        item = lookup.get(name.lower()) or next((lookup[key] for key in
                                                 sorted(item_keys(name)) if key in lookup), None)
        if item is not None:
            f1_items.add(item)
        elif not item_keys(name) & ship_keys:
            add("item", name, "F1?", "C", "Tên được map F1 dùng nhưng chưa ghép được ItemAttr.", unmatched=True)
    for group, items in sorted(by_group.items()):
        names = {i["name"] for i in items}
        for selected, suffix, code, reason in [
            (names & BASIC_ITEMS, " · item nền", "A", "Có pickup và stat trong phạm vi G1."),
            (names & ACTION_ITEMS, " · thao tác", "B", "Có wire và luật mẫu G1; cần đối chiếu biến thể và điều kiện."),
            (names - BASIC_ITEMS - ACTION_ITEMS, " · còn lại" if names & (BASIC_ITEMS | ACTION_ITEMS) else "",
             "C", "Chưa biết mapping, luật hoặc thẩm quyền cho các mục còn lại."),
        ]:
            if selected:
                add("item", str(group) + suffix, "F1" if selected & f1_items else "F2–F6?",
                    code, reason, entries=sum(i["name"] in selected for i in items),
                    names=sorted(selected), f1_items=sorted(selected & f1_items),
                    map_candidates=sorted({i["name"] for i in items if i["name"] in selected and
                                           (i["maps_drop"] or i["maps_placed"])}))

    # Pack keys can repeat. Preserve the row number rather than equating them.
    for i in data["ships"]:
        add("ship", i["name"].removeprefix("Ship/"), "F1", "C",
            "Chưa có wire lên/xuống và quyền cập nhật tốc độ/va chạm của phương tiện.", row=i["seq"])
    for row, i in enumerate(data["characters"], 1):
        baseline = i["name"] in {"C00", "C01"}
        add("character", i["name"], "F1", "A" if baseline else "B" if i["name"][0] == "C" else "C",
            "Dùng profile nhân vật G1." if baseline else
            "Đối chiếu key Room/Battle, cap và quyền chọn; không suy key từ tên." if i["name"][0] == "C"
            else "Chưa rõ key, quyền chọn hoặc modifier ngoài nhóm C.", row=row)

    extra_modes = [("Free 1v7", "F2"), ("Hướng dẫn", "F3"), ("Màn thưởng", "F3"),
                   ("Monster mùa 2", "F3"), ("Nhiệm vụ chơi với máy", "F3")]
    for mode in MODE_ROWS + [dict(name=name, wave=wave) for name, wave in extra_modes]:
        normal = mode["wave"] == "F1"
        add("mode", mode["name"], mode["wave"], "A" if normal else "C",
            "RoomRules hiện chạy lát trận thường G1." if normal else
            "Lượt M4 âm; không retry ở bước 0, chưa biết khác biệt luật gốc." if mode["name"] == "Free 1v7"
            else UNKNOWN, wiki=mode.get("wiki", ""))
    for i in data["systems"]:
        name = i["name"]
        wave = next((wave for wave, names in [
            ("F1", F1), ("F2", F2), ("F3", F3), ("F4", F4), ("F5", F5), ("F6", F6),
            ("giai đoạn 1", BASE), ("ngoài phạm vi", {"PCRoomBenefit"}),
        ] if name in names), UNSUPPORTED_WAVE)
        code, reason = SPECIAL.get(name, ("C", UNKNOWN))
        add("system", name, wave, code, reason, groups=i["groups"], wiki=i["wiki"])
    for name, wave, reason in [
        ("Túi đồ", "F4", "Chưa có ledger, projection túi đồ và wire đồng bộ quyền sở hữu."),
        ("Web nhúng: Dwarf/libcef", "F4/F6", "Đã chốt dựng web ở server; chưa biết hợp đồng từng màn."),
    ]:
        add("system", name, wave, "C", reason, roadmap_only=True)
    return dict(schema="kre-content-classification-v1", build=data["build"],
                source="kr_static", census_sha256=digest, client_sha256=data["client_exe_sha256"],
                idb_sha256="6be15a1fccea0837ee95c50893345e9f13dc217b0fa570975b5c284f0d547fd1",
                ida_input_sha256="cf72da112308d9f1ca256df8089594c5f787875310933073bf6bd5ba6f70e63b",
                rows=rows)


def markdown(data):
    rows = data["rows"]
    lines = [
        "# Xếp loại nội dung KR 00967",
        "",
        "Bảng sinh từ [kiểm kê pack](../CONTENT.md). Các lớp là ứng viên kỹ thuật, không phải mức tương thích",
        "của cả họ. **A**: server đã có hoặc gần đủ với dữ liệu; **B**: cần thêm wire hoặc luật đã hiểu;",
        "**C**: còn ẩn số cần RE. Dấu **—** là ngoài phạm vi. F1–F6 vẫn chưa mở.",
        "",
        "Nguồn: số liệu pack và wiki được định tuyến ở CONTENT; đối chiếu `boomkr-re/docs/kr-data.md` §2, §6, §12–13",
        "và `boom-re/docs/items.md`, `map.md` để định hướng, không nhập opcode/ID VN. IDA trên bản sao unpack 00967",
        "(hash input/DB trong JSON) có xref `Attr/ItemAttr` tại VA `0x1196ABF`, `Attr/BomberAttr` tại",
        "VA `0x13501B6`; đây chỉ là dấu vết reader tĩnh, không xác định wire, drop hoặc quyền server.",
        "Tên trùng trong bảng Attr/Ship được giữ theo số dòng; wiki chỉ giúp ghép tên.",
        "Lớp B của nhân vật dựa trên layout và luồng chọn đã biết; giá trị/cap từng dòng vẫn cần đối chiếu KR.",
        "",
        "| Nhóm | A | B | C | Ngoài |",
        "|---|---:|---:|---:|---:|",
    ]
    titles = dict(zip(ORDER, ("Họ map", "Nhóm item", "Phương tiện", "Nhân vật", "Mode", "Hệ thống"), strict=True))
    for kind, title in titles.items():
        n = Counter(r["classification"] for r in rows if r["kind"] == kind)
        lines.append(f"| {title} | {n['A']} | {n['B']} | {n['C']} | {n[None]} |")
    lines.extend([
        "",
        f"Họ map F1: {sum(r['kind'] == 'map' and r['wave'] == 'F1' and r['classification'] == 'B' for r in rows)} B, "
        f"{sum(r['kind'] == 'map' and r['wave'] == 'F1' and r['classification'] == 'C' for r in rows)} C.",
        "",
        "### Phép thử rẻ cho F1",
        "",
        "Cột Mẫu chỉ định một map mỗi họ và chưa có lượt native mới. Ưu tiên họ không script hoặc cùng adapter;",
        "đo hai client vào cùng map, nhận Start/Load và journal, một tương tác ô hoặc item rồi về phòng. Dừng",
        "ca khi lỗi chọn map, thiếu profile hay không có mốc bắt đầu trong vài phút. Ca C có script chỉ đo",
        "load/trigger tối thiểu khi profile phù hợp. Gộp map trong cùng lượt; trần bốn lượt, các ca còn lại giữ",
        f"`pending`. Route cũ chỉ được dùng theo phạm vi [ma trận G1]({G1}).",
    ])
    for kind, title in titles.items():
        lines.extend(["", f"## {title}", "",
                      "| Nội dung | Đợt | Lớp | Số lượng / mẫu | Server cần làm |",
                      "|---|---|---|---|---|"])
        items = sorted((r for r in rows if r["kind"] == kind),
                       key=lambda r: r.get("category", "")) if kind == "map" else (
                           r for r in rows if r["kind"] == kind)
        for row in items:
            count = row.get("entries", row.get("map_count", row.get("row", "—")))
            if "probe" in row:
                count = f'{count} · `{row["probe"]["sample"]}`'
            if row.get("native_g1"):
                count = f'{count}; G1: {", ".join(row["native_g1"])}'
            lines.append(f'| {row["name"]} | {row["wave"]} | {row["classification"] or "—"} | {count} | {row["reason"]} |')
    return "\n".join(lines) + "\n"


def main():
    p = argparse.ArgumentParser(description=__doc__)
    mode = p.add_mutually_exclusive_group(required=True)
    mode.add_argument("--generate", action="store_true")
    mode.add_argument("--check", action="store_true")
    args = p.parse_args()
    source = CENSUS.read_bytes()
    classification = generate(json.loads(source), hashlib.sha256(source).hexdigest())
    meta = {k: v for k, v in classification.items() if k != "rows"}
    rendered = json.dumps(meta, ensure_ascii=False, indent=2)[:-2] + ',\n  "rows": [\n' + ",\n".join(
        "    " + json.dumps(row, ensure_ascii=False) for row in classification["rows"]
    ) + "\n  ]\n}\n"
    for name, artifact in [("classification.json", rendered), ("classification.md", markdown(classification))]:
        path = CONTENT / name
        if args.generate:
            path.write_text(artifact, encoding="utf-8")
        elif path.read_text(encoding="utf-8") != artifact:
            p.error(f"{name} chưa cập nhật; chạy --generate")
    print(f"KR 00967: {len(classification['rows'])} dòng xếp loại.")


if __name__ == "__main__":
    main()
