use crate::{
    ActorProfile, FixturePin, LobbyPacketPin,
    operations::{F2Command, F2Event, F2Setup},
    room_io::{self, Session, Shared, registry_foundation_tests},
    room_modes,
    room_packets::packet,
    room_registry::RoomKey,
    room_service::{Config, Hub},
};
use kr_wire::{Width, kr00967 as wire, write_uint};
use serde_json::{Value, json};
use std::{collections::HashMap, fs, io, net::Ipv4Addr, path::PathBuf, sync::Arc};

fn uint(value: u64, width: Width) -> Vec<u8> {
    write_uint(value, width).unwrap()
}

fn create(manner: bool) -> Vec<u8> {
    let mut body = uint(1, Width::U16);
    body.extend(wire::encode_bytes(b"A").unwrap());
    body.extend(uint(0, Width::U16));
    body.extend(uint(0, Width::U32));
    body.extend(uint(u64::from(manner), Width::U32));
    body.extend((0..10).flat_map(|_| uint(0, Width::U32)));
    body.extend([0x5a; 4]);
    body
}

fn join(key: RoomKey) -> Vec<u8> {
    [
        (u64::from(key.session), Width::U16),
        (u64::from(key.id.0), Width::U32),
        (0, Width::U16),
        (1, Width::U8),
        (5, Width::U16),
    ]
    .into_iter()
    .flat_map(|(v, w)| uint(v, w))
    .collect()
}

fn hub(f2: Value) -> Arc<Hub> {
    managed_hub(
        registry_foundation_tests::f2_runtime_prototype(f2),
        &[0, 1, 0, 1],
    )
}

fn managed_hub(prototype: Shared, teams: &[u8]) -> Arc<Hub> {
    let cfg: Config = serde_json::from_value(json!({
        "max_rooms": 2,
        "round_profile": {"mode": "teams", "teams": teams,
            "initial_bomb_capacity": crate::rules::default_bomb_capacity(teams.len())}
    }))
    .unwrap();
    Hub::new(&cfg, prototype, None, None).unwrap()
}

struct PacketFixtures(PathBuf);

impl PacketFixtures {
    fn new() -> Self {
        let mut suffix = [0; 8];
        getrandom::fill(&mut suffix).unwrap();
        let dir =
            std::env::temp_dir().join(format!("kr-f2-runtime-{:016x}", u64::from_ne_bytes(suffix)));
        fs::create_dir(&dir).unwrap();
        Self(dir)
    }

    fn file(&self, name: &str, bytes: &[u8]) -> FixturePin {
        let path = self.0.join(name);
        fs::write(&path, bytes).unwrap();
        FixturePin {
            path,
            bytes: bytes.len(),
            sha256: crate::sha256_hex(bytes),
        }
    }

    fn pin(&self, name: &str, opcode: u32, body: Vec<u8>) -> (String, LobbyPacketPin) {
        let frame = wire::encode(opcode, &body, 0).unwrap();
        (
            name.into(),
            LobbyPacketPin {
                fixture: self.file(name, &frame),
                sequence: Some(0),
                opcode,
            },
        )
    }
}

impl Drop for PacketFixtures {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

/// Synthetic packets for the production eight-player loader. The existing
/// four-player fixture cannot exercise the measured operator team layout.
fn eight_player_hub(layout: &str, teams: &[u8; 8]) -> Arc<Hub> {
    loaded_hub(opt_in("free_1v7"), teams, Some(layout), None)
}

fn map_hub(f2: Value, name: &str) -> Arc<Hub> {
    loaded_hub(f2, &[0, 1, 0, 1], None, Some(name))
}

// Named fixtures exercise room policy on roomtype=0 without executing the
// corresponding map scripts or claiming their pack terrain or map codes.
fn loaded_hub(f2: Value, teams: &[u8], layout: Option<&str>, map: Option<&str>) -> Arc<Hub> {
    let files = PacketFixtures::new();
    let name = map.unwrap_or("ExVillage01").as_bytes();
    let dynamic_index = if map.is_some() { 0u32 } else { 3 };
    let count = teams.len() as u8;
    let mut stream = vec![0; 32];
    stream[0] = 24;
    stream[28..32].copy_from_slice(&(name.len() as u32).to_le_bytes());
    stream.extend(name);
    stream.extend(dynamic_index.to_le_bytes());
    stream.extend([0, 0, count, 0, 0, 0, 0]);
    for slot in 0..count {
        let mut row = [0; 26];
        row[0] = slot;
        row[2] = slot;
        row[3] = slot;
        row[5] = slot % 2;
        row[8] = 255;
        row[13..17].fill(255);
        stream.extend(row);
    }
    let size = stream.len();
    stream[1..5].copy_from_slice(&((size - 5) as u32).to_le_bytes());
    let mut start = uint(0, Width::U8);
    start.extend(uint(size as u64, Width::U32));
    start.extend(wire::encode_bytes(&stream).unwrap());
    start.extend([0; 18]);

    let mut floor_stream = vec![24, 37, 0, 0, 0, 0, 3, 0];
    for index in 0..3u32 {
        floor_stream.extend(index.to_le_bytes());
        floor_stream.extend([0, index as u8]);
        floor_stream.extend(1u32.to_le_bytes());
    }
    floor_stream.extend([0; 4]);
    let mut floor = uint(floor_stream.len() as u64, Width::U32);
    floor.extend(wire::encode_bytes(&floor_stream).unwrap());

    let (table, ready) = if map.is_some() {
        map_table_and_ready(name)
    } else {
        (vec![0; 72], vec![])
    };
    let mut play = vec![0; 7];
    if map.is_some() {
        play[5..].copy_from_slice(&uint(0, Width::U16));
    }

    let listing = wire::listing91_from_rows(&[wire::ListingRow {
        session: 1,
        room_id: 1,
        title: b"Template".to_vec(),
        opaque: [0; 22],
    }])
    .unwrap();
    let mut packets: HashMap<_, _> = [
        (
            "create",
            86,
            vec![
                0x5a, 0xa5, 0x68, 0x96, 0xca, 0x53, 0x95, 0x5a, 0x5a, 0x5a, 0x5a,
            ],
        ),
        ("join", 87, vec![0x5a, 0x5a, 0xa5, 0x68, 0x5a, 0x5a, 0x5a]),
        ("listing", 91, listing),
        ("table", 238, table),
        (
            "map",
            117,
            [
                uint(0, Width::U16),
                uint(0, Width::U16),
                uint(u64::from(dynamic_index), Width::U32),
            ]
            .concat(),
        ),
        ("floor", 582, floor),
        ("start", 121, start),
        ("go", 197, vec![0; 14]),
        ("play", 196, play),
        ("peer", 153, vec![0; 61 + 18 * usize::from(count)]),
    ]
    .into_iter()
    .map(|(name, op, body)| files.pin(name, op, body))
    .collect();
    let source = crate::account_bridge::local::native::fixtures::PLAYER;
    let body: Vec<_> = (0..source.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&source[i..i + 2], 16).unwrap())
        .collect();
    let template = wire::PlayerTemplate::parse(&body).unwrap();
    let mut actors = HashMap::new();
    for slot in 0..count {
        let identity = u32::from(slot) + 1;
        let name = format!("Test{slot}").into_bytes();
        let body = template.project(identity, &name).unwrap();
        let body = wire::player106_state(&body, slot, slot % 2, false, slot == 0).unwrap();
        let alias = format!("player_{}", char::from(b'A' + slot));
        let (alias, pin) = files.pin(&alias, 106, body);
        packets.insert(alias, pin);
        actors.insert(
            identity,
            ActorProfile {
                identity,
                name,
                packets: HashMap::from([("ready".into(), packet(241, ready.clone()))]),
                reported_login_ipv4: None,
                configured_peer: Ipv4Addr::LOCALHOST,
            },
        );
    }
    let mut cfg: room_io::Config = serde_json::from_value(json!({
        "title": "Template", "packets": {}, "eight_battle": layout.is_some(),
        "eight_team_layout": layout, "f2_modes": f2,
    }))
    .unwrap();
    cfg.packets = packets;
    if let Some(name) = map {
        let profile = json!({
            "name": name, "map_code": 0, "width": 15, "height": 13,
            "scripted": false, "payload_sha256": "0".repeat(64),
            "spawns": (0..count).map(|slot| [slot, 0]).collect::<Vec<_>>()
        });
        cfg.map_profile = Some(files.file("profile", &serde_json::to_vec(&profile).unwrap()));
    }
    let prototype = room_io::load(&cfg, &actors, true)
        .unwrap_or_else(|error| panic!("failed to load map fixture {map:?}: {error}"));
    managed_hub(prototype, teams)
}

fn map_table_and_ready(name: &[u8]) -> (Vec<u8>, Vec<u8>) {
    let mut table = vec![24, 0, 0, 0, 0, 0];
    table.extend(1u16.to_le_bytes()); // One map at index 0.
    table.extend([0; 9]); // Index, flag, empty display name.
    table.extend((name.len() as u32).to_le_bytes());
    table.extend(name);
    table.extend([0; 26]); // Remaining row fields, including test map code 0.
    let size = table.len();
    table[1..5].copy_from_slice(&((size - 5) as u32).to_le_bytes());
    let (a, b) = table[5..].iter().fold((0u32, 0u32), |(a, b), byte| {
        let a = (a + u32::from(*byte)) % 65521;
        (a, (b + a) % 65521)
    });
    let mut body = uint(size as u64, Width::U32);
    body.extend(wire::encode_bytes(&table).unwrap());
    let mut ready = body.clone();
    ready.extend(uint(u64::from((b << 16) | a), Width::U32));
    ready.extend([0; 9]);
    (body, ready)
}

struct TestRoom {
    hub: Arc<Hub>,
    key: RoomKey,
    room: Shared,
    clients: Vec<Session>,
    startup: Vec<Vec<(u32, Vec<u8>)>>,
}

impl TestRoom {
    fn new(f2: Value) -> Self {
        Self::on(hub(f2), false)
    }

    fn on(hub: Arc<Hub>, manner: bool) -> Self {
        Self::with_guests(hub, manner, &[43, 501, 71])
    }

    fn with_guests(hub: Arc<Hub>, manner: bool, guests: &[u32]) -> Self {
        let (key, room, host) = hub.select_fixture(81, &create(manner), 70000).unwrap();
        let mut clients = vec![host];
        clients.extend(
            guests
                .iter()
                .map(|&id| hub.select_fixture(82, &join(key), id).unwrap().2),
        );
        let mut result = Self {
            hub,
            key,
            room,
            clients,
            startup: vec![vec![]; guests.len() + 1],
        };
        result.drain_startup();
        result
    }

    fn drain_startup(&mut self) {
        for (client, packets) in self.clients.iter_mut().zip(&mut self.startup) {
            while let Ok(batch) = client.inbox.try_recv() {
                assert!(client.valid(&batch));
                packets.extend(
                    batch
                        .packets
                        .iter()
                        .map(|(_, p)| (p.opcode, p.payload.clone())),
                );
                client.finish(&batch).unwrap();
            }
        }
    }

    fn load(&mut self) -> u64 {
        registry_foundation_tests::ready_roster(&self.room);
        let observation = self.clients[0].packet(104, &[0; 20]).unwrap();
        assert!(
            observation.start_rejection.is_none(),
            "start rejected: {:?}",
            observation.start_rejection
        );
        self.drain_startup();
        let (phase, round) = room_io::health(&self.room).unwrap();
        assert_eq!(phase, "loading");
        round.unwrap()
    }

    fn arm(&mut self) {
        for (i, client) in self.clients.iter().enumerate() {
            let mut body = [0; 23];
            body[8..10].copy_from_slice(&uint(9000 + i as u64, Width::U16));
            client.packet(99, &body).unwrap();
        }
        self.drain_startup();
        assert_eq!(room_io::health(&self.room).unwrap().0, "active");
    }

    fn run(&self, command: F2Command) -> io::Result<Value> {
        self.hub
            .operator_f2(self.key, command)
            .map(|view| serde_json::to_value(view).unwrap())
    }

    fn start(&self, round: u64, setup: F2Setup) -> io::Result<Value> {
        self.run(F2Command::Start { round, setup })
    }

    fn event(&self, round: u64, sequence: u64, event: F2Event) -> io::Result<Value> {
        self.run(F2Command::Apply {
            round,
            sequence,
            event,
        })
    }

    fn no_packets(&mut self) {
        for client in &mut self.clients {
            assert!(client.inbox.try_recv().is_err());
        }
    }
}

fn opt_in(mode: &str) -> Value {
    json!({"runtime_overrides": {"free": mode}})
}

fn revive(score_limit: u32) -> F2Setup {
    F2Setup::Revive {
        team_budget: Some(2),
        player_budget: Some(1),
        duration_ms: 500,
        score_limit,
    }
}

fn defeat(slot: u8) -> F2Event {
    F2Event::Defeat { slots: vec![slot] }
}

fn rejects(result: io::Result<Value>, reason: &str) {
    let error = result.unwrap_err().to_string();
    assert!(error.contains(reason), "expected {reason}, got {error}");
}

#[test]
fn default_room_and_unselected_override_keep_legacy_packets_and_reject_f2() {
    assert!(
        room_modes::Policy::load(&room_modes::Config::default())
            .unwrap()
            .is_none()
    );
    let mut normal = TestRoom::new(Value::Null);
    let mut empty_rules = TestRoom::new(json!({"extended_rules": {}}));
    let mut explicit_default = TestRoom::new(json!({
        "create_overrides": {"free": {"game_type_id": 0}},
        "runtime_overrides": {"free": {"game_type_id": 0}},
        "visible_map_groups": {"bytes": [], "group_ids": {}},
        "candidate_map_family": null,
        "extended_rules": {
            "no_item": false, "no_shop": false, "basic_shop": false,
            "super_host": false, "vote_kick": false, "priority_room": null,
            "ladder_season": false, "channel_kind": null, "big_battle_12": false
        }
    }));
    let mut other_selector = TestRoom::new(json!({"runtime_overrides": {"manner": "revive"}}));
    for room in [
        &mut normal,
        &mut empty_rules,
        &mut explicit_default,
        &mut other_selector,
    ] {
        let round = room.load();
        room.arm();
        rejects(room.start(round, revive(3)), "disabled");
        rejects(room.event(round, 1, defeat(1)), "disabled");
        assert!(!Session::handles(185));
        assert!(room.clients[0].packet(185, &[0x5b]).is_err());
        room.clients[0].tick().unwrap();
        room.no_packets();
    }
    assert_eq!(normal.startup, empty_rules.startup);
    assert_eq!(normal.startup, explicit_default.startup);
    assert_eq!(normal.startup, other_selector.startup);
    assert_eq!(
        normal.startup[0][0],
        (
            86,
            vec![
                0x5a, 0xa5, 0x68, 0x96, 0xca, 0x53, 0x95, 0x5a, 0x5a, 0x5a, 0x5a
            ]
        )
    );
    assert!(
        normal.startup[1]
            .iter()
            .map(|(op, _)| *op)
            .eq([87, 582, 121, 197, 196, 153])
    );
}

fn rejected_room_config(f2: Value) -> String {
    let cfg = serde_json::from_value(json!({
        "title": "Template", "packets": {}, "f2_modes": f2
    }))
    .unwrap();
    room_io::load(&cfg, &HashMap::new(), true)
        .err()
        .expect("policy must reject before loading the room")
        .to_string()
}

// Literal export names from kr_pack_export.py; candidate groups from the
// KR00967 F2 mode catalog. Rabbit11 keeps the nonconsecutive suffix covered.
const F2_EXPORT_SAMPLES: [(&str, &str, &str); 15] = [
    ("CAPong", "CAPong01", "capong"),
    ("Clover", "Clover01", "clover"),
    ("Coloring", "Coloring01", "coloring"),
    ("Gumiho", "Gumiho01", "gumiho"),
    ("Hamster", "Hamster01", "hamster"),
    ("PopcadeA", "PopcadeA01", "popcade_a"),
    ("PopcadeB", "PopcadeB01", "popcade_b"),
    ("PopcadeC", "PopcadeC01", "popcade_c"),
    ("Rabbit", "Rabbit11", "farm_mission"),
    ("RescueET", "RescueET01", "rescue_et"),
    ("SnowyFields", "SnowyFields04", "snowy_fields"),
    ("SuperFighter", "SuperFighter16", "super_fighter"),
    ("Wriggle", "Wriggle01", "wriggle"),
    ("Zombie", "Zombie04", "zombie"),
    ("Zzolman", "Zzolman01", "zzolman"),
];

#[test]
fn f2_modes_status_snapshot_serializes_catalog_coverage_and_dispatch_policy() {
    let snapshot = room_modes::status_snapshot();
    assert_eq!(
        serde_json::to_value(snapshot).unwrap(),
        json!({
            "exported_families": F2_EXPORT_SAMPLES.map(|(family, _, _)| family),
            "exported_map_count": 97,
            "total_catalog_families": 18,
            "total_catalog_maps": 122,
            "tentative_family_count": 21,
            "live_dispatch_status": "opt_in_fail_closed_pending_wine_capture"
        })
    );
}

fn has_evaluation_preset(family: &str) -> bool {
    matches!(
        family,
        "CAPong"
            | "Coloring"
            | "Gumiho"
            | "Hamster"
            | "PopcadeA"
            | "PopcadeB"
            | "PopcadeC"
            | "SuperFighter"
    )
}

#[test]
fn f2_export_maps_require_an_explicit_runtime_preset_for_state_and_ticks() {
    for (family, map, preset) in F2_EXPORT_SAMPLES {
        let evaluates = has_evaluation_preset(family);
        let mut legacy_startup = None;
        for enabled in [false, true] {
            let policy = match (enabled, evaluates) {
                (true, true) => opt_in(preset),
                (true, false) => opt_in("timebomb"),
                (false, _) => Value::Null,
            };
            let mut room = TestRoom::on(map_hub(policy, map), false);
            let round = room.load();
            let (_, start) = room.startup[0].iter().find(|(op, _)| *op == 121).unwrap();
            let selected = wire::start121_map_context(start).unwrap();
            assert_eq!(selected.name, map.as_bytes(), "{family}");
            assert_eq!(selected.roomtype, 0, "{family}");
            room.arm();

            if enabled {
                if evaluates {
                    let initial = room.start(round, F2Setup::Evaluate).unwrap();
                    assert_eq!(initial["outcome"], "playing", "{family}");
                    let unresolved = room
                        .event(round, 1, F2Event::Objective { winner: None })
                        .unwrap();
                    assert_eq!(unresolved["outcome"], "unresolved", "{family}");
                } else {
                    let start = room.start(
                        round,
                        F2Setup::Timebomb {
                            initial_carrier: 0,
                            fuse_ms: 10,
                        },
                    );
                    let view = start.unwrap();
                    assert_eq!(view["bomb"], json!([0, 10]), "{family}");
                    assert_eq!(view["players"][0]["life"], "alive", "{family}");
                    let view = room
                        .event(round, 1, F2Event::AdvanceTo { milliseconds: 4 })
                        .unwrap();
                    assert_eq!(view["bomb"], json!([0, 6]), "{family}");
                    let view = room
                        .event(round, 2, F2Event::AdvanceTo { milliseconds: 10 })
                        .unwrap();
                    assert_eq!(view["players"][0]["life"], "eliminated", "{family}");
                    assert_eq!(view["bomb"], json!([1, 10]), "{family}");
                }
            } else {
                rejects(room.start(round, F2Setup::Evaluate), "disabled");
                rejects(
                    room.event(round, 1, F2Event::Objective { winner: None }),
                    "disabled",
                );
            }
            room.no_packets();
            if enabled {
                assert_eq!(legacy_startup.as_ref().unwrap(), &room.startup, "{family}");
            } else {
                legacy_startup = Some(room.startup);
            }
        }
    }
}

#[test]
fn f2_exported_rosters_match_the_census_and_reject_native_dispatch() {
    let census: Value = serde_json::from_str(include_str!(
        "../../../../docs/evidence/content/f2-f3-tile-grid-census.json"
    ))
    .unwrap();
    let mut seen_maps = std::collections::BTreeSet::new();
    for (family, sample, preset) in F2_EXPORT_SAMPLES {
        let row = census["families"]["F2"]
            .as_array()
            .unwrap()
            .iter()
            .find(|row| row["family"] == family)
            .unwrap();
        assert_eq!(row["priority"], "terrain_ready");
        let maps = row["maps"].as_array().unwrap();
        assert!(maps.iter().any(|map| map == sample), "{family}");
        let mut create_override = json!({"create_overrides": {"free": preset}});
        if matches!(family, "Coloring" | "SuperFighter") {
            create_override["visible_map_groups"] = json!([family]);
        }
        for map in maps {
            let map = map.as_str().unwrap();
            assert!(seen_maps.insert(map), "duplicate map: {map}");
            let hub = map_hub(create_override.clone(), map);
            let rejected = hub.select_fixture(81, &create(false), 70000);
            assert_eq!(
                rejected.err().unwrap().to_string(),
                "f2_start_dispatch_unmeasured",
                "{family} {map}"
            );
            assert!(hub.operator_rooms().unwrap().is_empty(), "{family} {map}");

            if has_evaluation_preset(family) {
                let mut room = TestRoom::on(map_hub(opt_in(preset), map), false);
                let round = room.load();
                room.arm();
                assert_eq!(
                    room.start(round, F2Setup::Evaluate).unwrap()["outcome"],
                    "playing",
                    "{map}"
                );
                assert_eq!(
                    room.event(round, 1, F2Event::Objective { winner: None })
                        .unwrap()["outcome"],
                    "unresolved",
                    "{map}"
                );
                room.no_packets();
            }
        }
    }
    assert_eq!(seen_maps.len(), 97);
    assert_eq!(
        seen_maps.len(),
        room_modes::status_snapshot().exported_map_count
    );
}

#[test]
fn f2_export_family_names_resolve_without_granting_unmeasured_dispatch() {
    for (index, (family, map, group)) in F2_EXPORT_SAMPLES.into_iter().enumerate() {
        assert_eq!(
            rejected_room_config(json!({"candidate_map_family": family})),
            "F2 candidate map family activation is unmeasured",
            "{family}"
        );
        assert_eq!(
            rejected_room_config(json!({"candidate_map_family": family.to_ascii_lowercase()})),
            "unknown F2 candidate map family",
            "{family}"
        );

        let mut create_override = json!({"create_overrides": {"free": group}});
        if matches!(family, "Coloring" | "SuperFighter") {
            create_override["visible_map_groups"] = json!([family]);
        }
        let malformed = format!("{map}x");
        let (_, other_family_map, _) = F2_EXPORT_SAMPLES[(index + 1) % F2_EXPORT_SAMPLES.len()];
        for (hub, expected) in [
            (
                map_hub(create_override.clone(), map),
                "f2_start_dispatch_unmeasured",
            ),
            (
                map_hub(create_override.clone(), &malformed),
                "f2_map_not_confirmed_in_group_or_family",
            ),
            (
                map_hub(create_override, other_family_map),
                "f2_map_not_confirmed_in_group_or_family",
            ),
        ] {
            let rejected = hub.select_fixture(81, &create(false), 70000);
            assert_eq!(rejected.err().unwrap().to_string(), expected, "{family}");
            assert!(hub.operator_rooms().unwrap().is_empty(), "{family}");
        }
        if !has_evaluation_preset(family) {
            let hub = map_hub(opt_in(group), map);
            let rejected = hub.select_fixture(81, &create(false), 70000);
            assert_eq!(
                rejected.err().unwrap().to_string(),
                "F2 runtime: UnsupportedMode",
                "{family}"
            );
            assert!(hub.operator_rooms().unwrap().is_empty(), "{family}");
        }
    }
}

#[test]
fn family_presets_evaluate_without_guessing_gameplay_rules() {
    for (map, preset) in [
        ("CAPong05", json!("capong")),
        ("Coloring01", json!("coloring")),
        ("Coloring02", json!("coloring")),
        ("Coloring03", json!("coloring")),
        ("Coloring04", json!("coloring")),
        ("Coloring05", json!("coloring")),
        ("Gumiho11", json!("gumiho")),
        ("PopcadeA05", json!("popcade_a")),
        ("PopcadeB05", json!("popcade_b")),
        ("PopcadeC05", json!("popcade_c")),
        ("SuperFighter16", json!("super_fighter")),
        ("SuperFighter19", json!({"game_type_id": 18})),
    ] {
        let mut room = TestRoom::on(
            map_hub(json!({"runtime_overrides": {"free": preset}}), map),
            false,
        );
        let round = room.load();
        room.arm();

        rejects(room.event(round, 1, tick(1)), "WrongRound");
        rejects(room.start(round, revive(3)), "InvalidSetup");
        let initial = room.start(round, F2Setup::Evaluate).unwrap();
        assert_eq!(initial["outcome"], "playing", "{map}");
        assert_eq!(initial["bomb"], Value::Null, "{map}");
        rejects(room.start(round, F2Setup::Evaluate), "WrongRound");

        rejects(room.event(round, 0, tick(0)), "SequenceGap");
        rejects(room.event(round, 2, tick(0)), "SequenceGap");
        assert_eq!(
            room.event(round, 1, tick(u64::MAX)).unwrap(),
            initial,
            "{map}"
        );
        rejects(room.event(round, 2, tick(0)), "ClockReversed");
        rejects(room.event(round, 2, defeat(0)), "UnsupportedAction");
        rejects(
            room.event(round, 2, F2Event::Respawn { slot: 0 }),
            "UnsupportedAction",
        );
        rejects(
            room.event(round, 2, F2Event::PassBomb { from: 0, to: 1 }),
            "UnsupportedAction",
        );
        for team in [0, u8::MAX] {
            rejects(
                room.event(round, 2, F2Event::Objective { winner: Some(team) }),
                "UnsupportedAction",
            );
        }
        let unresolved = room
            .event(round, 2, F2Event::Objective { winner: None })
            .unwrap();
        assert_eq!(unresolved["outcome"], "unresolved", "{map}");
        assert_eq!(unresolved["players"], initial["players"], "{map}");
        assert_eq!(room.event(round, 2, defeat(0)).unwrap()["replay"], true);
        rejects(room.event(round, 3, defeat(0)), "Finished");
        room.no_packets();
    }

    for (group, wrong_map) in [
        ("coloring", "SuperFighter16"),
        ("super_fighter", "Coloring01"),
        ("gumiho", "Zombie01"),
        ("popcade_a", "PopcadeB01"),
        ("popcade_b", "PopcadeC01"),
        ("popcade_c", "PopcadeA01"),
    ] {
        let room = TestRoom::on(map_hub(opt_in(group), wrong_map), false);
        registry_foundation_tests::ready_roster(&room.room);
        assert_eq!(
            room.clients[0]
                .packet(104, &[0; 20])
                .unwrap()
                .start_rejection,
            Some("f2_map_not_confirmed_in_group_or_family")
        );
    }
}

fn tick(milliseconds: u64) -> F2Event {
    F2Event::AdvanceTo { milliseconds }
}

#[test]
fn extended_evaluators_reset_round_state_and_replay_deterministically() {
    let mut roster = crate::room::Room::default();
    roster.enter(1, 1, true).unwrap();
    roster.enter(2, 2, false).unwrap();
    for preset in [
        "gumiho",
        "popcade_a",
        "popcade_b",
        "popcade_c",
        "super_fighter",
        "capong",
    ] {
        let config = serde_json::from_value(opt_in(preset)).unwrap();
        let policy = room_modes::Policy::load(&config).unwrap().unwrap();
        let mut mode = policy
            .room_mode(wire::RoomTeamSelector::Free)
            .unwrap()
            .unwrap();
        // Exercise two rounds through the same adapter. The managed room
        // intentionally has no F2 wire return path to start a second round.
        let mut run = |command| {
            let (view, replies) = mode.apply(command, roster.members.values()).unwrap();
            assert!(replies.is_empty(), "{preset}");
            let mut view = serde_json::to_value(view).unwrap();
            assert!(view.as_object_mut().unwrap().remove("round").is_some());
            view
        };
        let apply = |round, sequence, event| F2Command::Apply {
            round,
            sequence,
            event,
        };
        let mut previous = None;
        for round in [1, 2] {
            let initial = run(F2Command::Start {
                round,
                setup: F2Setup::Evaluate,
            });
            assert_eq!(initial["outcome"], "playing", "{preset}");
            if round == 2 {
                let replay = run(apply(1, 99, F2Event::Objective { winner: None }));
                let mut expected = initial.clone();
                expected["replay"] = json!(true);
                assert_eq!(replay, expected, "{preset}");
            }
            // A low tick must succeed after the preceding round advanced to
            // the maximum clock. Sequence numbers also start over at one.
            assert_eq!(run(apply(round, 1, tick(1))), initial, "{preset}");
            assert_eq!(run(apply(round, 2, tick(u64::MAX))), initial, "{preset}");
            let end = run(apply(round, 3, F2Event::Objective { winner: None }));
            let mut expected = initial;
            expected["outcome"] = json!("unresolved");
            assert_eq!(end, expected, "{preset}");
            if let Some(previous) = previous.replace(end.clone()) {
                assert_eq!(end, previous, "{preset}");
            }
        }
    }
}

#[test]
fn exported_evaluation_groups_require_explicit_operator_visibility_bindings() {
    // A binding is local policy. The same bytes may be bound to either label.
    let bytes = wire::MapGroupVisible(vec![0x2345]).encode().unwrap();
    for (family, map, preset) in [
        ("Coloring", "Coloring01", json!("coloring")),
        (
            "SuperFighter",
            "SuperFighter19",
            json!({"game_type_id": 18}),
        ),
    ] {
        for (visibility, expected) in [
            (json!([]), "f2_map_group_hidden"),
            (json!([family]), "f2_start_dispatch_unmeasured"),
            (
                json!({"bytes": bytes, "group_ids": { (family): 0x2345 }}),
                "f2_start_dispatch_unmeasured",
            ),
        ] {
            let hub = map_hub(
                json!({"visible_map_groups": visibility, "create_overrides": {"free": preset}}),
                map,
            );
            let rejected = hub.select_fixture(81, &create(false), 70000);
            assert_eq!(rejected.err().unwrap().to_string(), expected, "{family}");
            assert!(hub.operator_rooms().unwrap().is_empty(), "{family}");
        }

        let mut plain = TestRoom::on(map_hub(Value::Null, map), false);
        let mut visible =
            TestRoom::on(map_hub(json!({"visible_map_groups": [family]}), map), false);
        for room in [&mut plain, &mut visible] {
            let round = room.load();
            room.arm();
            rejects(room.start(round, F2Setup::Evaluate), "disabled");
            rejects(
                room.event(round, 1, F2Event::Objective { winner: None }),
                "disabled",
            );
            room.no_packets();
        }
        assert_eq!(plain.startup, visible.startup, "{family}");
    }
}

#[test]
fn extended_item_presets_conflict_and_unmeasured_rules_do_not_create_rooms() {
    for (left, right) in [
        ("no_shop", "basic_shop"),
        ("no_item", "no_shop"),
        ("no_item", "basic_shop"),
    ] {
        assert!(
            rejected_room_config(json!({"extended_rules": {left: true, right: true}}))
                .contains("mutually exclusive F2 item presets")
        );
    }
    for policy in [
        "no_item",
        "no_shop",
        "basic_shop",
        "super_host",
        "vote_kick",
        "ladder_season",
        "big_battle_12",
    ] {
        assert_eq!(
            rejected_room_config(json!({"extended_rules": {policy: true}})),
            format!("F2 {policy} policy is unmeasured")
        );
    }
}

#[test]
fn extended_priority_room_validates_slots_without_reserving_them() {
    for slot in 1..=50 {
        assert_eq!(
            rejected_room_config(json!({"extended_rules": {"priority_room": slot}})),
            format!("F2 priority_room S{slot:02} reservation is unmeasured")
        );
    }
    for slot in [0, 51, 256, u32::MAX] {
        let error = serde_json::from_value::<room_modes::Config>(json!({
            "extended_rules": {"priority_room": slot}
        }))
        .err()
        .unwrap();
        assert!(
            error
                .to_string()
                .contains("priority_room must be in 1..=50")
        );
    }
    for slot in [json!(-1), json!("S01"), json!(1.5), json!(true)] {
        assert!(
            serde_json::from_value::<room_modes::Config>(json!({
                "extended_rules": {"priority_room": slot}
            }))
            .is_err()
        );
    }
}

#[test]
fn extended_channel_kinds_are_recognized_but_grant_no_admission() {
    for channel in ["beginner", "event", "no_item", "practice"] {
        assert_eq!(
            rejected_room_config(json!({"extended_rules": {"channel_kind": channel}})),
            "F2 channel_kind policy is unmeasured"
        );
    }
    for channel in [json!("unknown"), json!("tournament"), json!(1)] {
        assert!(
            serde_json::from_value::<room_modes::Config>(json!({
                "extended_rules": {"channel_kind": channel}
            }))
            .is_err()
        );
    }
    assert!(
        serde_json::from_value::<room_modes::Config>(json!({
            "extended_rules": {"channel": "beginner"}
        }))
        .is_err()
    );
}

#[test]
fn tentative_map_families_reject_activation_and_exact_census_entries_reject_dispatch() {
    let matrix: Value = serde_json::from_str(include_str!(
        "../../../../docs/evidence/content/f2-extended-modes-matrix.json"
    ))
    .unwrap();
    let census: Value =
        serde_json::from_str(include_str!("../../../../docs/content/census.json")).unwrap();
    let families = matrix["map_families"].as_array().unwrap();
    assert_eq!(families.len(), 21);
    let cfg = serde_json::from_value(json!({
        "visible_map_groups": ["LadderMapGroup"],
        "create_overrides": {"manner": "ladder"},
        "runtime_overrides": {"free": "revive"}
    }))
    .unwrap();
    let policy = room_modes::Policy::load(&cfg).unwrap().unwrap();
    let mut map = wire::Start121Map {
        name: Vec::new(),
        map_code: 7,
        dynamic_index: 3,
        roomtype: 0,
    };
    let mut total = 0;
    for row in families {
        let family = row["family"].as_str().unwrap();
        assert_eq!(row["wave"], "F2?");
        assert_eq!(
            rejected_room_config(json!({"candidate_map_family": family})),
            "F2 candidate map family activation is unmeasured"
        );
        let mut count = 0;
        for entry in census["maps"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|m| m["category"] == "other" && m["family"] == family)
        {
            map.name = entry["name"].as_str().unwrap().as_bytes().to_vec();
            for team in [wire::RoomTeamSelector::Free, wire::RoomTeamSelector::Manner] {
                assert_eq!(
                    policy.check_start(team, &map, (7, 3)),
                    Err("f2_candidate_map_dispatch_unmeasured"),
                    "{:?}",
                    entry["name"]
                );
            }
            count += 1;
        }
        assert_eq!(row["map_count"], count, "{family}");
        total += count;
    }
    assert_eq!(total, 91);
    for unknown in ["BigBattle", "Factory", "baram"] {
        assert_eq!(
            rejected_room_config(json!({"candidate_map_family": unknown})),
            "unknown F2 candidate map family"
        );
    }
    map.name = b"Factory07".to_vec();
    assert_eq!(
        policy.check_start(wire::RoomTeamSelector::Free, &map, (7, 3)),
        Ok(())
    );
}

#[test]
fn revive_queues_one_ghost_frame_per_transport_and_keeps_respawn_and_win_in_runtime() {
    let mut room = TestRoom::new(opt_in("revive"));
    let round = room.load();
    rejects(room.start(round, revive(3)), "armed round");
    room.arm();
    assert_eq!(
        room.start(round, revive(3)).unwrap()["players"][1]["life"],
        "alive"
    );
    let ghost = room.event(round, 1, defeat(1)).unwrap();
    assert_eq!(ghost["players"][1]["life"], "ghost");
    assert_eq!(ghost["players"][0]["team_score"], 1);
    for client in &mut room.clients {
        let mut batch = client.inbox.try_recv().unwrap();
        assert!(client.prepare_delivery(&mut batch).unwrap());
        assert_eq!(batch.packets.len(), 1);
        let reply = client.project(&batch.packets[0].1).unwrap().unwrap();
        let frame = wire::encode(reply.opcode, &reply.payload, 17).unwrap();
        assert!(wire::header_matches(&frame[..37]));
        assert_eq!(&frame[5..9], &[185, 0, 0, 0]);
        assert_eq!(&frame[13..17], &[1, 0, 0, 0]);
        let decoded = kr_wire::decode(&frame, 17).unwrap();
        assert_eq!(decoded.payload, [0x5b]);
        assert_eq!(
            wire::RebirthToGhost::decode(&decoded.payload)
                .unwrap()
                .selector,
            1
        );
        client.finish(&batch).unwrap();
    }
    assert_eq!(room.event(round, 1, defeat(1)).unwrap()["replay"], true);
    room.no_packets();
    let respawned = room.event(round, 2, F2Event::Respawn { slot: 1 }).unwrap();
    assert_eq!(respawned["players"][1]["life"], "alive");
    room.no_packets();
    assert_eq!(
        room.event(round, 3, defeat(1)).unwrap()["outcome"],
        "playing"
    );
    rejects(
        room.event(round, 4, F2Event::Respawn { slot: 1 }),
        "NoRevives",
    );
    for client in &mut room.clients {
        assert_eq!(
            client.inbox.try_recv().unwrap().packets[0].1.payload,
            [0x5b]
        );
    }
    assert_eq!(
        room.event(round, 4, defeat(3)).unwrap()["outcome"],
        json!({"winner": 0})
    );
    rejects(
        room.event(round, 5, F2Event::Respawn { slot: 3 }),
        "Finished",
    );
    for client in &mut room.clients {
        client.tick().unwrap();
        let batch = client.inbox.try_recv().unwrap();
        assert_eq!(batch.packets.len(), 1);
        assert_eq!(batch.packets[0].1.opcode, 185);
        assert_eq!(batch.packets[0].1.payload, [0x59]);
    }
    room.no_packets();
}

#[test]
fn timebomb_pass_preserves_deadline_and_fuse_elimination_decides_outcome_without_wire() {
    let mut room = TestRoom::new(opt_in("timebomb"));
    let round = room.load();
    room.arm();
    room.start(
        round,
        F2Setup::Timebomb {
            initial_carrier: 0,
            fuse_ms: 10,
        },
    )
    .unwrap();
    let advance = |ms| F2Event::AdvanceTo { milliseconds: ms };
    let pass = |from, to| F2Event::PassBomb { from, to };
    room.event(round, 1, advance(6)).unwrap();
    assert_eq!(
        room.event(round, 2, pass(0, 1)).unwrap()["bomb"],
        json!([1, 4])
    );
    let first = room.event(round, 3, advance(10)).unwrap();
    assert_eq!(first["players"][1]["life"], "eliminated");
    assert_eq!(first["bomb"], json!([0, 10]));
    assert_eq!(first["outcome"], "playing");
    room.event(round, 4, pass(0, 3)).unwrap();
    let last = room.event(round, 5, advance(20)).unwrap();
    assert_eq!(last["players"][3]["life"], "eliminated");
    assert_eq!(last["outcome"], json!({"winner": 0}));
    assert!(last["bomb"].is_null());
    room.no_packets();
}

#[test]
fn ghost_queue_backpressure_leaves_other_rooms_and_event_sequence_unchanged() {
    let hub = hub(opt_in("revive"));
    let mut room = TestRoom::on(hub.clone(), false);
    let mut other = TestRoom::on(hub, true);
    let round = room.load();
    room.arm();
    room.start(round, revive(3)).unwrap();
    let full = registry_foundation_tests::fill_recipient_queue(&room.room, 2);
    assert!(full > 0);
    rejects(
        room.event(round, 1, defeat(1)),
        "recipient queue unavailable",
    );
    for (slot, client) in room.clients.iter_mut().enumerate() {
        let packets = std::iter::from_fn(|| client.inbox.try_recv().ok()).collect::<Vec<_>>();
        assert_eq!(packets.len(), if slot == 2 { full } else { 0 });
        assert!(packets.iter().all(|batch| batch.packets.is_empty()));
    }
    let retry = room.event(round, 1, defeat(1)).unwrap();
    assert_eq!(retry["players"][1]["life"], "ghost");
    assert_eq!(retry["players"][0]["team_score"], 1);
    for client in &mut room.clients {
        let batch = client.inbox.try_recv().unwrap();
        assert_eq!(batch.packets[0].1.opcode, 185);
        assert!(!other.clients[0].valid(&batch));
    }
    other.no_packets();
    room.no_packets();
}

#[test]
fn leader_capture_settles_only_when_a_leader_falls_without_wire() {
    let mut room = TestRoom::new(opt_in("leader_capture"));
    let round = room.load();
    room.arm();
    room.start(
        round,
        F2Setup::LeaderCapture {
            leaders: vec![0, 1],
        },
    )
    .unwrap();
    let nonleader = room.event(round, 1, defeat(3)).unwrap();
    assert_eq!(nonleader["players"][3]["life"], "eliminated");
    assert_eq!(nonleader["outcome"], "playing");
    assert_eq!(
        room.event(round, 2, defeat(0)).unwrap()["outcome"],
        json!({"winner": 1})
    );
    room.no_packets();
}

#[test]
fn hedgehog_preserves_unknown_outcome_without_packets() {
    let mut room = TestRoom::new(json!({"runtime_overrides": {"free": {"game_type_id": 11}}}));
    let round = room.load();
    room.arm();
    // Parse the same nested command that kr-admin accepts.
    let start = serde_json::from_value(json!({
        "command": "start", "round": round, "setup": {"mode": "evaluate"}
    }))
    .unwrap();
    assert_eq!(room.run(start).unwrap()["outcome"], "playing");
    rejects(
        room.event(round, 1, F2Event::Objective { winner: Some(0) }),
        "UnsupportedAction",
    );
    rejects(room.event(round, 1, defeat(1)), "UnsupportedAction");
    let result = room
        .event(round, 1, F2Event::Objective { winner: None })
        .unwrap();
    assert_eq!(result["outcome"], "unresolved");
    assert_eq!(result["players"][1]["life"], "alive");
    assert_eq!(room.event(round, 1, defeat(1)).unwrap()["replay"], true);
    rejects(room.event(round, 2, defeat(1)), "Finished");
    room.no_packets();
}

#[test]
fn free_1v7_requires_asymmetric_roster_and_uses_surviving_team_without_packets() {
    let guests = [43, 501, 71, 88, 602, 777, 1024];
    let mut balanced = TestRoom::with_guests(
        eight_player_hub("4v4", &[0, 1, 0, 1, 0, 1, 0, 1]),
        false,
        &guests,
    );
    let round = balanced.load();
    balanced.arm();
    rejects(balanced.start(round, F2Setup::Evaluate), "InvalidRoster");
    balanced.no_packets();

    let mut room = TestRoom::with_guests(
        eight_player_hub("1v7", &[0, 1, 1, 1, 1, 1, 1, 1]),
        false,
        &guests,
    );
    let round = room.load();
    room.arm();
    let start = room.start(round, F2Setup::Evaluate).unwrap();
    let players = start["players"].as_array().unwrap();
    assert_eq!(players.len(), 8);
    assert_eq!(players[0]["team"], 0);
    assert!(players[1..].iter().all(|player| player["team"] == 1));
    let first = room.event(round, 1, defeat(1)).unwrap();
    assert_eq!(first["players"][1]["life"], "eliminated");
    assert_eq!(first["outcome"], "playing");
    rejects(room.event(round, 2, defeat(1)), "InvalidTransition");
    let last = room.event(round, 2, defeat(0)).unwrap();
    assert_eq!(last["players"][0]["life"], "eliminated");
    assert_eq!(last["outcome"], json!({"winner": 1}));
    room.no_packets();
}

#[test]
fn co_op_uses_only_an_authoritative_objective_result_without_packets() {
    let mut room = TestRoom::new(opt_in("co_op"));
    let round = room.load();
    room.arm();
    assert_eq!(
        room.start(round, F2Setup::Evaluate).unwrap()["outcome"],
        "playing"
    );
    rejects(room.event(round, 1, defeat(1)), "UnsupportedAction");
    rejects(
        room.event(round, 1, F2Event::Objective { winner: Some(2) }),
        "UnknownTeam",
    );
    let result = room
        .event(round, 1, F2Event::Objective { winner: Some(1) })
        .unwrap();
    assert_eq!(result["outcome"], json!({"winner": 1}));
    assert_eq!(result["players"][1]["life"], "alive");
    rejects(
        room.event(round, 2, F2Event::Objective { winner: None }),
        "Finished",
    );
    room.no_packets();
}

#[test]
fn game_type_ids_fail_closed_and_do_not_bypass_native_or_runtime_gates() {
    for id in [12, 14, 16, 17, 20, 21, 25, 27, 28, 29, 43, 100] {
        assert!(wire::GameTypeId::try_from(id).is_ok());
        let cfg =
            serde_json::from_value(json!({"runtime_overrides": {"free": {"game_type_id": id}}}))
                .unwrap();
        assert!(
            room_modes::Policy::load(&cfg)
                .err()
                .unwrap()
                .to_string()
                .contains("unsupported F2 game type alias")
        );
    }
    for id in [1, 15, 266, u32::MAX] {
        assert!(
            serde_json::from_value::<room_modes::Config>(json!({
                "runtime_overrides": {"free": {"game_type_id": id}}
            }))
            .is_err()
        );
    }
    let f3 = serde_json::from_value(json!({"create_overrides": {"free": {"game_type_id": 19}}}))
        .unwrap();
    assert_eq!(
        room_modes::Policy::load(&f3).err().unwrap().to_string(),
        "room create override is outside F2"
    );
    {
        let hub = hub(json!({"runtime_overrides": {"free": {"game_type_id": 19}}}));
        assert_eq!(
            hub.select_fixture(81, &create(false), 70000)
                .err()
                .unwrap()
                .to_string(),
            "F2 runtime: UnsupportedMode"
        );
        assert!(hub.operator_rooms().unwrap().is_empty());
    }

    let hub = hub(json!({"create_overrides": {"free": {"game_type_id": 10}}}));
    assert_eq!(
        hub.select_fixture(81, &create(false), 70000)
            .err()
            .unwrap()
            .to_string(),
        "f2_map_group_hidden"
    );
    assert!(hub.operator_rooms().unwrap().is_empty());
}

#[test]
fn map_group_visibility_codec_and_named_policy_have_the_same_room_gate() {
    // Arbitrary operator bindings for this test; these are not KR group IDs.
    let bindings = json!({"LadderMapGroup": 0x1234, "ReBirthMapGroup": 0xabcd});
    for (bytes, named, ladder, revive) in [
        (
            vec![],
            json!([]),
            "f2_map_group_hidden",
            "f2_map_group_hidden",
        ),
        (
            vec![0x34, 0x12],
            json!(["LadderMapGroup"]),
            "f2_start_dispatch_unmeasured",
            "f2_map_group_hidden",
        ),
        (
            vec![0xcd, 0xab],
            json!(["ReBirthMapGroup"]),
            "f2_map_group_hidden",
            "f2_map_not_confirmed_in_group_or_family",
        ),
        (
            vec![0xcd, 0xab, 0x34, 0x12, 0xcd, 0xab],
            json!(["LadderMapGroup", "ReBirthMapGroup"]),
            "f2_start_dispatch_unmeasured",
            "f2_map_not_confirmed_in_group_or_family",
        ),
    ] {
        let decoded = wire::MapGroupVisible::decode(&bytes).unwrap();
        assert_eq!(decoded.encode().unwrap(), bytes);
        let encoded = json!({"bytes": bytes, "group_ids": bindings});
        for visibility in [named, encoded] {
            let hub = hub(json!({
                "visible_map_groups": visibility,
                "create_overrides": {"free": "ladder", "manner": {"game_type_id": 10}}
            }));
            for (manner, expected) in [(false, ladder), (true, revive)] {
                let result = hub.select_fixture(81, &create(manner), 70000);
                assert_eq!(result.err().unwrap().to_string(), expected);
            }
            assert!(hub.operator_rooms().unwrap().is_empty());
        }
    }
    for (bytes, bindings, expected) in [
        (
            vec![0x34],
            bindings.clone(),
            "partial MapGroupVisible entry",
        ),
        (
            vec![0; 3002],
            bindings.clone(),
            "oversized MapGroupVisible value",
        ),
        (
            vec![0xff, 0xff],
            bindings.clone(),
            "unbound MapGroupVisible entry",
        ),
        (
            vec![0x34, 0x12],
            json!({"LadderMapGroup": 0x1234, "ReBirthMapGroup": 0x1234}),
            "ambiguous MapGroupVisible binding",
        ),
    ] {
        let cfg = serde_json::from_value(
            json!({"visible_map_groups": {"bytes": bytes, "group_ids": bindings}}),
        )
        .unwrap();
        assert!(
            room_modes::Policy::load(&cfg)
                .err()
                .unwrap()
                .to_string()
                .contains(expected)
        );
    }
}

#[test]
fn rejects_unsupported_presets_and_bad_events_without_consuming_round_or_sequence() {
    for mode in ["ladder", "confused_fight", "baseball"] {
        let hub = hub(opt_in(mode));
        let denied = hub.select_fixture(81, &create(false), 70000);
        assert_eq!(
            denied.err().unwrap().to_string(),
            "F2 runtime: UnsupportedMode"
        );
        assert!(hub.operator_rooms().unwrap().is_empty());
        let other = TestRoom::on(hub, true);
        assert_eq!(other.key.session, 1);
    }

    let mut room = TestRoom::new(opt_in("revive"));
    let round = room.load();
    room.arm();
    rejects(room.event(round, 1, defeat(1)), "WrongRound");
    let late = room.hub.select_fixture(82, &join(room.key), 704);
    assert_eq!(
        late.err().unwrap().to_string(),
        "F2 runtime freezes admission during a round"
    );
    rejects(room.start(round + 1, revive(3)), "armed round");
    rejects(
        room.start(
            round,
            F2Setup::Timebomb {
                initial_carrier: 0,
                fuse_ms: 10,
            },
        ),
        "InvalidSetup",
    );
    rejects(room.start(round, revive(0)), "InvalidSetup");
    room.start(round, revive(3)).unwrap();
    rejects(room.start(round, revive(3)), "WrongRound");
    rejects(
        room.event(round, 1, F2Event::Respawn { slot: 1 }),
        "InvalidTransition",
    );
    rejects(room.event(round, 2, defeat(1)), "SequenceGap");
    rejects(
        room.event(round, 1, F2Event::PassBomb { from: 0, to: 1 }),
        "UnsupportedAction",
    );
    room.event(round, 1, F2Event::AdvanceTo { milliseconds: 21 })
        .unwrap();
    rejects(
        room.event(round, 2, F2Event::AdvanceTo { milliseconds: 20 }),
        "ClockReversed",
    );
    assert!(room.clients[1].packet(185, &[0x5b]).is_err());
    room.clients[1].packet(127, &[]).unwrap();
    room.no_packets();
    assert_eq!(
        room.event(round, 2, defeat(1)).unwrap()["players"][1]["life"],
        "ghost"
    );

    // A departure revokes the round and the queued ghost frame, even before
    // another transport drains its inbox.
    drop(room.clients.remove(1));
    let batch = room.clients[0].inbox.try_recv().unwrap();
    assert!(!room.clients[0].valid(&batch));
    rejects(room.event(round, 3, defeat(3)), "armed round");
}
