use kr_service::f3_pve_policy::{F3PvePolicy, PveModeFamily};
use kr_service::f3_pve_runtime::{
    DispatchEvidence, F3PveRegistry, F3PveRuntime, Limits, MonsterAttack, MonsterSpawn, Phase,
    RuntimeError,
};
use kr_service::f3_pve_runtime::{
    MonActionBodyCandidate, inspect_f3_stage_jump, validate_mon_action_body,
};
use kr_wire::kr00967::f3_pve::F3JumpTableProvenance;
use kr_wire::kr00967::f3_pve::{
    C_GAME_STAGE_JUMP_ENTRIES, F3WireError, F3WireStatus, MonActionStep132Candidate,
    MonActionStep132Payload, MonActionStep180Candidate, MonActionStep180Payload,
};
use kr_wire::kr00967::f3_pve::{MonActionDelegationCandidate, MonActionStepReportCandidate};
use kr_wire::{CodecError, FRAME_OVERHEAD, MAX_FRAME_LEN};
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

// Static Mode2 vectors from kr-wire's #824 tests; these are not live captures.
const DELEGATION_204: [u8; 3] = [0xb7, 0x5d, 0x59];
const REPORT_132: [u8; 10] = [0x97, 0xc8, 0x50, 0x91, 0xb7, 0x5d, 0xa5, 0x65, 0x59, 0xa5];
const REPORT_180: [u8; 6] = [0xb7, 0x5d, 0xa5, 0x6f, 0x54, 0x48];

#[test]
fn stage_jump_inspection_preserves_recorded_entries_and_receive_direction() {
    assert!(C_GAME_STAGE_JUMP_ENTRIES.is_sorted_by(|a, b| a.wire_opcode < b.wire_opcode));
    for entry in C_GAME_STAGE_JUMP_ENTRIES {
        let inspection = inspect_f3_stage_jump(entry.wire_opcode).unwrap();
        assert_eq!(inspection.entry, entry);
        assert_eq!(inspection.provenance.on_packet_va, 0xE5D3A0);
        assert_eq!(inspection.provenance.jump_table_va, 0xE5DE9C);
        assert_eq!(inspection.provenance.opcode_index_subtrahend, 20);
        assert_eq!(
            inspection.provenance.status,
            F3WireStatus::StaticCandidatePendingCapture
        );
        assert_eq!(inspection.entry.case + 20, entry.wire_opcode);
    }
    for (opcode, case, selector, dispatch_va, handler_va) in [
        (29, 9, 53, 0xE5DCA9, None),
        (100, 80, 53, 0xE5DCA9, None),
        (132, 112, 6, 0xE5D691, Some(0xE610C0)),
        (152, 132, 11, 0xE5D4BC, Some(0xE5E640)),
        (180, 160, 34, 0xE5D610, Some(0xE72490)),
        (200, 180, 44, 0xE5D82B, Some(0xE683C0)),
        (204, 184, 53, 0xE5DCA9, None),
    ] {
        let entry = inspect_f3_stage_jump(opcode).unwrap().entry;
        assert_eq!(
            (
                entry.case,
                entry.table_selector,
                entry.dispatch_va,
                entry.handler_va
            ),
            (case, selector, dispatch_va, handler_va)
        );
    }
    assert_eq!(inspect_f3_stage_jump(20).unwrap().entry.case, 0);
    assert_eq!(inspect_f3_stage_jump(274).unwrap().entry.case, 254);
    for absent in [0, 19, 21, 203, 205, 275, u32::MAX] {
        assert_eq!(inspect_f3_stage_jump(absent), None);
    }
}

#[test]
fn mon_action_body_validation_keeps_all_three_layouts_capture_pending() {
    let vectors = [
        (
            204,
            DELEGATION_204.as_slice(),
            MonActionBodyCandidate::Delegation204(MonActionDelegationCandidate {
                opcode: 204,
                object_id: 0x1234,
                owner_slot: 3,
            }),
        ),
        (
            132,
            REPORT_132.as_slice(),
            MonActionBodyCandidate::StepReport132(MonActionStep132Candidate {
                session_or_owner: 0x0102_0304,
                object_id: 0x1234,
                payload: MonActionStep132Payload::Kind12 {
                    selector: 3,
                    value: 0xff,
                },
            }),
        ),
        (
            180,
            REPORT_180.as_slice(),
            MonActionBodyCandidate::StepReport180(MonActionStep180Candidate {
                object_id: 0x1234,
                payload: MonActionStep180Payload::Kind6 {
                    selector: 14,
                    value: 0x12,
                },
            }),
        ),
    ];
    for (opcode, body, expected) in vectors {
        let candidate = validate_mon_action_body(opcode, body).unwrap();
        assert_eq!(candidate, expected);
        let provenance = candidate.provenance();
        assert_eq!(provenance.owner_gate_va, 0xB91F60);
        assert_eq!(provenance.delegation_opcode, 204);
        assert_eq!(provenance.step_report_opcodes, [132, 180]);
        assert_eq!(
            provenance.status,
            F3WireStatus::StaticCandidatePendingCapture
        );
    }
}

#[test]
fn mon_action_body_validation_rejects_short_trailing_and_malformed_bodies() {
    for (opcode, body, object_offset) in [
        (204, DELEGATION_204.as_slice(), 0),
        (132, REPORT_132.as_slice(), 4),
        (180, REPORT_180.as_slice(), 0),
    ] {
        for end in 0..body.len() {
            assert_eq!(
                validate_mon_action_body(opcode, &body[..end]),
                Err(F3WireError::TruncatedEof)
            );
        }
        let mut trailing = body.to_vec();
        trailing.push(0x5a);
        assert_eq!(
            validate_mon_action_body(opcode, &trailing),
            Err(F3WireError::TrailingBytes)
        );

        let mut zero_object = body.to_vec();
        zero_object[object_offset..object_offset + 2].copy_from_slice(&[0xa5, 0x69]);
        assert_eq!(
            validate_mon_action_body(opcode, &zero_object),
            Err(F3WireError::ZeroMonActionObjectId)
        );
    }
    let mut bad_slot = DELEGATION_204;
    bad_slot[2] = 8 ^ 0x5a;
    assert_eq!(
        validate_mon_action_body(204, &bad_slot),
        Err(F3WireError::MonActionOwnerSlotOutOfRange(8))
    );
    for (opcode, mut body, selector_offset, invalid_selector) in [
        (132, REPORT_132.to_vec(), 8, 4),
        (180, REPORT_180.to_vec(), 4, 17),
    ] {
        body[selector_offset] = invalid_selector ^ 0x5a;
        assert_eq!(
            validate_mon_action_body(opcode, &body),
            Err(F3WireError::InvalidMonActionSelector {
                opcode,
                selector: invalid_selector
            })
        );
    }
    assert_eq!(
        validate_mon_action_body(132, &[0x97, 0xc8, 0x50, 0x91, 0xb7, 0x5d, 0xa5, 0x7d]),
        Err(F3WireError::UnsupportedMonActionKind {
            opcode: 132,
            kind: 20
        })
    );
    assert_eq!(
        validate_mon_action_body(180, &[0xb7, 0x5d, 0xa5, 0x60, 0x5a, 0x96, 0x5a]),
        Err(F3WireError::TruncatedEof)
    );
    let oversized = vec![0; MAX_FRAME_LEN - FRAME_OVERHEAD + 1];
    for opcode in [132, 180] {
        assert_eq!(
            validate_mon_action_body(opcode, &oversized),
            Err(F3WireError::Codec(CodecError::FrameTooLarge))
        );
    }
    for opcode in [152, 200, 205, u32::MAX] {
        assert_eq!(
            validate_mon_action_body(opcode, &REPORT_132),
            Err(F3WireError::InvalidMonActionOpcode(opcode))
        );
    }
}

#[test]
fn successful_static_inspection_does_not_supply_room_dispatch_evidence() {
    let MonActionBodyCandidate::Delegation204(delegation) =
        validate_mon_action_body(204, &DELEGATION_204).unwrap()
    else {
        panic!("204 did not produce the delegation candidate");
    };
    let report = MonActionStepReportCandidate::new(180, delegation.object_id, 0, 0).unwrap();
    let disabled = F3PveRuntime::new(F3PvePolicy::default(), FAMILY, FIXTURE_EVIDENCE);
    let evidence = DispatchEvidence::missing();
    assert_eq!(
        evidence.provenance,
        inspect_f3_stage_jump(204).unwrap().provenance
    );
    assert!(!evidence.is_present());
    let mut runtime = F3PveRuntime::new(ENABLED_POLICY, FAMILY, evidence);
    for (runtime, error) in [
        (&disabled, RuntimeError::DispatchDisabled),
        (&runtime, RuntimeError::WineDispatchEvidenceRequired),
    ] {
        assert_eq!(
            runtime.validate_mon_action_delegation(7, delegation),
            Err(error)
        );
        assert_eq!(
            runtime.validate_mon_action_step_report(7, report),
            Err(error)
        );
    }
    let required = RuntimeError::WineDispatchEvidenceRequired;
    assert_eq!(
        runtime.begin_briefing_with_slot(1, 7, Some(3), 11),
        Err(required)
    );
    assert_eq!(runtime.phase(), Phase::Idle);

    let registry = F3PveRegistry::new(ENABLED_POLICY, 1);
    assert_eq!(
        registry.start_room(1, FAMILY, evidence, 1, 7, 11),
        Err(required)
    );
    assert_eq!(
        registry.validate_mon_action_delegation(1, 7, delegation),
        Err(RuntimeError::InvalidTransition)
    );
    assert_eq!(registry.snapshot().active_rooms, 0);
}
