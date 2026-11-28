use kr_service::{
    account_serving::F4ExtendedPolicyBundle,
    f4_extended_policy::{
        ItemOrigin, KitDefinition, KitRoomRules, KitStats, OwnedItem, PolicyError,
        PowerupKitPolicy, PrimarySession, SecondPasswordAction as Action,
        SecondPasswordOutcome as Outcome, SecondPasswordPolicy,
    },
    f4_security_kits::{F4SecurityKitsRuntime, MAX_UNLOCKED_SESSIONS_PER_PLAYER},
};
use serde_json::{json, to_value};
use std::{sync::Barrier, thread};

const fn stats(bombs: u8, blast: u8, speed: u8) -> KitStats {
    KitStats {
        bombs,
        blast,
        speed,
    }
}

const BASE: KitStats = stats(1, 1, 1);
const ROOM: KitRoomRules = KitRoomRules {
    allows_kits: true,
    character_caps: stats(3, 8, 8),
    mode_caps: stats(8, 3, 8),
    map_caps: stats(8, 8, 4),
};

fn bundle() -> F4ExtendedPolicyBundle {
    F4ExtendedPolicyBundle {
        powerup_kits: PowerupKitPolicy {
            enabled: true,
            max_equipped_slots: 2,
            kits: vec![
                KitDefinition {
                    sku: "blast".into(),
                    bonus: stats(1, 4, 0),
                },
                KitDefinition {
                    sku: "speed".into(),
                    bonus: stats(0, 1, 5),
                },
                KitDefinition {
                    sku: "bombs".into(),
                    bonus: stats(8, 0, 0),
                },
            ],
        },
        second_password: SecondPasswordPolicy {
            enabled: true,
            max_failed_attempts: 2,
            failure_window_seconds: 60,
            cooldown_seconds: 60,
        },
        ..Default::default()
    }
}

fn item(owner: u64, sku: &str) -> OwnedItem<'_> {
    OwnedItem {
        owner,
        sku,
        quantity: 1,
        expires_at: None,
        origin: ItemOrigin::PersistentInventory,
    }
}

fn session(account: u64, session: u64) -> PrimarySession {
    PrimarySession { account, session }
}

#[test]
fn default_policies_reject_mutations_and_deny_protected_actions() {
    let runtime = F4SecurityKitsRuntime::default();
    let kits = &runtime.powerup_kits;
    let passwords = &runtime.second_password;
    let actor = session(1, 1);
    assert_eq!(
        kits.equip(1, 1, 0, item(1, "blast")),
        Err(PolicyError::Disabled)
    );
    assert_eq!(kits.clear(1, 0), Err(PolicyError::Disabled));
    assert_eq!(
        kits.starting_stats(1, 1, BASE, ROOM),
        Err(PolicyError::Disabled)
    );
    assert_eq!(passwords.setup(actor), Err(PolicyError::Disabled));
    assert_eq!(passwords.may_attempt(actor, 1), Err(PolicyError::Disabled));
    for action in [Action::Unlock, Action::Change, Action::Disable] {
        assert_eq!(
            passwords.attempt(actor, 1, action, true),
            Err(PolicyError::Disabled)
        );
    }
    assert_eq!(passwords.is_unlocked(actor), Ok(false));
    assert_eq!(passwords.revoke_session(actor), Err(PolicyError::Disabled));
    let status = runtime.snapshot().unwrap();
    assert!(!status.powerup_kits.enabled);
    assert!(!status.second_password.enabled);
    assert_eq!(status.powerup_kits.tracked_players, 0);
    assert_eq!(status.second_password.tracked_players, 0);
}

#[test]
fn kit_slots_validate_replacements_and_project_capped_stats() {
    let runtime = F4SecurityKitsRuntime::with_capacity(&bundle(), 2).unwrap();
    let kits = &runtime.powerup_kits;
    assert_eq!(kits.starting_stats(1, 10, BASE, ROOM), Ok(BASE));
    let expiring = OwnedItem {
        expires_at: Some(20),
        ..item(1, "blast")
    };
    kits.equip(1, 10, 1, expiring).unwrap();
    kits.equip(1, 10, 0, item(1, "speed")).unwrap();
    for _ in 0..2 {
        assert_eq!(kits.starting_stats(1, 19, BASE, ROOM), Ok(stats(2, 3, 4)));
    }
    assert_eq!(kits.starting_stats(2, 19, BASE, ROOM), Ok(BASE));
    assert_eq!(
        kits.starting_stats(
            1,
            19,
            BASE,
            KitRoomRules {
                allows_kits: false,
                ..ROOM
            }
        ),
        Ok(BASE)
    );
    assert_eq!(kits.starting_stats(1, 20, BASE, ROOM), Ok(stats(1, 2, 4)));
    assert_eq!(
        kits.starting_stats(1, 20, stats(4, 1, 1), ROOM),
        Err(PolicyError::InvalidInput)
    );

    for (kit, error) in [
        (item(1, "unknown"), PolicyError::NotAllowed),
        (item(2, "bombs"), PolicyError::NotOwnedOrExpired),
        (expiring, PolicyError::NotOwnedOrExpired),
        (
            OwnedItem {
                quantity: 0,
                ..item(1, "bombs")
            },
            PolicyError::NotOwnedOrExpired,
        ),
        (
            OwnedItem {
                origin: ItemOrigin::SessionEntitlement,
                ..item(1, "bombs")
            },
            PolicyError::NotOwnedOrExpired,
        ),
    ] {
        assert_eq!(kits.equip(1, 20, 0, kit), Err(error));
    }
    assert_eq!(
        kits.equip(1, 20, 1, item(1, "speed")),
        Err(PolicyError::AlreadyEquipped)
    );
    assert_eq!(
        kits.equip(1, 20, 2, item(1, "bombs")),
        Err(PolicyError::InvalidInput)
    );
    assert_eq!(kits.clear(1, 2), Err(PolicyError::InvalidInput));
    assert_eq!(kits.starting_stats(1, 20, BASE, ROOM), Ok(stats(1, 2, 4)));

    kits.equip(1, 20, 0, item(1, "bombs")).unwrap();
    assert_eq!(kits.starting_stats(1, 20, BASE, ROOM), Ok(stats(3, 1, 1)));
    kits.clear(1, 1).unwrap();
    kits.clear(1, 0).unwrap();
    kits.clear(1, 0).unwrap();
    assert_eq!(kits.starting_stats(1, 20, BASE, ROOM), Ok(BASE));
    assert_eq!(runtime.snapshot().unwrap().powerup_kits.tracked_players, 0);
}

#[test]
fn configured_slot_limit_narrows_the_fixed_loadout() {
    let mut bundle = bundle();
    bundle.powerup_kits.max_equipped_slots = 1;
    let runtime = F4SecurityKitsRuntime::with_capacity(&bundle, 1).unwrap();
    let kits = &runtime.powerup_kits;
    assert_eq!(
        kits.equip(1, 1, 1, item(1, "speed")),
        Err(PolicyError::InvalidInput)
    );
    assert_eq!(kits.clear(1, 1), Err(PolicyError::InvalidInput));
    kits.equip(1, 1, 0, item(1, "speed")).unwrap();
    kits.equip(1, 1, 0, item(1, "blast")).unwrap();
    assert_eq!(kits.starting_stats(1, 1, BASE, ROOM), Ok(stats(2, 3, 1)));
}

#[test]
fn concurrent_admission_stays_bounded_and_clear_or_disable_frees_capacity() {
    let runtime = F4SecurityKitsRuntime::with_capacity(&bundle(), 1).unwrap();
    let barrier = Barrier::new(8);
    let (kit_winners, password_winners) = thread::scope(|scope| {
        let threads: Vec<_> = (1..=8)
            .map(|account| {
                let (runtime, barrier) = (&runtime, &barrier);
                scope.spawn(move || {
                    barrier.wait();
                    let kits = runtime
                        .powerup_kits
                        .equip(account, 1, 0, item(account, "blast"));
                    let passwords = runtime.second_password.setup(session(account, account));
                    for result in [kits, passwords] {
                        assert!(matches!(
                            result,
                            Ok(()) | Err(PolicyError::CapacityExceeded)
                        ));
                    }
                    (
                        kits.is_ok().then_some(account),
                        passwords.is_ok().then_some(account),
                    )
                })
            })
            .collect();
        let (kits, passwords): (Vec<_>, Vec<_>) =
            threads.into_iter().map(|task| task.join().unwrap()).unzip();
        (
            kits.into_iter().flatten().collect::<Vec<_>>(),
            passwords.into_iter().flatten().collect::<Vec<_>>(),
        )
    });
    let ([kit_account], [password_account]) = (kit_winners.as_slice(), password_winners.as_slice())
    else {
        panic!("each runtime must admit exactly one player");
    };
    let status = runtime.snapshot().unwrap();
    assert_eq!(status.powerup_kits.tracked_players, 1);
    assert_eq!(status.second_password.tracked_players, 1);
    let kits = &runtime.powerup_kits;
    kits.equip(*kit_account, 1, 0, item(*kit_account, "speed"))
        .unwrap();
    kits.clear(*kit_account, 0).unwrap();
    kits.equip(9, 1, 0, item(9, "blast")).unwrap();
    let passwords = &runtime.second_password;
    let actor = session(*password_account, *password_account);
    passwords.revoke_session(actor).unwrap();
    assert_eq!(
        passwords.setup(session(9, 9)),
        Err(PolicyError::CapacityExceeded)
    );
    assert_eq!(
        passwords.attempt(actor, 1, Action::Disable, true),
        Ok(Outcome::Disabled)
    );
    passwords.setup(session(9, 9)).unwrap();
}

#[test]
fn second_password_lifecycle_keeps_lockout_across_sessions() {
    let runtime = F4SecurityKitsRuntime::with_capacity(&bundle(), 2).unwrap();
    let passwords = &runtime.second_password;
    let (first, second, third) = (session(1, 1), session(1, 2), session(1, 3));
    assert_eq!(passwords.is_unlocked(first), Ok(false));
    assert_eq!(passwords.setup(session(1, 0)), Err(PolicyError::NotAllowed));
    assert_eq!(
        passwords.attempt(first, 9, Action::Unlock, true),
        Err(PolicyError::NotAllowed)
    );
    passwords.setup(first).unwrap();
    assert_eq!(passwords.setup(second), Err(PolicyError::NotAllowed));
    assert_eq!(passwords.is_unlocked(first), Ok(false));
    assert_eq!(passwords.may_attempt(first, 10), Ok(true));
    assert_eq!(
        passwords.attempt(first, 10, Action::Unlock, false),
        Ok(Outcome::Incorrect)
    );
    assert_eq!(
        passwords.attempt(second, 11, Action::Change, false),
        Ok(Outcome::Locked)
    );
    passwords.revoke_session(second).unwrap();
    assert_eq!(passwords.may_attempt(first, 70), Ok(false));
    for action in [Action::Unlock, Action::Change, Action::Disable] {
        assert_eq!(
            passwords.attempt(first, 70, action, true),
            Ok(Outcome::Locked)
        );
    }
    assert_eq!(passwords.is_unlocked(first), Ok(false));

    assert_eq!(passwords.may_attempt(first, 71), Ok(true));
    assert_eq!(
        passwords.attempt(first, 71, Action::Unlock, true),
        Ok(Outcome::Unlocked)
    );
    assert_eq!(passwords.is_unlocked(first), Ok(true));
    assert_eq!(passwords.is_unlocked(second), Ok(false));
    assert_eq!(passwords.is_unlocked(session(2, 1)), Ok(false));
    assert_eq!(
        passwords.attempt(third, 72, Action::Unlock, true),
        Ok(Outcome::Unlocked)
    );
    assert_eq!(
        passwords.attempt(first, 73, Action::Change, true),
        Ok(Outcome::Changed)
    );
    assert_eq!(passwords.is_unlocked(first), Ok(false));
    assert_eq!(passwords.is_unlocked(third), Ok(false));

    assert_eq!(
        passwords.attempt(first, 74, Action::Unlock, true),
        Ok(Outcome::Unlocked)
    );
    passwords.revoke_session(first).unwrap();
    assert_eq!(passwords.is_unlocked(first), Ok(false));
    let next = session(1, 4);
    assert_eq!(passwords.is_unlocked(next), Ok(false));
    assert_eq!(
        passwords.attempt(next, 75, Action::Unlock, true),
        Ok(Outcome::Unlocked)
    );
    assert_eq!(
        passwords.attempt(next, 76, Action::Disable, true),
        Ok(Outcome::Disabled)
    );
    assert_eq!(passwords.is_unlocked(next), Ok(false));
    assert_eq!(
        passwords.may_attempt(next, 77),
        Err(PolicyError::NotAllowed)
    );
    passwords.setup(next).unwrap();
    assert_eq!(passwords.is_unlocked(next), Ok(false));
}

#[test]
fn second_password_bounds_unlocked_sessions_even_for_one_account() {
    let runtime = F4SecurityKitsRuntime::with_capacity(&bundle(), 1).unwrap();
    let passwords = &runtime.second_password;
    let first = session(1, 1);
    passwords.setup(first).unwrap();
    for id in 1..=MAX_UNLOCKED_SESSIONS_PER_PLAYER as u64 {
        assert_eq!(
            passwords.attempt(session(1, id), 1, Action::Unlock, true),
            Ok(Outcome::Unlocked)
        );
    }
    assert_eq!(
        passwords.attempt(first, 2, Action::Unlock, true),
        Ok(Outcome::Unlocked)
    );
    let extra = session(1, MAX_UNLOCKED_SESSIONS_PER_PLAYER as u64 + 1);
    assert_eq!(
        passwords.attempt(extra, 2, Action::Unlock, true),
        Err(PolicyError::CapacityExceeded)
    );
    assert_eq!(passwords.is_unlocked(extra), Ok(false));
    assert_eq!(
        passwords.attempt(first, 3, Action::Unlock, false),
        Ok(Outcome::Incorrect)
    );
    assert_eq!(passwords.is_unlocked(first), Ok(false));
    assert_eq!(
        passwords.attempt(extra, 4, Action::Unlock, true),
        Ok(Outcome::Unlocked)
    );
    passwords.revoke_session(extra).unwrap();
    passwords.revoke_session(extra).unwrap();
    assert_eq!(passwords.is_unlocked(session(1, 2)), Ok(true));
    assert_eq!(
        passwords.attempt(first, 5, Action::Unlock, true),
        Ok(Outcome::Unlocked)
    );
    assert_eq!(
        passwords.attempt(extra, 5, Action::Unlock, true),
        Err(PolicyError::CapacityExceeded)
    );
}

#[test]
fn snapshot_serializes_only_flags_counts_and_bounds() {
    let mut bundle = bundle();
    bundle.second_password.enabled = false;
    let runtime = F4SecurityKitsRuntime::with_capacity(&bundle, 3).unwrap();
    let empty = runtime.snapshot().unwrap();
    runtime
        .powerup_kits
        .equip(1, 1, 0, item(1, "blast"))
        .unwrap();
    assert_eq!(empty.powerup_kits.tracked_players, 0);
    assert_eq!(
        to_value(runtime.snapshot().unwrap()).unwrap(),
        json!({
            "powerup_kits": {"enabled": true, "tracked_players": 1, "max_players": 3},
            "second_password": {"enabled": false, "tracked_players": 0, "max_players": 3},
            "max_unlocked_sessions_per_player": MAX_UNLOCKED_SESSIONS_PER_PLAYER,
        })
    );
    bundle.second_password.enabled = true;
    let runtime = F4SecurityKitsRuntime::with_capacity(&bundle, 0).unwrap();
    assert_eq!(
        runtime.powerup_kits.equip(1, 1, 0, item(1, "blast")),
        Err(PolicyError::CapacityExceeded)
    );
    assert_eq!(
        runtime.second_password.setup(session(1, 1)),
        Err(PolicyError::CapacityExceeded)
    );
    bundle.powerup_kits.max_equipped_slots = 3;
    assert!(matches!(
        F4SecurityKitsRuntime::with_capacity(&bundle, 3),
        Err(PolicyError::InvalidConfig("kit slot limit"))
    ));
}
