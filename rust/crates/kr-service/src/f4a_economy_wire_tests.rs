use super::*;
use crate::accounts::{
    Admin, CatalogAccess, EconomyPolicy, ItemGrant, MatchKey, MatchParticipant, MatchStart,
    RewardAmount, ShopCatalog, ShopCategory, ShopItem, Store, TEST_SERIAL,
};
use crate::match_journal::{Manager, utc};
use kr_wire::{Receiver, Width, write_uint};
use std::{
    fs,
    net::Ipv4Addr,
    os::unix::fs::PermissionsExt,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
    time::{Duration, Instant},
};
use tokio::io::AsyncReadExt;

static NEXT: AtomicU64 = AtomicU64::new(0);

struct TempDir(PathBuf);
impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn temp_dir(tag: &str) -> TempDir {
    let root = std::env::temp_dir().join(format!(
        "kr-f4a-wire-{tag}-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&root).unwrap();
    fs::set_permissions(&root, fs::Permissions::from_mode(0o700)).unwrap();
    TempDir(root)
}

fn base_config(dir: &Path) -> Config {
    Config {
        backup: None,
        prelogin_tls: None,
        public_launcher: None,
        persist_character: false,
        persist_matches: true,
        database: dir.join("accounts.sqlite"),
        launch_socket: dir.join("launch.sock"),
        launch_ttl_seconds: 60,
        worker_queue_capacity: 8,
        profile_mode: ProfileMode::DatabaseZero,
        patched_949x_names: false,
        max_sessions: Some(16),
        diagnostic_identity_strings: Default::default(),
        economy_policy: EconomyPolicy::default(),
        shop_catalog: ShopCatalog::default(),
        economy_profile_path: None,
        f6_events_policy: None,
        emit_lucci_135: false,
        project_own291_level: false,
        unlock_all_catalog: false,
        web_shop: None,
    }
}

fn sample_policy() -> EconomyPolicy {
    EconomyPolicy {
        enabled: true,
        win: RewardAmount {
            exp: 100,
            lucci: 250,
        },
        loss: RewardAmount { exp: 30, lucci: 50 },
        draw: RewardAmount { exp: 50, lucci: 80 },
        grant_on_disconnect_or_left: false,
        level_thresholds: vec![0, 100, 400],
    }
}

fn sample_catalog(unlock_all: bool) -> ShopCatalog {
    ShopCatalog {
        items: vec![
            ShopItem {
                sku_id: "char.bazzi".into(),
                category: ShopCategory::Character,
                price_lucci: if unlock_all { 0 } else { 3000 },
                grant: ItemGrant::Permanent,
                equip_slot: Some("character".into()),
                enabled: true,
            },
            ShopItem {
                sku_id: "emote.disabled".into(),
                category: ShopCategory::Emotion,
                price_lucci: 0,
                grant: ItemGrant::Permanent,
                equip_slot: None,
                enabled: false,
            },
        ],
    }
}

#[test]
fn config_economy_profile_loader_accepts_exporter_json_and_rejects_ambiguity_or_oversize() {
    let dir = temp_dir("profile");
    let profile_path = dir.0.join("f4-export.json");
    let exporter_json = serde_json::json!({
        "economy_policy": sample_policy(),
        "shop_catalog": sample_catalog(false),
        "summary": {
            "level_count": 3,
            "unlock_all": false,
            "catalog_sha256": "deadbeef"
        }
    });
    fs::write(&profile_path, serde_json::to_vec(&exporter_json).unwrap()).unwrap();

    let mut cfg = base_config(&dir.0);
    cfg.economy_profile_path = Some(profile_path.clone());
    let loaded = cfg.economy().unwrap();
    assert!(loaded.rewards_enabled());
    assert_eq!(loaded.catalog().items.len(), 2);

    // Ambiguous mix with inline non-default economy config must fail closed.
    let mut mixed = base_config(&dir.0);
    mixed.economy_profile_path = Some(profile_path.clone());
    mixed.economy_policy = sample_policy();
    assert!(mixed.economy().is_err());

    let mut mixed_cat = base_config(&dir.0);
    mixed_cat.economy_profile_path = Some(profile_path);
    mixed_cat.shop_catalog = sample_catalog(false);
    assert!(mixed_cat.economy().is_err());

    // ParentDir traversal rejected.
    let mut traversal = base_config(&dir.0);
    traversal.economy_profile_path = Some(dir.0.join("../escape.json"));
    assert!(traversal.economy().is_err());

    // Fixed profile mode rejects opt-in F4 flags.
    let mut fixed = base_config(&dir.0);
    fixed.profile_mode = ProfileMode::Fixed;
    fixed.persist_matches = false;
    fixed.emit_lucci_135 = true;
    assert!(fixed.economy().is_err());
    fixed.emit_lucci_135 = false;
    fixed.project_own291_level = true;
    assert!(fixed.economy().is_err());
    fixed.project_own291_level = false;
    fixed.unlock_all_catalog = true;
    assert!(fixed.economy().is_err());

    // Oversized profile file (>16 MiB) rejected before parsing.
    let big_path = dir.0.join("oversize.json");
    let f = fs::File::create(&big_path).unwrap();
    f.set_len(MAX_ECONOMY_PROFILE_BYTES + 1).unwrap();
    let mut oversize = base_config(&dir.0);
    oversize.economy_profile_path = Some(big_path);
    assert!(oversize.economy().is_err());
}

#[test]
fn config_web_shop_requires_database_zero_and_matching_unlock_all() {
    let dir = temp_dir("ws-validate");
    let mut cfg = base_config(&dir.0);
    fs::write(&cfg.database, b"").unwrap();
    fs::set_permissions(&cfg.database, fs::Permissions::from_mode(0o600)).unwrap();

    let mut ws = crate::web_shop::Config {
        bind: std::net::SocketAddrV4::new(Ipv4Addr::LOCALHOST, 8080),
        allowed_peers: vec![Ipv4Addr::LOCALHOST],
        max_connections: 4,
        request_timeout_ms: 1000,
        max_body_bytes: 1024,
        session_ttl_seconds: 60,
        unlock_all: false,
    };

    // Valid DatabaseZero + matching unlock_all = false.
    cfg.web_shop = Some(ws.clone());
    assert!(cfg.validate().is_ok());

    // Mismatched unlock_all rejected.
    ws.unlock_all = true;
    cfg.web_shop = Some(ws.clone());
    assert!(cfg.validate().is_err());

    // Matching unlock_all = true accepted in DatabaseZero.
    cfg.unlock_all_catalog = true;
    assert!(cfg.validate().is_ok());

    // ProfileMode::Fixed rejected even when unlock_all matches.
    cfg.profile_mode = ProfileMode::Fixed;
    cfg.persist_matches = false;
    cfg.unlock_all_catalog = false;
    ws.unlock_all = false;
    cfg.web_shop = Some(ws);
    assert!(cfg.validate().is_err());
}

#[tokio::test]
async fn config_unlock_all_catalog_projects_enabled_items_without_db_mutation() {
    let dir = temp_dir("unlock");
    let now = utc().unwrap();
    let player_id = {
        let _serial = TEST_SERIAL.lock().unwrap();
        let mut store = Store::open(&dir.0.join("accounts.sqlite")).unwrap();
        let admin = Admin {
            actor: "f4a-test",
            reason: "unlock_all test",
        };
        store.begin_instance(&admin, now).unwrap();
        store
            .create_account(&admin, "o1user", "O1User", b"unlock-all-password-1", now)
            .unwrap()
            .player_id
    };

    let worker = Worker::open_with_economy(
        dir.0.join("accounts.sqlite"),
        8,
        Duration::from_secs(2),
        EconomySettings::new(EconomyPolicy::default(), sample_catalog(true)).unwrap(),
    )
    .await
    .unwrap();
    let service = worker.service();
    let active = Active {
        authenticated: None,
        bridge: Bridge::new(worker.client(), 4, Duration::from_secs(30)).unwrap(),
        client: worker.client(),
        service: Some(service.clone()),
        emit_lucci_135: false,
        project_own291_level: false,
        unlock_all_catalog: true,
        web_shop: None,
        projections: Arc::new(Default::default()),
        projection: None,
    };

    assert_eq!(active.catalog_access(), CatalogAccess { unlock_all: true });
    let usable = service
        .usable_items(player_id, active.catalog_access())
        .await
        .unwrap();
    assert_eq!(usable.len(), 1);
    assert_eq!(usable[0].sku_id, "char.bazzi");
    assert!(usable[0].unlocked_by_session);
    // DB inventory stays completely empty (zero ownership mutation).
    assert!(service.inventory_list(player_id).await.unwrap().is_empty());
    assert_eq!(service.profile_get(player_id).await.unwrap().lucci, 0);
    worker.shutdown().await.unwrap();
}

fn login_body(secret: &str, peer: Ipv4Addr) -> Vec<u8> {
    let mut out = vec![];
    for (value, width) in [
        (4, Width::U8),
        (1, Width::U8),
        (secret.len() as u64, Width::U16),
    ] {
        out.extend(write_uint(value, width).unwrap());
    }
    out.extend(kr_wire::crypt(secret.as_bytes(), 0x1122_3344, false));
    out.extend([0xa5; 16]);
    out.extend(write_uint(0, Width::U32).unwrap());
    out.extend(write_uint(u32::from_le_bytes(peer.octets()) as u64, Width::U32).unwrap());
    out.extend(write_uint(0, Width::U8).unwrap());
    out
}

fn channel_body(player: &crate::accounts::Player, echo: u32) -> Vec<u8> {
    let wire_name = kr_wire::kr00967::text949x::encode(&player.display_name).unwrap();
    let mut out = vec![];
    out.extend(write_uint(1, Width::U8).unwrap());
    out.extend(write_uint(255, Width::U8).unwrap());
    out.extend(write_uint(wire_name.len() as u64, Width::U16).unwrap());
    let mut name = wire_name.as_slice();
    while !name.is_empty() {
        let (n, w) = if name.len() >= 4 {
            (4, Width::U32)
        } else if name.len() >= 2 {
            (2, Width::U16)
        } else {
            (1, Width::U8)
        };
        let mut bytes = [0u8; 8];
        bytes[..n].copy_from_slice(&name[..n]);
        out.extend(write_uint(u64::from_le_bytes(bytes), w).unwrap());
        name = &name[n..];
    }
    out.extend(write_uint(player.wire_id as u64, Width::U32).unwrap());
    out.extend(write_uint(echo as u64, Width::U32).unwrap());
    out.extend([0x8c; 24]);
    out[31 + wire_name.len()] = write_uint(0, Width::U8).unwrap()[0];
    out
}

fn dummy_validated(dir: &Path, active: Option<Active>) -> crate::Validated {
    crate::Validated {
        config: crate::Config {
            udp_relay: None,
            authenticated_relay: None,
            schema: "kr-rust-diagnostic-v1".into(),
            diagnostic_login_error_only: true,
            bind: Ipv4Addr::LOCALHOST,
            allowed_peers: vec![Ipv4Addr::LOCALHOST],
            network_scope: crate::network::NetworkScope::PrivateLab,
            ports: vec![8484],
            max_connections: 4,
            idle_timeout_seconds: 10,
            write_timeout_ms: 1000,
            bootstrap: crate::FixturePin {
                path: dir.join("boot"),
                bytes: 15,
                sha256: "75be47b3067e2b1e9d2cf99b5a7eb7f949a074482409fb3eafc1a553cec53cd1".into(),
            },
            login_error: crate::FixturePin {
                path: dir.join("err"),
                bytes: 39,
                sha256: "9aeae7b7937ad0c54d968297c48fffb4d506087f7826b2e9fdde35d84ed3ac19".into(),
            },
            wcp_second: crate::FixturePin {
                path: dir.join("wcp"),
                bytes: 15,
                sha256: "5930ad297fc2c1540d8cbd894e88d3c1d2831bd4565dfb8ad96644568b87ad38".into(),
            },
            diagnostic_local_lobby: None,
            diagnostic_two_lobby: None,
        },
        bootstrap: vec![0; 15],
        error_header: [0; 37],
        wcp_second: vec![0; 15],
        lobby: None,
        bound: None,
        accounts: active,
    }
}

#[tokio::test]
async fn opt_in_emit_lucci_135_sends_valid_135_frame_with_settled_lucci_and_default_emits_nothing()
{
    let dir = temp_dir("emit135");
    let db = dir.0.join("accounts.sqlite");
    let now = utc().unwrap();
    let (epoch, player, loser, grant) = {
        let _serial = TEST_SERIAL.lock().unwrap();
        let mut store = Store::open(&db).unwrap();
        let admin = Admin {
            actor: "f4a-wire-test",
            reason: "settled 135 wire test",
        };
        let epoch = store.begin_instance(&admin, now).unwrap();
        let p = store
            .create_account(&admin, "lucciuser", "Lucci", b"lucci-wire-password-1", now)
            .unwrap();
        let l = store
            .create_account(&admin, "luccilose", "Lose", b"lucci-wire-password-2", now)
            .unwrap();
        let g = store
            .login("lucciuser", b"lucci-wire-password-1", 60, now)
            .unwrap();
        (epoch, p, l, g)
    };

    let worker = Worker::open_with_economy(
        db.clone(),
        8,
        Duration::from_secs(2),
        EconomySettings::new(sample_policy(), ShopCatalog::default()).unwrap(),
    )
    .await
    .unwrap();

    // Settle one winning match through the real match_journal -> AccountWorker seam (+250 Lucci).
    let manager = Manager::open(worker.client(), epoch);
    let mut owner = manager.factory().owner();
    owner
        .begin(MatchStart {
            key: MatchKey {
                boot_epoch: epoch,
                room_session: 1,
                room_id: 1,
                room_generation: 1,
                round: 256,
            },
            map_code: 278,
            map_name: "ExVillage01".into(),
            started_at: now,
            players: vec![
                MatchParticipant {
                    account_id: player.account_id,
                    player_id: player.player_id,
                    slot: 0,
                    character_key: 0,
                    team: 0,
                },
                MatchParticipant {
                    account_id: loser.account_id,
                    player_id: loser.player_id,
                    slot: 1,
                    character_key: 0,
                    team: 1,
                },
            ],
        })
        .unwrap();
    owner.ticket().unwrap().wait().await.unwrap();
    owner.finish(0).unwrap();
    owner.ticket().unwrap().wait().await.unwrap();

    let bridge = Bridge::new(worker.client(), 4, Duration::from_secs(30)).unwrap();
    let peer = Ipv4Addr::new(127, 0, 0, 2);
    let launch = bridge
        .issue(&grant.token, player.player_id, peer)
        .await
        .unwrap();
    let secret = launch.token().expose_secret().to_owned();
    launch.delivered();
    let login = bridge
        .login(&login_body(&secret, peer), peer, 1)
        .await
        .unwrap();
    let echo = login.echo;
    login.redirected().unwrap();
    let channel = bridge
        .channel(&channel_body(&player, echo), peer, 2)
        .await
        .unwrap();
    let lease = Lease::Channel(channel);

    let (log_tx, mut log_rx) = tokio::sync::mpsc::channel(16);
    let (_stop_tx, mut stop_rx) = tokio::sync::watch::channel(false);
    let start = Instant::now();

    // Default (`emit_lucci_135 = false`) emits zero bytes and leaves sequence unchanged.
    let default_active = Active {
        authenticated: None,
        bridge: bridge.clone(),
        client: worker.client(),
        service: Some(worker.service()),
        emit_lucci_135: false,
        project_own291_level: false,
        unlock_all_catalog: false,
        web_shop: None,
        projections: Arc::new(Default::default()),
        projection: None,
    };
    let listener = tokio::net::TcpListener::bind((Ipv4Addr::LOCALHOST, 0))
        .await
        .unwrap();
    let addr = listener.local_addr().unwrap();
    let mut client_sock = tokio::net::TcpStream::connect(addr).await.unwrap();
    let (mut server_sock, _) = listener.accept().await.unwrap();
    let mut seq = 43u32;
    crate::emit_lucci_135_if_enabled(
        &mut server_sock,
        &dummy_validated(&dir.0, Some(default_active)),
        Some(&lease),
        player.wire_id,
        &mut seq,
        "lobby_own291",
        &log_tx,
        2,
        &mut stop_rx,
        start,
    )
    .await
    .unwrap();
    assert_eq!(seq, 43);

    // Opt-in (`emit_lucci_135 = true`) emits valid S->C 135 carrying settled 250 Lucci.
    let opt_in_active = Active {
        authenticated: None,
        bridge,
        client: worker.client(),
        service: Some(worker.service()),
        emit_lucci_135: true,
        project_own291_level: false,
        unlock_all_catalog: false,
        web_shop: None,
        projections: Arc::new(Default::default()),
        projection: None,
    };
    crate::emit_lucci_135_if_enabled(
        &mut server_sock,
        &dummy_validated(&dir.0, Some(opt_in_active)),
        Some(&lease),
        player.wire_id,
        &mut seq,
        "post_match_175",
        &log_tx,
        2,
        &mut stop_rx,
        start,
    )
    .await
    .unwrap();
    assert_eq!(seq, 46);
    drop(server_sock);

    let mut wire_bytes = Vec::new();
    client_sock.read_to_end(&mut wire_bytes).await.unwrap();
    let mut rx = Receiver::new(43);
    rx.push(&wire_bytes).unwrap();
    let packet = rx.next_packet().unwrap().unwrap();
    assert!(rx.next_packet().unwrap().is_none());
    assert_eq!(packet.opcode, 135);
    assert_eq!(
        kr_wire::kr00967::decode_lucci135(&packet.payload).unwrap(),
        (0, 250)
    );
    let logged = log_rx.recv().await.unwrap();
    assert_eq!(logged["event"], "economy_lucci_135_sent");
    assert_eq!(logged["lucci"], 250);
    assert_eq!(logged["stage"], "post_match_175");
    worker.shutdown().await.unwrap();
}
