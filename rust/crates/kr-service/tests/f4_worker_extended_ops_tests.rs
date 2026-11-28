use kr_service::{
    account_worker::{AccountService, EconomySettings, Error, Worker},
    accounts::{
        self, Admin, EconomyPolicy, ItemGrant, ItemUpgradeOperation, LedgerEntry, MatchEndReason,
        MatchExit, MatchFinish, MatchKey, MatchOutcome, MatchParticipant, MatchPlayerFinish,
        MatchStart, Player, RecordEraserOperation, RewardAmount, ShopCatalog, ShopCategory,
        ShopItem, Store,
    },
    f4_extended_policy::{
        CouponOffer, CouponRequest, Decision, EraserDefinition, GiftCouponPolicy,
        ItemGrant as PolicyItemGrant, LucciVaultPolicy, OperationKey, RecordEraserPolicy,
        RecordScope, RecordValue, TitlePolicy, TitleUnlock, TitleUnlockDecision, UpgradeFailure,
        UpgradeFamily, UpgradePolicy, UpgradeRecipe,
    },
};
use std::{
    fs,
    future::Future,
    os::unix::fs::DirBuilderExt,
    path::PathBuf,
    pin::pin,
    sync::{Mutex, PoisonError},
    task::{Context, Waker},
    thread,
    time::{Duration, Instant},
};

static CREATE_ACCOUNT_GATE: Mutex<()> = Mutex::new(());

const ADMIN: Admin<'static> = Admin {
    actor: "f4-worker-ops-test",
    reason: "prepare extended economy fixture",
};
const STACK: ItemGrant = ItemGrant::Stackable { max_stack: 3 };

struct TempDir(PathBuf);

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn create_account(store: &mut Store, login: &str) -> Player {
    // HashPermit allows at most two concurrent Argon2id operations per process.
    let _gate = CREATE_ACCOUNT_GATE
        .lock()
        .unwrap_or_else(PoisonError::into_inner);
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        match store.create_account(&ADMIN, login, login, b"f4-worker-test-password", None) {
            Ok(player) => return player,
            Err(accounts::Error::Busy) if Instant::now() < deadline => {
                thread::sleep(Duration::from_millis(10));
            }
            Err(error) => panic!("create account: {error:?}"),
        }
    }
}

struct Fixture {
    store: Store,
    player: Player,
    epoch: i64,
    temp: TempDir,
}

impl Fixture {
    fn new() -> Self {
        let mut nonce = [0; 8];
        getrandom::fill(&mut nonce).unwrap();
        let dir = std::env::temp_dir().join(format!(
            "kr-f4-worker-ops-{}-{:016x}",
            std::process::id(),
            u64::from_ne_bytes(nonce),
        ));
        fs::DirBuilder::new().mode(0o700).create(&dir).unwrap();
        let temp = TempDir(dir);
        let mut store = Store::open(&temp.0.join("accounts.sqlite")).unwrap();
        let epoch = store
            .begin_instance(&ADMIN, accounts::utc().unwrap())
            .unwrap();
        let player = create_account(&mut store, "worker");
        Self {
            store,
            player,
            epoch,
            temp,
        }
    }

    fn grant(&self, sku: &str, grant: ItemGrant, quantity: u32) {
        self.store
            .admin_grant_item(
                &ADMIN,
                self.player.player_id,
                sku,
                grant,
                quantity,
                sku,
                None,
            )
            .unwrap();
    }

    fn settle_win(&mut self) {
        let other = create_account(&mut self.store, "opponent");
        let now = accounts::utc().unwrap();
        let key = MatchKey {
            boot_epoch: self.epoch,
            room_session: 1,
            room_id: 1,
            room_generation: 1,
            round: 1,
        };
        let players = [&self.player, &other]
            .into_iter()
            .zip(0..)
            .map(|(player, slot)| MatchParticipant {
                account_id: player.account_id,
                player_id: player.player_id,
                slot,
                character_key: 0,
                team: slot,
            })
            .collect();
        let start = MatchStart {
            key,
            map_code: 1,
            map_name: "Factory07".into(),
            started_at: now,
            players,
        };
        self.store.begin_match(&start, now).unwrap();
        let finish = MatchFinish {
            ended_at: now,
            reason: MatchEndReason::Elimination,
            players: [MatchOutcome::Win, MatchOutcome::Loss]
                .into_iter()
                .zip(0..)
                .map(|(outcome, slot)| MatchPlayerFinish {
                    slot,
                    outcome,
                    exit: MatchExit::Present,
                })
                .collect(),
        };
        let economy = EconomyPolicy {
            enabled: true,
            win: RewardAmount { exp: 100, lucci: 0 },
            level_thresholds: vec![0, 100],
            ..EconomyPolicy::default()
        };
        self.store
            .finish_match_settled(&start.key, &finish, &economy, now)
            .unwrap();
    }

    async fn open(&self, capacity: usize, settings: EconomySettings) -> (Worker, AccountService) {
        let worker = Worker::open_with_economy(
            self.temp.0.join("accounts.sqlite"),
            capacity,
            Duration::from_secs(10),
            settings,
        )
        .await
        .unwrap();
        let service = worker.service();
        (worker, service)
    }
}

fn settings() -> EconomySettings {
    let catalog = ShopCatalog {
        items: vec![ShopItem {
            sku_id: "material".into(),
            category: ShopCategory::Consumable,
            price_lucci: 0,
            grant: STACK,
            equip_slot: None,
            enabled: true,
        }],
    };
    EconomySettings::new(EconomyPolicy::default(), catalog).unwrap()
}

fn coupon_policy() -> GiftCouponPolicy {
    GiftCouponPolicy {
        enabled: true,
        coupon_offers: [("credit", "lucci", 10), ("materials", "material", 2)]
            .into_iter()
            .map(|(offer, sku, quantity)| CouponOffer {
                offer: offer.into(),
                valid_for_seconds: 3600,
                grant: PolicyItemGrant {
                    sku: sku.into(),
                    quantity,
                },
            })
            .collect(),
        ..GiftCouponPolicy::default()
    }
}

fn eraser_policy() -> RecordEraserPolicy {
    RecordEraserPolicy {
        enabled: true,
        allowed_scopes: vec![RecordScope::Level],
        erasers: vec![EraserDefinition {
            sku: "eraser".into(),
            reset_to: RecordValue::Level(1),
        }],
        ..RecordEraserPolicy::default()
    }
}

fn upgrade_policy() -> UpgradePolicy {
    UpgradePolicy {
        enabled: true,
        recipes: vec![UpgradeRecipe {
            family: UpgradeFamily::Item,
            sku: "gear".into(),
            from_level: 0,
            to_level: 1,
            max_level: 1,
            materials: vec![PolicyItemGrant {
                sku: "material".into(),
                quantity: 2,
            }],
            lucci_cost: 5,
            success_bps: 5000,
            on_failure: UpgradeFailure::Keep,
            charge_on_failure: false,
        }],
    }
}

fn title_policy() -> TitlePolicy {
    TitlePolicy {
        enabled: true,
        equip_slot: Some("title".into()),
        notify_on_unlock: true,
        unlock_table: Some(vec![
            TitleUnlock {
                title: "newbie".into(),
                condition: "level>=1".into(),
            },
            TitleUnlock {
                title: "winner".into(),
                condition: "wins>=1".into(),
            },
        ]),
    }
}

fn opted_in(settings: EconomySettings) -> EconomySettings {
    settings
        .with_extended_ops_policies(
            coupon_policy(),
            eraser_policy(),
            upgrade_policy(),
            title_policy(),
        )
        .unwrap()
}

fn coupon<'a>(player: &Player, id: u64, offer: &'a str) -> CouponRequest<'a> {
    let account = u64::try_from(player.account_id).unwrap();
    let now = u64::try_from(accounts::utc().unwrap()).unwrap();
    CouponRequest {
        key: OperationKey { account, id },
        coupon_id: id,
        offer,
        issued_to: account,
        issued_at: now,
        now,
        locally_issued: true,
    }
}

fn eraser() -> RecordEraserOperation<'static> {
    RecordEraserOperation {
        key: 1,
        sku: "eraser",
        scope: RecordScope::Level,
    }
}

fn upgrade() -> ItemUpgradeOperation<'static> {
    ItemUpgradeOperation {
        key: 1,
        sku: "gear",
        family: UpgradeFamily::Item,
        level: 0,
        roll_bps: 0,
    }
}

fn applied(result: Result<Decision<LedgerEntry>, Error>) -> LedgerEntry {
    match result.unwrap() {
        Decision::Apply(entry) => entry,
        Decision::Replay(entry) => panic!("unexpected replay: {entry:?}"),
    }
}

#[test]
fn policy_setting_is_validated_and_independent_of_settlement() {
    let vault = LucciVaultPolicy {
        enabled: true,
        base_lucci_cap: Some(10),
        expansion_step: Some(1),
        max_expansions: Some(1),
        max_lucci_cap: Some(11),
    };
    let mut settings = settings()
        .with_extended_policies(Default::default(), Default::default(), vault.clone())
        .unwrap();
    assert!(!settings.gift_coupon_policy().enabled);
    assert!(!settings.record_eraser_policy().enabled);
    assert!(!settings.upgrade_policy().enabled);
    assert!(!settings.title_policy().enabled);

    settings
        .set_extended_ops_policies(
            coupon_policy(),
            eraser_policy(),
            upgrade_policy(),
            title_policy(),
        )
        .unwrap();
    assert_eq!(settings.gift_coupon_policy(), &coupon_policy());
    assert_eq!(settings.record_eraser_policy(), &eraser_policy());
    assert_eq!(settings.upgrade_policy(), &upgrade_policy());
    assert_eq!(settings.title_policy(), &title_policy());
    assert_eq!(settings.lucci_vault_policy(), &vault);

    let before = settings.clone();
    assert_eq!(
        settings.set_extended_ops_policies(
            Default::default(),
            Default::default(),
            Default::default(),
            TitlePolicy {
                enabled: true,
                ..TitlePolicy::default()
            },
        ),
        Err(Error::InvalidInput),
    );
    assert_eq!(settings, before);
}

#[tokio::test]
async fn new_settings_reject_all_four_operations_without_mutation() {
    let fixture = Fixture::new();
    let player = fixture.player.player_id;
    let (worker, service) = fixture.open(4, settings()).await;
    let before = service.profile_get(player).await.unwrap();
    assert_eq!(
        service
            .redeem_gift_coupon(player, coupon(&fixture.player, 1, "credit"))
            .await,
        Err(Error::InvalidInput),
    );
    assert_eq!(
        service.apply_record_eraser(player, eraser()).await,
        Err(Error::InvalidInput)
    );
    assert_eq!(
        service.apply_item_upgrade(player, upgrade()).await,
        Err(Error::InvalidInput)
    );
    assert_eq!(
        service.evaluate_title_unlock(player, "newbie").await,
        Err(Error::InvalidInput)
    );
    assert_eq!(service.profile_get(player).await.unwrap(), before);
    assert!(service.inventory_list(player).await.unwrap().is_empty());
    assert!(service.ledger_list(player, 10).await.unwrap().is_empty());
    worker.shutdown().await.unwrap();
}

#[tokio::test]
async fn coupon_replays_once_and_enforces_account_ownership_expiry_and_vault_cap() {
    let mut fixture = Fixture::new();
    let other = create_account(&mut fixture.store, "other");
    let player = fixture.player.player_id;
    let vault = LucciVaultPolicy {
        enabled: true,
        base_lucci_cap: Some(10),
        expansion_step: Some(10),
        max_expansions: Some(1),
        max_lucci_cap: Some(20),
    };
    let settings = opted_in(settings())
        .with_extended_policies(Default::default(), Default::default(), vault)
        .unwrap();
    let (worker, service) = fixture.open(4, settings).await;

    let request = coupon(&fixture.player, 1, "credit");
    let expired = CouponRequest {
        coupon_id: 2,
        issued_at: request.now - 3600,
        now: request.now - 3600,
        ..request
    };
    assert_eq!(
        service.redeem_gift_coupon(player, expired).await,
        Err(Error::Backend(accounts::Error::InsufficientQuantity)),
    );

    let entry = applied(service.redeem_gift_coupon(player, request).await);
    assert_eq!(entry.lucci_delta, 10);
    assert_eq!(entry.sku_id, None);
    let after = service.profile_get(player).await.unwrap();
    assert_eq!(after.lucci, 10);
    assert_eq!(
        service.redeem_gift_coupon(player, request).await,
        Ok(Decision::Replay(entry.clone())),
    );

    let changed = CouponRequest {
        key: OperationKey {
            id: 2,
            ..request.key
        },
        ..request
    };
    for (target, request, error) in [
        (player, changed, accounts::Error::Conflict),
        (other.player_id, request, accounts::Error::InvalidInput),
        (
            other.player_id,
            coupon(&other, 1, "credit"),
            accounts::Error::Conflict,
        ),
        (
            player,
            coupon(&fixture.player, 2, "credit"),
            accounts::Error::Conflict,
        ),
    ] {
        assert_eq!(
            service.redeem_gift_coupon(target, request).await,
            Err(Error::Backend(error)),
        );
    }
    assert_eq!(service.profile_get(player).await.unwrap(), after);
    assert_eq!(service.ledger_list(player, 10).await.unwrap(), vec![entry]);
    assert_eq!(service.profile_get(other.player_id).await.unwrap().lucci, 0);
    assert!(
        service
            .ledger_list(other.player_id, 10)
            .await
            .unwrap()
            .is_empty()
    );
    worker.shutdown().await.unwrap();
}

#[tokio::test]
async fn coupon_grants_catalog_inventory_without_exceeding_stack_limit() {
    let fixture = Fixture::new();
    let player = fixture.player.player_id;
    let (worker, service) = fixture.open(4, opted_in(settings())).await;
    let request = coupon(&fixture.player, 1, "materials");
    let entry = applied(service.redeem_gift_coupon(player, request).await);
    assert_eq!(entry.sku_id.as_deref(), Some("material"));
    assert_eq!(entry.lucci_delta, 0);
    let inventory = service.inventory_list(player).await.unwrap();
    assert_eq!(inventory.len(), 1);
    assert_eq!(inventory[0].sku_id, "material");
    assert_eq!(inventory[0].quantity, 2);
    assert_eq!(
        service.redeem_gift_coupon(player, request).await,
        Ok(Decision::Replay(entry.clone())),
    );
    assert_eq!(
        service
            .redeem_gift_coupon(player, coupon(&fixture.player, 2, "materials"))
            .await,
        Err(Error::Backend(accounts::Error::Conflict)),
    );
    assert_eq!(service.inventory_list(player).await.unwrap(), inventory);
    assert_eq!(service.ledger_list(player, 10).await.unwrap(), vec![entry]);
    worker.shutdown().await.unwrap();
}

#[tokio::test]
async fn record_eraser_resets_level_and_consumes_once() {
    let mut fixture = Fixture::new();
    fixture.settle_win();
    fixture.grant("eraser", STACK, 1);
    let player = fixture.player.player_id;
    let (worker, service) = fixture.open(4, opted_in(settings())).await;
    let before = service.profile_get(player).await.unwrap();
    assert_eq!((before.level, before.exp), (2, 100));
    let ledger_before = service.ledger_list(player, 10).await.unwrap();

    let request = eraser();
    let entry = applied(service.apply_record_eraser(player, request).await);
    assert_eq!(entry.sku_id.as_deref(), Some("eraser"));
    let after = service.profile_get(player).await.unwrap();
    assert_eq!((after.level, after.exp), (1, before.exp));
    assert!(service.inventory_list(player).await.unwrap().is_empty());
    assert_eq!(
        service.apply_record_eraser(player, request).await,
        Ok(Decision::Replay(entry.clone())),
    );
    assert_eq!(
        service
            .apply_record_eraser(
                player,
                RecordEraserOperation {
                    scope: RecordScope::WinLoss,
                    ..request
                }
            )
            .await,
        Err(Error::Backend(accounts::Error::Conflict)),
    );
    assert_eq!(
        service
            .apply_record_eraser(player, RecordEraserOperation { key: 2, ..request })
            .await,
        Err(Error::Backend(accounts::Error::InsufficientQuantity)),
    );
    let mut expected = vec![entry];
    expected.extend(ledger_before);
    assert_eq!(service.ledger_list(player, 10).await.unwrap(), expected);
    assert_eq!(service.profile_get(player).await.unwrap(), after);
    worker.shutdown().await.unwrap();
}

#[tokio::test]
async fn upgrade_checks_resources_and_records_a_single_debit_and_replacement() {
    let fixture = Fixture::new();
    fixture.grant(
        "gear",
        ItemGrant::Timed {
            duration_seconds: 3600,
        },
        1,
    );
    fixture.grant("material", STACK, 1);
    let player = fixture.player.player_id;
    let (worker, service) = fixture.open(4, opted_in(settings())).await;

    let request = upgrade();
    let initial_ledger = service.ledger_list(player, 10).await.unwrap();
    let initial_inventory = service.inventory_list(player).await.unwrap();
    let expiry = initial_inventory
        .iter()
        .find(|item| item.sku_id == "gear")
        .unwrap()
        .expires_at;
    assert!(expiry.is_some());
    assert_eq!(
        service.apply_item_upgrade(player, request).await,
        Err(Error::Backend(accounts::Error::InsufficientFunds)),
    );
    assert_eq!(
        service.ledger_list(player, 10).await.unwrap(),
        initial_ledger
    );
    assert_eq!(
        service.inventory_list(player).await.unwrap(),
        initial_inventory
    );

    applied(
        service
            .redeem_gift_coupon(player, coupon(&fixture.player, 1, "credit"))
            .await,
    );
    let funded_ledger = service.ledger_list(player, 10).await.unwrap();
    assert_eq!(
        service.apply_item_upgrade(player, request).await,
        Err(Error::Backend(accounts::Error::InsufficientQuantity)),
    );
    assert_eq!(service.profile_get(player).await.unwrap().lucci, 10);
    assert_eq!(
        service.ledger_list(player, 10).await.unwrap(),
        funded_ledger
    );
    assert_eq!(
        service.inventory_list(player).await.unwrap(),
        initial_inventory
    );
    applied(
        service
            .redeem_gift_coupon(player, coupon(&fixture.player, 2, "materials"))
            .await,
    );

    let ledger_before = service.ledger_list(player, 10).await.unwrap();
    let entry = applied(service.apply_item_upgrade(player, request).await);
    assert_eq!(entry.lucci_delta, -5);
    assert_eq!(entry.sku_id.as_deref(), Some("gear.f4-item-1"));
    assert_eq!(
        service
            .apply_item_upgrade(
                player,
                ItemUpgradeOperation {
                    roll_bps: 9999,
                    ..request
                }
            )
            .await,
        Ok(Decision::Replay(entry.clone())),
    );
    assert_eq!(
        service
            .apply_item_upgrade(
                player,
                ItemUpgradeOperation {
                    level: 1,
                    ..request
                }
            )
            .await,
        Err(Error::Backend(accounts::Error::Conflict)),
    );
    assert_eq!(
        service
            .apply_item_upgrade(player, ItemUpgradeOperation { key: 2, ..request })
            .await,
        Err(Error::Backend(accounts::Error::InsufficientQuantity)),
    );

    assert_eq!(service.profile_get(player).await.unwrap().lucci, 5);
    let inventory = service.inventory_list(player).await.unwrap();
    assert_eq!(inventory.len(), 2);
    let gear = inventory
        .iter()
        .find(|item| item.sku_id == "gear.f4-item-1")
        .unwrap();
    assert_eq!(
        (gear.grant_kind.as_str(), gear.quantity, gear.expires_at),
        ("timed", 1, expiry)
    );
    let material = inventory
        .iter()
        .find(|item| item.sku_id == "material")
        .unwrap();
    assert_eq!(material.quantity, 1);
    let mut expected = vec![entry];
    expected.extend(ledger_before);
    assert_eq!(service.ledger_list(player, 10).await.unwrap(), expected);
    worker.shutdown().await.unwrap();
}

#[tokio::test]
async fn title_eligibility_is_repeatable_and_read_only() {
    let fixture = Fixture::new();
    let player = fixture.player.player_id;
    let (worker, service) = fixture.open(4, opted_in(settings())).await;
    let before = service.profile_get(player).await.unwrap();
    for _ in 0..2 {
        assert_eq!(
            service.evaluate_title_unlock(player, "newbie").await,
            Ok(Some(TitleUnlockDecision::New {
                notify_after_commit: true
            })),
        );
    }
    assert_eq!(
        service.evaluate_title_unlock(player, "winner").await,
        Ok(None)
    );
    assert_eq!(
        service.evaluate_title_unlock(player, "unknown").await,
        Ok(None)
    );
    assert_eq!(service.profile_get(player).await.unwrap(), before);
    assert!(service.inventory_list(player).await.unwrap().is_empty());
    assert!(service.ledger_list(player, 10).await.unwrap().is_empty());
    worker.shutdown().await.unwrap();
}

#[tokio::test]
async fn all_four_operations_return_busy_when_worker_queue_is_full() {
    let fixture = Fixture::new();
    let player = fixture.player.player_id;
    let (worker, service) = fixture.open(1, opted_in(settings())).await;

    // The worker waits for the received trial grant to be accepted, so the
    // first economy call fills the queue without racing execution.
    let handoff = worker
        .client()
        .try_trial_local_current(1, 60, None)
        .unwrap()
        .receive()
        .await
        .unwrap();
    let request = coupon(&fixture.player, 1, "credit");
    let mut admitted = pin!(service.redeem_gift_coupon(player, request));
    assert!(
        admitted
            .as_mut()
            .poll(&mut Context::from_waker(Waker::noop()))
            .is_pending()
    );
    assert_eq!(
        service.redeem_gift_coupon(player, request).await,
        Err(Error::Busy)
    );
    assert_eq!(
        service.apply_record_eraser(player, eraser()).await,
        Err(Error::Busy)
    );
    assert_eq!(
        service.apply_item_upgrade(player, upgrade()).await,
        Err(Error::Busy)
    );
    assert_eq!(
        service.evaluate_title_unlock(player, "newbie").await,
        Err(Error::Busy)
    );

    handoff.accept().unwrap();
    let entry = applied(admitted.await);
    assert_eq!(service.ledger_list(player, 10).await.unwrap(), vec![entry]);
    worker.shutdown().await.unwrap();
}
