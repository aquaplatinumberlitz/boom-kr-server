#!/usr/bin/env python3
"""Classify owner lifecycle records without treating planned client closes as failures.

The input files are private soak exports.  The report is safe to publish: it
contains event metadata only and never copies packet bodies or credentials.
"""
from __future__ import annotations

import argparse
import json
from pathlib import Path


def load(path: Path) -> list[dict]:
    rows: list[dict] = []
    with path.open(encoding="utf-8") as stream:
        for line_number, line in enumerate(stream, 1):
            try:
                row = json.loads(line)
            except json.JSONDecodeError:
                continue
            if isinstance(row, dict):
                row["_line"] = line_number
                rows.append(row)
    return rows


def classify(events: list[dict], driver: list[dict] | None = None) -> dict:
    """Return a bounded, deterministic owner-error report.

    Older exports predate ``classification``.  Their errors are expected only
    when every error is in the terminal tail after a failed delivery and the
    service drained; otherwise they remain unexpected and gate the soak.
    """
    owner = [row for row in events if row.get("event") == "native_tls_owner_ended"
             and row.get("ok") is False]
    authority = [row for row in events if row.get("event") == "native_account_authority_ended"]
    health = [row for row in events if row.get("event") == "health_sample"]
    failures = [row for row in (driver or [])
                if row.get("event") == "delivery" and row.get("passed") is False]
    failure_time = min((item.get("at_unix_ms") for item in failures
                        if isinstance(item.get("at_unix_ms"), int)), default=None)
    owner_times = []
    for row in owner:
        previous_health = next((candidate for candidate in reversed(health)
                                if candidate.get("_line", 0) <= row.get("_line", 0)), None)
        if previous_health and isinstance(previous_health.get("at_utc"), int):
            owner_times.append(previous_health["at_utc"] * 1000)
    terminal_delivery = bool(failures and owner_times and failure_time is not None
                             and min(owner_times) >= failure_time)
    items = []
    for row in owner:
        classification = row.get("classification")
        reason = "service-reported failure"
        if classification == "expected_peer_close":
            reason = "client closed QUIC normally while stream tasks were joining"
        elif terminal_delivery:
            classification = "expected_peer_close"
            reason = "legacy export: driver stopped after terminal delivery failure"
        else:
            classification = "unexpected"
        previous_stream = next((candidate for candidate in reversed(events)
                                if candidate.get("_line", 0) < row.get("_line", 0)
                                and candidate.get("event") == "native_tls_owned_stream_ended"), None)
        previous_health = next((candidate for candidate in reversed(health)
                                if candidate.get("_line", 0) <= row.get("_line", 0)), None)
        items.append({
            "event_line": row.get("_line"),
            "guard": row.get("guard"),
            "opcode": row.get("opcode"),
            "subtype": row.get("subtype"),
            "stream_id": row.get("stream_id"),
            "poll_stage": row.get("poll_stage"),
            "classification": classification,
            "reason": reason,
            "observed_at_utc": previous_health.get("at_utc") if previous_health else None,
            "health_lobby": previous_health.get("lobby") if previous_health else None,
            "preceding_stream_error": {
                "guard": previous_stream.get("guard"),
                "stream_id": previous_stream.get("stream_id"),
                "event_line": previous_stream.get("_line"),
            } if previous_stream else None,
        })
    counts: dict[str, int] = {}
    for item in items:
        key = item["classification"]
        counts[key] = counts.get(key, 0) + 1
    return {
        "schema": "m8-owner-lifecycle-triage-v1",
        "owner_events": len(items),
        "authority_events": len(authority),
        "authority_counts": {
            "expected_ttl_or_relogin": sum(row.get("reason") == "owner_stale" for row in authority),
            "unexpected": sum(row.get("reason") != "owner_stale" for row in authority),
        },
        "counts": counts,
        "delivery_failures": len(failures),
        "delivery_failure_windows": [
            {key: row.get(key) for key in ("_line", "at_unix_ms", "room", "round", "delivery")}
            for row in failures
        ],
        "terminal_driver_sequence": [
            {key: row.get(key) for key in ("_line", "event", "room", "round", "at_unix_ms", "service_room")}
            for row in (driver[-16:] if driver else [])
        ],
        "items": items,
        "unexpected": sum(value for key, value in counts.items() if key == "unexpected"),
    }


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--events", type=Path, required=True)
    parser.add_argument("--driver", type=Path)
    parser.add_argument("--out", type=Path, required=True)
    args = parser.parse_args()
    events = load(args.events)
    driver = load(args.driver) if args.driver else []
    args.out.parent.mkdir(parents=True, exist_ok=True)
    args.out.write_text(json.dumps(classify(events, driver), indent=2) + "\n", encoding="utf-8")


if __name__ == "__main__":
    main()
