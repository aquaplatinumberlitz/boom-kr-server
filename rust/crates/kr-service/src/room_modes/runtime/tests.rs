use super::*;
use crate::room_modes::Config;
use serde_json::json;

const DUEL: [Player; 2] = [Player { slot: 3, team: 1 }, Player { slot: 7, team: 2 }];
const TEAMS: [Player; 4] = [
    Player { slot: 12, team: 2 },
    Player { slot: 7, team: 2 },
    Player { slot: 3, team: 1 },
    Player { slot: 2, team: 1 },
];

fn policy(config: serde_json::Value) -> Option<Policy> {
    Policy::load(&serde_json::from_value::<Config>(config).unwrap()).unwrap()
}

fn runtime(mode: &str) -> Runtime {
    policy(json!({"runtime_overrides": {"free": mode}}))
        .unwrap()
        .runtime_for(RoomTeamSelector::Free)
        .unwrap()
        .unwrap()
}

fn event(runtime: &mut Runtime, round: u64, seq: u64, action: Action<'_>) {
    assert_eq!(runtime.apply(round, seq, action), Ok(Applied::Applied));
}

fn revive_setup(team_budget: Option<u16>, player_budget: Option<u16>, score_limit: u32) -> Setup {
    Setup::Revive {
        team_budget,
        player_budget,
        duration_ms: 50,
        score_limit,
    }
}

#[test]
fn runtime_requires_its_own_explicit_override_and_keeps_native_gate_closed() {
    assert!(policy(json!({})).is_none());
    let visible = policy(json!({"visible_map_groups": ["ReBirthMapGroup"]})).unwrap();
    assert!(
        visible
            .runtime_for(RoomTeamSelector::Free)
            .unwrap()
            .is_none()
    );

    let timebomb = policy(json!({"runtime_overrides": {"free": "timebomb"}})).unwrap();
    assert!(
        timebomb
            .runtime_for(RoomTeamSelector::Manner)
            .unwrap()
            .is_none()
    );
    assert_eq!(
        timebomb
            .runtime_for(RoomTeamSelector::Free)
            .unwrap()
            .unwrap()
            .outcome(),
        None
    );
    let map = kr_wire::kr00967::Start121Map {
        map_code: 15,
        name: b"Factory07".to_vec(),
        dynamic_index: 0,
        roomtype: 0,
    };
    assert_eq!(
        timebomb.check_start(RoomTeamSelector::Free, &map, (15, 0)),
        Ok(())
    );
    let native = policy(json!({"create_overrides": {"free": "timebomb"}})).unwrap();
    assert!(
        native
            .check_start(RoomTeamSelector::Free, &map, (15, 0))
            .is_err()
    );
    assert!(
        native
            .runtime_for(RoomTeamSelector::Free)
            .unwrap()
            .is_none()
    );
    for unsupported in ["ladder", "big_battle", "confused_fight", "matchmaking"] {
        let p = policy(json!({"runtime_overrides": {"free": unsupported}})).unwrap();
        assert!(matches!(
            p.runtime_for(RoomTeamSelector::Free),
            Err(Error::UnsupportedMode)
        ));
    }
}

#[test]
fn revive_team_and_player_budgets_are_independent_and_spent_only_on_respawn() {
    let mut r = runtime("revive");
    r.start_round(1, &TEAMS, revive_setup(Some(2), Some(1), 10))
        .unwrap();

    assert_eq!(
        r.apply(1, 1, Action::Respawn(2)),
        Err(Error::InvalidTransition)
    );
    event(&mut r, 1, 1, Action::Defeat(&[2]));
    assert_eq!(r.life(2), Some(Life::Ghost));
    assert_eq!(r.score(2), Some(1));
    event(&mut r, 1, 2, Action::Respawn(2));
    assert_eq!(r.life(2), Some(Life::Alive));
    assert_eq!(r.apply(1, 2, Action::Respawn(2)), Ok(Applied::Replay));

    event(&mut r, 1, 3, Action::Defeat(&[2, 3, 7]));
    assert_eq!(r.apply(1, 4, Action::Respawn(2)), Err(Error::NoRevives));
    // A failed check of the personal limit did not consume the team's use.
    event(&mut r, 1, 4, Action::Respawn(3));
    // The opposing team's budget is separate.
    event(&mut r, 1, 5, Action::Respawn(7));
    event(&mut r, 1, 6, Action::Defeat(&[3]));
    assert_eq!(r.apply(1, 7, Action::Respawn(3)), Err(Error::NoRevives));
    assert_eq!(r.life(3), Some(Life::Ghost));

    r.start_round(2, &TEAMS, revive_setup(Some(1), None, 10))
        .unwrap();
    assert_eq!(r.life(2), Some(Life::Alive));
    assert_eq!(r.score(2), Some(0));
    event(&mut r, 2, 1, Action::Defeat(&[2, 3]));
    event(&mut r, 2, 2, Action::Respawn(2));
    assert_eq!(r.apply(2, 3, Action::Respawn(3)), Err(Error::NoRevives));
}

#[test]
fn revive_can_finish_by_score_or_clock_and_leaves_ties_unresolved() {
    let mut r = runtime("revive");
    r.start_round(1, &DUEL, revive_setup(None, Some(1), 2))
        .unwrap();
    event(&mut r, 1, 1, Action::Defeat(&[3]));
    event(&mut r, 1, 2, Action::Respawn(3));
    event(&mut r, 1, 3, Action::Defeat(&[3]));
    assert_eq!(r.outcome(), Some(Outcome::Winner(2)));
    assert_eq!(r.apply(1, 4, Action::Respawn(3)), Err(Error::Finished));

    r.start_round(2, &DUEL, revive_setup(None, Some(0), 2))
        .unwrap();
    event(&mut r, 2, 1, Action::Defeat(&[7]));
    event(&mut r, 2, 2, Action::AdvanceTo(49));
    assert_eq!(r.outcome(), Some(Outcome::Playing));
    event(&mut r, 2, 3, Action::AdvanceTo(50));
    assert_eq!(r.outcome(), Some(Outcome::Winner(1)));

    r.start_round(3, &DUEL, revive_setup(Some(1), None, 1))
        .unwrap();
    event(&mut r, 3, 1, Action::Defeat(&[7, 3]));
    assert_eq!(r.outcome(), Some(Outcome::Unresolved));

    r.start_round(4, &DUEL, revive_setup(Some(1), None, 1))
        .unwrap();
    event(&mut r, 4, 1, Action::AdvanceTo(51));
    assert_eq!(r.outcome(), Some(Outcome::Unresolved));
}

#[test]
fn invalid_batches_and_sequences_do_not_mutate_rounds() {
    let mut r = runtime("revive");
    let setup = || revive_setup(Some(1), None, 10);
    assert_eq!(r.apply(1, 1, Action::Defeat(&[3])), Err(Error::WrongRound));
    assert_eq!(r.start_round(0, &DUEL, setup()), Err(Error::WrongRound));
    assert_eq!(
        r.start_round(1, &DUEL[..1], setup()),
        Err(Error::InvalidRoster)
    );
    assert_eq!(
        r.start_round(1, &[DUEL[0], DUEL[0], DUEL[1]], setup()),
        Err(Error::InvalidRoster)
    );
    assert_eq!(
        r.start_round(1, &DUEL, revive_setup(None, None, 10)),
        Err(Error::InvalidSetup)
    );
    r.start_round(1, &DUEL, setup()).unwrap();
    assert_eq!(r.apply(2, 1, Action::Defeat(&[3])), Err(Error::WrongRound));
    assert_eq!(r.apply(1, 0, Action::Defeat(&[3])), Err(Error::SequenceGap));
    assert_eq!(r.apply(1, 2, Action::Defeat(&[3])), Err(Error::SequenceGap));
    assert_eq!(
        r.apply(1, 1, Action::Defeat(&[3, 0])),
        Err(Error::InvalidPlayer)
    );
    assert_eq!(
        r.apply(1, 1, Action::Defeat(&[3, 3])),
        Err(Error::InvalidTransition)
    );
    assert_eq!(r.life(3), Some(Life::Alive));
    assert_eq!(r.score(2), Some(0));
    event(&mut r, 1, 1, Action::Defeat(&[3]));
    assert_eq!(r.apply(1, 1, Action::Defeat(&[7])), Ok(Applied::Replay));
    assert_eq!(r.life(7), Some(Life::Alive));
    assert_eq!(r.start_round(1, &DUEL, setup()), Err(Error::WrongRound));
    assert_eq!(
        r.start_round(2, &DUEL, Setup::Evaluate),
        Err(Error::InvalidSetup)
    );
    assert_eq!(r.life(3), Some(Life::Ghost));

    r.start_round(2, &DUEL, setup()).unwrap();
    assert_eq!(r.apply(1, 99, Action::Defeat(&[3])), Ok(Applied::Replay));
    assert_eq!(r.life(3), Some(Life::Alive));
    event(&mut r, 2, 1, Action::Defeat(&[3]));
}

fn bomb_setup(initial_carrier: Slot) -> Setup {
    Setup::Timebomb {
        initial_carrier,
        fuse_ms: 100,
    }
}

#[test]
fn timebomb_transfers_only_from_carrier_and_preserves_fuse() {
    let mut r = runtime("timebomb");
    assert_eq!(
        r.start_round(1, &DUEL, bomb_setup(8)),
        Err(Error::InvalidSetup)
    );
    assert_eq!(
        r.start_round(
            1,
            &DUEL,
            Setup::Timebomb {
                initial_carrier: 3,
                fuse_ms: 0
            }
        ),
        Err(Error::InvalidSetup)
    );
    r.start_round(1, &DUEL, bomb_setup(3)).unwrap();
    event(&mut r, 1, 1, Action::AdvanceTo(40));
    assert_eq!(
        r.apply(1, 2, Action::AdvanceTo(39)),
        Err(Error::ClockReversed)
    );
    assert_eq!(
        r.apply(1, 2, Action::PassBomb { from: 3, to: 3 }),
        Err(Error::InvalidTransition)
    );
    assert_eq!(
        r.apply(1, 2, Action::PassBomb { from: 3, to: 1 }),
        Err(Error::InvalidPlayer)
    );
    assert_eq!(
        r.apply(1, 2, Action::PassBomb { from: 7, to: 3 }),
        Err(Error::InvalidTransition)
    );
    event(&mut r, 1, 2, Action::PassBomb { from: 3, to: 7 });
    assert_eq!(
        r.apply(1, 2, Action::PassBomb { from: 7, to: 3 }),
        Ok(Applied::Replay)
    );
    event(&mut r, 1, 3, Action::AdvanceTo(40));
    assert_eq!(r.bomb(), Some((7, 60)));
    event(&mut r, 1, 4, Action::AdvanceTo(99));
    assert_eq!(r.bomb(), Some((7, 1)));
    event(&mut r, 1, 5, Action::AdvanceTo(100));
    assert_eq!(r.life(7), Some(Life::Eliminated));
    assert_eq!(r.outcome(), Some(Outcome::Winner(1)));
    assert_eq!(r.bomb(), None);

    r.start_round(2, &DUEL, bomb_setup(7)).unwrap();
    assert_eq!(r.life(7), Some(Life::Alive));
    assert_eq!(r.bomb(), Some((7, 100)));
    assert_eq!(
        r.apply(1, 5, Action::AdvanceTo(u64::MAX)),
        Ok(Applied::Replay)
    );
    assert_eq!(r.outcome(), Some(Outcome::Playing));
}

#[test]
fn timebomb_large_clock_jump_matches_smaller_ticks() {
    let mut stepwise = runtime("timebomb");
    let mut jump = runtime("timebomb");
    for r in [&mut stepwise, &mut jump] {
        r.start_round(1, &TEAMS, bomb_setup(12)).unwrap();
    }
    event(&mut stepwise, 1, 1, Action::AdvanceTo(100));
    assert_eq!(stepwise.bomb(), Some((2, 100)));
    assert_eq!(
        stepwise.apply(1, 2, Action::PassBomb { from: 2, to: 12 }),
        Err(Error::InvalidTransition)
    );
    event(&mut stepwise, 1, 2, Action::AdvanceTo(200));
    event(&mut stepwise, 1, 3, Action::AdvanceTo(250));
    event(&mut jump, 1, 1, Action::AdvanceTo(250));
    for player in TEAMS {
        assert_eq!(stepwise.life(player.slot), jump.life(player.slot));
    }
    assert_eq!(jump.bomb(), Some((3, 50)));
    assert_eq!(stepwise.bomb(), jump.bomb());
    assert_eq!(jump.outcome(), Some(Outcome::Playing));
    event(&mut jump, 1, 2, Action::AdvanceTo(u64::MAX));
    assert_eq!(jump.outcome(), Some(Outcome::Winner(2)));
}

#[test]
fn leader_capture_requires_one_leader_per_team_then_resets_assignments() {
    let mut r = runtime("leader_capture");
    let setup = |a, b| Setup::LeaderCapture {
        leaders: vec![a, b],
    };
    assert_eq!(
        r.start_round(1, &TEAMS, setup(2, 3)),
        Err(Error::InvalidSetup)
    );
    assert_eq!(
        r.start_round(1, &TEAMS, setup(2, 2)),
        Err(Error::InvalidSetup)
    );
    assert_eq!(
        r.start_round(1, &TEAMS, setup(3, 0)),
        Err(Error::InvalidPlayer)
    );
    r.start_round(1, &TEAMS, setup(3, 7)).unwrap();
    event(&mut r, 1, 1, Action::Defeat(&[2]));
    assert_eq!(r.outcome(), Some(Outcome::Playing));
    event(&mut r, 1, 2, Action::Defeat(&[7]));
    assert_eq!(r.outcome(), Some(Outcome::Winner(1)));
    assert_eq!(r.apply(1, 2, Action::Defeat(&[7])), Ok(Applied::Replay));
    assert_eq!(r.apply(1, 3, Action::Defeat(&[3])), Err(Error::Finished));

    r.start_round(2, &TEAMS, setup(2, 12)).unwrap();
    assert_eq!(r.life(2), Some(Life::Alive));
    event(&mut r, 2, 1, Action::Defeat(&[7, 3]));
    assert_eq!(r.outcome(), Some(Outcome::Playing));
    event(&mut r, 2, 2, Action::Defeat(&[12, 2]));
    assert_eq!(r.outcome(), Some(Outcome::Unresolved));
}

#[test]
fn free_1v7_only_resolves_a_unique_surviving_side() {
    let mut r = runtime("free_1v7");
    let mut players: Vec<Player> = (3..11)
        .map(|slot| Player {
            slot,
            team: if slot == 3 { 9 } else { 4 },
        })
        .collect();
    players[1].team = 9;
    assert_eq!(
        r.start_round(1, &players, Setup::Evaluate),
        Err(Error::InvalidRoster)
    );
    players[1].team = 4;
    assert_eq!(
        r.start_round(1, &players[..7], Setup::Evaluate),
        Err(Error::InvalidRoster)
    );
    r.start_round(1, &players, Setup::Evaluate).unwrap();
    event(&mut r, 1, 1, Action::Defeat(&[4, 5, 6, 7, 8, 9]));
    assert_eq!(r.outcome(), Some(Outcome::Playing));
    event(&mut r, 1, 2, Action::Defeat(&[10]));
    assert_eq!(r.outcome(), Some(Outcome::Winner(9)));

    r.start_round(2, &players, Setup::Evaluate).unwrap();
    event(&mut r, 2, 1, Action::Defeat(&[3]));
    assert_eq!(r.outcome(), Some(Outcome::Winner(4)));

    r.start_round(3, &players, Setup::Evaluate).unwrap();
    event(&mut r, 3, 1, Action::Defeat(&[3, 4, 5, 6, 7, 8, 9, 10]));
    assert_eq!(r.outcome(), Some(Outcome::Unresolved));
}

#[test]
fn coop_and_hedgehog_do_not_inherit_normal_elimination_or_draw() {
    let mut coop = runtime("co_op");
    coop.start_round(1, &DUEL, Setup::Evaluate).unwrap();
    assert_eq!(
        coop.apply(1, 1, Action::Defeat(&[3])),
        Err(Error::UnsupportedAction)
    );
    assert_eq!(
        coop.apply(1, 1, Action::Objective(Some(8))),
        Err(Error::UnknownTeam)
    );
    event(&mut coop, 1, 1, Action::Objective(Some(2)));
    assert_eq!(coop.outcome(), Some(Outcome::Winner(2)));
    coop.start_round(2, &DUEL, Setup::Evaluate).unwrap();
    event(&mut coop, 2, 1, Action::Objective(None));
    assert_eq!(coop.outcome(), Some(Outcome::Unresolved));

    let mut hedgehog = runtime("hedgehog");
    hedgehog.start_round(1, &DUEL, Setup::Evaluate).unwrap();
    assert_eq!(
        hedgehog.apply(1, 1, Action::Defeat(&[3])),
        Err(Error::UnsupportedAction)
    );
    assert_eq!(
        hedgehog.apply(1, 1, Action::Objective(Some(2))),
        Err(Error::UnsupportedAction)
    );
    assert_eq!(hedgehog.outcome(), Some(Outcome::Playing));
    event(&mut hedgehog, 1, 1, Action::Objective(None));
    assert_eq!(hedgehog.outcome(), Some(Outcome::Unresolved));
}
