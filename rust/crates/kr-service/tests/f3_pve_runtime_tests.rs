use kr_service::f3_pve_policy::{F3PvePolicy, PveModeFamily};
use kr_service::f3_pve_runtime::{
    DispatchEvidence, F3PveRegistry, F3PveRuntime, Limits, MonsterAttack, MonsterSpawn, Phase,
    RuntimeError,
};
use kr_wire::kr00967::f3_pve::F3JumpTableProvenance;
use kr_wire::kr00967::f3_pve::{MonActionDelegationCandidate, MonActionStepReportCandidate};
use serde_json::json;
use std::{sync::Barrier, thread};

fn enabled() -> F3PveRuntime {
    F3PveRuntime::new(
        F3PvePolicy { enabled: true },
        PveModeFamily::MonsterS1,
        DispatchEvidence::present(F3JumpTableProvenance::C_GAME_STAGE),
    )
}

#[test]
fn disabled_default_policy_fails_closed_on_every_entry_point() {
    let mut runtime = F3PveRuntime::new(
        F3PvePolicy::default(),
        PveModeFamily::MonsterS1,
        DispatchEvidence::present(F3JumpTableProvenance::C_GAME_STAGE),
    );
    assert_eq!(
        runtime.begin_briefing(1, 7, 1),
        Err(RuntimeError::DispatchDisabled)
    );
    assert_eq!(
        runtime.start_wave(1, &[MonsterSpawn { id: 1, hp: 10 }]),
        Err(RuntimeError::DispatchDisabled)
    );
    assert_eq!(
        runtime.monster_attack(MonsterAttack {
            actor_id: 7,
            round_epoch: 1,
            attack_sequence: 1,
            monster_id: 1,
            damage: 1
        }),
        Err(RuntimeError::DispatchDisabled)
    );
    assert_eq!(runtime.clear_stage(), Err(RuntimeError::DispatchDisabled));
    assert_eq!(runtime.fail_stage(), Err(RuntimeError::DispatchDisabled));
    assert_eq!(runtime.abort(), Err(RuntimeError::DispatchDisabled));
    assert_eq!(runtime.phase(), Phase::Idle);
}

#[test]
fn opt_in_without_wine_dispatch_evidence_fails_closed() {
    let mut runtime = F3PveRuntime::new(
        F3PvePolicy { enabled: true },
        PveModeFamily::MonsterS1,
        DispatchEvidence::missing(),
    );
    assert_eq!(
        runtime.begin_briefing(1, 7, 1),
        Err(RuntimeError::WineDispatchEvidenceRequired)
    );
    assert_eq!(runtime.phase(), Phase::Idle);
}

#[test]
fn host_authority_rejects_non_host_without_hp_mutation() {
    let mut runtime = enabled();
    runtime.begin_briefing(1, 7, 1).unwrap();
    runtime
        .start_wave(1, &[MonsterSpawn { id: 9, hp: 20 }])
        .unwrap();
    let attack = MonsterAttack {
        actor_id: 8,
        round_epoch: 1,
        attack_sequence: 1,
        monster_id: 9,
        damage: 5,
    };
    assert_eq!(runtime.monster_attack(attack), Err(RuntimeError::NotHost));
    assert_eq!(runtime.monster_hp(9), Some(20));
}

#[test]
fn sequence_replay_and_unknown_monster_are_rejected() {
    let mut runtime = enabled();
    runtime.begin_briefing(1, 7, 1).unwrap();
    runtime
        .start_wave(1, &[MonsterSpawn { id: 9, hp: 20 }])
        .unwrap();
    let attack = MonsterAttack {
        actor_id: 7,
        round_epoch: 1,
        attack_sequence: 4,
        monster_id: 9,
        damage: 5,
    };
    runtime.monster_attack(attack).unwrap();
    assert_eq!(
        runtime.monster_attack(attack),
        Err(RuntimeError::NonIncreasingSequence)
    );
    assert_eq!(
        runtime.monster_attack(MonsterAttack {
            attack_sequence: 5,
            monster_id: 99,
            ..attack
        }),
        Err(RuntimeError::UnknownMonster)
    );
    assert_eq!(runtime.monster_hp(9), Some(15));
}

#[test]
fn valid_clear_and_failed_lifecycle() {
    let mut runtime = enabled();
    runtime.begin_briefing(2, 7, 10).unwrap();
    assert_eq!(runtime.phase(), Phase::Briefing { stage: 2 });
    runtime
        .start_wave(1, &[MonsterSpawn { id: 1, hp: 3 }])
        .unwrap();
    runtime
        .monster_attack(MonsterAttack {
            actor_id: 7,
            round_epoch: 10,
            attack_sequence: 1,
            monster_id: 1,
            damage: 3,
        })
        .unwrap();
    let result = runtime.clear_stage().unwrap();
    assert_eq!(result.stage, 2);
    assert_eq!(runtime.phase(), Phase::Cleared { stage: 2 });

    let mut failed = enabled();
    failed.begin_briefing(1, 7, 1).unwrap();
    failed
        .start_wave(1, &[MonsterSpawn { id: 2, hp: 3 }])
        .unwrap();
    failed.fail_stage().unwrap();
    assert_eq!(failed.phase(), Phase::Failed { stage: 1 });
}

#[test]
fn capacities_are_bounded() {
    assert_eq!(Limits::new(0, 1, 1), Err(RuntimeError::CapacityExceeded));
    assert_eq!(Limits::new(1, 1, 129), Err(RuntimeError::CapacityExceeded));
    let limits = Limits::new(1, 1, 2).unwrap();
    let mut runtime = F3PveRuntime::with_limits(
        F3PvePolicy { enabled: true },
        PveModeFamily::MonsterS1,
        DispatchEvidence::present(F3JumpTableProvenance::C_GAME_STAGE),
        limits,
    )
    .unwrap();
    runtime.begin_briefing(1, 7, 1).unwrap();
    assert_eq!(
        runtime.start_wave(
            1,
            &[
                MonsterSpawn { id: 1, hp: 1 },
                MonsterSpawn { id: 2, hp: 1 },
                MonsterSpawn { id: 3, hp: 1 }
            ]
        ),
        Err(RuntimeError::CapacityExceeded)
    );
    assert_eq!(runtime.phase(), Phase::Briefing { stage: 1 });
}

const FAMILY: PveModeFamily = PveModeFamily::MonsterS1;
const ENABLED_POLICY: F3PvePolicy = F3PvePolicy { enabled: true };
// Synthetic assertion for exercising session behavior; no live Wine evidence.
const FIXTURE_EVIDENCE: DispatchEvidence =
    DispatchEvidence::present(F3JumpTableProvenance::C_GAME_STAGE);
const WAVE: [MonsterSpawn; 1] = [MonsterSpawn { id: 1, hp: 10 }];
const ATTACK: MonsterAttack = MonsterAttack {
    actor_id: 7,
    round_epoch: 11,
    attack_sequence: 1,
    monster_id: 1,
    damage: 10,
};

#[test]
fn disabled_registries_fail_closed_on_all_room_operations() {
    for registry in [
        F3PveRegistry::default(),
        F3PveRegistry::new(F3PvePolicy::default(), 2),
        F3PveRegistry::new(ENABLED_POLICY, 0),
    ] {
        let disabled = RuntimeError::DispatchDisabled;
        assert_eq!(
            registry.start_room(1, FAMILY, FIXTURE_EVIDENCE, 1, 7, 11),
            Err(disabled)
        );
        assert_eq!(registry.advance_wave(1, 1, &WAVE), Err(disabled));
        assert_eq!(registry.apply_host_attack(1, ATTACK), Err(disabled));
        assert_eq!(registry.end_room(1), Err(disabled));
        assert_eq!(registry.room_status(1), Err(disabled));
        let snapshot = registry.snapshot();
        assert!(!snapshot.enabled);
        assert_eq!(snapshot.active_rooms, 0);
    }
}

#[test]
fn failed_starts_leave_capacity_free_and_end_room_releases_it() {
    let registry = F3PveRegistry::new(ENABLED_POLICY, 1);
    assert_eq!(
        registry.start_room(1, FAMILY, DispatchEvidence::missing(), 1, 7, 11),
        Err(RuntimeError::WineDispatchEvidenceRequired)
    );
    assert_eq!(
        registry.start_room(1, FAMILY, FIXTURE_EVIDENCE, 0, 7, 11),
        Err(RuntimeError::InvalidStage)
    );
    assert_eq!(registry.snapshot().active_rooms, 0);
    registry
        .start_room(1, FAMILY, FIXTURE_EVIDENCE, 1, 7, 11)
        .unwrap();
    let original = registry.room_status(1).unwrap();
    assert_eq!(original.phase, Phase::Briefing { stage: 1 });
    assert_eq!(
        registry.start_room(1, FAMILY, FIXTURE_EVIDENCE, 2, 8, 12),
        Err(RuntimeError::InvalidTransition)
    );
    assert_eq!(
        registry.start_room(2, FAMILY, FIXTURE_EVIDENCE, 1, 8, 12),
        Err(RuntimeError::CapacityExceeded)
    );
    assert_eq!(registry.room_status(1), Ok(original));

    registry.end_room(1).unwrap();
    let missing = RuntimeError::InvalidTransition;
    assert_eq!(registry.room_status(1), Err(missing));
    assert_eq!(registry.advance_wave(1, 1, &WAVE), Err(missing));
    assert_eq!(registry.apply_host_attack(1, ATTACK), Err(missing));
    assert_eq!(registry.end_room(1), Err(missing));
    assert_eq!(registry.snapshot().active_rooms, 0);
    registry
        .start_room(2, FAMILY, FIXTURE_EVIDENCE, 1, 8, 12)
        .unwrap();
    assert_eq!(registry.snapshot().active_rooms, 1);
}

#[test]
fn rooms_isolate_host_epoch_monsters_and_attack_sequences() {
    let registry = F3PveRegistry::new(ENABLED_POLICY, 2);
    registry
        .start_room(1, FAMILY, FIXTURE_EVIDENCE, 1, 7, 11)
        .unwrap();
    registry
        .start_room(2, PveModeFamily::Mission, FIXTURE_EVIDENCE, 2, 8, 12)
        .unwrap();
    registry.advance_wave(1, 1, &WAVE).unwrap();
    registry.advance_wave(2, 1, &WAVE).unwrap();
    assert_eq!(
        registry.apply_host_attack(2, ATTACK),
        Err(RuntimeError::NotHost)
    );
    let other_host = MonsterAttack {
        actor_id: 8,
        ..ATTACK
    };
    assert_eq!(
        registry.apply_host_attack(2, other_host),
        Err(RuntimeError::WrongRoundEpoch)
    );

    registry.apply_host_attack(1, ATTACK).unwrap();
    registry.advance_wave(1, 2, &WAVE).unwrap();
    assert_eq!(
        registry.apply_host_attack(1, ATTACK),
        Err(RuntimeError::NonIncreasingSequence)
    );
    assert_eq!(
        registry.advance_wave(2, 2, &WAVE),
        Err(RuntimeError::InvalidWave)
    );
    registry.end_room(1).unwrap();
    registry
        .apply_host_attack(
            2,
            MonsterAttack {
                round_epoch: 12,
                ..other_host
            },
        )
        .unwrap();
    registry.advance_wave(2, 2, &WAVE).unwrap();
    let status = registry.room_status(2).unwrap();
    assert_eq!(status.phase, Phase::Active { stage: 2, wave: 2 });
    assert_eq!(status.host_id, 8);
    assert_eq!(status.round_epoch, 12);
    assert_eq!(status.monster_count, 1);
}

#[test]
fn simultaneous_starts_cannot_overbook_the_registry() {
    const CONTENDERS: u32 = 8;
    const CAPACITY: usize = 2;
    let registry = F3PveRegistry::new(ENABLED_POLICY, CAPACITY);
    let barrier = Barrier::new(CONTENDERS as usize);
    thread::scope(|scope| {
        let starts: Vec<_> = (1..=CONTENDERS)
            .map(|room_id| {
                let registry = &registry;
                let barrier = &barrier;
                scope.spawn(move || {
                    barrier.wait();
                    registry.start_room(room_id, FAMILY, FIXTURE_EVIDENCE, 1, 7, 11)
                })
            })
            .collect();
        let accepted = starts
            .into_iter()
            .map(|start| match start.join().unwrap() {
                Ok(()) => 1,
                Err(RuntimeError::CapacityExceeded) => 0,
                unexpected => panic!("unexpected room start result: {unexpected:?}"),
            })
            .sum::<usize>();
        assert_eq!(accepted, CAPACITY);
    });
    assert_eq!(registry.snapshot().active_rooms, CAPACITY);
}

#[test]
fn snapshot_serializes_counts_and_only_static_dispatch_provenance() {
    let registry = F3PveRegistry::new(ENABLED_POLICY, 2);
    registry
        .start_room(1, FAMILY, FIXTURE_EVIDENCE, 1, 7, 11)
        .unwrap();
    assert_eq!(
        serde_json::to_value(registry.snapshot()).unwrap(),
        json!({
            "enabled": true,
            "active_rooms": 1,
            "max_rooms": 2,
            "dispatch": {
                "status": "static_candidate_pending_live_wine_capture",
                "jump_table_va": "0xE5DE9C",
                "opcode_index_subtrahend": 20,
            },
        })
    );
    registry.end_room(1).unwrap();
    let snapshot = registry.snapshot();
    assert!(snapshot.enabled);
    assert_eq!(snapshot.active_rooms, 0);
}

#[test]
fn stages_advance_only_after_a_clear_and_return_a_serializable_summary() {
    let mut runtime = F3PveRuntime::with_limits(
        ENABLED_POLICY,
        FAMILY,
        FIXTURE_EVIDENCE,
        Limits::new(2, 1, 2).unwrap(),
    )
    .unwrap();
    runtime.begin_briefing(1, 3, 21).unwrap();
    assert_eq!(
        runtime.advance_stage(2),
        Err(RuntimeError::InvalidTransition)
    );
    runtime
        .start_wave(1, &[MonsterSpawn { id: 42, hp: 2 }])
        .unwrap();
    runtime
        .monster_attack(MonsterAttack {
            actor_id: 3,
            round_epoch: 21,
            attack_sequence: 1,
            monster_id: 42,
            damage: 2,
        })
        .unwrap();
    let summary = runtime.clear_stage().unwrap();
    assert_eq!(summary.stage, 1);
    assert_eq!(serde_json::to_value(summary).unwrap()["defeated"], 1);
    assert_eq!(runtime.advance_stage(2), Ok(summary));
    assert_eq!(runtime.phase(), Phase::Briefing { stage: 2 });
    assert_eq!(
        runtime.advance_stage(3),
        Err(RuntimeError::InvalidTransition)
    );
}

#[test]
fn mon_action_validation_requires_active_host_slot_and_object() {
    let mut runtime = F3PveRuntime::new(ENABLED_POLICY, FAMILY, FIXTURE_EVIDENCE);
    runtime
        .begin_briefing_with_slot(1, 77, Some(2), 31)
        .unwrap();
    runtime
        .start_wave(1, &[MonsterSpawn { id: 55, hp: 10 }])
        .unwrap();
    let delegation = MonActionDelegationCandidate::new(204, 55, 2).unwrap();
    let report = MonActionStepReportCandidate::new(132, 55, 0, 0).unwrap();
    assert_eq!(
        runtime.validate_mon_action_delegation(77, delegation),
        Ok(())
    );
    assert_eq!(runtime.validate_mon_action_step_report(77, report), Ok(()));
    assert_eq!(
        runtime.validate_mon_action_delegation(
            76,
            MonActionDelegationCandidate::new(204, 55, 2).unwrap()
        ),
        Err(RuntimeError::MonActionHostMismatch)
    );
    assert_eq!(
        runtime.validate_mon_action_delegation(
            77,
            MonActionDelegationCandidate::new(204, 55, 1).unwrap()
        ),
        Err(RuntimeError::MonActionOwnerSlotMismatch)
    );
    assert_eq!(
        runtime.validate_mon_action_step_report(
            77,
            MonActionStepReportCandidate::new(132, 56, 0, 0).unwrap()
        ),
        Err(RuntimeError::MonActionObjectInactive)
    );
}

#[test]
fn registry_lists_rooms_in_id_order_and_respects_capacity() {
    let registry = F3PveRegistry::new(ENABLED_POLICY, 2);
    registry
        .start_room(9, FAMILY, FIXTURE_EVIDENCE, 1, 1, 41)
        .unwrap();
    registry
        .start_room(3, FAMILY, FIXTURE_EVIDENCE, 1, 2, 42)
        .unwrap();
    let listed = registry.list_active_rooms().unwrap();
    assert_eq!(listed.len(), 2);
    assert_eq!(listed[0].0, 3);
    assert_eq!(listed[1].0, 9);
    assert_eq!(listed[0].1.host_id, 2);
    assert_eq!(
        registry.start_room(12, FAMILY, FIXTURE_EVIDENCE, 1, 3, 43),
        Err(RuntimeError::CapacityExceeded)
    );
    registry.end_room(3).unwrap();
    assert_eq!(registry.list_active_rooms().unwrap().len(), 1);
}
