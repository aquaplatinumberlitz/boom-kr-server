use super::*;
use crate::map_profile::{ChoiceConfig, Profile};
use serde_json::{Value, json};
use std::{
    collections::HashMap,
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};

const SPACE_228: [u8; 67] = [
    0xa5, 0x64, 0xf3, 0xa6, 0x3a, 0xe1, 0xe5, 0x95, 0x65, 0xa6, 0xff, 0xbe, 0x32, 0xe1, 0x39, 0xa5,
    0x6f, 0xf3, 0xa6, 0x3a, 0xe1, 0x95, 0x59, 0xa5, 0x6f, 0xf3, 0xa6, 0x3a, 0xe1, 0x94, 0x59, 0xa5,
    0x6d, 0xfa, 0xa6, 0x32, 0xf3, 0x96, 0xca, 0x54, 0x45, 0x96, 0xca, 0x53, 0xcf, 0x96, 0xca, 0x53,
    0x91, 0x96, 0x35, 0x53, 0x95, 0x96, 0xca, 0x53, 0x94, 0x69, 0x35, 0xac, 0x6a, 0x96, 0xca, 0x53,
    0x95, 0x5b, 0x5a,
];

const PANGLAND_148_2CELLS: [u8; 20] = [
    0x69, 0x35, 0xac, 0x6a, 0x58, 0xa5, 0x78, 0xa4, 0x4b, 0x5b, 0x96, 0xca, 0x53, 0x94, 0xa5, 0x6a,
    0x5d, 0x58, 0x5e, 0x5b,
];

fn armed_room(allow_f1e_dynamic_scripts: bool) -> (Shared, Session, Session, Membership, u64) {
    let shared = registry_foundation_tests::empty(0);
    let mut a = Session::open(&shared, 1, 11).unwrap();
    let mut b = Session::open(&shared, 2, 22).unwrap();
    let (host, round) = {
        let mut runtime = shared.lock().unwrap();
        runtime.rules.allow_f1e_dynamic_scripts = allow_f1e_dynamic_scripts;
        let model = &mut runtime.model;
        model.terrain = Some(crate::terrain::tests::map());
        let first = model.enter(1, 11, true).unwrap();
        let peer = model.enter(2, 22, false).unwrap();
        model.setup(first).unwrap();
        model.setup(peer).unwrap();
        model.ready(peer).unwrap();
        let round = model.begin(first).unwrap();
        model.ack(first, round).unwrap();
        model.ack(peer, round).unwrap();
        (first, round)
    };
    while a.inbox.try_recv().is_ok() {}
    while b.inbox.try_recv().is_ok() {}
    (shared, a, b, host, round)
}

fn bomb_118(round: u64, serial: u32, row: u8, col: u8) -> Vec<u8> {
    [
        (u64::from(crate::room::wire_key(round).unwrap()), Width::U32),
        (0, Width::U8),
        (0, Width::U8),
        (serial.into(), Width::U32),
        (row.into(), Width::U8),
        (col.into(), Width::U8),
        (1, Width::U8),
        (0, Width::U8),
        (0, Width::U8),
        (0, Width::U8),
        (1, Width::U8),
        (1, Width::U32),
        (0, Width::U8),
    ]
    .into_iter()
    .flat_map(|(v, w)| kr_wire::write_uint(v, w).unwrap())
    .collect()
}

#[test]
fn f1e_dynamic_scripts_require_explicit_opt_in() {
    let (_shared, a, _b, _host, _round) = armed_room(false);
    assert!(a.packet(228, &SPACE_228).is_err());
    assert!(a.packet(148, &PANGLAND_148_2CELLS).is_err());
}

#[test]
fn space228_broadcasts_exact_350_only_from_host() {
    let (shared, mut a, mut b, host, round) = armed_room(true);
    // Guest 228 is ignored cleanly without broadcasting.
    b.packet(228, &SPACE_228).unwrap();
    assert!(a.inbox.try_recv().is_err() && b.inbox.try_recv().is_err());
    assert_eq!(shared.lock().unwrap().model.script_events, 0);

    a.packet(228, &SPACE_228).unwrap();
    for session in [&mut a, &mut b] {
        let batch = session.inbox.try_recv().unwrap();
        assert!(session.valid(&batch));
        assert_eq!(batch.packets.len(), 1);
        assert_eq!(batch.packets[0].1.opcode, 350);
        assert_eq!(batch.packets[0].1.payload, SPACE_228);
        assert_eq!(batch.subject, Some(host));
        assert!(matches!(batch.guard, Guard::Member(_, Some(r)) if r == round));
    }
    assert_eq!(shared.lock().unwrap().model.script_events, 1);
}

#[test]
fn pangland148_spawns_159_and_2200ms_158_without_colliding_with_bomb118() {
    let (shared, mut a, mut b, _host, round) = armed_room(true);
    let bomb = a
        .packet(118, &bomb_118(round, 1, 3, 3))
        .unwrap()
        .bomb
        .unwrap();
    assert_eq!(bomb.id, Some(1));
    for inbox in [&mut a.inbox, &mut b.inbox] {
        let _ = inbox.try_recv().unwrap(); // 159 for bomb 1
        let _ = inbox.try_recv().unwrap(); // 158 for bomb 1
    }

    // Guest 148 does not emit bombs.
    b.packet(148, &PANGLAND_148_2CELLS).unwrap();
    assert!(a.inbox.try_recv().is_err() && b.inbox.try_recv().is_err());

    a.packet(148, &PANGLAND_148_2CELLS).unwrap();
    assert_eq!(shared.lock().unwrap().model.script_events, 2);
    for (session, member_id) in [(&mut a, 1u32), (&mut b, 2u32)] {
        let key = shared.lock().unwrap().model.members[&member_id].key;
        let expected_ids = [2u16, 3u16];
        for (idx, (row, col)) in [(7u8, 2u8), (4u8, 1u8)].into_iter().enumerate() {
            let spawn = session.inbox.try_recv().unwrap();
            assert!(session.valid(&spawn));
            assert_eq!(spawn.packets[0].1.opcode, 159);
            assert_eq!(
                spawn.packets[0].1.payload,
                wire::sprinkler_spawn159(u32::MAX, 290, 17, row, col, 1, 1, expected_ids[idx])
                    .unwrap()
            );
            let blast = session.inbox.try_recv().unwrap();
            assert!(session.valid(&blast));
            assert_eq!(blast.packets[0].1.opcode, 158);
            assert!(matches!(
                blast.guard,
                Guard::ScriptBomb { member, round: r } if member == key && r == round
            ));
            let delay = blast.due.saturating_duration_since(spawn.due);
            assert!(delay >= Duration::from_millis(2150) && delay <= Duration::from_millis(2250));
        }
    }
}

#[test]
fn script_packets_observe_safely_when_ending_or_idle() {
    let (shared, a, _b, _host, round) = armed_room(true);
    {
        let mut r = shared.lock().unwrap();
        r.model.end(round).unwrap();
    }
    let obs_end = a.packet(228, &SPACE_228).unwrap();
    assert!(obs_end.dropped_after_abort);

    {
        let mut r = shared.lock().unwrap();
        let member_keys: Vec<_> = r.model.members.values().map(|m| m.key).collect();
        for key in member_keys {
            r.model.refreshed(key, round).unwrap();
        }
        assert_eq!(r.model.phase, Phase::Idle);
    }
    let obs_idle = a.packet(148, &PANGLAND_148_2CELLS).unwrap();
    assert!(obs_idle.dropped_after_abort);
    assert!(a.channel_current());
}

struct Fixtures(PathBuf);

impl Fixtures {
    fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let path = std::env::temp_dir().join(format!(
            "kr-f1e-dyn-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }

    fn pin(&self, name: &str, bytes: &[u8]) -> crate::FixturePin {
        let path = self.0.join(name);
        fs::write(&path, bytes).unwrap();
        crate::FixturePin {
            path,
            bytes: bytes.len(),
            sha256: crate::sha256_hex(bytes),
        }
    }
}

impl Drop for Fixtures {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn map_profile_admits_space_and_pangland_batch4b_allowlist() {
    let dir = Fixtures::new();
    let valid = [
        (167, "CaJJ_Space02"),
        (1063, "Space01"),
        (871, "NXCL_PangLand15"),
        (887, "PangLand03"),
        (893, "Pangland14"),
    ];
    let pack_items = json!({
        "Item/ItemBomb": 0,
        "Item/ItemFire": 1,
        "Item/ItemRoller": 2,
        "Item/ItemShoe": 3,
        "Item/ItemPunch": 4,
        "Item/ItemMaxPower": 5,
        "Item/ItemNiddle": 145
    });
    for (idx, &(code, name)) in valid.iter().enumerate() {
        let profile = json!({
            "name": name,
            "map_code": code,
            "width": 15,
            "height": 13,
            "spawns": [[0, 0], [1, 0], [2, 0], [3, 0], [4, 0], [5, 0], [6, 0], [7, 0]],
            "scripted": true,
            "payload_sha256": format!("{idx:064x}"),
            "adapter": "f1e-adapter-r1",
            "drops": [],
            "pack_items": pack_items,
        });
        let pin = dir.pin(
            &format!("{name}.json"),
            &serde_json::to_vec(&profile).unwrap(),
        );
        assert!(Profile::load(&pin).is_ok(), "{name} should load");
    }

    // Wrong map_code or unverified Retro/Aqua map stays rejected.
    let bad = json!({
        "name": "CaJJ_Space02",
        "map_code": 887,
        "width": 15,
        "height": 13,
        "spawns": [[0, 0], [1, 0], [2, 0], [3, 0], [4, 0], [5, 0], [6, 0], [7, 0]],
        "scripted": true,
        "payload_sha256": "f".repeat(64),
        "adapter": "f1e-adapter-r1",
        "drops": [],
        "pack_items": pack_items,
    });
    let bad_pin = dir.pin("bad.json", &serde_json::to_vec(&bad).unwrap());
    assert!(Profile::load(&bad_pin).is_err());

    let _: Option<ChoiceConfig> = None;
    let _: Option<Value> = None;
    let _: HashMap<String, Reply> = HashMap::new();
}
