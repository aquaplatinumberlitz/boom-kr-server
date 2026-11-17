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
fn spawn_choices(c: &mut Case, counts: &[usize]) {
    let mut r = c.shared.lock().unwrap();
    let choices = counts
        .iter()
        .enumerate()
        .map(|(index, &spawn_count)| crate::map_profile::Choice {
            code: 800 + index as u16,
            index: index as u32,
            spawn_count,
            bodies: HashMap::from([(
                Template::Map117,
                encoded(&[
                    (0, Width::U16),
                    (800 + index as u64, Width::U16),
                    (index as u64, Width::U32),
                ]),
            )]),
            floor: pickups::Catalog::default(),
            terrain: None,
            bomb_cancellation: None,
        })
        .collect();
    r.maps = Some(Arc::new(crate::map_profile::Catalog { choices }));
}
#[test]
fn selected_map_bounds_admission_and_sparse_spawns_without_reassigning_member_slots() {
    let mut c = case(8);
    spawn_choices(&mut c, &[8, 6]);
    {
        let mut r = c.shared.lock().unwrap();
        r.model.end(c.round).unwrap();
        for key in &c.keys {
            r.model.refreshed(*key, c.round).unwrap();
        }
    }
    let select = encoded(&[(801, Width::U16), (1, Width::U32)]);
    assert_eq!(
        c.sessions[0].packet(96, &select).unwrap().map_rejection,
        Some("roster_exceeds_map_spawns")
    );
    {
        let mut r = c.shared.lock().unwrap();
        assert_eq!(r.selected_map, 0);
        assert_eq!(r.map_revision, 0);
        assert_eq!(r.admission_capacity(), 8);
        assert_eq!(r.player_spawns(&r.model).unwrap()[&7], 7);
        // Keep high membership slots in a six-player roster. Compaction affects
        // only the explicit per-round spawn ordinals, never identities/slots.
        for i in [1, 3] {
            r.model.leave(c.keys[i]).unwrap();
        }
        for member in r.model.members.values_mut() {
            member.setup = false;
        }
        r.requested.clear();
    }
    c.sessions[0].packet(96, &select).unwrap();
    {
        let r = c.shared.lock().unwrap();
        assert_eq!(r.selected_map, 1);
        assert_eq!(r.admission_capacity(), 6);
        assert_eq!(r.model.capacity(), 8);
        let assigned = r.player_spawns(&r.model).unwrap();
        for (ordinal, slot) in [0, 2, 4, 5, 6, 7].into_iter().enumerate() {
            assert_eq!(assigned[&slot], ordinal as u8);
            let member = &r.model.members[&c.keys[usize::from(slot)].identity];
            assert_eq!(member.slot, slot);
            assert_eq!(member.key, c.keys[usize::from(slot)]);
        }
    }
    assert!(full_idle(&c.shared).unwrap());
    let back = encoded(&[(800, Width::U16), (0, Width::U32)]);
    c.sessions[0].packet(96, &back).unwrap();
    assert!(!full_idle(&c.shared).unwrap());
    {
        let r = c.shared.lock().unwrap();
        assert_eq!(r.admission_capacity(), 8);
        assert_eq!(r.player_spawns(&r.model).unwrap()[&7], 7);
    }
    assert!(c.sessions.iter_mut().all(|s| s.inbox.try_recv().is_err()));
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
    reveal_aux(slot, row, column, u32::MAX)
}
fn reveal_aux(slot: u8, row: u8, column: u8, aux: u32) -> Vec<u8> {
    encoded(&[
        (row.into(), Width::U8),
        (column.into(), Width::U8),
        (slot.into(), Width::U8),
        (aux.into(), Width::U32),
    ])
}
#[test]
fn opaque_hidden_report_requires_configured_current_cell_and_all_blast_completions() {
    // Actual Village10 terrain6-r1, channel5/sequence259: row0,col8,
    // selected slot4, aux0. The body is from the retained PCAP decoder,
    // not produced by this implementation. Aux semantics are still unknown.
    let actual = bytes("5a525e96ca5395");
    let mut captured = case(6);
    assert_eq!(
        captured.sessions[4]
            .packet(142, &actual)
            .unwrap()
            .terrain_notice,
        Some("unknown_hidden_cell")
    );
    assert!(
        captured
            .sessions
            .iter_mut()
            .all(|s| s.inbox.try_recv().is_err())
    );
    for aux in [0, 3, 5, 37, 0x12345678, u32::MAX] {
        let mut c = case(6);
        let key = crate::room::wire_key(c.round).unwrap();
        let report = reveal_aux(4, 2, 3, aux);
        assert!(c.sessions[4].packet(142, &report[..6]).is_err());
        assert!(c.sessions[3].packet(142, &report).is_err());
        c.sessions[4].connection += 1;
        assert!(c.sessions[4].packet(142, &report).is_err());
        c.sessions[4].connection -= 1;
        assert_eq!(
            c.sessions[4].packet(142, &report).unwrap().terrain_notice,
            Some("no_current_committed_clearance")
        );
        assert!(
            c.sessions[4]
                .packet(118, &bomb(4, 1, 1, key))
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
        for (s, batch) in c.sessions.iter().zip(&blasts).take(5) {
            s.finish(batch).unwrap();
        }
        assert_eq!(
            c.sessions[4].packet(142, &report).unwrap().terrain_notice,
            Some("no_current_committed_clearance")
        );
        c.sessions[5].finish(&blasts[5]).unwrap();
        assert_eq!(
            c.sessions[4]
                .packet(142, &reveal_aux(4, 0, 0, aux))
                .unwrap()
                .terrain_notice,
            Some("unknown_hidden_cell")
        );
        assert_eq!(
            c.sessions[4].packet(142, &report).unwrap().terrain_notice,
            Some("revealed_current_hidden_cell")
        );
        assert_eq!(
            c.sessions[4].packet(142, &report).unwrap().terrain_notice,
            Some("already_revealed")
        );
        assert!(c.sessions.iter_mut().all(|s| s.inbox.try_recv().is_err()));
        let claim = pickup(4, 2, 3, 27, 0, key);
        assert!(
            c.sessions[4]
                .packet(124, &claim)
                .unwrap()
                .pickup
                .unwrap()
                .accepted
        );
        confirmations(&mut c);
        assert!(
            !c.sessions[4]
                .packet(124, &claim)
                .unwrap()
                .pickup
                .unwrap()
                .accepted
        );
        {
            let mut r = c.shared.lock().unwrap();
            r.model.end(c.round).unwrap();
        }
        assert_eq!(
            c.sessions[4].packet(142, &report).unwrap().terrain_notice,
            Some("inactive_round")
        );
        {
            let mut r = c.shared.lock().unwrap();
            for member in &c.keys {
                r.model.refreshed(*member, c.round).unwrap();
            }
            for member in &c.keys[1..] {
                r.model.ready(*member).unwrap();
            }
            let next = r.model.begin(c.keys[0]).unwrap();
            r.model.pickups = pickups::Round::begin(next, &r.floor_catalog).unwrap();
            for member in &c.keys {
                r.model.ack(*member, next).unwrap();
            }
        }
        // Identical retained body cannot reuse the preceding round's clearance.
        assert_eq!(
            c.sessions[4].packet(142, &report).unwrap().terrain_notice,
            Some("no_current_committed_clearance")
        );
        assert!(
            !c.sessions[4]
                .packet(124, &claim)
                .unwrap()
                .pickup
                .unwrap()
                .accepted
        );
    }
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
fn maxpower_claim_is_atomic_current_member_owned_and_gates_actual_power_until_reset() {
    // CP045 floor582-kind5.bin, SHA256 9a2e4a18fc34e082357956c960c4b4635
    // 91a95592b9ac8839c68803103e21b34, decoded body at sequence73.
    let catalog = pickups::Catalog::from_floor(&bytes(
        "96ca538396ca428d96cb539596ca538496cf509196ca5395a569",
    ))
    .unwrap();
    let characters = crate::characters::Catalog::measured(&[0, 1]).unwrap();
    // Identity/slot/capacity do not select either the character or the effect.
    for character in [0, 1] {
        let mut c = case(3);
        let slot = 2;
        let seed = characters.get(character).unwrap();
        {
            let mut r = c.shared.lock().unwrap();
            r.floor_catalog = catalog.clone();
            r.model.pickups = pickups::Round::begin(c.round, &catalog).unwrap();
            r.model
                .members
                .get_mut(&c.keys[slot].identity)
                .unwrap()
                .item_stats = Some(seed);
        }
        let key = crate::room::wire_key(c.round).unwrap();
        let claim = pickup(slot as u8, 4, 3, 17, 5, key);
        c.sessions[slot].connection += 1;
        assert!(c.sessions[slot].packet(124, &claim).is_err());
        c.sessions[slot].connection -= 1;
        assert!(
            !c.sessions[slot]
                .packet(124, &pickup(slot as u8, 4, 3, 17, 5, key.wrapping_sub(1)))
                .unwrap()
                .pickup
                .unwrap()
                .accepted
        );
        {
            c.shared
                .lock()
                .unwrap()
                .model
                .members
                .get_mut(&c.keys[slot].identity)
                .unwrap()
                .life
                .death = Some(Instant::now());
        }
        assert!(
            !c.sessions[slot]
                .packet(124, &claim)
                .unwrap()
                .pickup
                .unwrap()
                .accepted
        );
        c.shared
            .lock()
            .unwrap()
            .model
            .members
            .get_mut(&c.keys[slot].identity)
            .unwrap()
            .life
            .begin();
        // A full peer queue must not consume the item or publish the new power.
        {
            let r = c.shared.lock().unwrap();
            for _ in 0..8 {
                r.channels[&c.keys[0].identity]
                    .sender
                    .try_send(Runtime::member_batch(c.keys[0], Some(c.round), 0, vec![]))
                    .unwrap_or_else(|_| panic!("fill bounded queue"));
            }
        }
        assert!(c.sessions[slot].packet(124, &claim).is_err());
        assert_eq!(
            c.shared.lock().unwrap().model.members[&c.keys[slot].identity].item_stats,
            Some(seed)
        );
        for _ in 0..8 {
            c.sessions[0].inbox.try_recv().unwrap();
        }
        assert!(
            c.sessions[slot]
                .packet(124, &claim)
                .unwrap()
                .pickup
                .unwrap()
                .accepted
        );
        confirmations(&mut c);
        {
            let r = c.shared.lock().unwrap();
            let mut capped = seed;
            capped.power = seed.maximum_power;
            assert_eq!(
                r.model.members[&c.keys[slot].identity].item_stats,
                Some(capped)
            );
            assert_eq!(
                r.model.members[&c.keys[0].identity].item_stats,
                Some(characters.get(0).unwrap())
            );
        }
        assert!(
            !c.sessions[slot]
                .packet(124, &claim)
                .unwrap()
                .pickup
                .unwrap()
                .accepted
        );
        assert!(c.sessions.iter_mut().all(|s| s.inbox.try_recv().is_err()));
        assert!(
            !c.sessions[slot]
                .packet(118, &bomb(slot as u8, 1, 1, key))
                .unwrap()
                .bomb
                .unwrap()
                .accepted
        );
        assert!(
            c.sessions[slot]
                .packet(118, &bomb(slot as u8, 1, seed.maximum_power, key))
                .unwrap()
                .bomb
                .unwrap()
                .accepted
        );
        for s in &mut c.sessions {
            s.inbox.try_recv().unwrap();
            s.inbox.try_recv().unwrap();
        }
        let next = {
            let mut r = c.shared.lock().unwrap();
            r.model.end(c.round).unwrap();
            for key in &c.keys {
                r.model.refreshed(*key, c.round).unwrap();
            }
            for key in &c.keys[1..] {
                r.model.ready(*key).unwrap();
            }
            let next = r.model.begin(c.keys[0]).unwrap();
            assert!(r.model.members.values().all(|m| m.item_stats.is_none()));
            r.model.pickups = pickups::Round::begin(next, &catalog).unwrap();
            next
        };
        assert_ne!(crate::room::wire_key(next).unwrap(), key);
        assert!(
            !c.sessions[slot]
                .packet(124, &claim)
                .unwrap()
                .pickup
                .unwrap()
                .accepted
        );
        assert!(c.sessions.iter_mut().all(|s| s.inbox.try_recv().is_err()));
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
                    spawn_count: 8,
                    code: 281,
                    index: 0,
                    bodies: HashMap::from([(
                        Template::Map117,
                        encoded(&[(0, Width::U16), (281, Width::U16), (0, Width::U32)]),
                    )]),
                    floor: r.floor_catalog.clone(),
                    terrain: Some(crate::terrain::tests::basic_items_map()),
                    bomb_cancellation: None,
                },
                crate::map_profile::Choice {
                    spawn_count: 8,
                    code: 1104,
                    index: 1,
                    bodies: HashMap::from([(
                        Template::Map117,
                        encoded(&[(0, Width::U16), (1104, Width::U16), (1, Width::U32)]),
                    )]),
                    floor: pickups::Catalog::default(),
                    terrain: Some(crate::terrain::tests::map()),
                    bomb_cancellation: None,
                },
                crate::map_profile::Choice {
                    spawn_count: 8,
                    code: 777,
                    index: 2,
                    bodies: HashMap::from([(
                        Template::Map117,
                        encoded(&[(0, Width::U16), (777, Width::U16), (2, Width::U32)]),
                    )]),
                    floor: pickups::Catalog::default(),
                    terrain: None,
                    bomb_cancellation: None,
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

// Native CP307 report bytes. These synthetic scenarios check server authority;
// they do not establish native acceptance of the new policy.
fn cancellation_case() -> Case {
    let mut c = case(2);
    spawn_choices(&mut c, &[8, 8]);
    let mut r = c.shared.lock().unwrap();
    let mut choices = Arc::try_unwrap(r.maps.take().unwrap()).ok().unwrap();
    let cfg: rules::BombCancellationConfig = serde_json::from_value(serde_json::json!({
        "map_sha256": "a".repeat(64), "cells": [[6, 2]]
    }))
    .unwrap();
    choices.choices[0].bomb_cancellation = Some(cfg.validate(&"a".repeat(64)).unwrap());
    r.maps = Some(Arc::new(choices));
    r.rules.map_bomb_cancellation = true;
    r.model.terrain = None;
    drop(r);
    c
}
fn cancelled_bomb(serial: u32, key: u32) -> Vec<u8> {
    let mut body = bomb(0, serial, 1, key);
    body[10..12].copy_from_slice(&encoded(&[(6, Width::U8), (2, Width::U8)]));
    body
}
fn report122() -> Vec<u8> {
    bytes("6935ac6aa56fa56b96ca5391a596ca5395")
}
#[test]
fn cancellation_consumes_serial_without_spawn_deadline_or_quota_and_reports_cannot_release_normal_bomb()
 {
    let mut c = cancellation_case();
    let old_key = crate::room::wire_key(c.round).unwrap();
    let report = report122();
    let cancelled = cancelled_bomb(1, old_key);
    let reason = |o: Observation| o.bomb.unwrap().reason;
    assert_eq!(
        reason(c.sessions[1].packet(118, &cancelled).unwrap()),
        "owner_mismatch"
    );
    assert_eq!(
        reason(
            c.sessions[0]
                .packet(118, &cancelled_bomb(1, old_key + 1))
                .unwrap()
        ),
        "round_key_mismatch"
    );
    assert_eq!(
        reason(c.sessions[0].packet(118, &cancelled).unwrap()),
        "map_cell_cancelled"
    );
    assert!(!c.shared.lock().unwrap().model.bombs.pending());
    assert!(c.sessions.iter_mut().all(|s| s.inbox.try_recv().is_err()));
    assert_eq!(
        reason(c.sessions[0].packet(118, &cancelled).unwrap()),
        "duplicate_serial"
    );
    assert_eq!(
        reason(c.sessions[0].packet(118, &bomb(0, 1, 1, old_key)).unwrap()),
        "conflicting_serial"
    );
    let stats = c.shared.lock().unwrap().model.members[&c.keys[0].identity].item_stats;
    for cycle in 0..2 {
        let key = crate::room::wire_key(c.round).unwrap();
        if cycle != 0 {
            assert_eq!(
                reason(c.sessions[0].packet(118, &cancelled).unwrap()),
                "round_key_mismatch"
            );
        }
        let accepted = c.sessions[0]
            .packet(118, &bomb(0, 2, 1, key))
            .unwrap()
            .bomb
            .unwrap();
        assert!(accepted.accepted);
        if cycle == 0 {
            assert_eq!(accepted.id, Some(1));
        }
        c.sessions[0].connection += 1;
        assert!(c.sessions[0].packet(122, &report).is_err());
        c.sessions[0].connection -= 1;
        assert!(c.sessions[0].packet(122, &report[..16]).is_err());
        for _ in 0..64 {
            c.sessions[0].packet(122, &report).unwrap();
        }
        assert!(c.sessions[0].packet(122, &report).is_err());
        assert!(c.shared.lock().unwrap().model.bombs.pending());
        assert_eq!(
            reason(c.sessions[0].packet(118, &bomb(0, 3, 1, key)).unwrap()),
            "owner_capacity_reserved"
        );
        for s in &mut c.sessions {
            let spawn = s.inbox.try_recv().unwrap();
            assert_eq!(spawn.packets[0].1.opcode, 159);
            let deadline = s.inbox.try_recv().unwrap();
            assert_eq!(deadline.packets[0].1.opcode, 158);
            assert!(s.valid(&deadline));
            s.finish(&deadline).unwrap();
            assert!(s.inbox.try_recv().is_err());
        }
        assert!(!c.shared.lock().unwrap().model.bombs.pending());
        {
            let mut r = c.shared.lock().unwrap();
            r.model.end(c.round).unwrap();
        }
        // A tail can be observed during Ending, but never emits a reply.
        c.sessions[1].packet(122, &report).unwrap();
        {
            let mut r = c.shared.lock().unwrap();
            for key in &c.keys {
                r.model.refreshed(*key, c.round).unwrap();
            }
        }
        c.sessions[0].packet(122, &report).unwrap();
        assert!(!c.shared.lock().unwrap().model.bombs.pending());
        if cycle == 0 {
            let mut r = c.shared.lock().unwrap();
            r.model.ready(c.keys[1]).unwrap();
            c.round = r.model.begin(c.keys[0]).unwrap();
            for key in &c.keys {
                r.model.ack(*key, c.round).unwrap();
            }
            for member in r.model.members.values_mut() {
                member.item_stats = stats;
            }
        }
    }
}
#[test]
fn cancellation_and_report_policy_follow_selected_map_and_explicit_enable() {
    let mut c = cancellation_case();
    let report = report122();
    {
        let mut r = c.shared.lock().unwrap();
        r.selected_map = 1;
    }
    assert!(c.sessions[0].packet(122, &report).is_err());
    let key = crate::room::wire_key(c.round).unwrap();
    assert!(
        c.sessions[0]
            .packet(118, &cancelled_bomb(1, key))
            .unwrap()
            .bomb
            .unwrap()
            .accepted
    );
    for s in &mut c.sessions {
        s.inbox.try_recv().unwrap();
        s.inbox.try_recv().unwrap();
    }
    {
        let mut r = c.shared.lock().unwrap();
        r.selected_map = 0;
        r.rules.map_bomb_cancellation = false;
    }
    assert!(c.sessions[0].packet(122, &report).is_err());
    c.shared.lock().unwrap().rules.map_bomb_cancellation = true;
    c.sessions[0].packet(122, &report).unwrap();
    assert!(c.sessions.iter_mut().all(|s| s.inbox.try_recv().is_err()));
}

#[test]
fn durable_box_claim_waits_for_fifth_full_delivery_and_reset_restores_hit_count() {
    let mut c = case(2);
    c.shared.lock().unwrap().model.terrain = Some(crate::terrain::tests::durable_items_map());
    {
        let mut r = c.shared.lock().unwrap();
        r.model.terrain.as_mut().unwrap().begin(c.round);
    }
    let key = crate::room::wire_key(c.round).unwrap();
    for serial in 1..=5 {
        assert!(
            c.sessions[0]
                .packet(118, &bomb(0, serial, 1, key))
                .unwrap()
                .bomb
                .unwrap()
                .accepted
        );
        let mut deadlines = Vec::new();
        for s in &mut c.sessions {
            s.inbox.try_recv().unwrap();
            deadlines.push(s.inbox.try_recv().unwrap());
        }
        c.sessions[0].finish(&deadlines[0]).unwrap();
        assert_eq!(
            c.sessions[0]
                .packet(142, &reveal(0, 2, 3))
                .unwrap()
                .terrain_notice,
            Some("no_current_committed_clearance")
        );
        c.sessions[1].finish(&deadlines[1]).unwrap();
        let expected = if serial == 5 {
            "revealed_current_hidden_cell"
        } else {
            "no_current_committed_clearance"
        };
        assert_eq!(
            c.sessions[0]
                .packet(142, &reveal(0, 2, 3))
                .unwrap()
                .terrain_notice,
            Some(expected)
        );
        let claim = c.sessions[0]
            .packet(124, &pickup(0, 2, 3, 27, 0, key))
            .unwrap()
            .pickup
            .unwrap();
        assert_eq!(claim.accepted, serial == 5);
    }
    confirmations(&mut c);
    {
        let mut r = c.shared.lock().unwrap();
        r.model.end(c.round).unwrap();
        for key in &c.keys {
            r.model.refreshed(*key, c.round).unwrap();
        }
        r.model.ready(c.keys[1]).unwrap();
        c.round = r.model.begin(c.keys[0]).unwrap();
        for key in &c.keys {
            r.model.ack(*key, c.round).unwrap();
        }
        assert!(
            r.model
                .terrain
                .as_ref()
                .unwrap()
                .clearance(c.round, 2, 3)
                .is_none()
        );
        assert!(
            r.model
                .terrain
                .as_ref()
                .unwrap()
                .clearance(c.round - 1, 2, 3)
                .is_none()
        );
    }
    assert!(
        !c.sessions[0]
            .packet(124, &pickup(0, 2, 3, 27, 0, key))
            .unwrap()
            .pickup
            .unwrap()
            .accepted
    );
}

// Candidate Punch slice integration; these are server invariants, not a native oracle.
fn punch_case() -> Case {
    let c = case(2);
    {
        let mut r = c.shared.lock().unwrap();
        r.rules.item_policy = pickups::Policy::ReportDerivedPunch;
        let catalog = pickups::tests::punch_catalog();
        r.model.pickups = pickups::Round::begin(c.round, &catalog).unwrap();
        r.floor_catalog = catalog;
        let mut terrain = crate::terrain::tests::motion_map();
        terrain.begin(c.round);
        r.model.terrain = Some(terrain);
    }
    c
}
fn punch_body(slot: u8, id: u16, source_row: u8) -> Vec<u8> {
    let mut body = wire::BombMotion162 {
        context: 0,
        slot,
        selector: 3,
        bomb_id: id,
        source_row,
        source_column: 3,
        target_row: source_row - 1,
        target_column: 3,
    }
    .encode()
    .unwrap();
    body.extend(encoded(&[(1715, Width::U16), (1, Width::U16)]));
    body
}
fn punch_pickup(c: &mut Case) {
    let key = crate::room::wire_key(c.round).unwrap();
    assert!(
        c.sessions[0]
            .packet(124, &pickup(0, 5, 3, 18, 4, key))
            .unwrap()
            .pickup
            .unwrap()
            .accepted
    );
    confirmations(c);
}
fn punch_place(c: &mut Case, row: u8) -> (u16, Vec<Batch>) {
    let key = crate::room::wire_key(c.round).unwrap();
    let mut request = bomb(0, 1, 1, key);
    request[10] = write_uint(u64::from(row), Width::U8).unwrap()[0];
    let id = c.sessions[0]
        .packet(118, &request)
        .unwrap()
        .bomb
        .unwrap()
        .id
        .unwrap();
    let mut deadlines = Vec::new();
    for session in &mut c.sessions {
        assert_eq!(session.inbox.try_recv().unwrap().packets[0].1.opcode, 159);
        deadlines.push(session.inbox.try_recv().unwrap());
    }
    (id, deadlines)
}
#[test]
fn punch_keeps_deadline_quota_and_owner_then_commits_terrain_at_moved_root() {
    let mut c = punch_case();
    punch_pickup(&mut c);
    let (id, mut deadlines) = punch_place(&mut c, 6);
    let due = deadlines.iter().map(|b| b.due).collect::<Vec<_>>();
    let request = punch_body(0, id, 6);
    assert!(
        c.sessions[0]
            .packet(119, &request)
            .unwrap()
            .bomb
            .unwrap()
            .accepted
    );
    for session in &mut c.sessions {
        let motion = session.inbox.try_recv().unwrap();
        assert!(session.valid(&motion));
        assert_eq!(motion.packets[0].1.opcode, 162);
        assert_eq!(motion.packets[0].1.payload, request[..9]);
    }
    assert_eq!(
        c.sessions[0]
            .packet(119, &request)
            .unwrap()
            .bomb
            .unwrap()
            .reason,
        "bomb_already_moved"
    );
    let key = crate::room::wire_key(c.round).unwrap();
    assert_eq!(
        c.sessions[0]
            .packet(118, &bomb(0, 2, 1, key))
            .unwrap()
            .bomb
            .unwrap()
            .reason,
        "owner_capacity_reserved"
    );
    for (i, session) in c.sessions.iter().enumerate() {
        assert!(session.prepare_delivery(&mut deadlines[i]).unwrap());
        assert_eq!(deadlines[i].due, due[i]);
        assert_eq!(
            deadlines[i]
                .packets
                .iter()
                .map(|p| p.1.payload.len())
                .collect::<Vec<_>>(),
            [4, 13]
        );
        assert_eq!(
            deadlines[i].packets[0].1.payload,
            wire::rebind158(id).unwrap()
        );
        session.finish(&deadlines[i]).unwrap();
        let r = c.shared.lock().unwrap();
        assert_eq!(
            r.model
                .terrain
                .as_ref()
                .unwrap()
                .clearance(c.round, 5, 4)
                .is_some(),
            i == 1
        );
    }
    assert!(
        c.sessions[0]
            .packet(118, &bomb(0, 2, 1, key))
            .unwrap()
            .bomb
            .unwrap()
            .accepted
    );
}
#[test]
fn punch_missing_item_transport_slot_foreign_owner_and_old_round_are_fenced() {
    let mut c = punch_case();
    let (id, _) = punch_place(&mut c, 6);
    let request = punch_body(0, id, 6);
    assert_eq!(
        c.sessions[0]
            .packet(119, &request)
            .unwrap()
            .bomb
            .unwrap()
            .reason,
        "missing_punch"
    );
    punch_pickup(&mut c);
    c.sessions[0].connection += 1;
    assert!(c.sessions[0].packet(119, &request).is_err());
    c.sessions[0].connection -= 1;
    assert_eq!(
        c.sessions[0]
            .packet(119, &punch_body(1, id, 6))
            .unwrap()
            .bomb
            .unwrap()
            .reason,
        "action_actor_mismatch"
    );
    c.shared
        .lock()
        .unwrap()
        .model
        .members
        .get_mut(&c.keys[1].identity)
        .unwrap()
        .punch = true;
    assert_eq!(
        c.sessions[1]
            .packet(119, &punch_body(1, id, 6))
            .unwrap()
            .bomb
            .unwrap()
            .reason,
        "foreign_bomb"
    );
    c.shared
        .lock()
        .unwrap()
        .model
        .members
        .get_mut(&c.keys[0].identity)
        .unwrap()
        .life
        .death = Some(Instant::now());
    assert_eq!(
        c.sessions[0]
            .packet(119, &request)
            .unwrap()
            .bomb
            .unwrap()
            .reason,
        "report_ineligible"
    );
    {
        let mut r = c.shared.lock().unwrap();
        r.model.end(c.round).unwrap();
        for key in &c.keys {
            r.model.refreshed(*key, c.round).unwrap();
        }
        for key in &c.keys[1..] {
            r.model.ready(*key).unwrap();
        }
        let round = r.model.begin(c.keys[0]).unwrap();
        assert!(r.model.members.values().all(|m| !m.punch));
        for key in &c.keys {
            r.model.ack(*key, round).unwrap();
        }
        r.model.members.get_mut(&c.keys[0].identity).unwrap().punch = true;
    }
    assert_eq!(
        c.sessions[0]
            .packet(119, &request)
            .unwrap()
            .bomb
            .unwrap()
            .reason,
        "inactive_bomb"
    );
}
#[test]
fn punch_queue_and_terrain_failures_leave_retryable_original_ledger_and_root() {
    let mut c = punch_case();
    punch_pickup(&mut c);
    let (id, _) = punch_place(&mut c, 6);
    let request = punch_body(0, id, 6);
    {
        let r = c.shared.lock().unwrap();
        for _ in 0..8 {
            r.channels[&c.keys[1].identity]
                .sender
                .try_send(Runtime::member_batch(c.keys[1], Some(c.round), 0, vec![]))
                .unwrap_or_else(|_| panic!("fill peer queue"));
        }
    }
    assert!(c.sessions[0].packet(119, &request).is_err());
    assert!(c.sessions[0].inbox.try_recv().is_err());
    for _ in 0..8 {
        c.sessions[1].inbox.try_recv().unwrap();
    }
    assert!(
        c.sessions[0]
            .packet(119, &request)
            .unwrap()
            .bomb
            .unwrap()
            .accepted
    );
    let mut blocked = punch_case();
    punch_pickup(&mut blocked);
    let (id, _) = punch_place(&mut blocked, 3); // row2/col3 is a box.
    assert_eq!(
        blocked.sessions[0]
            .packet(119, &punch_body(0, id, 3))
            .unwrap()
            .bomb
            .unwrap()
            .reason,
        "motion_terrain"
    );
    assert!(
        blocked
            .sessions
            .iter_mut()
            .all(|s| s.inbox.try_recv().is_err())
    );
    // The rejected candidate never marks the original ledger as already moved.
    assert_eq!(
        blocked.sessions[0]
            .packet(119, &punch_body(0, id, 3))
            .unwrap()
            .bomb
            .unwrap()
            .reason,
        "motion_terrain"
    );
}

#[test]
fn queued_punch_cannot_survive_actor_departure_or_a_new_membership_visit() {
    let mut c = punch_case();
    punch_pickup(&mut c);
    let (id, _) = punch_place(&mut c, 6);
    assert!(
        c.sessions[0]
            .packet(119, &punch_body(0, id, 6))
            .unwrap()
            .bomb
            .unwrap()
            .accepted
    );
    let old_motion = c.sessions[1].inbox.try_recv().unwrap();
    assert!(c.sessions[1].valid(&old_motion));
    let old = c.keys[0];
    let replacement = {
        let mut r = c.shared.lock().unwrap();
        r.model.leave(old).unwrap();
        let replacement = r
            .model
            .enter(old.identity, old.connection + 1000, false)
            .unwrap();
        assert_ne!(replacement.visit, old.visit);
        assert!(!r.model.members[&old.identity].punch);
        replacement
    };
    assert!(!c.sessions[1].valid(&old_motion));
    assert!(!c.shared.lock().unwrap().model.current(old));
    assert!(c.shared.lock().unwrap().model.current(replacement));
    assert!(c.sessions[0].packet(119, &punch_body(0, id, 6)).is_err());
}
