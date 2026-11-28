use kr_service::{
    accounts::{Admin, ECONOMY_AMOUNT_MAX, Error, ItemGrant, Player, Store},
    f6_policy::{F6EventPolicy, QuotaRejection},
    f6_runtime::{ClaimError, ClaimOutcome, F6Claim, F6EventRuntime, F6Reward},
};
use rusqlite::Connection;
use serde_json::{Value, json};
use std::{
    fs,
    os::unix::fs::DirBuilderExt,
    path::PathBuf,
    sync::{
        Mutex,
        atomic::{AtomicU64, Ordering},
    },
    thread,
    time::Duration,
};

const DISABLED: &str = include_str!("../../../../server/profiles/f6-events-disabled.json");
const START: i64 = 1_798_761_600; // 2027-01-01T00:00:00Z
const DAY: i64 = 86_400;
const END: i64 = START + 3 * DAY;
const CORE: &str = "CAPass";
const EXTENDED: &str = "attendance";
const LUCCI: F6Reward<'static> = F6Reward::Lucci { amount: 25 };
const ITEM: F6Reward<'static> = F6Reward::Item {
    sku_id: "event-chip",
    grant: ItemGrant::Stackable { max_stack: 10 },
    quantity: 2,
};
static NEXT: AtomicU64 = AtomicU64::new(0);
static CREATE_ACCOUNT_GATE: Mutex<()> = Mutex::new(());

struct Temp(PathBuf);

impl Drop for Temp {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

struct Fixture {
    store: Store,
    observer: Connection,
    player: Player,
    dir: Temp,
}

impl Fixture {
    fn new() -> Self {
        let dir = Temp(std::env::temp_dir().join(format!(
            "kr-f6-runtime-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        )));
        fs::DirBuilder::new().mode(0o700).create(&dir.0).unwrap();
        let path = dir.0.join("accounts.sqlite");
        let mut store = Store::open(&path).unwrap();
        let player = create_account(&mut store, "Alice");
        let observer = Connection::open(path).unwrap();
        Self {
            store,
            observer,
            player,
            dir,
        }
    }

    fn runtime(&self) -> F6EventRuntime<'_> {
        F6EventRuntime::new(&self.store, opted_in_policy()).unwrap()
    }

    // On the observing connection this changes whenever another connection
    // commits a write, including a clock update without a reward.
    fn version(&self) -> i64 {
        self.observer
            .pragma_query_value(None, "data_version", |row| row.get(0))
            .unwrap()
    }
}

fn create_account(store: &mut Store, name: &str) -> Player {
    let _gate = CREATE_ACCOUNT_GATE
        .lock()
        .unwrap_or_else(|poison| poison.into_inner());
    let admin = Admin {
        actor: "f6-runtime-tests",
        reason: "create fixture account",
    };
    for _ in 0..100 {
        match store.create_account(&admin, name, name, b"f6-test-password", START - 1) {
            Ok(player) => return player,
            Err(Error::Busy) => thread::sleep(Duration::from_millis(10)),
            Err(error) => panic!("create account: {error:?}"),
        }
    }
    panic!("account creation stayed busy");
}

fn opted_in_policy() -> F6EventPolicy {
    let mut profile: Value = serde_json::from_str(DISABLED).unwrap();
    profile["systems"][CORE] = json!({
        "enabled": true,
        "start_utc": "2027-01-01T00:00:00Z",
        "end_utc": "2027-01-04T00:00:00Z",
        "rate_caps": {"per_account_per_hour": 1, "global_per_minute": 1}
    });
    profile["systems"]["Câu cá"] = profile["systems"][CORE].clone();
    let disabled = json!({
        "enabled": false,
        "start_utc": null,
        "end_utc": null,
        "rate_caps": {"per_account_per_hour": 0, "global_per_minute": 0},
        "max_grants_per_day": 0,
        "max_grants_per_season": 0
    });
    profile["extended_systems"] = F6EventPolicy::extended_system_names()
        .iter()
        .map(|&name| (name.to_owned(), disabled.clone()))
        .collect::<serde_json::Map<_, _>>()
        .into();
    profile["extended_systems"][EXTENDED] = json!({
        "enabled": true,
        "start_utc": "2027-01-01T00:00:00Z",
        "end_utc": "2027-01-04T00:00:00Z",
        "rate_caps": {"per_account_per_hour": 10, "global_per_minute": 10},
        "season_key": "winter-2027",
        "max_grants_per_day": 1,
        "max_grants_per_season": 2
    });
    F6EventPolicy::from_json_str(&profile.to_string()).unwrap()
}

fn claim<'a>(key: &'a str, system: &'a str, player: &Player, reward: F6Reward<'a>) -> F6Claim<'a> {
    F6Claim {
        key,
        system,
        player_id: player.player_id,
        reward,
    }
}

#[test]
fn default_policy_rejects_all_44_systems_without_writes() {
    let f = Fixture::new();
    let before = f.version();
    let mut runtime = F6EventRuntime::new(&f.store, F6EventPolicy::disabled_default()).unwrap();
    for &system in F6EventPolicy::system_names()
        .iter()
        .chain(F6EventPolicy::extended_system_names())
    {
        assert_eq!(
            runtime.claim(claim("disabled", system, &f.player, LUCCI), START),
            Err(ClaimError::Policy(QuotaRejection::Disabled)),
            "{system}"
        );
    }
    assert_eq!(
        runtime.claim(claim("unknown", "unlisted", &f.player, ITEM), START),
        Err(ClaimError::Policy(QuotaRejection::UnknownSystem))
    );
    assert_eq!(f.version(), before);
}

#[test]
fn status_snapshot_reports_disabled_defaults_and_policy_windows() {
    let f = Fixture::new();
    let disabled = F6EventRuntime::new(&f.store, F6EventPolicy::disabled_default()).unwrap();
    assert_eq!(
        disabled.status_snapshot(START),
        kr_service::f6_runtime::F6EventStatusSnapshot {
            enabled: false,
            configured_systems: Vec::new(),
            active_systems: Vec::new(),
            quota_usage: kr_service::f6_runtime::F6QuotaUsage {
                claims: 0,
                hourly: 0,
                minute: 0,
                daily: 0,
                seasonal: 0,
            },
            wire_status: "policy_runtime_only_pending_client_capture",
        }
    );

    let mut runtime = f.runtime();
    let outside = runtime.status_snapshot(START - 1);
    assert!(outside.enabled);
    assert_eq!(outside.configured_systems, vec![CORE, "Câu cá", EXTENDED]);
    assert!(outside.active_systems.is_empty());
    assert_eq!(
        runtime.claim(claim("snapshot", EXTENDED, &f.player, ITEM), START),
        Ok(ClaimOutcome::Granted)
    );
    let inside = runtime.status_snapshot(START);
    assert_eq!(inside.active_systems, vec![CORE, "Câu cá", EXTENDED]);
    assert_eq!(inside.quota_usage.claims, 1);
    assert_eq!(inside.quota_usage.hourly, 1);
    assert_eq!(inside.quota_usage.minute, 1);
    assert_eq!(inside.quota_usage.daily, 1);
    assert_eq!(inside.quota_usage.seasonal, 1);
    assert_eq!(
        serde_json::to_value(inside).unwrap(),
        json!({
            "enabled": true,
            "configured_systems": [CORE, "Câu cá", EXTENDED],
            "active_systems": [CORE, "Câu cá", EXTENDED],
            "quota_usage": {
                "claims": 1,
                "hourly": 1,
                "minute": 1,
                "daily": 1,
                "seasonal": 1
            },
            "wire_status": "policy_runtime_only_pending_client_capture"
        })
    );
}

#[test]
fn status_snapshot_tracks_extended_reset_and_replay_without_double_charge() {
    let f = Fixture::new();
    let mut runtime = f.runtime();
    let first = claim("snapshot-day-1", EXTENDED, &f.player, ITEM);
    assert_eq!(runtime.claim(first, START), Ok(ClaimOutcome::Granted));
    assert_eq!(
        runtime.claim(first, START + DAY),
        Ok(ClaimOutcome::Replayed)
    );
    let replay = runtime.status_snapshot(START + DAY);
    assert_eq!(replay.quota_usage.claims, 1);
    assert_eq!(replay.quota_usage.daily, 0);
    assert_eq!(replay.quota_usage.seasonal, 1);

    let second = claim("snapshot-day-2", EXTENDED, &f.player, ITEM);
    assert_eq!(
        runtime.claim(second, START + DAY),
        Ok(ClaimOutcome::Granted)
    );
    let reset = runtime.status_snapshot(START + DAY);
    assert_eq!(reset.quota_usage.claims, 2);
    assert_eq!(reset.quota_usage.daily, 1);
    assert_eq!(reset.quota_usage.seasonal, 2);
    assert_eq!(
        runtime.claim(
            F6Claim {
                reward: LUCCI,
                ..first
            },
            START + DAY
        ),
        Err(ClaimError::Conflict)
    );
}

#[test]
fn pc_room_is_excluded_even_with_other_systems_opted_in() {
    let f = Fixture::new();
    let mut runtime = f.runtime();
    let before = f.version();
    for system in ["PCRoomBenefit", "pcroombenefit", "PCROOMBENEFIT"] {
        assert_eq!(
            runtime.claim(claim("pc-room", system, &f.player, ITEM), START),
            Err(ClaimError::Policy(QuotaRejection::ExcludedSystem))
        );
    }
    assert_eq!(f.version(), before);
}

#[test]
fn uses_utc_windows_and_rechecks_schedule_on_replay() {
    let f = Fixture::new();
    let mut runtime = f.runtime();
    let before = f.version();
    let core = claim("core", CORE, &f.player, LUCCI);
    let extended = claim("extended", EXTENDED, &f.player, ITEM);
    let outside = Err(ClaimError::Policy(QuotaRejection::OutsideSchedule));
    for (request, now) in [
        (core, START - 1),
        (extended, START - 1),
        (core, END + 1),
        (extended, END),
    ] {
        assert_eq!(runtime.claim(request, now), outside);
    }
    assert_eq!(f.version(), before);

    assert_eq!(runtime.claim(extended, END - 1), Ok(ClaimOutcome::Granted));
    assert_eq!(runtime.claim(core, END), Ok(ClaimOutcome::Granted));
    let before = f.version();
    assert_eq!(runtime.claim(core, END + 1), outside);
    assert_eq!(runtime.claim(extended, END), outside);
    assert_eq!(f.version(), before);
}

#[test]
fn core_lucci_grants_once_and_enforces_both_rate_caps() {
    let mut f = Fixture::new();
    let bob = create_account(&mut f.store, "Bobby");
    let mut runtime = f.runtime();
    let alice_claim = claim("match/1/đủ", "Câu cá", &f.player, LUCCI);
    let bob_claim = claim("match/2", "Câu cá", &bob, LUCCI);
    let next_claim = claim("match/3", "Câu cá", &f.player, LUCCI);

    assert_eq!(runtime.claim(alice_claim, START), Ok(ClaimOutcome::Granted));
    let before = f.version();
    assert_eq!(
        runtime.claim(alice_claim, START),
        Ok(ClaimOutcome::Replayed)
    );
    assert_eq!(
        runtime.claim(bob_claim, START),
        Err(ClaimError::Policy(QuotaRejection::GlobalQuotaExceeded))
    );
    assert_eq!(
        runtime.claim(next_claim, START + 60),
        Err(ClaimError::Policy(QuotaRejection::AccountQuotaExceeded))
    );
    assert_eq!(f.version(), before);
    assert_eq!(
        runtime.claim(bob_claim, START + 60),
        Ok(ClaimOutcome::Granted)
    );
    let ledger = f.store.list_ledger(f.player.player_id, 10).unwrap();
    assert_eq!(ledger.len(), 1);
    assert_eq!(ledger[0].lucci_delta, 25);
    assert!(!ledger[0].idempotency_key.contains('/'));
    assert_eq!(f.store.get_profile(f.player.player_id).unwrap().lucci, 25);
    assert_eq!(f.store.get_profile(bob.player_id).unwrap().lucci, 25);
    assert_eq!(
        runtime.claim(next_claim, START + 3_600),
        Ok(ClaimOutcome::Granted)
    );
}

#[test]
fn extended_item_grants_once_and_enforces_daily_and_season_caps() {
    let f = Fixture::new();
    let mut runtime = f.runtime();
    let first = claim("day-1", EXTENDED, &f.player, ITEM);
    let second = claim("day-2", EXTENDED, &f.player, ITEM);
    let third = claim("day-3", EXTENDED, &f.player, ITEM);
    assert_eq!(runtime.claim(first, START), Ok(ClaimOutcome::Granted));
    let before = f.version();
    assert_eq!(
        runtime.claim(second, START + 100),
        Err(ClaimError::Policy(
            QuotaRejection::ExtendedDailyQuotaExceeded
        ))
    );
    // A replay after midnight must not consume the new day's grant.
    assert_eq!(
        runtime.claim(first, START + DAY),
        Ok(ClaimOutcome::Replayed)
    );
    assert_eq!(f.version(), before);
    assert_eq!(
        runtime.claim(second, START + DAY),
        Ok(ClaimOutcome::Granted)
    );

    let before = f.version();
    assert_eq!(
        runtime.claim(third, START + 2 * DAY),
        Err(ClaimError::Policy(
            QuotaRejection::ExtendedSeasonQuotaExceeded
        ))
    );
    assert_eq!(f.version(), before);
    let items = f.store.list_inventory(f.player.player_id, START).unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].quantity, 4);
    assert_eq!(
        f.store.list_ledger(f.player.player_id, 10).unwrap().len(),
        2
    );
}

#[test]
fn invalid_reward_or_key_never_writes_or_consumes_quota() {
    let f = Fixture::new();
    let mut runtime = f.runtime();
    let before = f.version();
    let base = claim("valid", CORE, &f.player, LUCCI);
    for reward in [
        F6Reward::Lucci { amount: 0 },
        F6Reward::Lucci { amount: -1 },
        F6Reward::Lucci {
            amount: ECONOMY_AMOUNT_MAX as i64 + 1,
        },
        F6Reward::Item {
            sku_id: "invalid/sku",
            grant: ItemGrant::Permanent,
            quantity: 1,
        },
        F6Reward::Item {
            sku_id: "permanent",
            grant: ItemGrant::Permanent,
            quantity: 2,
        },
        F6Reward::Item {
            sku_id: "stackable",
            grant: ItemGrant::Stackable { max_stack: 1 },
            quantity: 0,
        },
        F6Reward::Item {
            sku_id: "stackable",
            grant: ItemGrant::Stackable { max_stack: 1 },
            quantity: 2,
        },
        F6Reward::Item {
            sku_id: "timed",
            grant: ItemGrant::Timed {
                duration_seconds: 0,
            },
            quantity: 1,
        },
    ] {
        assert_eq!(
            runtime.claim(F6Claim { reward, ..base }, START),
            Err(ClaimError::Accounts(Error::InvalidInput))
        );
    }
    assert_eq!(
        runtime.claim(F6Claim { key: "", ..base }, START),
        Err(ClaimError::InvalidKey)
    );
    assert_eq!(f.version(), before);
    assert_eq!(runtime.claim(base, START), Ok(ClaimOutcome::Granted));
}

#[test]
fn claim_key_binds_player_system_and_reward_across_runtime_restart() {
    let mut f = Fixture::new();
    let bob = create_account(&mut f.store, "Bobby");
    let original = claim("one-logical-claim", CORE, &f.player, LUCCI);
    let item_claim = claim("another-logical-claim", EXTENDED, &f.player, ITEM);
    {
        let mut runtime = f.runtime();
        assert_eq!(runtime.claim(original, START), Ok(ClaimOutcome::Granted));
        assert_eq!(runtime.claim(item_claim, START), Ok(ClaimOutcome::Granted));
        assert_eq!(
            runtime.claim(
                F6Claim {
                    reward: ITEM,
                    ..original
                },
                START
            ),
            Err(ClaimError::Conflict)
        );
    }

    let store = Store::open_existing(&f.dir.0.join("accounts.sqlite")).unwrap();
    let mut runtime = F6EventRuntime::new(&store, opted_in_policy()).unwrap();
    let before = f.version();
    assert_eq!(
        runtime.claim(original, START + 1),
        Ok(ClaimOutcome::Replayed)
    );
    assert_eq!(
        runtime.claim(item_claim, START + 1),
        Ok(ClaimOutcome::Replayed)
    );
    for changed in [
        F6Claim {
            reward: F6Reward::Lucci { amount: 26 },
            ..original
        },
        F6Claim {
            system: "Câu cá",
            ..original
        },
        F6Claim {
            player_id: bob.player_id,
            ..original
        },
        F6Claim {
            reward: F6Reward::Item {
                sku_id: "event-chip",
                grant: ItemGrant::Stackable { max_stack: 10 },
                quantity: 3,
            },
            ..item_claim
        },
    ] {
        assert_eq!(runtime.claim(changed, START + 1), Err(ClaimError::Conflict));
    }
    assert_eq!(f.version(), before);
}

#[test]
fn failed_grant_rolls_back_db_writes_claim_key_and_all_quota_charges() {
    let f = Fixture::new();
    let mut runtime = f.runtime();
    let first = claim("first", EXTENDED, &f.player, ITEM);
    let retry = claim("retry", EXTENDED, &f.player, ITEM);
    assert_eq!(runtime.claim(first, START), Ok(ClaimOutcome::Granted));
    // The audit insert is last in the grant transaction, after inventory,
    // ledger, profile and clock writes. Force the entire transaction to abort.
    f.observer
        .execute_batch(
            "CREATE TRIGGER reject_grant_audit BEFORE INSERT ON admin_audit
             BEGIN SELECT RAISE(ABORT, 'test grant rollback'); END;",
        )
        .unwrap();
    let before = f.version();
    assert_eq!(
        runtime.claim(retry, START + DAY + 1),
        Err(ClaimError::Conflict)
    );
    assert_eq!(f.version(), before);
    assert_eq!(
        f.store.list_inventory(f.player.player_id, START).unwrap()[0].quantity,
        2
    );
    f.observer
        .execute_batch("DROP TRIGGER reject_grant_audit")
        .unwrap();

    // The original day's accepted claim must still count. Neither today's
    // quota buckets nor the clock may have advanced on the failed grant.
    assert_eq!(
        runtime.claim(retry, START + 1),
        Err(ClaimError::Policy(
            QuotaRejection::ExtendedDailyQuotaExceeded
        ))
    );
    assert_eq!(runtime.claim(retry, START + DAY), Ok(ClaimOutcome::Granted));
    assert_eq!(
        runtime.claim(claim("third", EXTENDED, &f.player, LUCCI), START + 2 * DAY),
        Err(ClaimError::Policy(
            QuotaRejection::ExtendedSeasonQuotaExceeded
        ))
    );
    assert_eq!(
        f.store.list_inventory(f.player.player_id, START).unwrap()[0].quantity,
        4
    );
    assert_eq!(
        f.store.list_ledger(f.player.player_id, 10).unwrap().len(),
        2
    );
}
