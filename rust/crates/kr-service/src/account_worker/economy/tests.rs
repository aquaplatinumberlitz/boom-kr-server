//! F4 wiring: real match journal -> bounded DB worker -> `AccountService`.
use super::*;
use crate::{
    account_worker::{PersistenceJob, PersistenceResult},
    accounts::{
        Admin, ItemGrant, MatchEndReason, MatchExit, MatchKey, MatchParticipant, MatchStart,
        RewardAmount, ShopCategory, ShopItem, TEST_SERIAL,
    },
    match_journal::{Manager, utc},
};
use std::{
    fs,
    os::unix::fs::PermissionsExt,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
    time::Duration,
};

static NEXT: AtomicU64 = AtomicU64::new(0);
struct Temp(PathBuf);
impl Drop for Temp {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
impl Temp {
    fn db(&self) -> PathBuf {
        self.0.join("accounts.sqlite")
    }
    fn count(&self, sql: &str) -> i64 {
        rusqlite::Connection::open_with_flags(self.db(), rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)
            .unwrap()
            .query_row(sql, [], |r| r.get(0))
            .unwrap()
    }
}
struct Fixture {
    temp: Temp,
    epoch: i64,
    players: Vec<MatchParticipant>,
}
fn fixture() -> Fixture {
    let root = std::env::temp_dir().join(format!(
        "kr-f4-wiring-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&root).unwrap();
    fs::set_permissions(&root, fs::Permissions::from_mode(0o700)).unwrap();
    let temp = Temp(root);
    let _serial = TEST_SERIAL.lock().unwrap();
    let mut store = Store::open(&temp.db()).unwrap();
    let admin = Admin {
        actor: "f4-wiring-test",
        reason: "economy wiring integration",
    };
    let now = utc().unwrap();
    let epoch = store.begin_instance(&admin, now).unwrap();
    let players = [(0, "Alice", 0), (6, "Bob", 3)]
        .into_iter()
        .map(|(slot, name, team)| {
            let p = store
                .create_account(&admin, name, name, b"f4-wiring-password", now)
                .unwrap();
            MatchParticipant {
                account_id: p.account_id,
                player_id: p.player_id,
                slot,
                character_key: 0,
                team,
            }
        })
        .collect();
    Fixture {
        temp,
        epoch,
        players,
    }
}
fn start(fixture: &Fixture, round: i64) -> MatchStart {
    MatchStart {
        key: MatchKey {
            boot_epoch: fixture.epoch,
            room_session: 5,
            room_id: 7,
            room_generation: 9,
            round,
        },
        map_code: 281,
        map_name: "Factory07".into(),
        started_at: utc().unwrap(),
        players: fixture.players.clone(),
    }
}
fn policy() -> EconomyPolicy {
    EconomyPolicy {
        enabled: true,
        win: RewardAmount {
            exp: 150,
            lucci: 500,
        },
        loss: RewardAmount {
            exp: 50,
            lucci: 100,
        },
        draw: RewardAmount { exp: 80, lucci: 0 },
        grant_on_disconnect_or_left: false,
        level_thresholds: vec![0, 100, 200],
    }
}
fn catalog() -> ShopCatalog {
    let item = |sku: &str, category, price, grant, slot: Option<&str>, enabled| ShopItem {
        sku_id: sku.into(),
        category,
        price_lucci: price,
        grant,
        equip_slot: slot.map(str::to_owned),
        enabled,
    };
    ShopCatalog {
        items: vec![
            item(
                "hat_red",
                ShopCategory::Decoration,
                300,
                ItemGrant::Permanent,
                Some("head"),
                true,
            ),
            item(
                "potion",
                ShopCategory::Consumable,
                50,
                ItemGrant::Stackable { max_stack: 10 },
                None,
                true,
            ),
            item(
                "retired",
                ShopCategory::Decoration,
                1,
                ItemGrant::Permanent,
                Some("head"),
                false,
            ),
        ],
    }
}
const REWARD_SUM: &str = "SELECT coalesce(sum(reward),0) FROM match_players";
const LEDGER_ROWS: &str = "SELECT count(*) FROM ledger_entries";
const PROFILE_ROWS: &str = "SELECT count(*) FROM player_profiles";

/// Journal one round to its durable finish and return the persisted finish.
async fn play(
    manager: &Manager,
    start: MatchStart,
    end: impl FnOnce(&mut crate::match_journal::Owner),
) {
    let mut owner = manager.factory().owner();
    owner.begin(start).unwrap();
    owner.ticket().unwrap().wait().await.unwrap();
    end(&mut owner);
    owner.ticket().unwrap().wait().await.unwrap();
}

#[tokio::test]
async fn disabled_default_economy_keeps_m6_zero_reward_and_empty_ledger() {
    let fixture = fixture();
    let worker = Worker::open(fixture.temp.db(), 4, Duration::from_secs(2))
        .await
        .unwrap();
    let service = worker.service();
    assert_eq!(*service.settings(), EconomySettings::default());
    assert!(!service.settings().rewards_enabled());
    let manager = Manager::open(worker.client(), fixture.epoch);
    play(&manager, start(&fixture, 100), |owner| {
        owner.finish(0).unwrap()
    })
    .await;
    play(&manager, start(&fixture, 101), |owner| {
        owner.all_dead().unwrap()
    })
    .await;
    play(&manager, start(&fixture, 102), |owner| {
        owner.cancel(MatchEndReason::AdminClosed).unwrap()
    })
    .await;
    assert_eq!(
        fixture
            .temp
            .count("SELECT count(*) FROM matches WHERE state<>'running'"),
        3
    );
    assert_eq!(fixture.temp.count(REWARD_SUM), 0);
    assert_eq!(fixture.temp.count(LEDGER_ROWS), 0);
    assert_eq!(fixture.temp.count(PROFILE_ROWS), 0);
    let alice = fixture.players[0].player_id;
    let profile = service.profile_get(alice).await.unwrap();
    assert_eq!((profile.exp, profile.level, profile.lucci), (0, 1, 0));
    assert!(service.ledger_list(alice, 10).await.unwrap().is_empty());
    assert!(service.inventory_list(alice).await.unwrap().is_empty());
    // Empty default catalog: nothing can be bought, equipped or unlocked.
    assert_eq!(
        service
            .purchase_shop_item(alice, "hat_red", 1, "buy-1")
            .await,
        Err(Error::InvalidInput)
    );
    assert_eq!(
        service.equip_item(alice, "hat_red", "head").await,
        Err(Error::InvalidInput)
    );
    assert!(
        service
            .usable_items(alice, CatalogAccess { unlock_all: true })
            .await
            .unwrap()
            .is_empty()
    );
    assert_eq!(fixture.temp.count(LEDGER_ROWS), 0);
    assert_eq!(fixture.temp.count(PROFILE_ROWS), 0);
    manager.shutdown().await.unwrap();
    worker.shutdown().await.unwrap();
}

#[tokio::test]
async fn enabled_economy_settles_once_then_shop_equip_and_profile_read_back() {
    let fixture = fixture();
    let settings = EconomySettings::new(policy(), catalog()).unwrap();
    let worker = Worker::open_with_economy(fixture.temp.db(), 4, Duration::from_secs(2), settings)
        .await
        .unwrap();
    let service = worker.service();
    let (alice, bob) = (fixture.players[0].player_id, fixture.players[1].player_id);
    let manager = Manager::open(worker.client(), fixture.epoch);

    // Win/loss through the real journal settles in the finish transaction.
    let first = start(&fixture, 100);
    play(&manager, first.clone(), |owner| owner.finish(0).unwrap()).await;
    let alice_profile = service.profile_get(alice).await.unwrap();
    assert_eq!(
        (alice_profile.exp, alice_profile.level, alice_profile.lucci),
        (150, 2, 500)
    );
    let bob_profile = service.profile_get(bob).await.unwrap();
    assert_eq!(
        (bob_profile.exp, bob_profile.level, bob_profile.lucci),
        (50, 1, 100)
    );
    assert_eq!(fixture.temp.count(LEDGER_ROWS), 2);
    assert_eq!(fixture.temp.count(REWARD_SUM), 0);

    // Replaying the exact finish event and a direct settle are idempotent.
    let match_id: i64 = rusqlite::Connection::open(fixture.temp.db())
        .unwrap()
        .query_row("SELECT id FROM matches WHERE round=100", [], |r| r.get(0))
        .unwrap();
    let finish = {
        let conn = rusqlite::Connection::open(fixture.temp.db()).unwrap();
        let json: String = conn
            .query_row(
                "SELECT finish_json FROM matches WHERE id=?1",
                [match_id],
                |r| r.get(0),
            )
            .unwrap();
        serde_json::from_str(&json).unwrap()
    };
    let PersistenceResult::Match(replayed) = worker
        .client()
        .try_persist(PersistenceJob::FinishMatch(first.key.clone(), finish))
        .unwrap()
        .wait()
        .await
        .unwrap()
    else {
        panic!("wrong persistence result")
    };
    assert_eq!(replayed.id, match_id);
    assert_eq!(replayed.reward_per_player, 0);
    {
        let _serial = TEST_SERIAL.lock().unwrap();
        let store = Store::open(&fixture.temp.db()).unwrap();
        let receipt = store
            .settle_match_rewards(match_id, &policy(), utc().unwrap())
            .unwrap();
        assert!(receipt.settled && receipt.replayed);
    }
    assert_eq!(fixture.temp.count(LEDGER_ROWS), 2);
    assert_eq!(service.profile_get(alice).await.unwrap(), alice_profile);

    // A cancelled round grants nothing even with rewards enabled.
    play(&manager, start(&fixture, 101), |owner| {
        owner.cancel(MatchEndReason::AdminClosed).unwrap()
    })
    .await;
    assert_eq!(fixture.temp.count(LEDGER_ROWS), 2);
    // A left player is not paid by default; the present winner is.
    play(&manager, start(&fixture, 102), |owner| {
        owner.departed(6, MatchExit::Left);
        owner.finish(0).unwrap()
    })
    .await;
    assert_eq!(fixture.temp.count(LEDGER_ROWS), 4);
    let bob_after = service.profile_get(bob).await.unwrap();
    assert_eq!((bob_after.exp, bob_after.lucci), (50, 100));
    let alice_profile = service.profile_get(alice).await.unwrap();
    assert_eq!(
        (alice_profile.exp, alice_profile.level, alice_profile.lucci),
        (300, 3, 1000)
    );

    // Shop: debit Lucci once per key and add the SKU to the inventory.
    let bought = service
        .purchase_shop_item(alice, "hat_red", 1, "buy-hat-1")
        .await
        .unwrap();
    assert!(!bought.replayed);
    assert_eq!(bought.lucci_spent, 300);
    assert_eq!(bought.profile.lucci, 700);
    let again = service
        .purchase_shop_item(alice, "hat_red", 1, "buy-hat-1")
        .await
        .unwrap();
    assert!(again.replayed);
    assert_eq!(again.ledger_id, bought.ledger_id);
    assert_eq!(service.profile_get(alice).await.unwrap().lucci, 700);
    assert_eq!(
        service
            .purchase_shop_item(bob, "hat_red", 1, "bob-hat")
            .await,
        Err(Error::Backend(accounts::Error::InsufficientFunds))
    );
    assert_eq!(
        service.purchase_shop_item(alice, "unknown", 1, "x").await,
        Err(Error::InvalidInput)
    );
    let potions = service
        .purchase_shop_item(alice, "potion", 3, "buy-potion-1")
        .await
        .unwrap();
    assert_eq!(potions.profile.lucci, 550);
    assert_eq!(potions.item.as_ref().unwrap().quantity, 3);

    // Equip into the catalog slot and read everything back through the service.
    assert_eq!(
        service.equip_item(alice, "hat_red", "body").await,
        Err(Error::Backend(accounts::Error::InvalidInput))
    );
    assert_eq!(
        service.equip_item(bob, "hat_red", "head").await,
        Err(Error::Backend(accounts::Error::Conflict))
    );
    let equipped = service.equip_item(alice, "hat_red", "head").await.unwrap();
    assert_eq!(
        (equipped.slot.as_str(), equipped.sku_id.as_str()),
        ("head", "hat_red")
    );
    let listed = service.equipped_list(alice).await.unwrap();
    assert_eq!(listed, vec![equipped]);
    let inventory = service.inventory_list(alice).await.unwrap();
    assert_eq!(
        inventory
            .iter()
            .map(|i| (i.sku_id.as_str(), i.quantity, i.equipped_slots.clone()))
            .collect::<Vec<_>>(),
        [
            ("hat_red", 1, vec!["head".to_owned()]),
            ("potion", 3, vec![])
        ]
    );
    let ledger = service.ledger_list(alice, 10).await.unwrap();
    assert_eq!(
        ledger.iter().map(|e| e.kind.as_str()).collect::<Vec<_>>(),
        [
            "shop_purchase",
            "shop_purchase",
            "match_reward",
            "match_reward"
        ]
    );
    assert_eq!(ledger.iter().map(|e| e.lucci_delta).sum::<i64>(), 550);
    let profile = service.profile_get(alice).await.unwrap();
    assert_eq!((profile.exp, profile.level, profile.lucci), (300, 3, 550));
    assert!(service.unequip_slot(alice, "head").await.unwrap());
    assert!(!service.unequip_slot(alice, "head").await.unwrap());
    assert!(service.equipped_list(alice).await.unwrap().is_empty());

    // O1 unlock_all: enabled catalog SKUs only, with no DB read or write.
    let before = (
        fixture.temp.count(LEDGER_ROWS),
        fixture
            .temp
            .count("SELECT sum(version) FROM player_profiles"),
        fixture.temp.count("SELECT count(*) FROM player_inventory"),
    );
    let resume = crate::account_worker::tests::pause(&worker.client()).await;
    let unlocked = tokio::time::timeout(
        Duration::from_millis(500),
        service.usable_items(bob, CatalogAccess { unlock_all: true }),
    )
    .await
    .expect("unlock_all must not wait for the paused DB worker")
    .unwrap();
    resume.send(()).unwrap();
    assert_eq!(
        unlocked
            .iter()
            .map(|i| (i.sku_id.as_str(), i.unlocked_by_session))
            .collect::<Vec<_>>(),
        [("hat_red", true), ("potion", true)]
    );
    let owned = service
        .usable_items(bob, CatalogAccess { unlock_all: false })
        .await
        .unwrap();
    assert!(owned.is_empty());
    assert_eq!(
        before,
        (
            fixture.temp.count(LEDGER_ROWS),
            fixture
                .temp
                .count("SELECT sum(version) FROM player_profiles"),
            fixture.temp.count("SELECT count(*) FROM player_inventory"),
        )
    );
    assert_eq!(fixture.temp.count(REWARD_SUM), 0);
    assert_eq!(
        rusqlite::Connection::open(fixture.temp.db())
            .unwrap()
            .query_row(
                "SELECT outcome FROM match_players WHERE match_id=?1 AND slot=0",
                [match_id],
                |r| r.get::<_, String>(0)
            )
            .unwrap(),
        "win"
    );
    manager.shutdown().await.unwrap();
    worker.shutdown().await.unwrap();
}

#[tokio::test]
async fn live_lane_and_bad_input_never_reach_the_economy() {
    let fixture = fixture();
    let settings = EconomySettings::new(policy(), catalog()).unwrap();
    let worker = Worker::open_with_economy(fixture.temp.db(), 4, Duration::from_secs(2), settings)
        .await
        .unwrap();
    let service = worker.service();
    let alice = fixture.players[0].player_id;
    assert_eq!(service.profile_get(0).await, Err(Error::InvalidInput));
    assert_eq!(
        service.ledger_list(alice, 0).await,
        Err(Error::InvalidInput)
    );
    assert_eq!(
        service.ledger_list(alice, 101).await,
        Err(Error::InvalidInput)
    );
    assert_eq!(
        service.purchase_shop_item(alice, "potion", 0, "k").await,
        Err(Error::InvalidInput)
    );
    assert_eq!(
        service.purchase_shop_item(alice, "potion", 1, "").await,
        Err(Error::InvalidInput)
    );
    assert_eq!(
        service.unequip_slot(alice, "").await,
        Err(Error::InvalidInput)
    );
    assert_eq!(
        service.profile_get(i64::MAX).await,
        Err(Error::Backend(accounts::Error::InvalidInput))
    );
    let live = Worker::open_live(fixture.temp.db(), 1, Duration::from_secs(1))
        .await
        .unwrap();
    assert_eq!(*live.service().settings(), EconomySettings::default());
    assert_eq!(
        live.service().profile_get(alice).await,
        Err(Error::InvalidInput)
    );
    live.shutdown().await.unwrap();
    assert_eq!(fixture.temp.count(LEDGER_ROWS), 0);
    worker.shutdown().await.unwrap();
}

#[test]
fn invalid_economy_settings_are_rejected_before_the_worker_opens() {
    let mut bad = policy();
    bad.level_thresholds = vec![0, 100, 100];
    assert_eq!(
        EconomySettings::new(bad, ShopCatalog::default()),
        Err(Error::Backend(accounts::Error::InvalidInput))
    );
    let mut dup = catalog();
    dup.items.push(dup.items[0].clone());
    assert_eq!(
        EconomySettings::new(EconomyPolicy::default(), dup),
        Err(Error::Backend(accounts::Error::InvalidInput))
    );
}

#[test]
fn account_config_economy_fields_are_optional_and_validated() {
    use crate::account_serving::Config;
    let base = serde_json::json!({
        "database": "/srv/kr/accounts.sqlite",
        "launch_socket": "/srv/kr/launch.sock",
        "launch_ttl_seconds": 60,
        "persist_matches": true,
        "profile_mode": "database_zero",
    });
    // Pre-F4 JSON without the new fields keeps the disabled default.
    let old: Config = serde_json::from_value(base.clone()).unwrap();
    assert_eq!(old.economy().unwrap(), EconomySettings::default());
    let with = |extra: serde_json::Value| {
        let mut value = base.clone();
        value
            .as_object_mut()
            .unwrap()
            .extend(extra.as_object().unwrap().clone());
        serde_json::from_value::<Config>(value)
    };
    let enabled = with(serde_json::json!({
        "economy_policy": serde_json::to_value(policy()).unwrap(),
        "shop_catalog": serde_json::to_value(catalog()).unwrap(),
    }))
    .unwrap();
    let settings = enabled.economy().unwrap();
    assert!(settings.rewards_enabled());
    assert_eq!(settings.catalog(), &catalog());
    // Rewards only settle for journaled matches.
    let mut unjournaled = enabled;
    unjournaled.persist_matches = false;
    assert!(unjournaled.economy().is_err());
    // A shop needs DB-backed players.
    let mut fixed = with(serde_json::json!({
        "shop_catalog": serde_json::to_value(catalog()).unwrap(),
    }))
    .unwrap();
    fixed.persist_matches = false;
    assert!(fixed.economy().is_ok());
    fixed.profile_mode = crate::account_serving::ProfileMode::Fixed;
    assert!(fixed.economy().is_err());
    let mut bad = policy();
    bad.level_thresholds = vec![5];
    assert!(
        with(serde_json::json!({"economy_policy": serde_json::to_value(bad).unwrap()}))
            .unwrap()
            .economy()
            .is_err()
    );
    assert!(with(serde_json::json!({"economy_policy": {"enabled": true}})).is_err());
    assert!(with(serde_json::json!({"shop_catalog": {"items": [], "extra": 1}})).is_err());
}
