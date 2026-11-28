use super::*;
use std::net::{Ipv4Addr, SocketAddrV4};
use std::sync::Arc;

fn config() -> Config {
    Config {
        bind: SocketAddrV4::new(Ipv4Addr::LOCALHOST, 8080),
        allowed_peers: vec![Ipv4Addr::LOCALHOST],
        max_connections: 4,
        request_timeout_ms: 1000,
        max_body_bytes: 1024,
        session_ttl_seconds: 60,
        unlock_all: false,
    }
}

fn opted_f6_policy() -> Arc<crate::f6_policy::F6EventPolicy> {
    use serde_json::json;
    let systems = crate::f6_policy::F6EventPolicy::system_names()
        .iter()
        .map(|name| {
            (
                (*name).to_owned(),
                json!({"enabled": false, "start_utc": null, "end_utc": null,
                       "rate_caps": {"per_account_per_hour": 0, "global_per_minute": 0}}),
            )
        })
        .collect::<serde_json::Map<_, _>>();
    let mut systems = systems;
    systems.insert(
        "CAPass".into(),
        json!({"enabled": true, "start_utc": "2027-01-01T00:00:00Z",
               "end_utc": "2027-01-02T00:00:00Z",
               "rate_caps": {"per_account_per_hour": 6, "global_per_minute": 43}}),
    );
    let extended_systems = crate::f6_policy::F6EventPolicy::extended_system_names()
        .iter()
        .map(|name| {
            (
                (*name).to_owned(),
                json!({"enabled": false, "start_utc": null, "end_utc": null,
                       "season_key": null, "max_grants_per_day": 0,
                       "max_grants_per_season": 0,
                       "rate_caps": {"per_account_per_hour": 0, "global_per_minute": 0}}),
            )
        })
        .collect::<serde_json::Map<_, _>>();
    let mut extended_systems = extended_systems;
    extended_systems.insert(
        "attendance".into(),
        json!({"enabled": true, "start_utc": "2027-01-01T00:00:00Z",
               "end_utc": "2027-01-02T00:00:00Z", "season_key": "spring-2027",
               "max_grants_per_day": 12, "max_grants_per_season": 100,
               "rate_caps": {"per_account_per_hour": 6, "global_per_minute": 43}}),
    );
    let profile = json!({"schema": "kre.f6-events-policy.v1", "default_enabled": false,
                         "systems": systems, "extended_systems": extended_systems});
    Arc::new(crate::f6_policy::F6EventPolicy::from_json_str(&profile.to_string()).unwrap())
}
#[test]
fn config_bounds() {
    assert!(config().validate().is_ok());
    let mut c = config();
    c.bind = SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 8080);
    assert!(c.validate().is_err());
    let mut c = config();
    c.bind = SocketAddrV4::new(Ipv4Addr::LOCALHOST, 0);
    assert!(c.validate().is_err());
    let mut c = config();
    c.allowed_peers.clear();
    assert!(c.validate().is_err());
    let mut c = config();
    c.max_body_bytes = 255;
    assert!(c.validate().is_err());
    let mut c = config();
    c.request_timeout_ms = 249;
    assert!(c.validate().is_err());
}
#[test]
fn auth_issue_revoke_and_expiry() {
    let auth = WebShopAuth::new(1).unwrap();
    assert!(auth.issue_token(0).is_err());
    let token = auth.issue_token(7).unwrap();
    assert_eq!(auth.player(&token_digest(token.as_str())), Some(7));
    auth.revoke_token(&token);
    assert_eq!(auth.player(&token_digest(token.as_str())), None);
}

#[tokio::test]
async fn html_is_local_and_parser_bounds_body() {
    let page = super::PAGE;
    assert!(!page.contains("Content-Security-Policy"));
    assert!(!page.contains("http://"));
    assert!(!page.contains("https://"));
    assert!(!page.contains("innerHTML"));
    assert!(page.contains("history.replaceState(null,'',location.pathname)"));
    assert!(page.contains("if(q('#token').value.trim())load()"));
    assert!(page.contains("/api/v1/profile"));
    assert!(page.contains("p.level"));
    assert!(page.contains("p.exp"));
    assert!(page.contains("p.lucci"));
    assert!(page.contains("p.mode"));
    assert!(page.contains("/api/v1/inventory"));
    assert!(page.contains("/api/v1/cosmetics-summary"));
    let f4_panel = page
        .split_once("async function loadF4ExtendedStatus(load){")
        .unwrap()
        .1
        .split_once("async function loadEventsStatus")
        .unwrap()
        .0;
    assert!(page.contains("id=\"f4-extended-status\""));
    assert!(page.contains("loadCustomMaps(load),loadF4ExtendedStatus(load)"));
    assert!(f4_panel.contains("await api('/api/v1/f4-extended-status')"));
    assert!(f4_panel.contains("li.textContent=policy.enabled?"));
    assert!(f4_panel.contains("q('#f4-extended-policies').append(li)"));
    assert!(!page.contains("/api/v1/f4/claim-milestone"));
    assert!(page.contains("/api/v1/events-status"));
    assert!(page.contains("events-status"));
    assert!(page.contains("/api/v1/custom-maps"));
    assert!(page.contains("/api/v1/unequip"));
    assert!(page.contains("/api/social/friends"));
    assert!(page.contains("/api/social/friends/respond"));
    assert!(page.contains("/api/social/guild"));
    assert!(page.contains("/api/social/guild/create"));
    assert!(page.contains("/api/social/guild/leave"));
    assert!(page.contains("/api/social/guild/disband"));
    assert!(page.contains("create-guild"));
    assert!(page.contains("leave-guild"));
    assert!(page.contains("disband-guild"));
    assert!(page.contains("/api/social/blocks"));
    assert!(page.contains("/api/social/chat-status"));
    assert!(page.contains("/api/social/ranking?limit=10"));
    let listener = tokio::net::TcpListener::bind((Ipv4Addr::LOCALHOST, 0))
        .await
        .unwrap();
    let addr = listener.local_addr().unwrap();
    let task = tokio::spawn(async move {
        let (mut stream, _) = listener.accept().await.unwrap();
        super::http::read(&mut stream, 4).await
    });
    let mut client = tokio::net::TcpStream::connect(addr).await.unwrap();
    tokio::io::AsyncWriteExt::write_all(
        &mut client,
        b"POST / HTTP/1.1\r\nHost: local\r\nContent-Length: 5\r\n\r\n12345",
    )
    .await
    .unwrap();
    let result = task.await.unwrap();
    assert!(matches!(
        result,
        Err(Some(super::http::Status::RequestTooLarge))
    ));
}

#[test]
fn auth_token_expires() {
    let auth = WebShopAuth::new(1).unwrap();
    let token = auth.issue_token(9).unwrap();
    std::thread::sleep(std::time::Duration::from_millis(1_100));
    assert_eq!(auth.player(&token_digest(token.as_str())), None);
}

struct TempDir(std::path::PathBuf);
impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn temp_db(suffix: &str) -> (TempDir, std::path::PathBuf) {
    static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let root = std::env::temp_dir().join(format!(
        "kr-webshop-{suffix}-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    ));
    std::fs::create_dir(&root).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&root, std::fs::Permissions::from_mode(0o700)).unwrap();
    }
    let db = root.join("accounts.sqlite");
    (TempDir(root), db)
}

async fn send_http(
    addr: std::net::SocketAddr,
    method: &str,
    target: &str,
    token: Option<&str>,
    body: Option<&str>,
) -> (u16, String, serde_json::Value) {
    let body_bytes = body.unwrap_or("").as_bytes();
    let mut raw = format!("{method} {target} HTTP/1.1\r\nHost: 127.0.0.1\r\n");
    if let Some(t) = token {
        raw.push_str(&format!("Authorization: Bearer {t}\r\n"));
    }
    if body.is_some() {
        raw.push_str("Content-Type: application/json\r\n");
    }
    raw.push_str(&format!("Content-Length: {}\r\n\r\n", body_bytes.len()));
    send_raw_http(addr, &raw, body_bytes).await
}

async fn send_raw_http(
    addr: std::net::SocketAddr,
    raw: &str,
    body_bytes: &[u8],
) -> (u16, String, serde_json::Value) {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    let mut stream = tokio::net::TcpStream::connect(addr).await.unwrap();
    stream.write_all(raw.as_bytes()).await.unwrap();
    if !body_bytes.is_empty() {
        stream.write_all(body_bytes).await.unwrap();
    }
    let mut resp = Vec::new();
    stream.read_to_end(&mut resp).await.unwrap();
    let text = String::from_utf8(resp).unwrap();
    let (head, payload) = text.split_once("\r\n\r\n").unwrap();
    let code = head
        .lines()
        .next()
        .unwrap()
        .split_whitespace()
        .nth(1)
        .unwrap()
        .parse::<u16>()
        .unwrap();
    let json = serde_json::from_str(payload).unwrap_or(serde_json::Value::Null);
    (code, head.to_owned(), json)
}

fn test_settings() -> crate::account_worker::EconomySettings {
    use crate::accounts::{EconomyPolicy, ItemGrant, ShopCatalog, ShopCategory, ShopItem};
    let item = |sku: &str, category, price, slot: Option<&str>, enabled| ShopItem {
        sku_id: sku.into(),
        category,
        price_lucci: price,
        grant: ItemGrant::Permanent,
        equip_slot: slot.map(str::to_owned),
        enabled,
    };
    crate::account_worker::EconomySettings::new(
        EconomyPolicy::default(),
        ShopCatalog {
            items: vec![
                item(
                    "char.bazzi",
                    ShopCategory::Character,
                    100,
                    Some("character"),
                    true,
                ),
                item(
                    "hat.fancy",
                    ShopCategory::Decoration,
                    500,
                    Some("head"),
                    true,
                ),
                item("emote.off", ShopCategory::Emotion, 10, Some("emote"), false),
            ],
        },
    )
    .unwrap()
}

const F4_STATUS: &str = "/api/v1/f4-extended-status";
const F4_CLAIM: &str = "/api/v1/f4/claim-milestone";
const F4_LUCCI_CLAIM: &str = r#"{"level":2,"kind":"lucci"}"#;
const F4_CHARACTER: u16 = 17;

fn f4_settings() -> crate::account_worker::EconomySettings {
    test_settings()
        .with_extended_policies(
            serde_json::from_value(json!({
                "enabled": true, "exp_percent": 125, "lucci_percent": 150,
                "bonus_cap": {"exp": 100, "lucci": 200},
                "allowed_rooms": [{"room_group": "local-group", "mode": "normal", "maps": ["LocalMap"]}],
            }))
            .unwrap(),
            serde_json::from_value(json!({
                "enabled": true,
                "character_milestones": [
                    {"character": F4_CHARACTER, "level": 3, "chest": {"sku": "hat.fancy", "quantity": 1}},
                    {"character": F4_CHARACTER, "level": 2, "chest": {"sku": "hat.fancy", "quantity": 1}},
                ],
                "level_up_lucci_table": [
                    {"character": F4_CHARACTER, "level": 2, "lucci": 75},
                    {"character": F4_CHARACTER, "level": 4, "lucci": 150},
                ],
            }))
            .unwrap(),
            serde_json::from_value(json!({
                "enabled": true, "base_lucci_cap": 1000, "expansion_step": 500,
                "max_expansions": 2, "max_lucci_cap": 2000,
            }))
            .unwrap(),
        )
        .unwrap()
}

fn f4_player_fixture(suffix: &str) -> (TempDir, std::path::PathBuf, accounts::Player) {
    use crate::accounts::{Admin, Store, TEST_SERIAL};

    let (dir, db) = temp_db(suffix);
    let _serial = TEST_SERIAL.lock().unwrap();
    let mut store = Store::open(&db).unwrap();
    let fixture = rusqlite::Connection::open(&db).unwrap();
    fixture
        .execute(
            "INSERT INTO sqlite_sequence(name,seq) VALUES('accounts',40)",
            [],
        )
        .unwrap();
    let player = store
        .create_account(
            &Admin {
                actor: "webshop-test",
                reason: "F4 milestone fixture",
            },
            "milestoneuser",
            "MilestoneUser",
            b"milestone-test-password",
            accounts::utc().unwrap(),
        )
        .unwrap();
    assert_ne!(player.account_id, player.player_id);
    fixture
        .execute(
            "INSERT INTO player_profiles (player_id,level) VALUES (?1,3)",
            [player.player_id],
        )
        .unwrap();
    (dir, db, player)
}

async fn listen_f4(
    config: Config,
    worker: &crate::account_worker::Worker,
    auth: &WebShopAuth,
) -> (
    std::net::SocketAddr,
    tokio::task::JoinHandle<io::Result<()>>,
) {
    let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).await.unwrap();
    let addr = listener.local_addr().unwrap();
    let task = tokio::spawn(serve_listener(
        listener,
        config,
        worker.service(),
        auth.clone(),
    ));
    (addr, task)
}

fn sqlite_version(db: &rusqlite::Connection) -> i64 {
    db.pragma_query_value(None, "data_version", |row| row.get(0))
        .unwrap()
}

#[tokio::test]
async fn loopback_f4_status_requires_auth_and_exposes_only_public_policy() {
    use crate::account_worker::{Worker, tests::pause};

    let (_dir, db, player) = f4_player_fixture("f4-status");
    let observer = rusqlite::Connection::open(&db).unwrap();
    let auth = WebShopAuth::new(60).unwrap();
    let token = auth
        .issue_token_for_milestones(&player, F4_CHARACTER)
        .unwrap();
    let revoked = auth.issue_token(player.player_id).unwrap();
    auth.revoke_token(&revoked);
    let token = Some(token.as_str());
    for (settings, expected) in [
        (
            test_settings(),
            json!({
                "configured": false,
                "booster_tickets": {"configured": false, "enabled": false, "exp_percent": 100,
                    "lucci_percent": 100, "bonus_cap": null, "room_rule_count": 0},
                "level_rewards": {"configured": false, "enabled": false, "chest_rule_count": 0,
                    "lucci_rule_count": 0, "tiers": []},
                "lucci_vault": {"configured": false, "enabled": false, "base_lucci_cap": null,
                    "expansion_step": null, "max_expansions": null, "max_lucci_cap": null},
            }),
        ),
        (
            f4_settings(),
            json!({
                "configured": true,
                "booster_tickets": {"configured": true, "enabled": true, "exp_percent": 125,
                    "lucci_percent": 150, "bonus_cap": {"exp": 100, "lucci": 200}, "room_rule_count": 1},
                "level_rewards": {"configured": true, "enabled": true, "chest_rule_count": 2,
                    "lucci_rule_count": 2, "tiers": [2, 3, 4]},
                "lucci_vault": {"configured": true, "enabled": true, "base_lucci_cap": 1000,
                    "expansion_step": 500, "max_expansions": 2, "max_lucci_cap": 2000},
            }),
        ),
    ] {
        let worker = Worker::open_with_economy(db.clone(), 4, Duration::from_secs(2), settings)
            .await
            .unwrap();
        let (addr, task) = listen_f4(config(), &worker, &auth).await;
        let resume = pause(&worker.client()).await;
        let before = sqlite_version(&observer);
        for invalid in [None, Some(revoked.as_str())] {
            for (method, path, body) in [
                ("GET", F4_STATUS, None),
                ("POST", F4_CLAIM, Some(F4_LUCCI_CLAIM)),
            ] {
                let (code, _, result) = send_http(addr, method, path, invalid, body).await;
                assert_eq!((code, result), (401, json!({"error": "unauthorized"})));
            }
        }
        let (code, head, result) = send_http(addr, "GET", F4_STATUS, token, None).await;
        assert_eq!((code, result), (200, expected.clone()));
        assert!(head.contains("Cache-Control: no-store"));
        assert_eq!(send_http(addr, "POST", F4_STATUS, token, None).await.0, 405);
        let query = format!("{F4_STATUS}?player_id=1");
        assert_eq!(send_http(addr, "GET", &query, token, None).await.0, 400);
        if expected["configured"] == false {
            assert_eq!(
                send_http(addr, "POST", F4_CLAIM, token, Some(F4_LUCCI_CLAIM))
                    .await
                    .0,
                403
            );
        }
        assert_eq!(sqlite_version(&observer), before);
        resume.send(()).unwrap();
        task.abort();
        worker.shutdown().await.unwrap();
    }
}

#[tokio::test]
async fn loopback_f4_claim_is_idempotent_and_blocked_in_unlock_all() {
    use crate::account_worker::Worker;

    let (_dir, db, player) = f4_player_fixture("f4-claims");
    let worker = Worker::open_with_economy(db.clone(), 4, Duration::from_secs(2), f4_settings())
        .await
        .unwrap();
    let auth = WebShopAuth::new(60).unwrap();
    let claim_token = auth
        .issue_token_for_milestones(&player, F4_CHARACTER)
        .unwrap();
    let token = Some(claim_token.as_str());
    let observer = rusqlite::Connection::open(&db).unwrap();
    let (addr, unlock_task) = listen_f4(
        Config {
            unlock_all: true,
            ..config()
        },
        &worker,
        &auth,
    )
    .await;
    let before = sqlite_version(&observer);
    let (code, _, result) = send_http(addr, "POST", F4_CLAIM, token, Some(F4_LUCCI_CLAIM)).await;
    assert_eq!(
        (code, result),
        (409, json!({"error": "unlock_all_no_purchase"}))
    );
    assert_eq!(sqlite_version(&observer), before);
    unlock_task.abort();

    let (addr, task) = listen_f4(config(), &worker, &auth).await;
    let legacy = auth.issue_token(player.player_id).unwrap();
    assert_eq!(
        send_http(
            addr,
            "POST",
            F4_CLAIM,
            Some(legacy.as_str()),
            Some(F4_LUCCI_CLAIM)
        )
        .await
        .0,
        403
    );
    assert_eq!(send_http(addr, "GET", F4_CLAIM, token, None).await.0, 405);
    for invalid in [
        r#"[2,"lucci"]"#,
        r#"{"level":2,"kind":"lucci","character":17}"#,
        r#"{"level":65536,"kind":"lucci"}"#,
        r#"{"level":2,"kind":"invalid"}"#,
        r#"{"level":3,"kind":"lucci"}"#,
        r#"{"level":4,"kind":"lucci"}"#, // Configured, but the player has not reached it.
    ] {
        assert_eq!(
            send_http(addr, "POST", F4_CLAIM, token, Some(invalid))
                .await
                .0,
            400
        );
    }
    let oversized = format!(
        "POST {F4_CLAIM} HTTP/1.1\r\nHost: local\r\nAuthorization: Bearer {}\r\nContent-Length: {}\r\n\r\n",
        claim_token.as_str(),
        config().max_body_bytes + 1,
    );
    assert_eq!(send_raw_http(addr, &oversized, b"").await.0, 413);

    observer.execute_batch("BEGIN IMMEDIATE").unwrap();
    let (code, _, result) = send_http(addr, "POST", F4_CLAIM, token, Some(F4_LUCCI_CLAIM)).await;
    assert_eq!((code, result), (503, json!({"error": "request_failed"})));
    observer.execute_batch("ROLLBACK").unwrap();

    for replayed in [false, true] {
        let (code, _, result) =
            send_http(addr, "POST", F4_CLAIM, token, Some(F4_LUCCI_CLAIM)).await;
        assert_eq!(
            (code, result),
            (
                200,
                json!({
                    "ok": true, "level": 2, "kind": "lucci", "lucci_delta": 75,
                    "sku_id": null, "replayed": replayed,
                })
            )
        );
    }
    let chest_conflict = r#"{"level":2,"kind":"chest"}"#;
    let (code, _, result) = send_http(addr, "POST", F4_CLAIM, token, Some(chest_conflict)).await;
    assert_eq!((code, result), (409, json!({"error": "request_failed"})));
    let (lucci, ledger_rows): (i64, i64) = observer.query_row(
        "SELECT lucci, (SELECT count(*) FROM ledger_entries) FROM player_profiles WHERE player_id=?1",
        [player.player_id],
        |row| Ok((row.get(0)?, row.get(1)?)),
    ).unwrap();
    assert_eq!((lucci, ledger_rows), (75, 1));
    task.abort();
    worker.shutdown().await.unwrap();
}

#[tokio::test]
async fn loopback_custom_maps_require_auth_and_isolate_owner_storage() {
    use crate::account_worker::Worker;
    use crate::accounts::{Admin, Store, TEST_SERIAL};
    use crate::custom_map_store::CustomMapPolicy;
    use kr_wire::kr00967::custom_map::{CUSTOM_MAP_SCHEMA, MAX_CUSTOM_MAP_BYTES};

    const MAPS: &str = "/api/v1/custom-maps";
    const DELETE: &str = "/api/v1/custom-maps/delete";
    const DELETE_BODY: &str = r#"{"map_id":"first"}"#;

    fn encode_base64(payload: &[u8]) -> String {
        const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
        let mut encoded = String::new();
        for chunk in payload.chunks(3) {
            let a = chunk[0];
            let b = chunk.get(1).copied().unwrap_or(0);
            let c = chunk.get(2).copied().unwrap_or(0);
            for (i, digit) in [
                a >> 2,
                (a & 3) << 4 | b >> 4,
                (b & 15) << 2 | c >> 6,
                c & 63,
            ]
            .into_iter()
            .enumerate()
            {
                encoded.push(if i > chunk.len() {
                    '='
                } else {
                    ALPHABET[usize::from(digit)] as char
                });
            }
        }
        encoded
    }

    async fn listen(
        shop: WebShop,
        worker: &Worker,
        auth: &WebShopAuth,
    ) -> (
        std::net::SocketAddr,
        tokio::task::JoinHandle<io::Result<()>>,
    ) {
        let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).await.unwrap();
        let addr = listener.local_addr().unwrap();
        let task = tokio::spawn(serve_loop(
            listener,
            shop,
            worker.service(),
            auth.clone(),
            None,
        ));
        (addr, task)
    }

    let (dir, db) = temp_db("custom-maps");
    let [owner, other] = {
        let _serial = TEST_SERIAL.lock().unwrap();
        let mut accounts = Store::open(&db).unwrap();
        let admin = Admin {
            actor: "webshop-test",
            reason: "F7 loopback fixture",
        };
        ["mapowner", "mapother"].map(|name| {
            accounts
                .create_account(&admin, name, name, b"maps-test-password", 1)
                .unwrap()
                .player_id
        })
    };
    let worker = Worker::open_with_economy(db, 8, Duration::from_secs(2), test_settings())
        .await
        .unwrap();
    let auth = WebShopAuth::new(60).unwrap();
    let owner_token = auth.issue_token(owner).unwrap();
    let other_token = auth.issue_token(other).unwrap();
    let revoked = auth.issue_token(owner).unwrap();
    auth.revoke_token(&revoked);
    let token = Some(owner_token.as_str());
    let other_token = Some(other_token.as_str());
    let map_root = dir.0.join("maps");
    let config = Config {
        max_body_bytes: 12 * 1024,
        ..config()
    };
    let disabled = CustomMapStore::new(&map_root, CustomMapPolicy::default()).unwrap();
    let disabled_response = json!({"error": "custom_maps_disabled", "wire_contract": "unmeasured"});
    let mut payload = serde_json::to_vec(&json!({
        "schema": CUSTOM_MAP_SCHEMA, "width": 15, "height": 13,
        "tiles": vec![0; 15 * 13], "spawns": [[0, 0], [14, 12]],
    }))
    .unwrap();
    payload.resize(MAX_CUSTOM_MAP_BYTES, b' ');
    let save = |map_id, bytes: &[u8]| {
        json!({
            "map_id": map_id, "payload_base64": encode_base64(bytes),
        })
        .to_string()
    };
    let first = save("first", &payload);

    for shop in [
        WebShop::new(config.clone()),
        WebShop::new(config.clone()).with_custom_map_store(disabled),
    ] {
        let (addr, task) = listen(shop, &worker, &auth).await;
        for (method, path, body) in [
            ("GET", MAPS, None),
            ("POST", MAPS, Some(first.as_str())),
            ("POST", DELETE, Some(DELETE_BODY)),
        ] {
            let (code, _, value) = send_http(addr, method, path, token, body).await;
            assert_eq!((code, value), (403, disabled_response.clone()));
        }
        task.abort();
        assert!(task.await.unwrap_err().is_cancelled());
    }
    assert!(!map_root.exists());

    let store = CustomMapStore::new(
        &map_root,
        CustomMapPolicy {
            enabled: true,
            max_maps_per_owner: 1,
            ..CustomMapPolicy::default()
        },
    )
    .unwrap();
    let (addr, task) = listen(
        WebShop::new(config).with_custom_map_store(store),
        &worker,
        &auth,
    )
    .await;
    for (method, path, body) in [
        ("GET", MAPS, None),
        ("POST", MAPS, Some(first.as_str())),
        ("POST", DELETE, Some(DELETE_BODY)),
    ] {
        for invalid_token in [None, Some("invalid"), Some(revoked.as_str())] {
            let (code, _, value) = send_http(addr, method, path, invalid_token, body).await;
            assert_eq!((code, value), (401, json!({"error": "unauthorized"})));
        }
        for query in ["?", "?owner_id=1"] {
            let target = format!("{path}{query}");
            assert_eq!(send_http(addr, method, &target, token, body).await.0, 400);
        }
    }
    for (method, path) in [
        ("PUT", MAPS),
        ("HEAD", MAPS),
        ("GET", DELETE),
        ("DELETE", DELETE),
    ] {
        assert_eq!(send_http(addr, method, path, token, None).await.0, 405);
    }

    let mut oversized = payload.clone();
    oversized.push(b' ');
    for (body, code, error) in [
        (save("first", &oversized), 413, "custom_map_too_large"),
        (save("first", b"{}"), 400, "invalid_custom_map"),
        (save("../escape", &payload), 400, "invalid_map_id"),
        (
            r#"{"map_id":"first","payload_base64":"e30!"}"#.into(),
            400,
            "request_failed",
        ),
        (
            r#"{"map_id":"first","payload_base64":"YR=="}"#.into(),
            400,
            "request_failed",
        ),
        (r#"["first","e30="]"#.into(), 400, "request_failed"),
        (
            r#"{"map_id":"first","payload_base64":"e30=","owner_id":"2"}"#.into(),
            400,
            "request_failed",
        ),
    ] {
        let (status, _, value) = send_http(addr, "POST", MAPS, token, Some(&body)).await;
        assert_eq!((status, &value["error"]), (code, &json!(error)));
    }
    assert!(
        !map_root.exists(),
        "rejected saves should not create storage"
    );

    let (code, _, value) = send_http(addr, "POST", MAPS, token, Some(&first)).await;
    assert_eq!(code, 200, "{value}");
    assert_eq!(value["wire_contract"], "unmeasured");
    let saved = &value["map"];
    assert_eq!(saved["owner_id"], owner.to_string());
    assert_eq!(saved["map_id"], "first");
    assert_eq!(saved["sha256"], crate::sha256_hex(&payload));
    assert_eq!(
        (saved["width"].as_u64(), saved["height"].as_u64()),
        (Some(15), Some(13))
    );

    let (code, head, listed) = send_http(addr, "GET", MAPS, token, None).await;
    assert_eq!(code, 200);
    assert!(head.contains("Cache-Control: no-store"));
    assert_eq!(
        listed,
        json!({"maps":[saved], "quota_remaining":0, "wire_contract":"unmeasured"})
    );
    let empty = json!({"maps":[], "quota_remaining":1, "wire_contract":"unmeasured"});
    let (code, _, listed) = send_http(addr, "GET", MAPS, other_token, None).await;
    assert_eq!((code, listed), (200, empty.clone()));
    let (code, _, value) = send_http(addr, "POST", DELETE, other_token, Some(DELETE_BODY)).await;
    assert_eq!(
        (code, &value["error"]),
        (404, &json!("custom_map_not_found"))
    );

    let second = save("second", &payload);
    let (code, _, value) = send_http(addr, "POST", MAPS, token, Some(&second)).await;
    assert_eq!(
        (code, &value["error"]),
        (409, &json!("custom_map_quota_exceeded"))
    );
    assert_eq!(
        send_http(addr, "POST", MAPS, token, Some(&first)).await.0,
        200
    );
    let (code, _, value) = send_http(addr, "POST", DELETE, token, Some(DELETE_BODY)).await;
    assert_eq!(
        (code, value),
        (200, json!({"removed":true, "wire_contract":"unmeasured"}))
    );
    let (code, _, listed) = send_http(addr, "GET", MAPS, token, None).await;
    assert_eq!((code, listed), (200, empty));

    task.abort();
    assert!(task.await.unwrap_err().is_cancelled());
    worker.shutdown().await.unwrap();
}

#[tokio::test]
async fn loopback_summary_routes_are_read_only_and_report_wire_limits() {
    use crate::account_worker::Worker;
    use crate::accounts::{Admin, ChatMuteReason, ItemGrant, Store, TEST_SERIAL};

    const CHAT: &str = "/api/social/chat-status";
    const COSMETICS: &str = "/api/v1/cosmetics-summary";
    const EVENTS: &str = "/api/v1/events-status";
    let (_dir, db) = temp_db("summaries");
    let (player, expired_player) = {
        let _serial = TEST_SERIAL.lock().unwrap();
        let mut store = Store::open(&db).unwrap();
        let now = crate::match_journal::utc().unwrap();
        let admin = Admin {
            actor: "webshop-test",
            reason: "Projection summary loopback fixture",
        };
        let player = store
            .create_account(
                &admin,
                "shopuser",
                "ShopUser",
                b"shop-test-password-1",
                now - 60,
            )
            .unwrap()
            .player_id;
        let expired_player = store
            .create_account(
                &admin,
                "expired",
                "ExpiredMute",
                b"shop-test-password-2",
                now - 60,
            )
            .unwrap()
            .player_id;
        assert_eq!(
            store
                .mute_social_player(expired_player, 60, ChatMuteReason::Moderation, now - 60)
                .unwrap(),
            now
        );
        assert_eq!(store.social_mute_status(expired_player, now).unwrap(), None);
        for sku in ["char.bazzi", "hat.fancy"] {
            store
                .admin_grant_item(&admin, player, sku, ItemGrant::Permanent, 1, sku, now)
                .unwrap();
        }
        (player, expired_player)
    };
    let worker = Worker::open_with_economy(db.clone(), 8, Duration::from_secs(2), test_settings())
        .await
        .unwrap();
    let auth = WebShopAuth::new(60).unwrap();
    let token = auth.issue_token(player).unwrap();
    let expired_token = auth.issue_token(expired_player).unwrap();
    let revoked = auth.issue_token(player).unwrap();
    auth.revoke_token(&revoked);
    let token = Some(token.as_str());
    let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).await.unwrap();
    let addr = listener.local_addr().unwrap();
    let task = tokio::spawn(serve_listener(listener, config(), worker.service(), auth));

    for path in [CHAT, COSMETICS, EVENTS] {
        for invalid in [None, Some("invalid"), Some(revoked.as_str())] {
            let (status, _, payload) = send_http(addr, "GET", path, invalid, None).await;
            assert_eq!((status, payload), (401, json!({"error": "unauthorized"})));
        }
        assert_eq!(send_http(addr, "POST", path, token, None).await.0, 405);
        for query in ["?", "?player_id=1"] {
            let path = format!("{path}{query}");
            assert_eq!(send_http(addr, "GET", &path, token, None).await.0, 400);
        }
    }

    let observer = rusqlite::Connection::open(&db).unwrap();
    let db_state = || {
        let version = observer
            .pragma_query_value::<i64, _>(None, "data_version", |r| r.get(0))
            .unwrap();
        let (quota, abuse_events, audit_events): (i64, i64, i64) = observer
            .query_row(
                "SELECT (SELECT coalesce(sum(accepted),0) FROM chat_rate_windows),
                        (SELECT count(*) FROM chat_abuse_events),
                        (SELECT count(*) FROM admin_audit)",
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .unwrap();
        (version, quota, abuse_events, audit_events)
    };
    let before = db_state();
    let (status, head, chat) = send_http(addr, "GET", CHAT, token, None).await;
    assert_eq!(status, 200);
    assert!(head.contains("Cache-Control: no-store"));
    assert_eq!(
        chat,
        json!({
            "mute_status": "available",
            "muted": false,
            "mute_until": null,
            "mute_reason": null,
            "choco_chat": {
                "extended_live_routing": false,
                "wire_status": "unmeasured",
                "local_policy_defaults": {
                    "whisper": false, "guild_chat": false, "megaphone": false,
                    "offline_note": false, "couple": false, "plaza": false,
                },
            },
        })
    );
    let (status, head, empty) = send_http(addr, "GET", COSMETICS, token, None).await;
    assert_eq!(status, 200);
    assert!(head.contains("Cache-Control: no-store"));
    assert_eq!(
        empty,
        json!({
            "source": "inventory",
            "equipped": [],
            "mutual_exclusion": {"valid": true, "conflicting_slot": null},
            "inventory_layout": {"status": "static_candidate", "gap": INVENTORY_CAPTURE_GAP},
        })
    );
    assert_eq!(db_state(), before);
    let (status, _, events) = send_http(addr, "GET", EVENTS, token, None).await;
    assert_eq!(status, 200);
    assert_eq!(
        events,
        json!({
            "configured": false,
            "core_total": 19,
            "extended_total": 25,
            "excluded_systems": ["PCRoomBenefit"],
            "enabled_systems": [],
            "extended_systems": [],
        })
    );
    assert_eq!(db_state(), before);

    let mute_until = worker
        .client()
        .try_mute_social_player(player, 3600, ChatMuteReason::Spam)
        .unwrap()
        .wait()
        .await
        .unwrap();
    worker
        .client()
        .try_mute_social_player(player, 60, ChatMuteReason::Harassment)
        .unwrap()
        .wait()
        .await
        .unwrap();
    for sku in ["char.bazzi", "hat.fancy"] {
        let input = json!({"sku": sku}).to_string();
        assert_eq!(
            send_http(addr, "POST", "/api/v1/equip", token, Some(&input))
                .await
                .0,
            200
        );
    }
    let before = db_state();
    let (status, _, chat_after_mute) = send_http(addr, "GET", CHAT, token, None).await;
    let mut active = chat.clone();
    active["muted"] = json!(true);
    active["mute_until"] = json!(mute_until);
    active["mute_reason"] = json!("spam");
    assert_eq!((status, chat_after_mute), (200, active));
    let (status, _, expired) =
        send_http(addr, "GET", CHAT, Some(expired_token.as_str()), None).await;
    assert_eq!((status, expired), (200, chat));
    let (status, _, equipped) = send_http(addr, "GET", COSMETICS, token, None).await;
    assert_eq!(status, 200);
    assert_eq!(equipped["mutual_exclusion"], empty["mutual_exclusion"]);
    assert_eq!(equipped["inventory_layout"], empty["inventory_layout"]);
    let rows = equipped["equipped"].as_array().unwrap();
    assert_eq!(rows.len(), 2);
    for (row, slot, sku, target) in [
        (&rows[0], "character", "char.bazzi", "character"),
        (&rows[1], "head", "hat.fancy", "hat"),
    ] {
        assert_eq!(row["slot"], slot);
        assert_eq!(row["sku_id"], sku);
        assert_eq!(row["target"], target);
        assert_eq!(row["projection_status"], "unmeasured");
        assert_eq!(row["wire_gap"], UNMEASURED_WIRE_GAP);
    }
    let character = rows[0]["target_surfaces"].as_array().unwrap();
    assert_eq!(character[0]["anchor"], "character92");
    assert_eq!(character[0]["status"], "measured");
    assert_eq!(character[1]["surface"], "own291");
    assert_eq!(character[1]["status"], "unmeasured");
    assert_eq!(character[1]["gap"], UNMEASURED_WIRE_GAP);
    let hat = rows[1]["target_surfaces"].as_array().unwrap();
    assert_eq!(hat.len(), 4);
    assert!(hat.iter().all(|surface| surface["status"] == "unmeasured"
        && surface["gap"] == UNMEASURED_WIRE_GAP
        && surface["anchor"].is_null()));
    assert_eq!(db_state(), before);

    task.abort();
    worker.shutdown().await.unwrap();
}

#[tokio::test]
async fn loopback_events_status_reports_opted_in_core_and_extended_policy() {
    use crate::account_worker::Worker;
    use crate::accounts::{Admin, Store, TEST_SERIAL};

    let (_dir, db) = temp_db("events-status");
    let player = {
        let _serial = TEST_SERIAL.lock().unwrap();
        Store::open(&db)
            .unwrap()
            .create_account(
                &Admin {
                    actor: "webshop-test",
                    reason: "F6 status fixture",
                },
                "eventuser",
                "EventUser",
                b"events-test-password",
                crate::match_journal::utc().unwrap() - 60,
            )
            .unwrap()
            .player_id
    };
    let worker = Worker::open_with_economy(db, 8, Duration::from_secs(2), test_settings())
        .await
        .unwrap();
    let auth = WebShopAuth::new(60).unwrap();
    let token = auth.issue_token(player).unwrap();
    let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).await.unwrap();
    let addr = listener.local_addr().unwrap();
    let shop = WebShop::new(config()).with_f6_policy(opted_f6_policy());
    let task = tokio::spawn(serve_loop(listener, shop, worker.service(), auth, None));

    let (status, _, events) = send_http(
        addr,
        "GET",
        "/api/v1/events-status",
        Some(token.as_str()),
        None,
    )
    .await;
    assert_eq!(status, 200);
    assert_eq!(events["configured"], true);
    assert_eq!(events["enabled_systems"][0]["system"], "CAPass");
    assert_eq!(events["enabled_systems"][0]["start_utc"], 1798761600i64);
    assert_eq!(events["enabled_systems"][0]["end_utc"], 1798848000i64);
    assert_eq!(events["extended_systems"][0]["system"], "attendance");
    assert_eq!(events["extended_systems"][0]["season_key"], "spring-2027");
    assert_eq!(events["extended_systems"][0]["max_grants_per_day"], 12);
    assert_eq!(events["extended_systems"][0]["max_grants_per_season"], 100);

    task.abort();
    assert!(task.await.unwrap_err().is_cancelled());
    worker.shutdown().await.unwrap();
}

#[tokio::test]
async fn guild_leader_actions_require_bearer_and_authority_and_change_the_roster() {
    use crate::account_worker::Worker;
    use crate::accounts::{Admin, Store, TEST_SERIAL};

    const ADD: &str = "/api/social/guild/add-member";
    const REMOVE: &str = "/api/social/guild/remove-member";
    const TRANSFER: &str = "/api/social/guild/transfer-leader";
    const GUILD: &str = "/api/social/guild";
    let (_dir, db) = temp_db("guild-leader");
    let now = crate::match_journal::utc().unwrap();
    let [leader, member, outsider, guest] = {
        let _serial = TEST_SERIAL.lock().unwrap();
        let mut store = Store::open(&db).unwrap();
        let admin = Admin {
            actor: "webshop-test",
            reason: "Guild management loopback E2E",
        };
        ["leader", "member", "outsider", "guest"].map(|name| {
            store
                .create_account(&admin, name, name, b"test-fixture-password", now)
                .unwrap()
                .player_id
        })
    };
    let worker = Worker::open_with_economy(db, 8, Duration::from_secs(2), test_settings())
        .await
        .unwrap();
    let auth = WebShopAuth::new(60).unwrap();
    let [leader_token, member_token, outsider_token, guest_token] =
        [leader, member, outsider, guest].map(|player| auth.issue_token(player).unwrap());
    let (lt, mt, ot, gt) = (
        leader_token.as_str(),
        member_token.as_str(),
        outsider_token.as_str(),
        guest_token.as_str(),
    );
    let listener = tokio::net::TcpListener::bind((Ipv4Addr::LOCALHOST, 0))
        .await
        .unwrap();
    let addr = listener.local_addr().unwrap();
    let task = tokio::spawn(serve_listener(
        listener,
        config(),
        worker.service(),
        auth.clone(),
    ));

    let (code, _, created) = send_http(
        addr,
        "POST",
        "/api/social/guild/create",
        Some(lt),
        Some(r#"{"name":"Blue Team"}"#),
    )
    .await;
    assert_eq!(code, 200);
    let guild_id = created["guild"]["id"].as_i64().unwrap();
    let (code, _, other) = send_http(
        addr,
        "POST",
        "/api/social/guild/create",
        Some(ot),
        Some(r#"{"name":"Red Team"}"#),
    )
    .await;
    assert_eq!(code, 200);
    let other_guild_id = other["guild"]["id"].as_i64().unwrap();

    let add_body = json!({"player_id":member, "guild_id":guild_id}).to_string();
    let (code, _, reply) = send_http(addr, "POST", ADD, Some(lt), Some(&add_body)).await;
    assert_eq!((code, reply), (200, json!({"changed":true})));
    let (code, _, roster) = send_http(addr, "GET", GUILD, Some(mt), None).await;
    assert_eq!(code, 200);
    assert_eq!(roster["is_leader"], false);
    assert_eq!(roster["members"][1]["player_id"], member);
    assert_eq!(roster["members"][1]["role"], "member");

    // Each request is otherwise valid and would change the guild with a bearer.
    for (path, target) in [(ADD, guest), (REMOVE, member), (TRANSFER, member)] {
        let body = json!({"player_id":target}).to_string();
        let forged_form = format!(
            "POST {path} HTTP/1.1\r\nHost: local\r\nOrigin: https://outside.invalid\r\nCookie: session={lt}\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n",
            body.len(),
        );
        let (code, headers, _) = send_raw_http(addr, &forged_form, body.as_bytes()).await;
        assert_eq!(code, 401, "{path}");
        assert!(
            !headers
                .to_ascii_lowercase()
                .contains("access-control-allow-origin")
        );
        let (code, _, _) = send_http(addr, "POST", path, Some(&"0".repeat(64)), Some(&body)).await;
        assert_eq!(code, 401, "{path}");
        for method in ["GET", "DELETE", "OPTIONS"] {
            let (code, _, _) = send_http(addr, method, path, Some(lt), Some(&body)).await;
            assert_eq!(code, 405, "{method} {path}");
        }
        let with_query = format!("{path}?player_id={target}");
        let (code, _, _) = send_http(addr, "POST", &with_query, Some(lt), Some(&body)).await;
        assert_eq!(code, 400, "{path}");
        let spoofed_actor = json!({"player_id":target, "leader_player_id":leader}).to_string();
        let (code, _, _) = send_http(addr, "POST", path, Some(mt), Some(&spoofed_actor)).await;
        assert_eq!(code, 400, "{path}");
    }

    for (path, token, target, status) in [
        (ADD, mt, guest, 409),
        (ADD, gt, member, 409),
        (REMOVE, mt, leader, 409),
        (TRANSFER, mt, leader, 409),
        (ADD, lt, member, 409),
        (ADD, lt, outsider, 409),
        (ADD, lt, i64::MAX, 400),
        (REMOVE, lt, outsider, 409),
        (REMOVE, lt, guest, 409),
        (REMOVE, lt, leader, 400),
        (TRANSFER, lt, outsider, 400),
        (TRANSFER, lt, guest, 400),
        (TRANSFER, lt, leader, 400),
    ] {
        let body = json!({"player_id":target}).to_string();
        let (code, _, _) = send_http(addr, "POST", path, Some(token), Some(&body)).await;
        assert_eq!(code, status, "{path} for target {target}");
    }
    let wrong_guild = json!({"player_id":guest, "guild_id":other_guild_id}).to_string();
    let (code, _, _) = send_http(addr, "POST", ADD, Some(lt), Some(&wrong_guild)).await;
    assert_eq!(code, 409);

    let guest_body = json!({"player_id":guest}).to_string();
    for (token, target) in [(lt, guest), (gt, leader)] {
        let block_body = json!({"player_id":target}).to_string();
        let (code, _, _) = send_http(
            addr,
            "POST",
            "/api/social/blocks",
            Some(token),
            Some(&block_body),
        )
        .await;
        assert_eq!(code, 200);
        let (code, _, _) = send_http(addr, "POST", ADD, Some(lt), Some(&guest_body)).await;
        assert_eq!(code, 409);
        let (code, _, _) = send_http(
            addr,
            "DELETE",
            "/api/social/blocks",
            Some(token),
            Some(&block_body),
        )
        .await;
        assert_eq!(code, 200);
    }
    let (code, _, empty) = send_http(addr, "GET", GUILD, Some(gt), None).await;
    assert_eq!(code, 200);
    assert!(empty["guild"].is_null());
    assert_eq!(empty["is_leader"], false);

    let (code, _, reply) = send_http(addr, "POST", TRANSFER, Some(lt), Some(&add_body)).await;
    assert_eq!((code, reply), (200, json!({"changed":true})));
    let (code, _, old_roster) = send_http(addr, "GET", GUILD, Some(lt), None).await;
    assert_eq!(code, 200);
    assert_eq!(old_roster["is_leader"], false);
    let (code, _, new_roster) = send_http(addr, "GET", GUILD, Some(mt), None).await;
    assert_eq!(code, 200);
    assert_eq!(new_roster["is_leader"], true);
    assert_eq!(new_roster["guild"]["leader_player_id"], member);
    assert_eq!(new_roster["members"].as_array().unwrap().len(), 2);
    assert_eq!(new_roster["members"][0]["role"], "leader");
    assert_eq!(new_roster["members"][1]["player_id"], leader);
    assert_eq!(new_roster["members"][1]["role"], "member");
    let (code, _, _) = send_http(addr, "POST", ADD, Some(lt), Some(&guest_body)).await;
    assert_eq!(code, 409);
    let old_leader = json!({"player_id":leader}).to_string();
    let (code, _, reply) = send_http(addr, "POST", REMOVE, Some(mt), Some(&old_leader)).await;
    assert_eq!((code, reply), (200, json!({"changed":true})));
    let (code, _, empty) = send_http(addr, "GET", GUILD, Some(lt), None).await;
    assert_eq!(code, 200);
    assert!(empty["guild"].is_null());

    auth.revoke_token(&member_token);
    let (code, _, _) = send_http(addr, "POST", ADD, Some(mt), Some(&guest_body)).await;
    assert_eq!(code, 401);
    task.abort();
}

#[tokio::test]
async fn loopback_standard_and_unlock_all_end_to_end() {
    use crate::account_worker::Worker;
    use crate::accounts::{Admin, Store, TEST_SERIAL};

    let (_dir, db) = temp_db("e2e");
    let now = crate::match_journal::utc().unwrap();
    let (player_id, friend_id, third_id) = {
        let _serial = TEST_SERIAL.lock().unwrap();
        let mut store = Store::open(&db).unwrap();
        let admin = Admin {
            actor: "webshop-test",
            reason: "F4-d loopback E2E",
        };
        let player = store
            .create_account(&admin, "shopuser", "ShopUser", b"shop-test-password-1", now)
            .unwrap();
        let friend = store
            .create_account(
                &admin,
                "frienduser",
                "FriendUser",
                b"shop-test-password-2",
                now,
            )
            .unwrap();
        let third = store
            .create_account(
                &admin,
                "thirduser",
                "ThirdUser",
                b"shop-test-password-3",
                now,
            )
            .unwrap();
        store
            .admin_grant_lucci(&admin, player.player_id, 250, "seed-lucci", now)
            .unwrap();
        (player.player_id, friend.player_id, third.player_id)
    };

    let worker = Worker::open_with_economy(db.clone(), 8, Duration::from_secs(2), test_settings())
        .await
        .unwrap();
    let auth = WebShopAuth::new(60).unwrap();
    let token = auth.issue_token(player_id).unwrap();

    // Start standard-mode web shop on ephemeral loopback port.
    let listener = tokio::net::TcpListener::bind((Ipv4Addr::LOCALHOST, 0))
        .await
        .unwrap();
    let addr = listener.local_addr().unwrap();
    let std_task = tokio::spawn(serve_listener(
        listener,
        config(),
        worker.service(),
        auth.clone(),
    ));

    let (code, head, health) = send_http(addr, "GET", "/health", None, None).await;
    assert_eq!(code, 200);
    assert_eq!(health["mode"], "standard");
    assert_eq!(health["catalog_items"], 3);
    assert!(head.contains("Cache-Control: no-store"));
    assert!(head.contains("X-Content-Type-Options: nosniff"));

    let (code, head, _) = send_http(addr, "GET", "/", None, None).await;
    assert_eq!(code, 200);
    assert!(head.contains("Content-Security-Policy: default-src 'none'"));

    let (code, _, _) = send_http(addr, "GET", "/api/v1/profile", None, None).await;
    assert_eq!(code, 401);
    let (code, _, _) = send_http(addr, "GET", "/api/v1/profile", Some(&"a".repeat(64)), None).await;
    assert_eq!(code, 401);
    let (code, _, _) = send_http(addr, "GET", "/api/social/friends", None, None).await;
    assert_eq!(code, 401);
    let (code, _, ranking) = send_http(
        addr,
        "GET",
        "/api/social/ranking?limit=10",
        Some(token.as_str()),
        None,
    )
    .await;
    assert_eq!(code, 200);
    assert!(ranking["ranking"].is_array());

    let tok = token.as_str();
    let friend_token = auth.issue_token(friend_id).unwrap();
    let third_token = auth.issue_token(third_id).unwrap();
    let (code, _, guild_before) =
        send_http(addr, "GET", "/api/social/guild", Some(tok), None).await;
    assert_eq!(code, 200);
    assert!(guild_before["guild"].is_null());
    let (code, _, request) = send_http(
        addr,
        "POST",
        "/api/social/friends",
        Some(tok),
        Some(&format!(r#"{{"player_id":{friend_id}}}"#)),
    )
    .await;
    assert_eq!(code, 200);
    assert_eq!(request["changed"], true);
    let (code, _, pending) = send_http(
        addr,
        "GET",
        "/api/social/friends",
        Some(friend_token.as_str()),
        None,
    )
    .await;
    assert_eq!(code, 200);
    assert_eq!(pending["incoming_requests"][0]["sender_id"], player_id);
    let (code, _, _) = send_http(
        addr,
        "POST",
        "/api/social/friends/respond",
        Some(third_token.as_str()),
        Some(&format!(
            r#"{{"requester_player_id":{player_id},"accept":true}}"#
        )),
    )
    .await;
    assert_eq!(code, 409);
    let (code, _, _) = send_http(
        addr,
        "POST",
        "/api/social/friends/respond",
        Some(friend_token.as_str()),
        Some(&format!(
            r#"{{"requester_player_id":{player_id},"accept":true}}"#
        )),
    )
    .await;
    assert_eq!(code, 200);
    let (code, _, accepted) = send_http(addr, "GET", "/api/social/friends", Some(tok), None).await;
    assert_eq!(code, 200);
    assert_eq!(accepted["friends"][0], friend_id);
    let (code, _, _) = send_http(
        addr,
        "POST",
        "/api/social/friends",
        Some(third_token.as_str()),
        Some(&format!(r#"{{"player_id":{friend_id}}}"#)),
    )
    .await;
    assert_eq!(code, 200);
    let (code, _, rejected) = send_http(
        addr,
        "POST",
        "/api/social/friends/respond",
        Some(friend_token.as_str()),
        Some(&format!(
            r#"{{"requester_player_id":{third_id},"accept":false}}"#
        )),
    )
    .await;
    assert_eq!(code, 200);
    assert_eq!(rejected["changed"], true);
    let (code, _, third_friends) = send_http(
        addr,
        "GET",
        "/api/social/friends",
        Some(third_token.as_str()),
        None,
    )
    .await;
    assert_eq!(code, 200);
    assert!(third_friends["friends"].as_array().unwrap().is_empty());
    let (code, _, created) = send_http(
        addr,
        "POST",
        "/api/social/guild/create",
        Some(tok),
        Some(r#"{"name":"Web Guild"}"#),
    )
    .await;
    assert_eq!(code, 200);
    let guild_id = created["guild"]["id"].as_i64().unwrap();
    assert_eq!(created["guild"]["leader_player_id"], player_id);
    assert_eq!(created["members"].as_array().unwrap().len(), 1);
    let (code, _, _) = send_http(
        addr,
        "POST",
        "/api/social/guild/create",
        Some(tok),
        Some(r#"{"name":"Web Guild 2"}"#),
    )
    .await;
    assert_eq!(code, 409);
    let (code, _, _) = send_http(
        addr,
        "POST",
        "/api/social/guild/create",
        Some(third_token.as_str()),
        Some(r#"{"name":"web guild"}"#),
    )
    .await;
    assert_eq!(code, 409);
    let (code, _, guild_after) = send_http(addr, "GET", "/api/social/guild", Some(tok), None).await;
    assert_eq!(code, 200);
    assert_eq!(guild_after["guild"]["id"], guild_id);
    assert_eq!(guild_after["members"].as_array().unwrap().len(), 1);
    worker
        .client()
        .try_social(
            friend_id,
            crate::account_worker::SocialAction::JoinGuild(guild_id),
        )
        .unwrap()
        .wait()
        .await
        .unwrap();
    let (code, _, _) = send_http(
        addr,
        "POST",
        "/api/social/guild/disband",
        Some(friend_token.as_str()),
        Some("{}"),
    )
    .await;
    assert_eq!(code, 409);
    let (code, _, left) = send_http(
        addr,
        "POST",
        "/api/social/guild/leave",
        Some(friend_token.as_str()),
        Some("{}"),
    )
    .await;
    assert_eq!(code, 200);
    assert_eq!(left["changed"], true);
    let (code, _, _) = send_http(
        addr,
        "POST",
        "/api/social/guild/leave",
        Some(friend_token.as_str()),
        Some("{}"),
    )
    .await;
    assert_eq!(code, 409);
    let (code, _, _) = send_http(
        addr,
        "POST",
        "/api/social/guild/leave",
        Some(tok),
        Some("{}"),
    )
    .await;
    assert_eq!(code, 409);
    let (code, _, disbanded) = send_http(
        addr,
        "POST",
        "/api/social/guild/disband",
        Some(tok),
        Some(r#"{"confirm":true}"#),
    )
    .await;
    assert_eq!(code, 200);
    assert_eq!(disbanded["changed"], true);
    let (code, _, guild_left) = send_http(
        addr,
        "GET",
        "/api/social/guild",
        Some(friend_token.as_str()),
        None,
    )
    .await;
    assert_eq!(code, 200);
    assert!(guild_left["guild"].is_null());
    let (code, _, guild_after_disband) =
        send_http(addr, "GET", "/api/social/guild", Some(tok), None).await;
    assert_eq!(code, 200);
    assert!(guild_after_disband["guild"].is_null());
    auth.revoke_token(&third_token);
    let (code, _, _) = send_http(
        addr,
        "GET",
        "/api/social/guild",
        Some(third_token.as_str()),
        None,
    )
    .await;
    assert_eq!(code, 401);
    let short_auth = WebShopAuth::new(1).unwrap();
    let short_token = short_auth.issue_token(player_id).unwrap();
    let short_listener = tokio::net::TcpListener::bind((Ipv4Addr::LOCALHOST, 0))
        .await
        .unwrap();
    let short_addr = short_listener.local_addr().unwrap();
    let short_task = tokio::spawn(serve_listener(
        short_listener,
        config(),
        worker.service(),
        short_auth,
    ));
    tokio::time::sleep(Duration::from_millis(1_100)).await;
    let (code, _, _) = send_http(
        short_addr,
        "GET",
        "/api/social/guild",
        Some(short_token.as_str()),
        None,
    )
    .await;
    assert_eq!(code, 401);
    short_task.abort();
    let (code, _, profile) = send_http(addr, "GET", "/api/v1/profile", Some(tok), None).await;
    assert_eq!(code, 200);
    assert_eq!(profile["lucci"], 250);

    let (code, _, cat) = send_http(
        addr,
        "GET",
        "/api/v1/catalog?category=character",
        Some(tok),
        None,
    )
    .await;
    assert_eq!(code, 200);
    assert_eq!(cat["items"].as_array().unwrap().len(), 1);
    assert_eq!(cat["items"][0]["sku_id"], "char.bazzi");
    assert_eq!(cat["items"][0]["usable"], false);

    // Purchase + idempotent replay + conflicting replay with same request_id.
    let buy = r#"{"sku":"char.bazzi","request_id":"req-1"}"#;
    let (code, _, r1) = send_http(addr, "POST", "/api/v1/purchase", Some(tok), Some(buy)).await;
    assert_eq!(code, 200);
    assert_eq!(r1["receipt"]["profile"]["lucci"], 150);
    assert_eq!(r1["receipt"]["replayed"], false);
    let (code, _, r2) = send_http(addr, "POST", "/api/v1/purchase", Some(tok), Some(buy)).await;
    assert_eq!(code, 200);
    assert_eq!(r2["receipt"]["profile"]["lucci"], 150);
    assert_eq!(r2["receipt"]["replayed"], true);
    let (code, _, _) = send_http(
        addr,
        "POST",
        "/api/v1/purchase",
        Some(tok),
        Some(r#"{"sku":"hat.fancy","request_id":"req-1"}"#),
    )
    .await;
    assert_eq!(code, 409);

    // Insufficient Lucci (500 > 150) & disabled/unknown SKU.
    let (code, _, _) = send_http(
        addr,
        "POST",
        "/api/v1/purchase",
        Some(tok),
        Some(r#"{"sku":"hat.fancy","request_id":"req-2"}"#),
    )
    .await;
    assert_eq!(code, 402);
    let (code, _, _) = send_http(
        addr,
        "POST",
        "/api/v1/purchase",
        Some(tok),
        Some(r#"{"sku":"emote.off","request_id":"req-3"}"#),
    )
    .await;
    assert_eq!(code, 400);

    // Equip -> inventory check -> unequip.
    let (code, _, _) = send_http(
        addr,
        "POST",
        "/api/v1/equip",
        Some(tok),
        Some(r#"{"sku":"char.bazzi"}"#),
    )
    .await;
    assert_eq!(code, 200);
    let (code, _, inv) = send_http(addr, "GET", "/api/v1/inventory", Some(tok), None).await;
    assert_eq!(code, 200);
    assert_eq!(inv["inventory"].as_array().unwrap().len(), 1);
    assert_eq!(inv["equipped"][0]["sku_id"], "char.bazzi");
    let (code, _, un) = send_http(
        addr,
        "POST",
        "/api/v1/unequip",
        Some(tok),
        Some(r#"{"slot":"character"}"#),
    )
    .await;
    assert_eq!(code, 200);
    assert_eq!(un["removed"], true);
    std_task.abort();

    // O1 unlock_all mode: all enabled SKUs usable in session, purchases blocked 409 without DB changes.
    let mut o1_cfg = config();
    o1_cfg.unlock_all = true;
    let o1_listener = tokio::net::TcpListener::bind((Ipv4Addr::LOCALHOST, 0))
        .await
        .unwrap();
    let o1_addr = o1_listener.local_addr().unwrap();
    let o1_task = tokio::spawn(serve_listener(
        o1_listener,
        o1_cfg,
        worker.service(),
        auth.clone(),
    ));
    let (code, _, o1_cat) = send_http(o1_addr, "GET", "/api/v1/catalog", Some(tok), None).await;
    assert_eq!(code, 200);
    assert_eq!(o1_cat["mode"], "unlock_all");
    assert!(
        o1_cat["items"]
            .as_array()
            .unwrap()
            .iter()
            .all(|i| i["usable"] == true && i["unlocked_by_session"] == true)
    );
    let (code, _, rej) = send_http(
        o1_addr,
        "POST",
        "/api/v1/purchase",
        Some(tok),
        Some(r#"{"sku":"hat.fancy","request_id":"req-o1"}"#),
    )
    .await;
    assert_eq!(code, 409);
    assert_eq!(rej["error"], "unlock_all_no_purchase");
    let (code, _, p_after) = send_http(o1_addr, "GET", "/api/v1/profile", Some(tok), None).await;
    assert_eq!(code, 200);
    assert_eq!(p_after["lucci"], 150);
    o1_task.abort();
    worker.shutdown().await.unwrap();
}

#[tokio::test]
async fn unlock_all_selection_is_isolated_per_token_and_bypasses_sqlite() {
    use crate::account_worker::Worker;
    use crate::accounts::{Admin, ItemGrant, Store, TEST_SERIAL};

    let (_dir, db) = temp_db("o1-selection");
    let settings = test_settings();
    let player = {
        let _serial = TEST_SERIAL.lock().unwrap();
        let mut store = Store::open(&db).unwrap();
        let now = accounts::utc().unwrap();
        let admin = Admin {
            actor: "webshop-test",
            reason: "standard ownership for O1 control",
        };
        let player = store
            .create_account(&admin, "shopuser", "ShopUser", b"shop-test-password-1", now)
            .unwrap()
            .player_id;
        store
            .admin_grant_item(
                &admin,
                player,
                "hat.fancy",
                ItemGrant::Permanent,
                1,
                "grant:hat",
                now,
            )
            .unwrap();
        store
            .equip_item(player, settings.catalog(), "hat.fancy", "head", now)
            .unwrap();
        player
    };
    let worker = Worker::open_with_economy(db.clone(), 4, Duration::from_secs(2), settings)
        .await
        .unwrap();
    let mut o1 = config();
    o1.unlock_all = true;
    let auth = WebShopAuth::new(60).unwrap();
    let first = auth.issue_token(player).unwrap();
    let second = auth.issue_token(player).unwrap();
    let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).await.unwrap();
    let addr = listener.local_addr().unwrap();
    let task = tokio::spawn(serve_listener(listener, o1, worker.service(), auth.clone()));
    let observer = rusqlite::Connection::open(&db).unwrap();
    let data_version = || {
        observer
            .pragma_query_value::<i64, _>(None, "data_version", |r| r.get(0))
            .unwrap()
    };
    let resume = crate::account_worker::tests::pause(&worker.client()).await;
    let before = data_version();
    let tok = Some(first.as_str());
    let inventory = "/api/v1/inventory";
    let equip = "/api/v1/equip";
    let unequip = "/api/v1/unequip";
    let hat = r#"{"slot":"head"}"#;

    let (code, _, empty) = send_http(addr, "GET", inventory, tok, None).await;
    assert_eq!(code, 200);
    assert_eq!(empty, json!({"inventory": [], "equipped": []}));
    let (code, _, result) =
        send_http(addr, "POST", equip, tok, Some(r#"{"sku":"char.bazzi"}"#)).await;
    assert_eq!(code, 200);
    let character = json!({"sku_id":"char.bazzi", "slot":"character"});
    assert_eq!(result["equipped"], character);
    let (code, _, _) = send_http(
        addr,
        "POST",
        equip,
        tok,
        Some(r#"{"sku":"hat.fancy","slot":"head"}"#),
    )
    .await;
    assert_eq!(code, 200);
    for invalid in [
        r#"{"sku":"emote.off","slot":"emote"}"#,
        r#"{"sku":"char.bazzi","slot":"head"}"#,
        r#"{"sku":"unknown","slot":"head"}"#,
    ] {
        assert_eq!(
            send_http(addr, "POST", equip, tok, Some(invalid)).await.0,
            400
        );
    }
    let (code, _, selected) = send_http(addr, "GET", inventory, tok, None).await;
    assert_eq!(code, 200);
    assert_eq!(
        selected,
        json!({"inventory": [], "equipped": [character, {"sku_id":"hat.fancy","slot":"head"}]})
    );
    let (code, _, other) = send_http(addr, "GET", inventory, Some(second.as_str()), None).await;
    assert_eq!(code, 200);
    assert_eq!(other, empty);
    let summary = "/api/v1/cosmetics-summary";
    let (code, _, first_summary) = send_http(addr, "GET", summary, tok, None).await;
    assert_eq!(code, 200);
    assert_eq!(first_summary["source"], "session");
    assert_eq!(first_summary["equipped"][0]["sku_id"], "char.bazzi");
    assert_eq!(first_summary["equipped"][1]["sku_id"], "hat.fancy");
    let (code, _, second_summary) =
        send_http(addr, "GET", summary, Some(second.as_str()), None).await;
    assert_eq!(code, 200);
    assert_eq!(second_summary["equipped"], json!([]));
    for expected in [true, false] {
        let (code, _, result) = send_http(addr, "POST", unequip, tok, Some(hat)).await;
        assert_eq!(code, 200);
        assert_eq!(result["removed"], expected);
    }
    let (code, _, selected) = send_http(addr, "GET", inventory, tok, None).await;
    assert_eq!(code, 200);
    assert_eq!(selected["equipped"], json!([character]));
    auth.revoke_player(player);
    for token in [&first, &second] {
        assert_eq!(
            send_http(addr, "GET", inventory, Some(token.as_str()), None)
                .await
                .0,
            401
        );
    }
    let replacement = auth.issue_token(player).unwrap();
    let (code, _, fresh) =
        send_http(addr, "GET", inventory, Some(replacement.as_str()), None).await;
    assert_eq!(code, 200);
    assert_eq!(fresh, empty);
    assert_eq!(data_version(), before);

    resume.send(()).unwrap();
    let standard = worker.service().equipped_list(player).await.unwrap();
    assert_eq!(standard.len(), 1);
    assert_eq!(standard[0].sku_id, "hat.fancy");
    task.abort();
    worker.shutdown().await.unwrap();
}

#[test]
fn peer_allowlist_is_exact() {
    let c = config();
    assert!(c.allows(std::net::IpAddr::V4(Ipv4Addr::LOCALHOST)));
    assert!(!c.allows(std::net::IpAddr::V4(Ipv4Addr::new(127, 0, 0, 2))));
}
