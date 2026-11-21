"""Synthetic controls for direction, round boundaries and invalid captures."""
import struct
import unittest

from m8_udp_profile import SECOND, pcap_records, summarize


def capture(events):
    raw = bytearray(struct.pack("<IHHIIII", 0xa1b2c3d4, 2, 4, 0, 0, 65535, 1))
    rows = []
    for stamp, packet_type, source, destination in events:
        body = b"pattern"
        length = len(body) + 8
        frame = (b"\x00" * 12 + b"\x08\x00" + struct.pack("!BBHHHBBHII", 0x45, 0, 20 + length,
                 0, 0, 64, 17, 0, 0x7f000001, 0x7f000001)
                 + struct.pack("!HHHH", source, destination, length, 0) + body)
        sec, nanos = divmod(stamp, SECOND)
        raw.extend(struct.pack("<IIII", sec, nanos // 1000, len(frame), len(frame)) + frame)
        rows.append(dict(monotonic_ns=stamp, epoch_ns=stamp, packet_type=packet_type))
    return bytes(raw), {"rows": rows}


class ProfileTest(unittest.TestCase):
    def setUp(self):
        self.rounds = dict(clock_ns=[SECOND, 4 * SECOND, 8 * SECOND], rank_ns=[3 * SECOND, 6 * SECOND],
                           native_proxy_ports={"1": {"proxy": 3001}, "2": {"proxy": 3002}})
        self.raw, self.metadata = capture([
            (0, 4, 9000, 3001),                 # before clock
            (SECOND, 0, 9000, 3001),            # mirrored copy
            (SECOND, 4, 9000, 3001),            # include clock
            (2 * SECOND, 4, 3001, 9000),        # direction toward client
            (2 * SECOND, 4, 9000, 3001),        # outside first rolling second
            (3 * SECOND, 4, 9000, 3001),        # exclude result boundary
            (4 * SECOND, 4, 9000, 3001),        # stale proxy after reset
            (4 * SECOND, 4, 9000, 3002),
            (4 * SECOND + 2000, 4, 9000, 3002),
            (8 * SECOND, 4, 9000, 3002),        # unfinished third round
        ])

    def test_counts_only_native_egress_in_two_finished_rounds(self):
        first, second = summarize(self.raw, self.metadata, self.rounds)
        self.assertEqual([r["sent"] for r in [first, second]], [2, 2])
        self.assertEqual([r["peak_rolling_1s_packets"] for r in [first, second]], [1, 2])
        self.assertEqual(second["payload_bytes"], {7: 2})
        self.assertEqual(first["per_peer_packets_per_second"], [1.0])

    def test_rejects_wrong_sidecar_and_partial_pcap(self):
        self.metadata["rows"][0]["epoch_ns"] += 1000
        with self.assertRaisesRegex(ValueError, "sidecar disagree"):
            summarize(self.raw, self.metadata, self.rounds)
        with self.assertRaisesRegex(ValueError, "truncated"):
            list(pcap_records(self.raw[:-1]))


if __name__ == "__main__":
    unittest.main()
