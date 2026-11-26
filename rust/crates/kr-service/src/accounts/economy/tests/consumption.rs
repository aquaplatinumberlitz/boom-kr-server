use super::*;

fn running(store: &mut Store, a: &Player, b: &Player) -> MatchRecord {
    let at = now(store);
    store
        .begin_match(
            &MatchStart {
                key: MatchKey {
                    boot_epoch: 1,
                    room_session: 1,
                    room_id: 1,
                    room_generation: 1,
                    round: 1,
                },
                map_code: 278,
                map_name: "ExVillage01".into(),
                started_at: at,
                players: vec![participant(a, 0), participant(b, 1)],
            },
            at,
        )
        .unwrap()
}

fn request(player: &Player, record: &MatchRecord) -> MatchItemConsumptionRequest {
    MatchItemConsumptionRequest {
        player_id: player.player_id,
        match_id: record.id,
        sku_id: "potion".into(),
        quantity: 1,
        idempotency_key: "round1:action1".into(),
        request_hash: "a".repeat(64),
    }
}

fn equipped_stack(store: &Store, player: &Player, shop: &mut ShopCatalog, qty: u32) {
    let potion = shop
        .items
        .iter_mut()
        .find(|i| i.sku_id == "potion")
        .unwrap();
    potion.equip_slot = Some("quick".into());
    store
        .admin_grant_item(
            &admin(),
            player.player_id,
            "potion",
            potion.grant,
            qty,
            "grant:potion",
            101,
        )
        .unwrap();
    store
        .equip_item(player.player_id, shop, "potion", "quick", 101)
        .unwrap();
}

#[test]
fn schema7_upgrade_preserves_inventory_and_equipment_and_enables_consumption() {
    let _serial = TEST_SERIAL.lock().unwrap();
    let temp = Temp::new();
    let path = temp.0.join("accounts.sqlite");
    let db = old_schema_fixture(&path, 7);
    db.execute_batch(
        "INSERT INTO player_inventory VALUES
           (11,'potion','stackable',3,NULL,98,99),(12,'char.bazzi','permanent',1,NULL,98,98);
         INSERT INTO player_equipped VALUES (11,'quick','potion',99),(12,'character','char.bazzi',99);",
    )
    .unwrap();
    let before = economy_state(&db);
    drop(db);

    let mut store = Store::open(&path).unwrap();
    assert_eq!(
        store
            .db
            .pragma_query_value::<i32, _>(None, "user_version", |r| r.get(0))
            .unwrap(),
        crate::accounts::SCHEMA_VERSION
    );
    assert_eq!(economy_state(&store.db), before);
    assert_eq!(count(&store.db, "match_item_consumptions"), 0);
    let backup = std::fs::read_dir(&temp.0)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .find(|p| {
            p.file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .starts_with(&format!(
                    "accounts.sqlite.before-v7-to-v{}-",
                    crate::accounts::SCHEMA_VERSION
                ))
        })
        .unwrap();
    assert_eq!(verify_snapshot(&backup), Ok(7));
    let saved = Connection::open_with_flags(backup, OpenFlags::SQLITE_OPEN_READ_ONLY).unwrap();
    assert_eq!(economy_state(&saved), before);

    let record = store
        .begin_match(
            &MatchStart {
                key: MatchKey {
                    boot_epoch: 1,
                    room_session: 1,
                    room_id: 1,
                    room_generation: 1,
                    round: 2,
                },
                map_code: 278,
                map_name: "ExVillage01".into(),
                started_at: 100,
                players: vec![
                    MatchParticipant {
                        account_id: 3,
                        player_id: 11,
                        slot: 0,
                        character_key: 0,
                        team: 0,
                    },
                    MatchParticipant {
                        account_id: 4,
                        player_id: 12,
                        slot: 1,
                        character_key: 0,
                        team: 1,
                    },
                ],
            },
            100,
        )
        .unwrap();
    let receipt = store
        .consume_item_in_match(
            &MatchItemConsumptionRequest {
                player_id: 11,
                match_id: record.id,
                sku_id: "potion".into(),
                quantity: 3,
                idempotency_key: "round2:action1".into(),
                request_hash: "a".repeat(64),
            },
            101,
        )
        .unwrap();
    assert_eq!(receipt.remaining_qty, 0);
    assert!(store.list_inventory(11, 101).unwrap().is_empty());
    assert!(store.list_equipped(11).unwrap().is_empty());
    assert_eq!(store.list_equipped(12).unwrap()[0].sku_id, "char.bazzi");
    assert!(
        !store
            .db
            .prepare("PRAGMA foreign_key_check")
            .unwrap()
            .exists([])
            .unwrap()
    );
}

#[test]
fn partial_and_full_use_preserve_receipts_for_replay_after_the_match() {
    let _serial = TEST_SERIAL.lock().unwrap();
    let (temp, mut store, a) = fresh();
    let b = player(&mut store, "bob");
    let mut shop = catalog();
    equipped_stack(&store, &a, &mut shop, 3);
    // A catalog change can leave an earlier slot referencing the same stack.
    shop.items
        .iter_mut()
        .find(|i| i.sku_id == "potion")
        .unwrap()
        .equip_slot = Some("backup".into());
    store
        .equip_item(a.player_id, &shop, "potion", "backup", 101)
        .unwrap();
    let record = running(&mut store, &a, &b);
    let first = request(&a, &record);

    let r1 = store.consume_item_in_match(&first, 102).unwrap();
    assert_eq!(
        (
            r1.consumed_qty,
            r1.remaining_qty,
            r1.created_at,
            r1.replayed
        ),
        (1, 2, 102, false)
    );
    let inventory = store.list_inventory(a.player_id, 102).unwrap();
    assert_eq!(
        (
            inventory[0].quantity,
            inventory[0].acquired_at,
            inventory[0].updated_at
        ),
        (2, 101, 102)
    );
    assert_eq!(inventory[0].equipped_slots, ["backup", "quick"]);
    assert_eq!(
        store.consume_item_in_match(&first, 103).unwrap(),
        MatchItemConsumptionReceipt {
            replayed: true,
            ..r1.clone()
        }
    );
    assert_eq!(now(&store), 102);
    let final_use = MatchItemConsumptionRequest {
        quantity: 2,
        idempotency_key: "round1:action2".into(),
        request_hash: "b".repeat(64),
        ..first.clone()
    };
    let r2 = store.consume_item_in_match(&final_use, 103).unwrap();
    assert_eq!((r2.consumed_qty, r2.remaining_qty), (2, 0));
    assert!(store.list_inventory(a.player_id, 103).unwrap().is_empty());
    assert!(store.list_equipped(a.player_id).unwrap().is_empty());
    store
        .finish_match(
            &record.start.key,
            &MatchFinish {
                ended_at: 104,
                reason: MatchEndReason::AllDead,
                players: [0, 1]
                    .map(|slot| MatchPlayerFinish {
                        slot,
                        outcome: MatchOutcome::Draw,
                        exit: MatchExit::Present,
                    })
                    .to_vec(),
            },
            104,
        )
        .unwrap();
    drop(store);
    let store = temp.store();
    for (request, receipt) in [(&first, r1), (&final_use, r2)] {
        assert_eq!(
            store.consume_item_in_match(request, 105).unwrap(),
            MatchItemConsumptionReceipt {
                replayed: true,
                ..receipt
            }
        );
    }
    for conflict in [
        MatchItemConsumptionRequest {
            request_hash: "c".repeat(64),
            ..first.clone()
        },
        MatchItemConsumptionRequest {
            quantity: 2,
            ..first.clone()
        },
        MatchItemConsumptionRequest {
            match_id: record.id + 1,
            ..first.clone()
        },
    ] {
        assert_eq!(
            store.consume_item_in_match(&conflict, 105),
            Err(Error::Conflict)
        );
    }
    let late = MatchItemConsumptionRequest {
        idempotency_key: "late".into(),
        ..first
    };
    assert_eq!(
        store.consume_item_in_match(&late, 105),
        Err(Error::InvalidSession)
    );
    assert_eq!(count(&store.db, "match_item_consumptions"), 2);
    assert_eq!(count(&store.db, "ledger_entries"), 1);
}

#[test]
fn denied_uses_and_failed_receipt_insert_leave_the_stack_and_slots_unchanged() {
    let _serial = TEST_SERIAL.lock().unwrap();
    let (_temp, mut store, a) = fresh();
    let b = player(&mut store, "bob");
    let outsider = player(&mut store, "carol");
    let mut shop = catalog();
    equipped_stack(&store, &a, &mut shop, 2);
    for sku in ["char.bazzi", "hat.red"] {
        let item = shop.item(sku).unwrap();
        store
            .admin_grant_item(&admin(), a.player_id, sku, item.grant, 1, sku, 101)
            .unwrap();
    }
    let record = running(&mut store, &a, &b);
    let first = request(&a, &record);
    let before = economy_state(&store.db);
    let at = now(&store);
    for (invalid, error) in [
        (
            MatchItemConsumptionRequest {
                quantity: 0,
                ..first.clone()
            },
            Error::InvalidInput,
        ),
        (
            MatchItemConsumptionRequest {
                idempotency_key: "bad key".into(),
                ..first.clone()
            },
            Error::InvalidInput,
        ),
        (
            MatchItemConsumptionRequest {
                request_hash: "A".repeat(64),
                ..first.clone()
            },
            Error::InvalidInput,
        ),
        (
            MatchItemConsumptionRequest {
                request_hash: "f".repeat(63),
                ..first.clone()
            },
            Error::InvalidInput,
        ),
        (
            MatchItemConsumptionRequest {
                quantity: 3,
                ..first.clone()
            },
            Error::InsufficientQuantity,
        ),
        (
            MatchItemConsumptionRequest {
                player_id: b.player_id,
                ..first.clone()
            },
            Error::InsufficientQuantity,
        ),
        (
            MatchItemConsumptionRequest {
                sku_id: "char.bazzi".into(),
                ..first.clone()
            },
            Error::InvalidInput,
        ),
        (
            MatchItemConsumptionRequest {
                sku_id: "hat.red".into(),
                ..first.clone()
            },
            Error::InvalidInput,
        ),
        (
            MatchItemConsumptionRequest {
                player_id: outsider.player_id,
                ..first.clone()
            },
            Error::InvalidSession,
        ),
    ] {
        assert_eq!(store.consume_item_in_match(&invalid, 102), Err(error));
    }

    store
        .db
        .execute_batch(
            "CREATE TEMP TRIGGER deny_receipt BEFORE INSERT ON match_item_consumptions
         BEGIN SELECT RAISE(ABORT,'injected receipt failure'); END;",
        )
        .unwrap();
    let full = MatchItemConsumptionRequest {
        quantity: 2,
        ..first
    };
    assert_eq!(
        store.consume_item_in_match(&full, 102),
        Err(Error::Conflict)
    );
    assert_eq!(economy_state(&store.db), before);
    assert_eq!(now(&store), at);
    assert_eq!(count(&store.db, "match_item_consumptions"), 0);
    store
        .db
        .execute_batch("DROP TRIGGER temp.deny_receipt")
        .unwrap();
    assert_eq!(
        store
            .consume_item_in_match(&full, 102)
            .unwrap()
            .remaining_qty,
        0
    );
}
