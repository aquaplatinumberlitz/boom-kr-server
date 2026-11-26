"""Kiểm hành vi audit từ fixture metadata; không dùng pack hay client."""

from copy import deepcopy
import hashlib
import json
import unittest

from kr_f2_mode_audit import build_catalog, render_json, validate_classification


def map_row(name, family, *, category="mission", **extra):
    return dict(name=name, family=family, category=category, width=15, height=13,
                spawns=4, d1=2, d2=2, parse="exact", scripted=False, drops=[], placements=[]) | extra


def fixture():
    census = {
        "build": "KR00967",
        "maps": [
            map_row("Kart01", "Kart", code=123, drops=[["Speed", 10], ["Boot", 0]],
                    placements=["Speed", "Speed"], scripted=True),
            map_row("Kart02", "Kart", spawns=8, width=19, d2=4, parse="partial",
                    drops=[["Speed", 1]], placements=["Boot"]),
            map_row("PopcadeA01", "PopcadeA"),
            map_row("ConfusedFight01", "ConfusedFight", category="other"),
            map_row("Tomb12", "Tomb", category="battle_theme"),
        ],
        "resource_groups": [
            dict(pack="FxContents", name="ConfusedFight", entries=3, system="ConfusedFight"),
            dict(pack="FxMap", name="Map", entries=4, system="Map"),
        ],
        "systems": [dict(name="ConfusedFight", ids=["KR-TEST"]), dict(name="Map", ids=[])],
        "map_groups": [dict(name="ReBirthMapGroup", maps=["Tomb?", "Cemetery?", "Tomb12"]),
                       dict(name="LadderMapGroup", maps=["Village10"])],
    }
    classification = {"rows": [
        dict(kind="map", name="Kart", map_count=2, classification="B", status="candidate"),
        dict(kind="mode", name="팝케이드 (Popcade)", wave="F3", classification="C", status="candidate"),
        dict(kind="system", name="ConfusedFight", wave="F2", classification="C"),
        dict(kind="ship", name="Kart", classification="A"),
        dict(kind="ship", name="Kart", classification="A"),
    ]}
    return census, classification


def audit(census, classification=None, **limits):
    return build_catalog(census, classification, **({"expected_maps": 3, "expected_families": 2} | limits))


class ModeAuditTests(unittest.TestCase):
    def test_family_uses_observed_rows_and_leaves_partial_script_unknown(self):
        catalog = audit(*fixture())
        kart = catalog["families"][0]
        self.assertEqual(kart["maps"], [{"name": "Kart01", "code": 123}, {"name": "Kart02"}])
        self.assertEqual(kart["grid_sizes"], [(15, 13), (19, 13)])
        self.assertEqual(kart["spawn_range"], [4, 8])
        self.assertEqual(kart["d1_d2_pairs"], [(2, 2), (2, 4)])
        self.assertEqual(kart["has_script"], {"yes": 1, "no": 0, "unknown": 1})
        self.assertEqual(kart["drops"]["exact"], {
            "maps_with_rows": 1, "rows": 2, "positive_rows": 1, "distinct_positive_names": 1,
        })
        self.assertEqual(kart["drops"]["partial_observed"]["rows"], 1)
        self.assertEqual(kart["placements"]["exact"]["by_name"], {"Speed": 2})
        self.assertEqual(kart["placements"]["partial_observed"]["by_name"], {"Boot": 1})
        self.assertEqual(kart["classification"]["classification"], "B")
        self.assertEqual(catalog["summary"]["classification_counts"], {"A": 0, "B": 1, "C": 0, "unknown": 1})

    def test_mode_evidence_does_not_change_scope_or_promote_family_grade(self):
        census, classification = fixture()
        catalog = audit(census, classification)
        modes = {row["ruleset_group"]: row for row in catalog["modes"]}
        self.assertEqual(catalog["summary"]["total_maps"], 3)
        self.assertEqual(modes["confused_fight"]["related_families"],
                         [{"family": "ConfusedFight", "map_count": 1, "mission_map_count": 0}])
        self.assertEqual(modes["confused_fight"]["pack"]["resource_groups"], [{
            "locator": "FxContents/ConfusedFight", "entries": 3, "system": "ConfusedFight",
            "system_ids": ["KR-TEST"], "system_classification": {"classification": "C", "wave": "F2"},
        }])
        revive_group = modes["revive"]["pack"]["map_groups"][0]
        self.assertEqual(revive_group["entries"], ["Tomb?", "Cemetery?", "Tomb12"])
        self.assertEqual(revive_group["exact_census_matches"], [
            {"entry": "Tomb12", "category": "battle_theme", "family": "Tomb"},
        ])
        self.assertIsNone(modes["revive"]["wire_mode"])
        self.assertEqual(modes["popcade"]["wave"], "F3")
        self.assertEqual(modes["popcade"]["classification"]["classification"], "C")
        self.assertIsNone(catalog["families"][1]["classification"])
        self.assertEqual(catalog["families"][1]["grouping"]["basis"], "wiki_inferred")
        self.assertEqual(audit(census)["summary"]["classification_counts"]["unknown"], 2)

    def test_invariants_and_canonical_hash(self):
        census, classification = fixture()
        catalog = audit(census, classification)
        serialized = render_json(catalog)
        reordered = deepcopy(census)
        for rows in reordered.values():
            if isinstance(rows, list):
                rows.reverse()
        self.assertEqual(serialized, render_json(audit(reordered, {"rows": classification["rows"][::-1]})))
        reordered["map_groups"][-1]["maps"].reverse()
        self.assertNotEqual(catalog["catalog_sha256"], audit(reordered, classification)["catalog_sha256"])

        payload = json.loads(serialized)
        digest = payload.pop("catalog_sha256")
        canonical = json.dumps(payload, separators=(",", ":"), sort_keys=True, ensure_ascii=False) + "\n"
        self.assertEqual(digest, hashlib.sha256(canonical.encode("utf-8")).hexdigest())
        self.assertEqual(payload["summary"]["duplicate_map_entries"], 0)
        with self.assertRaisesRegex(ValueError, "122 map / 18 họ"):
            build_catalog(census, classification)
        with self.assertRaisesRegex(ValueError, "3 map / 1 họ"):
            audit(census, classification, expected_families=1)
        with self.assertRaisesRegex(ValueError, "trùng tên map"):
            audit(census | {"maps": [*census["maps"], map_row("kart01", "Other", category="other")]})
        private = deepcopy(census)
        private["maps"][0]["code"] = "/home/test/game"
        with self.assertRaisesRegex(ValueError, "dữ liệu riêng|đường dẫn máy"):
            audit(private)

    def test_classification_must_match_declared_input(self):
        census, classification = fixture()
        validate_classification(census, "source-hash", classification)
        with self.assertRaisesRegex(ValueError, "census_sha256"):
            validate_classification(census, "other-hash", classification | {"census_sha256": "source-hash"})


if __name__ == "__main__":
    unittest.main()
