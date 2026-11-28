use kr_service::accounts::{Admin, Error, ItemGrant, ShopCatalog, ShopCategory, ShopItem, Store};
use kr_service::f4_extended_policy::{
    CouponRequest, Decision, GiftCouponPolicy, LucciVaultPolicy, OperationKey, RecordEraserPolicy,
    RecordScope, TitlePolicy, TitleUnlockDecision, UpgradeFamily,
};
use serde_json::json;
use std::{fs, os::unix::fs::DirBuilderExt, path::PathBuf, sync::Mutex, thread, time::Duration};

static CREATE_ACCOUNT_GATE: Mutex<()> = Mutex::new(());

struct Fixture {
    dir: PathBuf,
    store: Store,
    player: i64,
    account: u64,
}

impl Fixture {
    fn new() -> Self {
        let mut nonce = [0_u8; 8];
        getrandom::fill(&mut nonce).unwrap();
        let dir = std::env::temp_dir().join(format!(
            "f4-ops-{}-{:x}",
            std::process::id(),
            u64::from_ne_bytes(nonce)
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::DirBuilder::new()
            .mode(0o700)
            .recursive(true)
            .create(&dir)
            .unwrap();
        let mut store = Store::open(&dir.join("accounts.sqlite")).unwrap();
        let admin = Admin {
            actor: "f4-tests",
            reason: "f4 operation tests",
        };
        store.begin_instance(&admin, 1).unwrap();
        let _gate = CREATE_ACCOUNT_GATE
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let player = loop {
            match store.create_account(&admin, "f4ops", "f4ops", b"password-1234", 1) {
                Ok(player) => break player,
                Err(Error::Busy) => thread::sleep(Duration::from_millis(10)),
                Err(error) => panic!("create account: {error:?}"),
            }
        };
        Self {
            dir,
            store,
            player: player.player_id,
            account: player.account_id as u64,
        }
    }

    fn grant_item(&self, sku: &str, grant: ItemGrant, quantity: u32) {
        let admin = Admin {
            actor: "f4-tests",
            reason: "seed",
        };
        self.store
            .admin_grant_item(&admin, self.player, sku, grant, quantity, "seed-item", 2)
            .unwrap();
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.dir);
    }
}

fn coupon_policy() -> GiftCouponPolicy {
    serde_json::from_value(json!({
        "enabled": true,
        "coupon_offers": [{"offer":"ten-lucci", "valid_for_seconds": 60,
                            "grant":{"sku":"lucci", "quantity":10}}]
    }))
    .unwrap()
}

#[test]
fn disabled_defaults_fail_closed_without_a_ledger_row() {
    let f = Fixture::new();
    let before = f.store.list_ledger(f.player, 100).unwrap().len();
    let request = CouponRequest {
        key: OperationKey {
            account: f.account,
            id: 1,
        },
        coupon_id: 1,
        offer: "ten-lucci",
        issued_to: f.account,
        issued_at: 1,
        now: 2,
        locally_issued: true,
    };
    assert_eq!(
        f.store.redeem_gift_coupon(
            f.player,
            request,
            &GiftCouponPolicy::default(),
            &LucciVaultPolicy::default(),
            &ShopCatalog::default(),
        ),
        Err(Error::InvalidInput)
    );
    assert_eq!(f.store.list_ledger(f.player, 100).unwrap().len(), before);
}

#[test]
fn coupon_replay_and_conflict_are_exact_and_vault_is_atomic() {
    let f = Fixture::new();
    let policy = coupon_policy();
    let vault: LucciVaultPolicy = serde_json::from_value(json!({
        "enabled": true, "base_lucci_cap": 15, "expansion_step": 1,
        "max_expansions": 0, "max_lucci_cap": 15
    }))
    .unwrap();
    let request = CouponRequest {
        key: OperationKey {
            account: f.account,
            id: 7,
        },
        coupon_id: 7,
        offer: "ten-lucci",
        issued_to: f.account,
        issued_at: 1,
        now: 2,
        locally_issued: true,
    };
    let first = f
        .store
        .redeem_gift_coupon(f.player, request, &policy, &vault, &ShopCatalog::default())
        .unwrap();
    assert!(matches!(first, Decision::Apply(_)));
    let replay = f
        .store
        .redeem_gift_coupon(f.player, request, &policy, &vault, &ShopCatalog::default())
        .unwrap();
    assert!(matches!(replay, Decision::Replay(_)));
    let conflict = CouponRequest {
        offer: "different",
        ..request
    };
    assert_eq!(
        f.store
            .redeem_gift_coupon(f.player, conflict, &policy, &vault, &ShopCatalog::default(),),
        Err(Error::Conflict)
    );
    assert_eq!(f.store.get_profile(f.player).unwrap().lucci, 10);
    let over_cap = CouponRequest {
        coupon_id: 8,
        key: OperationKey {
            id: 8,
            ..request.key
        },
        ..request
    };
    assert_eq!(
        f.store
            .redeem_gift_coupon(f.player, over_cap, &policy, &vault, &ShopCatalog::default(),),
        Err(Error::Conflict)
    );
    assert_eq!(f.store.list_ledger(f.player, 100).unwrap().len(), 1);
}

#[test]
fn upgrade_and_eraser_reject_missing_resources_without_receipts() {
    let f = Fixture::new();
    let upgrade: kr_service::f4_extended_policy::UpgradePolicy = serde_json::from_value(json!({
        "enabled": true,
        "recipes": [{"family":"item","sku":"sword","from_level":0,"to_level":1,
                     "max_level":1,"materials":[],"lucci_cost":5,"success_bps":10000,
                     "on_failure":"keep","charge_on_failure":true}]
    }))
    .unwrap();
    f.grant_item("sword", ItemGrant::Permanent, 1);
    assert_eq!(
        f.store.apply_item_upgrade(
            f.player,
            kr_service::accounts::ItemUpgradeOperation {
                key: 1,
                sku: "sword",
                family: UpgradeFamily::Item,
                level: 0,
                roll_bps: 0,
            },
            &upgrade,
            2,
        ),
        Err(Error::InsufficientFunds)
    );
    let eraser: RecordEraserPolicy = serde_json::from_value(json!({
        "enabled": true,
        "allowed_scopes":["level"],
        "erasers":[{"sku":"eraser","reset_to":{"level":1}}]
    }))
    .unwrap();
    assert_eq!(
        f.store.apply_record_eraser(
            f.player,
            kr_service::accounts::RecordEraserOperation {
                key: 2,
                sku: "eraser",
                scope: RecordScope::Level,
            },
            &eraser,
            2,
        ),
        Err(Error::InsufficientQuantity)
    );
    assert!(
        f.store
            .list_ledger(f.player, 100)
            .unwrap()
            .iter()
            .all(|row| row.idempotency_key == "seed-item")
    );
}

#[test]
fn title_evaluation_is_read_only_and_disabled_is_none() {
    let f = Fixture::new();
    let disabled = TitlePolicy::default();
    assert_eq!(
        f.store
            .evaluate_title_unlock(f.player, "newbie", &disabled, 2)
            .unwrap(),
        None
    );
    let policy: TitlePolicy = serde_json::from_value(json!({
        "enabled": true, "equip_slot":"title",
        "unlock_table":[{"title":"newbie","condition":"level>=1"}]
    }))
    .unwrap();
    assert_eq!(
        f.store
            .evaluate_title_unlock(f.player, "newbie", &policy, 2)
            .unwrap(),
        Some(TitleUnlockDecision::New {
            notify_after_commit: false
        })
    );
    assert!(f.store.list_ledger(f.player, 100).unwrap().is_empty());
}

#[test]
fn policy_types_remain_available_for_upgrade_and_eraser_callers() {
    let _ = (
        ItemGrant::Stackable { max_stack: 2 },
        ShopItem {
            sku_id: "mat".into(),
            category: ShopCategory::Consumable,
            price_lucci: 0,
            grant: ItemGrant::Stackable { max_stack: 2 },
            equip_slot: None,
            enabled: true,
        },
        RecordEraserPolicy::default(),
        RecordScope::Level,
        UpgradeFamily::Item,
    );
}
