use kr_service::f3_pve_policy::{
    DispatchRejection, F3PvePolicy, MonsterAttackGuard, MonsterAttackRejection, PveAuthority,
    PveModeFamily,
};

#[test]
fn default_is_disabled_and_unknown_configuration_is_rejected() {
    let policy = F3PvePolicy::default();
    assert!(!policy.enabled);
    assert!(policy.authority_is_unmeasured(PveModeFamily::MonsterS1));
    for invalid in [
        "null",
        "[]",
        "[false]",
        r#"{"enable":true}"#,
        r#"{"enabled":null}"#,
        r#"{"enabled":false,"enabled":true}"#,
    ] {
        assert!(
            serde_json::from_str::<F3PvePolicy>(invalid).is_err(),
            "{invalid}"
        );
    }
}

#[test]
fn every_catalog_family_fails_closed_without_wine_dispatch_evidence() {
    let policy = F3PvePolicy { enabled: true };
    let families = [
        PveModeFamily::MonsterS1,
        PveModeFamily::MonsterS2,
        PveModeFamily::Mission,
        PveModeFamily::CAHero,
        PveModeFamily::HiddenCatch,
        PveModeFamily::AIBattle,
        PveModeFamily::Popcade,
        PveModeFamily::SandBox,
        PveModeFamily::IamBoss,
        PveModeFamily::IamWriggle,
        PveModeFamily::Tutorial,
        PveModeFamily::Antarctic,
        PveModeFamily::DragonPalace,
        PveModeFamily::EscapeMaze,
        PveModeFamily::GolemRun,
        PveModeFamily::MineExp,
        PveModeFamily::Minibot,
        PveModeFamily::SavetheEarth,
    ];
    for family in families {
        assert_eq!(policy.authority(family), PveAuthority::Unmeasured);
        assert_eq!(
            policy.ensure_dispatch_allowed(family),
            Err(DispatchRejection::WineEvidenceRequired(family))
        );
    }
}

#[test]
fn monster_attack_guard_rejects_invalid_requests_without_state_mutation() {
    let mut guard = MonsterAttackGuard::new(41);
    assert_eq!(
        guard.accept(7, 9, 41, 1),
        Err(MonsterAttackRejection::NotHost)
    );
    assert_eq!(
        guard.accept(9, 9, 40, 1),
        Err(MonsterAttackRejection::WrongRoundEpoch)
    );
    assert_eq!(guard.accept(9, 9, 41, 3), Ok(()));
    assert_eq!(
        guard.accept(9, 9, 41, 3),
        Err(MonsterAttackRejection::NonIncreasingAttackSequence)
    );
    assert_eq!(
        guard.accept(9, 9, 41, 2),
        Err(MonsterAttackRejection::NonIncreasingAttackSequence)
    );
    assert_eq!(guard.accept(9, 9, 41, 4), Ok(()));
    assert_eq!(
        guard.begin_round(41),
        Err(MonsterAttackRejection::NonIncreasingRoundEpoch)
    );
    assert_eq!(guard.begin_round(42), Ok(()));
    assert_eq!(guard.accept(9, 9, 42, 0), Ok(()));
}
