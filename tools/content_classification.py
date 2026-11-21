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
NATIVE_F1 = {
    "Factory": {
        "map": "Factory07", "source": "kr_runtime", "status": "confirmed",
        "scope": "Hai client Wine đặt bom, nhận kết quả và về phòng; journal complete.",
        "checkpoint": "CP381",
    },
    "Patrit": {
        "map": "Patrit01", "source": "kr_runtime", "status": "confirmed",
        "scope": "Có tên trong menu map; chưa nhận Start/Load native cho Patrit01.",
        "checkpoint": "CP381",
    },
}
NATIVE_BATCH_CHECKPOINT = "CP382"
# These names are exact pack keys, never a family-name match.
NATIVE_BATCH_A = {
    "Camp01", "Cemetery01", "Cookie01", "Desert00", "Factory07",
    "Forest02", "Ice00", "Island01", "Kfg02", "Mine00",
    "Ocean00", "Patrit01", "Tomb01", "Village00", "Xmas01",
}
NATIVE_BATCH_SCOPE = "Hai client Wine chọn map, Load, đặt bom, nhận kết quả và về phòng qua TLS/relay."
NATIVE_F1_GAMEPLAY_CHECKPOINT = "CP384"
NATIVE_F1_GAMEPLAY_SCOPE = "Hai client Wine vào trận, đặt bom, nhận kết quả và về phòng qua TLS/relay."
# Map keys with the full native cycle. Content observations describe only what
# was seen; family promotion additionally requires reviewed item/terrain scope.
NATIVE_F1_GAMEPLAY = {
    "BSSmallCamp08": "Thấy dải lưới và thùng trong vùng đấu thu nhỏ.",
    "BSSmallVillage10": "Thấy hàng cây bao quanh cụm nhà và thùng.",
    "BSSmallPatrit14": "Thấy tường bao và cột trong vùng đấu.",
    "CaJJ_CAPong01": "Thấy nền lưới với chướng ngại hình vuông và tròn.",
    "CaJJ_Camp08": "Thấy dải lưới và cột trong vùng đấu.",
    "CaJJ_Clover01": "Thấy cụm chướng ngại hình cỏ bốn lá.",
    "CaJJ_Factory07": "Thấy lưới và cột có sọc trong vùng đấu.",
    "CaJJ_Forest07": "Thấy ao ở góc dưới bên phải.",
    "CaJJ_Ocean24": "Thấy nền nước, cụm đá và các khối sáng.",
    "CaJJ_Patrit14": "Thấy các dãy thùng, cột và giàn ở giữa sân.",
    "CaJJ_RetroCamp08": "Thấy dải lưới và thùng gỗ; Fire đặt thử chưa có yêu cầu nhặt từ client.",
    "CaJJ_RetroFactory07": "Thấy cột và dải chướng ngại có sọc.",
    "CaJJ_RetroForest07": "Thấy ao ở góc dưới bên phải.",
    "CaJJ_RetroPatrit14": "Thấy cột và giàn ở giữa sân.",
    "CaJJ_RetroVillage10": "Thấy dãy nhà, cây và thùng dọc đường giữa.",
    "CaJJ_Tomb12": "Thấy rãnh dài và cầu ngang.",
    "CaJJ_Village10": "Thấy các dãy nhà, cây và thùng ở đường giữa.",
    "EventCamp00": "Thấy dãy rào và lưới bao quanh vùng giữa.",
    "EventFactory00": "Thấy cột có biểu tượng và dải có sọc.",
    "EventOcean00": "Thấy các cầu gỗ nối vùng đất.",
    "ExCamp01": "Thấy dải lưới và chướng ngại hình lốp.",
    "ExFactory01": "Thấy các cột có biểu tượng quanh vùng đấu.",
    "ExForest01": "Thấy ao và cây lớn ở giữa.",
    "ExPatrit01": "Thấy cột tròn và công trình ở giữa.",
    "ExTomb01": "Thấy rãnh tối và cầu ở giữa.",
    "ExVillage02": "Thấy cụm nhà, thùng và cụm hình cây ở giữa.",
    "Fm_CAPong01": "Thấy nền lưới với chướng ngại hình vuông và tròn.",
    "Fm_Camp08": "Thấy nhân vật tiến đến cạnh thùng; có báo cáo 142, chưa xác nhận MaxPower lộ hoặc được nhặt.",
    "Fm_Patrit14": "Thấy các dãy thùng, cột và giàn ở giữa sân.",
    "Fm_Tomb22": "Thấy dãy cột và cụm tượng bao quanh vùng giữa.",
    "NXCL_Ocean02": "Thấy nền nước, các khối sáng và cụm đá giữa.",
    "forest21": "Thấy cụm cây và hoa nhiều màu chia hai phía.",
}
# Full family coverage is explicit: a one-map family can still have unobserved
# item or terrain mechanics. Do not promote it from a match cycle alone.
NATIVE_F1_FAMILY_A = frozenset()
NATIVE_F1_CONTROLLER_A = {"BSSmallPatrit14", "CaJJ_Clover01", "NXCL_Ocean02", "ExTomb01"}
NATIVE_F1_CONTROLLER_SCOPE = "Windows VM và Wine tải map, tới kết quả và về phòng; chỉ xác nhận chu trình."

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
    def add(kind, name, wave, code, reason, *, status=None, **evidence):
        rows.append(dict(kind=kind, name=name, wave=wave, classification=code,
                         status=status or ("design" if code is None else "candidate"), reason=reason, **evidence))

    by_family = defaultdict(list)
    f1_used = set()
    for m in data["maps"]:
        by_family[m["category"], m["family"]].append(m)
    seen_batch_maps = set()
    seen_gameplay_maps = set()
    seen_controller_maps = set()
    for (category, family), maps in sorted(by_family.items()):
        base = base_theme(family) or family
        family_maps = {m["name"] for m in maps}
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
            tested = sorted(family_maps & NATIVE_BATCH_A)
            seen_batch_maps.update(tested)
            observations = [
                dict(map=name, source="kr_runtime", status="confirmed", classification="A",
                     scope=NATIVE_BATCH_SCOPE, checkpoint=NATIVE_BATCH_CHECKPOINT)
                for name in tested
            ]
            gameplay = sorted(family_maps & NATIVE_F1_GAMEPLAY.keys())
            seen_gameplay_maps.update(gameplay)
            observations.extend(
                dict(map=name, source="kr_runtime", status="confirmed", classification="A",
                     scope=NATIVE_F1_GAMEPLAY_SCOPE, content=NATIVE_F1_GAMEPLAY[name],
                     checkpoint=NATIVE_F1_GAMEPLAY_CHECKPOINT)
                for name in gameplay
            )
            controller = sorted(family_maps & NATIVE_F1_CONTROLLER_A)
            seen_controller_maps.update(controller)
            observations.extend(
                dict(map=name, source="kr_runtime", status="confirmed", classification="A",
                     scope=NATIVE_F1_CONTROLLER_SCOPE, checkpoint="CP387")
                for name in controller
            )
            sample = extra["probe"]["sample"]
            if sample in tested or sample in gameplay or sample in controller:
                extra["probe"].update(status="confirmed", classification="A",
                                      checkpoint=("CP387" if sample in controller else NATIVE_F1_GAMEPLAY_CHECKPOINT
                                                  if sample in gameplay else NATIVE_BATCH_CHECKPOINT))
            prior = NATIVE_F1.get(family)
            if prior:
                assert prior["map"] in family_maps
                extra["native_step0"] = prior
            if observations:
                extra["native_samples"] = observations
            if family in NATIVE_F1_FAMILY_A:
                assert code == "B" and family_maps <= set(gameplay)
                code, reason = "A", "Cả họ đã qua chu trình và các đối chứng nội dung nêu bên dưới."
                extra["status"] = "confirmed"
        add("map", family, wave, code, reason, category=category, base=base,
            map_count=len(maps), script_count=sum(m["scripted"] for m in maps),
            script_unknown=len(substantial), native_g1=native, **extra)
    assert seen_batch_maps == NATIVE_BATCH_A
    assert seen_gameplay_maps == NATIVE_F1_GAMEPLAY.keys()
    assert seen_controller_maps == NATIVE_F1_CONTROLLER_A

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
        "Bảng sinh từ [kiểm kê pack](../CONTENT.md). **A**: chạy được với server hiện tại trong phạm vi ghi ở dòng",
        "tương ứng; **B**: cần thêm dữ liệu, wire hoặc luật đã hiểu; **C**: còn ẩn số cần RE.",
        "Bậc của mẫu không tự áp dụng cho cả họ. Dấu **—** là ngoài phạm vi. F1 đang mở; F2–F6 tiếp tục chờ.",
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
    f1 = Counter(r["classification"] for r in rows if r["kind"] == "map" and r["wave"] == "F1")
    lines.extend([
        "",
        f"Họ map F1: {f1['A']} A, {f1['B']} B, {f1['C']} C. Có {len(NATIVE_BATCH_A | NATIVE_F1_GAMEPLAY.keys() | NATIVE_F1_CONTROLLER_A)} map đạt A ở cấp map.",
        "Bốn map trong số đó đã lặp lại trọn chu trình với cặp Windows/Wine ở [CP387](../evidence/checkpoints/records/CP387.json).",
        "",
        "### Phép thử rẻ cho F1",
        "",
        "Cột Mẫu chỉ định một map mỗi họ. [CP381](../evidence/checkpoints/records/CP381.json) giữ đối chứng",
        f"ban đầu. [{NATIVE_BATCH_CHECKPOINT}](../evidence/checkpoints/records/{NATIVE_BATCH_CHECKPOINT}.json) ghi",
        f"đối chứng chu trình. [{NATIVE_F1_GAMEPLAY_CHECKPOINT}](../evidence/checkpoints/records/{NATIVE_F1_GAMEPLAY_CHECKPOINT}.json) mở rộng mẫu trên",
        "catalog dựng từ pack, qua TLS/relay trong LAN cô lập. Mẫu A có Start/Load, tương tác trong trận, kết quả",
        "và về phòng. Cột quan sát chỉ ghi nội dung thấy hoặc đã đối chứng, kèm giới hạn nếu có. Giữ họ ở lớp B",
        "nếu còn map chưa qua chu trình hoặc còn thiếu căn cứ về item, đồ rơi và địa hình đặc biệt của họ.",
        f"Route cũ chỉ được dùng theo phạm vi [ma trận G1]({G1}).",
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
            reason = row["reason"]
            observations = row.get("native_samples", [])
            prior = row.get("native_step0")
            if prior and not any(s["map"] == prior["map"] for s in observations):
                observations = [*observations, prior]
            for observed in observations:
                result = " (A)" if observed.get("classification") == "A" else ""
                reason += f' **{observed["map"]}{result}:** {observed["scope"]}'
                if observed.get("content"):
                    reason += f' {observed["content"]}'
            lines.append(f'| {row["name"]} | {row["wave"]} | {row["classification"] or "—"} | {count} | {reason} |')
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
