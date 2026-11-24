//! Room packet tests with a synthetic floor/roster and the p97b dismount body.
use super::*;
use pickups::Policy;

const VEHICLES: [u16; 5] = [40, 41, 42, 43, 44];
const OWL: u16 = 41;
const TURTLE0: u16 = 42;
// p97b Cookie07 forward case, C2S 127 at A/370 and S2C 170 at B/163.
const DISMOUNT: [u8; 14] = [
    0x4a, 0x5a, 0xa5, 0xc9, 0xa7, 0x6e, 0x96, 0xca, 0x4f, 0x1c, 0x96, 0xca, 0x53, 0x94,
];
const PEER_DISMOUNT: [u8; 6] = [0x4a, 0x5a, 0xa5, 0xc9, 0xa7, 0x6e];

fn floor(kinds: &[u16], hidden: bool) -> Vec<u8> {
    let mut raw = vec![24];
    raw.extend((7 + 10 * kinds.len() as u32).to_le_bytes());
    raw.push(0);
    if hidden {
        raw.extend([0; 2]);
    }
    raw.extend((kinds.len() as u16).to_le_bytes());
    for (column, &kind) in kinds.iter().enumerate() {
        raw.extend(u32::from(kind).to_le_bytes());
        raw.extend([12, column as u8]);
        raw.extend(u32::from(kind).to_le_bytes());
    }
    raw.extend(if hidden { &[0; 2][..] } else { &[0; 4] });
    let mut body = kr_wire::write_uint(raw.len() as u64, Width::U32).unwrap();
    body.extend(wire::encode_bytes(&raw).unwrap());
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
            item_policy: serde_json::from_str(r#""report_derived_f1d_vehicles""#).unwrap(),
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
        let body = floor(&VEHICLES, false);
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
            assert!(catalog.visible_items().map(|item| item.3).eq(VEHICLES));
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

    fn claim(&mut self, slot: usize, kind: u16, round: u64) {
        let column = u64::from(kind - 40);
        let claim: Vec<_> = [
            (0, Width::U8),
            (slot as u64, Width::U8),
            (12, Width::U8),
            (column, Width::U8),
            (u64::from(kind), Width::U32),
            (u64::from(kind), Width::U16),
            (1000 + column, Width::U32),
            (u64::from(crate::room::wire_key(round).unwrap()), Width::U32),
            (0, Width::U8),
        ]
        .into_iter()
        .flat_map(|(v, w)| kr_wire::write_uint(v, w).unwrap())
        .collect();
        assert!(
            self.players[slot]
                .packet(124, &claim)
                .unwrap()
                .pickup
                .unwrap()
                .accepted
        );
        for s in &mut self.players {
            let batch = receive(s, &[165]);
            let body = &batch.packets[0].1.payload;
            assert_eq!(body.len(), 12);
            assert_eq!(read_uint(&body[..1], Width::U8).unwrap(), 1);
            assert_eq!(read_uint(&body[2..3], Width::U8).unwrap(), slot as u64);
            assert_eq!(
                read_uint(&body[9..11], Width::U16).unwrap(),
                u64::from(kind)
            );
        }
    }

    fn mounted(&self, slot: usize) -> Option<u16> {
        self.shared
            .lock()
            .unwrap()
            .model
            .pickups
            .mounted_vehicle(self.keys[slot])
    }

    fn dismount(&self, slot: usize, body: &[u8], reason: &str, forward: bool) {
        let decision = self.players[slot].packet(127, body).unwrap().life.unwrap();
        assert_eq!(decision.reason, reason);
        assert_eq!(decision.accepted, reason == "accepted");
        assert_eq!(decision.forward, forward);
        assert!(self.players.iter().all(Session::channel_current));
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
fn only_f1d_accepts_the_five_vehicle_kinds_in_preload_list1() {
    for kind in VEHICLES {
        let visible = pickups::Catalog::from_floor(&floor(&[kind], false)).unwrap();
        for policy in [
            Policy::Disabled,
            Policy::ReportDerivedBasic,
            Policy::ReportDerivedPunch,
            Policy::ReportDerivedShoePunch,
            Policy::ReportDerivedF1bExtended,
        ] {
            assert!(!policy.allows(kind));
            assert!(visible.validate_stat_policy(policy).is_err());
        }
        assert!(
            visible
                .validate_stat_policy(Policy::ReportDerivedF1dVehicles)
                .is_ok()
        );
        assert!(pickups::Catalog::from_floor(&floor(&[kind], true)).is_err());
    }
    for kind in [39, 45] {
        assert!(pickups::Catalog::from_floor(&floor(&[kind], false)).is_err());
    }
}

#[test]
fn pickups_mount_per_player_and_resolve_without_replacing_an_active_vehicle() {
    let mut c = Case::new();
    let round = c.start();
    c.arm();
    c.claim(0, OWL, round);
    assert_eq!(c.mounted(0), Some(OWL));
    assert_eq!(c.mounted(1), None);
    c.claim(0, TURTLE0, round);
    assert_eq!(c.mounted(0), Some(OWL));
    c.claim(1, 40, round);
    assert_eq!(c.mounted(1), Some(40));
    assert_eq!(c.mounted(0), Some(OWL));
    c.no_packets();
}

#[test]
fn dismount_checks_shape_and_sender_then_forwards_only_to_peers() {
    let mut c = Case::new();
    let round = c.start();
    c.arm();
    c.claim(0, OWL, round);
    c.dismount(1, &DISMOUNT, "slot_mismatch", false);
    assert!(c.players[0].packet(127, &DISMOUNT[..13]).is_err());
    assert!(
        c.players[0]
            .packet(127, &[DISMOUNT.as_slice(), &[0]].concat())
            .is_err()
    );
    assert_eq!(c.mounted(0), Some(OWL));
    c.no_packets();

    c.dismount(0, &DISMOUNT, "accepted", true);
    assert_eq!(c.mounted(0), None);
    assert_eq!(
        receive(&mut c.players[1], &[170]).packets[0].1.payload,
        PEER_DISMOUNT
    );
    c.no_packets();
    c.dismount(0, &DISMOUNT, "accepted", false);
    c.no_packets();
    c.claim(0, TURTLE0, round);
    assert_eq!(c.mounted(0), Some(TURTLE0));
}

#[test]
fn late_dismount_after_rank_keeps_sessions_alive_and_round_two_starts_unmounted() {
    let mut c = Case::new();
    let first = c.start();
    c.arm();
    c.claim(0, OWL, first);
    c.claim(1, 44, first);
    assert_eq!(c.mounted(1), Some(44));
    // Seed aged result witnesses so the actual 175 path can run without waiting.
    {
        let mut r = c.shared.lock().unwrap();
        let now = Instant::now();
        r.model
            .members
            .get_mut(&c.keys[0].identity)
            .unwrap()
            .life
            .death = Some(now - Duration::from_secs(9));
        r.model
            .members
            .get_mut(&c.keys[1].identity)
            .unwrap()
            .life
            .survivor = Some(now);
    }
    c.players[0].tick().unwrap();
    for s in &mut c.players {
        receive(s, &[175]);
    }
    let mut slot_one = DISMOUNT;
    slot_one[1] = 0x5b;
    c.dismount(1, &slot_one, "accepted", false);
    c.dismount(1, &DISMOUNT, "slot_mismatch", false);
    assert!(c.players[1].packet(127, &slot_one[..13]).is_err());
    for s in &mut c.players {
        receive(s, &[180]);
        receive(s, &[110, 110]);
    }
    c.no_packets();
    c.dismount(1, &slot_one, "accepted", false);
    c.no_packets();

    let second = c.start();
    assert_ne!(first, second);
    assert_eq!([c.mounted(0), c.mounted(1)], [None, None]);
    c.arm();
    c.claim(1, 43, second);
    assert_eq!(c.mounted(1), Some(43));
    assert_eq!(c.mounted(0), None);
}
