//! Synthetic integration of current-roster basic-item policy; not native acceptance.
use super::*;
use crate::rules::{LifePolicy, RoundRoster, SlotAllocation};
use kr_wire::write_uint;
fn bytes(s: &str) -> Vec<u8> {
    s.as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|x| u8::from_str_radix(std::str::from_utf8(x).unwrap(), 16).unwrap())
        .collect()
}
// Exact CP239 catalog body from the retained independently captured profile.
const FLOOR: &str =
    "96ca53bf96ca768d96cb539596ca538496c8509196c8539596ca538e96ca509796d6539594c9539596ca5394a569";
struct Case {
    shared: Shared,
    sessions: Vec<Session>,
    keys: Vec<Membership>,
    round: u64,
}
impl Drop for Case {
    fn drop(&mut self) {
        self.shared.lock().unwrap().channels.clear();
    }
}
fn case(count: usize) -> Case {
    let catalog = pickups::Catalog::from_floor(&bytes(FLOOR)).unwrap();
    let mut model = Room::with_allocation(8, SlotAllocation::FirstVacant).unwrap();
    model.round_roster = RoundRoster::Players;
    let mut keys = Vec::new();
    let mut receivers = Vec::new();
    let mut channels = HashMap::new();
    for index in 0..count {
        let identity = 70000 + index as u32 * 13;
        let connection = 100 + index as u64;
        let key = model.enter(identity, connection, index == 0).unwrap();
        model.setup(key).unwrap();
        model.choose(key, Some(0), None).unwrap();
        model
            .choose(key, None, Some(if index % 2 == 0 { 0 } else { 3 }))
            .unwrap();
        keys.push(key);
        let (sender, inbox) = mpsc::channel(8);
        receivers.push(inbox);
        channels.insert(
            identity,
            Channel {
                connection,
                sender,
                endpoint: 0,
                relay_authority: None,
            },
        );
    }
    let terrain = crate::terrain::tests::basic_items_map();
    model.terrain = Some(terrain);
    for key in &keys[1..] {
        model.ready(*key).unwrap();
    }
    let round = model.begin(keys[0]).unwrap();
    for key in &keys {
        model.ack(*key, round).unwrap();
    }
    let characters = crate::characters::Catalog::measured(&[0, 1]).unwrap();
    for member in model.members.values_mut() {
        member.item_stats = Some(characters.get(0).unwrap());
        model.bomb_capacity[usize::from(member.slot)] = 1;
    }
    model.pickups = pickups::Round::begin(round, &catalog).unwrap();
    let rules = RoomRules {
        item_policy: pickups::Policy::ReportDerivedBasic,
        life: LifePolicy::TeamReports,
        roster: RoundRoster::Players,
        ..RoomRules::default()
    };
    let shared = Arc::new(Mutex::new(Runtime {
        maps: None,
        selected_map: 0,
        map_revision: 0,
        journal: None,
        public_key: None,
        managed_round: Some(Arc::new(ManagedRound {
            characters: Some(characters),
            bodies: HashMap::new(),
            initial_bomb_capacity: Vec::new(),
            rules,
            limits: rules::ReportLimits {
                power: Vec::new(),
                opaque167: rules::ReportLimits::PLAYER_OPAQUE167,
            },
        })),
        relay: None,
        authenticated: None,
        scope: Arc::new(()),
        model,
        players: HashMap::new(),
        bodies: HashMap::new(),
        channels,
        last_visits: HashMap::new(),
        requested: HashSet::new(),
        rostered: HashSet::new(),
        title: b"Test".to_vec(),
        selector: 0,
        ready: packet(241, vec![]),
        ips: HashMap::new(),
        ports: HashMap::new(),
        floor_catalog: catalog,
        rules,
        abort: None,
    }));
    let sessions = receivers
        .into_iter()
        .zip(&keys)
        .map(|(inbox, key)| Session {
            shared: shared.clone(),
            inbox,
            identity: key.identity,
            connection: key.connection,
        })
        .collect();
    Case {
        shared,
        sessions,
        keys,
        round,
    }
}
fn encoded(fields: &[(u64, Width)]) -> Vec<u8> {
    fields
        .iter()
        .flat_map(|&(v, w)| write_uint(v, w).unwrap())
        .collect()
}
fn pickup(slot: u8, row: u8, column: u8, index: u32, kind: u16, key: u32) -> Vec<u8> {
    encoded(&[
        (0, Width::U8),
        (slot.into(), Width::U8),
        (row.into(), Width::U8),
        (column.into(), Width::U8),
        (index.into(), Width::U32),
        (kind.into(), Width::U16),
        (500, Width::U32),
        (key.into(), Width::U32),
        (0, Width::U8),
    ])
}
fn reveal(slot: u8, row: u8, column: u8) -> Vec<u8> {
    encoded(&[
        (row.into(), Width::U8),
        (column.into(), Width::U8),
        (slot.into(), Width::U8),
        (u32::MAX.into(), Width::U32),
    ])
}
fn bomb(slot: u8, serial: u32, power: u8, key: u32) -> Vec<u8> {
    encoded(&[
        (key.into(), Width::U32),
        (0, Width::U8),
        (slot.into(), Width::U8),
        (serial.into(), Width::U32),
        (3, Width::U8),
        (3, Width::U8),
        (1, Width::U8),
        (0, Width::U8),
        (0, Width::U8),
        (0, Width::U8),
        (power.into(), Width::U8),
        (1, Width::U32),
        (0, Width::U8),
    ])
}
fn confirmations(c: &mut Case) {
    for s in &mut c.sessions {
        let b = s.inbox.try_recv().unwrap();
        assert!(s.valid(&b));
        assert_eq!(b.packets[0].1.opcode, 165);
        s.finish(&b).unwrap();
    }
}
#[test]
fn captured_patrit167_is_bounded_current_observation_without_item_or_reply_effects() {
    // Actual Patrit8-r1 PCAP, identity1/channel167/body52/sequence352.
    let body = bytes(
        "a569a56c96ca62aca569a56d6935ac6aa563a56f5a5b6935ac6aa560a56e5a5b6935ac6aa562a56e5b5b6935ac6aa563a5615b5b",
    );
    let mut c = case(3);
    let before = {
        let r = c.shared.lock().unwrap();
        (
            r.model.phase.clone(),
            r.model.members[&c.keys[0].identity].item_stats,
        )
    };
    c.sessions[0].connection += 1;
    assert!(c.sessions[0].packet(167, &body).is_err());
    c.sessions[0].connection -= 1;
    for invalid in [&body[..51], &[0; 53][..]] {
        assert!(c.sessions[0].packet(167, invalid).is_err());
    }
    // Repeated opaque observations cannot produce world effects, extend the
    // per-round budget, or publish acknowledgements to any recipient.
    for _ in 0..64 {
        c.sessions[0].packet(167, &body).unwrap();
    }
    assert!(c.sessions[0].packet(167, &body).is_err());
    assert!(c.sessions.iter_mut().all(|s| s.inbox.try_recv().is_err()));
    let r = c.shared.lock().unwrap();
    assert_eq!(
        (
            r.model.phase.clone(),
            r.model.members[&c.keys[0].identity].item_stats
        ),
        before
    );
    assert!(r.model.pickups.hidden_cell(2, 3));
    assert!(r.model.last_blast.is_none());
}
#[test]
fn basic_pickup_queue_atomicity_current_owner_and_duplicate_are_independent_of_slot_identity() {
    for count in [4, 8] {
        let mut c = case(count);
        let slot = (count - 1) as u8;
        let i = usize::from(slot);
        let key = crate::room::wire_key(c.round).unwrap();
        let body = pickup(slot, 4, 3, 17, 2, key);
        c.sessions[i].connection += 1;
        assert!(c.sessions[i].packet(124, &body).is_err());
        c.sessions[i].connection -= 1;
        {
            let r = c.shared.lock().unwrap();
            for _ in 0..8 {
                r.channels[&c.keys[0].identity]
                    .sender
                    .try_send(Runtime::member_batch(c.keys[0], Some(c.round), 0, vec![]))
                    .unwrap_or_else(|_| panic!("fill bounded queue"));
            }
        }
        assert!(c.sessions[i].packet(124, &body).is_err());
        assert!(c.sessions[i].inbox.try_recv().is_err());
        assert_eq!(
            c.shared.lock().unwrap().model.members[&c.keys[i].identity]
                .item_stats
                .unwrap()
                .speed,
            5
        );
        for _ in 0..8 {
            c.sessions[0].inbox.try_recv().unwrap();
        }
        assert!(
            c.sessions[i]
                .packet(124, &body)
                .unwrap()
                .pickup
                .unwrap()
                .accepted
        );
        confirmations(&mut c);
        assert_eq!(
            c.shared.lock().unwrap().model.members[&c.keys[i].identity]
                .item_stats
                .unwrap()
                .speed,
            6
        );
        assert!(
            !c.sessions[i]
                .packet(124, &body)
                .unwrap()
                .pickup
                .unwrap()
                .accepted
        );
        assert!(c.sessions.iter_mut().all(|s| s.inbox.try_recv().is_err()));
    }
}
#[test]
fn hidden_items_wait_for_last_current_blast_write_then_adjust_current_character_power_and_capacity()
{
    for count in [4, 8] {
        let mut c = case(count);
        let i = count - 1;
        let slot = i as u8;
        let key = crate::room::wire_key(c.round).unwrap();
        let cap = pickup(slot, 2, 3, 27, 0, key);
        assert!(
            !c.sessions[i]
                .packet(124, &cap)
                .unwrap()
                .pickup
                .unwrap()
                .accepted
        );
        assert_eq!(
            c.sessions[i]
                .packet(142, &reveal(slot, 2, 3))
                .unwrap()
                .terrain_notice,
            Some("no_current_committed_clearance")
        );
        assert!(
            c.sessions[i]
                .packet(118, &bomb(slot, 1, 1, key))
                .unwrap()
                .bomb
                .unwrap()
                .accepted
        );
        let mut blasts = Vec::new();
        for s in &mut c.sessions {
            let spawn = s.inbox.try_recv().unwrap();
            assert_eq!(spawn.packets[0].1.opcode, 159);
            let blast = s.inbox.try_recv().unwrap();
            assert_eq!(blast.packets[0].1.opcode, 158);
            blasts.push(blast);
        }
        for (s, b) in c.sessions.iter().zip(&blasts).take(count - 1) {
            assert!(s.valid(b));
            s.finish(b).unwrap();
        }
        assert_eq!(
            c.sessions[i]
                .packet(142, &reveal(slot, 2, 3))
                .unwrap()
                .terrain_notice,
            Some("no_current_committed_clearance")
        );
        c.sessions[i].finish(&blasts[i]).unwrap();
        for (row, col) in [(2, 3), (3, 2)] {
            assert_eq!(
                c.sessions[i]
                    .packet(142, &reveal(slot, row, col))
                    .unwrap()
                    .terrain_notice,
                Some("revealed_current_hidden_cell")
            );
        }
        assert!(
            c.sessions[i]
                .packet(124, &cap)
                .unwrap()
                .pickup
                .unwrap()
                .accepted
        );
        confirmations(&mut c);
        let fire = pickup(slot, 3, 2, 28, 1, key);
        assert!(
            c.sessions[i]
                .packet(124, &fire)
                .unwrap()
                .pickup
                .unwrap()
                .accepted
        );
        confirmations(&mut c);
        {
            let r = c.shared.lock().unwrap();
            let stats = r.model.members[&c.keys[i].identity].item_stats.unwrap();
            assert_eq!((stats.capacity, stats.power), (2, 2));
            assert_eq!(r.model.bomb_capacity[i], 2);
        }
        assert!(
            !c.sessions[i]
                .packet(118, &bomb(slot, 2, 1, key))
                .unwrap()
                .bomb
                .unwrap()
                .accepted
        );
        assert!(
            c.sessions[i]
                .packet(118, &bomb(slot, 2, 2, key))
                .unwrap()
                .bomb
                .unwrap()
                .accepted
        );
    }
}
#[test]
fn final_pending_recipient_departure_commits_only_delivered_current_terrain_blast_and_reset_clears_stats()
 {
    let mut c = case(4);
    let key = crate::room::wire_key(c.round).unwrap();
    assert!(
        c.sessions[0]
            .packet(118, &bomb(0, 1, 1, key))
            .unwrap()
            .bomb
            .unwrap()
            .accepted
    );
    let mut blasts = Vec::new();
    for s in &mut c.sessions {
        s.inbox.try_recv().unwrap();
        blasts.push(s.inbox.try_recv().unwrap());
    }
    for (s, b) in c.sessions.iter().zip(&blasts).take(3) {
        s.finish(b).unwrap();
    }
    {
        let mut r = c.shared.lock().unwrap();
        assert!(
            r.model
                .terrain
                .as_ref()
                .unwrap()
                .clearance(c.round, 2, 3)
                .is_none()
        );
        r.model.depart_player(c.keys[3], Instant::now()).unwrap();
        assert!(
            r.model
                .terrain
                .as_ref()
                .unwrap()
                .clearance(c.round, 2, 3)
                .is_some()
        );
    }
    assert!(!c.sessions[3].valid(&blasts[3]));
    assert!(c.sessions[3].finish(&blasts[3]).is_err());
    {
        let mut r = c.shared.lock().unwrap();
        r.model.end(c.round).unwrap();
        for key in &c.keys[..3] {
            r.model.refreshed(*key, c.round).unwrap();
        }
        for key in &c.keys[1..3] {
            r.model.ready(*key).unwrap();
        }
        let next = r.model.begin(c.keys[0]).unwrap();
        assert_eq!(next, c.round + 1);
        assert!(r.model.members.values().all(|m| m.item_stats.is_none()));
        assert!(
            r.model
                .terrain
                .as_ref()
                .unwrap()
                .clearance(c.round, 2, 3)
                .is_none()
        );
    }
    assert!(!c.sessions[0].valid(&blasts[0]));
}
#[test]
fn sibling_room_and_current_generation_controls_do_not_publish_foreign_or_malformed_item_work() {
    let mut a = case(4);
    let mut b = case(4);
    let key = crate::room::wire_key(a.round).unwrap();
    let body = pickup(0, 4, 3, 17, 2, key);
    assert!(a.sessions[0].packet(124, &body[..body.len() - 1]).is_err());
    assert!(
        !a.sessions[0]
            .packet(124, &pickup(1, 4, 3, 17, 2, key))
            .unwrap()
            .pickup
            .unwrap()
            .accepted
    );
    assert!(
        !a.sessions[0]
            .packet(124, &pickup(0, 4, 3, 17, 2, key + 1))
            .unwrap()
            .pickup
            .unwrap()
            .accepted
    );
    assert!(
        a.sessions[0]
            .packet(124, &body)
            .unwrap()
            .pickup
            .unwrap()
            .accepted
    );
    let batch = a.sessions[0].inbox.try_recv().unwrap();
    assert!(!b.sessions[0].valid(&batch));
    // Dispatch must reject this batch through valid() before sending.
    assert!(b.sessions.iter_mut().all(|s| s.inbox.try_recv().is_err()));
    assert_eq!(
        b.shared.lock().unwrap().model.members[&b.keys[0].identity]
            .item_stats
            .unwrap()
            .speed,
        5
    );
    // Same account/slot/key bytes in the other room still refer to its own item.
    assert!(
        b.sessions[0]
            .packet(124, &body)
            .unwrap()
            .pickup
            .unwrap()
            .accepted
    );
    confirmations(&mut b);
    {
        let mut r = a.shared.lock().unwrap();
        r.model.end(a.round).unwrap();
        for member in &a.keys {
            r.model.refreshed(*member, a.round).unwrap();
        }
        for member in &a.keys[1..] {
            r.model.ready(*member).unwrap();
        }
        let next = r.model.begin(a.keys[0]).unwrap();
        for member in &a.keys {
            r.model.ack(*member, next).unwrap();
        }
        r.model.pickups = pickups::Round::begin(next, &r.floor_catalog).unwrap();
    }
    assert!(!a.sessions[0].valid(&batch));
    // The stale batch is rejected by dispatch before writing.
    assert!(
        !a.sessions[0]
            .packet(124, &body)
            .unwrap()
            .pickup
            .unwrap()
            .accepted
    );
    assert_eq!(
        b.shared.lock().unwrap().model.members[&b.keys[0].identity]
            .item_stats
            .unwrap()
            .speed,
        6
    );
}

#[test]
fn map_selection_replaces_terrain_and_claims_atomically_and_fences_prior_map_batches() {
    let mut c = case(4);
    let sibling = case(4);
    {
        let mut r = c.shared.lock().unwrap();
        r.model.end(c.round).unwrap();
        for key in &c.keys {
            r.model.refreshed(*key, c.round).unwrap();
        }
        // Native initial96 can arrive before final setup; isolate map replies
        // here from player projection fixtures tested by the existing harness.
        for member in r.model.members.values_mut() {
            member.setup = false;
        }
        r.requested.insert(c.keys[0]);
        r.maps = Some(Arc::new(crate::map_profile::Catalog {
            choices: vec![
                crate::map_profile::Choice {
                    code: 281,
                    index: 0,
                    bodies: HashMap::from([(
                        Template::Map117,
                        encoded(&[(0, Width::U16), (281, Width::U16), (0, Width::U32)]),
                    )]),
                    floor: r.floor_catalog.clone(),
                    terrain: Some(crate::terrain::tests::basic_items_map()),
                },
                crate::map_profile::Choice {
                    code: 1104,
                    index: 1,
                    bodies: HashMap::from([(
                        Template::Map117,
                        encoded(&[(0, Width::U16), (1104, Width::U16), (1, Width::U32)]),
                    )]),
                    floor: pickups::Catalog::default(),
                    terrain: Some(crate::terrain::tests::map()),
                },
                crate::map_profile::Choice {
                    code: 777,
                    index: 2,
                    bodies: HashMap::from([(
                        Template::Map117,
                        encoded(&[(0, Width::U16), (777, Width::U16), (2, Width::U32)]),
                    )]),
                    floor: pickups::Catalog::default(),
                    terrain: None,
                },
            ],
        }));
        for _ in 0..8 {
            r.channels[&c.keys[0].identity]
                .sender
                .try_send(Runtime::member_batch(c.keys[0], None, 0, vec![]))
                .unwrap_or_else(|_| panic!("fill map queue"));
        }
    }
    let choose = |code, index| encoded(&[(code, Width::U16), (index, Width::U32)]);
    assert!(c.sessions[1].packet(96, &choose(1104, 1)).is_err());
    assert!(c.sessions[0].packet(96, &choose(1104, 0)).is_err());
    assert!(c.sessions[0].packet(96, &choose(1104, 1)).is_err());
    {
        let r = c.shared.lock().unwrap();
        assert_eq!((r.selected_map, r.map_revision), (0, 0));
        assert!(!r.model.pickups.hidden_cell(2, 3));
        assert!(
            pickups::Round::begin(42, &r.floor_catalog)
                .unwrap()
                .hidden_cell(2, 3)
        );
    }
    for _ in 0..8 {
        c.sessions[0].inbox.try_recv().unwrap();
    }
    c.sessions[0].packet(96, &choose(1104, 1)).unwrap();
    let previous = c.sessions[0].inbox.try_recv().unwrap();
    assert!(c.sessions[0].valid(&previous));
    {
        let r = c.shared.lock().unwrap();
        assert_eq!((r.selected_map, r.map_revision), (1, 1));
        assert!(!r.model.pickups.hidden_cell(2, 3));
        let mut terrain = r.model.terrain.clone().unwrap();
        terrain.begin(42);
        assert!(
            terrain
                .place(crate::terrain::Bomb {
                    id: 1,
                    owner: 0,
                    row: 3,
                    column: 5,
                    power: 1
                })
                .is_err()
        );
        let empty = pickups::Round::begin(42, &r.floor_catalog).unwrap();
        assert!(!empty.hidden_cell(2, 3));
    }
    c.sessions[0].packet(96, &choose(777, 2)).unwrap();
    assert!(!c.sessions[0].valid(&previous));
    assert!(c.shared.lock().unwrap().model.terrain.is_none());
    c.sessions[0].inbox.try_recv().unwrap();
    c.sessions[0].packet(96, &choose(281, 0)).unwrap();
    let r = c.shared.lock().unwrap();
    assert_eq!((r.selected_map, r.map_revision), (0, 3));
    let mut terrain = r.model.terrain.clone().unwrap();
    terrain.begin(43);
    assert!(
        terrain
            .place(crate::terrain::Bomb {
                id: 1,
                owner: 7,
                row: 3,
                column: 5,
                power: 1
            })
            .is_ok()
    );
    assert!(
        pickups::Round::begin(43, &r.floor_catalog)
            .unwrap()
            .hidden_cell(2, 3)
    );
    assert_eq!(
        sibling.shared.lock().unwrap().model.phase,
        Phase::Armed {
            round: sibling.round
        }
    );
    assert!(
        sibling
            .shared
            .lock()
            .unwrap()
            .model
            .pickups
            .hidden_cell(2, 3)
    );
}

#[test]
fn captured59_is_current_no_reply_visit_budget_and_cannot_change_round_or_stats() {
    let body = bytes(
        "5ba56c96ca53bf96ca768d96f852954cb25395560a9cee94ca9905ce0865fe37d2abfaa6c81a39dae977039e0953835b6c",
    );
    let mut c = case(3);
    assert!(Session::handles(59));
    c.sessions[0].connection += 1;
    assert!(c.sessions[0].packet(59, &body).is_err());
    c.sessions[0].connection -= 1;
    assert!(c.sessions[0].packet(59, &body[..body.len() - 1]).is_err());
    for _ in 0..15 {
        c.sessions[0].packet(59, &body).unwrap();
    }
    {
        let mut r = c.shared.lock().unwrap();
        assert_eq!(r.model.phase, Phase::Armed { round: c.round });
        assert_eq!(
            r.model.members[&c.keys[0].identity]
                .item_stats
                .unwrap()
                .speed,
            5
        );
        assert!(r.model.pickups.hidden_cell(2, 3));
        assert!(r.model.last_blast.is_none());
        r.model.end(c.round).unwrap();
        for key in &c.keys {
            r.model.refreshed(*key, c.round).unwrap();
        }
    }
    // Idle after reset is still the same visit; the budget cannot be renewed.
    c.sessions[0].packet(59, &body).unwrap();
    assert!(c.sessions[0].packet(59, &body).is_err());
    assert!(c.sessions.iter_mut().all(|s| s.inbox.try_recv().is_err()));
    assert_eq!(c.shared.lock().unwrap().model.phase, Phase::Idle);
}
