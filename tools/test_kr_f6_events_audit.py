"""Kiểm hành vi F6-0 bằng fixture metadata, không đọc pack hay client."""

from copy import deepcopy
import hashlib
import json
import unittest

from tools.kr_f6_events_audit import build_catalog, render_json, render_markdown, validate_classification


def fixture():
    source_rows = [
        ("Nhiệm vụ", "F6", [("FxContents", "DailyMission", 22), ("FxDesktop", "Quest", 59)]),
        ("CAPass", "F6", [("FxContents", "CAPass", 9)]),
        ("Câu cá", "F6", [("fx", "FishingObjects", 70)]),
        ("PCRoomBenefit", "ngoài phạm vi", [("fx", "PCRoomBenefit", 16)]),
        ("Antarctic", "F2", [("FxContents", "Antarctic", 15)]),
        ("Bộ sưu tập item", "F4", [("FxContents", "ItemCollection", 27)]),
        ("IamWriggle", "F3", [("FxContents", "IamWriggle", 7)]),
        ("Monster", "F3", [("FxMonster", "MonsterSeason2", 236)]),
        ("SnowyChamp", "F2", [("FxContents", "SnowyChamp", 6)]),
        ("Chat", "F5", [("FxDesktop", "Chatting", 5)]),
    ]
    census = {"build": "KR00967", "client_exe_sha256": "a" * 64, "packs": {},
              "systems": [], "resource_groups": []}
    classification = {"build": "KR00967", "census_sha256": "b" * 64,
                      "client_sha256": "a" * 64, "rows": []}
    for name, wave, groups in source_rows:
        census["systems"].append({"name": name, "entries": sum(entries for _, _, entries in groups),
                                  "groups": [f"{pack}/{group}" for pack, group, _ in groups], "wiki": []})
        for pack, group, entries in groups:
            census["packs"][pack] = {"idx_sha256": "c" * 64, "entries": 999}
            census["resource_groups"].append({"pack": pack, "name": group, "entries": entries, "system": name})
        classification["rows"].append({"kind": "system", "name": name, "wave": wave,
                                       "classification": None if name == "PCRoomBenefit" else "C",
                                       "status": "design" if name == "PCRoomBenefit" else "candidate"})
    classification["rows"].append({"kind": "mode", "name": "Chat", "wave": "F6"})
    return census, classification


class EventsAuditTests(unittest.TestCase):
    def test_scope_and_resource_groups(self):
        catalog = build_catalog(*fixture())
        rows = {row["system"]: row for row in catalog["systems"]}
        self.assertEqual({name for name, row in rows.items() if row["scope"] == "in_scope"},
                         {"CAPass", "Câu cá", "Nhiệm vụ"})
        self.assertNotIn("Chat", rows)
        self.assertEqual(rows["Nhiệm vụ"]["static_metadata"], {
            "entries": 81, "wiki_labels": [], "resource_groups": [
                {"locator": "FxContents/DailyMission", "entries": 22},
                {"locator": "FxDesktop/Quest", "entries": 59},
            ],
        })
        self.assertEqual(rows["SnowyChamp"]["scope"], "related_candidate")
        self.assertEqual(rows["SnowyChamp"]["classification"]["wave"], "F2")
        self.assertEqual(catalog["summary"], {"census_systems": 10, "in_scope": 3,
                                             "related_candidate": 5, "excluded_policy": 1})

    def test_pc_bang_is_visible_but_excluded(self):
        census, classification = fixture()
        catalog = build_catalog(census, classification)
        excluded, = (row for row in catalog["systems"] if row["scope"] == "excluded_policy")
        self.assertEqual(excluded["system"], "PCRoomBenefit")
        self.assertEqual(excluded["policy"]["status"], "excluded_policy")
        self.assertEqual(excluded["policy"]["locator"], "docs/PLAN.md#full-game-roadmap")
        self.assertIn("Nexon (PC-bang)", render_markdown(catalog))

        row = next(row for row in classification["rows"] if row["name"] == "PCRoomBenefit")
        row["wave"] = "F6"
        with self.assertRaisesRegex(ValueError, "PCRoomBenefit"):
            build_catalog(census, classification)

    def test_wire_prices_rates_and_activation_remain_unknown(self):
        census, classification = fixture()
        census["systems"][0].update(kr_wire=123, price=90, active_in_build=True)
        census["systems"][0]["wiki"] = ["Nhãn thử (đã ngừng)"]
        catalog = build_catalog(census, classification)
        for row in catalog["systems"]:
            with self.subTest(system=row["system"]):
                for key in ("kr_wire", "original_server_prices_and_rates", "active_in_build"):
                    self.assertEqual(row[key], "unknown")
        self.assertIn("đều là `unknown`", render_markdown(catalog))

    def test_idempotence_and_canonical_hash(self):
        census, classification = fixture()
        catalog = build_catalog(census, classification)
        reordered = deepcopy(census)
        reordered["systems"].reverse()
        reordered["resource_groups"].reverse()
        for row in reordered["systems"]:
            row["groups"].reverse()
        again = build_catalog(reordered, {"rows": classification["rows"][::-1]})
        raw = render_json(catalog)
        self.assertEqual(raw, render_json(again))
        self.assertEqual(render_markdown(catalog), render_markdown(again))

        payload = json.loads(raw)
        digest = payload.pop("catalog_sha256")
        canonical = json.dumps(payload, ensure_ascii=False, sort_keys=True, separators=(",", ":")) + "\n"
        self.assertEqual(digest, hashlib.sha256(canonical.encode("utf-8")).hexdigest())
        next(row for row in reordered["systems"] if row["name"] == "Nhiệm vụ")["wiki"] = ["Nhãn mới"]
        self.assertNotEqual(digest, build_catalog(reordered, classification)["catalog_sha256"])

    def test_rejects_private_metadata(self):
        tokens = ("/home/test/game", "/Users/test/game", "10.5.4.3", "172.31.7.9", "192.168.4.2",
                  "198." + "18.1.2", "ocid" + "1.example", "BEGIN TEST PRIVATE" + " KEY")
        for token in tokens:
            with self.subTest(token=token):
                census, classification = fixture()
                census["systems"][0]["wiki"] = [token]
                with self.assertRaisesRegex(ValueError, "đường dẫn máy") as exc:
                    build_catalog(census, classification)
                self.assertNotIn(token, str(exc.exception))

    def test_inconsistent_group_is_not_published(self):
        census, classification = fixture()
        census["resource_groups"][0]["entries"] += 1
        with self.assertRaisesRegex(ValueError, "số mục tài nguyên không khớp"):
            build_catalog(census, classification)

    def test_classification_matches_saved_census(self):
        census, classification = fixture()
        validate_classification(census, "b" * 64, classification)
        for key in ("build", "census_sha256", "client_sha256"):
            with self.subTest(key=key), self.assertRaisesRegex(ValueError, key):
                validate_classification(census, "b" * 64, classification | {key: "out-of-date"})


if __name__ == "__main__":
    unittest.main()
