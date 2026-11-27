use kr_service::f3_pve_policy::{F3PvePolicy, PveModeFamily};
use kr_service::f3_pve_runtime::{
    DispatchEvidence, F3PveRuntime, Limits, MonsterAttack, MonsterSpawn, Phase, RuntimeError,
};
use kr_wire::kr00967::f3_pve::F3JumpTableProvenance;

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
