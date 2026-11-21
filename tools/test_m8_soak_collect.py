"""Offline test data for the process identity and health joins."""
import json
import os
from pathlib import Path
from tempfile import TemporaryDirectory
import unittest
from unittest.mock import patch

from m8_soak_collect import Collector, SampleError, collect


class SoakTest(unittest.TestCase):
    def setUp(self):
        temporary = TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.run = Path(temporary.name)
        self.proc = self.run / "300"
        (self.proc / "fd").mkdir(parents=True)
        (self.proc / "fd" / "0").touch()
        (self.proc / "fd" / "1").touch()
        self.write("rust-events.jsonl", {"event": "boot", "pid": 300, "boot": "55"})
        self.write("service-state.json", {"ready": True, "stopped": False})
        self.health = dict(schema="kr-health-v1", boot="55", at_utc=100, healthy=True,
                           lobby=dict(online_players=8, room_count=1, active_rounds=1),
                           relay=dict(udp_listeners=1, companions=8, reserved_routes=56),
                           database_probe_errors=0)
        self.write("health.json", self.health)
        self.stat(20)
        self.collector = Collector(self.run, self.run)

    def write(self, name, value):
        (self.run / name).write_text(json.dumps(value) + "\n", encoding="utf-8")

    def stat(self, ticks, start=23):
        fields = ["0"] * 22
        fields[0], fields[11], fields[12], fields[19], fields[21] = "S", str(ticks), "10", str(start), "100"
        (self.proc / "stat").write_text("300 (spaces ) and ( brackets) " + " ".join(fields), encoding="utf-8")

    def test_resources_and_delta(self):
        first = self.collector.sample(1_000_000_000, 100)
        self.assertEqual(first["fd_count"], 2)
        self.assertEqual(first["rss_bytes"], 100 * self.collector.page_size)
        self.assertEqual(first["reserved_routes"], 56)
        self.assertIsNone(first["cpu_one_core_pct"])
        self.stat(20 + self.collector.hertz)
        next_sample = self.collector.sample(3_000_000_000, 102)
        self.assertEqual(next_sample["cpu_one_core_pct"], 50)
        self.assertEqual(next_sample["health_age_seconds"], 2)

    def test_rejects_stale_health_wrong_boot_and_reused_pid(self):
        self.collector.sample(1_000_000_000, 100)
        with self.assertRaisesRegex(SampleError, "stale"):
            self.collector.sample(2_000_000_000, 116)
        self.write("health.json", self.health | {"boot": "56"})
        with self.assertRaisesRegex(SampleError, "different boot"):
            self.collector.sample(2_000_000_000, 100)
        self.write("health.json", self.health)
        self.stat(22, start=24)
        with self.assertRaisesRegex(SampleError, "process generation"):
            self.collector.sample(2_000_000_000, 100)

    def test_rejects_shutdown(self):
        self.write("service-state.json", {"ready": False, "stopped": True})
        with self.assertRaisesRegex(SampleError, "not ready"):
            self.collector.sample(1_000_000_000, 100)


    def test_real_proc_with_synthetic_health(self):
        self.write("rust-events.jsonl", {"event": "boot", "pid": os.getpid(), "boot": "55"})
        sample = Collector(self.run).sample(1_000_000_000, 100)
        self.assertGreater(sample["fd_count"], 0)
        self.assertGreater(sample["rss_bytes"], 0)

    def test_phase_log_and_no_overwrite(self):
        out = self.run / "samples.jsonl"
        stamps = iter([0, *(t for delta in range(0, 50, 5) for t in [delta, delta])])
        with (
            patch("m8_soak_collect.Collector", return_value=self.collector),
            patch("m8_soak_collect.time.monotonic_ns", side_effect=lambda: next(stamps) * 1_000_000_000),
            patch("m8_soak_collect.time.time", return_value=100),
        ):
            collect(self.run, out, 5, 15, 15, 15)
        rows = [json.loads(line) for line in out.read_text().splitlines()]
        self.assertEqual(rows[-1]["samples_by_phase"], {"baseline": 3, "load": 3, "recovery": 3})
        self.assertEqual(rows[1]["phase"], "baseline")
        with self.assertRaises(FileExistsError):
            collect(self.run, out, 5, 15, 15, 15)

    def test_overdue_sample_aborts(self):
        out = self.run / "overdue.jsonl"
        with (
            patch("m8_soak_collect.Collector", return_value=self.collector),
            patch("m8_soak_collect.time.monotonic_ns", side_effect=[0, 0, 5_000_000_000]),
        ):
            with self.assertRaisesRegex(SampleError, "missed sample"):
                collect(self.run, out, 5, 15, 15, 15)
        self.assertEqual(json.loads(out.read_text().splitlines()[-1]), {"event": "aborted", "samples": 0})
