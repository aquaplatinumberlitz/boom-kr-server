//! Room packet tests with synthetic floor/roster and p100b/o8 Ctrl & group 11 wires.
use super::*;
use crate::room::wire_key;

const AUX_ITEMS: [u16; 7] = [
    wire::DART_KIND,
    wire::SCANNER_KIND,
    wire::SPRING_KIND,
    wire::BOND_KIND,
    wire::TRAP_KIND,
    wire::PIN_KIND,
    wire::BANANA1_KIND,
];

fn floor(kinds: &[u16], hidden: bool) -> Vec<u8> {
    let mut raw = vec![24];
    raw.extend((7 + 10 * kinds.len() as u32).to_le_bytes());
    raw.push(0);
    if hidden {
        raw.extend([0; 2]);
    }
    raw.extend((kinds.len() as u16).to_le_bytes());
    for (column, &kind) in kinds.iter().enumerate() {
        raw.extend((column as u32).to_le_bytes());
        raw.extend([12, column as u8]);
        raw.extend(u32::from(kind).to_le_bytes());
    }
    raw.extend(if hidden { &[0; 2][..] } else { &[0; 4] });
    let mut body = kr_wire::write_uint(raw.len() as u64, Width::U32).unwrap();
    body.extend(wire::encode_bytes(&raw).unwrap());
    body
}

fn item_use_sub0(slot: u8, kind: u16, tick: u32) -> Vec<u8> {
    let mut body = Vec::with_capacity(22);
    for (v, w) in [
        (0, Width::U8),
        (u64::from(slot), Width::U8),
        (160, Width::U16),
        (480, Width::U16),
        (u64::from(tick), Width::U32),
        (2, Width::U32),
        (1, Width::U8),
        (u64::from(kind), Width::U16),
        (0, Width::U8),
        (0, Width::U32),
    ] {
        body.extend(kr_wire::write_uint(v, w).unwrap());
    }
    body
}

struct Case {
    shared: Shared,
    players: [Session; 2],
    keys: [Membership; 2],
}

impl Case {
    fn new() -> Self {
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
            item_policy: serde_json::from_str(r#""report_derived_f1b_aux""#).unwrap(),
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
        let body = floor(&AUX_ITEMS, false);
        r.floor_catalog = pickups::Catalog::from_floor(&body).unwrap();
        r.floor_catalog
            .validate_stat_policy(r.rules.item_policy)
            .unwrap();
        r.bodies.insert(Template::Floor582, body);
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
            let batch = receive(s, &[582, 121]);
            let catalog = pickups::Catalog::from_floor(&batch.packets[0].1.payload).unwrap();
            assert!(catalog.visible_items().map(|item| item.3).eq(AUX_ITEMS));
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

    fn claim(&mut self, slot: usize, column: usize, kind: u16, round: u64, tick: u32) {
        let claim: Vec<_> = [
            (0, Width::U8),
            (slot as u64, Width::U8),
            (12, Width::U8),
            (column as u64, Width::U8),
            (column as u64, Width::U32),
            (u64::from(kind), Width::U16),
            (u64::from(tick), Width::U32),
            (u64::from(wire_key(round).unwrap()), Width::U32),
            (0, Width::U8),
        ]
        .into_iter()
        .flat_map(|(v, w)| kr_wire::write_uint(v, w).unwrap())
        .collect();
        self.players[slot].packet(124, &claim).unwrap();
        for s in &mut self.players {
            receive(s, &[165]);
        }
    }

    fn kill_peer(&mut self) {
        {
            let mut r = self.shared.lock().unwrap();
            let now = std::time::Instant::now();
            r.model
                .members
                .get_mut(&self.keys[1].identity)
                .unwrap()
                .life
                .death = Some(now - std::time::Duration::from_secs(9));
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
        self.no_packets();
    }

    fn no_packets(&mut self) {
        assert!(
            self.players
                .iter_mut()
                .all(|s| matches!(s.inbox.try_recv(), Err(mpsc::error::TryRecvError::Empty)))
        );
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
fn non_aux_policies_reject_aux_floor_items_and_opcodes() {
    let aux_floor = floor(
        &[wire::SPRING_KIND, wire::PIN_KIND, wire::BANANA1_KIND],
        false,
    );
    let catalog = pickups::Catalog::from_floor(&aux_floor).unwrap();
    for name in [
        r#""disabled""#,
        r#""report_derived_basic""#,
        r#""report_derived_punch""#,
        r#""report_derived_shoe_punch""#,
        r#""report_derived_f1b_extended""#,
        r#""report_derived_f1d_vehicles""#,
    ] {
        let policy: pickups::Policy = serde_json::from_str(name).unwrap();
        assert!(!policy.f1b_aux());
        assert!(catalog.validate_stat_policy(policy).is_err());
    }

    let mut case = Case::new();
    case.shared.lock().unwrap().rules.item_policy =
        serde_json::from_str(r#""report_derived_f1b_extended""#).unwrap();
    case.start();
    case.arm();
    assert!(
        case.players[0]
            .packet(138, &[0x5a, 0x5a, 0xa5, 0xc9, 0xa7, 0x61, 0x5a, 0xa5, 0xc9])
            .is_err()
    );
    assert!(
        case.players[0]
            .packet(139, &[0x5a, 0x5b, 0x51, 0x58])
            .is_err()
    );
    assert!(
        case.players[0]
            .packet(126, &[0x5a, 0x5b, 0x51, 0x58])
            .is_err()
    );
}

#[test]
fn dart_group11_spring_and_shield_relay_and_reset_between_rounds() {
    let mut case = Case::new();
    let r1 = case.start();
    case.arm();

    // Claim Dart(143, x3), Bond(224, x3), Pin(226, x3), Spring(148, x3).
    case.claim(0, 0, wire::DART_KIND, r1, 100);
    case.claim(0, 3, wire::BOND_KIND, r1, 110);
    case.claim(0, 5, wire::PIN_KIND, r1, 120);
    case.claim(0, 2, wire::SPRING_KIND, r1, 130);

    // 1. Dart: 3 uses (127/sub0 before 138, matching p100b) -> 4th is no-op without drop.
    let dart_wire = [0x5a, 0x5a, 0xa5, 0xc9, 0xa7, 0x61, 0x5a, 0xa5, 0xc9];
    for step in 0..3 {
        case.players[0]
            .packet(127, &item_use_sub0(0, wire::DART_KIND, 200 + step))
            .unwrap();
        case.players[0].packet(138, &dart_wire).unwrap();
        let peer_dart = receive(&mut case.players[1], &[172]);
        assert_eq!(peer_dart.packets[0].1.payload, dart_wire);
    }
    assert_eq!(
        case.shared
            .lock()
            .unwrap()
            .model
            .pickups
            .inventory_count(case.keys[0], wire::DART_KIND),
        0
    );
    // Depleted 4th throw emits no peer 172 and keeps session alive.
    case.players[0].packet(138, &dart_wire).unwrap();
    case.no_packets();

    // 2. Group 11: Bond placement (139 -> 173, slot stripped) then sub0, duplicate blocked, then 126 -> 168 trigger & clear.
    let bond_place = [0x5a, 0x5b, 0x51, 0x58]; // slot=0, kind_flags=1, row=11, col=2
    case.players[0].packet(139, &bond_place).unwrap();
    let peer_place = receive(&mut case.players[1], &[173]);
    assert_eq!(peer_place.packets[0].1.payload, [0x5b, 0x51, 0x58]);
    case.players[0]
        .packet(127, &item_use_sub0(0, wire::BOND_KIND, 210))
        .unwrap();
    assert_eq!(
        case.shared.lock().unwrap().model.pickups.placed_cell(11, 2),
        Some(1)
    );
    // Placing on already occupied (11,2) refunds action charge and emits no 173.
    case.players[0].packet(139, &bond_place).unwrap();
    case.no_packets();

    // Peer steps on (11,2) -> C2S 126 -> S2C 168 with result=1 clears cell.
    let trig_bond = [0x5a, 0x5b, 0x51, 0x58]; // world=0, slot=1, row=11, col=2
    case.players[1].packet(126, &trig_bond).unwrap();
    let a_168 = receive(&mut case.players[0], &[168]);
    let b_168 = receive(&mut case.players[1], &[168]);
    assert_eq!(a_168.packets[0].1.payload[0], 0x5b); // 1 ^ 0x5a = result 1
    assert_eq!(b_168.packets[0].1.payload[0], 0x5b);
    assert_eq!(
        case.shared.lock().unwrap().model.pickups.placed_cell(11, 2),
        None
    );

    // 2nd trigger on now-empty (11,2) yields result=0.
    case.players[1].packet(126, &trig_bond).unwrap();
    let a_empty = receive(&mut case.players[0], &[168]);
    receive(&mut case.players[1], &[168]);
    assert_eq!(a_empty.packets[0].1.payload[0], 0x5a); // 0 ^ 0x5a = result 0

    // Pin (selector 4) triggers with result=1 without clearing cell.
    let pin_place = [0x5a, 0xde, 0x51, 0x5a]; // slot=0, kind_flags=132, row=11, col=0
    case.players[0].packet(139, &pin_place).unwrap();
    let peer_pin = receive(&mut case.players[1], &[173]);
    assert_eq!(peer_pin.packets[0].1.payload, [0xde, 0x51, 0x5a]);
    let trig_pin = [0x5a, 0x5b, 0x51, 0x5a];
    case.players[1].packet(126, &trig_pin).unwrap();
    assert_eq!(
        receive(&mut case.players[0], &[168]).packets[0].1.payload[0],
        0x5b
    );
    receive(&mut case.players[1], &[168]);
    assert_eq!(
        case.shared.lock().unwrap().model.pickups.placed_cell(11, 0),
        Some(132)
    );

    // 3. Spring live 27B packet -> 9B plain peer 170/sub2, then sub0 drain.
    let spring_wire = [
        0x58, 0x5a, 0xa5, 0x41, 0xa4, 0xde, 0x96, 0xca, 0x55, 0xee, 0x96, 0xca, 0x53, 0x9d, 0x96,
        0xca, 0x53, 0x9c, 0x96, 0xca, 0x57, 0x8d, 0x97, 0xc2, 0x53, 0x95, 0x4b,
    ];
    case.players[0].packet(127, &spring_wire).unwrap();
    let peer_spring = receive(&mut case.players[1], &[170]);
    assert_eq!(peer_spring.packets[0].1.payload.len(), 9);
    case.players[0]
        .packet(127, &item_use_sub0(0, wire::SPRING_KIND, 220))
        .unwrap();

    // 4. Shield 127/sub5 -> 170/sub5 forward.
    let mut shield_wire = Vec::new();
    for (v, w) in [
        (u64::from(wire::SHIELD_SUBTYPE), Width::U8),
        (0, Width::U8),
        (80, Width::U16),
        (160, Width::U16),
        (999, Width::U32),
        (1, Width::U32),
    ] {
        shield_wire.extend(kr_wire::write_uint(v, w).unwrap());
    }
    case.players[0].packet(127, &shield_wire).unwrap();
    let peer_shield = receive(&mut case.players[1], &[170]);
    assert_eq!(peer_shield.packets[0].1.payload, shield_wire[..6]);

    // Finish round 1 cleanly -> Start 121 round 2 resets placed cells and inventory.
    case.kill_peer();
    let r2 = case.start();
    assert_ne!(r1, r2);
    case.arm();
    assert_eq!(
        case.shared.lock().unwrap().model.pickups.placed_cell(11, 0),
        None
    );
    assert_eq!(
        case.shared
            .lock()
            .unwrap()
            .model
            .pickups
            .inventory_count(case.keys[0], wire::SPRING_KIND),
        0
    );
    // Empty cell trigger returns result=0 in round 2, and re-claiming Dart works.
    case.players[1].packet(126, &trig_pin).unwrap();
    assert_eq!(
        receive(&mut case.players[0], &[168]).packets[0].1.payload[0],
        0x5a
    );
    receive(&mut case.players[1], &[168]);
    case.claim(0, 0, wire::DART_KIND, r2, 300);
    case.players[0].packet(138, &dart_wire).unwrap();
    assert_eq!(
        receive(&mut case.players[1], &[172]).packets[0].1.payload,
        dart_wire
    );
}
