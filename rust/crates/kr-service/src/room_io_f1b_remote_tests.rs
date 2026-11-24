//! Room packet tests for the o11 Sensor/Shield opt-in and RemoteBomb codec bounds.
//! Wires are synthetic from the o8 static writer layout; no Wine capture backs them.
use super::*;
use crate::room::wire_key;

const REMOTE_POLICY: &str = r#""report_derived_f1b_remote_sensor""#;
const AUX_POLICY: &str = r#""report_derived_f1b_aux""#;
const FLOOR: [u16; 3] = [wire::SENSOR_KIND, wire::SHIELD_KIND, wire::DART_KIND];

fn floor(kinds: &[u16]) -> Vec<u8> {
    let mut raw = vec![24];
    raw.extend((7 + 10 * kinds.len() as u32).to_le_bytes());
    raw.push(0);
    raw.extend((kinds.len() as u16).to_le_bytes());
    for (column, &kind) in kinds.iter().enumerate() {
        raw.extend((column as u32).to_le_bytes());
        raw.extend([12, column as u8]);
        raw.extend(u32::from(kind).to_le_bytes());
    }
    raw.extend([0; 4]);
    let mut body = kr_wire::write_uint(raw.len() as u64, Width::U32).unwrap();
    body.extend(wire::encode_bytes(&raw).unwrap());
    body
}

fn fields(values: &[(u64, Width)]) -> Vec<u8> {
    values
        .iter()
        .flat_map(|&(v, w)| kr_wire::write_uint(v, w).unwrap())
        .collect()
}

fn item_use_sub0(slot: u8, kind: u16, tick: u32) -> Vec<u8> {
    fields(&[
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
    ])
}

fn shield_sub5(slot: u8, tick: u32) -> Vec<u8> {
    fields(&[
        (u64::from(wire::SHIELD_SUBTYPE), Width::U8),
        (u64::from(slot), Width::U8),
        (80, Width::U16),
        (160, Width::U16),
        (u64::from(tick), Width::U32),
        (1, Width::U32),
    ])
}

fn report122(reason: u32, delay: u32) -> Vec<u8> {
    fields(&[
        (7, Width::U32),
        (11, Width::U16),
        (2, Width::U16),
        (u64::from(reason), Width::U32),
        (0, Width::U8),
        (u64::from(delay), Width::U32),
    ])
}

struct Case {
    shared: Shared,
    players: [Session; 2],
    keys: [Membership; 2],
}

impl Case {
    /// The floor is validated under the remote-sensor opt-in; `policy` is what the room runs.
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
        let body = floor(&FLOOR);
        r.floor_catalog = pickups::Catalog::from_floor(&body).unwrap();
        r.floor_catalog
            .validate_stat_policy(serde_json::from_str(REMOTE_POLICY).unwrap())
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

    fn claim_body(slot: usize, column: usize, kind: u16, round: u64, tick: u32) -> Vec<u8> {
        fields(&[
            (0, Width::U8),
            (slot as u64, Width::U8),
            (12, Width::U8),
            (column as u64, Width::U8),
            (column as u64, Width::U32),
            (u64::from(kind), Width::U16),
            (u64::from(tick), Width::U32),
            (u64::from(wire_key(round).unwrap()), Width::U32),
            (0, Width::U8),
        ])
    }

    fn claim(&mut self, column: usize, kind: u16, round: u64, tick: u32) {
        let body = Self::claim_body(0, column, kind, round, tick);
        let decision = self.players[0].packet(124, &body).unwrap().pickup.unwrap();
        assert!(decision.accepted);
        for s in &mut self.players {
            receive(s, &[165]);
        }
    }

    fn life_reason(&self, opcode: u32, body: &[u8]) -> &'static str {
        self.players[0]
            .packet(opcode, body)
            .unwrap()
            .life
            .unwrap()
            .reason
    }

    fn count(&self, kind: u16) -> u16 {
        self.shared
            .lock()
            .unwrap()
            .model
            .pickups
            .inventory_count(self.keys[0], kind)
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
fn sensor_and_shield_floor_need_the_new_opt_in_and_other_rows_stay_rejected() {
    let catalog = pickups::Catalog::from_floor(&floor(&[wire::SENSOR_KIND])).unwrap();
    let shield = pickups::Catalog::from_floor(&floor(&[wire::SHIELD_KIND])).unwrap();
    for name in [
        r#""disabled""#,
        r#""report_derived_basic""#,
        r#""report_derived_shoe_punch""#,
        r#""report_derived_f1b_extended""#,
        AUX_POLICY,
        r#""report_derived_f1d_vehicles""#,
        r#""report_derived_f1d_all_ships""#,
    ] {
        let policy: pickups::Policy = serde_json::from_str(name).unwrap();
        assert!(!policy.f1b_remote_sensor());
        assert!(!policy.allows(wire::SENSOR_KIND) && !policy.allows(wire::SHIELD_KIND));
        assert!(catalog.validate_stat_policy(policy).is_err());
        assert!(shield.validate_stat_policy(policy).is_err());
        for kind in [wire::SENSOR_KIND, wire::SHIELD_KIND] {
            let item = wire::item_use127(&item_use_sub0(0, kind, 1)).unwrap().kind;
            assert!(!policy.charges_item_use(item));
        }
    }
    let remote: pickups::Policy = serde_json::from_str(REMOTE_POLICY).unwrap();
    assert!(catalog.validate_stat_policy(remote).is_ok());
    assert!(shield.validate_stat_policy(remote).is_ok());
    // The new opt-in is a strict superset of the aux and extended slices.
    assert!(remote.f1b_aux() && remote.f1b_extended() && !remote.f1d_vehicles());
    for kind in [wire::SPRING_KIND, wire::BOND_KIND, wire::SCANNER_KIND, 3, 4] {
        assert!(remote.allows(kind));
    }
    assert!(!remote.allows(40));
    assert!(!remote.charges_item_use(wire::ItemUseKind::Needle));

    // Variants and the group 53 AutoShield have no opened contract: the measured floor
    // parser still refuses them; RemoteBomb 150 / Drill 149 need F1-b remote-drill opt-in.
    for kind in [wire::REMOTE_BOMB_KIND, wire::DRILL_KIND] {
        let cat = pickups::Catalog::from_floor(&floor(&[kind])).unwrap();
        assert!(cat.validate_stat_policy(remote).is_err());
        assert!(!remote.allows(kind));
    }
    for kind in [176, 258, 174, 188, 186, 183, 195] {
        assert!(pickups::Catalog::from_floor(&floor(&[kind])).is_err());
        assert!(!remote.allows(kind));
    }
}

#[test]
fn remote_detonation122_and_item_use_codec_bounds() {
    for delay in [0, 1, wire::REMOTE_DETONATION_MAX_DELAY] {
        let body = report122(wire::REMOTE_DETONATION_REASON, delay);
        assert_eq!(
            wire::remote_detonation122(&body).unwrap(),
            wire::RemoteDetonation122 {
                bomb_id: 7,
                row: 11,
                column: 2,
                slot: 0,
                delay,
            }
        );
    }
    for body in [
        report122(2, 201),
        report122(2, u32::MAX),
        report122(0, 0),
        report122(4, 0),
    ] {
        assert!(wire::remote_detonation122(&body).is_err());
    }
    let good = report122(2, 100);
    assert!(wire::remote_detonation122(&good[..16]).is_err());
    assert!(wire::remote_detonation122(&[good.as_slice(), &[0x5a]].concat()).is_err());

    let sensor = item_use_sub0(1, wire::SENSOR_KIND, 300);
    let decoded = wire::item_use127(&sensor).unwrap();
    assert_eq!((decoded.slot, decoded.tick), (1, 300));
    assert_eq!(decoded.kind, wire::ItemUseKind::Sensor);
    assert_eq!(
        wire::item_use127(&item_use_sub0(0, wire::SHIELD_KIND, 1))
            .unwrap()
            .kind,
        wire::ItemUseKind::Shield
    );
    // RemoteBomb/Drill now decode under sub0 (charged only under report_derived_f1b_remote_drill);
    // variant rows remain rejected.
    for (kind, expected) in [
        (wire::REMOTE_BOMB_KIND, wire::ItemUseKind::RemoteBomb),
        (wire::DRILL_KIND, wire::ItemUseKind::Drill),
    ] {
        let parsed = wire::item_use127(&item_use_sub0(0, kind, 1)).unwrap();
        assert_eq!(parsed.kind, expected);
    }
    for kind in [174, 186] {
        assert!(wire::item_use127(&item_use_sub0(0, kind, 1)).is_err());
    }
    for len in [0, 13, 21, 23] {
        let mut body = sensor.clone();
        body.resize(len, 0x5a);
        assert!(wire::item_use127(&body).is_err());
    }
    let shield = shield_sub5(0, 1);
    assert!(wire::shield127(&shield).is_ok());
    assert!(wire::shield127(&shield[..13]).is_err());
}

#[test]
fn sensor_consumes_six_charges_and_shield_relays_two_charges_then_resets() {
    let mut case = Case::new(REMOTE_POLICY);
    let r1 = case.start();
    case.arm();
    case.claim(0, wire::SENSOR_KIND, r1, 100);
    case.claim(1, wire::SHIELD_KIND, r1, 110);
    assert_eq!(case.count(wire::SENSOR_KIND), 6);
    assert_eq!(case.count(wire::SHIELD_KIND), 2);

    // Sensor: sub0 is the only report; it consumes and never reaches a peer.
    for step in 0..6 {
        let body = item_use_sub0(0, wire::SENSOR_KIND, 200 + step);
        assert_eq!(case.life_reason(127, &body), "accepted");
    }
    case.no_packets();
    assert_eq!(case.count(wire::SENSOR_KIND), 0);
    let empty = item_use_sub0(0, wire::SENSOR_KIND, 210);
    assert_eq!(case.life_reason(127, &empty), "scanner_no_charge");
    // A report from the wrong slot is refused before any charge is touched.
    let wrong = item_use_sub0(1, wire::SHIELD_KIND, 211);
    assert_eq!(case.life_reason(127, &wrong), "slot_mismatch");
    assert_eq!(case.count(wire::SHIELD_KIND), 2);

    // Shield: sub5 needs a charge, relays the 6-byte header, then sub0 consumes.
    for step in 0..2 {
        let body = shield_sub5(0, 300 + step);
        assert_eq!(case.life_reason(127, &body), "accepted");
        let peer = receive(&mut case.players[1], &[170]);
        assert_eq!(peer.packets[0].1.payload, body[..6]);
        let use_body = item_use_sub0(0, wire::SHIELD_KIND, 310 + step);
        assert_eq!(case.life_reason(127, &use_body), "accepted");
    }
    assert_eq!(
        case.life_reason(127, &shield_sub5(0, 320)),
        "shield_ineligible"
    );
    assert_eq!(case.life_reason(127, &shield_sub5(1, 321)), "slot_mismatch");
    assert_eq!(
        case.life_reason(127, &item_use_sub0(0, wire::SHIELD_KIND, 322)),
        "scanner_no_charge"
    );
    case.no_packets();

    // Existing aux items keep working under the superset policy.
    case.claim(2, wire::DART_KIND, r1, 400);
    let dart = [0x5a, 0x5a, 0xa5, 0xc9, 0xa7, 0x61, 0x5a, 0xa5, 0xc9];
    case.players[0].packet(138, &dart).unwrap();
    assert_eq!(
        receive(&mut case.players[1], &[172]).packets[0].1.payload,
        dart
    );

    // A new round clears the leftovers and charges.
    case.kill_peer();
    let r2 = case.start();
    assert_ne!(r1, r2);
    case.arm();
    assert_eq!(case.count(wire::DART_KIND), 0);
    assert_eq!(
        case.life_reason(127, &shield_sub5(0, 500)),
        "shield_ineligible"
    );
    case.no_packets();
    case.claim(0, wire::SENSOR_KIND, r2, 600);
    assert_eq!(
        case.life_reason(127, &item_use_sub0(0, wire::SENSOR_KIND, 600)),
        "scanner_tick_not_after_pickup"
    );
    assert_eq!(
        case.life_reason(127, &item_use_sub0(0, wire::SENSOR_KIND, 601)),
        "accepted"
    );
    assert_eq!(case.count(wire::SENSOR_KIND), 5);

    // Wrong payload length on a recognized subtype is a hard error.
    assert!(
        case.players[0]
            .packet(127, &shield_sub5(0, 700)[..13])
            .is_err()
    );
}

#[test]
fn aux_policy_keeps_sensor_and_shield_closed_and_its_legacy_shield_relay() {
    let mut case = Case::new(AUX_POLICY);
    let r1 = case.start();
    case.arm();
    // Sensor/Shield sub0 is not charged: it falls through to the life report path.
    for (step, kind) in [wire::SENSOR_KIND, wire::SHIELD_KIND]
        .into_iter()
        .enumerate()
    {
        let body = item_use_sub0(0, kind, 100 + step as u32);
        let reason = case.life_reason(127, &body);
        assert_eq!(reason, "unsupported_four_report_subtype");
    }
    // Legacy F1-b aux Shield relay is unchanged (no charge gate).
    let body = shield_sub5(0, 200);
    assert_eq!(case.life_reason(127, &body), "accepted");
    assert_eq!(
        receive(&mut case.players[1], &[170]).packets[0].1.payload,
        body[..6]
    );
    case.no_packets();
    // Pickup of a Sensor without the opt-in is refused outright.
    let claim = Case::claim_body(0, 0, wire::SENSOR_KIND, r1, 300);
    assert!(case.players[0].packet(124, &claim).is_err());
}
