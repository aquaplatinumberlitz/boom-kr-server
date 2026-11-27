use kr_service::accounts::{ECONOMY_AMOUNT_MAX, RewardAmount};
use kr_service::f4_extended_policy::*;
use serde::de::DeserializeOwned;
use serde_json::json;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Debug;

fn parse<T: DeserializeOwned>(value: serde_json::Value) -> T {
    serde_json::from_value(value).unwrap()
}

fn default_is_strict<T: DeserializeOwned + Default + PartialEq + Debug>() {
    assert_eq!(serde_json::from_str::<T>("{}").unwrap(), T::default());
    for invalid in [
        "null",
        "[]",
        "[false]",
        r#"{"enable":true}"#,
        r#"{"enabled":null}"#,
        r#"{"enabled":"true"}"#,
        r#"{"enabled":false,"enabled":true}"#,
    ] {
        assert!(serde_json::from_str::<T>(invalid).is_err(), "{invalid}");
    }
}

fn item(sku: &str) -> OwnedItem<'_> {
    OwnedItem {
        owner: 1,
        sku,
        quantity: 1,
        expires_at: None,
        origin: ItemOrigin::PersistentInventory,
    }
}

fn operation(id: u64) -> OperationKey {
    OperationKey { account: 1, id }
}

fn applied<T: Debug>(decision: Decision<T>) -> T {
    match decision {
        Decision::Apply(value) => value,
        Decision::Replay(value) => panic!("unexpected replay: {value:?}"),
    }
}

#[test]
fn defaults_are_strict_and_every_policy_starts_disabled() {
    default_is_strict::<PowerupKitPolicy>();
    default_is_strict::<BoosterTicketPolicy>();
    default_is_strict::<TitlePolicy>();
    default_is_strict::<LevelRewardPolicy>();
    default_is_strict::<LucciVaultPolicy>();
    default_is_strict::<RecordEraserPolicy>();
    default_is_strict::<GiftCouponPolicy>();
    default_is_strict::<SecondPasswordPolicy>();
    default_is_strict::<UpgradePolicy>();
    assert!(!PowerupKitPolicy::default().enabled);
    assert!(!BoosterTicketPolicy::default().enabled);
    assert!(!TitlePolicy::default().enabled);
    assert!(!LevelRewardPolicy::default().enabled);
    assert!(!LucciVaultPolicy::default().enabled);
    assert!(!RecordEraserPolicy::default().enabled);
    assert!(!GiftCouponPolicy::default().enabled);
    assert!(!SecondPasswordPolicy::default().enabled);
    assert!(!UpgradePolicy::default().enabled);
    assert_eq!(BoosterTicketPolicy::default().exp_percent, 100);
    assert_eq!(BoosterTicketPolicy::default().lucci_percent, 100);
    assert_eq!(TitlePolicy::default().equip_slot, None);
    assert_eq!(LevelRewardPolicy::default().level_up_lucci_table, None);
    assert_eq!(LucciVaultPolicy::default().base_lucci_cap, None);
    assert!(RecordEraserPolicy::default().require_audit);
    assert_eq!(SecondPasswordPolicy::default().max_failed_attempts, 5);
    assert_eq!(SecondPasswordPolicy::default().failure_window_seconds, 900);
    assert_eq!(SecondPasswordPolicy::default().cooldown_seconds, 300);
}

#[test]
fn configuration_rejects_invalid_values_and_nested_struct_shapes() {
    fn rejects<T: DeserializeOwned>(values: &[&str]) {
        for value in values {
            assert!(serde_json::from_str::<T>(value).is_err(), "{value}");
        }
    }

    rejects::<PowerupKitPolicy>(&[
        r#"{"max_equipped_slots":3}"#,
        r#"{"enabled":true}"#,
        r#"{"enabled":true,"max_equipped_slots":0,"kits":[{"sku":"a","bonus":{"bombs":1,"blast":0,"speed":0}}]}"#,
        r#"{"kits":[{"sku":"a","bonus":[2,0,0]}]}"#,
        r#"{"kits":[{"sku":"a","bonus":{"bombs":1,"blast":0,"speed":0,"cap":99}}]}"#,
        r#"{"kits":[{"sku":"a","bonus":{"bombs":1,"bombs":2,"blast":0,"speed":0}}]}"#,
    ]);
    rejects::<BoosterTicketPolicy>(&[
        r#"{"enabled":true}"#,
        r#"{"exp_percent":99}"#,
        r#"{"exp_percent":1001}"#,
        r#"{"exp_percent":150}"#,
        r#"{"bonus_cap":{"exp":1000000000001,"lucci":0}}"#,
        r#"{"paid_entitlements":true}"#,
        r#"{"allowed_rooms":[{"room_group":"g","mode":"m","maps":["a","a"]}]}"#,
        r#"{"allowed_rooms":[{"room_group":"g","mode":"m","maps":["a"],"bypass_room_rules":true}]}"#,
    ]);
    rejects::<TitlePolicy>(&[
        r#"{"enabled":true,"equip_slot":"title"}"#,
        r#"{"unlock_table":[{"title":"a","condition":"one"},{"title":"a","condition":"two"}]}"#,
    ]);
    rejects::<LevelRewardPolicy>(&[
        r#"{"enabled":true,"character_milestones":[]}"#,
        r#"{"level_up_lucci_table":[{"character":0,"level":1,"lucci":10}]}"#,
        r#"{"level_up_lucci_table":[{"character":0,"level":2,"lucci":10},{"character":0,"level":2,"lucci":11}]}"#,
        r#"{"character_milestones":[{"character":0,"level":2,"chest":{"sku":"c","quantity":0}}]}"#,
    ]);
    rejects::<LucciVaultPolicy>(&[
        r#"{"enabled":true,"base_lucci_cap":10}"#,
        r#"{"base_lucci_cap":10,"expansion_step":0,"max_expansions":1,"max_lucci_cap":10}"#,
        r#"{"base_lucci_cap":10,"expansion_step":2,"max_expansions":2,"max_lucci_cap":13}"#,
        r#"{"base_lucci_cap":1,"expansion_step":9223372036854775807,"max_expansions":2,"max_lucci_cap":9223372036854775807}"#,
        r#"{"base_lucci_cap":9223372036854775807,"expansion_step":1,"max_expansions":1,"max_lucci_cap":9223372036854775807}"#,
    ]);
    rejects::<RecordEraserPolicy>(&[
        r#"{"require_audit":false}"#,
        r#"{"allowed_scopes":["history"]}"#,
        r#"{"allowed_scopes":["ladder"],"erasers":[{"sku":"ticket","reset_to":{"level":1}}]}"#,
        r#"{"allowed_scopes":["win_loss"],"erasers":[{"sku":"ticket","reset_to":{"win_loss":[0,0]}}]}"#,
    ]);
    rejects::<GiftCouponPolicy>(&[
        r#"{"giftable_skus":["a"]}"#,
        r#"{"coupon_offers":[{"offer":"local","valid_for_seconds":0,"grant":{"sku":"a","quantity":1}}]}"#,
        r#"{"coupon_offers":[{"offer":"local","valid_for_seconds":10,"grant":["a",1]}]}"#,
        r#"{"o1_transferable":true}"#,
    ]);
    rejects::<SecondPasswordPolicy>(&[
        r#"{"max_failed_attempts":0}"#,
        r#"{"max_failed_attempts":6}"#,
        r#"{"failure_window_seconds":59}"#,
        r#"{"failure_window_seconds":901}"#,
        r#"{"cooldown_seconds":59}"#,
        r#"{"cooldown_seconds":901}"#,
    ]);
    rejects::<UpgradePolicy>(&[
        r#"{"enabled":true,"recipes":[]}"#,
        r#"{"allow_unconfigured_recipes":true}"#,
        r#"{"recipes":[{"family":"item","sku":"a","from_level":0,"to_level":1,"max_level":1,"materials":[],"lucci_cost":0,"success_bps":5000,"on_failure":"keep"}]}"#,
    ]);
}

fn kit_stats(bombs: u8, blast: u8, speed: u8) -> KitStats {
    KitStats {
        bombs,
        blast,
        speed,
    }
}

#[test]
fn kit_equip_preserves_slots_on_rejection_and_can_reuse_expired_slots() {
    let mut policy: PowerupKitPolicy = parse(json!({
        "enabled":true,
        "kits":[
            {"sku":"a","bonus":{"bombs":1,"blast":0,"speed":0}},
            {"sku":"b","bonus":{"bombs":0,"blast":1,"speed":0}},
            {"sku":"c","bonus":{"bombs":0,"blast":0,"speed":1}}
        ]
    }));
    let expiring = OwnedItem {
        expires_at: Some(10),
        ..item("a")
    };
    let empty = [None; MAX_EQUIPPED_KITS];
    assert_eq!(
        PowerupKitPolicy::default().equip(1, 0, &empty, expiring),
        Err(PolicyError::Disabled)
    );
    let first = policy.equip(1, 0, &empty, expiring).unwrap();
    let third = item("c");
    assert_eq!(
        policy.equip(1, 0, &first, expiring),
        Err(PolicyError::AlreadyEquipped)
    );
    assert_eq!(
        policy.equip(2, 0, &first, third),
        Err(PolicyError::NotOwnedOrExpired)
    );
    assert_eq!(
        policy.equip(1, 10, &first, expiring),
        Err(PolicyError::NotOwnedOrExpired)
    );
    assert_eq!(
        policy.equip(1, 0, &first, item("unknown")),
        Err(PolicyError::NotAllowed)
    );
    let full = policy.equip(1, 9, &first, item("b")).unwrap();
    assert_eq!(
        policy.equip(1, 9, &full, third),
        Err(PolicyError::CapacityExceeded)
    );
    assert_eq!(full, [Some(expiring), Some(item("b"))]);
    assert_eq!(
        policy.equip(1, 10, &full, third),
        Ok([Some(third), Some(item("b"))])
    );
    policy.max_equipped_slots = 1;
    assert_eq!(
        policy.equip(1, 10, &full, third),
        Err(PolicyError::CapacityExceeded)
    );
    policy.max_equipped_slots = 3;
    assert!(matches!(
        policy.equip(1, 10, &full, third),
        Err(PolicyError::InvalidConfig(_))
    ));
}

#[test]
fn kit_stats_respect_all_three_room_caps_and_ban() {
    let policy: PowerupKitPolicy = parse(json!({
        "enabled":true,
        "kits":[
            {"sku":"a","bonus":{"bombs":3,"blast":3,"speed":3}},
            {"sku":"b","bonus":{"bombs":255,"blast":255,"speed":255}}
        ]
    }));
    let timed = OwnedItem {
        expires_at: Some(4),
        ..item("b")
    };
    let slots = [Some(item("a")), Some(timed)];
    let base = kit_stats(2, 2, 2);
    let room = KitRoomRules {
        allows_kits: true,
        character_caps: kit_stats(3, 8, 9),
        mode_caps: kit_stats(9, 7, 9),
        map_caps: kit_stats(9, 8, 6),
    };
    assert_eq!(
        policy.starting_stats(1, 3, &slots, base, room),
        Ok(kit_stats(3, 7, 6))
    );
    assert_eq!(
        policy.starting_stats(1, 4, &slots, base, room),
        Ok(kit_stats(3, 5, 5))
    );
    assert_eq!(policy.starting_stats(2, 3, &slots, base, room), Ok(base));
    assert_eq!(
        policy.starting_stats(
            1,
            3,
            &slots,
            base,
            KitRoomRules {
                allows_kits: false,
                ..room
            }
        ),
        Ok(base)
    );
    assert_eq!(
        PowerupKitPolicy::default().starting_stats(1, 3, &slots, base, room),
        Ok(base)
    );
    assert_eq!(
        policy.starting_stats(1, 3, &slots, kit_stats(4, 2, 2), room),
        Err(PolicyError::InvalidInput)
    );
    let lower = PowerupKitPolicy {
        max_equipped_slots: 1,
        ..policy
    };
    assert_eq!(
        lower.starting_stats(1, 3, &slots, base, room),
        Err(PolicyError::CapacityExceeded)
    );
}

#[test]
fn booster_intersects_room_rules_and_settles_once_per_match_and_account() {
    let policy: BoosterTicketPolicy = parse(json!({
        "enabled":true, "exp_percent":150, "lucci_percent":200,
        "bonus_cap":{"exp":10,"lucci":3},
        "allowed_rooms":[{"room_group":"casual","mode":"normal","maps":["village"]}]
    }));
    let room = BoosterRoomContext {
        room_group: "casual",
        mode: "normal",
        map: "village",
        room_rules_allow: true,
        normal_unranked: true,
    };
    let ticket = BoosterTicket {
        account: 1,
        expires_at: 20,
        free_server_grant: true,
    };
    let key = SettlementKey {
        match_id: 1,
        account: 1,
    };
    let base = RewardAmount { exp: 3, lucci: 10 };
    let mut receipts = BTreeMap::new();
    assert_eq!(policy.allows_ticket_map(1, 19, ticket, room), Ok(true));
    for disallowed in [
        BoosterRoomContext {
            room_rules_allow: false,
            ..room
        },
        BoosterRoomContext {
            normal_unranked: false,
            ..room
        },
        BoosterRoomContext {
            room_group: "other",
            ..room
        },
        BoosterRoomContext {
            mode: "other",
            ..room
        },
        BoosterRoomContext {
            map: "other",
            ..room
        },
    ] {
        assert_eq!(
            policy.allows_ticket_map(1, 19, ticket, disallowed),
            Ok(false)
        );
        let settled = applied(
            policy
                .settle(key, 19, Some(ticket), disallowed, base, &receipts)
                .unwrap(),
        );
        assert_eq!(settled.total, base);
        assert_eq!(settled.bonus, RewardAmount::default());
    }
    assert_eq!(policy.allows_ticket_map(1, 20, ticket, room), Ok(false));
    assert_eq!(policy.allows_ticket_map(2, 19, ticket, room), Ok(false));
    let not_free = BoosterTicket {
        free_server_grant: false,
        ..ticket
    };
    assert_eq!(policy.allows_ticket_map(1, 19, not_free, room), Ok(false));
    let defaults = BoosterTicketPolicy::default();
    let neutral = applied(
        defaults
            .settle(key, 19, Some(ticket), room, base, &receipts)
            .unwrap(),
    );
    assert_eq!(neutral.total, base);
    assert_eq!(neutral.bonus, RewardAmount::default());

    let settled = applied(
        policy
            .settle(key, 19, Some(ticket), room, base, &receipts)
            .unwrap(),
    );
    assert_eq!(settled.bonus, RewardAmount { exp: 1, lucci: 3 });
    assert_eq!(settled.total, RewardAmount { exp: 4, lucci: 13 });
    receipts.insert(key, settled);
    assert_eq!(
        defaults.settle(key, 21, None, room, RewardAmount::default(), &receipts),
        Ok(Decision::Replay(settled))
    );
    let other_account = SettlementKey { account: 2, ..key };
    assert_eq!(
        policy.settle(other_account, 19, Some(ticket), room, base, &receipts),
        Ok(Decision::Apply(neutral))
    );
    receipts.insert(other_account, neutral);
    assert_eq!(
        policy.settle(
            other_account,
            19,
            Some(BoosterTicket {
                account: 2,
                ..ticket
            }),
            room,
            base,
            &receipts
        ),
        Ok(Decision::Replay(neutral))
    );
    let limit = RewardAmount {
        exp: ECONOMY_AMOUNT_MAX,
        lucci: ECONOMY_AMOUNT_MAX - 1,
    };
    let distinct = SettlementKey { match_id: 2, ..key };
    let capped = applied(
        policy
            .settle(distinct, 19, Some(ticket), room, limit, &receipts)
            .unwrap(),
    );
    assert_eq!(capped.bonus, RewardAmount { exp: 0, lucci: 1 });
    assert_eq!(
        capped.total,
        RewardAmount {
            exp: ECONOMY_AMOUNT_MAX,
            lucci: ECONOMY_AMOUNT_MAX
        }
    );
    assert_eq!(
        policy.settle(
            distinct,
            19,
            Some(ticket),
            room,
            RewardAmount {
                exp: u64::MAX,
                lucci: 0
            },
            &receipts
        ),
        Err(PolicyError::InvalidInput)
    );
}

#[test]
fn title_notification_only_follows_a_new_unlock() {
    let policy: TitlePolicy = parse(json!({
        "enabled":true, "equip_slot":"title", "notify_on_unlock":true,
        "unlock_table":[{"title":"explorer","condition":"maps_seen"}]
    }));
    let mut unlocked = BTreeSet::new();
    let conditions = BTreeSet::from(["maps_seen".into()]);
    assert_eq!(
        TitlePolicy::default().unlock("explorer", &unlocked, &conditions),
        Err(PolicyError::Disabled)
    );
    assert_eq!(
        policy.equip("explorer", &unlocked),
        Err(PolicyError::NotAllowed)
    );
    assert_eq!(
        policy.unlock("explorer", &unlocked, &BTreeSet::new()),
        Err(PolicyError::NotAllowed)
    );
    assert_eq!(
        policy.unlock("explorer", &unlocked, &conditions),
        Ok(TitleUnlockDecision::New {
            notify_after_commit: true
        })
    );
    unlocked.insert("explorer".into());
    assert_eq!(policy.equip("explorer", &unlocked), Ok("title"));
    assert_eq!(policy.equip("explorer", &unlocked), Ok("title"));
    assert_eq!(
        policy.unlock("explorer", &unlocked, &conditions),
        Ok(TitleUnlockDecision::AlreadyUnlocked)
    );
    assert_eq!(
        policy.equip("unknown", &unlocked),
        Err(PolicyError::NotAllowed)
    );
}

#[test]
fn level_jumps_claim_each_character_level_and_reward_kind_once() {
    let policy: LevelRewardPolicy = parse(json!({
        "enabled":true,
        "character_milestones":[
            {"character":0,"level":4,"chest":{"sku":"chest-4","quantity":1}},
            {"character":0,"level":2,"chest":{"sku":"chest-2","quantity":1}},
            {"character":1,"level":2,"chest":{"sku":"other","quantity":1}}
        ],
        "level_up_lucci_table":[
            {"character":0,"level":3,"lucci":20},{"character":0,"level":2,"lucci":10},
            {"character":0,"level":5,"lucci":30},{"character":1,"level":2,"lucci":40}
        ]
    }));
    let mut claims = BTreeSet::new();
    assert_eq!(
        LevelRewardPolicy::default().grants_for_level_gain(1, 0, 1, 2, &claims),
        Err(PolicyError::Disabled)
    );
    let grants = policy.grants_for_level_gain(1, 0, 1, 4, &claims).unwrap();
    assert_eq!(
        grants
            .iter()
            .map(|grant| (grant.claim.level, grant.claim.kind))
            .collect::<Vec<_>>(),
        [
            (2, MilestoneKind::Lucci),
            (2, MilestoneKind::Chest),
            (3, MilestoneKind::Lucci),
            (4, MilestoneKind::Chest)
        ]
    );
    assert_eq!(grants[0].reward, LevelReward::Lucci(10));
    claims.insert(grants[0].claim);
    assert_eq!(
        policy.grants_for_level_gain(1, 0, 1, 4, &claims).unwrap(),
        grants[1..]
    );
    claims.extend(grants.iter().map(|grant| grant.claim));
    assert!(
        policy
            .grants_for_level_gain(1, 0, 1, 4, &claims)
            .unwrap()
            .is_empty()
    );
    assert_eq!(
        policy.grants_for_level_gain(1, 0, 2, 5, &claims).unwrap()[0].reward,
        LevelReward::Lucci(30)
    );
    assert_eq!(
        policy
            .grants_for_level_gain(1, 1, 1, 2, &claims)
            .unwrap()
            .len(),
        2
    );
    assert_eq!(
        policy
            .grants_for_level_gain(2, 0, 1, 2, &claims)
            .unwrap()
            .len(),
        2
    );
    assert_eq!(
        policy.grants_for_level_gain(1, 0, 4, 3, &claims),
        Err(PolicyError::InvalidInput)
    );
    assert!(
        policy
            .grants_for_level_gain(1, 0, 4, 4, &claims)
            .unwrap()
            .is_empty()
    );
}

#[test]
fn vault_enforces_current_capacity_and_signed_storage_limit() {
    let vault: LucciVaultPolicy = parse(json!({
        "enabled":true, "base_lucci_cap":100, "expansion_step":25,
        "max_expansions":2, "max_lucci_cap":150
    }));
    assert_eq!(
        LucciVaultPolicy::default().capacity(0),
        Err(PolicyError::Disabled)
    );
    assert_eq!(vault.deposit(0, 99, 1), Ok(100));
    assert_eq!(vault.deposit(0, 99, 2), Err(PolicyError::CapacityExceeded));
    assert_eq!(vault.deposit(1, -1, 1), Err(PolicyError::InvalidInput));
    assert_eq!(vault.deposit(1, 10, -1), Err(PolicyError::InvalidInput));
    assert_eq!(vault.expand(0, 101), Err(PolicyError::CapacityExceeded));
    assert_eq!(
        vault.expand(1, 125),
        Ok(VaultExpansion {
            expansions: 2,
            capacity: 150
        })
    );
    assert_eq!(vault.expand(2, 150), Err(PolicyError::CapacityExceeded));
    assert_eq!(vault.capacity(3), Err(PolicyError::CapacityExceeded));
    let large = LucciVaultPolicy {
        base_lucci_cap: Some(i64::MAX - 1),
        expansion_step: Some(1),
        max_expansions: Some(1),
        max_lucci_cap: Some(i64::MAX),
        ..vault
    };
    assert_eq!(large.deposit(1, i64::MAX - 1, 1), Ok(i64::MAX));
    assert_eq!(large.deposit(1, i64::MAX, 1), Err(PolicyError::Overflow));
}

#[test]
fn record_erase_requires_scope_permission_active_item_and_audit() {
    let policy: RecordEraserPolicy = parse(json!({
        "enabled": true,
        "allowed_scopes": ["ladder", "win_loss"],
        "erasers": [
            {"sku":"ladder-pass","reset_to":{"ladder":1000}},
            {"sku":"record-pass","reset_to":{"win_loss":{"wins":0,"losses":0}}}
        ]
    }));
    let request = RecordEraseRequest {
        key: OperationKey { account: 2, id: 1 },
        actor: 1,
        authorized: true,
        audit_available: true,
        eraser: item("ladder-pass"),
        before: RecordValue::Ladder(1500),
        now: 10,
    };
    let mut receipts = BTreeMap::new();
    assert_eq!(
        RecordEraserPolicy::default().erase(request.clone(), &receipts),
        Err(PolicyError::Disabled)
    );
    for denied in [
        RecordEraseRequest {
            authorized: false,
            ..request.clone()
        },
        RecordEraseRequest {
            audit_available: false,
            ..request.clone()
        },
        RecordEraseRequest {
            before: RecordValue::Level(5),
            ..request.clone()
        },
        RecordEraseRequest {
            before: RecordValue::WinLoss(WinLossRecord {
                wins: 10,
                losses: 9,
            }),
            ..request.clone()
        },
    ] {
        assert_eq!(
            policy.erase(denied, &receipts),
            Err(PolicyError::NotAllowed)
        );
    }
    let expired = OwnedItem {
        expires_at: Some(10),
        ..request.eraser
    };
    let other_owner = OwnedItem {
        owner: 2,
        ..request.eraser
    };
    for eraser in [expired, other_owner] {
        assert_eq!(
            policy.erase(
                RecordEraseRequest {
                    eraser,
                    ..request.clone()
                },
                &receipts
            ),
            Err(PolicyError::NotOwnedOrExpired)
        );
    }
    let reset = applied(policy.erase(request.clone(), &receipts).unwrap());
    assert_eq!(reset.key, request.key);
    assert_eq!(reset.actor, 1);
    assert_eq!(reset.consume_sku, "ladder-pass");
    assert_eq!(reset.before, RecordValue::Ladder(1500));
    assert_eq!(reset.after, RecordValue::Ladder(1000));
    receipts.insert(request.key, reset.clone());
    assert_eq!(
        policy.erase(
            RecordEraseRequest {
                eraser: expired,
                before: reset.after.clone(),
                ..request.clone()
            },
            &receipts
        ),
        Ok(Decision::Replay(reset))
    );
    assert_eq!(
        policy.erase(
            RecordEraseRequest {
                actor: 3,
                ..request
            },
            &receipts
        ),
        Err(PolicyError::ReceiptConflict)
    );
}

fn gift_coupon_policy() -> GiftCouponPolicy {
    parse(json!({
        "enabled":true,
        "giftable_skus":["giftable"],
        "gift_limits":{"max_quantity":2,"max_per_window":3,"window_seconds":3600},
        "coupon_offers":[{"offer":"local-promo","valid_for_seconds":30,"grant":{"sku":"prize","quantity":2}}]
    }))
}

#[test]
fn gifting_checks_ownership_expiry_quota_and_request_identity() {
    let policy = gift_coupon_policy();
    let request = GiftRequest {
        key: operation(1),
        recipient: 2,
        quantity: 2,
        now: 10,
        gifts_in_window: 2,
        item: OwnedItem {
            expires_at: Some(20),
            quantity: 3,
            ..item("giftable")
        },
    };
    let mut receipts = BTreeMap::new();
    assert_eq!(
        GiftCouponPolicy::default().gift(request, &receipts),
        Err(PolicyError::Disabled)
    );
    assert_eq!(
        policy.gift(
            GiftRequest {
                recipient: 1,
                ..request
            },
            &receipts
        ),
        Err(PolicyError::InvalidInput)
    );
    assert_eq!(
        policy.gift(
            GiftRequest {
                gifts_in_window: 3,
                ..request
            },
            &receipts
        ),
        Err(PolicyError::CapacityExceeded)
    );
    assert_eq!(
        policy.gift(
            GiftRequest {
                quantity: 3,
                ..request
            },
            &receipts
        ),
        Err(PolicyError::CapacityExceeded)
    );
    assert_eq!(
        policy.gift(
            GiftRequest {
                item: item("blocked"),
                ..request
            },
            &receipts
        ),
        Err(PolicyError::NotAllowed)
    );
    for denied in [
        GiftRequest { now: 20, ..request },
        GiftRequest {
            item: OwnedItem {
                owner: 3,
                ..request.item
            },
            ..request
        },
        GiftRequest {
            item: OwnedItem {
                origin: ItemOrigin::SessionEntitlement,
                ..request.item
            },
            ..request
        },
    ] {
        assert_eq!(
            policy.gift(denied, &receipts),
            Err(PolicyError::NotOwnedOrExpired)
        );
    }
    let transfer = applied(policy.gift(request, &receipts).unwrap());
    assert_eq!(
        transfer,
        GiftCouponGrant::Gift {
            recipient: 2,
            item: ItemGrant {
                sku: "giftable".to_owned(),
                quantity: 2
            },
            expires_at: Some(20),
        }
    );
    receipts.insert(request.key, transfer.clone());
    assert_eq!(
        policy.gift(
            GiftRequest {
                now: 30,
                gifts_in_window: 9,
                ..request
            },
            &receipts
        ),
        Ok(Decision::Replay(transfer))
    );
    assert_eq!(
        policy.gift(
            GiftRequest {
                recipient: 4,
                ..request
            },
            &receipts
        ),
        Err(PolicyError::ReceiptConflict)
    );
}

#[test]
fn coupon_redemption_rejects_nonlocal_expired_or_previously_used_coupons() {
    let policy = gift_coupon_policy();
    let request = CouponRequest {
        key: operation(1),
        coupon_id: 7,
        offer: "local-promo",
        issued_to: 1,
        issued_at: 10,
        now: 39,
        locally_issued: true,
    };
    let mut receipts = BTreeMap::new();
    let mut used = BTreeSet::new();
    for denied in [
        CouponRequest {
            issued_to: 2,
            ..request
        },
        CouponRequest {
            locally_issued: false,
            ..request
        },
        CouponRequest {
            offer: "unknown",
            ..request
        },
    ] {
        assert_eq!(
            policy.redeem(denied, &used, &receipts),
            Err(PolicyError::NotAllowed)
        );
    }
    assert_eq!(
        policy.redeem(CouponRequest { now: 9, ..request }, &used, &receipts),
        Err(PolicyError::InvalidInput)
    );
    assert_eq!(
        policy.redeem(CouponRequest { now: 40, ..request }, &used, &receipts),
        Err(PolicyError::NotOwnedOrExpired)
    );
    let grant = applied(policy.redeem(request, &used, &receipts).unwrap());
    assert_eq!(
        grant,
        GiftCouponGrant::Coupon {
            coupon_id: 7,
            offer: "local-promo".to_owned(),
            item: ItemGrant {
                sku: "prize".to_owned(),
                quantity: 2
            },
        }
    );
    receipts.insert(request.key, grant.clone());
    used.insert(request.coupon_id);
    assert_eq!(
        policy.redeem(CouponRequest { now: 40, ..request }, &used, &receipts),
        Ok(Decision::Replay(grant))
    );
    assert_eq!(
        policy.redeem(
            CouponRequest {
                key: operation(2),
                ..request
            },
            &used,
            &receipts
        ),
        Err(PolicyError::AlreadyRedeemed)
    );
    assert_eq!(
        policy.redeem(
            CouponRequest {
                coupon_id: 8,
                ..request
            },
            &used,
            &receipts
        ),
        Err(PolicyError::ReceiptConflict)
    );
    let gift = GiftRequest {
        key: request.key,
        recipient: 2,
        item: item("giftable"),
        quantity: 1,
        gifts_in_window: 0,
        now: 39,
    };
    assert_eq!(
        policy.gift(gift, &receipts),
        Err(PolicyError::ReceiptConflict)
    );
}

fn primary(session: u64) -> Option<PrimarySession> {
    Some(PrimarySession {
        account: 1,
        session,
    })
}

#[test]
fn password_failures_span_sessions_and_actions_without_extending_lockout() {
    let policy = SecondPasswordPolicy {
        enabled: true,
        ..Default::default()
    };
    let mut state = SecondPasswordState::new(1, true).unwrap();
    use SecondPasswordAction as Action;
    use SecondPasswordOutcome as Outcome;
    for (index, action) in [
        Action::Unlock,
        Action::Change,
        Action::Disable,
        Action::Unlock,
        Action::Change,
    ]
    .into_iter()
    .enumerate()
    {
        let at = 100 + index as u64;
        let session = primary(index as u64 + 1);
        assert_eq!(policy.may_attempt(&state, session, at), Ok(true));
        let failed = policy.attempt(&state, session, at, action, false).unwrap();
        assert_eq!(
            failed.outcome,
            if at == 104 {
                Outcome::Locked
            } else {
                Outcome::Incorrect
            }
        );
        assert_eq!(policy.is_unlocked(&failed.state, session), Ok(false));
        state = failed.state;
    }
    assert_eq!(policy.may_attempt(&state, primary(99), 403), Ok(false));
    let during_lock = policy
        .attempt(&state, primary(99), 403, Action::Unlock, true)
        .unwrap();
    assert_eq!(during_lock.outcome, Outcome::Locked);
    assert_eq!(during_lock.state, state);
    let after_lock = policy
        .attempt(&state, primary(99), 404, Action::Unlock, false)
        .unwrap();
    assert_eq!(after_lock.outcome, Outcome::Incorrect);
    let verified = policy
        .attempt(&after_lock.state, primary(99), 405, Action::Unlock, true)
        .unwrap();
    assert_eq!(verified.outcome, Outcome::Unlocked);
    assert_eq!(policy.is_unlocked(&verified.state, primary(99)), Ok(true));
    assert_eq!(policy.is_unlocked(&verified.state, primary(1)), Ok(false));
    assert_eq!(
        policy.may_attempt(&verified.state, primary(1), 404),
        Err(PolicyError::InvalidInput)
    );
}

#[test]
fn password_session_lifecycle_and_exact_failure_window_are_fail_closed() {
    let policy = SecondPasswordPolicy {
        enabled: true,
        max_failed_attempts: 2,
        failure_window_seconds: 60,
        cooldown_seconds: 900,
    };
    let absent = SecondPasswordState::new(1, false).unwrap();
    use SecondPasswordAction as Action;
    use SecondPasswordOutcome as Outcome;
    assert_eq!(policy.setup(&absent, None), Err(PolicyError::NotAllowed));
    assert_eq!(
        SecondPasswordPolicy::default().setup(&absent, primary(1)),
        Err(PolicyError::Disabled)
    );
    let state = policy.setup(&absent, primary(1)).unwrap();
    assert_eq!(
        policy.setup(&state, primary(1)),
        Err(PolicyError::NotAllowed)
    );
    assert_eq!(policy.is_unlocked(&state, primary(1)), Ok(false));
    assert_eq!(
        policy.attempt(&state, None, 0, Action::Unlock, true),
        Err(PolicyError::NotAllowed)
    );
    let foreign = Some(PrimarySession {
        account: 2,
        session: 1,
    });
    assert_eq!(
        policy.attempt(&state, foreign, 0, Action::Unlock, true),
        Err(PolicyError::NotAllowed)
    );
    let first = policy
        .attempt(&state, primary(1), 100, Action::Unlock, false)
        .unwrap();
    let next = policy
        .attempt(&first.state, primary(2), 160, Action::Unlock, false)
        .unwrap();
    assert_eq!(next.outcome, Outcome::Incorrect);
    let first_unlocked = policy
        .attempt(&next.state, primary(1), 161, Action::Unlock, true)
        .unwrap();
    let failure = policy
        .attempt(
            &first_unlocked.state,
            primary(3),
            162,
            Action::Unlock,
            false,
        )
        .unwrap();
    assert_eq!(
        failure.outcome,
        Outcome::Incorrect,
        "success cleared previous failures"
    );
    let both = policy
        .attempt(&failure.state, primary(2), 163, Action::Unlock, true)
        .unwrap()
        .state;
    let relocked = both.revoke_session(primary(1).unwrap()).unwrap();
    assert_eq!(policy.is_unlocked(&relocked, primary(1)), Ok(false));
    assert_eq!(policy.is_unlocked(&relocked, primary(2)), Ok(true));
    assert_eq!(policy.is_unlocked(&relocked, foreign), Ok(false));
    assert_eq!(
        SecondPasswordPolicy::default().is_unlocked(&relocked, primary(2)),
        Ok(false)
    );
    let changed = policy
        .attempt(&both, primary(1), 164, Action::Change, true)
        .unwrap();
    assert_eq!(changed.outcome, Outcome::Changed);
    assert_eq!(policy.is_unlocked(&changed.state, primary(1)), Ok(false));
    assert_eq!(policy.is_unlocked(&changed.state, primary(2)), Ok(false));
    let disabled = policy
        .attempt(&both, primary(2), 165, Action::Disable, true)
        .unwrap();
    assert_eq!(disabled.outcome, Outcome::Disabled);
    assert_eq!(
        policy.may_attempt(&disabled.state, primary(1), 166),
        Err(PolicyError::NotAllowed)
    );
    assert_eq!(policy.is_unlocked(&disabled.state, primary(2)), Ok(false));
}

#[test]
fn upgrade_enforces_recipe_family_resources_and_single_outcome() {
    let policy: UpgradePolicy = parse(json!({
        "enabled":true,
        "recipes":[{"family":"pet","sku":"pet","from_level":1,"to_level":2,"max_level":2,
          "materials":[{"sku":"stone","quantity":3}],"lucci_cost":15,"success_bps":5000,
          "on_failure":"destroy","charge_on_failure":false}]
    }));
    let request = UpgradeRequest {
        key: operation(1),
        instance: 2,
        item: item("pet"),
        family: UpgradeFamily::Pet,
        level: 1,
        normal_persistent_inventory: true,
        now: 10,
    };
    let materials = [
        item("stone"),
        OwnedItem {
            quantity: 2,
            ..item("stone")
        },
    ];
    let resources = UpgradeResources {
        materials: &materials,
        lucci_balance: 15,
        roll_bps: 4999,
    };
    let mut receipts = BTreeMap::new();
    assert_eq!(
        UpgradePolicy::default().upgrade(request, resources, &receipts),
        Err(PolicyError::Disabled)
    );
    for denied in [
        UpgradeRequest {
            family: UpgradeFamily::Item,
            ..request
        },
        UpgradeRequest {
            level: 2,
            ..request
        },
        UpgradeRequest {
            normal_persistent_inventory: false,
            ..request
        },
    ] {
        assert_eq!(
            policy.upgrade(denied, resources, &receipts),
            Err(PolicyError::NotAllowed)
        );
    }
    let expired = OwnedItem {
        expires_at: Some(10),
        ..request.item
    };
    assert_eq!(
        policy.upgrade(
            UpgradeRequest {
                item: expired,
                ..request
            },
            resources,
            &receipts
        ),
        Err(PolicyError::NotOwnedOrExpired)
    );
    let insufficient = [
        item("stone"),
        OwnedItem {
            owner: 2,
            quantity: 3,
            ..item("stone")
        },
    ];
    assert_eq!(
        policy.upgrade(
            request,
            UpgradeResources {
                materials: &insufficient,
                ..resources
            },
            &receipts
        ),
        Err(PolicyError::NotOwnedOrExpired)
    );
    assert_eq!(
        policy.upgrade(
            request,
            UpgradeResources {
                lucci_balance: 14,
                ..resources
            },
            &receipts
        ),
        Err(PolicyError::CapacityExceeded)
    );
    assert_eq!(
        policy.upgrade(
            request,
            UpgradeResources {
                roll_bps: 10000,
                ..resources
            },
            &receipts
        ),
        Err(PolicyError::InvalidInput)
    );

    let success = applied(policy.upgrade(request, resources, &receipts).unwrap());
    assert_eq!(success.change, UpgradeChange::Level(2));
    assert_eq!(success.consume_materials[0].quantity, 3);
    assert_eq!(success.debit_lucci, 15);
    let failure_resources = UpgradeResources {
        roll_bps: 5000,
        ..resources
    };
    let failure = applied(
        policy
            .upgrade(request, failure_resources, &receipts)
            .unwrap(),
    );
    assert_eq!(failure.change, UpgradeChange::Destroy);
    assert!(failure.consume_materials.is_empty());
    assert_eq!(failure.debit_lucci, 0);
    receipts.insert(request.key, failure.clone());
    assert_eq!(
        policy.upgrade(
            UpgradeRequest {
                item: expired,
                ..request
            },
            UpgradeResources {
                materials: &[],
                lucci_balance: 0,
                ..resources
            },
            &receipts
        ),
        Ok(Decision::Replay(failure))
    );
    assert_eq!(
        policy.upgrade(
            UpgradeRequest {
                instance: 3,
                ..request
            },
            resources,
            &receipts
        ),
        Err(PolicyError::ReceiptConflict)
    );

    let mut changed = policy;
    changed.recipes[0].on_failure = UpgradeFailure::Keep;
    changed.recipes[0].charge_on_failure = true;
    let second = UpgradeRequest {
        key: operation(2),
        ..request
    };
    let charged = applied(
        changed
            .upgrade(second, failure_resources, &receipts)
            .unwrap(),
    );
    assert_eq!(charged.change, UpgradeChange::Keep);
    assert_eq!(charged.consume_materials, success.consume_materials);
    assert_eq!(charged.debit_lucci, success.debit_lucci);
    changed.recipes[0].max_level = 1;
    assert!(matches!(
        changed.upgrade(second, resources, &receipts),
        Err(PolicyError::InvalidConfig(_))
    ));
}
