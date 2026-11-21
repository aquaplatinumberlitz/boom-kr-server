"""Pin private session artifacts and extract a publishable two-round replay."""
import argparse
import hashlib
import json
import ipaddress
import os
from pathlib import Path
import re
import signal
import subprocess
import sys


def read_jsonl(path):
    with path.open(encoding="utf-8") as source:
        for line in source:
            yield json.loads(line)


def pin(path):
    with path.open("rb") as stream:
        digest = hashlib.file_digest(stream, "sha256").hexdigest()
    return {"sha256": digest, "bytes": path.stat().st_size}


def project(events, gates, capture=None):
    """Project known milestones; never carry packet bodies, names or addresses."""
    rounds = []
    current = None
    request = "native_tls_room_request"
    for row in read_jsonl(events):
        event, opcode = row["event"], row.get("opcode")
        if (event, opcode) == (request, 95):
            current = {"steps": []}
            rounds.append(current)
        if current is None:
            continue
        if (event, opcode) in {
            (request, 95), (request, 104), (request, 99), (request, 118),
            (request, 202), (request, 585),
        } or event in {
            "native_tls_room_ready_written", "native_tls_room_start_written",
            "native_tls_room_return_refreshed",
        }:
            current["steps"].append({
                "native_tls_room_ready_written": "ready",
                "native_tls_room_start_written": "start",
                "native_tls_room_return_refreshed": "return",
            }.get(event, str(opcode)))
    journal = [
        row["rounds"][-1] for row in read_jsonl(gates)
        if re.fullmatch(r"journal-complete-[12]", row["state"])
    ]
    if len(rounds) != 2 or len(journal) != 2:
        raise ValueError("expected one complete two-round run")
    for index, (round_, match) in enumerate(zip(rounds, journal, strict=True), 1):
        expected = ["95", "ready", "ready", "104", "start", "start", "99", "99",
                    "118", "118", "202", "202", "585", "585", "return", "return"]
        # Return refresh may be delivered before the second input is logged.
        steps = round_["steps"]
        if steps[:12] != expected[:12] or sorted(steps[12:]) != sorted(expected[12:]):
            raise ValueError(f"round {index}: unsupported event sequence")
        if (match[0] != 256 * index or match[3] != "complete" or
                match[4] not in {"elimination", "all_dead"} or match[5] != "Factory07"):
            raise ValueError(f"round {index}: unsupported journal row")
        round_["result"] = match[4]
    provenance = {"events": pin(events), "journal_markers": pin(gates)}
    if capture is not None:
        provenance["encrypted_capture"] = pin(capture)
    return {"schema": "kre-semantic-session-v1", "map": "Factory07",
            "source": provenance, "rounds": rounds}


def local_target(path):
    local = Path(__file__).resolve().parents[1] / "local"
    if not path.resolve().is_relative_to(local.resolve()):
        raise ValueError("private manifest must be under this worktree's local/")


def capture(interface, hosts, seconds, output):
    """Capture full frames on the declared isolated lab interface."""
    local_target(output)
    if not re.fullmatch(r"(?!any$)[a-zA-Z0-9_.:-]{1,15}", interface):
        raise ValueError("specify one isolated capture interface")
    if not 1 <= seconds <= 900 or not 1 <= len(hosts) <= 8:
        raise ValueError("capture requires 1..900 seconds and 1..8 hosts")
    addresses = [str(ipaddress.ip_address(host)) for host in hosts]
    packet_filter = " or ".join("host " + a for a in addresses)
    packet_filter = f"(udp or tcp) and ({packet_filter})"
    # Write through a private file descriptor rather than asking privileged
    # tcpdump to create or overwrite a file by name.
    with os.fdopen(os.open(output, os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600), "wb") as trace:
        with subprocess.Popen(["tcpdump", "-n", "-i", interface, "-p", "-s", "0", "-U",
                               "-w", "-", packet_filter], stdout=trace) as recorder:
            try:
                recorder.wait(timeout=seconds)
            except (subprocess.TimeoutExpired, KeyboardInterrupt):
                recorder.send_signal(signal.SIGINT)
                try:
                    recorder.wait(timeout=5)
                except subprocess.TimeoutExpired:
                    recorder.kill()
                    raise ValueError("capture failed to flush") from None
            if recorder.returncode or trace.tell() <= 24:
                raise ValueError("capture ended without a valid packet stream")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(required=True, dest="command")
    receiver = sub.add_parser("capture")
    receiver.add_argument("--interface", required=True)
    receiver.add_argument("--host", action="append", required=True)
    receiver.add_argument("--seconds", type=int, required=True)
    receiver.add_argument("--output", type=Path, required=True)
    replay = sub.add_parser("replay")
    replay.add_argument("fixture", type=Path)
    for name in ("pin", "project"):
        command = sub.add_parser(name)
        command.add_argument("--events", required=True, type=Path)
        command.add_argument("--gate", required=True, type=Path)
        command.add_argument("--pcap", type=Path, required=name == "pin")
        command.add_argument("--output", required=True, type=Path)
        if name == "pin":
            for label in ("companion", "journal", "client", "server"):
                command.add_argument("--" + label, required=True, type=Path,
                                     action="append" if label in ("companion", "client") else "store")
    args = parser.parse_args()
    if args.command == "capture":
        capture(args.interface, args.host, args.seconds, args.output)
        return
    if args.command == "replay":
        root = Path(__file__).resolve().parents[1]
        env = dict(os.environ, KR_SEMANTIC_SESSION=str(args.fixture.resolve()))
        sys.exit(subprocess.run(["cargo", "test", "--locked", "-p", "kr-service",
                                 "--lib", "replay_local_session", "--", "--ignored"], cwd=root / "rust", env=env).returncode)
    if args.command == "pin":
        local_target(args.output)
        inputs = ("events", "gate", "pcap", "companion", "journal", "client", "server")
        data = {"schema": "kre-private-session-v1", "files": {
            key: [pin(p) for p in paths] if isinstance(paths := getattr(args, key), list)
            else pin(paths) for key in inputs
        }}
    else:
        data = project(args.events, args.gate, args.pcap)
    with args.output.open("x", encoding="utf-8") as output:
        json.dump(data, output, indent=2)
        output.write("\n")


if __name__ == "__main__":
    main()
