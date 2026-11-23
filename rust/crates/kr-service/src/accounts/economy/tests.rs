use super::*;
use crate::accounts::tests::{Temp, fresh};
use std::{fs::OpenOptions, os::unix::fs::OpenOptionsExt};

const PASSWORD: &[u8] = b"test-only-password-one";
fn admin() -> Admin<'static> {
    Admin {
        actor: "F4-test",
        reason: "economy acceptance",
    }
}
fn policy() -> EconomyPolicy {
    EconomyPolicy {
        enabled: true,
        win: RewardAmount {
            exp: 250,
            lucci: 100,
        },
        loss: RewardAmount { exp: 50, lucci: 20 },
        draw: RewardAmount { exp: 80, lucci: 30 },
        grant_on_disconnect_or_left: false,
        level_thresholds: vec![0, 100, 200, 300],
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
                "char.bazzi",
                ShopCategory::Character,
                50,
                ItemGrant::Permanent,
                Some("character"),
                true,
            ),
            item(
                "hat.red",
                ShopCategory::Decoration,
                10,
                ItemGrant::Timed {
                    duration_seconds: 100,
                },
                Some("head"),
                true,
            ),
            item(
                "potion",
                ShopCategory::Consumable,
                5,
                ItemGrant::Stackable { max_stack: 3 },
                None,
                true,
            ),
            item(
                "emote.wave",
                ShopCategory::Emotion,
                1,
                ItemGrant::Permanent,
                Some("emote"),
                false,
            ),
            item(
                "hat.blue",
                ShopCategory::Collection,
                0,
                ItemGrant::Permanent,
                Some("head"),
                true,
            ),
        ],
    }
}
fn now(store: &Store) -> i64 {
    store
        .db
        .query_row("SELECT last_utc FROM runtime_state WHERE id=1", [], |r| {
            r.get(0)
        })
        .unwrap()
}
fn player(store: &mut Store, login: &str) -> Player {
    let at = now(store);
    store
        .create_account(&admin(), login, login, PASSWORD, at)
        .unwrap()
}
fn participant(p: &Player, slot: u8) -> MatchParticipant {
    MatchParticipant {
        account_id: p.account_id,
        player_id: p.player_id,
        slot,
        character_key: 1,
        team: slot,
    }
}
fn finished(
    store: &mut Store,
    players: &[&Player],
    round: i64,
    reason: MatchEndReason,
    outcomes: &[(MatchOutcome, MatchExit)],
) -> i64 {
    let at = now(store);
    let start = MatchStart {
        key: MatchKey {
            boot_epoch: 1,
            room_session: 1,
            room_id: 1,
            room_generation: 1,
            round,
        },
        map_code: 278,
        map_name: "ExVillage01".into(),
        started_at: at,
        players: players
            .iter()
            .enumerate()
            .map(|(i, p)| participant(p, i as u8))
            .collect(),
    };
    let record = store.begin_match(&start, at).unwrap();
    let finish = MatchFinish {
        ended_at: at,
        reason,
        players: outcomes
            .iter()
            .enumerate()
            .map(|(i, (outcome, exit))| MatchPlayerFinish {
                slot: i as u8,
                outcome: *outcome,
                exit: *exit,
            })
            .collect(),
    };
    store.finish_match(&start.key, &finish, at).unwrap();
    record.id
}
fn win_loss(store: &mut Store, a: &Player, b: &Player, round: i64, b_exit: MatchExit) -> i64 {
    finished(
        store,
        &[a, b],
        round,
        MatchEndReason::Elimination,
        &[
            (MatchOutcome::Win, MatchExit::Present),
            (MatchOutcome::Loss, b_exit),
        ],
    )
}
fn rows(db: &Connection, sql: &str) -> Vec<Vec<rusqlite::types::Value>> {
    let mut stmt = db.prepare(sql).unwrap();
    let n = stmt.column_count();
    stmt.query_map([], |r| {
        (0..n)
            .map(|i| r.get(i))
            .collect::<std::result::Result<Vec<_>, _>>()
    })
    .unwrap()
    .collect::<std::result::Result<_, _>>()
    .unwrap()
}
fn count(db: &Connection, table: &str) -> i64 {
    db.query_row(&format!("SELECT count(*) FROM {table}"), [], |r| r.get(0))
        .unwrap()
}
fn economy_state(db: &Connection) -> Vec<Vec<Vec<rusqlite::types::Value>>> {
    [
        "SELECT * FROM player_profiles ORDER BY player_id",
        "SELECT * FROM ledger_entries ORDER BY id",
        "SELECT * FROM player_inventory ORDER BY player_id,sku_id",
        "SELECT * FROM player_equipped ORDER BY player_id,slot",
    ]
    .iter()
    .map(|sql| rows(db, sql))
    .collect()
}
fn fund(store: &Store, p: &Player, lucci: i64, key: &str) {
    store
        .admin_grant_lucci(&admin(), p.player_id, lucci, key, 102)
        .unwrap();
}

#[test]
fn policy_and_catalog_validation_is_strict_and_default_is_disabled() {
    let default = EconomyPolicy::default();
    assert!(!default.enabled && !default.grant_on_disconnect_or_left);
    assert_eq!(default.validate(), Ok(()));
    assert_eq!(policy().validate(), Ok(()));
    for thresholds in [
        vec![],
        vec![1, 2],
        vec![0, 5, 5],
        vec![0, 9, 3],
        vec![0, u64::MAX],
    ] {
        let p = EconomyPolicy {
            level_thresholds: thresholds,
            ..policy()
        };
        assert_eq!(p.validate(), Err(Error::InvalidInput));
    }
    let p = EconomyPolicy {
        win: RewardAmount {
            exp: ECONOMY_AMOUNT_MAX + 1,
            lucci: 0,
        },
        ..policy()
    };
    assert_eq!(p.validate(), Err(Error::InvalidInput));
    assert!(
        serde_json::from_value::<EconomyPolicy>(serde_json::json!({
            "enabled":true,"win":{"exp":1,"lucci":1},"loss":{"exp":0,"lucci":0},
            "draw":{"exp":0,"lucci":0},"grant_on_disconnect_or_left":false,
            "level_thresholds":[0],"bonus":1
        }))
        .is_err()
    );
    let json = serde_json::to_value(policy()).unwrap();
    assert_eq!(
        serde_json::from_value::<EconomyPolicy>(json).unwrap(),
        policy()
    );
    assert_eq!(policy().level_for(0), 1);
    assert_eq!(policy().level_for(299), 3);
    assert_eq!(policy().level_for(u64::MAX), 4);

    assert_eq!(catalog().validate(), Ok(()));
    let mut c = catalog();
    c.items.push(c.items[0].clone());
    assert_eq!(c.validate(), Err(Error::InvalidInput));
    let edits: [fn(&mut ShopItem); 7] = [
        |i: &mut ShopItem| i.sku_id = "bad sku".into(),
        |i: &mut ShopItem| i.sku_id = "x".repeat(65),
        |i: &mut ShopItem| i.equip_slot = Some(String::new()),
        |i: &mut ShopItem| i.equip_slot = Some("s".repeat(33)),
        |i: &mut ShopItem| i.price_lucci = ECONOMY_AMOUNT_MAX + 1,
        |i: &mut ShopItem| i.grant = ItemGrant::Stackable { max_stack: 0 },
        |i: &mut ShopItem| {
            i.grant = ItemGrant::Timed {
                duration_seconds: 0,
            }
        },
    ];
    for edit in edits {
        let mut c = catalog();
        edit(&mut c.items[0]);
        assert_eq!(c.validate(), Err(Error::InvalidInput));
    }
    assert!(
        serde_json::from_value::<ShopCatalog>(serde_json::json!({"items":[{
            "sku_id":"a","category":"character","price_lucci":1,"grant":{"kind":"permanent"},
            "equip_slot":null,"enabled":true,"discount":1
        }]}))
        .is_err()
    );
    assert!(
        serde_json::from_value::<ItemGrant>(
            serde_json::json!({"kind":"timed","duration_seconds":1,"x":1})
        )
        .is_err()
    );
}

#[test]
fn match_settlement_is_atomic_idempotent_conflicting_and_levels_cap() {
    let _serial = TEST_SERIAL.lock().unwrap();
    let (_temp, mut store, a) = fresh();
    let b = player(&mut store, "bob");
    let m1 = win_loss(&mut store, &a, &b, 1, MatchExit::Present);
    let first = store.settle_match_rewards(m1, &policy(), 103).unwrap();
    assert!(first.settled && !first.replayed);
    assert_eq!(
        first
            .players
            .iter()
            .map(|p| (p.player_id, p.exp_delta, p.lucci_delta))
            .collect::<Vec<_>>(),
        vec![(a.player_id, 250, 100), (b.player_id, 50, 20)]
    );
    // One match jumps two levels (1 -> 3).
    assert_eq!(
        store.get_profile(a.player_id).unwrap(),
        PlayerProfile {
            player_id: a.player_id,
            exp: 250,
            level: 3,
            lucci: 100,
            version: 2
        }
    );
    let before = economy_state(&store.db);
    let replay = store.settle_match_rewards(m1, &policy(), 104).unwrap();
    assert!(replay.replayed && replay.settled);
    assert_eq!(replay.players, first.players);
    assert_eq!(replay.request_hash, first.request_hash);
    assert_eq!(economy_state(&store.db), before);
    let changed = EconomyPolicy {
        win: RewardAmount {
            exp: 251,
            lucci: 100,
        },
        ..policy()
    };
    assert_eq!(
        store.settle_match_rewards(m1, &changed, 104),
        Err(Error::Conflict)
    );
    let thresholds = EconomyPolicy {
        level_thresholds: vec![0, 10],
        ..policy()
    };
    assert_eq!(
        store.settle_match_rewards(m1, &thresholds, 104),
        Err(Error::Conflict)
    );
    assert_eq!(
        store.settle_match_rewards(m1, &EconomyPolicy::default(), 104),
        Err(Error::Conflict)
    );
    assert_eq!(economy_state(&store.db), before);
    // Two more wins cap at the maximum level while EXP keeps accumulating.
    for round in [2, 3] {
        let id = win_loss(&mut store, &a, &b, round, MatchExit::Present);
        store.settle_match_rewards(id, &policy(), 105).unwrap();
    }
    let capped = store.get_profile(a.player_id).unwrap();
    assert_eq!((capped.exp, capped.level, capped.lucci), (750, 4, 300));
    // M6 invariant is untouched: journal rewards remain zero.
    assert_eq!(
        store
            .db
            .query_row("SELECT max(reward) FROM match_players", [], |r| r
                .get::<_, i64>(0))
            .unwrap(),
        0
    );
    assert!(
        store
            .match_history(&admin(), HistoryFilter::Account(a.account_id), 10)
            .unwrap()
            .iter()
            .all(|m| m.reward_per_player == 0)
    );
    let ledger = store.list_ledger(a.player_id, 100).unwrap();
    assert_eq!(ledger.len(), 3);
    assert!(ledger.iter().all(|e| e.kind == "match_reward"
        && e.idempotency_key == format!("match:{}", e.match_id.unwrap())
        && e.request_hash.len() == 64));
    assert_eq!(store.list_ledger(a.player_id, 0), Err(Error::InvalidInput));
    assert_eq!(
        store.settle_match_rewards(999, &policy(), 106),
        Err(Error::InvalidInput)
    );
}

#[test]
fn zero_reward_paths_never_touch_profiles() {
    let _serial = TEST_SERIAL.lock().unwrap();
    let (_temp, mut store, a) = fresh();
    let b = player(&mut store, "bob");
    // Running match is rejected.
    let running = store
        .begin_match(
            &MatchStart {
                key: MatchKey {
                    boot_epoch: 1,
                    room_session: 2,
                    room_id: 2,
                    room_generation: 1,
                    round: 1,
                },
                map_code: 1,
                map_name: "Map".into(),
                started_at: 100,
                players: vec![participant(&a, 0), participant(&b, 1)],
            },
            100,
        )
        .unwrap();
    assert_eq!(
        store.settle_match_rewards(running.id, &policy(), 103),
        Err(Error::Conflict)
    );
    let cancelled = finished(
        &mut store,
        &[&a, &b],
        1,
        MatchEndReason::AdminClosed,
        &[
            (MatchOutcome::Cancelled, MatchExit::Present),
            (MatchOutcome::Cancelled, MatchExit::Left),
        ],
    );
    let disabled = win_loss(&mut store, &a, &b, 2, MatchExit::Present);
    let before = economy_state(&store.db);
    for (id, policy) in [
        (cancelled, policy()),
        (disabled, EconomyPolicy::default()),
        (
            disabled,
            EconomyPolicy {
                enabled: false,
                ..policy()
            },
        ),
    ] {
        let receipt = store.settle_match_rewards(id, &policy, 103).unwrap();
        assert!(!receipt.settled && receipt.request_hash.is_none());
        assert!(
            receipt
                .players
                .iter()
                .all(|p| p.exp_delta == 0 && p.lucci_delta == 0)
        );
    }
    assert_eq!(economy_state(&store.db), before);
    assert_eq!(count(&store.db, "player_profiles"), 0);
    // A disabled policy recorded nothing, so a later enabled settlement applies.
    store
        .settle_match_rewards(disabled, &policy(), 104)
        .unwrap();
    assert_eq!(store.get_profile(a.player_id).unwrap().exp, 250);

    // Left/disconnected players get zero unless the policy flag is set.
    let c = player(&mut store, "carol");
    let d = player(&mut store, "dave");
    let left = finished(
        &mut store,
        &[&a, &c, &d],
        3,
        MatchEndReason::Elimination,
        &[
            (MatchOutcome::Win, MatchExit::Present),
            (MatchOutcome::Loss, MatchExit::Left),
            (MatchOutcome::Loss, MatchExit::Disconnected),
        ],
    );
    let r = store.settle_match_rewards(left, &policy(), 105).unwrap();
    assert_eq!(
        r.players
            .iter()
            .map(|p| (p.exp_delta, p.lucci_delta))
            .collect::<Vec<_>>(),
        vec![(250, 100), (0, 0), (0, 0)]
    );
    for p in [&c, &d] {
        assert_eq!(store.get_profile(p.player_id).unwrap().version, 1);
    }
    assert_eq!(count(&store.db, "player_profiles"), 2);
    let generous = EconomyPolicy {
        grant_on_disconnect_or_left: true,
        ..policy()
    };
    assert_eq!(
        store.settle_match_rewards(left, &generous, 105),
        Err(Error::Conflict)
    );
    let again = finished(
        &mut store,
        &[&a, &c, &d],
        4,
        MatchEndReason::Elimination,
        &[
            (MatchOutcome::Win, MatchExit::Present),
            (MatchOutcome::Loss, MatchExit::Left),
            (MatchOutcome::Loss, MatchExit::Disconnected),
        ],
    );
    store.settle_match_rewards(again, &generous, 106).unwrap();
    for p in [&c, &d] {
        let profile = store.get_profile(p.player_id).unwrap();
        assert_eq!((profile.exp, profile.lucci, profile.version), (50, 20, 2));
    }
    // Draws use the draw reward.
    let draw = finished(
        &mut store,
        &[&c, &d],
        5,
        MatchEndReason::AllDead,
        &[
            (MatchOutcome::Draw, MatchExit::Present),
            (MatchOutcome::Draw, MatchExit::Present),
        ],
    );
    store.settle_match_rewards(draw, &policy(), 107).unwrap();
    assert_eq!(store.get_profile(c.player_id).unwrap().exp, 130);
}

#[test]
fn purchases_are_atomic_idempotent_and_enforce_balance_ownership_and_stack_rules() {
    let _serial = TEST_SERIAL.lock().unwrap();
    let (_temp, mut store, a) = fresh();
    let shop = catalog();
    // Insufficient balance is a redacted, side-effect-free denial.
    let before = economy_state(&store.db);
    let denied = store.purchase_shop_item(a.player_id, &shop, "char.bazzi", 1, "k1", 102);
    assert_eq!(denied, Err(Error::InsufficientFunds));
    assert_eq!(format!("{}", denied.unwrap_err()), "InsufficientFunds");
    assert_eq!(economy_state(&store.db), before);
    fund(&store, &a, 100, "fund-1");
    let bought = store
        .purchase_shop_item(a.player_id, &shop, "char.bazzi", 1, "k1", 103)
        .unwrap();
    assert!(!bought.replayed);
    assert_eq!((bought.lucci_spent, bought.profile.lucci), (50, 50));
    let state = economy_state(&store.db);
    let replay = store
        .purchase_shop_item(a.player_id, &shop, "char.bazzi", 1, "k1", 104)
        .unwrap();
    assert!(replay.replayed);
    assert_eq!(
        (replay.ledger_id, replay.lucci_spent, replay.profile.lucci),
        (bought.ledger_id, 50, 50)
    );
    assert_eq!(economy_state(&store.db), state);
    // Same key, different request.
    assert_eq!(
        store.purchase_shop_item(a.player_id, &shop, "potion", 1, "k1", 104),
        Err(Error::Conflict)
    );
    let mut repriced = shop.clone();
    repriced.items[0].price_lucci = 49;
    assert_eq!(
        store.purchase_shop_item(a.player_id, &repriced, "char.bazzi", 1, "k1", 104),
        Err(Error::Conflict)
    );
    // Permanent cannot be bought twice or with quantity != 1.
    assert_eq!(
        store.purchase_shop_item(a.player_id, &shop, "char.bazzi", 1, "k2", 104),
        Err(Error::Conflict)
    );
    assert_eq!(
        store.purchase_shop_item(a.player_id, &shop, "hat.blue", 2, "k2", 104),
        Err(Error::InvalidInput)
    );
    // Stack cap and invalid inputs.
    store
        .purchase_shop_item(a.player_id, &shop, "potion", 2, "k3", 104)
        .unwrap();
    assert_eq!(
        store.purchase_shop_item(a.player_id, &shop, "potion", 2, "k4", 104),
        Err(Error::Conflict)
    );
    let third = store
        .purchase_shop_item(a.player_id, &shop, "potion", 1, "k4", 104)
        .unwrap();
    assert_eq!(third.item.unwrap().quantity, 3);
    for (sku, qty, key) in [
        ("potion", 0, "k5"),
        ("missing", 1, "k5"),
        ("emote.wave", 1, "k5"),
        ("potion", 1, ""),
        ("potion", 1, "bad key"),
    ] {
        assert_eq!(
            store.purchase_shop_item(a.player_id, &shop, sku, qty, key, 104),
            Err(Error::InvalidInput),
            "{sku} {qty} {key}"
        );
    }
    let long = "k".repeat(129);
    assert_eq!(
        store.purchase_shop_item(a.player_id, &shop, "potion", 1, &long, 104),
        Err(Error::InvalidInput)
    );
    assert_eq!(
        store.purchase_shop_item(999, &shop, "potion", 1, "k5", 104),
        Err(Error::InvalidInput)
    );
    let profile = store.get_profile(a.player_id).unwrap();
    assert_eq!(profile.lucci, 100 - 50 - 15);
    // Disabled account is denied without changes.
    let state = economy_state(&store.db);
    store
        .set_disabled(&admin(), a.account_id, true, 105)
        .unwrap();
    assert_eq!(
        store.purchase_shop_item(a.player_id, &shop, "hat.red", 1, "k6", 105),
        Err(Error::InvalidCredentials)
    );
    assert_eq!(economy_state(&store.db), state);
    store
        .set_disabled(&admin(), a.account_id, false, 105)
        .unwrap();
    store
        .purchase_shop_item(a.player_id, &shop, "hat.red", 1, "k6", 105)
        .unwrap();
    // The ledger is the full balance history and is append-only.
    let ledger = store.list_ledger(a.player_id, 100).unwrap();
    assert_eq!(
        ledger.iter().map(|e| e.lucci_delta).sum::<i64>(),
        store.get_profile(a.player_id).unwrap().lucci
    );
    assert_eq!(
        store
            .db
            .execute("UPDATE ledger_entries SET lucci_delta=0", [])
            .map_err(Error::from),
        Err(Error::Conflict)
    );
    assert_eq!(
        store
            .db
            .execute("DELETE FROM ledger_entries", [])
            .map_err(Error::from),
        Err(Error::Conflict)
    );
}

#[test]
fn timed_items_extend_from_now_or_existing_expiry_with_checked_math() {
    let _serial = TEST_SERIAL.lock().unwrap();
    let (_temp, store, a) = fresh();
    let shop = catalog();
    fund(&store, &a, 1000, "fund");
    let first = store
        .purchase_shop_item(a.player_id, &shop, "hat.red", 1, "t1", 1000)
        .unwrap()
        .item
        .unwrap();
    assert_eq!(
        (first.expires_at, first.acquired_at, first.updated_at),
        (Some(1100), 1000, 1000)
    );
    // Still valid: extend from the future expiry.
    let extended = store
        .purchase_shop_item(a.player_id, &shop, "hat.red", 2, "t2", 1050)
        .unwrap()
        .item
        .unwrap();
    assert_eq!(
        (
            extended.expires_at,
            extended.acquired_at,
            extended.updated_at
        ),
        (Some(1300), 1000, 1050)
    );
    // Expired in the past: extend from now.
    let renewed = store
        .purchase_shop_item(a.player_id, &shop, "hat.red", 1, "t3", 5000)
        .unwrap()
        .item
        .unwrap();
    assert_eq!(
        (renewed.expires_at, renewed.acquired_at, renewed.updated_at),
        (Some(5100), 1000, 5000)
    );
    assert_eq!(store.get_profile(a.player_id).unwrap().lucci, 1000 - 40);
    // Overflow is rejected before any write.
    let state = economy_state(&store.db);
    assert_eq!(
        store.admin_grant_item(
            &admin(),
            a.player_id,
            "hat.red",
            ItemGrant::Timed {
                duration_seconds: ECONOMY_AMOUNT_MAX
            },
            u32::MAX,
            "huge",
            5001,
        ),
        Err(Error::Exhausted)
    );
    // A different grant kind for an owned SKU is a conflict.
    assert_eq!(
        store.admin_grant_item(
            &admin(),
            a.player_id,
            "hat.red",
            ItemGrant::Permanent,
            1,
            "kind",
            5001,
        ),
        Err(Error::Conflict)
    );
    assert_eq!(economy_state(&store.db), state);
}

#[test]
fn equip_requires_owned_active_item_in_its_catalog_slot_and_replaces_slot() {
    let _serial = TEST_SERIAL.lock().unwrap();
    let (_temp, mut store, a) = fresh();
    let b = player(&mut store, "bob");
    let shop = catalog();
    fund(&store, &a, 1000, "fund");
    assert_eq!(
        store.equip_item(a.player_id, &shop, "hat.red", "head", 200),
        Err(Error::Conflict)
    );
    store
        .purchase_shop_item(a.player_id, &shop, "hat.red", 1, "p1", 200)
        .unwrap();
    store
        .purchase_shop_item(a.player_id, &shop, "hat.blue", 1, "p2", 200)
        .unwrap();
    store
        .purchase_shop_item(a.player_id, &shop, "potion", 1, "p3", 200)
        .unwrap();
    // Not owned by another player, wrong slot, no slot, unknown SKU.
    assert_eq!(
        store.equip_item(b.player_id, &shop, "hat.red", "head", 201),
        Err(Error::Conflict)
    );
    for (sku, slot) in [
        ("hat.red", "character"),
        ("potion", "head"),
        ("missing", "head"),
        ("hat.red", ""),
    ] {
        assert_eq!(
            store.equip_item(a.player_id, &shop, sku, slot, 201),
            Err(Error::InvalidInput)
        );
    }
    let equipped = store
        .equip_item(a.player_id, &shop, "hat.red", "head", 201)
        .unwrap();
    assert_eq!(equipped.sku_id, "hat.red");
    assert_eq!(
        store
            .equip_item(a.player_id, &shop, "hat.red", "head", 202)
            .unwrap(),
        equipped
    );
    // Replacing the slot.
    store
        .equip_item(a.player_id, &shop, "hat.blue", "head", 203)
        .unwrap();
    assert_eq!(
        store.list_equipped(a.player_id).unwrap(),
        vec![EquippedItem {
            slot: "head".into(),
            sku_id: "hat.blue".into(),
            equipped_at: 203
        }]
    );
    let inventory = store.list_inventory(a.player_id, 203).unwrap();
    let blue = inventory.iter().find(|i| i.sku_id == "hat.blue").unwrap();
    assert_eq!(blue.equipped_slots, vec!["head".to_owned()]);
    // Still valid one second before expiry; expires_at == now is expired.
    store
        .equip_item(a.player_id, &shop, "hat.red", "head", 299)
        .unwrap();
    assert_eq!(
        store.equip_item(a.player_id, &shop, "hat.red", "head", 300),
        Err(Error::Conflict)
    );
    assert!(
        !store.list_inventory(a.player_id, 300).unwrap()[1].active,
        "hat.red expired"
    );
    assert_eq!(store.unequip_slot(a.player_id, "head", 301), Ok(true));
    assert_eq!(store.unequip_slot(a.player_id, "head", 301), Ok(false));
    assert!(store.list_equipped(a.player_id).unwrap().is_empty());
    assert_eq!(
        store.unequip_slot(a.player_id, "bad slot", 301),
        Err(Error::InvalidInput)
    );
}

#[test]
fn o1_unlock_all_uses_enabled_catalog_without_charging_or_writing() {
    let _serial = TEST_SERIAL.lock().unwrap();
    let (_temp, store, a) = fresh();
    let shop = catalog();
    fund(&store, &a, 100, "fund");
    store
        .purchase_shop_item(a.player_id, &shop, "hat.red", 1, "p1", 200)
        .unwrap();
    store
        .purchase_shop_item(a.player_id, &shop, "potion", 2, "p2", 200)
        .unwrap();
    let state = economy_state(&store.db);
    let runtime = rows(&store.db, "SELECT * FROM runtime_state");
    let unlocked = CatalogAccess { unlock_all: true };
    let all = store
        .usable_items(a.player_id, &shop, unlocked, 400)
        .unwrap();
    let skus: Vec<_> = all.iter().map(|i| i.sku_id.as_str()).collect();
    assert_eq!(skus, vec!["char.bazzi", "hat.red", "potion", "hat.blue"]);
    assert!(all.iter().all(|i| i.unlocked_by_session));
    assert!(
        store
            .can_use_sku(a.player_id, &shop, unlocked, "char.bazzi", 400)
            .unwrap()
    );
    assert!(
        !store
            .can_use_sku(a.player_id, &shop, unlocked, "emote.wave", 400)
            .unwrap()
    );
    let owned = CatalogAccess { unlock_all: false };
    let mine = store.usable_items(a.player_id, &shop, owned, 250).unwrap();
    assert_eq!(
        mine.iter().map(|i| i.sku_id.as_str()).collect::<Vec<_>>(),
        vec!["hat.red", "potion"]
    );
    let later = store.usable_items(a.player_id, &shop, owned, 300).unwrap();
    assert_eq!(
        later.iter().map(|i| i.sku_id.as_str()).collect::<Vec<_>>(),
        vec!["potion"]
    );
    assert!(
        !store
            .can_use_sku(a.player_id, &shop, owned, "char.bazzi", 250)
            .unwrap()
    );
    assert_eq!(economy_state(&store.db), state);
    assert_eq!(rows(&store.db, "SELECT * FROM runtime_state"), runtime);
    assert_eq!(store.get_profile(a.player_id).unwrap().lucci, 100 - 20);
}

#[test]
fn admin_grants_are_audited_idempotent_and_conflict_on_changed_request() {
    let _serial = TEST_SERIAL.lock().unwrap();
    let (_temp, store, a) = fresh();
    let audits = |op: &str| -> i64 {
        store
            .db
            .query_row(
                "SELECT count(*) FROM admin_audit WHERE operation=?1 AND target_kind='player' AND target_id=?2",
                params![op, a.player_id.to_string()],
                |r| r.get(0),
            )
            .unwrap()
    };
    let g = store
        .admin_grant_lucci(&admin(), a.player_id, 500, "g1", 102)
        .unwrap();
    assert!(!g.replayed && g.profile.lucci == 500);
    let r = store
        .admin_grant_lucci(&admin(), a.player_id, 500, "g1", 103)
        .unwrap();
    assert!(r.replayed && r.ledger_id == g.ledger_id && r.profile.lucci == 500);
    assert_eq!(
        store.admin_grant_lucci(&admin(), a.player_id, 501, "g1", 103),
        Err(Error::Conflict)
    );
    assert_eq!(
        store.admin_grant_lucci(&admin(), a.player_id, -501, "g2", 103),
        Err(Error::InsufficientFunds)
    );
    assert_eq!(
        store
            .admin_grant_lucci(&admin(), a.player_id, -100, "g2", 103)
            .unwrap()
            .profile
            .lucci,
        400
    );
    assert_eq!(
        store.admin_grant_lucci(&admin(), a.player_id, 0, "g3", 103),
        Err(Error::InvalidInput)
    );
    assert_eq!(audits("admin_grant_lucci"), 2);
    let item = store
        .admin_grant_item(
            &admin(),
            a.player_id,
            "potion",
            ItemGrant::Stackable { max_stack: 5 },
            2,
            "i1",
            104,
        )
        .unwrap();
    assert_eq!(item.item.as_ref().unwrap().quantity, 2);
    let again = store
        .admin_grant_item(
            &admin(),
            a.player_id,
            "potion",
            ItemGrant::Stackable { max_stack: 5 },
            2,
            "i1",
            105,
        )
        .unwrap();
    assert!(again.replayed);
    assert_eq!(again.item.unwrap().quantity, 2);
    assert_eq!(
        store.admin_grant_item(
            &admin(),
            a.player_id,
            "potion",
            ItemGrant::Stackable { max_stack: 5 },
            3,
            "i1",
            105
        ),
        Err(Error::Conflict)
    );
    // Lucci and item grants share the admin_grant key space.
    assert_eq!(
        store.admin_grant_item(
            &admin(),
            a.player_id,
            "potion",
            ItemGrant::Stackable { max_stack: 5 },
            1,
            "g1",
            105
        ),
        Err(Error::Conflict)
    );
    assert_eq!(audits("admin_grant_item"), 1);
    assert_eq!(
        store.admin_grant_lucci(&admin(), 999, 1, "x", 105),
        Err(Error::InvalidInput)
    );
    let bad = Admin {
        actor: "",
        reason: "r",
    };
    assert_eq!(
        store.admin_grant_lucci(&bad, a.player_id, 1, "x", 105),
        Err(Error::InvalidInput)
    );
}

#[test]
fn tampered_balance_fails_schema_validation_and_v7_snapshot_restore_round_trips() {
    let _serial = TEST_SERIAL.lock().unwrap();
    let (temp, mut store, a) = fresh();
    let b = player(&mut store, "bob");
    let shop = catalog();
    let m = win_loss(&mut store, &a, &b, 1, MatchExit::Present);
    store.settle_match_rewards(m, &policy(), 102).unwrap();
    store
        .purchase_shop_item(a.player_id, &shop, "hat.blue", 1, "p1", 103)
        .unwrap();
    store
        .purchase_shop_item(a.player_id, &shop, "hat.red", 1, "p2", 103)
        .unwrap();
    store
        .equip_item(a.player_id, &shop, "hat.red", "head", 104)
        .unwrap();
    let state = economy_state(&store.db);
    assert_eq!(count(&store.db, "player_equipped"), 1);
    let db = temp.0.join("accounts.sqlite");
    let saved = temp.0.join("saved.sqlite");
    assert_eq!(snapshot(&db, &saved), Ok(SCHEMA_VERSION));
    assert_eq!(verify_snapshot(&saved), Ok(7));
    let target = Temp::new();
    let restored = target.0.join("restored.sqlite");
    assert_eq!(restore(&saved, &restored), Ok(7));
    let reopened = Store::open(&restored).unwrap();
    assert_eq!(economy_state(&reopened.db), state);
    assert_eq!(reopened.get_profile(a.player_id).unwrap().level, 3);
    let integrity: String = reopened
        .db
        .query_row("PRAGMA integrity_check", [], |r| r.get(0))
        .unwrap();
    assert_eq!(integrity, "ok");
    assert!(
        !reopened
            .db
            .prepare("PRAGMA foreign_key_check")
            .unwrap()
            .exists([])
            .unwrap()
    );
    drop(reopened);
    // A balance edited outside the ledger is rejected on open and in backups.
    let edit = Connection::open(&restored).unwrap();
    edit.execute("UPDATE player_profiles SET lucci=lucci+1", [])
        .unwrap();
    drop(edit);
    assert!(matches!(Store::open(&restored), Err(Error::Schema)));
    let tampered = target.0.join("tampered.sqlite");
    assert_eq!(snapshot(&restored, &tampered), Err(Error::Schema));
}

fn v6_fixture(path: &std::path::Path) -> Connection {
    let _file = OpenOptions::new()
        .create_new(true)
        .write(true)
        .mode(0o600)
        .open(path)
        .unwrap();
    let db = Connection::open(path).unwrap();
    for (i, script) in SCRIPTS.iter().take(6).enumerate() {
        db.execute_batch(script).unwrap();
        db.execute(
            "INSERT INTO schema_migrations VALUES(?1,?2)",
            params![i as i64 + 1, hex(&Sha256::digest(script.as_bytes()))],
        )
        .unwrap();
    }
    db.pragma_update(None, "application_id", APPLICATION_ID)
        .unwrap();
    db.pragma_update(None, "user_version", 6).unwrap();
    let phc = password_hash(PASSWORD, b"fixture-v6-salt0").unwrap();
    db.execute_batch(&format!(
        "UPDATE runtime_state SET epoch=1,last_utc=100,next_wire_id=50 WHERE id=1;
         INSERT INTO accounts(id,login,password_phc,created_at) VALUES(3,'alice','{phc}',90),(4,'bob','{phc}',91);
         INSERT INTO players(id,account_id,wire_id,display_name,profile_version) VALUES(11,3,40,'Alice',2),(12,4,41,'Bob',1);
         INSERT INTO player_preferences VALUES(11,5,3);
         INSERT INTO matches(id,boot_epoch,room_session,room_id,room_generation,round,map_code,map_name,started_at,ended_at,state,reason,start_json,finish_json)
           VALUES(21,1,1,1,1,1,278,'ExVillage01',95,96,'complete','elimination','{{}}','{{}}');
         INSERT INTO match_players VALUES(21,3,11,0,1,0,'win','present',0),(21,4,12,1,0,1,'loss','present',0);
         INSERT INTO admin_audit(request_id,actor,operation,account_id,reason,at_utc,details,target_kind,target_id,result)
           VALUES('{}','test','create_account',3,'fixture',90,'{{}}','account','3','ok');",
        "c".repeat(64)
    ))
    .unwrap();
    db
}
fn v6_data(db: &Connection) -> Vec<Vec<Vec<rusqlite::types::Value>>> {
    [
        "SELECT * FROM accounts ORDER BY id",
        "SELECT * FROM players ORDER BY id",
        "SELECT * FROM runtime_state",
        "SELECT * FROM player_preferences ORDER BY player_id",
        "SELECT * FROM matches ORDER BY id",
        "SELECT * FROM match_players ORDER BY match_id,slot",
        "SELECT * FROM admin_audit ORDER BY id",
        "SELECT * FROM sessions ORDER BY id",
    ]
    .iter()
    .map(|sql| rows(db, sql))
    .collect()
}

#[test]
fn schema6_upgrade_backs_up_first_and_preserves_every_old_row() {
    let _serial = TEST_SERIAL.lock().unwrap();
    let temp = Temp::new();
    let path = temp.0.join("accounts.sqlite");
    let db = v6_fixture(&path);
    let before = v6_data(&db);
    drop(db);
    let store = Store::open(&path).unwrap();
    assert_eq!(
        store
            .db
            .pragma_query_value::<i32, _>(None, "user_version", |r| r.get(0))
            .unwrap(),
        7
    );
    assert_eq!(v6_data(&store.db), before);
    let backups: Vec<_> = std::fs::read_dir(&temp.0)
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| {
            p.file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .starts_with("accounts.sqlite.before-v6-to-v7-")
        })
        .collect();
    assert_eq!(backups.len(), 1);
    assert_eq!(verify_snapshot(&backups[0]), Ok(6));
    let backup =
        Connection::open_with_flags(&backups[0], OpenFlags::SQLITE_OPEN_READ_ONLY).unwrap();
    assert_eq!(v6_data(&backup), before);
    assert!(backup.prepare("SELECT 1 FROM player_profiles").is_err());
    drop(backup);
    // Upgraded players start from the default profile and can be settled.
    assert_eq!(
        store.get_profile(11).unwrap(),
        PlayerProfile {
            player_id: 11,
            exp: 0,
            level: 1,
            lucci: 0,
            version: 1
        }
    );
    store.settle_match_rewards(21, &policy(), 200).unwrap();
    assert_eq!(store.get_profile(12).unwrap().lucci, 20);
    drop(store);
    // Reopening a current DB takes no new backup.
    drop(Store::open(&path).unwrap());
    assert_eq!(
        std::fs::read_dir(&temp.0)
            .unwrap()
            .filter(|e| e
                .as_ref()
                .unwrap()
                .file_name()
                .to_str()
                .unwrap()
                .contains(".before-v"))
            .count(),
        1
    );
}
