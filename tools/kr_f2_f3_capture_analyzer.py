"""Compare normalized KR00967 JSON/JSONL captures from A-host and B-host runs.

Report sender patterns, static anchors and bounded candidate frame summaries.
In normalized events, payload_hex is the TCP body after framing and opcode
removal, with KR typed fields still encoded. Raw bytes, strings and notes are
not printed by the frame annotation.
"""

import argparse
from collections import Counter, defaultdict
from dataclasses import dataclass, field
import hashlib
import ipaddress
import json
from pathlib import Path
import re
import sys


ROOT = Path(__file__).resolve().parents[1]
IDA_ANCHORS = "docs/evidence/content/f2-f5-f3-ida-anchors.json"
PROBE_PLAN = "docs/evidence/content/f3-wine-probe-plan.json"
DIRECTIONS = ("C2S", "S2C")
ROLES = ("host", "guest", "server")
ROLE_LAYOUTS = {
    "host_a": {"host": "client_a", "guest": "client_b", "server": "server"},
    "host_b": {"host": "client_b", "guest": "client_a", "server": "server"},
}
DISPATCHER = "0xe5d3a0"
JUMP_TABLE = "0xe5de9c"
CASE_OFFSET = 20
# Diagnostic limits, not claims about the original server's wire limits.
MAX_CANDIDATE_BODY = 65_536
MAX_FRAME_SAMPLES = 8  # Per direction/opcode in the combined report.
KR_UINT_MASK = {1: 0x5a, 2: 0xa569, 4: 0x96ca5395, 8: 0x71b18589}
WAVE_M_IDB_SHA256 = "6be15a1fccea0837ee95c50893345e9f13dc217b0fa570975b5c284f0d547fd1"
WAVE_M_PROTOCOLS = {
    "F2": ("#810", "docs/protocol/f2-bula-tail-and-mode-wire-re.md"),
    "F3": ("#811", "docs/protocol/f3-pve-monaction-wire-re.md"),
    "F4": ("#809", "docs/protocol/f4-shop-ingame-wire-re.md"),
}

# Wire opcodes for the nondefault entries listed in #811, not Hex-Rays cases.
# 182 is absent; 204 uses the object sync fallback instead of this table.
STAGE_HANDLERS = {
    25: "0xe65a10", 29: "0xe6aa90", 84: "0xe6e1a0", 100: "0xe6d730",
    110: "0xe5e2c0", 111: "0xe6df60", 123: "vtable this + 0x2d4", 132: "0xe610c0",
    139: "0xe662b0", 140: "0xe66320", 141: "0xe66360", 142: "0xe66380",
    152: "0xe5e640", 153: "0xe5e6d0", 154: "0xe5e860", 155: "0xe67640",
    156: "0xe67770", 157: "0xe5e600", 158: "0xe5f450", 159: "0xe5e8f0",
    160: "0xe5eec0", 162: "0xe5efc0", 164: "0xe5f2c0", 165: "0xe63260",
    166: "0xe875e0", 167: "0xe63e50", 168: "0xe63f10", 170: "0xe85b60",
    171: "0xe64310", 172: "0xe64570", 173: "0xe646e0", 175: "0xe5f610",
    176: "0xe77d20", 177: "0xe6dd60", 178: "0xe77dd0", 180: "0xe72490 / vtable",
    181: "0xe652f0", 183: "0xe65350", 184: "0xe654f0", 185: "0xe67470",
    191: "0xe64070", 196: "0xe73760", 197: "0xe751f0", 198: "0xe74410",
    199: "0xe72ec0", 200: "0xe683c0", 201: "0xe678a0", 209: "0xe68280",
    210: "0xe67a60", 215: "0xe7f500", 220: "0xe65d40", 221: "0xe65f20",
    222: "0xe65e40", 274: "0xd73100",
}

FRAME_ANCHORS = {
    ("S2C", 185): ("F2", "RebirthToGhost", "0xe67470"),
    ("S2C", 204): ("F3", "monaction_delegation", "0x115cb10 -> 0x115d2e0"),
    ("C2S", 132): ("F3", "monaction_step", "0xf53320 / 0xf50dd0"),
    ("C2S", 180): ("F3", "monaction_step", "0x629940 / 0x6292a0 / 0x627de0"),
    ("S2C", 20): ("F4", "profile_wallet_setter", "0xd6e890"),
    ("S2C", 219): ("F4", "inventory_group_snapshot", "0xd76860 -> 0x11cea10"),
    ("S2C", 224): ("F4", "inventory_group_replacement", "0xd6d370 -> 0x11ce4c0"),
    ("S2C", 225): ("F4", "eight_slot_snapshot", "0xd6d410 -> 0x11ce6a0"),
    ("S2C", 313): ("F4", "item_add_or_update", "0xd779c0"),
    ("S2C", 314): ("F4", "item_delta", "0xd77bd0"),
    ("S2C", 315): ("F4", "item_operation_result", "0xd77d60"),
    ("S2C", 316): ("F4", "item_removal_or_consumption", "0xd77ef0"),
    ("S2C", 317): ("F4", "item_period_response", "0xd84410"),
    ("C2S", 566): ("F4", "item_context_byte", "0x11afa52..0x11afa79"),
}


def frame_anchor(direction, opcode):
    return (direction, opcode) in FRAME_ANCHORS or (
        direction == "S2C" and opcode in STAGE_HANDLERS
    )

HEX = re.compile(r"[0-9a-fA-F]*")
PRIVATE_TEXT = re.compile(
    r"/home/|/Users/|ocid" r"1\.|\bBEGIN[ \t]+[^\r\n]*PRIVATE[ \t]+KEY\b", re.IGNORECASE
)
LOCAL_IPV4 = re.compile(
    r"(?<![0-9.])(?:10\.[0-9]{1,3}|172\.(?:1[6-9]|2[0-9]|3[01])|192\.168|198\.1[89]"
    r"|127\.[0-9]{1,3}|169\.254|100\.(?:6[4-9]|[7-9][0-9]|1[01][0-9]|12[0-7]))"
    r"(?:\.[0-9]{1,3}){2}(?![0-9]|\.[0-9])"
)
IPV6_LITERAL = re.compile(r"(?<![\w:])[a-f0-9:]*:[a-f0-9:]+(?![\w:])", re.IGNORECASE)


@dataclass(frozen=True)
class Event:
    direction: str
    opcode: int
    sender_role: str
    ts_ms: int
    map_name: str | None
    # Retained only for candidate opcodes and never emitted in a report.
    payload_hex: str = field(default="", repr=False, compare=False)


@dataclass(frozen=True)
class Capture:
    events: tuple[Event, ...]
    sha256: str


@dataclass(frozen=True)
class Evidence:
    provenance: dict
    exact: dict
    f3_cases: dict
    authority_questions: dict


def read_bytes(path, label):
    try:
        return path.read_bytes()
    except (OSError, ValueError):
        raise ValueError(f"could not read {label}") from None


def parse_event(record, label):
    if not isinstance(record, dict):
        raise ValueError(f"{label}: expected an event object")
    direction, opcode = record.get("direction"), record.get("opcode")
    role, timestamp = record.get("sender_role"), record.get("ts_ms")
    payload = record.get("payload_hex")
    if direction not in DIRECTIONS:
        raise ValueError(f"{label}: direction must be C2S or S2C")
    if type(opcode) is not int or opcode < 0:
        raise ValueError(f"{label}: opcode must be a non-negative integer")
    if role not in ROLES:
        raise ValueError(f"{label}: invalid sender_role")
    if type(timestamp) is not int:
        raise ValueError(f"{label}: ts_ms must be an integer")
    if not isinstance(payload, str) or len(payload) % 2 or not HEX.fullmatch(payload):
        raise ValueError(f"{label}: payload_hex must contain complete hex byte pairs")
    for field in ("map_name", "note"):
        if field in record and not isinstance(record[field], str):
            raise ValueError(f"{label}: {field} must be a string")
    return Event(direction, opcode, role, timestamp, record.get("map_name"),
                 payload if frame_anchor(direction, opcode) else "")


def parse_capture(raw, label="capture"):
    """Read a UTF-8 JSON array or one JSON object per nonblank line."""
    try:
        body = raw.decode("utf-8-sig")
    except UnicodeError:
        raise ValueError(f"{label}: expected UTF-8") from None
    if body.lstrip().startswith("["):
        try:
            records = json.loads(body)
        except json.JSONDecodeError as exc:
            raise ValueError(f"{label}: invalid JSON array at line {exc.lineno}") from None
        events = [parse_event(record, f"{label} event {n}") for n, record in enumerate(records, 1)]
    else:
        events = []
        for n, line in enumerate(body.split("\n"), 1):
            if line.strip():
                location = f"{label} line {n}"
                try:
                    record = json.loads(line)
                except json.JSONDecodeError:
                    raise ValueError(f"{location}: invalid JSONL event") from None
                events.append(parse_event(record, location))
    return Capture(tuple(events), hashlib.sha256(raw).hexdigest())


class FrameDecodeError(Exception):
    """A bounded candidate decoder error safe to expose in a report."""

    def __init__(self, tag, message):
        super().__init__(message)
        self.tag = tag


class FrameReader:
    """Reader for the KR encoded integer primitives used by these packets."""

    def __init__(self, payload):
        self.payload = payload
        self.offset = 0

    def take(self, width):
        end = self.offset + width
        if end > len(self.payload):
            raise FrameDecodeError("truncated", f"need {width} bytes at offset {self.offset}")
        value = int.from_bytes(self.payload[self.offset:end], "big")
        self.offset = end
        return value ^ KR_UINT_MASK[width]

    def u8(self):
        return self.take(1)

    def u16(self):
        return self.take(2)

    def u32(self):
        return self.take(4)

    def u64(self):
        return self.take(8)

    @property
    def remaining(self):
        return len(self.payload) - self.offset


def _frame_payload(payload_hex):
    if not isinstance(payload_hex, str) or len(payload_hex) % 2 or not HEX.fullmatch(payload_hex):
        raise FrameDecodeError("error", "payload_hex is not complete hexadecimal bytes")
    if len(payload_hex) // 2 > MAX_CANDIDATE_BODY:
        raise FrameDecodeError("out_of_range", "candidate body exceeds bounded decoder limit")
    try:
        return bytes.fromhex(payload_hex)
    except ValueError:
        raise FrameDecodeError("error", "payload_hex is not complete hexadecimal bytes") from None


def _finish(reader, summary, *, allow_tail=False):
    if reader.remaining:
        if not allow_tail:
            raise FrameDecodeError("error", f"{reader.remaining} trailing bytes")
        summary["trailing_bytes"] = reader.remaining
    return summary


def _read_item(reader):
    # sub_11CE930: the encoded string is deliberately counted but not decoded.
    item = {
        "a": reader.u32(), "b": reader.u16(), "c": reader.u32(),
    }
    text_length = reader.u16()
    if text_length > 4096:
        raise FrameDecodeError("out_of_range", "item string exceeds bounded length")
    if reader.remaining < text_length + 13:  # raw8, d and e
        raise FrameDecodeError("truncated", "item record ends before fixed tail")
    reader.offset += text_length
    reader.offset += 8
    item.update({"string_length": text_length, "raw8_bytes": 8,
                 "d": reader.u32(), "e": reader.u8()})
    return item


def _decode_groups(payload, *, replacement):
    reader = FrameReader(payload)
    count = reader.u8()
    if count > 64:
        raise FrameDecodeError("out_of_range", "group count exceeds 64")
    groups = []
    for _ in range(count):
        selector, item_count = reader.u8(), reader.u32()
        if item_count > 256:
            raise FrameDecodeError("out_of_range", "item count exceeds 256")
        items = [_read_item(reader) for _ in range(item_count)]
        groups.append({"selector": selector, "item_count": item_count,
                       "items": items})
    return _finish(reader, {"group_count": count, "groups": groups,
                            "snapshot_replacement": replacement})


def _decode_225(payload):
    reader = FrameReader(payload)
    count = reader.u8()
    if count > 32:
        raise FrameDecodeError("out_of_range", "row count exceeds 32")
    rows = []
    for _ in range(count):
        slots = []
        for slot in range(8):
            item_id = reader.u32()
            text_length = reader.u16()
            if text_length > 4096:
                raise FrameDecodeError("out_of_range", "slot string exceeds bounded length")
            if reader.remaining < text_length:
                raise FrameDecodeError("truncated", "slot string is truncated")
            reader.offset += text_length
            slots.append({"slot": slot, "item_id": item_id,
                          "string_length": text_length})
        rows.append(slots)
    return _finish(reader, {"row_count": count, "rows": rows})


def _decode_fixed20(payload):
    reader = FrameReader(payload)
    values = [reader.u8(), reader.u8(), reader.u32(), reader.u32(), reader.u32(), reader.u32()]
    return _finish(reader, {"fields": values, "field_widths": [1, 1, 4, 4, 4, 4]}, allow_tail=True)


def _decode_inventory_prefix(payload, opcode):
    reader = FrameReader(payload)
    widths = {313: (1, 4, 4, 4, 2, 1), 314: (1, 4, 4, 4),
              315: (1, 1, 4, 4, 4, 8), 316: (1, 4, 4)}[opcode]
    values = [reader.take(width) for width in widths]
    return _finish(reader, {"fields": values, "field_widths": list(widths)}, allow_tail=True)


def _decode_317(payload):
    reader = FrameReader(payload)
    ok, slot, old_item, item = reader.u8(), reader.u8(), reader.u32(), reader.u32()
    if ok not in (0, 1):
        raise FrameDecodeError("out_of_range", "317 result byte must be 0 or 1")
    summary = {"ok": ok, "slot": slot, "old_item": old_item, "item": item}
    if ok:
        words = [reader.u16() for _ in range(8)]
        summary["period_words"] = words
        summary["period_value"] = reader.u32()
    return _finish(reader, summary, allow_tail=True)


def _decode_204(payload):
    reader = FrameReader(payload)
    object_id, slot = reader.u16(), reader.u8()
    if object_id == 0:
        raise FrameDecodeError("out_of_range", "object_id must be nonzero")
    if slot >= 8:
        raise FrameDecodeError("out_of_range", "owner slot must be below 8")
    return _finish(reader, {"object_id": object_id, "owner_slot": slot})


def _decode_132(payload):
    reader = FrameReader(payload)
    owner, object_id, kind = reader.u32(), reader.u16(), reader.u16()
    if object_id == 0:
        raise FrameDecodeError("out_of_range", "object_id must be nonzero")
    # Static writers use several lists (including 21/22 and the older
    # StageClear values); keep a bounded candidate range instead of treating
    # an unlisted kind as a malformed frame.
    if kind > 64:
        raise FrameDecodeError("out_of_range", f"MonAction kind {kind} exceeds bound")
    summary = {"owner_or_session": owner, "object_id": object_id, "kind": kind}
    if kind == 0:
        summary["target_id"] = reader.u32()
    else:
        summary["parameter_bytes"] = reader.remaining
    return _finish(reader, summary, allow_tail=True)


def _decode_180(payload):
    reader = FrameReader(payload)
    object_id, kind = reader.u16(), reader.u16()
    known_kinds = {0, 3, 4, 5, 6, 8, 9, 10}
    if object_id == 0:
        raise FrameDecodeError("out_of_range", "object_id must be nonzero")
    if kind not in known_kinds:
        raise FrameDecodeError("out_of_range", f"unknown MonAction kind {kind}")
    summary = {"object_id": object_id, "kind": kind}
    if kind in {3, 4, 5}:
        summary["u16_fields"] = [reader.u16(), reader.u16()]
    elif kind == 6:
        selector = reader.u8()
        if selector not in {14, 15, 16}:
            raise FrameDecodeError("out_of_range", "kind 6 selector must be 14, 15 or 16")
        summary.update({"selector": selector, "value": reader.u8()})
    elif kind == 9:
        summary["target_id"] = reader.u32()
    elif kind == 10:
        summary["parameter_bytes"] = reader.remaining
    return _finish(reader, summary, allow_tail=True)


def _decode_stage(payload, opcode):
    case = opcode - CASE_OFFSET
    summary = {"case_index": case, "handler_va": STAGE_HANDLERS[opcode],
               "body_length": len(payload)}
    if opcode in (155, 156):
        reader = FrameReader(payload)
        summary["fields"] = [reader.u8(), reader.u8()]
        return _finish(reader, summary, allow_tail=True)
    if opcode == 197:
        reader = FrameReader(payload)
        summary["fields"] = [reader.u32(), reader.u8(), reader.u32(), reader.u32(), reader.u8()]
        return _finish(reader, summary, allow_tail=True)
    summary["decode_status"] = "opaque_static_layout"
    return summary


def classify_frame(direction, opcode, payload_hex):
    """Return a bounded candidate annotation for a normalized TCP body.

    The result intentionally distinguishes static provenance from runtime
    measurement. Unknown/truncated bodies become data in the report rather
    than exceptions, so malformed captures cannot abort a whole analysis.
    """
    anchor = FRAME_ANCHORS.get((direction, opcode))
    is_stage = direction == "S2C" and opcode in STAGE_HANDLERS
    if anchor is None and not is_stage:
        return None
    feature, role, locator = anchor or ("F3", "stage_jump_table", STAGE_HANDLERS[opcode])
    issue, protocol_path = WAVE_M_PROTOCOLS[feature]
    annotation = {
        "direction": direction, "wire_opcode": opcode, "feature": feature,
        "classification": role, "source": "kr_static",
        "status": "candidate_unmeasured", "static_status": "candidate_unmeasured",
        "static_provenance": {"issue": issue, "source": "kr_static",
                               "status": "candidate_unmeasured",
                               "protocol": protocol_path, "idb_sha256": WAVE_M_IDB_SHA256,
                               "locator": locator},
    }
    try:
        payload = _frame_payload(payload_hex)
        if (direction, opcode) == ("S2C", 185):
            reader = FrameReader(payload)
            annotation["summary"] = _finish(reader, {"selector": reader.u8()})
        elif (direction, opcode) == ("S2C", 204):
            annotation["summary"] = _decode_204(payload)
        elif (direction, opcode) == ("C2S", 132):
            annotation["summary"] = _decode_132(payload)
        elif (direction, opcode) == ("C2S", 180):
            annotation["summary"] = _decode_180(payload)
        elif opcode == 20:
            annotation["summary"] = _decode_fixed20(payload)
        elif opcode in (219, 224):
            annotation["summary"] = _decode_groups(payload, replacement=opcode == 224)
        elif opcode == 225:
            annotation["summary"] = _decode_225(payload)
        elif opcode in (313, 314, 315, 316):
            annotation["summary"] = _decode_inventory_prefix(payload, opcode)
        elif opcode == 317:
            annotation["summary"] = _decode_317(payload)
        elif (direction, opcode) == ("C2S", 566):
            reader = FrameReader(payload)
            annotation["summary"] = _finish(reader, {"action_byte": reader.u8()})
        else:
            annotation["summary"] = _decode_stage(payload, opcode)
        annotation["decode_status"] = annotation["summary"].pop("decode_status", "ok")
    except FrameDecodeError as exc:
        annotation["decode_status"] = exc.tag
        annotation["error"] = str(exc)
    except (KeyError, TypeError, ValueError, IndexError):
        annotation["decode_status"] = "error"
        annotation["error"] = "candidate payload could not be decoded"
    return annotation


# Alias used by callers that describe this operation as annotation rather than
# classification; both names are intentionally kept small and side-effect free.
annotate_frame = classify_frame


def load_evidence(root=ROOT):
    ida_raw = read_bytes(root / IDA_ANCHORS, "IDA anchors")
    plan_raw = read_bytes(root / PROBE_PLAN, "F3 probe plan")
    ida, plan = json.loads(ida_raw), json.loads(plan_raw)
    ida_ref = {"locator": IDA_ANCHORS, "sha256": hashlib.sha256(ida_raw).hexdigest()}
    if (ida["schema"], plan["schema"], plan["build"]) != (
        "boom.f2f5f3.ida_anchors.v1", "kre.f3-wine-probe-plan.v1", "KR00967"
    ) or plan["inputs"]["ida_anchors"] != ida_ref:
        raise ValueError("checked-in IDA anchors and F3 probe plan do not match")
    provenance = {
        "build": plan["build"],
        "ida_anchors": {**ida_ref, "source": "kr_static", "artifacts": ida["source_artifacts"]},
        "probe_plan": {"locator": PROBE_PLAN, "sha256": hashlib.sha256(plan_raw).hexdigest(),
                       "source": plan["source"]},
    }

    exact = {}
    for row in ida["f2_modes"]["cgamestage_handlers"]:
        if row["direction"] == "S->C":
            exact["S2C", row["opcode"]] = {
                **row, "direction": "S2C", "feature": "F2", "match": "static_wire_opcode",
                "source": "kr_static", "static_status": "confirmed",
                "symbol": row["notes"].partition(";")[0],
            }

    # F5 entries from the shared registry are context, with their own stage and direction.
    social = ida["f5_social"]
    for direction, rows in (("S2C", social["cstage_handlers"]), ("C2S", social["c2s_anchors"])):
        for row in rows:
            anchor = {**row, "direction": direction, "feature": "F5", "match": "static_wire_opcode",
                      "source": "kr_static", "static_status": "confirmed"}
            if direction == "S2C":
                anchor["dispatcher_va"] = social["cstage_dispatcher_va"]
            exact[direction, row["opcode"]] = anchor

    ida_cases = {row["opcode"]: row["handler_va"] for row in ida["f3_pve"]["cgamestage_handlers"]}
    cases = {}
    for name, row in plan["static_anchors"]["anchors"].items():
        if "static_case_label" in row:
            case = row["static_case_label"]
            if row["dispatcher_va"] != DISPATCHER or ida_cases.get(case) != row["handler_va"] or case in cases:
                raise ValueError("F3 stage case anchors do not match")
            cases[case] = {
                "anchor_id": name, "static_case_label": case, "handler_va": row["handler_va"],
                "source": "kr_static", "static_status": "confirmed", "wire_mapping_status": "unknown",
            }
    if cases.keys() != ida_cases.keys():
        raise ValueError("F3 stage case anchors are incomplete")
    return Evidence(provenance, exact, cases, dict.fromkeys(plan["authority_questions"], "unknown"))


def annotate_opcode(direction, opcode, evidence):
    result = {"static_wire_anchor": evidence.exact.get((direction, opcode))}
    candidate = FRAME_ANCHORS.get((direction, opcode))
    if candidate is None and direction == "S2C" and opcode in STAGE_HANDLERS:
        candidate = ("F3", "stage_jump_table", STAGE_HANDLERS[opcode])
    if candidate is not None:
        feature, classification, locator = candidate
        issue, protocol = WAVE_M_PROTOCOLS[feature]
        result["frame_candidate"] = {
            "feature": feature, "classification": classification,
            "source": "kr_static", "status": "candidate_unmeasured",
            "static_status": "candidate_unmeasured",
            "static_provenance": {"issue": issue, "protocol": protocol,
                                   "source": "kr_static",
                                   "status": "candidate_unmeasured",
                                   "locator": locator, "idb_sha256": WAVE_M_IDB_SHA256},
        }
    if direction == "S2C":
        result["f3_jump_table_check"] = {
            "dispatcher_va": DISPATCHER,
            "jump_table_va": JUMP_TABLE,
            "raw_opcode": opcode,
            "opcode_minus_20": opcode - CASE_OFFSET,
            "raw_case_match": evidence.f3_cases.get(opcode),
            "minus_20_case_match": evidence.f3_cases.get(opcode - CASE_OFFSET),
            "wire_mapping_status": "unknown",
        }
    return result


def classify_authority(direction, a_by_map, b_by_map):
    """Classify matching sender-role sets per map across both runs."""
    if not a_by_map or a_by_map.keys() != b_by_map.keys():
        return "inconclusive"
    candidates = set()
    for name, a in a_by_map.items():
        b = b_by_map[name]
        if a != b:
            return "inconclusive"
        if direction == "C2S" and a == {"host"}:
            candidates.add("follows_host")
        elif direction == "C2S" and a == {"host", "guest"}:
            candidates.add("both_clients")
        elif direction == "S2C" and a == {"server"}:
            candidates.add("server_originated")
        else:
            return "inconclusive"
    return candidates.pop() if len(candidates) == 1 else "inconclusive"


def summarize_capture(capture):
    grouped = defaultdict(lambda: defaultdict(list))
    for event in capture.events:
        grouped[event.direction, event.opcode][event.map_name].append(event)
    return grouped


def summarize_maps(by_map):
    summaries, roles_by_map = [], {}
    for name, events in sorted(by_map.items(), key=lambda item: (item[0] is not None, item[0] or "")):
        counts = Counter(event.sender_role for event in events)
        roles_by_map[name] = set(counts)
        summaries.append({
            "map_name": name,
            "sender_role_counts": dict(counts),
            "first_ts_ms": min(event.ts_ms for event in events),
            "last_ts_ms": max(event.ts_ms for event in events),
        })
    return summaries, roles_by_map


def analyze_captures(host_a, host_b=None, *, evidence):
    captures = {"host_a": host_a}
    if host_b is not None:
        captures["host_b"] = host_b
    groups = {name: summarize_capture(capture) for name, capture in captures.items()}
    opcodes = []
    for direction, opcode in sorted(set().union(*groups.values())):
        observations, roles = {}, {}
        frame_annotations = []
        for name, group in groups.items():
            observations[name], roles[name] = summarize_maps(group.get((direction, opcode), {}))
        for name, group in groups.items():
            for events in group.get((direction, opcode), {}).values():
                for event in events[:MAX_FRAME_SAMPLES]:
                    if frame_anchor(direction, opcode):
                        annotation = classify_frame(direction, opcode, event.payload_hex)
                        if annotation is not None:
                            frame_annotations.append({
                                **annotation, "capture": name, "map_name": event.map_name,
                                "sender_role": event.sender_role, "ts_ms": event.ts_ms,
                            })
                if len(frame_annotations) >= MAX_FRAME_SAMPLES:
                    break
            if len(frame_annotations) >= MAX_FRAME_SAMPLES:
                break
        row = {
            "direction": direction,
            "opcode": opcode,
            "observations": observations,
            "authority_candidate": classify_authority(direction, roles["host_a"], roles.get("host_b", {})),
            "authority_status": "unknown",
            **annotate_opcode(direction, opcode, evidence),
        }
        if frame_annotations:
            row["frame_annotations"] = frame_annotations
        opcodes.append({
            **row,
        })
    return {
        "schema": "kre.f2-f3-capture-analysis.v1",
        "evidence": evidence.provenance,
        "captures": {
            name: {"sha256": capture.sha256, "event_count": len(capture.events),
                   "sender_role_layout": ROLE_LAYOUTS[name]}
            for name, capture in captures.items()
        },
        "authority_questions": evidence.authority_questions,
        "limits": [
            "Sender candidates assume comparable runs with the same two clients and reversed host roles.",
            "Candidates require matching patterns in both runs on each map. A null map is unlabeled.",
            "Sender roles cannot distinguish server decisions from relays or show when clients apply state.",
            "Static matches do not prove handler execution. F3 case matches are numerical checks only.",
        ],
        "opcodes": opcodes,
    }


def validate_output(value):
    """Screen decoded JSON keys and string values before publishing a report."""
    if isinstance(value, dict):
        for key, item in value.items():
            validate_output(key)
            validate_output(item)
    elif isinstance(value, list):
        for item in value:
            validate_output(item)
    elif isinstance(value, str):
        if PRIVATE_TEXT.search(value) or LOCAL_IPV4.search(value):
            raise ValueError("report contains private data")
        for match in IPV6_LITERAL.finditer(value):
            try:
                address = ipaddress.IPv6Address(match[0])
            except ValueError:
                continue
            if address.is_private:
                raise ValueError("report contains private data")


def render_report(report):
    validate_output(report)
    return json.dumps(report, indent=2, sort_keys=True, ensure_ascii=True, allow_nan=False) + "\n"


def self_check(evidence):
    raw = (b'[{"direction":"S2C","opcode":185,"sender_role":"server","payload_hex":"","ts_ms":0},'
           b'{"direction":"S2C","opcode":197,"sender_role":"server","payload_hex":"","ts_ms":1}]')
    capture = parse_capture(raw)
    report = analyze_captures(capture, capture, evidence=evidence)
    ghost, timer = report["opcodes"]
    shifted = timer["f3_jump_table_check"]["minus_20_case_match"]
    if ((ghost["static_wire_anchor"] or {}).get("symbol") != "RebirthToGhost"
            or (shifted or {}).get("static_case_label") != 177
            or timer["authority_candidate"] != "server_originated"
            or timer["authority_status"] != "unknown"):
        raise ValueError("synthetic capture self-check failed")
    render_report(report)


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    mode = parser.add_mutually_exclusive_group(required=True)
    mode.add_argument("--host-a", type=Path, metavar="FILE", help="normalized capture with client A hosting")
    mode.add_argument("--self-check", action="store_true", help="check with synthetic events and checked-in evidence")
    parser.add_argument("--host-b", type=Path, metavar="FILE", help="normalized capture with client B hosting")
    parser.add_argument("--out", type=Path, metavar="FILE", help="write the JSON report to a new file")
    args = parser.parse_args(argv)
    if args.self_check and (args.host_b is not None or args.out is not None):
        parser.error("--self-check cannot be combined with --host-b or --out")
    try:
        try:
            evidence = load_evidence()
        except (ValueError, KeyError, TypeError):
            raise ValueError("could not load matching checked-in evidence") from None
        if args.self_check:
            self_check(evidence)
            print("F2/F3 capture analyzer self-check OK")
        else:
            host_a = parse_capture(read_bytes(args.host_a, "host_a"), "host_a")
            host_b = (parse_capture(read_bytes(args.host_b, "host_b"), "host_b")
                      if args.host_b is not None else None)
            output = render_report(analyze_captures(host_a, host_b, evidence=evidence))
            if args.out is None:
                sys.stdout.write(output)
            else:
                try:
                    args.out.write_text(output, encoding="utf-8", newline="\n")
                except (OSError, ValueError):
                    raise ValueError("could not write --out") from None
    except (ValueError, RecursionError) as exc:
        parser.exit(1, f"F2/F3 capture analyzer: {exc}\n")


if __name__ == "__main__":
    main()
