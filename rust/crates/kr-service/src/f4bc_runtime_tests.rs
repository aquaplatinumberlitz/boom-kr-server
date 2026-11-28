use super::*;
use crate::{
    account_bridge::{
        Channel,
        local::{
            LocalSession,
            tests::{channel, login},
        },
        tests::{PASSWORD, Temp, admin, channel_body, login_body},
    },
    account_serving::Active,
    account_worker::{self, EconomySettings, Worker},
    accounts::{EconomyPolicy, ItemGrant, ShopCatalog, ShopCategory, ShopItem, Store, TEST_SERIAL},
    match_journal::{Manager, utc},
    room_io::{self, registry_foundation_tests::player_round_prototype},
    room_registry::{RoomId, RoomKey},
    rules::ResultPolicy,
    web_shop,
};
use rusqlite::Connection;
use serde_json::{Value, json};
use std::{
    net::{Ipv4Addr, SocketAddr},
    time::{Duration, Instant},
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    sync::watch,
};

const QUICK: &str = "quick";
const STACK: &str = "test.stack";

async fn login_with_busy_retry(
    client: &account_worker::Client,
    login: &str,
) -> account_worker::Result<crate::accounts::Grant> {
    for attempt in 0..200 {
        let result = match client.try_login(login, PASSWORD, 60, utc().unwrap()) {
            Ok(pending) => pending.wait().await,
            Err(error) => Err(error),
        };
        match result {
            Err(
                account_worker::Error::Busy
                | account_worker::Error::Backend(crate::accounts::Error::Busy),
            ) if attempt < 199 => tokio::time::sleep(Duration::from_millis(25)).await,
            result => return result,
        }
    }
    unreachable!("busy retry loop returns on its final attempt")
}

async fn fixture(
    unlock_all: bool,
) -> (
    Temp,
    Worker,
    Active,
    LocalSession,
    Channel,
    Vec<Session>,
    Option<Manager>,
    u64,
) {
    let (temp, a, _) = {
        let _serial = TEST_SERIAL
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        Temp::new()
    };
    let stack = ItemGrant::Stackable { max_stack: 8 };
    let settings = EconomySettings::new(
        EconomyPolicy {
            enabled: !unlock_all,
            ..EconomyPolicy::default()
        },
        ShopCatalog {
            items: vec![
                ShopItem {
                    sku_id: STACK.into(),
                    category: ShopCategory::Consumable,
                    price_lucci: 0,
                    grant: stack,
                    equip_slot: Some(QUICK.into()),
                    enabled: true,
                },
                ShopItem {
                    sku_id: "test.hat".into(),
                    category: ShopCategory::Decoration,
                    price_lucci: 0,
                    grant: ItemGrant::Permanent,
                    equip_slot: Some("head".into()),
                    enabled: true,
                },
            ],
        },
    )
    .unwrap();
    if !unlock_all {
        temp.store()
            .admin_grant_item(
                &admin(),
                a.player_id,
                STACK,
                stack,
                2,
                "fixture:stack",
                utc().unwrap(),
            )
            .unwrap();
    }
    let worker = Worker::open_with_economy(temp.db(), 8, Duration::from_secs(3), settings)
        .await
        .unwrap();
    let mut active = Active::login_test_client(worker.client());
    assert!(active.match_items(!unlock_all).is_none());
    active.service = Some(worker.service());
    active.unlock_all_catalog = unlock_all;
    active.bridge = active.bridge.with_unlock_all(unlock_all);
    assert_eq!(active.match_items(false).is_some(), unlock_all);
    let items = active.match_items(!unlock_all).unwrap();

    let shared = player_round_prototype();
    {
        let mut room = shared.lock().unwrap();
        room.rules.result = ResultPolicy::Players;
        room.public_key = Some(RoomKey {
            id: RoomId(1),
            session: 1,
            generation: 1,
        });
        room.model
            .set_allocation(crate::rules::SlotAllocation::FirstVacant)
            .unwrap();
        room.model.round_roster = crate::rules::RoundRoster::Players;
    }
    let journal = (!unlock_all).then(|| {
        let epoch = Store::probe_epoch(&temp.db()).unwrap();
        let manager = Manager::open(worker.client(), epoch);
        room_io::install_journal(&shared, manager.factory().owner()).unwrap();
        manager
    });
    install_match_items(&shared, items).unwrap();

    let peer = Ipv4Addr::LOCALHOST;
    let grant = login_with_busy_retry(&worker.client(), "Alice")
        .await
        .unwrap();
    let issued = active
        .bridge
        .issue(&grant.token, a.player_id, peer)
        .await
        .unwrap();
    let body = login_body(issued.token().expose_secret(), peer);
    issued.delivered();
    let pending = active.bridge.login(&body, peer, 11).await.unwrap();
    let body = channel_body(&pending.player, pending.echo);
    pending.redirected().unwrap();
    let launcher = active.bridge.channel(&body, peer, 22).await.unwrap();

    let mut local = LocalSession::new(worker.client(), 33).unwrap();
    let rows = local.login(&login(&["Bob"]), 60, None).await.unwrap();
    local.redirected(33).unwrap();
    let authorities = local.channel(&channel(&rows), 44).await.unwrap();
    let sessions = [launcher.relay_authority(), authorities[0].relay_authority()]
        .into_iter()
        .zip([22, 44])
        .map(|(owner, conn)| {
            Session::open_with_authority(&shared, owner.player().wire_id, conn, None, Some(owner))
                .unwrap()
        })
        .collect::<Vec<_>>();
    let round = {
        // Drive the real room model and journal; the fixture supplies no KR item packets.
        let mut room = shared.lock().unwrap();
        let mut next = room.model.clone();
        let keys = sessions
            .iter()
            .enumerate()
            .map(|(i, session)| {
                let key = next
                    .enter(session.identity, session.connection, i == 0)
                    .unwrap();
                next.restore_character(key, 0).unwrap();
                next.setup(key).unwrap();
                key
            })
            .collect::<Vec<_>>();
        next.choose(keys[1], None, Some(3)).unwrap();
        next.ready(keys[1]).unwrap();
        let round = next.begin(keys[0]).unwrap();
        if let Some(start) = room.journal_start(&next).unwrap() {
            room.journal.as_mut().unwrap().begin(start).unwrap();
        }
        for key in keys {
            next.ack(key, round).unwrap();
        }
        room.model = next;
        round
    };
    sessions[0].wait_journal().await.unwrap();
    (
        temp, worker, active, local, launcher, sessions, journal, round,
    )
}

#[tokio::test]
async fn standard_room_use_requires_equipment_and_persists_one_receipt_on_replay() {
    let (temp, worker, active, local, _launcher, sessions, journal, round) = fixture(false).await;
    let member = &sessions[0];
    let service = active.account_service().unwrap();
    let player = member.shared.lock().unwrap().channels[&member.identity]
        .relay_authority
        .as_ref()
        .unwrap()
        .player()
        .player_id;
    assert_eq!(
        member
            .consume_item_in_match(round, 1, QUICK, STACK, 1)
            .await,
        Err(Error::Backend(accounts::Error::InvalidInput))
    );
    service.equip_item(player, STACK, QUICK).await.unwrap();
    assert_eq!(
        member
            .consume_item_in_match(round + 1, 1, QUICK, STACK, 1)
            .await,
        Err(STALE)
    );
    let MatchItemConsumptionOutcome::Persisted(receipt) = member
        .consume_item_in_match(round, 1, QUICK, STACK, 1)
        .await
        .unwrap()
    else {
        panic!("standard use must persist");
    };
    assert_eq!(
        (
            receipt.consumed_qty,
            receipt.remaining_qty,
            receipt.replayed
        ),
        (1, 1, false)
    );
    assert_eq!(
        member
            .consume_item_in_match(round, 1, QUICK, STACK, 1)
            .await
            .unwrap(),
        MatchItemConsumptionOutcome::Persisted(crate::accounts::MatchItemConsumptionReceipt {
            replayed: true,
            ..receipt.clone()
        })
    );
    assert_eq!(
        member
            .consume_item_in_match(round, 1, QUICK, STACK, 2)
            .await,
        Err(Error::Backend(accounts::Error::Conflict))
    );
    assert_eq!(service.inventory_list(player).await.unwrap()[0].quantity, 1);
    let db = Connection::open(temp.db()).unwrap();
    let row = db
        .query_row(
            "SELECT count(*),max(id),max(match_id),max(remaining_qty) FROM match_item_consumptions",
            [],
            |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, i64>(2)?,
                    row.get::<_, i64>(3)?,
                ))
            },
        )
        .unwrap();
    assert_eq!(row, (1, receipt.id, receipt.match_id, 1));
    room_io::cancel_journal(&member.shared).unwrap();
    member.wait_journal().await.unwrap();
    drop(sessions);
    local.finish().await.unwrap();
    journal.unwrap().shutdown().await.unwrap();
    worker.shutdown().await.unwrap();
}

async fn http(
    addr: SocketAddr,
    token: &web_shop::WebShopToken,
    body: Option<&str>,
) -> (u16, Value) {
    let mut stream = TcpStream::connect(addr).await.unwrap();
    let request = match body {
        Some(body) => format!(
            "POST /api/v1/equip HTTP/1.1\r\nHost: local\r\nAuthorization: Bearer {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{body}",
            token.as_str(),
            body.len()
        ),
        None => format!(
            "GET /api/v1/inventory HTTP/1.1\r\nHost: local\r\nAuthorization: Bearer {}\r\n\r\n",
            token.as_str()
        ),
    };
    stream.write_all(request.as_bytes()).await.unwrap();
    let mut reply = String::new();
    stream.read_to_string(&mut reply).await.unwrap();
    let (headers, body) = reply.split_once("\r\n\r\n").unwrap();
    (
        headers.split_whitespace().nth(1).unwrap().parse().unwrap(),
        serde_json::from_str(body).unwrap(),
    )
}

#[tokio::test]
async fn o1_room_and_web_shop_bypass_worker_and_revoke_on_logout() {
    let (temp, worker, mut active, local, _launcher, sessions, _, round) = fixture(true).await;
    let member = &sessions[0];
    let (player, token) = {
        let room = member.shared.lock().unwrap();
        let owner = room.channels[&member.identity]
            .relay_authority
            .as_ref()
            .unwrap();
        (owner.player().player_id, owner.item_use_token().unwrap())
    };
    let address = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).await.unwrap();
    let SocketAddr::V4(addr) = address.local_addr().unwrap() else {
        unreachable!()
    };
    drop(address);
    let shop = web_shop::open(web_shop::Config {
        bind: addr,
        allowed_peers: vec![Ipv4Addr::LOCALHOST],
        max_connections: 4,
        request_timeout_ms: 1000,
        max_body_bytes: 1024,
        session_ttl_seconds: 60,
        unlock_all: true,
    })
    .await
    .unwrap();
    let auth = shop.auth();
    active.web_shop = Some(auth.clone());
    let (stop, stopped) = watch::channel(false);
    let task = tokio::spawn(web_shop::serve_until(shop, worker.service(), stopped));
    let db = Connection::open(temp.db()).unwrap();
    let version = || {
        db.pragma_query_value::<i64, _>(None, "data_version", |row| row.get(0))
            .unwrap()
    };
    let before = version();
    let first = active.issue_web_shop_token(&token, player).await.unwrap();
    let other = active.issue_web_shop_token(&token, player).await.unwrap();
    let expiring = auth
        .issue_until(player, Some(Instant::now() + Duration::from_secs(1)))
        .unwrap();
    let resume = crate::account_worker::tests::pause(&worker.client()).await;
    let empty = json!({"inventory":[],"equipped":[]});
    let equip = Some(r#"{"sku":"test.hat"}"#);
    assert_eq!(http(addr.into(), &first, equip).await.0, 200);
    assert_eq!(http(addr.into(), &other, None).await, (200, empty));
    assert_eq!(
        http(addr.into(), &first, None).await.1["equipped"],
        json!([{"sku_id":"test.hat","slot":"head"}])
    );
    assert_eq!(http(addr.into(), &expiring, equip).await.0, 200);
    assert_eq!(
        member
            .consume_item_in_match(round, 1, QUICK, STACK, 3)
            .await,
        Ok(MatchItemConsumptionOutcome::SessionUnlocked)
    );
    assert_eq!(
        member
            .consume_item_in_match(round, 2, "head", "test.hat", 1)
            .await,
        Err(Error::InvalidInput)
    );
    tokio::time::sleep(Duration::from_secs(1)).await;
    assert_eq!(http(addr.into(), &expiring, None).await.0, 401);
    assert_eq!(version(), before);
    for table in [
        "player_inventory",
        "player_equipped",
        "match_item_consumptions",
        "matches",
    ] {
        assert_eq!(
            db.query_row(&format!("SELECT count(*) FROM {table}"), [], |row| row
                .get::<_, i64>(0))
                .unwrap(),
            0
        );
    }
    resume.send(()).unwrap();
    assert!(active.logout_public(token, player).await.unwrap());
    for revoked in [&first, &other] {
        assert_eq!(http(addr.into(), revoked, None).await.0, 401);
    }
    assert_eq!(
        member
            .consume_item_in_match(round, 3, QUICK, STACK, 1)
            .await,
        Err(STALE)
    );
    stop.send_replace(true);
    task.await.unwrap().unwrap();
    drop(sessions);
    local.finish().await.unwrap();
    worker.shutdown().await.unwrap();
}
