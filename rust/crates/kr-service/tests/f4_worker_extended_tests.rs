use kr_service::{
    account_worker::{AccountService, EconomySettings, Error, Worker},
    accounts::{
        Admin, EconomyPolicy, MatchEndReason, MatchExit, MatchFinish, MatchKey, MatchOutcome,
        MatchParticipant, MatchPlayerFinish, MatchStart, RewardAmount, ShopCatalog, Store,
    },
    f4_extended_policy::{
        BoosterRoom, BoosterRoomContext, BoosterTicket, BoosterTicketPolicy, Decision, LevelGrant,
        LevelLucci, LevelReward, LevelRewardPolicy, LucciVaultPolicy, MilestoneClaim,
        MilestoneKind, SettlementKey,
    },
};
use std::{
    fs,
    os::unix::fs::PermissionsExt,
    path::PathBuf,
    sync::{
        Mutex,
        atomic::{AtomicU64, Ordering},
    },
    time::Duration,
};

static NEXT: AtomicU64 = AtomicU64::new(0);
static CREATE_ACCOUNT_GATE: Mutex<()> = Mutex::new(());

struct Temp(PathBuf);
impl Drop for Temp {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn temp() -> Temp {
    let path = std::env::temp_dir().join(format!(
        "kr-f4-worker-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&path).unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o700)).unwrap();
    Temp(path)
}

fn admin() -> Admin<'static> {
    Admin {
        actor: "f4-worker-test",
        reason: "extended facade",
    }
}

fn create_account(store: &mut Store, login: &str) -> kr_service::accounts::Player {
    let _gate = CREATE_ACCOUNT_GATE
        .lock()
        .unwrap_or_else(|poison| poison.into_inner());
    loop {
        match store.create_account(
            &admin(),
            login,
            login,
            b"extended-worker-password",
            kr_service::accounts::utc().unwrap(),
        ) {
            Ok(player) => return player,
            Err(kr_service::accounts::Error::Busy) => std::thread::yield_now(),
            Err(error) => panic!("create account: {error:?}"),
        }
    }
}

fn economy() -> EconomyPolicy {
    EconomyPolicy {
        enabled: true,
        win: RewardAmount {
            exp: 100,
            lucci: 100,
        },
        loss: RewardAmount::default(),
        draw: RewardAmount::default(),
        grant_on_disconnect_or_left: false,
        level_thresholds: vec![0, 100, 200],
    }
}

fn match_start(
    player: &kr_service::accounts::Player,
    other: &kr_service::accounts::Player,
    epoch: i64,
) -> MatchStart {
    MatchStart {
        key: MatchKey {
            boot_epoch: epoch,
            room_session: 1,
            room_id: 1,
            room_generation: 1,
            round: 1,
        },
        map_code: 1,
        map_name: "Factory07".into(),
        started_at: kr_service::accounts::utc().unwrap().saturating_sub(2),
        players: vec![
            MatchParticipant {
                account_id: player.account_id,
                player_id: player.player_id,
                slot: 0,
                character_key: 0,
                team: 0,
            },
            MatchParticipant {
                account_id: other.account_id,
                player_id: other.player_id,
                slot: 1,
                character_key: 0,
                team: 1,
            },
        ],
    }
}

fn finish() -> MatchFinish {
    MatchFinish {
        ended_at: kr_service::accounts::utc().unwrap().saturating_sub(1),
        reason: MatchEndReason::Elimination,
        players: vec![
            MatchPlayerFinish {
                slot: 0,
                outcome: MatchOutcome::Win,
                exit: MatchExit::Present,
            },
            MatchPlayerFinish {
                slot: 1,
                outcome: MatchOutcome::Loss,
                exit: MatchExit::Present,
            },
        ],
    }
}

fn booster_policy() -> BoosterTicketPolicy {
    BoosterTicketPolicy {
        enabled: true,
        exp_percent: 200,
        lucci_percent: 200,
        bonus_cap: Some(kr_service::f4_extended_policy::BoosterBonusCap {
            exp: 1_000,
            lucci: 1_000,
        }),
        allowed_rooms: vec![BoosterRoom {
            room_group: "standard".into(),
            mode: "normal".into(),
            maps: vec!["Factory07".into()],
        }],
    }
}

fn room() -> BoosterRoomContext<'static> {
    BoosterRoomContext {
        room_group: "standard",
        mode: "normal",
        map: "Factory07",
        room_rules_allow: true,
        normal_unranked: true,
    }
}

async fn prepared(
    vault: LucciVaultPolicy,
    level_rewards: LevelRewardPolicy,
) -> (
    Temp,
    Worker,
    AccountService,
    kr_service::accounts::Player,
    u64,
) {
    let temp = temp();
    let mut store = Store::open(&temp.0.join("accounts.sqlite")).unwrap();
    let player = create_account(&mut store, "extended");
    let other = create_account(&mut store, "other");
    let now = kr_service::accounts::utc().unwrap();
    let epoch = store.begin_instance(&admin(), now).unwrap();
    let start = match_start(&player, &other, epoch);
    let match_id = store
        .begin_match(&start, kr_service::accounts::utc().unwrap())
        .unwrap()
        .id;
    store
        .finish_match_settled(
            &start.key,
            &finish(),
            &economy(),
            kr_service::accounts::utc().unwrap(),
        )
        .unwrap();
    drop(store);
    let settings = EconomySettings::new(economy(), ShopCatalog::default())
        .unwrap()
        .with_extended_policies(booster_policy(), level_rewards, vault)
        .unwrap();
    let worker = Worker::open_with_economy(
        temp.0.join("accounts.sqlite"),
        4,
        Duration::from_secs(2),
        settings,
    )
    .await
    .unwrap();
    let service = worker.service();
    (temp, worker, service, player, match_id as u64)
}

#[tokio::test]
async fn default_settings_fail_closed_without_mutation() {
    let temp = temp();
    let mut store = Store::open(&temp.0.join("accounts.sqlite")).unwrap();
    let player = create_account(&mut store, "closed");
    drop(store);
    let worker = Worker::open(temp.0.join("accounts.sqlite"), 2, Duration::from_secs(2))
        .await
        .unwrap();
    let service = worker.service();
    let key = SettlementKey {
        match_id: 1,
        account: player.account_id as u64,
    };
    assert_eq!(
        service
            .apply_booster_settlement(
                player.player_id,
                key,
                room(),
                BoosterTicket {
                    account: key.account,
                    expires_at: u64::MAX,
                    free_server_grant: true,
                },
            )
            .await,
        Err(Error::InvalidInput)
    );
    let grant = LevelGrant {
        claim: MilestoneClaim {
            account: key.account,
            character: 0,
            level: 2,
            kind: MilestoneKind::Lucci,
        },
        reward: LevelReward::Lucci(1),
    };
    assert_eq!(
        service
            .claim_level_milestone_reward(player.player_id, &grant)
            .await,
        Err(Error::InvalidInput)
    );
    assert_eq!(
        service.profile_get(player.player_id).await.unwrap().lucci,
        0
    );
    assert!(
        service
            .ledger_list(player.player_id, 10)
            .await
            .unwrap()
            .is_empty()
    );
    worker.shutdown().await.unwrap();
}

#[tokio::test]
async fn booster_and_milestone_replays_are_idempotent() {
    let (temp, worker, service, player, match_id) = prepared(
        LucciVaultPolicy::default(),
        LevelRewardPolicy {
            enabled: true,
            character_milestones: None,
            level_up_lucci_table: Some(vec![LevelLucci {
                character: 0,
                level: 2,
                lucci: 25,
            }]),
        },
    )
    .await;
    let key = SettlementKey {
        match_id,
        account: player.account_id as u64,
    };
    let ticket = BoosterTicket {
        account: key.account,
        expires_at: u64::MAX,
        free_server_grant: true,
    };
    let first = service
        .apply_booster_settlement(player.player_id, key, room(), ticket)
        .await
        .unwrap();
    let replay = service
        .apply_booster_settlement(player.player_id, key, room(), ticket)
        .await
        .unwrap();
    assert!(matches!(first, Decision::Apply(_)));
    assert!(matches!(replay, Decision::Replay(_)));
    let grant = LevelGrant {
        claim: MilestoneClaim {
            account: key.account,
            character: 0,
            level: 2,
            kind: MilestoneKind::Lucci,
        },
        reward: LevelReward::Lucci(25),
    };
    let first = service
        .claim_level_milestone_reward(player.player_id, &grant)
        .await
        .unwrap();
    let replay = service
        .claim_level_milestone_reward(player.player_id, &grant)
        .await
        .unwrap();
    assert!(matches!(first, Decision::Apply(_)));
    assert!(matches!(replay, Decision::Replay(_)));
    assert_eq!(
        service
            .ledger_list(player.player_id, 10)
            .await
            .unwrap()
            .len(),
        3
    );
    worker.shutdown().await.unwrap();
    drop(temp);
}

#[tokio::test]
async fn vault_cap_rejects_extended_credit() {
    let vault = LucciVaultPolicy {
        enabled: true,
        base_lucci_cap: Some(100),
        expansion_step: Some(10),
        max_expansions: Some(2),
        max_lucci_cap: Some(120),
    };
    let (temp, worker, service, player, match_id) =
        prepared(vault, LevelRewardPolicy::default()).await;
    let key = SettlementKey {
        match_id,
        account: player.account_id as u64,
    };
    let result = service
        .apply_booster_settlement(
            player.player_id,
            key,
            room(),
            BoosterTicket {
                account: key.account,
                expires_at: u64::MAX,
                free_server_grant: true,
            },
        )
        .await;
    assert_eq!(
        result,
        Err(Error::Backend(kr_service::accounts::Error::Conflict))
    );
    assert_eq!(
        service.profile_get(player.player_id).await.unwrap().lucci,
        100
    );
    worker.shutdown().await.unwrap();
    drop(temp);
}

#[tokio::test]
async fn parallel_calls_remain_bounded_and_safe() {
    let (temp, worker, service, player, match_id) =
        prepared(LucciVaultPolicy::default(), LevelRewardPolicy::default()).await;
    let key = SettlementKey {
        match_id,
        account: player.account_id as u64,
    };
    let mut calls = Vec::new();
    for _ in 0..32 {
        let service = service.clone();
        let ticket = BoosterTicket {
            account: key.account,
            expires_at: u64::MAX,
            free_server_grant: true,
        };
        calls.push(tokio::spawn(async move {
            service
                .apply_booster_settlement(player.player_id, key, room(), ticket)
                .await
        }));
    }
    for call in calls {
        assert!(matches!(
            call.await.unwrap(),
            Ok(Decision::Apply(_)) | Ok(Decision::Replay(_)) | Err(Error::Busy)
        ));
    }
    worker.shutdown().await.unwrap();
    drop(temp);
}
