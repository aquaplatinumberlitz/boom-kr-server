//! Synthetic room integration controls; captured bodies do not make these native runs.
use super::*;
use crate::{
    life_actions::Policy,
    rules::{LifePolicy, RoundRoster, SlotAllocation},
};
fn bytes(s: &str) -> Vec<u8> {
    s.as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|x| u8::from_str_radix(std::str::from_utf8(x).unwrap(), 16).unwrap())
        .collect()
}
const TRAP: &str = "4b5aa511a46496ca59fb96ca539096ca539b96ca5a8d96c8539596ca5395a569";
const ONE: &str = "5b5aa511a46496ca593596ca5390";
const ZERO: &str = "5a5aa511a46496ca593596ca53935ba5f85a96ca5395";
const RESCUE: &str = "485aa511a5f696ca591c96ca539096ca539f96ca568d96cb5395a569";
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
            .choose(key, None, Some(if index < 2 { 0 } else { 3 }))
            .unwrap();
        keys.push(key);
        let (sender, receiver) = mpsc::channel(8);
        receivers.push(receiver);
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
    for key in &keys[1..] {
        model.ready(*key).unwrap();
    }
    let round = model.begin(keys[0]).unwrap();
    for key in &keys {
        model.ack(*key, round).unwrap();
    }
    let rules = RoomRules {
        life_actions: Policy::NeedleRescue,
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
            characters: None,
            bodies: HashMap::new(),
            initial_bomb_capacity: Vec::new(),
            rules,
            limits: rules::ReportLimits {
                power: Vec::new(),
                opaque167: &[4, 12, 16, 22, 32, 42],
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
        floor_catalog: pickups::Catalog::default(),
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
fn trap_and_credit(c: &mut Case) {
    assert!(
        c.sessions[0]
            .packet(127, &bytes(TRAP))
            .unwrap()
            .life
            .unwrap()
            .accepted
    );
    for s in &mut c.sessions[1..] {
        let batch = s.inbox.try_recv().unwrap();
        assert!(s.valid(&batch));
        s.finish(&batch).unwrap();
    }
    let mut r = c.shared.lock().unwrap();
    let p = &mut r.model.members.get_mut(&c.keys[0].identity).unwrap().life;
    p.stage_needle_credit("synthetic-completed165".into(), &c.keys)
        .unwrap();
    for key in &c.keys {
        p.needle_credit_delivered("synthetic-completed165", *key);
    }
}
#[test]
fn captured_needle167_keeps_transport_but_has_no_ack_or_life_authority_and_is_bounded() {
    let mut c = case(3);
    let body = bytes("a569a569a5f8a568");
    let before = c.shared.lock().unwrap().model.members[&c.keys[0].identity]
        .life
        .clone();
    for _ in 0..64 {
        c.sessions[0].packet(167, &body).unwrap();
    }
    assert!(c.sessions[0].packet(167, &body).is_err());
    assert!(c.sessions.iter_mut().all(|s| s.inbox.try_recv().is_err()));
    let r = c.shared.lock().unwrap();
    let p = &r.model.members[&c.keys[0].identity].life;
    assert_eq!(p.trap.is_some(), before.trap.is_some());
    assert!(!p.delivery_pending());
    drop(r);
    let disabled = case(3);
    disabled.shared.lock().unwrap().rules.life_actions = Policy::Disabled;
    assert!(disabled.sessions[0].packet(167, &body).is_err());
}
#[test]
fn needle_broadcast_rolls_back_on_full_queue_and_commits_after_all_recipients() {
    for count in [3, 8] {
        let mut c = case(count);
        trap_and_credit(&mut c);
        assert!(
            c.sessions[0]
                .packet(127, &bytes(ONE))
                .unwrap()
                .life
                .unwrap()
                .accepted
        );
        {
            let r = c.shared.lock().unwrap();
            let key = c.keys[count - 1];
            for _ in 0..8 {
                r.channels[&key.identity]
                    .sender
                    .try_send(Runtime::member_batch(key, Some(c.round), 0, vec![]))
                    .unwrap_or_else(|_| panic!("fill test queue"));
            }
        }
        assert!(c.sessions[0].packet(127, &bytes(ZERO)).is_err());
        assert!(c.sessions[1].inbox.try_recv().is_err());
        for _ in 0..8 {
            c.sessions[count - 1].inbox.try_recv().unwrap();
        }
        assert!(
            c.sessions[0]
                .packet(127, &bytes(ZERO))
                .unwrap()
                .life
                .unwrap()
                .forward
        );
        for index in 1..count {
            assert!(
                c.shared.lock().unwrap().model.members[&c.keys[0].identity]
                    .life
                    .trap
                    .is_some()
            );
            let s = &mut c.sessions[index];
            let batch = s.inbox.try_recv().unwrap();
            assert!(s.valid(&batch));
            assert_eq!(batch.packets[0].1.payload, bytes(ONE)[..6]);
            s.finish(&batch).unwrap();
        }
        assert!(
            c.shared.lock().unwrap().model.members[&c.keys[0].identity]
                .life
                .trap
                .is_none()
        );
        assert!(
            !c.sessions[0]
                .packet(127, &bytes(ZERO))
                .unwrap()
                .life
                .unwrap()
                .accepted
        );
    }
}
#[test]
fn rescue_requires_current_friendly_living_slot_and_old_clear_cannot_cross_round() {
    let mut c = case(3);
    trap_and_credit(&mut c);
    {
        c.shared
            .lock()
            .unwrap()
            .model
            .members
            .get_mut(&c.keys[1].identity)
            .unwrap()
            .team = 3;
    }
    assert_eq!(
        c.sessions[0]
            .packet(127, &bytes(RESCUE))
            .unwrap()
            .life
            .unwrap()
            .reason,
        "rescuer_not_current_friendly_eligible"
    );
    c.shared
        .lock()
        .unwrap()
        .model
        .members
        .get_mut(&c.keys[1].identity)
        .unwrap()
        .team = 0;
    assert!(
        c.sessions[0]
            .packet(127, &bytes(RESCUE))
            .unwrap()
            .life
            .unwrap()
            .forward
    );
    let first = c.sessions[1].inbox.try_recv().unwrap();
    let last = c.sessions[2].inbox.try_recv().unwrap();
    assert!(c.sessions[1].valid(&first));
    c.sessions[1].finish(&first).unwrap();
    assert!(
        c.shared.lock().unwrap().model.members[&c.keys[0].identity]
            .life
            .trap
            .is_some()
    );
    // Internal phase transition simulates reset while a recipient is backpressured.
    {
        let mut r = c.shared.lock().unwrap();
        r.model.phase = Phase::Armed { round: c.round + 1 };
        r.model
            .members
            .get_mut(&c.keys[0].identity)
            .unwrap()
            .life
            .begin();
    }
    assert!(!c.sessions[2].valid(&last));
    c.sessions[2].finish(&last).unwrap();
    assert!(
        !c.shared.lock().unwrap().model.members[&c.keys[0].identity]
            .life
            .delivery_pending()
    );
}
