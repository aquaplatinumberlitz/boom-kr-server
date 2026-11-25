use super::*;
use crate::map_profile::{Catalog, ChoiceConfig, Profile};
use serde_json::{Value, json};
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};

fn fields(fields: &[(u64, Width)]) -> Vec<u8> {
    fields
        .iter()
        .flat_map(|&(value, width)| kr_wire::write_uint(value, width).unwrap())
        .collect()
}

fn request(round: u64, serial: u32, row: u8, col: u8) -> Vec<u8> {
    fields(&[
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
    ])
}

#[test]
fn map_option_rejects_bomb118_without_allocating_or_dropping_members() {
    for disabled in [true, false] {
        let shared = registry_foundation_tests::empty(0);
        let mut a = Session::open(&shared, 1, 11).unwrap();
        let mut b = Session::open(&shared, 2, 22).unwrap();
        let (first, peer, round) = {
            let mut runtime = shared.lock().unwrap();
            let model = &mut runtime.model;
            model.terrain = Some(if disabled {
                crate::terrain::tests::lovely_room_map()
            } else {
                crate::terrain::tests::map()
            });
            let first = model.enter(1, 11, true).unwrap();
            let peer = model.enter(2, 22, false).unwrap();
            model.setup(first).unwrap();
            model.setup(peer).unwrap();
            model.ready(peer).unwrap();
            let round = model.begin(first).unwrap();
            model.ack(first, round).unwrap();
            model.ack(peer, round).unwrap();
            (first, peer, round)
        };
        let request = request(round, 1, 3, 3);
        let result = a.packet(118, &request).unwrap().bomb.unwrap();
        if disabled {
            assert_eq!(result.reason, "map_bombs_disabled");
            assert!(!result.accepted && result.id.is_none());
            assert!(a.inbox.try_recv().is_err() && b.inbox.try_recv().is_err());
            let again = a.packet(118, &request).unwrap().bomb.unwrap();
            assert_eq!(again.reason, "map_bombs_disabled");
            assert!(!shared.lock().unwrap().model.bombs.pending());
        } else {
            assert!(result.accepted);
            assert_eq!(result.id, Some(1));
            for inbox in [&mut a.inbox, &mut b.inbox] {
                assert_eq!(inbox.try_recv().unwrap().packets[0].1.opcode, 159);
                assert_eq!(inbox.try_recv().unwrap().packets[0].1.opcode, 158);
            }
        }
        assert!(a.channel_current() && b.channel_current());
        let runtime = shared.lock().unwrap();
        assert!(runtime.model.current(first) && runtime.model.current(peer));
        assert_eq!(runtime.model.phase, Phase::Armed { round });
    }
}

// Synthetic packet pins in the exporter's shape; no KR client or pack oracle.
struct Fixtures(PathBuf);

impl Fixtures {
    fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let path = std::env::temp_dir().join(format!(
            "kr-f1e-{}-{}",
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

    fn choice(&self, profile: &Value, index: u32) -> (ChoiceConfig, HashMap<String, Reply>) {
        let code = profile["map_code"].as_u64().unwrap();
        let name = profile["name"].as_str().unwrap();
        let mut inner = vec![0; 37 + name.len() + 2 * 26];
        inner[20..22].copy_from_slice(&(code as u16).to_le_bytes());
        inner[22..26].copy_from_slice(&(name.len() as u32).to_le_bytes());
        inner[26..26 + name.len()].copy_from_slice(name.as_bytes());
        inner[26 + name.len()..30 + name.len()].copy_from_slice(&index.to_le_bytes());
        inner[32 + name.len()] = 2;
        for (slot, row) in inner[37 + name.len()..]
            .as_chunks_mut::<26>()
            .0
            .iter_mut()
            .enumerate()
        {
            row[0] = slot as u8;
            row[2..6].copy_from_slice(&[slot as u8, slot as u8, 0, slot as u8]);
            row[8] = 255;
            row[13..17].fill(255);
        }
        let mut start = fields(&[(0, Width::U8)]);
        start.extend(raw_blob(&inner));
        start.extend([0; 18]);
        let mut replies = HashMap::new();
        let mut packets = HashMap::new();
        for (alias, opcode, body) in [
            (
                "map",
                117,
                fields(&[
                    (0, Width::U16),
                    (code, Width::U16),
                    (index.into(), Width::U32),
                ]),
            ),
            ("start", 121, start),
            (
                "play",
                196,
                fields(&[(0, Width::U8), (0, Width::U32), (code, Width::U16)]),
            ),
            ("floor", 582, floor582(12, 14)),
        ] {
            let fixture = self.pin(
                &format!("{name}-{alias}"),
                &wire::encode(opcode, &body, 1).unwrap(),
            );
            packets.insert(
                alias.into(),
                LobbyPacketPin {
                    fixture,
                    opcode,
                    sequence: Some(1),
                },
            );
            replies.insert(alias.into(), packet(opcode, body));
        }
        let width = profile["width"].as_u64().unwrap() as usize;
        let height = profile["height"].as_u64().unwrap() as usize;
        let mut kinds = vec![0; width * height];
        if width == 19 {
            kinds[16 * width + 17] = 2;
        }
        let terrain = json!({
            "width": width, "height": height, "kinds": kinds,
            "disable_bombs": profile["disable_bombs"],
            "map_sha256": profile["payload_sha256"], "seed_sha256": crate::sha256_hex(&kinds),
            "fixtures": {"map": packets["map"].fixture.sha256, "start": packets["start"].fixture.sha256},
            "maximum_power": 9
        });
        let cfg = ChoiceConfig {
            map_profile: self.pin(name, &serde_json::to_vec(profile).unwrap()),
            packets,
            terrain: Some(serde_json::from_value(terrain).unwrap()),
            bomb_cancellation: (name == "BSBigCamp08").then(|| {
                serde_json::from_value(
                    json!({"map_sha256": profile["payload_sha256"], "cells": [[16, 18]]}),
                )
                .unwrap()
            }),
        };
        (cfg, replies)
    }
}

impl Drop for Fixtures {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn profile(name: &str, code: u16, grid: (u8, u8), scripted: bool, disable_bombs: bool) -> Value {
    json!({
        "name": name, "map_code": code, "width": grid.0, "height": grid.1,
        "scripted": scripted, "spawns": (0..8).map(|x| [x, 0]).collect::<Vec<_>>(),
        "adapter": "f1e-adapter-r1", "disable_bombs": disable_bombs,
        "payload_sha256": crate::sha256_hex(name.as_bytes()),
        "pack_items": {
            "Item/ItemBomb": 0, "Item/ItemFire": 1, "Item/ItemRoller": 2, "Item/ItemShoe": 3,
            "Item/ItemPunch": 4, "Item/ItemMaxPower": 5, "Item/ItemNiddle": 145
        },
    })
}

fn raw_blob(inner: &[u8]) -> Vec<u8> {
    let mut stream = vec![24];
    stream.extend(((inner.len() + 1) as u32).to_le_bytes());
    stream.push(0);
    stream.extend(inner);
    let mut body = fields(&[(stream.len() as u64, Width::U32)]);
    body.extend(wire::encode_bytes(&stream).unwrap());
    body
}

fn floor582(row: u8, col: u8) -> Vec<u8> {
    let mut inner = vec![1, 0, 0, 0, 0, 0, row, col];
    inner.extend(1_u32.to_le_bytes()); // Fixed Fire.
    inner.extend([0; 4]); // No hidden/third group.
    raw_blob(&inner)
}

fn table(profiles: &[Value]) -> Reply {
    let mut inner = (profiles.len() as u16).to_le_bytes().to_vec();
    for (index, profile) in profiles.iter().enumerate() {
        let name = profile["name"].as_str().unwrap().as_bytes();
        inner.extend((index as u32).to_le_bytes());
        inner.push(0);
        inner.extend(0_u32.to_le_bytes());
        inner.extend((name.len() as u32).to_le_bytes());
        inner.extend(name);
        inner.extend([0; 19]);
        inner.extend((profile["map_code"].as_u64().unwrap() as u16).to_le_bytes());
        inner.extend([0; 5]);
    }
    packet(238, raw_blob(&inner))
}

#[test]
fn batch4a_catalog_selects_and_runs_big_battle_lovely_room_and_retro_popup() {
    let fixtures = Fixtures::new();
    let profiles = [
        profile("BSBigCamp08", 81, (19, 17), false, false),
        profile("LovelyRoom01", 467, (15, 13), true, true),
        // RetroCampN06's lone popup at ms=0 is represented by the exporter marker.
        profile("RetroCampN06", 934, (15, 13), true, false),
    ];
    let (big, mut root) = fixtures.choice(&profiles[0], 0);
    let (lovely, _) = fixtures.choice(&profiles[1], 1);
    let (retro, _) = fixtures.choice(&profiles[2], 2);
    root.insert("table".into(), table(&profiles));
    let mut configs = [big, lovely, retro];
    let load = |configs: &[ChoiceConfig]| {
        Catalog::load(
            configs,
            2,
            &root,
            &HashMap::new(),
            false,
            pickups::Policy::ReportDerivedBasic,
            true,
        )
    };
    let catalog = load(&configs).unwrap();
    let lovely_terrain = configs[1].terrain.take();
    assert!(load(&configs).is_err());
    let wrong_terrain = json!({
        "width": 15, "height": 13, "kinds": vec![0_u8; 195],
        "map_sha256": profiles[1]["payload_sha256"], "seed_sha256": crate::sha256_hex(&[0; 195]),
        "fixtures": {"map": configs[1].packets["map"].fixture.sha256,
            "start": configs[1].packets["start"].fixture.sha256},
        "maximum_power": 9
    });
    configs[1].terrain = Some(serde_json::from_value(wrong_terrain).unwrap());
    assert!(load(&configs).is_err());
    configs[1].terrain = lovely_terrain;
    assert!(
        Profile::load(&configs[0].map_profile)
            .unwrap()
            .validate(2, &root, &HashMap::new())
            .is_err()
    );
    assert!(pickups::Catalog::from_floor(&floor582(16, 18)).is_err());

    let shared = registry_foundation_tests::local_setup_prototype();
    {
        let mut r = shared.lock().unwrap();
        let mut player = r.players[&1].clone();
        player.payload[1..5].copy_from_slice(&fields(&[(2, Width::U32)]));
        r.players.insert(2, player);
        r.model.terrain = catalog.choices[0].terrain.clone();
        r.floor_catalog = catalog.choices[0].floor.clone();
        r.maps = Some(Arc::new(catalog));
        r.rules.item_policy = pickups::Policy::ReportDerivedBasic;
        r.rules.map_bomb_cancellation = true;
    }
    let player_round = load_player_round(
        &PlayerRoundConfig {
            characters: vec![0, 1],
            pack_stats: None,
        },
        &shared,
    )
    .unwrap();
    shared.lock().unwrap().managed_round = Some(player_round);
    let mut a = Session::open(&shared, 1, 11).unwrap();
    let mut b = Session::open(&shared, 2, 22).unwrap();
    let (host, peer) = {
        let mut r = shared.lock().unwrap();
        let host = r.model.enter(1, 11, true).unwrap();
        let peer = r.model.enter(2, 22, false).unwrap();
        for (member, character) in [(host, 0), (peer, 1)] {
            r.model.setup(member).unwrap();
            r.model.choose(member, Some(character), None).unwrap();
        }
        (host, peer)
    };

    for (index, profile) in profiles.iter().enumerate() {
        let code = profile["map_code"].as_u64().unwrap();
        let name = profile["name"].as_str().unwrap();
        a.packet(
            96,
            &fields(&[(code, Width::U16), (index as u64, Width::U32)]),
        )
        .unwrap();
        for inbox in [&mut a.inbox, &mut b.inbox] {
            assert_eq!(inbox.try_recv().unwrap().packets[0].1.opcode, 117);
        }
        shared.lock().unwrap().model.ready(peer).unwrap();
        assert!(a.packet(104, &[0; 20]).unwrap().start_rejection.is_none());
        for inbox in [&mut a.inbox, &mut b.inbox] {
            let start = inbox.try_recv().unwrap().packets;
            assert_eq!(start[0].1.payload, floor582(12, 14));
            assert_eq!(
                wire::start121_map(&start[1].1.payload).unwrap(),
                (code as u16, name.as_bytes().to_vec())
            );
        }
        let round = {
            let mut r = shared.lock().unwrap();
            let Phase::Loading { round, .. } = r.model.phase else {
                panic!("expected loading")
            };
            r.model.ack(host, round).unwrap();
            r.model.ack(peer, round).unwrap();
            round
        };
        let edge = a
            .packet(118, &request(round, 1, 16, 18))
            .unwrap()
            .bomb
            .unwrap();
        let expected = match index {
            0 => "map_cell_cancelled",
            1 => "map_bombs_disabled",
            _ => "unsupported_cell_or_power",
        };
        assert_eq!(edge.reason, expected);
        assert!(!edge.accepted && edge.id.is_none());
        assert!(a.inbox.try_recv().is_err() && b.inbox.try_recv().is_err());
        if index != 1 {
            let (row, col) = if index == 0 { (16, 16) } else { (3, 3) };
            let bomb = a
                .packet(118, &request(round, 2, row, col))
                .unwrap()
                .bomb
                .unwrap();
            assert!(bomb.accepted);
            assert_eq!(bomb.id, Some(if index == 0 { 1 } else { 2 }));
            for session in [&mut a, &mut b] {
                assert_eq!(session.inbox.try_recv().unwrap().packets[0].1.opcode, 159);
                let mut batch = session.inbox.try_recv().unwrap();
                assert!(session.prepare_delivery(&mut batch).unwrap());
                assert_eq!(batch.packets[0].1.opcode, 158);
                session.finish(&batch).unwrap();
            }
        } else {
            let in_bounds = a
                .packet(118, &request(round, 2, 3, 3))
                .unwrap()
                .bomb
                .unwrap();
            assert_eq!(in_bounds.reason, "map_bombs_disabled");
        }
        let mut r = shared.lock().unwrap();
        assert!(!r.model.bombs.pending());
        if index == 0 {
            assert_eq!(
                r.model.terrain.as_ref().unwrap().clearance(round, 16, 17),
                Some(1)
            );
        }
        r.model.end(round).unwrap();
        r.model.refreshed(host, round).unwrap();
        r.model.refreshed(peer, round).unwrap();
    }
}

#[test]
fn batch4a_profile_policy_fails_closed() {
    let fixtures = Fixtures::new();
    let loads = |value| {
        Profile::load(&fixtures.pin("profile", &serde_json::to_vec(&value).unwrap())).is_ok()
    };
    assert!(!loads(profile("RetroCampN06", 933, (15, 13), true, false)));
    assert!(!loads(profile("LovelyRoom01", 467, (15, 13), true, false)));
    assert!(!loads(profile("BSBigCamp08", 81, (19, 16), false, false)));
    assert!(!loads(profile("BSBigCamp08", 81, (19, 17), true, false)));
    for mut unmarked in [
        profile("RetroCampN06", 934, (15, 13), true, false),
        profile("BSBigCamp08", 81, (19, 17), false, false),
    ] {
        unmarked.as_object_mut().unwrap().remove("adapter");
        assert!(!loads(unmarked));
    }
    let mut legacy = profile("ExVillage01", 278, (15, 13), false, false);
    legacy.as_object_mut().unwrap().remove("adapter");
    assert!(loads(legacy));
    for (name, code) in [
        ("Aqua01", 1),
        ("Aqua06", 6),
        ("Aqua08", 8),
        ("NXCL_Aqua01", 862),
        ("RetroCampN01", 929),
        ("RetroCampN05", 933),
        ("CaJJ_Space02", 167),
        ("NXCL_PangLand15", 871),
    ] {
        let mut allowed = profile(name, code, (15, 13), true, false);
        assert!(loads(allowed.clone()), "{name}");
        allowed.as_object_mut().unwrap().remove("adapter");
        assert!(!loads(allowed), "{name} without adapter");
    }
}

#[test]
fn f1e_script_profiles_require_marker_code_and_exact_distinct_spawns() {
    let fixtures = Fixtures::new();
    let load =
        |value| Profile::load(&fixtures.pin("profile", &serde_json::to_vec(&value).unwrap()));
    for (name, code, expected_spawns) in [
        ("Aqua03", 3, 8),
        ("RetroForestN06", 946, 8),
        ("RetroVillageN01", 953, 8),
        ("Fm_RescueET01", 318, 4),
        ("CaJJ_Wriggle07", 177, 4),
        ("Fm_Wriggle04", 326, 4),
        ("RetroWriggleN05", 963, 4),
        ("Fm_Coloring03", 301, 4),
    ] {
        let mut allowed = profile(name, code, (15, 13), true, false);
        allowed["spawns"]
            .as_array_mut()
            .unwrap()
            .truncate(expected_spawns);
        assert!(load(allowed.clone()).is_ok(), "{name}");

        let mut unmarked = allowed.clone();
        unmarked.as_object_mut().unwrap().remove("adapter");
        assert!(load(unmarked).is_err(), "{name} without adapter");

        let mut wrong_code = allowed.clone();
        wrong_code["map_code"] = json!(code + 1);
        assert!(load(wrong_code).is_err(), "{name} with wrong code");

        let mut no_bombs = allowed.clone();
        no_bombs["disable_bombs"] = json!(true);
        assert!(load(no_bombs).is_err(), "{name} with bombs disabled");

        let mut wrong_count = allowed.clone();
        let spawns = wrong_count["spawns"].as_array_mut().unwrap();
        if expected_spawns == 8 {
            spawns.truncate(4);
        } else {
            spawns.extend((4..8).map(|x| json!([x, 0])));
        }
        assert!(load(wrong_count).is_err(), "{name} with wrong spawn count");

        let duplicate = allowed["spawns"][0].clone();
        allowed["spawns"][1] = duplicate;
        assert!(load(allowed).is_err(), "{name} with duplicate spawn");
    }
}
