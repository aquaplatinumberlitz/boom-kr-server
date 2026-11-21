"""Record Linux process resources and M8 health during a separately driven load run."""
import argparse
import json
import os
from pathlib import Path
import sys
import time


class SampleError(ValueError):
    """A sample must not silently cross process, clock or service generations."""


def metadata(run):
    with (run / "rust-events.jsonl").open(encoding="utf-8") as log:
        for _ in range(64):
            line = log.readline(65537)
            if not line or len(line) > 65536:
                break
            event = json.loads(line)
            if event.get("event") == "boot":
                pid, boot = event.get("pid"), event.get("boot")
                if type(pid) is int and pid > 0 and isinstance(boot, str) and boot.isascii() and boot.isdecimal():
                    return pid, boot
                break
    raise SampleError("boot event missing or invalid")


def proc_stat(stat, expected_pid):
    # comm, the parenthesized second field, may contain spaces and parentheses.
    prefix, sep, tail = stat.rpartition(")")
    if not sep or not prefix.startswith(f"{expected_pid} ("):
        raise SampleError("unexpected /proc stat process")
    fields = tail.split()
    if len(fields) < 22 or fields[0] in {"Z", "X", "x"}:
        raise SampleError("process unavailable")
    return int(fields[19]), int(fields[11]) + int(fields[12]), int(fields[21])


class Collector:
    def __init__(self, run, proc=Path("/proc")):
        self.run = run
        self.pid, self.boot = metadata(run)
        self.proc = proc / str(self.pid)
        self.hertz = os.sysconf("SC_CLK_TCK")
        self.page_size = os.sysconf("SC_PAGE_SIZE")
        self.start = None
        self.previous = None

    def read_stat(self):
        return proc_stat((self.proc / "stat").read_text(encoding="utf-8", errors="replace"), self.pid)

    def sample(self, stamp_ns, utc):
        before = self.read_stat()
        state = json.loads((self.run / "service-state.json").read_bytes())
        health = json.loads((self.run / "health.json").read_bytes())
        with os.scandir(self.proc / "fd") as entries:
            fds = sum(1 for _ in entries)
        start, ticks, rss_pages = self.read_stat()
        if before[0] != start or (self.start is not None and start != self.start):
            raise SampleError("process generation changed")
        if state.get("ready") is not True or state.get("stopped") is not False:
            raise SampleError("service stopped or not ready")
        if health.get("schema") != "kr-health-v1" or health.get("boot") != self.boot:
            raise SampleError("health sample is from a different boot")
        at = health.get("at_utc")
        if type(at) is not int or not 0 <= utc - at <= 15:
            raise SampleError("health sample is stale or from the future")
        lobby, relay = health.get("lobby"), health.get("relay")
        if health.get("healthy") is not True or not isinstance(lobby, dict) or not isinstance(relay, dict):
            raise SampleError("unhealthy service or missing relay")
        gauges = {
            "online_players": lobby.get("online_players"),
            "rooms": lobby.get("room_count"),
            "active_rounds": lobby.get("active_rounds"),
            "udp_listeners": relay.get("udp_listeners"),
            "companions": relay.get("companions"),
            "reserved_routes": relay.get("reserved_routes"),
            "database_probe_errors": health.get("database_probe_errors"),
        }
        if any(type(value) is not int or value < 0 for value in gauges.values()) or rss_pages < 0:
            raise SampleError("invalid resource counter")
        cpu = None
        if self.previous is not None:
            last_ns, last_ticks, last_db_errors = self.previous
            if stamp_ns <= last_ns or ticks < last_ticks or gauges["database_probe_errors"] < last_db_errors:
                raise SampleError("time or cumulative counter reversed")
            cpu = round(100 * (ticks - last_ticks) * 1e9 / (self.hertz * (stamp_ns - last_ns)), 3)
        self.start = start
        self.previous = stamp_ns, ticks, gauges["database_probe_errors"]
        return dict(at_utc=utc, health_age_seconds=utc - at, cpu_one_core_pct=cpu,
                    rss_bytes=rss_pages * self.page_size, fd_count=fds, **gauges)


def collect(run, output, period, baseline, load, recovery):
    collector = Collector(run)
    started = time.monotonic_ns()
    deadline = started
    count = 0
    samples_by_phase = {"baseline": 0, "load": 0, "recovery": 0}
    with output.open("x", encoding="utf-8") as file:
        def write(row):
            file.write(json.dumps(row, separators=(",", ":")) + "\n")
            file.flush()

        write(dict(schema="kr-m8-soak-v1", boot=collector.boot, interval_seconds=period,
                   baseline_seconds=baseline, load_seconds=load, recovery_seconds=recovery))
        try:
            while True:
                delay = (deadline - time.monotonic_ns()) / 1e9
                if delay > 0:
                    time.sleep(delay)
                now = time.monotonic_ns()
                elapsed = (now - started) / 1e9
                if now - deadline >= period * 1_000_000_000:
                    raise SampleError("missed sample interval")
                if elapsed >= baseline + load + recovery:
                    break
                phase = "baseline" if elapsed < baseline else "load" if elapsed < baseline + load else "recovery"
                row = collector.sample(now, int(time.time()))
                write(dict(phase=phase, elapsed_seconds=round(elapsed, 3), **row))
                count += 1
                samples_by_phase[phase] += 1
                deadline = now + period * 1_000_000_000
            if any(samples < 2 for samples in samples_by_phase.values()):
                raise SampleError("fewer than two samples in a phase")
        except (OSError, ValueError, KeyboardInterrupt):
            write(dict(event="aborted", samples=count))
            raise
        write(dict(event="finished", samples=count, samples_by_phase=samples_by_phase))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--run", type=Path, required=True)
    parser.add_argument("--out", type=Path, required=True)
    parser.add_argument("--interval", type=int, default=5)
    parser.add_argument("--baseline", type=int, default=60)
    parser.add_argument("--load", type=int, default=86400)
    parser.add_argument("--recovery", type=int, default=60)
    args = parser.parse_args()
    if not 5 <= args.interval <= 60 or any(
        not 3 * args.interval <= v <= 86400 for v in (args.baseline, args.load, args.recovery)
    ):
        parser.error("each phase must last at least three intervals and at most 86400 seconds")
    try:
        collect(args.run, args.out, args.interval, args.baseline, args.load, args.recovery)
    except (OSError, ValueError) as error:
        sys.exit(str(error))


if __name__ == "__main__":
    main()
