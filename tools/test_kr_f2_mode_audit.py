"""Kiểm hành vi audit từ fixture metadata; không dùng pack hay client."""

from copy import deepcopy
import hashlib
import json
from pathlib import Path
import sys
import unittest

sys.path.insert(0, str(Path(__file__).parent))
from kr_f2_mode_audit import (
    CENSUS, CLASSIFICATION, RULES, build_catalog, read_json, render_json, render_markdown,
    validate_classification, validate_rule_group_parity,
)


FIXTURE_RULES = [
    {"group": group, "wave": wave}
    for wave, groups in (
        ("F2", (
            "revive", "timebomb", "leader_capture", "hedgehog", "co_op", "free_1v7",
            "ladder", "matchmaking", "confused_fight", "snowball_event", "super_fighter",
        )),
        ("F2?", ("airplane", "antarctic", "baseball", "big_battle", "snowy_champ", "snowy_fields")),
        ("F3", ("popcade",)),
    )
    for group in groups
]


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
    return build_catalog(census, classification, rules=FIXTURE_RULES,
                         **({"expected_maps": 3, "expected_families": 2} | limits))


class ModeAuditTests(unittest.TestCase):
    def test_partial_tail_diagnostics_copy_only_census_metadata(self):
        census, classification = fixture()
        census["maps"].extend([
            map_row("Bula01", "Bula", parse="partial", drops=[["Item/ItemBond", 100]],
                    script_tags=[], script_funcs=[], script_triggers=[]),
            map_row("Bula00", "Bula", parse="partial", drops=[["Item/ItemBond", 100]], placements=["Boot"],
                    partial_reason="missing trailer", missing_tags=["time"],
                    tail={"offset": 99, "remaining_bytes": 7}),
            map_row("ExBula01", "ExBula", category="battle_variant", parse="partial"),
        ])
        catalog = audit(census, classification, expected_maps=5, expected_families=3)
        bula = catalog["families"][0]
        self.assertEqual(bula["partial_maps"], ["Bula00", "Bula01"])
        self.assertEqual(bula["has_script"], {"yes": 0, "no": 0, "unknown": 2})
        known, unknown = bula["partial_tail_diagnostics"]
        self.assertEqual(known, {
            "map": "Bula00",
            "reason": "missing trailer",
            "missing_tags": ["time"],
            "tail": {"offset": 99, "remaining_bytes": 7},
            "observed": {
                "drop_rows": 1, "placement_rows": 1,
                "script_tags": None, "script_funcs": None, "script_triggers": None,
            },
        })
        self.assertEqual(unknown, {
            "map": "Bula01", "reason": None, "missing_tags": None, "tail": None,
            "observed": {"drop_rows": 1, "placement_rows": 0,
                         "script_tags": [], "script_funcs": [], "script_triggers": []},
        })
        report = render_markdown(catalog)
        self.assertIn('| Bula / `Bula01` | 1/0 | [] | — | — | — |', report)
        self.assertIn('"remaining_bytes":7', report)
        self.assertNotIn("ExBula01", report)

    def test_partial_tail_diagnostics_reject_invalid_metadata(self):
        for key, value in (
            ("tail", {"remaining_bytes": -1}),
            ("tail", {"offset": True}),
            ("tail", {"unexpected": 5}),
            ("missing_tags", "time"),
            ("partial_reason", [4]),
        ):
            with self.subTest(key=key, value=value):
                census, classification = fixture()
                census["maps"][1][key] = value
                with self.assertRaisesRegex(ValueError, key):
                    audit(census, classification)
        census["maps"][1]["partial_reason"] = "/home/test/trailer"
        with self.assertRaisesRegex(ValueError, "đường dẫn máy"):
            audit(census, classification)

    def test_rule_group_parity_rejects_drift_and_duplicate_links(self):
        census, classification = fixture()
        census["maps"].append(map_row("SnowyFields01", "SnowyFields"))
        catalog = audit(census, classification, expected_maps=4, expected_families=3)
        rules = FIXTURE_RULES
        self.assertEqual(catalog["rule_group_parity"]["mode_count"], 18)
        self.assertEqual(catalog["rule_group_parity"]["linked_mission_families"], 2)
        self.assertEqual(catalog["families"][-1]["grouping"]["ruleset_group"], "snowy_fields")
        drift = [rules[0] | {"wave": "F3"}, *rules[1:]]
        for table in (rules[:-1], [*rules, rules[0]], drift):
            with self.assertRaisesRegex(ValueError, "nhóm luật"):
                validate_rule_group_parity(catalog, table)

        wrong_group = deepcopy(catalog)
        wrong_group["families"][-1]["grouping"]["ruleset_group"] = "snowball_event"
        with self.assertRaisesRegex(ValueError, "họ SnowyFields"):
            validate_rule_group_parity(wrong_group, rules)
        missing_ref = deepcopy(catalog)
        missing_ref["families"][-1]["grouping"]["mode_refs"].clear()
        with self.assertRaisesRegex(ValueError, "họ SnowyFields"):
            validate_rule_group_parity(missing_ref, rules)
        duplicate_link = deepcopy(catalog)
        refs = next(row for row in duplicate_link["modes"] if row["mode"] == "SnowyFields")["related_families"]
        refs.append(refs[0])
        with self.assertRaisesRegex(ValueError, "trùng liên kết"):
            validate_rule_group_parity(duplicate_link, rules)

    def test_checked_in_rules_match_modes_and_family_only_groups(self):
        census, _ = read_json(CENSUS)
        classification, _ = read_json(CLASSIFICATION)
        rules, _ = read_json(RULES)
        catalog = build_catalog(census, classification, rules=rules)

        without_farm = [row for row in rules if row["group"] != "farm_mission"]
        extra = {"group": "unexpected", "wave": "F2?"}
        for table in (without_farm, [*without_farm, {"group": "farm_mission", "wave": "F2"}], [*rules, extra]):
            with self.assertRaisesRegex(ValueError, "nhóm luật"):
                validate_rule_group_parity(catalog, table)

        rabbit = next(row for row in catalog["families"] if row["family"] == "Rabbit")
        rabbit["grouping"]["ruleset_group"] = "rabbit"
        with self.assertRaisesRegex(ValueError, "họ Rabbit"):
            validate_rule_group_parity(catalog, rules)

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
            build_catalog(census, classification, rules=FIXTURE_RULES)
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
