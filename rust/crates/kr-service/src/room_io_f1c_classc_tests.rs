//! Rust room packet consumer tests; the Start template is synthetic.
use super::*;
use crate::characters::{Catalog, Stats};

fn class_c_keys() -> Vec<u16> {
    (4000..=4018)
        .chain(4020..=4024)
        .chain(4027..=4030)
        .chain(5000..=5004)
        .collect()
}

#[test]
fn catalog_requires_opt_in_and_only_admits_pinned_class_c_rows() {
    let keys = [vec![0, 1], class_c_keys()].concat();
    assert_eq!(keys.len() - 2, 33);
    assert!(Catalog::player_round(&keys, None, false).is_err());
    let catalog = Catalog::player_round(&keys, None, true).unwrap();
    for &(key, expected) in &[
        (4000, (5, 9, 2, 9, 1, 9)),
        (4006, (7, 10, 3, 10, 3, 10)),
        (4024, (6, 9, 3, 7, 5, 8)),
        (5003, (2, 2, 1, 1, 3, 7)),
    ] {
        let Stats {
            speed,
            maximum_speed,
            capacity,
            maximum_capacity,
            power,
            maximum_power,
        } = catalog.get(key).unwrap();
        assert_eq!(
            (
                speed,
                maximum_speed,
                capacity,
                maximum_capacity,
                power,
                maximum_power
            ),
            expected
        );
    }
    for &key in &keys[2..] {
        assert!(catalog.get(key).is_ok(), "key {key}");
    }
    for key in [3999, 4019, 4025, 4026, 4031, 5005, 0x1fff, u16::MAX] {
        assert!(
            Catalog::player_round(&[0, key], None, true).is_err(),
            "key {key}"
        );
    }
    assert!(Catalog::player_round(&[0, 4000, 4000], None, true).is_err());
    assert!(Catalog::player_round(&[4000], None, true).is_err());
}

#[test]
fn class_c_uses_pinned_stats_alongside_the_existing_pack_catalog() {
    use std::collections::BTreeMap;
    let mut rows = BTreeMap::from([(0, Catalog::measured(&[0]).unwrap().get(0).unwrap())]);
    for key in [120, 3000] {
        rows.insert(key, rows[&0]);
    }
    let keys = [0, 120, 3000, 4000];
    let catalog = Catalog::player_round(&keys, Some(&rows), true).unwrap();
    assert!(Catalog::player_round(&keys, Some(&rows), false).is_err());
    assert_eq!(catalog.get(120).unwrap(), rows[&120]);
    assert_eq!(catalog.get(3000).unwrap(), rows[&3000]);
    assert_eq!(catalog.get(4000).unwrap().speed, 5);
    // A pack configuration cannot replace the pinned class C stat vector.
    rows.insert(
        4000,
        Stats {
            speed: 11,
            ..rows[&0]
        },
    );
    assert!(Catalog::player_round(&keys, Some(&rows), true).is_err());
}

fn room(config: &str, characters: &[u16]) -> (Shared, [Session; 2], [Membership; 2]) {
    let config: Config = serde_json::from_str(config).unwrap();
    let prototype = registry_foundation_tests::player_round_prototype();
    {
        let mut r = prototype.lock().unwrap();
        r.rules = RoomRules::from_config(&config, 2).unwrap().0;
        let template = wire::PlayerTemplate::parse(&r.players[&1].payload).unwrap();
        r.players
            .insert(2, packet(106, template.project(2, b"Bobby").unwrap()));
    }
    let profile = load_player_round(
        &PlayerRoundConfig {
            characters: characters.to_vec(),
            pack_stats: None,
        },
        &prototype,
    )
    .unwrap();
    let shared = fork(
        &prototype,
        crate::room_registry::RoomKey {
            id: crate::room_registry::RoomId(1),
            session: 1,
            generation: 1,
        },
        Arc::default(),
        Some(profile),
        true,
        None,
    )
    .unwrap();
    let players = [1, 2].map(|id| Session::open(&shared, id, u64::from(id)).unwrap());
    let keys = {
        let mut r = shared.lock().unwrap();
        players.each_ref().map(|player| {
            r.model
                .enter(player.identity, player.connection, player.identity == 1)
                .unwrap()
        })
    };
    (shared, players, keys)
}

const OFF: &str = r#"{"title":"Test","packets":{}}"#;
const ON: &str = r#"{"title":"Test","packets":{},"allow_class_c_characters":true}"#;

fn choice(key: u16) -> [u8; 3] {
    let [hi, lo] = (key ^ 0xa569).to_be_bytes();
    [0x5a, hi, lo]
}

fn setup(shared: &Shared, keys: [Membership; 2]) {
    let mut r = shared.lock().unwrap();
    for key in keys {
        r.model.setup(key).unwrap();
    }
    r.model.choose(keys[1], None, Some(3)).unwrap();
}

fn start(players: &mut [Session; 2], chars: [u16; 2]) {
    assert!(
        players[0]
            .packet(104, &[0; 20])
            .unwrap()
            .start_rejection
            .is_none()
    );
    for player in players {
        let batch = player.inbox.try_recv().unwrap();
        assert!(player.valid(&batch));
        assert_eq!(
            batch
                .packets
                .iter()
                .map(|(_, p)| p.opcode)
                .collect::<Vec<_>>(),
            [582, 121]
        );
        let body = &batch.packets[1].1.payload;
        assert_eq!(
            wire::start121_characters(body).unwrap(),
            [(0, chars[0]), (1, chars[1])]
        );
        let size = read_uint(&body[1..5], Width::U32).unwrap() as usize;
        let roster = wire::decode_bytes(&body[5..5 + size]).unwrap();
        // Roster rows are 26 bytes with a little-endian u16 at row+3.
        let rows = &roster[roster.len() - 52..];
        assert_eq!(&rows[3..5], &chars[0].to_le_bytes());
        assert_eq!(&rows[29..31], &chars[1].to_le_bytes());
    }
}

#[test]
fn class_c_selection_retains_full_start121_key_and_observes_loading79_once() {
    let available = [vec![0, 1], class_c_keys()].concat();
    for key in class_c_keys() {
        let (shared, mut players, keys) = room(ON, &available);
        setup(&shared, keys);
        let body = choice(key);
        let selection = players[0].prepare_character(&body).unwrap().unwrap();
        assert_eq!(selection, (keys[0], key));
        assert!(
            players[0]
                .persisted_character(selection.0, &body)
                .unwrap()
                .character_rejection
                .is_none()
        );
        for player in &mut players {
            let batch = player.inbox.try_recv().unwrap();
            assert!(batch.packets.iter().all(|(_, packet)| packet.opcode == 110));
        }
        // XOR masks from the KR writer. A Class C key's low byte belongs
        // to the selected member, and byte 5 is the ownership fallback.
        let low = [0x5b, ((key % 256) as u8) ^ 0x5a];
        let fallback = [0x5b, 0x5f];
        assert!(players[0].packet(79, &low).is_err()); // Still Idle.
        shared.lock().unwrap().model.ready(keys[1]).unwrap();
        start(&mut players, [key, 0]);
        let before = shared.lock().unwrap().model.phase.clone();
        assert!(matches!(before, Phase::Loading { .. }));
        assert!(players[0].packet(79, &[0x58, low[1]]).is_err()); // Count 2.
        assert!(players[0].packet(79, &[0x5b, 0x5a]).is_err()); // Wrong byte.
        assert!(players[1].packet(79, &low).is_err()); // Wrong member.
        let (first, second) = if key % 2 == 0 {
            (low, fallback)
        } else {
            (fallback, low)
        };
        players[0].packet(79, &first).unwrap();
        assert!(players[0].packet(79, &second).is_err()); // Shared budget.
        assert!(players[1].packet(79, &fallback).is_err()); // Ordinary choice.
        players[1].packet(79, &[0x5b, 0x5a]).unwrap();
        let r = shared.lock().unwrap();
        assert_eq!(r.model.phase, before);
        assert_eq!(r.model.members[&1].character, Some(key));
        assert_eq!(r.model.members[&2].character, Some(0));
        assert!(
            players
                .iter_mut()
                .all(|player| player.inbox.try_recv().is_err())
        );
    }
}

#[test]
fn class_c_disabled_unconfigured_and_unknown_choices_leave_room_intact() {
    for (config, available, rejected) in [
        (OFF, &[0, 1][..], &[4000, 5000, 65535][..]),
        (
            ON,
            &[0, 1, 4000][..],
            &[4001, 4019, 4025, 4026, 5005, 65535][..],
        ),
    ] {
        let (shared, mut players, keys) = room(config, available);
        setup(&shared, keys);
        shared.lock().unwrap().model.ready(keys[1]).unwrap();
        for &key in rejected {
            let body = choice(key);
            assert!(players[0].prepare_character(&body).unwrap().is_none());
            assert_eq!(
                players[0].packet(92, &body).unwrap().character_rejection,
                Some("unmeasured_character")
            );
            let batch = players[0].inbox.try_recv().unwrap();
            assert_eq!(batch.packets.len(), 1);
            assert_eq!(batch.packets[0].1.opcode, 110);
            assert!(players[1].inbox.try_recv().is_err());
            let r = shared.lock().unwrap();
            assert_eq!(r.model.members[&1].character, Some(0));
            assert!(r.model.members[&2].ready);
        }
        start(&mut players, [0, 0]);
        assert!(players[0].packet(79, &[0x5b, 0xfa]).is_err()); // Low byte of 4000.
        assert!(players[0].packet(79, &[0x5b, 0x5f]).is_err()); // Fallback.
        players[0].packet(79, &[0x5b, 0x5a]).unwrap();
    }
}

#[test]
fn class_c_preference_is_restored_only_for_current_members_before_setup() {
    let (shared, mut players, keys) = room(ON, &[0, 1, 5004]);
    assert!(players[0].restore_character(5000).is_err());
    players[0].restore_character(5004).unwrap();
    setup(&shared, keys);
    assert!(players[1].restore_character(5004).is_err());
    shared.lock().unwrap().model.ready(keys[1]).unwrap();
    start(&mut players, [5004, 0]);

    let (_shared, players, _keys) = room(OFF, &[0, 1]);
    assert!(players[0].restore_character(5004).is_err());
    players[0].restore_character(1).unwrap();
}
