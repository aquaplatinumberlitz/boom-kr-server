"""Run declared Windows/Wine input after server gates, without screen matching."""
import argparse
from os import fstat
from contextlib import closing
import json
from pathlib import Path
import sqlite3
import subprocess
import time


def bounded_int(value, low, high):
    if type(value) is not int or not low <= value <= high:
        raise ValueError(f"integer required in {low}..{high}")
    return value


class Events:
    def __init__(self, path):
        self.path = path
        self.position = 0
        self.identity = None
        self.rows = []

    def poll(self):
        try:
            source = self.path.open("rb")
        except FileNotFoundError:
            if self.identity is not None:
                raise ValueError("event log disappeared during scenario") from None
            return
        with source:
            # fstat the descriptor, rather than a concurrently replaced pathname.
            stat = fstat(source.fileno())
            identity = (stat.st_dev, stat.st_ino)
            if self.identity not in (None, identity) or stat.st_size < self.position:
                raise ValueError("event log was replaced or truncated")
            self.identity = identity
            source.seek(self.position)
            while line := source.readline(1 << 16):
                if not line.endswith(b"\n"):
                    if len(line) == 1 << 16:
                        raise ValueError("event line too long")
                    break  # append writer has not finished this line
                if len(self.rows) >= 50_000:
                    raise ValueError("event count exceeds controller budget")
                row = json.loads(line)
                if not isinstance(row, dict):
                    raise ValueError("invalid event row")
                self.rows.append(row)
                self.position = source.tell()
                if row.get("event") == "stopped":
                    raise ValueError("server stopped during scenario")


class Server:
    def __init__(self, run, db):
        self.run = run
        self.db = db
        self.events = Events(run / "rust-events.jsonl")
        self.identity = None

    def ready(self):
        def read(name):
            return json.loads((self.run / name).read_text(encoding="utf-8"))
        self.events.poll()
        try:
            state = read("service-state.json")
            health = read("health.json")
        except (FileNotFoundError, json.JSONDecodeError):
            return False
        if state.get("stopped") is True:
            raise ValueError("server stopped during scenario")
        stamp = health.get("at_utc")
        if (state.get("ready") is not True or state.get("stopped") is not False or
                health.get("schema") != "kr-health-v1" or health.get("healthy") is not True or
                type(stamp) is not int or not 0 <= int(time.time()) - stamp <= 15):
            return False
        identity = (health.get("boot"), health.get("epoch"))
        if not isinstance(identity[0], str) or type(identity[1]) is not int:
            raise ValueError("health has no boot or epoch")
        if self.identity not in (None, identity):
            raise ValueError("server restarted during scenario")
        self.identity = identity
        return True

    def matches(self, gate, marks):
        if not self.ready():
            return False
        for condition in gate.get("events", []):
            count = bounded_int(condition.get("count", 1), 1, 256)
            expected = condition["match"]
            if "event" not in expected:
                raise ValueError("event condition requires an event name")
            rows = [r for r in self.events.rows[marks[condition["after"]]:]
                    if all(k in r and r[k] == v for k, v in expected.items())]
            distinct = condition.get("distinct")
            if distinct is not None:
                rows = {r[distinct] for r in rows if type(r.get(distinct)) in (str, int)}
            if len(rows) < count:
                return False
        if "journal" in gate:
            if self.db is None:
                raise ValueError("journal gate requires --db")
            journal = gate["journal"]
            predicates = ("room_id", "room_session", "round", "state")
            values = [journal[key] for key in predicates]
            if "reason" in journal:
                predicates += ("reason",)
                values.append(journal["reason"])
            query = "SELECT 1 FROM matches WHERE " + " AND ".join(
                f"{column} = ?" for column in predicates)
            try:
                with closing(sqlite3.connect(self.db.resolve().as_uri() + "?mode=ro", uri=True, timeout=0.1)) as db:
                    return db.execute(query, values).fetchone() is not None
            except sqlite3.OperationalError as error:
                if error.sqlite_errorcode in (sqlite3.SQLITE_BUSY, sqlite3.SQLITE_LOCKED):
                    return False
                raise
        return True


def action_args(action, plan_dir):
    operation = action["op"]
    if operation in {"click", "key"}:
        hold = bounded_int(action.get("hold_ms", 100), 1, 2000)
        if operation == "click":
            x, y = [bounded_int(v, 0, 16383) for v in action["xy"]]
            settle = bounded_int(action.get("settle_ms", 100), 0, 1000)
            return [operation, str(x), str(y), str(hold), str(settle)]
        return [operation, str(bounded_int(action["vk"], 1, 254)), str(hold)]
    if operation == "unicode":
        text = (plan_dir / action["text_file"]).read_text(encoding="utf-8")
        data = text.encode("utf-16le")
        if not 0 < len(data) <= 128 or any(ord(c) < 32 for c in text):
            raise ValueError("Unicode input must be 1..64 non-control UTF-16 code units")
        return [operation, data.hex(), str(bounded_int(action.get("delay_ms", 120), 0, 120))]
    raise ValueError("unknown input operation")


def prepare(plan, plan_dir, backend):
    if set(plan) != {"steps"} or not 1 <= len(plan["steps"]) <= 100:
        raise ValueError("scenario requires 1..100 steps")
    ids = {"begin"}
    prepared = []
    for step in plan["steps"]:
        label = step["id"]
        if not isinstance(label, str) or not label or label in ids:
            raise ValueError("step IDs must be unique")
        gate = step["gate"]
        if not isinstance(gate, dict) or set(gate) - {"events", "journal"}:
            raise ValueError("unknown gate")
        for condition in gate.get("events", []):
            if condition["after"] not in ids:
                raise ValueError("event gate must follow begin or an earlier step")
            bounded_int(condition.get("count", 1), 1, 256)
            if not isinstance(condition["match"].get("event"), str):
                raise ValueError("event condition requires an event name")
        if "journal" in gate:
            j = gate["journal"]
            if set(j) - {"room_id", "room_session", "round", "state", "reason"}:
                raise ValueError("unknown journal field")
            for key in ("room_id", "room_session", "round"):
                bounded_int(j[key], 0, 2**63 - 1)
            if j["state"] not in {"running", "complete", "cancelled"}:
                raise ValueError("unknown journal state")
        actions = [backend + action_args(a, plan_dir) for a in step.get("actions", [])]
        if len(actions) > 16:
            raise ValueError("at most 16 actions per step")
        prepared.append((label, bounded_int(step.get("timeout_seconds", 20), 1, 180), gate, actions))
        ids.add(label)
    return prepared


def send(command):
    # Never expose Unicode argv (or native helper output) in error messages.
    try:
        result = subprocess.run(command, stdin=subprocess.DEVNULL,
                                stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL,
                                timeout=12, check=False)
    except (OSError, subprocess.TimeoutExpired):
        raise ValueError("input helper could not complete") from None
    if result.returncode:
        raise ValueError("input helper refused input or lost foreground")


def run(server, steps, log):
    server.events.poll()
    marks = {"begin": len(server.events.rows)}
    for label, timeout, gate, actions in steps:
        end = time.monotonic() + timeout
        while not server.matches(gate, marks):
            if time.monotonic() >= end:
                raise TimeoutError(f"{label}: server gate timed out; no input sent for this step")
            time.sleep(0.1)
        marks[label] = len(server.events.rows)
        log({"step": label, "state": "gate_open"})
        for command in actions:
            if not server.ready():
                raise ValueError(f"{label}: server became unavailable after gate")
            send(command)
        log({"step": label, "state": "complete"})


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("plan", type=Path)
    parser.add_argument("--service-run", type=Path, required=True)
    parser.add_argument("--db", type=Path)
    parser.add_argument("--helper", type=Path, required=True)
    parser.add_argument("--pid", type=int, required=True, help="Win32 client PID")
    parser.add_argument("--wine", help="Wine executable; inherit the owned WINEPREFIX and DISPLAY")
    args = parser.parse_args()
    try:
        backend = ([args.wine] if args.wine else []) + [
            str(args.helper.resolve()), str(bounded_int(args.pid, 1, 2**32 - 1))]
        plan = json.loads(args.plan.read_text(encoding="utf-8"))
        steps = prepare(plan, args.plan.parent, backend)
        run(Server(args.service_run, args.db), steps, lambda row: print(json.dumps(row), flush=True))
    except (KeyError, TypeError, ValueError, OSError, sqlite3.Error) as error:
        parser.exit(1, f"Controller stopped: {error}\n")


if __name__ == "__main__":
    main()
