use super::{Config, Owner};
use crate::{
    account_bridge::local::native::fixtures,
    account_worker::{EconomySettings, Error as WorkerError, SocialReply},
    accounts::{Admin, Error, ItemUpgradeOperation, Player, RecordEraserOperation, Store},
    choco_chat::F5SocialPolicy,
    f4_extended_policy::{
        CouponRequest, Decision, OperationKey, RecordScope, TitleUnlockDecision, UpgradeFamily,
    },
    platform, web_shop,
};
use kr_wire::kr00967::AccountTemplate;
use serde_json::{Value, json};
use std::{
    fs, io,
    net::{Ipv4Addr, SocketAddrV4, TcpListener},
    path::PathBuf,
    sync::Arc,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    sync::watch,
    time::timeout,
};

const MAPS: &str = "/api/v1/custom-maps";
const EVENTS: &str = "/api/v1/events-status";
const SAVE: &str = r#"{"map_id":"test","payload_base64":"e30="}"#;
static CREATE_ACCOUNT_GATE: std::sync::Mutex<()> = std::sync::Mutex::new(());

struct TempDir(PathBuf);

impl TempDir {
    fn new() -> Self {
        let root = fs::canonicalize(std::env::temp_dir())
            .unwrap()
            .join(format!("kr-live-policy-{:016x}", getrandom::u64().unwrap()));
        platform::provision_private_dir(&root).unwrap();
        Self(root)
    }

    fn config(&self) -> Config {
        serde_json::from_value(json!({
            "database": self.0.join("accounts.sqlite"),
            "launch_socket": self.0.join("launch.sock"),
            "launch_ttl_seconds": 60,
            "profile_mode": "database_zero",
        }))
        .unwrap()
    }

    fn policy(&self, name: &str, value: Value) -> PathBuf {
        let path = self.0.join(name);
        fs::write(&path, value.to_string()).unwrap();
        path
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn create_account(config: &Config) -> Player {
    let mut store = Store::open(&config.database).unwrap();
    let admin = Admin {
        actor: "test",
        reason: "live policy wiring",
    };
    let _gate = CREATE_ACCOUNT_GATE
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        match store.create_account(&admin, "alice", "Alice", b"test-password", None) {
            Err(Error::Busy) if Instant::now() < deadline => {
                std::thread::sleep(Duration::from_millis(10));
            }
            result => return result.expect("create live policy test account"),
        }
    }
}

fn welcome_coupon(account_id: i64) -> CouponRequest<'static> {
    let account = u64::try_from(account_id).unwrap();
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    CouponRequest {
        key: OperationKey { account, id: 1 },
        coupon_id: 1,
        offer: "welcome",
        issued_to: account,
        issued_at: now,
        now,
        locally_issued: true,
    }
}

fn projection() -> Arc<AccountTemplate> {
    fn decode(hex: &str) -> Vec<u8> {
        (0..hex.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
            .collect()
    }
    Arc::new(
        AccountTemplate::parse(
            &decode(fixtures::AUTH),
            &decode(fixtures::OWN),
            &decode(fixtures::PLAYER),
        )
        .unwrap(),
    )
}

async fn open_owner(config: &mut Config) -> io::Result<(SocketAddrV4, Owner)> {
    // The production config requires a fixed port, so retry if the port
    // becomes occupied between releasing the reservation and opening the shop.
    drop(Store::open(&config.database).unwrap());
    for _ in 0..5 {
        let reservation = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        let bind = SocketAddrV4::new(
            Ipv4Addr::LOCALHOST,
            reservation.local_addr().unwrap().port(),
        );
        config.web_shop = Some(web_shop::Config {
            bind,
            allowed_peers: vec![Ipv4Addr::LOCALHOST],
            max_connections: 4,
            request_timeout_ms: 2000,
            max_body_bytes: 4096,
            session_ttl_seconds: 60,
            unlock_all: false,
        });
        let projection = projection();
        drop(reservation);
        match Owner::open(config, Default::default(), Some(projection)).await {
            Err(error) if error.kind() == io::ErrorKind::AddrInUse => continue,
            result => return result.map(|owner| (bind, owner)),
        }
    }
    Err(io::ErrorKind::AddrInUse.into())
}

async fn request(
    addr: SocketAddrV4,
    token: &web_shop::WebShopToken,
    target: &str,
    body: Option<&str>,
) -> (u16, Value) {
    let method = if body.is_some() { "POST" } else { "GET" };
    let body = body.unwrap_or_default();
    let raw = format!(
        "{method} {target} HTTP/1.1\r\nHost: local\r\nAuthorization: Bearer {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{body}",
        token.as_str(),
        body.len()
    );
    let response = timeout(Duration::from_secs(5), async {
        let mut stream = TcpStream::connect(addr).await.unwrap();
        stream.write_all(raw.as_bytes()).await.unwrap();
        let mut response = String::new();
        stream.read_to_string(&mut response).await.unwrap();
        response
    })
    .await
    .unwrap();
    let (headers, body) = response.split_once("\r\n\r\n").unwrap();
    let status = headers.split_whitespace().nth(1).unwrap().parse().unwrap();
    (status, serde_json::from_str(body).unwrap())
}

fn run_shop(
    owner: &mut Owner,
) -> (
    web_shop::WebShopToken,
    watch::Sender<bool>,
    tokio::task::JoinHandle<io::Result<()>>,
) {
    let opened = owner.web_shop.take().unwrap();
    // Exercise policy wiring using the trusted token issuer. These routes
    // need an authenticated owner ID but do not read an account row.
    let token = opened.auth().issue_token(1).unwrap();
    let service = owner.active.account_service().unwrap().clone();
    let (shutdown, stop) = watch::channel(false);
    let task = tokio::spawn(web_shop::serve_until(opened, service, stop));
    (token, shutdown, task)
}

#[tokio::test]
async fn omitted_and_disabled_f4_ops_and_f5_stay_closed_live() {
    for configured in [false, true] {
        let dir = TempDir::new();
        let mut config = dir.config();
        if configured {
            config.f4_extended_policy = Some(dir.policy(
                "f4.json",
                json!({
                    "gift_coupons": {"enabled": false},
                    "record_erasers": {"enabled": false},
                    "upgrades": {"enabled": false},
                    "titles": {"enabled": false}
                }),
            ));
            config.f5_social_policy = Some(dir.policy("f5.json", json!({"whisper": false})));
        }
        let Player {
            player_id,
            account_id,
            ..
        } = create_account(&config);
        let owner = Owner::open(&config, Default::default(), Some(projection()))
            .await
            .unwrap();
        let service = owner.active.account_service().unwrap();
        assert_eq!(service.settings(), &EconomySettings::default());
        assert_eq!(
            service
                .redeem_gift_coupon(player_id, welcome_coupon(account_id))
                .await,
            Err(WorkerError::InvalidInput)
        );
        assert_eq!(
            service
                .apply_record_eraser(
                    player_id,
                    RecordEraserOperation {
                        key: 2,
                        sku: "eraser",
                        scope: RecordScope::Level
                    },
                )
                .await,
            Err(WorkerError::InvalidInput)
        );
        assert_eq!(
            service
                .apply_item_upgrade(
                    player_id,
                    ItemUpgradeOperation {
                        key: 3,
                        sku: "balloon",
                        family: UpgradeFamily::WaterBalloon,
                        level: 0,
                        roll_bps: 0,
                    },
                )
                .await,
            Err(WorkerError::InvalidInput)
        );
        assert_eq!(
            service.evaluate_title_unlock(player_id, "new-player").await,
            Err(WorkerError::InvalidInput)
        );
        assert_eq!(
            service
                .client
                .try_f5_social_policy_status(player_id)
                .unwrap()
                .wait()
                .await,
            Ok(SocialReply::F5PolicyStatus(F5SocialPolicy::default()))
        );
        assert!(service.ledger_list(player_id, 10).await.unwrap().is_empty());
        owner.shutdown().await.unwrap();
    }
}

#[tokio::test]
async fn economy_settings_reach_the_live_account_worker() {
    let dir = TempDir::new();
    let mut config = dir.config();
    assert_eq!(config.economy().unwrap(), EconomySettings::default());
    config.f4_extended_policy = Some(dir.policy(
        "f4.json",
        json!({
            "booster_tickets": {
                "enabled": true, "exp_percent": 150, "bonus_cap": {"exp": 50, "lucci": 0},
                "allowed_rooms": [{"room_group": "standard", "mode": "normal", "maps": ["Factory07"]}]
            },
            "level_rewards": {
                "enabled": true, "level_up_lucci_table": [{"character": 0, "level": 2, "lucci": 50}]
            },
            "lucci_vault": {
                "enabled": true, "base_lucci_cap": 1000, "expansion_step": 100,
                "max_expansions": 2, "max_lucci_cap": 1200
            },
            "gift_coupons": {
                "enabled": true,
                "coupon_offers": [{
                    "offer": "welcome", "valid_for_seconds": 300,
                    "grant": {"sku": "lucci", "quantity": 25}
                }]
            },
            "record_erasers": {
                "enabled": true, "allowed_scopes": ["level"],
                "erasers": [{"sku": "eraser", "reset_to": {"level": 1}}]
            },
            "upgrades": {
                "enabled": true,
                "recipes": [{
                    "family": "water_balloon", "sku": "balloon",
                    "from_level": 0, "to_level": 1, "max_level": 1, "materials": [],
                    "lucci_cost": 10, "success_bps": 10000, "on_failure": "keep",
                    "charge_on_failure": false
                }]
            },
            "titles": {
                "enabled": true, "equip_slot": "title", "notify_on_unlock": true,
                "unlock_table": [{"title": "new-player", "condition": "level>=1"}]
            }
        }),
    ));
    let bundle = config.f4_extended_policy().unwrap();
    let settings = config.economy().unwrap();
    assert_eq!(settings.booster_ticket_policy(), &bundle.booster_tickets);
    assert_eq!(settings.level_reward_policy(), &bundle.level_rewards);
    assert_eq!(settings.lucci_vault_policy(), &bundle.lucci_vault);
    assert_eq!(settings.gift_coupon_policy(), &bundle.gift_coupons);
    assert_eq!(settings.record_eraser_policy(), &bundle.record_erasers);
    assert_eq!(settings.upgrade_policy(), &bundle.upgrades);
    assert_eq!(settings.title_policy(), &bundle.titles);
    let Player {
        player_id,
        account_id,
        ..
    } = create_account(&config);
    let owner = Owner::open(&config, Default::default(), Some(projection()))
        .await
        .unwrap();
    let service = owner.active.account_service().unwrap();
    assert_eq!(service.settings(), &settings);
    let coupon = service
        .redeem_gift_coupon(player_id, welcome_coupon(account_id))
        .await;
    assert!(matches!(coupon, Ok(Decision::Apply(entry)) if entry.lucci_delta == 25));
    assert_eq!(service.profile_get(player_id).await.unwrap().lucci, 25);
    assert_eq!(
        service.evaluate_title_unlock(player_id, "new-player").await,
        Ok(Some(TitleUnlockDecision::New {
            notify_after_commit: true
        }))
    );
    owner.shutdown().await.unwrap();
}

#[tokio::test]
async fn f5_policy_file_reaches_the_live_account_service() {
    let dir = TempDir::new();
    let mut config = dir.config();
    config.f5_social_policy =
        Some(dir.policy("f5.json", json!({"whisper": true, "guild_chat": true})));
    let player = create_account(&config);
    let owner = Owner::open(&config, Default::default(), Some(projection()))
        .await
        .unwrap();
    let service = owner.active.account_service().unwrap();
    assert_eq!(
        service
            .client
            .try_f5_social_policy_status(player.player_id)
            .unwrap()
            .wait()
            .await,
        Ok(SocialReply::F5PolicyStatus(F5SocialPolicy {
            whisper: true,
            guild_chat: true,
            ..Default::default()
        }))
    );
    owner.shutdown().await.unwrap();
}

#[tokio::test]
async fn invalid_f5_policy_files_prevent_account_serving_startup() {
    let dir = TempDir::new();
    let mut config = dir.config();
    drop(Store::open(&config.database).unwrap());
    let path = dir.0.join("f5.json");
    config.f5_social_policy = Some(path.clone());
    for policy in ["{", r#"{"note_inbox_cap":101}"#] {
        fs::write(&path, policy).unwrap();
        let error = Owner::open(&config, Default::default(), Some(projection()))
            .await
            .err()
            .expect("invalid F5 policy allowed account serving startup");
        assert_eq!(error.kind(), io::ErrorKind::InvalidData);
        assert!(!config.launch_socket.exists());
    }
}

#[tokio::test]
async fn embedded_web_shop_reports_opted_in_f6_status() {
    let dir = TempDir::new();
    let mut config = dir.config();
    let mut policy: Value = serde_json::from_str(include_str!(
        "../../../../server/profiles/f6-events-disabled.json"
    ))
    .unwrap();
    policy["systems"]["CAPass"] = json!({
        "enabled": true,
        "start_utc": "2027-01-01T00:00:00Z",
        "end_utc": "2027-01-02T00:00:00Z",
        "rate_caps": {"per_account_per_hour": 6, "global_per_minute": 43}
    });
    config.f6_events_policy = Some(dir.policy("f6.json", policy));

    let (addr, mut owner) = open_owner(&mut config).await.unwrap();
    let (token, shutdown, task) = run_shop(&mut owner);
    let (status, body) = request(addr, &token, EVENTS, None).await;
    assert_eq!(status, 200);
    assert_eq!(body["configured"], true);
    assert_eq!(
        body["enabled_systems"],
        json!([{
            "system": "CAPass",
            "start_utc": 1_798_761_600,
            "end_utc": 1_798_848_000,
            "rate_caps": {"per_account_per_hour": 6, "global_per_minute": 43}
        }])
    );
    shutdown.send(true).unwrap();
    task.await.unwrap().unwrap();
    owner.shutdown().await.unwrap();
}

#[tokio::test]
async fn omitted_and_disabled_f7_do_not_create_directories() {
    for configured in [false, true] {
        let dir = TempDir::new();
        let mut config = dir.config();
        if configured {
            config.f7_custom_map_policy = Some(dir.policy("f7.json", json!({"enabled": false})));
        }
        let (addr, mut owner) = open_owner(&mut config).await.unwrap();
        let (token, shutdown, task) = run_shop(&mut owner);
        let (status, events) = request(addr, &token, EVENTS, None).await;
        assert_eq!(status, 200);
        assert_eq!(events["configured"], false);
        assert_eq!(events["enabled_systems"], json!([]));
        assert_eq!(events["extended_systems"], json!([]));
        for body in [None, Some(SAVE)] {
            assert_eq!(
                request(addr, &token, MAPS, body).await,
                (
                    403,
                    json!({"error": "custom_maps_disabled", "wire_contract": "unmeasured"})
                )
            );
        }
        for entry in fs::read_dir(&dir.0).unwrap() {
            assert!(!entry.unwrap().file_type().unwrap().is_dir());
        }
        shutdown.send(true).unwrap();
        task.await.unwrap().unwrap();
        owner.shutdown().await.unwrap();
    }
}

#[tokio::test]
async fn opted_in_f7_uses_a_private_db_sibling_and_applies_limits() {
    let dir = TempDir::new();
    let mut config = dir.config();
    config.f7_custom_map_policy = Some(dir.policy(
        "f7.json",
        json!({"enabled": true, "max_maps_per_owner": 3, "max_payload_bytes": 1}),
    ));
    let (addr, mut owner) = open_owner(&mut config).await.unwrap();
    let root = dir.0.join("accounts.sqlite.custom-maps");
    platform::private_dir(&root).unwrap();
    let (token, shutdown, task) = run_shop(&mut owner);
    assert_eq!(
        request(addr, &token, MAPS, None).await,
        (
            200,
            json!({"maps": [], "quota_remaining": 3, "wire_contract": "unmeasured"})
        )
    );
    assert_eq!(
        request(addr, &token, MAPS, Some(SAVE)).await,
        (
            413,
            json!({"error": "custom_map_too_large", "wire_contract": "unmeasured"})
        )
    );
    assert!(fs::read_dir(&root).unwrap().next().is_none());
    shutdown.send(true).unwrap();
    task.await.unwrap().unwrap();
    owner.shutdown().await.unwrap();
}

#[cfg(unix)]
#[tokio::test]
async fn enabled_f7_rejects_a_symlink_store_root() {
    let dir = TempDir::new();
    let target = TempDir::new();
    std::os::unix::fs::symlink(&target.0, dir.0.join("accounts.sqlite.custom-maps")).unwrap();
    let mut config = dir.config();
    config.f7_custom_map_policy = Some(dir.policy("f7.json", json!({"enabled": true})));
    let error = open_owner(&mut config)
        .await
        .err()
        .expect("symlink accepted");
    assert!(error.to_string().contains("symlinks are not allowed"));
    assert!(fs::read_dir(&target.0).unwrap().next().is_none());
}
