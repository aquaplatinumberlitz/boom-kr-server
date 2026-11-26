use super::*;
use crate::room_modes::{CATALOG, Catalog, Config, Policy};
use kr_wire::kr00967::{RoomTeamSelector, Start121Map};
use serde_json::{Value, json};

#[test]
fn every_catalog_mode_resolves_but_cannot_dispatch() {
    let catalog: Catalog = serde_json::from_str(CATALOG).unwrap();
    let table = RuleTable::load(&catalog.modes).unwrap();
    assert_eq!(table.0.len(), 18);
    for mode in &catalog.modes {
        assert_eq!(
            table.check_start(mode.ruleset_group),
            if mode.ruleset_group == ModeKind::Popcade {
                Err("mode_outside_f2")
            } else {
                Err("f2_start_dispatch_unmeasured")
            },
            "{:?}",
            mode.ruleset_group
        );
    }
}

#[test]
fn stateful_design_contracts_distinguish_revive_bomb_leader_and_queue() {
    let catalog: Catalog = serde_json::from_str(CATALOG).unwrap();
    let table = RuleTable::load(&catalog.modes).unwrap();
    let revive = &table.0[&ModeKind::Revive];
    assert_eq!(revive.respawn, RespawnPolicy::Required);
    assert_eq!(revive.reset, ResetPolicy::RoundAndRespawn);
    let timebomb = &table.0[&ModeKind::Timebomb];
    assert_eq!(timebomb.timer, TimerPolicy::BombCountdown);
    assert_eq!(timebomb.reset, ResetPolicy::RoundAndBomb);
    let leader = &table.0[&ModeKind::LeaderCapture];
    assert_eq!(leader.win, WinCondition::LeaderObjective);
    assert_eq!(leader.reset, ResetPolicy::RoundAndLeader);
    let queue = &table.0[&ModeKind::Matchmaking];
    assert_eq!(queue.win, WinCondition::NotApplicable);
    assert_eq!(queue.tie, TiePolicy::NotApplicable);
    assert_eq!(queue.reset, ResetPolicy::QueueHandoff);
}

#[test]
fn malformed_or_inconsistent_descriptors_are_rejected() {
    let catalog: Catalog = serde_json::from_str(CATALOG).unwrap();
    let rows: Value = serde_json::from_str(DESCRIPTORS).unwrap();
    for (group, key, value) in [
        ("revive", "respawn", json!("unknown")),
        ("timebomb", "timer", json!("unknown")),
        ("leader_capture", "reset", json!("round")),
        ("ladder", "tie", json!("not_applicable")),
        ("matchmaking", "win", json!("unknown")),
        ("matchmaking", "reset", json!("round")),
        ("hopeless", "win", json!("unknown")),
        ("popcade", "wave", json!("F2")),
        ("revive", "wire_mode", json!(23)),
        ("timebomb", "timer", json!(200)),
    ] {
        let mut invalid = rows.clone();
        let index = rows
            .as_array()
            .unwrap()
            .iter()
            .position(|row| row["group"] == group);
        if let Some(index) = index {
            invalid[index][key] = value;
        } else {
            invalid[0]["group"] = json!(group);
        }
        assert!(
            RuleTable::parse(&invalid.to_string(), &catalog.modes).is_err(),
            "{group}.{key}"
        );
    }
    let mut missing_field = rows.clone();
    missing_field[0].as_object_mut().unwrap().remove("tie");
    assert!(RuleTable::parse(&missing_field.to_string(), &catalog.modes).is_err());
}

#[test]
fn missing_and_duplicate_catalog_or_descriptor_rows_are_rejected() {
    let mut catalog: Catalog = serde_json::from_str(CATALOG).unwrap();
    let mut rows: Vec<Value> = serde_json::from_str(DESCRIPTORS).unwrap();
    let removed = rows.pop().unwrap();
    assert!(RuleTable::parse(&json!(rows).to_string(), &catalog.modes).is_err());
    let duplicate = rows[0].clone();
    rows.extend([removed, duplicate]);
    assert!(RuleTable::parse(&json!(rows).to_string(), &catalog.modes).is_err());
    catalog.modes.pop();
    assert!(RuleTable::load(&catalog.modes).is_err());
    let mut duplicate: Catalog = serde_json::from_str(CATALOG).unwrap();
    duplicate.modes[1].ruleset_group = duplicate.modes[0].ruleset_group;
    duplicate.modes[1].wave = duplicate.modes[0].wave;
    assert!(RuleTable::load(&duplicate.modes).is_err());
}

#[test]
fn normal_start_and_selector_isolation_are_preserved() {
    let default: Config = serde_json::from_value(json!({})).unwrap();
    assert!(Policy::load(&default).unwrap().is_none());
    let visible_only: Config =
        serde_json::from_value(json!({"visible_map_groups": ["LadderMapGroup"]})).unwrap();
    let policy = Policy::load(&visible_only).unwrap().unwrap();
    let mut map = Start121Map {
        map_code: 900,
        name: b"Factory07".to_vec(),
        dynamic_index: 3,
        roomtype: 0,
    };
    for team in [RoomTeamSelector::Free, RoomTeamSelector::Manner] {
        assert!(!policy.has_create_override(team));
        assert_eq!(policy.check_start(team, &map, (900, 3)), Ok(()));
        assert_eq!(
            policy.check_start(team, &map, (900, 4)),
            Err("start_map_selection_mismatch")
        );
    }
    let opt_in: Config = serde_json::from_value(json!({
        "visible_map_groups": ["LadderMapGroup"], "create_overrides": {"free": "ladder"}
    }))
    .unwrap();
    let policy = Policy::load(&opt_in).unwrap().unwrap();
    assert_eq!(
        policy.check_start(RoomTeamSelector::Free, &map, (900, 3)),
        Err("f2_start_dispatch_unmeasured")
    );
    assert_eq!(
        policy.check_start(RoomTeamSelector::Manner, &map, (900, 3)),
        Ok(())
    );
    map.roomtype = 1;
    assert_eq!(
        policy.check_start(RoomTeamSelector::Manner, &map, (900, 3)),
        Err("start_roomtype_unmeasured")
    );
}
