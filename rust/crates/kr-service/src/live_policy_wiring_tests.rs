use super::{Active, Config, Owner};
use crate::{
    account_bridge::local::native::fixtures,
    account_worker::{EconomySettings, Error as WorkerError, SocialReply},
    accounts::{
        Admin, Error, Grant, ItemUpgradeOperation, Player, RecordEraserOperation, Store, Token,
    },
    choco_chat::{F5SocialPolicy, F5SocialRuntimeSnapshot},
    f3_pve_policy::{DispatchRejection, PveModeFamily},
    f3_pve_runtime::{DispatchEvidence, MonsterAttack, MonsterSpawn, RuntimeError},
    f4_extended_policy::{
        CouponRequest, Decision, ItemOrigin, OperationKey, OwnedItem, RecordScope,
        SecondPasswordAction, SecondPasswordOutcome, TitleUnlockDecision, UpgradeFamily,
    },
    f6_runtime::F6QuotaUsage,
    platform, web_shop,
};
use kr_wire::kr00967::{AccountTemplate, f3_pve::F3JumpTableProvenance};
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
const F3_PVE_STATUS: &str = "/api/v1/f3/pve-status";
const SECURITY_KITS: &str = "/api/v1/f4/security-kits-status";
const SOCIAL_POLICY: &str = "/api/social/policy-status";
const CAPASS_START: i64 = 1_798_761_600;
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

async fn login_with_retry(active: &Active) -> Grant {
    // The account worker admits at most five attempts per login in a minute.
    let mut retries = 4;
    loop {
        let pending = active
            .client
            .try_login_with_idle_current("alice", b"test-password", 60, 30)
            .expect("submit live policy test login");
        match pending.wait().await {
            Err(WorkerError::Backend(Error::Busy)) if retries > 0 => {
                retries -= 1;
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
            result => return result.expect("complete live policy test login"),
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

fn capass_policy(dir: &TempDir) -> PathBuf {
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
    dir.policy("f6.json", policy)
}

fn empty_f5_snapshot(enabled: bool) -> F5SocialRuntimeSnapshot {
    F5SocialRuntimeSnapshot {
        note_inboxes: 0,
        notes: 0,
        unread_notes: 0,
        couples: 0,
        ringed_couples: 0,
        plaza_occupants: 0,
        plaza_seated: 0,
        active_plaza_zones: 0,
        notes_enabled: enabled,
        couple_enabled: enabled,
        plaza_enabled: enabled,
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
    // These policy and map routes only need an authenticated owner ID.
    let token = owner
        .web_shop
        .as_ref()
        .unwrap()
        .auth()
        .issue_token(1)
        .unwrap();
    let (shutdown, task) = serve_shop(owner);
    (token, shutdown, task)
}

fn serve_shop(owner: &mut Owner) -> (watch::Sender<bool>, tokio::task::JoinHandle<io::Result<()>>) {
    let opened = owner.web_shop.take().unwrap();
    let service = owner.active.account_service().unwrap().clone();
    let (shutdown, stop) = watch::channel(false);
    let task = tokio::spawn(web_shop::serve_until(opened, service, stop));
    (shutdown, task)
}

#[tokio::test]
async fn default_active_and_embedded_web_shop_policies_are_disabled() {
    let dir = TempDir::new();
    let mut config = dir.config();
    let player = create_account(&config);
    let (addr, mut owner) = open_owner(&mut config).await.unwrap();
    let token = owner
        .web_shop
        .as_ref()
        .unwrap()
        .auth()
        .issue_token(player.player_id)
        .unwrap();
    let (shutdown, task) = serve_shop(&mut owner);
    let f4 = owner.active.f4_security_kits().snapshot().unwrap();
    assert!(!f4.powerup_kits.enabled);
    assert!(!f4.second_password.enabled);
    assert!(!owner.active.f3_pve_policy().enabled);
    assert_eq!(
        owner.active.f5_social_runtime_snapshot(),
        empty_f5_snapshot(false)
    );
    assert!(
        owner
            .active
            .f6_event_status_snapshot(CAPASS_START)
            .is_none()
    );
    let (status, body) = request(addr, &token, SECURITY_KITS, None).await;
    assert_eq!(status, 200);
    assert_eq!(body["powerup_kits"], json!(f4.powerup_kits));
    assert_eq!(body["second_password"], json!(f4.second_password));
    assert_eq!(body["second_password_unlocked"], false);
    assert_eq!(
        request(addr, &token, SOCIAL_POLICY, None).await,
        (
            200,
            json!({
                "enabled": {
                    "whisper": false,
                    "guild_chat": false,
                    "megaphone": false,
                    "offline_note": false,
                    "couple": false,
                    "plaza": false
                },
                "channel_eligibility": {
                    "whisper": false,
                    "guild_chat": false,
                    "megaphone_global": false,
                    "megaphone_channel": false
                }
            })
        )
    );
    shutdown.send(true).unwrap();
    task.await.unwrap().unwrap();
    owner.shutdown().await.unwrap();
}

#[tokio::test]
async fn embedded_web_shop_reports_live_f4_and_f5_and_revokes_sessions_on_logout() {
    let dir = TempDir::new();
    let mut config = dir.config();
    config.f4_extended_policy = Some(dir.policy(
        "f4-security.json",
        json!({
            "powerup_kits": {
                "enabled": true,
                "max_equipped_slots": 1,
                "kits": [{"sku": "kit.test", "bonus": {"bombs": 1, "blast": 0, "speed": 0}}]
            },
            "second_password": {
                "enabled": true,
                "max_failed_attempts": 2,
                "failure_window_seconds": 60,
                "cooldown_seconds": 60
            }
        }),
    ));
    config.f5_social_policy =
        Some(dir.policy("f5.json", json!({"whisper": true, "guild_chat": true})));
    let player = create_account(&config);
    let (addr, mut owner) = open_owner(&mut config).await.unwrap();

    let grant = login_with_retry(&owner.active).await;
    assert_eq!(grant.player, player);
    let session = Active::public_primary_session(&grant.token, player.player_id);
    let token = owner
        .web_shop
        .as_ref()
        .unwrap()
        .auth()
        .issue_token_for_primary_session(&player, session)
        .unwrap();
    let (shutdown, task) = serve_shop(&mut owner);
    let (status, before) = request(addr, &token, SECURITY_KITS, None).await;
    assert_eq!(status, 200);
    assert_eq!(before["powerup_kits"]["enabled"], true);
    assert_eq!(before["second_password"]["enabled"], true);
    assert_eq!(before["powerup_kits"]["tracked_players"], 0);
    assert_eq!(before["second_password_unlocked"], false);

    let (status, social) = request(addr, &token, SOCIAL_POLICY, None).await;
    assert_eq!(status, 200);
    for channel in ["whisper", "guild_chat"] {
        assert_eq!(social["enabled"][channel], true);
        assert_eq!(social["channel_eligibility"][channel], true);
    }

    let runtime = owner.active.f4_security_kits();
    runtime
        .powerup_kits
        .equip(
            session.account,
            1,
            0,
            OwnedItem {
                owner: session.account,
                sku: "kit.test",
                quantity: 1,
                expires_at: None,
                origin: ItemOrigin::PersistentInventory,
            },
        )
        .unwrap();
    let password = &runtime.second_password;
    password.setup(session).unwrap();
    assert_eq!(
        password.attempt(session, 1, SecondPasswordAction::Unlock, true),
        Ok(SecondPasswordOutcome::Unlocked)
    );

    let (status, after) = request(addr, &token, SECURITY_KITS, None).await;
    assert_eq!(status, 200);
    assert_eq!(after["powerup_kits"]["tracked_players"], 1);
    assert_eq!(after["second_password"]["tracked_players"], 1);
    assert_eq!(after["second_password_unlocked"], true);

    let wrong_token = Token::parse(&"0".repeat(64)).unwrap();
    assert!(
        !owner
            .active
            .logout_public(wrong_token, player.player_id)
            .await
            .unwrap()
    );
    assert_eq!(
        request(addr, &token, SECURITY_KITS, None).await,
        (200, after)
    );
    assert!(
        owner
            .active
            .logout_public(grant.token, player.player_id)
            .await
            .unwrap()
    );
    assert!(!password.is_unlocked(session).unwrap());
    for target in [SECURITY_KITS, SOCIAL_POLICY] {
        assert_eq!(request(addr, &token, target, None).await.0, 401);
    }
    shutdown.send(true).unwrap();
    task.await.unwrap().unwrap();
    owner.shutdown().await.unwrap();
}

#[tokio::test]
async fn embedded_web_shop_reports_disabled_f3_status_by_default() {
    let dir = TempDir::new();
    let mut config = dir.config();
    let (addr, mut owner) = open_owner(&mut config).await.unwrap();
    let (token, shutdown, task) = run_shop(&mut owner);
    assert_eq!(
        request(addr, &token, F3_PVE_STATUS, None).await,
        (
            200,
            json!({
                "enabled": false,
                "active_rooms": 0,
                "max_rooms": 0,
                "dispatch": {
                    "status": "static_candidate_pending_live_wine_capture",
                    "jump_table_va": "0xE5DE9C",
                    "opcode_index_subtrahend": 20,
                },
            })
        )
    );
    shutdown.send(true).unwrap();
    task.await.unwrap().unwrap();
    owner.shutdown().await.unwrap();
}

#[tokio::test]
async fn embedded_web_shop_reports_live_f3_room_lifecycle() {
    let dir = TempDir::new();
    let mut config = dir.config();
    config.f3_pve_policy = Some(dir.policy("f3.json", json!({"enabled": true})));
    let (addr, mut owner) = open_owner(&mut config).await.unwrap();
    let (token, shutdown, task) = run_shop(&mut owner);
    let registry = owner.active.f3_pve_registry();

    let fetch = || request(addr, &token, F3_PVE_STATUS, None);
    let (status, mut expected) = fetch().await;
    assert_eq!(status, 200);
    assert_eq!(expected["enabled"], true);
    assert_eq!(expected["max_rooms"], 64);
    assert_eq!(expected["active_rooms"], 0);

    // Synthetic evidence exercises the registry API without asserting a Wine capture.
    let evidence = DispatchEvidence::present(F3JumpTableProvenance::C_GAME_STAGE);
    registry
        .start_room(1, PveModeFamily::MonsterS1, evidence, 1, 7, 1)
        .unwrap();
    expected["active_rooms"] = json!(1);
    assert_eq!(fetch().await, (200, expected.clone()));

    registry.end_room(1).unwrap();
    expected["active_rooms"] = json!(0);
    assert_eq!(fetch().await, (200, expected));

    shutdown.send(true).unwrap();
    task.await.unwrap().unwrap();
    owner.shutdown().await.unwrap();
}

#[tokio::test]
async fn omitted_and_disabled_f3_policies_keep_registry_closed() {
    for configured in [false, true] {
        let dir = TempDir::new();
        let mut config = dir.config();
        if configured {
            config.f3_pve_policy = Some(dir.policy("f3.json", json!({"enabled": false})));
        }
        create_account(&config);
        let owner = Owner::open(&config, Default::default(), Some(projection()))
            .await
            .unwrap();
        let registry = owner.active.f3_pve_registry();
        let snapshot = registry.snapshot();
        assert!(!snapshot.enabled);
        assert_eq!(snapshot.max_rooms, 0);
        assert_eq!(
            registry.start_room(
                1,
                PveModeFamily::MonsterS1,
                DispatchEvidence::missing(),
                1,
                1,
                1,
            ),
            Err(RuntimeError::DispatchDisabled)
        );
        owner.shutdown().await.unwrap();
    }
}

#[tokio::test]
async fn enabled_f3_registry_shares_room_lifecycle_across_active_clones() {
    let dir = TempDir::new();
    let mut config = dir.config();
    config.f3_pve_policy = Some(dir.policy("f3.json", json!({"enabled": true})));
    create_account(&config);
    let owner = Owner::open(&config, Default::default(), Some(projection()))
        .await
        .unwrap();
    let policy = owner.active.f3_pve_policy();
    assert!(policy.enabled);
    let family = PveModeFamily::MonsterS1;
    assert_eq!(
        policy.ensure_dispatch_allowed(family),
        Err(DispatchRejection::WineEvidenceRequired(family))
    );
    let registry = owner.active.f3_pve_registry();
    let snapshot = registry.snapshot();
    assert!(snapshot.enabled);
    assert_eq!(snapshot.max_rooms, 64);
    assert_eq!(snapshot.active_rooms, 0);

    let (room_id, stage, host_id, round_epoch) = (1, 1, 7, 1);
    assert_eq!(
        registry.start_room(
            room_id,
            family,
            DispatchEvidence::missing(),
            stage,
            host_id,
            round_epoch,
        ),
        Err(RuntimeError::WineDispatchEvidenceRequired)
    );
    assert_eq!(registry.snapshot().active_rooms, 0);

    // Synthetic evidence exercises the registry API without asserting a Wine capture.
    let evidence = DispatchEvidence::present(F3JumpTableProvenance::C_GAME_STAGE);
    registry
        .start_room(room_id, family, evidence, stage, host_id, round_epoch)
        .unwrap();

    let active_clone = owner.active.clone();
    let shared_registry = active_clone.f3_pve_registry();
    assert_eq!(shared_registry.snapshot().active_rooms, 1);
    shared_registry
        .advance_wave(room_id, 1, &[MonsterSpawn { id: 1, hp: 10 }])
        .unwrap();
    shared_registry
        .apply_host_attack(
            room_id,
            MonsterAttack {
                actor_id: host_id,
                round_epoch,
                attack_sequence: 1,
                monster_id: 1,
                damage: 10,
            },
        )
        .unwrap();
    assert_eq!(registry.snapshot().active_rooms, 1);
    shared_registry.end_room(room_id).unwrap();
    assert_eq!(registry.snapshot().active_rooms, 0);
    owner.shutdown().await.unwrap();
}

#[tokio::test]
async fn invalid_f3_policy_fails_closed() {
    let invalid = TempDir::new();
    let mut invalid_config = invalid.config();
    drop(Store::open(&invalid_config.database).unwrap());
    invalid_config.f3_pve_policy =
        Some(invalid.policy("f3-invalid.json", json!({"unknown": true})));
    let error = Owner::open(&invalid_config, Default::default(), Some(projection()))
        .await
        .err()
        .expect("invalid F3 policy allowed account serving startup");
    assert_eq!(error.kind(), io::ErrorKind::InvalidData);
    assert!(!invalid_config.launch_socket.exists());
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
async fn configured_f5_and_f6_snapshots_are_available_without_a_web_shop() {
    let dir = TempDir::new();
    let mut config = dir.config();
    config.f5_social_policy = Some(dir.policy(
        "f5.json",
        json!({
            "note_inbox_cap": 4,
            "couple": true,
            "plaza_zones": {"lobby_square": {"max_occupants": 8, "seats": 2}}
        }),
    ));
    config.f6_events_policy = Some(capass_policy(&dir));
    drop(Store::open(&config.database).unwrap());
    let owner = Owner::open(&config, Default::default(), Some(projection()))
        .await
        .unwrap();
    let active = owner.active.clone();
    assert_eq!(active.f5_social_runtime_snapshot(), empty_f5_snapshot(true));

    let before = active.f6_event_status_snapshot(CAPASS_START - 1).unwrap();
    assert!(before.enabled);
    assert_eq!(before.configured_systems, ["CAPass"]);
    assert!(before.active_systems.is_empty());
    let during = owner.active.f6_event_status_snapshot(CAPASS_START).unwrap();
    assert_eq!(during.configured_systems, before.configured_systems);
    assert_eq!(during.active_systems, ["CAPass"]);
    assert_eq!(
        during.quota_usage,
        F6QuotaUsage {
            claims: 0,
            hourly: 0,
            minute: 0,
            daily: 0,
            seasonal: 0
        }
    );

    let login_only = Active::login_test_client(active.client.clone());
    assert_eq!(
        login_only.f5_social_runtime_snapshot(),
        empty_f5_snapshot(false)
    );
    assert!(login_only.f6_event_status_snapshot(CAPASS_START).is_none());

    owner.shutdown().await.unwrap();
    assert!(active.f6_event_status_snapshot(CAPASS_START).is_none());
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
    config.f6_events_policy = Some(capass_policy(&dir));

    let (addr, mut owner) = open_owner(&mut config).await.unwrap();
    let (token, shutdown, task) = run_shop(&mut owner);
    let (status, body) = request(addr, &token, EVENTS, None).await;
    assert_eq!(status, 200);
    assert_eq!(body["configured"], true);
    assert_eq!(
        body["enabled_systems"],
        json!([{
            "system": "CAPass",
            "start_utc": CAPASS_START,
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
