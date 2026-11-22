#!/usr/bin/env python3
"""Build a short, fail-fast M8 workload configuration for a Linux namespace.

This only writes new private input files. It never starts a service, changes a
remote process, or applies ``tc`` rules. The generated plan contains the
operator command to apply random netem faults inside the namespace.
"""
from __future__ import annotations

import argparse
import json
from pathlib import Path


def partition(accounts: list[dict], count: int) -> list[list[dict]]:
    if not 1 <= count <= 8:
        raise ValueError("rooms must be between 1 and 8")
    if len(accounts) < count * 2 or len(accounts) > count * 8:
        raise ValueError("account count cannot fill the requested rooms")
    rooms = [[] for _ in range(count)]
    for index, account in enumerate(accounts):
        rooms[index % count].append(account)
    if any(not 2 <= len(room) <= 8 for room in rooms):
        raise ValueError("partition produced a room outside 2..8 players")
    return rooms


def build_driver(source: dict, rooms: int, round_seconds: int,
                 run_seconds: int, ramp_ms: int) -> dict:
    if not 2 <= round_seconds <= 180:
        raise ValueError("round-seconds must be between 2 and 180")
    if not 1 <= run_seconds <= 86400:
        raise ValueError("run-seconds must be between 1 and 86400")
    if not 100 <= ramp_ms <= 10000:
        raise ValueError("ramp-ms must be between 100 and 10000")
    accounts = [account for room in source.get("rooms", []) for account in room]
    result = dict(source)
    result["rooms"] = partition(accounts, rooms)
    result["active_seconds"] = round_seconds
    result["minimum_run_seconds"] = run_seconds
    result["minimum_rounds"] = 1
    result["ramp_ms"] = ramp_ms
    return result


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--driver-config", type=Path, required=True)
    parser.add_argument("--out", type=Path, required=True)
    parser.add_argument("--rooms", type=int, default=8)
    parser.add_argument("--round-seconds", type=int, default=2)
    parser.add_argument("--run-seconds", type=int, default=180)
    parser.add_argument("--ramp-ms", type=int, default=100)
    parser.add_argument("--fault-rate", type=float, default=0.0,
                        help="random packet loss percentage for the emitted netem plan")
    parser.add_argument("--plan-out", type=Path)
    args = parser.parse_args()
    if not 0 <= args.fault_rate <= 100:
        parser.error("fault-rate must be between 0 and 100")
    source = json.loads(args.driver_config.read_text(encoding="utf-8"))
    result = build_driver(source, args.rooms, args.round_seconds,
                          args.run_seconds, args.ramp_ms)
    args.out.parent.mkdir(parents=True, exist_ok=True)
    args.out.write_text(json.dumps(result, indent=2) + "\n", encoding="utf-8")
    plan_path = args.plan_out or args.out.with_name("chaos-plan.json")
    plan = {
        "schema": "m8-accelerated-soak-plan-v1",
        "driver_config": str(args.out),
        "stop_on_first_driver_error": True,
        "fault_rate_percent": args.fault_rate,
        "namespace_command": (
            "tc qdisc replace dev veth-soak root netem "
            f"loss {args.fault_rate:.3f}% 25% delay 20ms 10ms"
        ) if args.fault_rate else None,
        "evidence": ["driver.out", "driver.err", "rust-events.jsonl", "health.json"],
        "notes": [
            "Use one veth pair in an unshare -Urn namespace on PC or VPS.",
            "Run m8_soak_triage.py immediately after kr-load exits.",
            "Do not reuse an accelerated result as the 12-hour M8 gate.",
        ],
    }
    plan_path.write_text(json.dumps(plan, indent=2) + "\n", encoding="utf-8")


if __name__ == "__main__":
    main()
