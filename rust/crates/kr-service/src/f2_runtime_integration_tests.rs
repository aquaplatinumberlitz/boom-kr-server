use crate::{
    operations::{F2Command, F2Event, F2Setup},
    room_io::{self, Session, Shared, registry_foundation_tests},
    room_registry::RoomKey,
    room_service::{Config, Hub},
};
use kr_wire::{Width, kr00967 as wire, write_uint};
use serde_json::{Value, json};
use std::{io, sync::Arc};

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
    let cfg: Config = serde_json::from_value(json!({
        "max_rooms": 2,
        "round_profile": {"mode": "teams", "teams": [0, 1, 0, 1], "initial_bomb_capacity": [1, 2, 1, 1]}
    }))
    .unwrap();
    Hub::new(
        &cfg,
        registry_foundation_tests::f2_runtime_prototype(f2),
        None,
        None,
    )
    .unwrap()
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
        let (key, room, host) = hub.select_fixture(81, &create(manner), 70000).unwrap();
        let mut clients = vec![host];
        clients.extend(
            [43, 501, 71]
                .into_iter()
                .map(|id| hub.select_fixture(82, &join(key), id).unwrap().2),
        );
        let mut result = Self {
            hub,
            key,
            room,
            clients,
            startup: vec![vec![]; 4],
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
        assert!(
            self.clients[0]
                .packet(104, &[0; 20])
                .unwrap()
                .start_rejection
                .is_none()
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
    assert!(
        result.unwrap_err().to_string().contains(reason),
        "expected {reason}"
    );
}

#[test]
fn default_room_and_unselected_override_keep_legacy_packets_and_reject_f2() {
    let mut normal = TestRoom::new(Value::Null);
    let mut other_selector = TestRoom::new(json!({"runtime_overrides": {"manner": "revive"}}));
    for room in [&mut normal, &mut other_selector] {
        let round = room.load();
        room.arm();
        rejects(room.start(round, revive(3)), "disabled");
        rejects(room.event(round, 1, defeat(1)), "disabled");
        assert!(!Session::handles(185));
        assert!(room.clients[0].packet(185, &[0x5b]).is_err());
        room.clients[0].tick().unwrap();
        room.no_packets();
    }
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
