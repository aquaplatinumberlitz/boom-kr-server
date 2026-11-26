"""Compare normalized KR00967 JSON/JSONL captures from A-host and B-host runs.

Report packet sender patterns and static anchor matches. Payload bytes and
free-form notes stay in the original capture for local correlation.
"""

import argparse
from collections import Counter, defaultdict
from dataclasses import dataclass
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
    return Event(direction, opcode, role, timestamp, record.get("map_name"))


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
        for name, group in groups.items():
            observations[name], roles[name] = summarize_maps(group.get((direction, opcode), {}))
        opcodes.append({
            "direction": direction,
            "opcode": opcode,
            "observations": observations,
            "authority_candidate": classify_authority(direction, roles["host_a"], roles.get("host_b", {})),
            "authority_status": "unknown",
            **annotate_opcode(direction, opcode, evidence),
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
