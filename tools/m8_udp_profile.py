"""Measure native client egress offline from the pinned CP287 loopback capture."""
import argparse
from collections import Counter
import hashlib
import json
from pathlib import Path
import statistics
import struct

ROOT = Path(__file__).resolve().parents[1]
RUN = "local/research/codex/n1rb-owned-four-native-r2"
RECEIPT = f"{RUN}/owned-four-verification.json"
ACTORS = "ABCD"
SECOND = 1_000_000_000


def pinned(root, row):
    path = (root / row["path"]).resolve()
    if not path.is_relative_to(root):
        raise ValueError("archive path escapes root")
    raw = path.read_bytes()
    if len(raw) != row["bytes"] or hashlib.sha256(raw).hexdigest() != row["sha256"]:
        raise ValueError(f"archive pin mismatch: {path.name}")
    return raw


def pcap_records(raw):
    if len(raw) < 24 or raw[:4] != bytes.fromhex("d4c3b2a1") or struct.unpack_from("<I", raw, 20)[0] != 1:
        raise ValueError("expected little-endian microsecond Ethernet PCAP")
    at = 24
    while at < len(raw):
        if len(raw) - at < 16:
            raise ValueError("truncated PCAP record")
        sec, usec, size, original = struct.unpack_from("<IIII", raw, at)
        end = at + 16 + size
        if end > len(raw) or size != original or usec >= 1_000_000:
            raise ValueError("truncated or invalid PCAP packet")
        packet = raw[at + 16:end]
        at = end
        if len(packet) < 42 or packet[12:14] != b"\x08\x00" or packet[14] >> 4 != 4:
            raise ValueError("expected Ethernet IPv4")
        udp = 14 + 4 * (packet[14] & 15)
        ip_len = struct.unpack_from("!H", packet, 16)[0]
        fragment = struct.unpack_from("!H", packet, 20)[0]
        if (udp < 34 or udp + 8 > len(packet) or packet[23] != 17
                or fragment & 0x3fff or 14 + ip_len > len(packet)):
            raise ValueError("expected unfragmented UDP")
        source, destination, length = struct.unpack_from("!HHH", packet, udp)
        if length < 8 or udp + length != 14 + ip_len:
            raise ValueError("invalid UDP length")
        yield (sec * SECOND + usec * 1_000), source, destination, length - 8


def summarize(raw, decoded, clock):
    # The sidecar provides the monotonic clock and Linux capture direction.
    # A timestamp match for every row prevents accidental cross-run joins.
    observed = []
    for (wall, source, destination, size), row in zip(pcap_records(raw), decoded["rows"], strict=True):
        if (abs(wall - row["epoch_ns"]) >= 1000
                or (observed and row["monotonic_ns"] < observed[-1][0])):
            raise ValueError("PCAP and timestamp sidecar disagree")
        observed.append((row["monotonic_ns"], row["packet_type"], source, destination, size))
    rounds = []
    # CP287 has a third start but only two observed results.
    for number, (start, stop) in enumerate(zip(clock["clock_ns"], clock["rank_ns"]), 1):
        peers = list(clock["native_proxy_ports"][str(number)].values())
        if stop <= start or not peers or len(set(peers)) != len(peers):
            raise ValueError("invalid round window or proxy set")
        sent = [(stamp, destination, size)
                for stamp, kind, source, destination, size in observed
                if kind == 4 and start <= stamp < stop and destination in peers and source not in peers]
        by_peer = Counter(destination for _, destination, _ in sent)
        if set(by_peer) != set(peers):
            raise ValueError("whole round must cover every recorded proxy")
        duration = (stop - start) / SECOND
        peak = 0
        left = 0
        for right, (stamp, _, _) in enumerate(sent):
            while stamp - sent[left][0] >= SECOND:
                left += 1
            peak = max(peak, right - left + 1)
        rounds.append(dict(round=number, duration_seconds=round(duration, 3), sent=len(sent),
                           packets_per_second=round(len(sent) / duration, 3),
                           peak_rolling_1s_packets=peak,
                           per_peer_packets_per_second=sorted(round(n / duration, 3) for n in by_peer.values()),
                           payload_bytes=dict(sorted(Counter(size for _, _, size in sent).items()))))
    if len(rounds) != 2:
        raise ValueError("CP287 requires two complete round windows")
    return rounds


def measure(archive):
    checkpoint = json.loads((ROOT / "docs/evidence/checkpoints/records/CP287.json").read_bytes())
    pointer, = (row for row in checkpoint["evidence_runs"] if row["run"] == RUN)
    header, *rows = pinned(archive, pointer).splitlines()
    if json.loads(header) != {"schema": "kre-evidence-run-v1", "run": RUN} or len(rows) != pointer["count"]:
        raise ValueError("invalid CP287 manifest")
    files = {r["path"]: r for r in [*checkpoint["evidence"], *map(json.loads, rows)]}

    def read(path):
        return pinned(archive, files[path])

    verification = json.loads(read(RECEIPT))
    clients = {}
    for name in ACTORS:
        stem = f"{RUN}/native/cycles/loopback-{name}"
        clients[name] = summarize(read(stem + ".pcap"), json.loads(read(stem + ".json")),
                                  verification["native"][name])
    windows = [r for rounds in clients.values() for r in rounds]
    rates = [r["packets_per_second"] for r in windows]
    return dict(schema="kr-m8-native-udp-profile-v1", source="kr_runtime", status="candidate", checkpoint="CP287",
                service_sha256=verification["service_binary_sha256"],
                companion_sha256=verification["companion_binary_sha256"],
                clients=clients, windows=len(windows), mean_window_rate_range_pps=[min(rates), max(rates)],
                median_window_rate_pps=round(statistics.median(rates), 3),
                max_rolling_1s_packets=max(r["peak_rolling_1s_packets"] for r in windows),
                limits="Historical native 4P loopback; egress to companion proxies, not outer QUIC or a 50P workload.")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--archive", type=Path, required=True, help="Repository root containing the archived local tree")
    parser.add_argument("--out", type=Path, required=True, help="New receipt under this checkout's local directory")
    args = parser.parse_args()
    out = args.out.resolve()
    if not out.is_relative_to(ROOT / "local"):
        parser.error("--out must be inside this checkout's local directory")
    result = measure(args.archive.resolve(strict=True))
    with out.open("x", encoding="utf-8") as file:
        json.dump(result, file, indent=2)
        file.write("\n")


if __name__ == "__main__":
    main()
