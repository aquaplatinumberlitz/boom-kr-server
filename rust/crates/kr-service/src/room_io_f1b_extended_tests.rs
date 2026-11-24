//! Room packet integration with p93 captures and a synthetic floor/roster.
use super::*;
use pickups::Policy;

// p93 run-2 wire.jsonl, verified C2S 124 bodies and their S2C 165 confirms.
// Only the input round key is rebound to this test room's current round.
const FOREST: [(u16, u16, &str, &str); 10] = [
    (
        67,
        1,
        "5a5a5a5796ca5395a52a96ca5ae096ca62ac5a",
        "5b5a5a5a5796ca5395a52a5a",
    ),
    (
        68,
        1,
        "5a5a5a5696ca5394a52d96ca5ae896ca62ac5a",
        "5b5a5a5a5696ca5394a52d5a",
    ),
    (
        69,
        1,
        "5a5a5a5196ca5397a52c96ca5f9c96ca62ac5a",
        "5b5a5a5a5196ca5397a52c5a",
    ),
    (
        70,
        1,
        "5a5a5a5096ca5396a52f96ca5f8496ca62ac5a",
        "5b5a5a5a5096ca5396a52f5a",
    ),
    (
        71,
        1,
        "5a5a5a5396ca5391a52e96ca5f8c96ca62ac5a",
        "5b5a5a5a5396ca5391a52e5a",
    ),
    (
        72,
        1,
        "5a5a5a5296ca5390a52196ca5fb496ca62ac5a",
        "5b5a5a5a5296ca5390a5215a",
    ),
    (
        77,
        1,
        "5a5a5a5c96ca5393a52496ca5fa496ca62ac5a",
        "5b5a5a5a5c96ca5393a5245a",
    ),
    (
        143,
        3,
        "5a5a5a5f96ca5392a5e696ca5fac96ca62ac5a",
        "5b5a5a5a5f96ca5392a5e65a",
    ),
    (
        224,
        3,
        "5a5b5c5796ca539da58996ca467e96ca62ac5a",
        "5b5a5b5c5796ca539da5895a",
    ),
    (
        225,
        2,
        "5a5b5c5196ca539ca58896ca45b196ca62ac5a",
        "5b5a5b5c5196ca539ca5885a",
    ),
];
const SCANNER: u16 = 146;
const SCANNER_FLOOR: [(u16, u16, &str, &str); 2] = [
    (
        SCANNER,
        1,
        "5a5a525a96ca5395a5fb96ca4c0396ca61ac5a",
        "5b5a5a525a96ca5395a5fb5a",
    ),
    (
        SCANNER,
        1,
        "5a5b505b96ca5394a5fb96ca4c2296ca61ac5a",
        "5b5a5b505b96ca5394a5fb5a",
    ),
];
const SCAN_A: &str = "5a5aa569a40e96ca4c3096ca53975ba5fb5a96ca5395";
const SCAN_B: &str = "5a5ba55da4de96ca4c4496ca53975ba5fb5a96ca5395";
const TRAP_A: &str = "4b5aa511a46496ca59fb96ca539096ca539b96ca5a8d96c8539596ca5395a569";

fn bytes(hex: &str) -> Vec<u8> {
    hex.as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
        .collect()
}

fn floor(rows: &[(u16, u16, &str, &str)]) -> Vec<u8> {
    let mut raw = vec![24];
    raw.extend((7 + 10 * rows.len() as u32).to_le_bytes());
    raw.push(0);
    raw.extend((rows.len() as u16).to_le_bytes());
    for (kind, _, claim, _) in rows {
        let body = bytes(claim);
        let index = read_uint(&body[4..8], Width::U32).unwrap() as u32;
        raw.extend(index.to_le_bytes());
        raw.extend([body[2] ^ 0x5a, body[3] ^ 0x5a]);
        raw.extend(u32::from(*kind).to_le_bytes());
    }
    raw.extend([0; 4]);
    let mut payload = kr_wire::write_uint(raw.len() as u64, Width::U32).unwrap();
    payload.extend(wire::encode_bytes(&raw).unwrap());
    payload
}

struct Case {
    shared: Shared,
    players: [Session; 2],
    keys: [Membership; 2],
}

impl Case {
    fn new(policy: Policy, floor_body: Vec<u8>) -> Self {
        let shared = registry_foundation_tests::player_round_prototype();
        let players = [70000, 42].map(|id| {
            Session::open_with_peer(
                &shared,
                id,
                u64::from(id),
                Some(std::net::Ipv4Addr::LOCALHOST),
            )
            .unwrap()
        });
        let mut r = shared.lock().unwrap();
        r.rules = RoomRules {
            item_policy: policy,
            life_actions: crate::life_actions::Policy::NeedleRescue,
            life: rules::LifePolicy::TeamReports,
            roster: rules::RoundRoster::Players,
            ..RoomRules::default()
        };
        r.managed_round = Some(Arc::new(ManagedRound {
            characters: Some(crate::characters::Catalog::measured(&[0, 1]).unwrap()),
            bodies: HashMap::new(),
            initial_bomb_capacity: Vec::new(),
            rules: r.rules,
            limits: rules::ReportLimits {
                power: Vec::new(),
                opaque167: &[],
            },
        }));
        r.floor_catalog = pickups::Catalog::from_floor(&floor_body).unwrap();
        r.bodies.insert(Template::Floor582, floor_body);
        r.model = Room::with_allocation(8, rules::SlotAllocation::FirstVacant).unwrap();
        r.model.round_roster = rules::RoundRoster::Players;
        let keys = std::array::from_fn(|i| {
            let s = &players[i];
            let key = r.model.enter(s.identity, s.connection, i == 0).unwrap();
            r.model.setup(key).unwrap();
            r.model.choose(key, Some(i as u16), None).unwrap();
            r.model.choose(key, None, Some(3 * i as u8)).unwrap();
            key
        });
        drop(r);
        Self {
            shared,
            players,
            keys,
        }
    }

    fn start(&mut self) -> u64 {
        self.shared
            .lock()
            .unwrap()
            .model
            .ready(self.keys[1])
            .unwrap();
        self.players[0].packet(104, &[0; 20]).unwrap();
        for s in &mut self.players {
            let batch = s.inbox.try_recv().unwrap();
            assert!(s.valid(&batch));
            assert!(batch.packets.iter().map(|(_, p)| p.opcode).eq([582, 121]));
            s.finish(&batch).unwrap();
        }
        self.shared.lock().unwrap().model.round_generation()
    }

    fn arm(&mut self) {
        for (slot, s) in self.players.iter().enumerate() {
            let mut body = [0; 23];
            body[8..10]
                .copy_from_slice(&kr_wire::write_uint(9000 + slot as u64, Width::U16).unwrap());
            s.packet(99, &body).unwrap();
        }
        for s in &mut self.players {
            for opcode in [197, 196, 153] {
                let batch = s.inbox.try_recv().unwrap();
                assert!(s.valid(&batch));
                assert_eq!(batch.packets[0].1.opcode, opcode);
                s.finish(&batch).unwrap();
            }
        }
    }

    fn end(&self, round: u64) {
        let mut r = self.shared.lock().unwrap();
        r.model.end(round).unwrap();
        for key in self.keys {
            r.model.refreshed(key, round).unwrap();
        }
    }

    fn count(&self, slot: usize, kind: u16) -> u16 {
        self.shared
            .lock()
            .unwrap()
            .model
            .pickups
            .inventory_count(self.keys[slot], kind)
    }

    fn confirm(&mut self, slot: usize, claim: &[u8], expected: &str) {
        assert!(
            self.players[slot]
                .packet(124, claim)
                .unwrap()
                .pickup
                .unwrap()
                .accepted
        );
        for s in &mut self.players {
            let batch = s.inbox.try_recv().unwrap();
            assert!(s.valid(&batch));
            assert_eq!(batch.packets.len(), 1);
            assert_eq!(batch.packets[0].1.opcode, 165);
            assert_eq!(batch.packets[0].1.payload, bytes(expected));
            s.finish(&batch).unwrap();
        }
    }

    fn scan(&mut self, slot: usize, report: &[u8], reason: &str) {
        let decision = self.players[slot]
            .packet(127, report)
            .unwrap()
            .life
            .unwrap();
        assert_eq!(decision.reason, reason);
        assert_eq!(decision.accepted, reason == "accepted");
        assert!(!decision.forward);
        assert!(self.players.iter().all(Session::channel_current));
        self.no_packets();
    }

    fn no_packets(&mut self) {
        assert!(self.players.iter_mut().all(|s| s.inbox.try_recv().is_err()));
    }
}

fn bind_claim(hex: &str, round: u64) -> Vec<u8> {
    let mut body = bytes(hex);
    let key = crate::room::wire_key(round).unwrap();
    body[14..18].copy_from_slice(&kr_wire::write_uint(u64::from(key), Width::U32).unwrap());
    body
}

#[test]
fn p93_pickups_confirm_to_both_players_and_round_two_starts_empty() {
    let policy = serde_json::from_str(r#""report_derived_f1b_extended""#).unwrap();
    let mut c = Case::new(policy, floor(&FOREST));
    let round = c.start();
    c.arm();
    for (kind, amount, claim, reply) in FOREST {
        let body = bind_claim(claim, round);
        let slot = usize::from(body[1] ^ 0x5a);
        c.confirm(slot, &body, reply);
        assert_eq!(c.count(slot, kind), amount);
        assert_eq!(c.count(1 - slot, kind), 0);
        assert!(
            !c.players[slot]
                .packet(124, &body)
                .unwrap()
                .pickup
                .unwrap()
                .accepted
        );
        assert_eq!(c.count(slot, kind), amount);
        c.no_packets();
    }
    c.end(round);
    assert_ne!(round, c.start());
    for (kind, _, _, _) in FOREST {
        assert_eq!(c.count(0, kind), 0);
        assert_eq!(c.count(1, kind), 0);
    }
    c.arm();
    let (_, _, old_claim, _) = FOREST[0];
    assert!(
        !c.players[0]
            .packet(124, &bind_claim(old_claim, round))
            .unwrap()
            .pickup
            .unwrap()
            .accepted
    );
    c.no_packets();
}

#[test]
fn scanner_consumes_only_current_alive_owners_charge_without_forwarding() {
    let mut c = Case::new(Policy::ReportDerivedF1bExtended, floor(&SCANNER_FLOOR));
    let round = c.start();
    c.arm();
    let (_, _, claim_a, reply_a) = SCANNER_FLOOR[0];
    let (_, _, claim_b, reply_b) = SCANNER_FLOOR[1];
    let scan_a = bytes(SCAN_A);
    let scan_b = bytes(SCAN_B);
    c.scan(0, &scan_a, "scanner_no_charge");
    c.confirm(0, &bind_claim(claim_a, round), reply_a);
    c.scan(0, &scan_b, "slot_mismatch");
    c.scan(1, &scan_b, "scanner_no_charge");
    assert_eq!(c.count(0, SCANNER), 1);
    c.scan(0, &scan_a, "accepted");
    assert_eq!(c.count(0, SCANNER), 0);
    c.scan(0, &scan_a, "duplicate_report_body");
    let mut later = scan_a.clone();
    later[6..10].copy_from_slice(&kr_wire::write_uint(8102, Width::U32).unwrap());
    c.scan(0, &later, "scanner_no_charge");

    c.confirm(1, &bind_claim(claim_b, round), reply_b);
    let mut trap = bytes(TRAP_A);
    trap[1] = scan_b[1];
    assert!(
        c.players[1]
            .packet(127, &trap)
            .unwrap()
            .life
            .unwrap()
            .accepted
    );
    assert_eq!(
        c.players[0].inbox.try_recv().unwrap().packets[0].1.opcode,
        170
    );
    c.scan(1, &scan_b, "scanner_actor_ineligible");
    // Synthetic death19 with an empty serialized list, through the room's
    // actual report admission rather than setting the actor's death bit.
    let mut death = trap[..14].to_vec();
    death[0] = 19 ^ 0x5a;
    death.extend(kr_wire::write_uint(11, Width::U32).unwrap());
    death.extend(wire::encode_bytes(&[24, 6, 0, 0, 0, 0, 0, 0, 0, 0, 0]).unwrap());
    assert!(
        c.players[1]
            .packet(127, &death)
            .unwrap()
            .life
            .unwrap()
            .accepted
    );
    assert_eq!(
        c.players[0].inbox.try_recv().unwrap().packets[0].1.opcode,
        170
    );
    c.scan(1, &scan_b, "scanner_actor_ineligible");
    assert_eq!(c.count(1, SCANNER), 1);

    c.end(round);
    let next_round = c.start();
    c.scan(1, &scan_b, "inactive_round");
    c.arm();
    c.scan(1, &scan_b, "scanner_no_charge");
    let mut new_claim = bind_claim(claim_b, next_round);
    new_claim[10..14].copy_from_slice(&kr_wire::write_uint(9000, Width::U32).unwrap());
    c.confirm(1, &new_claim, reply_b);
    c.scan(1, &scan_b, "scanner_tick_not_after_pickup");
    assert_eq!(c.count(1, SCANNER), 1);
    let mut current = scan_b;
    current[6..10].copy_from_slice(&kr_wire::write_uint(9001, Width::U32).unwrap());
    c.scan(1, &current, "accepted");
    assert_eq!(c.count(1, SCANNER), 0);
}

#[test]
fn scanner_pickup_has_no_charge_or_claim_if_a_confirmation_queue_is_full() {
    let mut c = Case::new(Policy::ReportDerivedF1bExtended, floor(&SCANNER_FLOOR));
    let round = c.start();
    c.arm();
    let (_, _, claim, reply) = SCANNER_FLOOR[0];
    let body = bind_claim(claim, round);
    let recipient = c.keys[1];
    let sender = c.shared.lock().unwrap().channels[&recipient.identity]
        .sender
        .clone();
    while sender.capacity() != 0 {
        sender
            .try_send(Runtime::member_batch(recipient, Some(round), 0, vec![]))
            .unwrap_or_else(|_| panic!("fill confirmation queue"));
    }
    assert!(c.players[0].packet(124, &body).is_err());
    assert_eq!(c.count(0, SCANNER), 0);
    while c.players[1].inbox.try_recv().is_ok() {}
    c.no_packets();
    c.confirm(0, &body, reply);
    assert_eq!(c.count(0, SCANNER), 1);
}

#[test]
fn g1_policies_reject_extended_catalogs_and_pickups() {
    for policy in [Policy::ReportDerivedBasic, Policy::ReportDerivedShoePunch] {
        for rows in [FOREST.as_slice(), SCANNER_FLOOR.as_slice()] {
            let body = floor(rows);
            let mut c = Case::new(policy, body);
            assert!(
                c.shared
                    .lock()
                    .unwrap()
                    .floor_catalog
                    .validate_stat_policy(policy)
                    .is_err()
            );
            let round = c.start();
            c.arm();
            // Bypass startup validation in the fixture to exercise the packet guard too.
            for &(_, _, claim, _) in rows {
                let body = bind_claim(claim, round);
                let slot = usize::from(body[1] ^ 0x5a);
                assert!(c.players[slot].packet(124, &body).is_err());
            }
            assert!(c.players[0].packet(127, &bytes(SCAN_A)).is_err());
            c.no_packets();
        }
    }
}
