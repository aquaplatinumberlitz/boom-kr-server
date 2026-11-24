//! Room packet tests for the opt-in F1-b RemoteBomb (118 flag 0x04 / 122 reason 2)
//! and Drill (127/sub3 -> 170/sub3 + terrain box clear) handlers.
//! Wires are derived from o8 / kr_static IDA analysis; no Wine run backs them yet.
use super::*;
use crate::room::wire_key;

const REMOTE_DRILL_POLICY: &str = r#""report_derived_f1b_remote_drill""#;
const REMOTE_SENSOR_POLICY: &str = r#""report_derived_f1b_remote_sensor""#;

fn fields(values: &[(u64, Width)]) -> Vec<u8> {
    values
        .iter()
        .flat_map(|&(v, w)| kr_wire::write_uint(v, w).unwrap())
        .collect()
}

fn floor_catalog(visible: &[(u8, u8, u16)], hidden: &[(u8, u8, u16)]) -> Vec<u8> {
    let mut raw = vec![24];
    let len = 7 + 10 * (visible.len() + hidden.len());
    raw.extend((len as u32).to_le_bytes());
    raw.push(0);
    let mut index = 0_u32;
    for group in [visible, hidden] {
        raw.extend((group.len() as u16).to_le_bytes());
        for &(row, col, kind) in group {
            raw.extend(index.to_le_bytes());
            raw.extend([row, col]);
            raw.extend(u32::from(kind).to_le_bytes());
            index += 1;
        }
    }
    raw.extend([0; 2]);
    let mut body = kr_wire::write_uint(raw.len() as u64, Width::U32).unwrap();
    body.extend(wire::encode_bytes(&raw).unwrap());
    body
}

fn bomb118(round: u64, slot: u8, serial: u32, row: u8, col: u8, remote: bool) -> Vec<u8> {
    fields(&[
        (u64::from(wire_key(round).unwrap()), Width::U32),
        (0, Width::U8),
        (u64::from(slot), Width::U8),
        (u64::from(serial), Width::U32),
        (u64::from(row), Width::U8),
        (u64::from(col), Width::U8),
        (1, Width::U8),
        (0, Width::U8),
        (0, Width::U8),
        (0, Width::U8),
        (1, Width::U8),
        (1, Width::U32),
        (if remote { 4 } else { 0 }, Width::U8),
    ])
}

fn remote122(bomb_id: u32, row: u16, col: u16, delay: u32) -> Vec<u8> {
    fields(&[
        (u64::from(bomb_id), Width::U32),
        (u64::from(row), Width::U16),
        (u64::from(col), Width::U16),
        (u64::from(wire::REMOTE_DETONATION_REASON), Width::U32),
        (0, Width::U8),
        (u64::from(delay), Width::U32),
    ])
}

fn drill_sub3(slot: u8, tick: u32, box_id: u32, row: u8, col: u8, dir: u8) -> Vec<u8> {
    let mut raw = vec![24, 8, 0, 0, 0, 0];
    raw.extend_from_slice(&box_id.to_le_bytes());
    raw.extend_from_slice(&[row, col, dir]);
    let mut out = fields(&[
        (u64::from(wire::DRILL_SUBTYPE), Width::U8),
        (u64::from(slot), Width::U8),
        (80, Width::U16),
        (120, Width::U16),
        (u64::from(tick), Width::U32),
        (1, Width::U32),
        (raw.len() as u64, Width::U32),
    ]);
    out.extend(wire::encode_bytes(&raw).unwrap());
    out
}

fn item_use_sub0(slot: u8, kind: u16, tick: u32) -> Vec<u8> {
    fields(&[
        (0, Width::U8),
        (u64::from(slot), Width::U8),
        (80, Width::U16),
        (120, Width::U16),
        (u64::from(tick), Width::U32),
        (2, Width::U32),
        (1, Width::U8),
        (u64::from(kind), Width::U16),
        (0, Width::U8),
        (0, Width::U32),
    ])
}

fn reveal142(slot: u8, row: u8, col: u8) -> Vec<u8> {
    fields(&[
        (u64::from(row), Width::U8),
        (u64::from(col), Width::U8),
        (u64::from(slot), Width::U8),
        (u64::from(u32::MAX), Width::U32),
    ])
}

struct Case {
    shared: Shared,
    players: [Session; 2],
    keys: [Membership; 2],
}

impl Case {
    fn new(policy: &str) -> Self {
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
        let player_template = wire::PlayerTemplate::parse(&r.players[&1].payload).unwrap();
        for (s, name) in players.iter().zip([b"Alice", b"Bobby"]) {
            let body = player_template.project(s.identity, name).unwrap();
            r.players.insert(s.identity, packet(106, body));
        }
        r.rules = RoomRules {
            item_policy: serde_json::from_str(policy).unwrap(),
            life: rules::LifePolicy::TeamReports,
            result: ResultPolicy::Players,
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
        let body = floor_catalog(
            &[
                (12, 0, wire::REMOTE_BOMB_KIND),
                (12, 1, wire::DRILL_KIND),
                (12, 2, wire::SENSOR_KIND),
            ],
            &[(3, 5, 1)],
        );
        r.floor_catalog = pickups::Catalog::from_floor(&body).unwrap();
        r.floor_catalog
            .validate_stat_policy(serde_json::from_str(REMOTE_DRILL_POLICY).unwrap())
            .unwrap();
        r.bodies.insert(Template::Floor582, body);
        r.model = Room::with_allocation(8, rules::SlotAllocation::FirstVacant).unwrap();
        r.model.terrain = Some(crate::terrain::tests::map());
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
            receive(s, &[582, 121]);
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
                receive(s, &[opcode]);
            }
        }
    }

    fn claim(&mut self, slot: usize, column: u8, kind: u16, round: u64, tick: u32) {
        let body = fields(&[
            (0, Width::U8),
            (slot as u64, Width::U8),
            (12, Width::U8),
            (u64::from(column), Width::U8),
            (u64::from(column), Width::U32),
            (u64::from(kind), Width::U16),
            (u64::from(tick), Width::U32),
            (u64::from(wire_key(round).unwrap()), Width::U32),
            (0, Width::U8),
        ]);
        let decision = self.players[slot]
            .packet(124, &body)
            .unwrap()
            .pickup
            .unwrap();
        assert!(decision.accepted);
        for s in &mut self.players {
            receive(s, &[165]);
        }
    }

    fn end_round(&mut self) {
        {
            let mut r = self.shared.lock().unwrap();
            let now = Instant::now();
            r.model.last_blast = None;
            r.model
                .members
                .get_mut(&self.keys[1].identity)
                .unwrap()
                .life
                .death = Some(now - Duration::from_secs(9));
            r.model
                .members
                .get_mut(&self.keys[0].identity)
                .unwrap()
                .life
                .survivor = Some(now);
        }
        self.players[0].tick().unwrap();
        for s in &mut self.players {
            receive(s, &[175]);
        }
        for s in &mut self.players {
            receive(s, &[180]);
            receive(s, &[110, 110]);
        }
    }
}

fn receive(s: &mut Session, opcodes: &[u32]) -> Batch {
    let batch = s.inbox.try_recv().unwrap();
    assert!(s.valid(&batch));
    assert!(
        batch
            .packets
            .iter()
            .map(|(_, p)| p.opcode)
            .eq(opcodes.iter().copied())
    );
    s.finish(&batch).unwrap();
    batch
}

#[test]
fn remote_drill_policy_gates_floor_and_extends_remote_sensor() {
    let drill_policy: pickups::Policy = serde_json::from_str(REMOTE_DRILL_POLICY).unwrap();
    let sensor_policy: pickups::Policy = serde_json::from_str(REMOTE_SENSOR_POLICY).unwrap();
    assert!(drill_policy.f1b_remote_drill());
    assert!(drill_policy.f1b_remote_sensor());
    assert!(drill_policy.f1b_aux());
    assert!(drill_policy.f1b_extended());
    assert!(!drill_policy.f1d_vehicles());
    assert!(drill_policy.allows(wire::REMOTE_BOMB_KIND));
    assert!(drill_policy.allows(wire::DRILL_KIND));
    assert!(drill_policy.allows(wire::SENSOR_KIND));
    assert!(drill_policy.charges_item_use(wire::ItemUseKind::RemoteBomb));
    assert!(drill_policy.charges_item_use(wire::ItemUseKind::Drill));
    assert!(!sensor_policy.f1b_remote_drill());
    assert!(!sensor_policy.charges_item_use(wire::ItemUseKind::RemoteBomb));
    assert!(!sensor_policy.charges_item_use(wire::ItemUseKind::Drill));
}

#[test]
fn remote_bomb_places_three_charges_without_auto_158_and_detonates_on_122() {
    let mut case = Case::new(REMOTE_DRILL_POLICY);
    let r1 = case.start();
    case.arm();
    case.claim(0, 0, wire::REMOTE_BOMB_KIND, r1, 100);

    // Increase Alice's capacity using two normal fires? No, place-detonate across 3 bombs,
    // verifying both normal capacity exhaustion doesn't linger and charge runs out at 3.
    let mut ids = Vec::new();
    for serial in 1..=3 {
        let cell = if serial == 1 {
            (3, 4)
        } else {
            (0, serial as u8)
        };
        let b = case.players[0]
            .packet(118, &bomb118(r1, 0, serial, cell.0, cell.1, true))
            .unwrap()
            .bomb
            .unwrap();
        assert!(b.accepted);
        let id = b.id.unwrap();
        ids.push(id);
        for s in &mut case.players {
            let spawn = receive(s, &[159]);
            assert_eq!(spawn.packets[0].1.opcode, 159);
            assert!(matches!(
                s.inbox.try_recv(),
                Err(mpsc::error::TryRecvError::Empty)
            ));
        }

        // Duplicate 118 or foreign 122 cannot detonate.
        let foreign = case.players[1]
            .packet(
                122,
                &remote122(u32::from(id), cell.0.into(), cell.1.into(), 80),
            )
            .unwrap()
            .bomb
            .unwrap();
        assert_eq!((foreign.accepted, foreign.reason), (false, "foreign_bomb"));

        let det = case.players[0]
            .packet(
                122,
                &remote122(u32::from(id), cell.0.into(), cell.1.into(), 120),
            )
            .unwrap()
            .bomb
            .unwrap();
        assert_eq!(
            (det.accepted, det.reason),
            (true, "remote_detonation_accepted")
        );
        for s in &mut case.players {
            let mut due = s.inbox.try_recv().unwrap();
            assert_eq!(due.packets[0].1.opcode, 158);
            assert!(due.due >= Instant::now() + Duration::from_millis(50));
            assert!(s.prepare_delivery(&mut due).unwrap());
            s.finish(&due).unwrap();
        }
        let dup = case.players[0]
            .packet(
                122,
                &remote122(u32::from(id), cell.0.into(), cell.1.into(), 0),
            )
            .unwrap()
            .bomb
            .unwrap();
        assert_eq!((dup.accepted, dup.reason), (false, "inactive_bomb"));
    }

    // Box (3, 5) adjacent to bomb 1 at (3, 4) was cleared on 122 delivery and reveals hidden item.
    assert!(
        case.shared
            .lock()
            .unwrap()
            .model
            .terrain
            .as_ref()
            .unwrap()
            .clearance(r1, 3, 5)
            .is_some()
    );
    assert_eq!(
        case.players[0]
            .packet(142, &reveal142(0, 3, 5))
            .unwrap()
            .terrain_notice,
        Some("revealed_current_hidden_cell")
    );

    // 4th remote bomb placement is denied after 3 charges are spent, while ordinary bombs work.
    let exhausted = case.players[0]
        .packet(118, &bomb118(r1, 0, 4, 0, 4, true))
        .unwrap()
        .bomb
        .unwrap();
    assert_eq!(
        (exhausted.accepted, exhausted.reason),
        (false, "remote_bomb_ineligible")
    );
    let normal = case.players[0]
        .packet(118, &bomb118(r1, 0, 5, 0, 5, false))
        .unwrap()
        .bomb
        .unwrap();
    assert!(normal.accepted);
    let normal_id = normal.id.unwrap();
    for s in &mut case.players {
        receive(s, &[159]);
        let mut due = s.inbox.try_recv().unwrap();
        assert!(due.due >= Instant::now() + Duration::from_secs(5));
        assert!(s.prepare_delivery(&mut due).unwrap());
        s.finish(&due).unwrap();
    }
    let wrong_normal = case.players[0]
        .packet(122, &remote122(u32::from(normal_id), 0, 5, 10))
        .unwrap()
        .bomb
        .unwrap();
    assert_eq!(
        (wrong_normal.accepted, wrong_normal.reason),
        (false, "inactive_bomb")
    );

    // Check 127/sub0 inventory consumption and exhaustion.
    for tick in 201..=203 {
        assert_eq!(
            case.players[0]
                .packet(127, &item_use_sub0(0, wire::REMOTE_BOMB_KIND, tick))
                .unwrap()
                .life
                .unwrap()
                .reason,
            "accepted"
        );
    }
    assert_eq!(
        case.players[0]
            .packet(127, &item_use_sub0(0, wire::REMOTE_BOMB_KIND, 204))
            .unwrap()
            .life
            .unwrap()
            .reason,
        "scanner_no_charge"
    );

    // Without remote_drill policy, v[12] == 4 on 118 fails parsing.
    let sensor_case = Case::new(REMOTE_SENSOR_POLICY);
    assert!(
        sensor_case.players[0]
            .packet(118, &bomb118(r1, 0, 99, 0, 0, true))
            .is_err()
    );

    case.end_round();
    let r2 = case.start();
    case.arm();
    let reset = case.players[0]
        .packet(118, &bomb118(r2, 0, 1, 0, 0, true))
        .unwrap()
        .bomb
        .unwrap();
    assert_eq!(
        (reset.accepted, reset.reason),
        (false, "remote_bomb_ineligible")
    );
}

#[test]
fn drill_sub3_relays_170_clears_terrain_box_and_respects_three_charges() {
    let mut case = Case::new(REMOTE_DRILL_POLICY);
    let r1 = case.start();
    case.arm();
    case.claim(0, 1, wire::DRILL_KIND, r1, 100);

    // Invalid slot, bad direction, or non-box cell rejects without spending Drill charges.
    assert_eq!(
        case.players[0]
            .packet(127, &drill_sub3(1, 150, 1, 3, 5, 2))
            .unwrap()
            .life
            .unwrap()
            .reason,
        "slot_mismatch"
    );
    assert_eq!(
        case.players[0]
            .packet(127, &drill_sub3(0, 151, 1, 3, 5, 4))
            .unwrap()
            .life
            .unwrap()
            .reason,
        "drill_ineligible"
    );
    assert_eq!(
        case.players[0]
            .packet(127, &drill_sub3(0, 152, 1, 0, 0, 2))
            .unwrap()
            .life
            .unwrap()
            .reason,
        "drill_ineligible"
    );

    // Drill box (3, 5): relays 170/sub3 to Bobby and unlocks reveal142.
    let drill = drill_sub3(0, 200, 0x1122_3344, 3, 5, 2);
    let d1 = case.players[0].packet(127, &drill).unwrap().life.unwrap();
    assert_eq!(
        (d1.accepted, d1.reason, d1.forward),
        (true, "accepted", true)
    );
    let peer_pkt = receive(&mut case.players[1], &[170]);
    assert_eq!(
        peer_pkt.packets[0].1.payload,
        wire::drill170(wire::drill127(&drill).unwrap()).unwrap()
    );
    assert_eq!(
        case.players[0]
            .packet(142, &reveal142(0, 3, 5))
            .unwrap()
            .terrain_notice,
        Some("revealed_current_hidden_cell")
    );

    // Drill remaining 2 charges across (4, 4) and another box on terrain (re-check or add box?
    // Default map has boxes at (3,5) and (4,4) -> 2 boxes! Claim again? Verify (4,4) succeeds,
    // then same (4,4) fails without box and stays at 1 charge until claimed? Actually default map
    // has 2 boxes: (3,5) and (4,4). So after drilling (4,4) cleanly, let's check both sub0 and reset!).
    let d2 = case.players[0]
        .packet(127, &drill_sub3(0, 201, 0x2233_4455, 4, 4, 1))
        .unwrap()
        .life
        .unwrap();
    assert_eq!(
        (d2.accepted, d2.reason, d2.forward),
        (true, "accepted", true)
    );
    receive(&mut case.players[1], &[170]);

    // Already drilled cell rejects cleanly.
    assert_eq!(
        case.players[0]
            .packet(127, &drill_sub3(0, 202, 0x2233_4455, 4, 4, 1))
            .unwrap()
            .life
            .unwrap()
            .reason,
        "drill_ineligible"
    );

    // 127/sub0 consumes 3 inventory charges, then returns scanner_no_charge.
    for tick in 210..=212 {
        assert_eq!(
            case.players[0]
                .packet(127, &item_use_sub0(0, wire::DRILL_KIND, tick))
                .unwrap()
                .life
                .unwrap()
                .reason,
            "accepted"
        );
    }
    assert_eq!(
        case.players[0]
            .packet(127, &item_use_sub0(0, wire::DRILL_KIND, 213))
            .unwrap()
            .life
            .unwrap()
            .reason,
        "scanner_no_charge"
    );

    // Invalid sub3 wire length stays a hard error.
    assert!(case.players[0].packet(127, &drill[..30]).is_err());

    case.end_round();
    let _r2 = case.start();
    case.arm();
    // Reset wiped remaining action charge even on a fresh map with intact box (3, 5).
    assert_eq!(
        case.players[0]
            .packet(127, &drill_sub3(0, 300, 1, 3, 5, 0))
            .unwrap()
            .life
            .unwrap()
            .reason,
        "drill_ineligible"
    );
}
