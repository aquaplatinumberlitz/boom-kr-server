//! Company-owned durable history. No KR authority or reward formula is inferred.
use super::*;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MatchKey {
    pub boot_epoch: i64,
    pub room_session: u16,
    pub room_id: u32,
    pub room_generation: i64,
    pub round: i64,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MatchParticipant {
    pub account_id: i64,
    pub player_id: i64,
    pub slot: u8,
    pub character_key: u16,
    pub team: u8,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MatchStart {
    pub key: MatchKey,
    pub map_code: u16,
    pub map_name: String,
    pub started_at: i64,
    pub players: Vec<MatchParticipant>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MatchOutcome {
    Win,
    Loss,
    Draw,
    Cancelled,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MatchExit {
    Present,
    Left,
    Disconnected,
    Unknown,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MatchEndReason {
    Elimination,
    AllDead,
    Forfeit,
    LoadingAborted,
    NoPlayers,
    ServerShutdown,
    ServerRestart,
}
impl MatchEndReason {
    fn cancelled(self) -> bool {
        !matches!(self, Self::Elimination | Self::AllDead | Self::Forfeit)
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MatchPlayerFinish {
    pub slot: u8,
    pub outcome: MatchOutcome,
    pub exit: MatchExit,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MatchFinish {
    pub ended_at: i64,
    pub reason: MatchEndReason,
    pub players: Vec<MatchPlayerFinish>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct MatchRecord {
    pub id: i64,
    pub start: MatchStart,
    pub finish: Option<MatchFinish>,
    /// Only zero rewards are supported. There is no balance or grant API here.
    pub reward_per_player: u8,
}
pub enum HistoryFilter {
    Account(i64),
    Room(u32),
}
fn json<T: Serialize>(value: &T) -> Result<String> {
    serde_json::to_string(value).map_err(|_| Error::InvalidInput)
}
fn label<T: Serialize>(value: &T) -> Result<String> {
    serde_json::to_value(value)
        .map_err(|_| Error::InvalidInput)?
        .as_str()
        .map(str::to_owned)
        .ok_or(Error::InvalidInput)
}
impl MatchStart {
    fn normalize(&self, now: i64) -> Result<Self> {
        let mut value = self.clone();
        value.players.sort_by_key(|p| p.slot);
        let key = &value.key;
        if key.boot_epoch <= 0
            || key.room_session == 0
            || key.room_id == 0
            || key.room_generation <= 0
            || key.round <= 0
            || value.started_at < 0
            || value.started_at > now
            || !(1..=64).contains(&value.map_name.len())
            || !value
                .map_name
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'_')
            || !(2..=8).contains(&value.players.len())
        {
            return Err(Error::InvalidInput);
        }
        let (mut slots, mut accounts, mut players) =
            (BTreeSet::new(), BTreeSet::new(), BTreeSet::new());
        for p in &value.players {
            if p.account_id <= 0
                || p.player_id <= 0
                || p.slot > 7
                || p.character_key > 8191
                || p.team > 31
                || !slots.insert(p.slot)
                || !accounts.insert(p.account_id)
                || !players.insert(p.player_id)
            {
                return Err(Error::InvalidInput);
            }
        }
        Ok(value)
    }
}
impl MatchFinish {
    fn normalize(&self, start: &MatchStart, now: i64, recovery: bool) -> Result<Self> {
        let mut value = self.clone();
        value.players.sort_by_key(|p| p.slot);
        if value.ended_at < start.started_at
            || value.ended_at > now
            || value.players.len() != start.players.len()
            || (!recovery && value.reason == MatchEndReason::ServerRestart)
        {
            return Err(Error::InvalidInput);
        }
        for (finish, begun) in value.players.iter().zip(&start.players) {
            if finish.slot != begun.slot
                || (!recovery && finish.exit == MatchExit::Unknown)
                || (value.reason.cancelled() != (finish.outcome == MatchOutcome::Cancelled))
                || (value.reason == MatchEndReason::AllDead && finish.outcome != MatchOutcome::Draw)
            {
                return Err(Error::InvalidInput);
            }
        }
        if matches!(
            value.reason,
            MatchEndReason::Elimination | MatchEndReason::Forfeit
        ) && (!value.players.iter().any(|p| p.outcome == MatchOutcome::Win)
            || !value
                .players
                .iter()
                .any(|p| p.outcome == MatchOutcome::Loss))
        {
            return Err(Error::InvalidInput);
        }
        Ok(value)
    }
}
fn read_record(db: &Connection, id: i64) -> Result<MatchRecord> {
    let (begun, ended): (String, Option<String>) = db
        .query_row(
            "SELECT start_json,finish_json FROM matches WHERE id=?1",
            [id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()?
        .ok_or(Error::InvalidInput)?;
    Ok(MatchRecord {
        id,
        start: serde_json::from_str(&begun).map_err(|_| Error::Schema)?,
        finish: ended
            .map(|s| serde_json::from_str(&s).map_err(|_| Error::Schema))
            .transpose()?,
        reward_per_player: 0,
    })
}
fn find(db: &Connection, key: &MatchKey) -> Result<Option<i64>> {
    Ok(db.query_row("SELECT id FROM matches WHERE boot_epoch=?1 AND room_session=?2 AND room_id=?3 AND room_generation=?4 AND round=?5",
        params![key.boot_epoch,key.room_session,key.room_id,key.room_generation,key.round], |r|r.get(0)).optional()?)
}
fn finish_tx(
    tx: &Transaction<'_>,
    record: MatchRecord,
    finish: MatchFinish,
) -> Result<MatchRecord> {
    if let Some(old) = &record.finish {
        return if old == &finish {
            Ok(record)
        } else {
            Err(Error::Conflict)
        };
    }
    let state = if finish.reason.cancelled() {
        "cancelled"
    } else {
        "complete"
    };
    tx.execute("UPDATE matches SET state=?1,reason=?2,ended_at=?3,finish_json=?4 WHERE id=?5 AND state='running'",
        params![state,label(&finish.reason)?,finish.ended_at,json(&finish)?,record.id])?;
    for player in &finish.players {
        if tx.execute("UPDATE match_players SET outcome=?1,exit_kind=?2 WHERE match_id=?3 AND slot=?4 AND outcome IS NULL",
            params![label(&player.outcome)?,label(&player.exit)?,record.id,player.slot])? != 1
        { return Err(Error::Conflict); }
    }
    Ok(MatchRecord {
        finish: Some(finish),
        ..record
    })
}
/// Instance startup and cancellation share one transaction with epoch/session
/// invalidation. A crash cannot leave a previous instance's match looking live.
pub(super) fn cancel_running(tx: &Transaction<'_>, now: i64) -> Result<()> {
    let ids: Vec<i64> = tx
        .prepare("SELECT id FROM matches WHERE state='running' ORDER BY id")?
        .query_map([], |r| r.get(0))?
        .collect::<std::result::Result<_, _>>()?;
    for id in ids {
        let record = read_record(tx, id)?;
        let finish = MatchFinish {
            ended_at: now,
            reason: MatchEndReason::ServerRestart,
            players: record
                .start
                .players
                .iter()
                .map(|p| MatchPlayerFinish {
                    slot: p.slot,
                    outcome: MatchOutcome::Cancelled,
                    exit: MatchExit::Unknown,
                })
                .collect(),
        };
        let finish = finish.normalize(&record.start, now, true)?;
        finish_tx(tx, record, finish)?;
    }
    Ok(())
}
impl Store {
    /// Trusted service API; caller must first validate current account/membership
    /// and retain this same key/snapshot while retrying Busy. No wire input enters
    /// this method directly. Start must commit before publishing a playable round.
    pub fn begin_match(&mut self, start: &MatchStart, now: i64) -> Result<MatchRecord> {
        let start = start.normalize(now)?;
        self.at(now,|tx| {
            let epoch:i64=tx.query_row("SELECT epoch FROM runtime_state WHERE id=1",[],|r|r.get(0))?;
            if epoch != start.key.boot_epoch { return Err(Error::InvalidSession); }
            if let Some(id)=find(tx,&start.key)? {
                let record=read_record(tx,id)?;
                return if record.start == start { Ok(record) } else { Err(Error::Conflict) };
            }
            for player in &start.players {
                let owner:Option<i64>=tx.query_row("SELECT account_id FROM players WHERE id=?1",[player.player_id],|r|r.get(0)).optional()?;
                if owner != Some(player.account_id) { return Err(Error::InvalidInput); }
            }
            let key=&start.key;
            tx.execute("INSERT INTO matches(boot_epoch,room_session,room_id,room_generation,round,map_code,map_name,started_at,state,start_json) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,'running',?9)",
                params![key.boot_epoch,key.room_session,key.room_id,key.room_generation,key.round,start.map_code,start.map_name,start.started_at,json(&start)?])?;
            let id=tx.last_insert_rowid();
            for player in &start.players {
                tx.execute("INSERT INTO match_players(match_id,account_id,player_id,slot,character_key,team) VALUES(?1,?2,?3,?4,?5,?6)",
                    params![id,player.account_id,player.player_id,player.slot,player.character_key,player.team])?;
            }
            Ok(MatchRecord{id,start:start.clone(),finish:None,reward_per_player:0})
        })
    }
    /// Commit the stable outcome event before sending a successful native result.
    /// A duplicate with different contents is a conflict, never a second grant.
    pub fn finish_match(
        &mut self,
        key: &MatchKey,
        finish: &MatchFinish,
        now: i64,
    ) -> Result<MatchRecord> {
        self.at(now, |tx| {
            let id = find(tx, key)?.ok_or(Error::InvalidInput)?;
            let record = read_record(tx, id)?;
            let finish = finish.normalize(&record.start, now, false)?;
            if record.finish.is_none() {
                let epoch: i64 =
                    tx.query_row("SELECT epoch FROM runtime_state WHERE id=1", [], |r| {
                        r.get(0)
                    })?;
                if epoch != key.boot_epoch {
                    return Err(Error::InvalidSession);
                }
            }
            finish_tx(tx, record, finish)
        })
    }
    pub fn match_history(
        &self,
        admin: &Admin<'_>,
        filter: HistoryFilter,
        limit: u8,
    ) -> Result<Vec<MatchRecord>> {
        admin.validate()?;
        if !(1..=100).contains(&limit) {
            return Err(Error::InvalidInput);
        }
        let (sql, arg) = match filter {
            HistoryFilter::Account(account) if account > 0 => (
                "SELECT match_id FROM match_players WHERE account_id=?1 ORDER BY match_id DESC LIMIT ?2",
                account,
            ),
            HistoryFilter::Room(room) if room > 0 => (
                "SELECT id FROM matches WHERE room_id=?1 ORDER BY id DESC LIMIT ?2",
                i64::from(room),
            ),
            _ => return Err(Error::InvalidInput),
        };
        let ids: Vec<i64> = self
            .db
            .prepare(sql)?
            .query_map(params![arg, limit], |r| r.get(0))?
            .collect::<std::result::Result<_, _>>()?;
        ids.into_iter()
            .map(|id| read_record(&self.db, id))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::accounts::tests::{Temp, fresh};
    fn start(a: &Player, b: &Player) -> MatchStart {
        MatchStart {
            key: MatchKey {
                boot_epoch: 1,
                room_session: 1,
                room_id: 1,
                room_generation: 1,
                round: 256,
            },
            map_code: 278,
            map_name: "ExVillage01".into(),
            started_at: 101,
            players: vec![
                MatchParticipant {
                    account_id: a.account_id,
                    player_id: a.player_id,
                    slot: 0,
                    character_key: 1,
                    team: 0,
                },
                MatchParticipant {
                    account_id: b.account_id,
                    player_id: b.player_id,
                    slot: 2,
                    character_key: 0,
                    team: 3,
                },
            ],
        }
    }
    fn finish() -> MatchFinish {
        MatchFinish {
            ended_at: 102,
            reason: MatchEndReason::Elimination,
            players: vec![
                MatchPlayerFinish {
                    slot: 0,
                    outcome: MatchOutcome::Win,
                    exit: MatchExit::Present,
                },
                MatchPlayerFinish {
                    slot: 2,
                    outcome: MatchOutcome::Loss,
                    exit: MatchExit::Disconnected,
                },
            ],
        }
    }
    fn admin() -> Admin<'static> {
        Admin {
            actor: "M6-test",
            reason: "durable history test",
        }
    }
    fn setup() -> (Temp, Store, MatchStart) {
        let (temp, mut store, a) = fresh();
        let b = store
            .create_account(&admin(), "bob", "Bob", b"correct horse battery staple", 101)
            .unwrap();
        let begun = start(&a, &b);
        (temp, store, begun)
    }
    #[test]
    fn durable_duplicate_and_conflicting_events_preserve_snapshot_zero_reward_and_history() {
        let _serial = TEST_SERIAL.lock().unwrap();
        let (temp, mut store, mut begun) = setup();
        let first = store.begin_match(&begun, 101).unwrap();
        begun.players.reverse();
        assert_eq!(store.begin_match(&begun, 101).unwrap(), first);
        let mut wrong = begun.clone();
        wrong.players[0].team = 5;
        assert!(matches!(
            store.begin_match(&wrong, 101),
            Err(Error::Conflict)
        ));
        let result = store.finish_match(&begun.key, &finish(), 102).unwrap();
        assert_eq!(
            store.finish_match(&begun.key, &finish(), 103).unwrap(),
            result
        );
        let mut conflicting = finish();
        conflicting.players[0].outcome = MatchOutcome::Loss;
        conflicting.players[1].outcome = MatchOutcome::Win;
        assert!(matches!(
            store.finish_match(&begun.key, &conflicting, 103),
            Err(Error::Conflict)
        ));
        drop(store);
        let mut store = temp.store();
        assert_eq!(
            store
                .match_history(
                    &admin(),
                    HistoryFilter::Account(first.start.players[0].account_id),
                    10
                )
                .unwrap(),
            vec![result.clone()]
        );
        assert_eq!(
            store
                .match_history(&admin(), HistoryFilter::Room(1), 10)
                .unwrap(),
            vec![result.clone()]
        );
        store.begin_instance(&admin(), 104).unwrap();
        assert_eq!(
            store.finish_match(&begun.key, &finish(), 105).unwrap(),
            result
        );
        assert!(matches!(
            store.begin_match(&begun, 105),
            Err(Error::InvalidSession)
        ));
        assert_eq!(
            store
                .db
                .query_row("SELECT count(*) FROM matches", [], |r| r.get::<_, i64>(0))
                .unwrap(),
            1
        );
        assert_eq!(
            store
                .db
                .query_row("SELECT sum(reward) FROM match_players", [], |r| r
                    .get::<_, i64>(0))
                .unwrap(),
            0
        );
    }
    #[test]
    fn restart_cancels_open_match_once_and_rejects_cross_owner_and_old_epoch() {
        let _serial = TEST_SERIAL.lock().unwrap();
        let (temp, mut store, begun) = setup();
        let mut wrong = begun.clone();
        wrong.players[0].account_id = begun.players[1].account_id;
        assert!(matches!(
            store.begin_match(&wrong, 101),
            Err(Error::InvalidInput)
        ));
        let mut wrong = begun.clone();
        wrong.players[0].player_id = 99999;
        assert!(matches!(
            store.begin_match(&wrong, 101),
            Err(Error::InvalidInput)
        ));
        let first = store.begin_match(&begun, 101).unwrap();
        drop(store);
        let mut store = temp.store();
        store.begin_instance(&admin(), 103).unwrap();
        let records = store
            .match_history(&admin(), HistoryFilter::Room(1), 10)
            .unwrap();
        assert_eq!(records.len(), 1);
        let record = &records[0];
        assert_eq!(record.id, first.id);
        let closed = record.finish.as_ref().unwrap();
        assert_eq!(closed.reason, MatchEndReason::ServerRestart);
        assert!(
            closed
                .players
                .iter()
                .all(|p| p.outcome == MatchOutcome::Cancelled && p.exit == MatchExit::Unknown)
        );
        store.begin_instance(&admin(), 104).unwrap();
        assert_eq!(
            store
                .match_history(&admin(), HistoryFilter::Room(1), 10)
                .unwrap(),
            records
        );
        assert!(matches!(
            store.finish_match(&begun.key, &finish(), 104),
            Err(Error::Conflict)
        ));
    }
    #[test]
    fn busy_and_insert_failure_leave_no_half_match_and_retry_same_event_succeeds() {
        let _serial = TEST_SERIAL.lock().unwrap();
        let (temp, mut store, begun) = setup();
        let mut lock = temp.store();
        let tx = lock
            .db
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .unwrap();
        assert!(matches!(store.begin_match(&begun, 101), Err(Error::Busy)));
        tx.rollback().unwrap();
        store.db.execute_batch("CREATE TEMP TRIGGER m6_insert_failure BEFORE INSERT ON main.match_players WHEN NEW.slot=2 BEGIN SELECT RAISE(ABORT,'injected'); END").unwrap();
        assert!(store.begin_match(&begun, 101).is_err());
        assert_eq!(
            store
                .db
                .query_row("SELECT count(*) FROM matches", [], |r| r.get::<_, i64>(0))
                .unwrap(),
            0
        );
        store
            .db
            .execute_batch("DROP TRIGGER m6_insert_failure")
            .unwrap();
        store.begin_match(&begun, 101).unwrap();
        let tx = lock
            .db
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .unwrap();
        assert!(matches!(
            store.finish_match(&begun.key, &finish(), 102),
            Err(Error::Busy)
        ));
        tx.rollback().unwrap();
        store.db.execute_batch("CREATE TEMP TRIGGER m6_finish_failure BEFORE UPDATE ON main.match_players WHEN NEW.slot=2 BEGIN SELECT RAISE(ABORT,'injected'); END").unwrap();
        assert!(store.finish_match(&begun.key, &finish(), 102).is_err());
        assert!(
            store
                .match_history(&admin(), HistoryFilter::Room(1), 1)
                .unwrap()[0]
                .finish
                .is_none()
        );
        store
            .db
            .execute_batch("DROP TRIGGER m6_finish_failure")
            .unwrap();
        store.finish_match(&begun.key, &finish(), 102).unwrap();
        assert_eq!(
            store
                .match_history(&admin(), HistoryFilter::Room(1), 1)
                .unwrap()[0]
                .finish,
            Some(finish())
        );
    }
    #[test]
    fn cancellation_and_roster_validation_never_record_cancelled_players_as_winners() {
        let _serial = TEST_SERIAL.lock().unwrap();
        let (_temp, mut store, begun) = setup();
        store.begin_match(&begun, 101).unwrap();
        let mut ended = finish();
        ended.reason = MatchEndReason::LoadingAborted;
        assert!(matches!(
            store.finish_match(&begun.key, &ended, 102),
            Err(Error::InvalidInput)
        ));
        for p in &mut ended.players {
            p.outcome = MatchOutcome::Cancelled;
        }
        let mut wrong = ended.clone();
        wrong.players[1].slot = 0;
        assert!(matches!(
            store.finish_match(&begun.key, &wrong, 102),
            Err(Error::InvalidInput)
        ));
        assert!(
            store
                .finish_match(&begun.key, &ended, 102)
                .unwrap()
                .finish
                .unwrap()
                .players
                .iter()
                .all(|p| p.outcome == MatchOutcome::Cancelled)
        );
    }
    #[test]
    fn crash_commit_child() {
        let Some(root) = std::env::var_os("KR_M6_CRASH_TEST_ROOT") else {
            return;
        };
        let root = std::path::PathBuf::from(root);
        let start: MatchStart =
            serde_json::from_slice(&std::fs::read(root.join("start.json")).unwrap()).unwrap();
        let mut store = Store::open(&root.join("accounts.sqlite")).unwrap();
        store.begin_match(&start, 101).unwrap();
        if std::env::var_os("KR_M6_CRASH_TEST_FINISH").is_some() {
            store.finish_match(&start.key, &finish(), 102).unwrap();
        }
        std::fs::write(root.join("committed"), b"committed").unwrap();
        loop {
            std::thread::park();
        }
    }
    #[test]
    fn process_kill_after_commit_preserves_exact_result_or_cancels_unfinished_on_restart() {
        let _serial = TEST_SERIAL.lock().unwrap();
        for completed in [false, true] {
            let (temp, store, begun) = setup();
            drop(store);
            std::fs::write(
                temp.0.join("start.json"),
                serde_json::to_vec(&begun).unwrap(),
            )
            .unwrap();
            let mut cmd = std::process::Command::new(std::env::current_exe().unwrap());
            cmd.args([
                "--exact",
                "accounts::matches::tests::crash_commit_child",
                "--nocapture",
            ])
            .env("KR_M6_CRASH_TEST_ROOT", &temp.0)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null());
            if completed {
                cmd.env("KR_M6_CRASH_TEST_FINISH", "1");
            }
            let mut child = cmd.spawn().unwrap();
            let deadline = std::time::Instant::now() + Duration::from_secs(10);
            while !temp.0.join("committed").exists() {
                if std::time::Instant::now() >= deadline || child.try_wait().unwrap().is_some() {
                    let _ = child.kill();
                    let _ = child.wait();
                    panic!("child failed to commit before kill test");
                }
                std::thread::sleep(Duration::from_millis(5));
            }
            child.kill().unwrap();
            let killed = child.wait().unwrap();
            use std::os::unix::process::ExitStatusExt;
            assert_eq!(killed.signal(), Some(9));
            let mut store = temp.store();
            let prior = store
                .match_history(&admin(), HistoryFilter::Room(1), 10)
                .unwrap();
            assert_eq!(prior.len(), 1);
            assert_eq!(
                prior[0].finish,
                if completed { Some(finish()) } else { None }
            );
            store.begin_instance(&admin(), 103).unwrap();
            let after = store
                .match_history(&admin(), HistoryFilter::Room(1), 10)
                .unwrap();
            assert_eq!(after.len(), 1);
            if completed {
                assert_eq!(after, prior);
            } else {
                assert_eq!(
                    after[0].finish.as_ref().unwrap().reason,
                    MatchEndReason::ServerRestart
                );
            }
            std::fs::write(temp.0.join("kill-control.json"),serde_json::to_vec_pretty(&serde_json::json!({
                "signal":killed.signal(),"completed_before_kill":completed,"records_before_restart":prior,"records_after_restart":after
            })).unwrap()).unwrap();
        }
    }
    #[test]
    fn instance_audit_failure_rolls_back_cancellation_epoch_and_session_invalidation() {
        let _serial = TEST_SERIAL.lock().unwrap();
        let (_temp, mut store, begun) = setup();
        store.begin_match(&begun, 101).unwrap();
        store.db.execute_batch("CREATE TEMP TRIGGER m6_audit_failure BEFORE INSERT ON main.admin_audit BEGIN SELECT RAISE(ABORT,'injected'); END").unwrap();
        assert!(store.begin_instance(&admin(), 103).is_err());
        assert_eq!(
            store
                .db
                .query_row("SELECT epoch FROM runtime_state WHERE id=1", [], |r| r
                    .get::<_, i64>(0))
                .unwrap(),
            1
        );
        assert!(
            store
                .match_history(&admin(), HistoryFilter::Room(1), 10)
                .unwrap()[0]
                .finish
                .is_none()
        );
        store
            .db
            .execute_batch("DROP TRIGGER m6_audit_failure")
            .unwrap();
        store.finish_match(&begun.key, &finish(), 103).unwrap();
    }
}
