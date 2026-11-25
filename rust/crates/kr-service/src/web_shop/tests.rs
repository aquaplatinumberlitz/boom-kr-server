use super::*;
use std::net::{Ipv4Addr, SocketAddrV4};

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
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    let mut stream = tokio::net::TcpStream::connect(addr).await.unwrap();
    let body_bytes = body.unwrap_or("").as_bytes();
    let mut raw = format!("{method} {target} HTTP/1.1\r\nHost: 127.0.0.1\r\n");
    if let Some(t) = token {
        raw.push_str(&format!("Authorization: Bearer {t}\r\n"));
    }
    if body.is_some() {
        raw.push_str("Content-Type: application/json\r\n");
    }
    raw.push_str(&format!("Content-Length: {}\r\n\r\n", body_bytes.len()));
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

#[tokio::test]
async fn loopback_standard_and_unlock_all_end_to_end() {
    use crate::account_worker::Worker;
    use crate::accounts::{Admin, Store, TEST_SERIAL};

    let (_dir, db) = temp_db("e2e");
    let now = crate::match_journal::utc().unwrap();
    let player_id = {
        let _serial = TEST_SERIAL.lock().unwrap();
        let mut store = Store::open(&db).unwrap();
        let admin = Admin {
            actor: "webshop-test",
            reason: "F4-d loopback E2E",
        };
        let player = store
            .create_account(&admin, "shopuser", "ShopUser", b"shop-test-password-1", now)
            .unwrap();
        store
            .admin_grant_lucci(&admin, player.player_id, 250, "seed-lucci", now)
            .unwrap();
        player.player_id
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

    let tok = token.as_str();
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

#[test]
fn peer_allowlist_is_exact() {
    let c = config();
    assert!(c.allows(std::net::IpAddr::V4(Ipv4Addr::LOCALHOST)));
    assert!(!c.allows(std::net::IpAddr::V4(Ipv4Addr::new(127, 0, 0, 2))));
}
