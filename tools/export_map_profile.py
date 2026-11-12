"""Xuất profile map KR vào local; đối chiếu seed CP094 trước mọi output."""
import argparse
import collections
import hashlib
import json
import struct
import xml.etree.ElementTree as ET
from pathlib import Path

from content_census import Pack, parse_map, text

MAP_SHA = "aeb57ffdb27a1405bd2ae3d29f229174ae5d584fc84eaf4e87884fe8518f86a4"
SEED_SHA = "c96dff8c4d148a229bef97ba8c3a279f1a1cac3516281de1689385353b972144"
OBJECT_KINDS = {1932: 2, 1933: 2, 1934: 2, 1938: 0, 1941: 0,
                1942: 0, 1943: 0, 1947: 0, 1955: 1, 1956: 1, 1957: 1}
FIELDS = ("tile", "object", "overlay")


def decode_map(data):
    """Dùng census kiểm cấu trúc, bổ sung các trường census chỉ đếm."""
    info = parse_map(data)
    if info["parse"] != "exact":
        raise ValueError("map parser không đọc hết payload")
    width, height = info["width"], info["height"]
    off = 80
    spawns = [list(struct.unpack_from("<2i", data, off + 8 * i))
              for i in range(info["spawns"])]
    off += 8 * info["spawns"]
    rect_count = struct.unpack_from("<i", data, off)[0]
    off += 4
    rects = [list(struct.unpack_from("<4i", data, off + 16 * i)) for i in range(rect_count)]
    off += 16 * rect_count
    stride = 2 if info["version"] == 1000 else 3
    cells = [list(struct.unpack_from(f"<{stride}i", data, off + i * stride * 4))
             for i in range(width * height)]
    off += width * height * stride * 4
    drop_count = struct.unpack_from("<i", data, off)[0]
    off += 4
    for _ in range(drop_count):
        _, off = text(data, off)
        off += 4
    placement_header, count = struct.unpack_from("<2i", data, off)
    off += 8
    placements = []
    for _ in range(count):
        name, off = text(data, off)
        x, y, probability = struct.unpack_from("<3B", data, off)
        off += 3
        placements.append(dict(name=name, x=x, y=y, probability=probability))
    if [p["name"] for p in placements] != info["placements"]:
        raise ValueError("placement không khớp parser census")
    return dict(version=info["version"], width=width, height=height, spawns=spawns,
                rectangles=rects, cell_fields=list(FIELDS[:stride]), cells=cells,
                drops=info["drops"], placement_header=placement_header,
                placements=placements, scripted=info["scripted"])


def classify(cells, baseline):
    known = [set() for _ in FIELDS]
    for cell in baseline:
        for index, value in enumerate(cell):
            known[index].add(value)
    unknown = [collections.Counter() for _ in FIELDS]
    kinds, unknown_cells = [], 0
    for cell in cells:
        novel = False
        for index, value in enumerate(cell):
            if value not in known[index]:
                unknown[index][value] += 1
                novel = True
        obj = cell[1]
        kind = 0 if obj == -1 else OBJECT_KINDS.get(obj)
        if novel or kind is None:
            unknown_cells += 1
            kind = None
        kinds.append(kind)
    return kinds, unknown_cells, {
        FIELDS[i]: dict(sorted(counter.items())) for i, counter in enumerate(unknown) if counter
    }


def load_tile_table(pack):
    """Nối thứ tự ObjectTable với TileInfo theo tên; không coi XML là bảng chỉ số."""
    data = pack.read("ObjectTable/ObjectTable")
    count = struct.unpack_from("<I", data)[0]
    off, names = 4, []
    for _ in range(count):
        length = struct.unpack_from("<I", data, off)[0]
        off += 4
        if not length or off + length + 4 > len(data):
            raise ValueError("ObjectTable sai độ dài tên")
        names.append(data[off:off + length].rstrip(b"\0").decode("cp949"))
        off += length + 4
    raw = pack.read("TileInfo/TileInfoList")
    length = struct.unpack_from("<I", raw)[0]
    if length != len(raw) - 4:
        raise ValueError("TileInfoList sai độ dài")
    root = ET.fromstring(raw[4:].rstrip(b"\0"))
    tiles = {}
    for element in root.iter("Tile"):
        name = element.attrib["Name"]
        data = bytes.fromhex(element.attrib["Data"])
        if name in tiles or len(data) != 72:
            raise ValueError("TileInfoList trùng tên hoặc sai Data")
        tiles[name] = struct.unpack("<18i", data)
    return names, tiles


def classify_rules(cells, table):
    """Các mẫu Data đầu chỉ là ứng viên theo seed, không phải luật vật lý đã xác nhận."""
    names, tiles = table
    unknown = collections.Counter()
    kinds = []

    def lookup(code):
        if code < 0 or code >= len(names):
            return None, None
        name = names[code]
        return name, tiles.get(name)

    for cell in cells:
        tile_name, tile = lookup(cell[0])
        obj_name, obj = lookup(cell[1])
        # Những giá trị bổ sung và địa hình đặc biệt chưa có oracle.
        floor = (tile is not None and tile[:11] == (0, 0, 0, 0, 0, 0, 1, 1, 0, 0, -1)
                 and "pin" not in tile_name.lower())
        kind = None
        if floor and all(value == -1 for value in cell[2:]):
            if cell[1] == -1:
                kind = 0
            elif obj is not None:
                if (obj[:11] == (0, 0, 0, 0, 0, 0, 1, 1, 0, 0, -1)
                        and "pin" not in obj_name.lower()):
                    kind = 0
                elif obj[:11] == (1, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0):
                    kind = 1
                elif (obj[:7] == (2, 0, 0, 0, 0, 0, 0) and obj[7] in (0, 1)
                      and obj[8:11] == (0, 0, 0)):
                    kind = 2
        if kind is None:
            def label(code):
                name, _ = lookup(code)
                return name if name is not None else str(code)
            unknown[" | ".join(label(value) for value in cell)] += 1
        kinds.append(kind)
    return kinds, sum(unknown.values()), dict(sorted(unknown.items()))


def verify_seed(baseline, fixture, classifier=None):
    spec = fixture["bomb_terrain"]
    expected = spec["kinds"]
    if (spec["width"], spec["height"]) != (baseline["width"], baseline["height"]):
        raise ValueError("fixture terrain sai kích thước")
    digest = hashlib.sha256(bytes(expected)).hexdigest()
    if digest != SEED_SHA or spec["seed_sha256"] != SEED_SHA or spec["map_sha256"] != MAP_SHA:
        raise ValueError("fixture không khớp hash seed/map CP094 đã pin")
    actual, _, _ = (classifier(baseline["cells"]) if classifier else
                    classify(baseline["cells"], baseline["cells"]))
    if len(actual) != len(expected):
        raise ValueError("fixture terrain sai số ô")
    differences = [(i % baseline["width"], i // baseline["width"], a, b)
                   for i, (a, b) in enumerate(zip(actual, expected)) if a != b]
    if differences:
        raise ValueError(f"terrain lệch (x, y, xuất, fixture): {differences}")
    return len(actual)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--classification", choices=("legacy", "rules"), default="legacy")
    parser.add_argument("--client", type=Path, required=True)
    parser.add_argument("--map", action="append", required=True, dest="maps")
    parser.add_argument("--terrain-fixture", type=Path, required=True,
                        help="profile.json CP094 có bomb_terrain đã pin")
    parser.add_argument("--out", type=Path, required=True)
    args = parser.parse_args()
    root = (Path(__file__).resolve().parents[1] / "local").resolve()
    if not args.out.resolve().is_relative_to(root):
        parser.error("--out phải nằm trong local/ của worktree")
    try:
        pack = Pack(args.client, "FxMap")
        names = [name for name, *_ in pack.order if name.startswith("Map/")]
        if len(names) != len(set(names)):
            raise ValueError("tên Map/* trùng trong pack")
        raw = pack.read("Map/ExVillage01")
        if hashlib.sha256(raw).hexdigest() != MAP_SHA:
            raise ValueError("payload ExVillage01 không khớp SHA CP090")
        baseline = decode_map(raw)
        count = verify_seed(baseline, json.loads(args.terrain_fixture.read_text()))
        table = load_tile_table(Pack(args.client, "fx")) if args.classification == "rules" else None
        if table is not None:
            verify_seed(baseline, json.loads(args.terrain_fixture.read_text()),
                        lambda cells: classify_rules(cells, table))
        profiles = []
        for name in args.maps:
            entry = "Map/" + name
            if entry not in names:
                raise ValueError(f"không có map {name!r} trong pack")
            data = pack.read(entry)
            profile = decode_map(data)
            kinds, unknown_cells, unknown = (classify_rules(profile["cells"], table) if table is not None
                                             else classify(profile["cells"], baseline["cells"]))
            profile.update(name=name, map_code=names.index(entry),
                           payload_sha256=hashlib.sha256(data).hexdigest(),
                           terrain=dict(classification=args.classification, kinds=kinds, unknown_cell_count=unknown_cells,
                                        unknown_values=unknown, unknown_meaning="chưa rõ nghĩa",
                                        limits="CP094 design, runtime hẹp ExVillage01; null là chưa rõ nghĩa"))
            profiles.append(profile)
        args.out.mkdir(parents=True, exist_ok=True)
        for profile in profiles:
            (args.out / (profile["name"] + ".json")).write_text(
                json.dumps(profile, ensure_ascii=False, indent=2) + "\n")
        print(f"ExVillage01: khớp {count}/{count} ô với seed CP094 {SEED_SHA}")
        for p in profiles:
            print(f"{p['name']}: terrain={dict(collections.Counter(p['terrain']['kinds']))}")
            print(f'{p["name"]}: {p["width"]}x{p["height"]}, spawn={len(p["spawns"])}, '
                  f'chưa rõ={p["terrain"]["unknown_cell_count"]}, giá trị={p["terrain"]["unknown_values"]}')
    except (KeyError, ValueError, OSError, struct.error, ET.ParseError) as exc:
        parser.exit(1, f"Lỗi: {exc}\n")


if __name__ == "__main__":
    main()
