use kr_service::{
    accounts::{
        Admin, BoosterSettlementRequest, EconomyPolicy, Error, InventoryItem, ItemGrant,
        LedgerEntry, MatchEndReason, MatchExit, MatchFinish, MatchKey, MatchOutcome,
        MatchParticipant, MatchPlayerFinish, MatchStart, Player, PlayerProfile, RewardAmount,
        ShopCatalog, ShopCategory, ShopItem, Store,
    },
    f4_extended_policy::{
        BoosterBonusCap, BoosterRoom, BoosterRoomContext, BoosterTicket, BoosterTicketPolicy,
        Decision, ItemGrant as ChestGrant, LevelChest, LevelGrant, LevelLucci, LevelReward,
        LevelRewardPolicy, LucciVaultPolicy, MilestoneClaim, MilestoneKind, SettlementKey,
    },
};
use std::{
    fs,
    os::unix::fs::DirBuilderExt,
    path::PathBuf,
    sync::{Barrier, Mutex},
    thread,
    time::Duration,
};

static CREATE_ACCOUNT_GATE: Mutex<()> = Mutex::new(());

const MAP: &str = "ExVillage01";
const FINISHED: i64 = 102;
const AFTER: i64 = FINISHED + 1;

struct Fixture {
    dir: PathBuf,
    store: Store,
    a: Player,
    b: Player,
    epoch: i64,
}

impl Fixture {
    fn new() -> Self {
        let mut random = [0; 16];
        getrandom::fill(&mut random).unwrap();
        let nonce = u128::from_ne_bytes(random);
        let dir = std::env::temp_dir().join(format!("kr-f4-extended-{nonce:032x}"));
        fs::DirBuilder::new().mode(0o700).create(&dir).unwrap();
        let mut store = Store::open(&dir.join("accounts.sqlite")).unwrap();
        let admin = Admin {
            actor: "test",
            reason: "extended settlement fixture",
        };
        let epoch = store.begin_instance(&admin, 100).unwrap();
        let mut player = |name| {
            // `Store::create_account` caps process-wide concurrent Argon2 hashes at 2 (`HashPermit`),
            // so serialize test fixture creation here to avoid starvation across parallel tests.
            let _gate = CREATE_ACCOUNT_GATE
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            for _ in 0..200 {
                match store.create_account(&admin, name, name, b"test-only-password", 100) {
                    Ok(p) => return p,
                    Err(Error::Busy) => thread::sleep(Duration::from_millis(25)),
                    Err(err) => panic!("create_account({name}) failed unexpectedly: {err:?}"),
                }
            }
            panic!("create_account({name}) stayed Busy after retries");
        };
        let (a, b) = (player("alice"), player("bravo"));
        Self {
            dir,
            store,
            a,
            b,
            epoch,
        }
    }

    fn reopen(&self) -> Store {
        Store::open(&self.dir.join("accounts.sqlite")).unwrap()
    }

    fn play(
        &mut self,
        round: i64,
        at: i64,
        policy: &EconomyPolicy,
    ) -> (i64, MatchKey, MatchFinish) {
        let key = MatchKey {
            boot_epoch: self.epoch,
            room_session: 1,
            room_id: 1,
            room_generation: 1,
            round,
        };
        let participants = [(0, &self.a), (1, &self.b)];
        let start = MatchStart {
            key: key.clone(),
            map_code: 278,
            map_name: MAP.into(),
            started_at: at,
            players: participants
                .into_iter()
                .map(|(slot, p)| MatchParticipant {
                    account_id: p.account_id,
                    player_id: p.player_id,
                    slot,
                    character_key: 1,
                    team: slot,
                })
                .collect(),
        };
        self.store.begin_match(&start, at).unwrap();
        let finish = MatchFinish {
            ended_at: at,
            reason: MatchEndReason::AllDead,
            players: participants
                .into_iter()
                .map(|(slot, _)| MatchPlayerFinish {
                    slot,
                    outcome: MatchOutcome::Draw,
                    exit: MatchExit::Present,
                })
                .collect(),
        };
        let (record, _) = self
            .store
            .finish_match_settled(&key, &finish, policy, at)
            .unwrap();
        (record.id, key, finish)
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.dir).unwrap();
    }
}

fn economy() -> EconomyPolicy {
    EconomyPolicy {
        enabled: true,
        draw: RewardAmount { exp: 40, lucci: 30 },
        level_thresholds: vec![0, 40, 60, 80],
        ..EconomyPolicy::default()
    }
}

fn boosters() -> BoosterTicketPolicy {
    BoosterTicketPolicy {
        enabled: true,
        exp_percent: 150,
        lucci_percent: 200,
        bonus_cap: Some(BoosterBonusCap {
            exp: 100,
            lucci: 100,
        }),
        allowed_rooms: vec![BoosterRoom {
            room_group: "casual".into(),
            mode: "normal".into(),
            maps: vec![MAP.into(), "ExVillage02".into()],
        }],
    }
}

fn booster_request(match_id: i64, player: &Player) -> BoosterSettlementRequest<'static> {
    let account = player.account_id as u64;
    BoosterSettlementRequest {
        key: SettlementKey {
            match_id: match_id as u64,
            account,
        },
        room: BoosterRoomContext {
            room_group: "casual",
            mode: "normal",
            map: MAP,
            room_rules_allow: true,
            normal_unranked: true,
        },
        ticket: BoosterTicket {
            account,
            free_server_grant: true,
            expires_at: AFTER as u64,
        },
    }
}

fn vault(cap: i64) -> LucciVaultPolicy {
    LucciVaultPolicy {
        enabled: true,
        base_lucci_cap: Some(cap),
        expansion_step: Some(20),
        max_expansions: Some(2),
        max_lucci_cap: Some(cap + 40),
    }
}

fn level_rewards() -> LevelRewardPolicy {
    LevelRewardPolicy {
        enabled: true,
        level_up_lucci_table: Some(vec![
            LevelLucci {
                character: 1,
                level: 2,
                lucci: 10,
            },
            LevelLucci {
                character: 2,
                level: 2,
                lucci: 12,
            },
        ]),
        character_milestones: Some(
            [2, 3]
                .into_iter()
                .map(|level| LevelChest {
                    character: 1,
                    level,
                    chest: chest(),
                })
                .collect(),
        ),
    }
}

fn chest() -> ChestGrant {
    ChestGrant {
        sku: "chest.level".into(),
        quantity: 2,
    }
}

fn milestone(player: &Player, level: u16, reward: LevelReward) -> LevelGrant {
    LevelGrant {
        claim: MilestoneClaim {
            account: player.account_id as u64,
            character: 1,
            level,
            kind: match &reward {
                LevelReward::Lucci(_) => MilestoneKind::Lucci,
                LevelReward::Chest(_) => MilestoneKind::Chest,
            },
        },
        reward,
    }
}

fn catalog(max_stack: u32) -> ShopCatalog {
    ShopCatalog {
        items: vec![ShopItem {
            sku_id: chest().sku,
            category: ShopCategory::Consumable,
            price_lucci: 0,
            grant: ItemGrant::Stackable { max_stack },
            equip_slot: None,
            enabled: false,
        }],
    }
}

fn state(
    store: &Store,
    player: i64,
    now: i64,
) -> (PlayerProfile, Vec<LedgerEntry>, Vec<InventoryItem>) {
    (
        store.get_profile(player).unwrap(),
        store.list_ledger(player, 100).unwrap(),
        store.list_inventory(player, now).unwrap(),
    )
}

fn applied(receipt: Decision<LedgerEntry>) -> LedgerEntry {
    match receipt {
        Decision::Apply(entry) => entry,
        Decision::Replay(_) => panic!("expected a new ledger entry"),
    }
}

#[test]
fn disabled_policies_reject_without_changing_profiles_ledgers_or_inventory() {
    let mut f = Fixture::new();
    let progression = economy();
    let (id, _, _) = f.play(1, FINISHED, &progression);
    let player = f.a.player_id;
    let before = state(&f.store, player, AFTER);
    let grant = milestone(&f.a, 2, LevelReward::Chest(chest()));
    let vault = LucciVaultPolicy::default();
    assert_eq!(
        f.store.apply_booster_settlement(
            player,
            booster_request(id, &f.a),
            &BoosterTicketPolicy::default(),
            &vault,
            &progression,
            AFTER,
        ),
        Err(Error::InvalidInput)
    );
    assert_eq!(
        f.store.claim_level_milestone_reward(
            player,
            &grant,
            &LevelRewardPolicy::default(),
            &vault,
            &catalog(4),
            AFTER,
        ),
        Err(Error::InvalidInput)
    );
    assert_eq!(state(&f.store, player, AFTER), before);
}

#[test]
fn booster_concurrent_retry_credits_only_the_bonus_and_preserves_base_settlement() {
    let mut f = Fixture::new();
    let progression = economy();
    let (id, key, finish) = f.play(1, FINISHED, &progression);
    let player = f.a.player_id;
    let request = booster_request(id, &f.a);
    let barrier = Barrier::new(2);
    let [one, two] = thread::scope(|scope| {
        let launch = |store: Store| {
            let (barrier, progression) = (&barrier, &progression);
            scope.spawn(move || {
                barrier.wait();
                store
                    .apply_booster_settlement(
                        player,
                        request,
                        &boosters(),
                        &LucciVaultPolicy::default(),
                        progression,
                        AFTER,
                    )
                    .unwrap()
            })
        };
        let one = launch(f.reopen());
        let two = launch(f.reopen());
        [one.join().unwrap(), two.join().unwrap()]
    });
    let receipt = match (one, two) {
        (Decision::Apply(new), Decision::Replay(old))
        | (Decision::Replay(old), Decision::Apply(new)) => {
            assert_eq!(old, new);
            new
        }
        other => panic!("expected one application and one replay: {other:?}"),
    };
    assert_eq!((receipt.exp_delta, receipt.lucci_delta), (20, 30));
    let (profile, ledger, _) = state(&f.reopen(), player, AFTER);
    assert_eq!(
        (profile.exp, profile.level, profile.lucci, profile.version),
        (60, 3, 60, 3)
    );
    assert_eq!(ledger.len(), 2);
    assert_eq!(ledger[0], receipt);
    assert_eq!(ledger[1].match_id, Some(id));
    assert!(
        f.store
            .finish_match_settled(&key, &finish, &progression, AFTER)
            .unwrap()
            .1
            .replayed
    );
    assert_eq!(f.store.get_profile(player).unwrap(), profile);

    let changed = BoosterSettlementRequest {
        ticket: BoosterTicket {
            expires_at: request.ticket.expires_at + 1,
            ..request.ticket
        },
        ..request
    };
    assert_eq!(
        f.store.apply_booster_settlement(
            player,
            changed,
            &boosters(),
            &LucciVaultPolicy::default(),
            &progression,
            AFTER,
        ),
        Err(Error::Conflict)
    );
    assert_eq!(f.store.list_ledger(player, 10).unwrap(), ledger);
}

#[test]
fn booster_requires_the_match_account_room_ticket_and_base_receipt() {
    let mut f = Fixture::new();
    let progression = economy();
    let (id, _, _) = f.play(1, FINISHED, &progression);
    let player = f.a.player_id;
    let request = booster_request(id, &f.a);
    let before = state(&f.store, player, AFTER);
    let other_before = state(&f.store, f.b.player_id, AFTER);
    let vault = LucciVaultPolicy::default();
    let apply = |player, request| {
        f.store
            .apply_booster_settlement(player, request, &boosters(), &vault, &progression, AFTER)
    };
    assert_eq!(apply(f.b.player_id, request), Err(Error::InvalidInput));
    for room in [
        BoosterRoomContext {
            room_rules_allow: false,
            ..request.room
        },
        BoosterRoomContext {
            normal_unranked: false,
            ..request.room
        },
        BoosterRoomContext {
            map: "ExVillage02",
            ..request.room
        },
    ] {
        assert_eq!(
            apply(player, BoosterSettlementRequest { room, ..request }),
            Err(Error::InvalidInput)
        );
    }
    for ticket in [
        BoosterTicket {
            expires_at: FINISHED as u64,
            ..request.ticket
        },
        BoosterTicket {
            free_server_grant: false,
            ..request.ticket
        },
    ] {
        assert_eq!(
            apply(player, BoosterSettlementRequest { ticket, ..request }),
            Err(Error::InvalidInput)
        );
    }
    assert_eq!(state(&f.store, f.b.player_id, AFTER), other_before);
    let (unsettled, _, _) = f.play(2, AFTER, &EconomyPolicy::default());
    let unset = BoosterSettlementRequest {
        key: SettlementKey {
            match_id: unsettled as u64,
            ..request.key
        },
        ticket: BoosterTicket {
            expires_at: AFTER as u64 + 1,
            ..request.ticket
        },
        ..request
    };
    assert_eq!(
        f.store
            .apply_booster_settlement(player, unset, &boosters(), &vault, &progression, AFTER),
        Err(Error::Conflict)
    );
    assert_eq!(state(&f.store, player, AFTER), before);
}

#[test]
fn vault_rejects_whole_credit_above_capacity_and_allows_exact_cap() {
    let mut f = Fixture::new();
    let progression = economy();
    let (id, _, _) = f.play(1, FINISHED, &progression);
    let player = f.a.player_id;
    let request = booster_request(id, &f.a);
    let grant = milestone(&f.a, 2, LevelReward::Lucci(10));
    let before = state(&f.store, player, AFTER);
    assert_eq!(
        f.store.apply_booster_settlement(
            player,
            request,
            &boosters(),
            &vault(59),
            &progression,
            AFTER
        ),
        Err(Error::Conflict)
    );
    assert_eq!(
        f.store.claim_level_milestone_reward(
            player,
            &grant,
            &level_rewards(),
            &vault(39),
            &catalog(4),
            AFTER
        ),
        Err(Error::Conflict)
    );
    assert_eq!(state(&f.store, player, AFTER), before);

    let bonus = applied(
        f.store
            .apply_booster_settlement(
                player,
                request,
                &boosters(),
                &vault(60),
                &progression,
                AFTER,
            )
            .unwrap(),
    );
    assert_eq!(f.store.get_profile(player).unwrap().lucci, 60);
    assert_eq!(
        f.store.apply_booster_settlement(
            player,
            request,
            &boosters(),
            &vault(60),
            &progression,
            AFTER
        ),
        Ok(Decision::Replay(bonus))
    );
    let reward = applied(
        f.store
            .claim_level_milestone_reward(
                player,
                &grant,
                &level_rewards(),
                &vault(70),
                &catalog(4),
                AFTER,
            )
            .unwrap(),
    );
    assert_eq!(f.store.get_profile(player).unwrap().lucci, 70);
    assert_eq!(
        f.store.claim_level_milestone_reward(
            player,
            &grant,
            &level_rewards(),
            &vault(70),
            &catalog(4),
            AFTER
        ),
        Ok(Decision::Replay(reward))
    );
    assert_eq!(f.store.get_profile(player).unwrap().lucci, 70);
}

#[test]
fn milestone_validates_grants_and_persists_one_claim_per_player_and_level() {
    let mut f = Fixture::new();
    f.play(1, FINISHED, &economy());
    let (a, b) = (f.a.player_id, f.b.player_id);
    let lucci = milestone(&f.a, 2, LevelReward::Lucci(10));
    let wrong_amount = LevelGrant {
        reward: LevelReward::Lucci(11),
        ..lucci.clone()
    };
    let other_character = LevelGrant {
        claim: MilestoneClaim {
            character: 2,
            ..lucci.claim
        },
        reward: LevelReward::Lucci(12),
    };
    let alternate = milestone(&f.a, 2, LevelReward::Chest(chest()));
    let unreached = milestone(&f.a, 3, LevelReward::Chest(chest()));
    let item = milestone(&f.b, 2, LevelReward::Chest(chest()));
    let levels = level_rewards();
    let vault = LucciVaultPolicy::default();
    let full_catalog = catalog(4);
    let claim = |store: &Store, player, grant: &LevelGrant, at| {
        store.claim_level_milestone_reward(player, grant, &levels, &vault, &full_catalog, at)
    };
    let before = state(&f.store, a, AFTER);
    assert_eq!(
        claim(&f.store, a, &wrong_amount, AFTER),
        Err(Error::InvalidInput)
    );
    assert_eq!(
        claim(&f.store, a, &unreached, AFTER),
        Err(Error::InvalidInput)
    );
    assert_eq!(claim(&f.store, a, &item, AFTER), Err(Error::InvalidInput));
    assert_eq!(state(&f.store, a, AFTER), before);

    let receipt = applied(claim(&f.store, a, &lucci, AFTER).unwrap());
    assert_eq!(receipt.lucci_delta, 10);
    assert_eq!(f.store.get_profile(a).unwrap().lucci, 40);
    assert_eq!(claim(&f.store, a, &alternate, AFTER), Err(Error::Conflict));
    assert_eq!(
        claim(&f.store, a, &other_character, AFTER),
        Err(Error::Conflict)
    );

    let before_item = state(&f.store, b, AFTER);
    assert_eq!(
        f.store
            .claim_level_milestone_reward(b, &item, &levels, &vault, &catalog(1), AFTER),
        Err(Error::Conflict)
    );
    assert_eq!(state(&f.store, b, AFTER), before_item);
    let item_receipt = applied(claim(&f.store, b, &item, AFTER).unwrap());
    assert_eq!(item_receipt.sku_id.as_deref(), Some("chest.level"));
    let (a_after, b_after) = (state(&f.store, a, AFTER), state(&f.store, b, AFTER));
    let reopened = f.reopen();
    assert_eq!(
        claim(&reopened, a, &lucci, AFTER),
        Ok(Decision::Replay(receipt))
    );
    assert_eq!(
        claim(&reopened, b, &item, AFTER),
        Ok(Decision::Replay(item_receipt))
    );
    assert_eq!(state(&reopened, a, AFTER), a_after);
    assert_eq!(state(&reopened, b, AFTER), b_after);
    assert_eq!(b_after.2.len(), 1);
    assert_eq!(b_after.2[0].quantity, 2);

    f.play(2, AFTER + 1, &economy());
    applied(claim(&f.store, a, &unreached, AFTER + 1).unwrap());
    assert_eq!(f.store.list_inventory(a, AFTER + 1).unwrap()[0].quantity, 2);
}
