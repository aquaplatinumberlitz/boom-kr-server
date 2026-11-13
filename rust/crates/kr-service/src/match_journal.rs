//! Durable gates for company-owned match history. No original authority inferred.
//! A bounded actor keeps DB awaits off room locks and owns exact retry events.
use crate::{
    account_worker::{Client, Error, PersistenceJob, PersistenceResult},
    accounts::{
        self, MatchEndReason, MatchExit, MatchFinish, MatchKey, MatchOutcome, MatchPlayerFinish,
        MatchStart,
    },
};
use std::{
    collections::BTreeMap,
    io,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tokio::{
    sync::{mpsc, watch},
    task::JoinHandle,
};

fn err(s: impl Into<String>) -> io::Error {
    io::Error::other(s.into())
}
pub(crate) fn utc() -> io::Result<i64> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(io::Error::other)?
        .as_secs()
        .try_into()
        .map_err(io::Error::other)
}
#[derive(Clone)]
enum State {
    Pending,
    Committed,
    Failed(String),
}
#[derive(Clone)]
pub(crate) struct Ticket(watch::Receiver<State>);
impl Ticket {
    pub(crate) fn ready(&self) -> io::Result<bool> {
        match &*self.0.borrow() {
            State::Pending => Ok(false),
            State::Committed => Ok(true),
            State::Failed(e) => Err(err(e.clone())),
        }
    }
    pub(crate) async fn wait(&self) -> io::Result<()> {
        let mut status = self.0.clone();
        loop {
            match &*status.borrow_and_update() {
                State::Committed => return Ok(()),
                State::Failed(e) => return Err(err(e.clone())),
                State::Pending => {}
            }
            status
                .changed()
                .await
                .map_err(|_| err("match journal completion closed"))?;
        }
    }
}
#[derive(Clone)]
enum Event {
    Start(MatchStart),
    Finish(MatchKey, MatchFinish),
}
impl Event {
    fn job(&self) -> PersistenceJob {
        match self {
            Self::Start(s) => PersistenceJob::BeginMatch(s.clone()),
            Self::Finish(k, f) => PersistenceJob::FinishMatch(k.clone(), f.clone()),
        }
    }
    fn key(&self) -> String {
        let k = match self {
            Self::Start(s) => &s.key,
            Self::Finish(k, _) => k,
        };
        format!(
            "{}/{}/{}/{}/{}",
            k.boot_epoch, k.room_session, k.room_id, k.room_generation, k.round
        )
    }
}
struct Command {
    event: Event,
    status: watch::Sender<State>,
}
#[derive(Clone)]
pub(crate) struct Factory {
    sender: mpsc::Sender<Command>,
    epoch: i64,
    unhealthy: Arc<AtomicBool>,
    stopping: Arc<AtomicBool>,
}
pub(crate) struct Manager {
    factory: Factory,
    stop: watch::Sender<bool>,
    task: JoinHandle<io::Result<()>>,
}
impl Factory {
    pub(crate) fn epoch(&self) -> i64 {
        self.epoch
    }
    pub(crate) fn owner(&self) -> Owner {
        Owner {
            factory: self.clone(),
            current: None,
            ticket: None,
        }
    }
    fn submit(&self, event: Event) -> io::Result<Ticket> {
        if self.unhealthy.load(Ordering::Acquire) {
            return Err(err("match journal unavailable"));
        }
        let (status, rx) = watch::channel(State::Pending);
        if self.sender.try_send(Command { event, status }).is_err() {
            self.unhealthy.store(true, Ordering::Release);
            return Err(err("match journal queue unavailable"));
        }
        Ok(Ticket(rx))
    }
}
impl Manager {
    pub(crate) fn open(client: Client, epoch: i64) -> Self {
        let (sender, mut receiver) = mpsc::channel::<Command>(128);
        let (stop, mut stopping) = watch::channel(false);
        let factory = Factory {
            sender,
            epoch,
            unhealthy: Arc::new(AtomicBool::new(false)),
            stopping: Arc::new(AtomicBool::new(false)),
        };
        let health = factory.unhealthy.clone();
        let task = tokio::spawn(async move {
            let mut open = BTreeMap::new();
            let mut failure = None;
            loop {
                let command = tokio::select! {biased; _=stopping.changed()=>{receiver.close();receiver.recv().await},c=receiver.recv()=>c};
                let Some(command) = command else { break };
                let result = if failure.is_some() {
                    Err(err("earlier match journal failure"))
                } else {
                    persist(&client, &command.event).await
                };
                match result {
                    Ok(()) => {
                        let key = command.event.key();
                        match command.event {
                            Event::Start(start) => {
                                open.insert(key, start);
                            }
                            Event::Finish(_, _) => {
                                open.remove(&key);
                            }
                        };
                        command.status.send_replace(State::Committed);
                    }
                    Err(e) => {
                        health.store(true, Ordering::Release);
                        command.status.send_replace(State::Failed(e.to_string()));
                        failure.get_or_insert(e);
                    }
                }
            }
            if !open.is_empty() {
                failure.get_or_insert_with(|| {
                    err("match journal shutdown has unfinished rows; next boot must cancel")
                });
            }
            failure.map_or(Ok(()), Err)
        });
        Self {
            factory,
            stop,
            task,
        }
    }
    pub(crate) fn factory(&self) -> Factory {
        self.factory.clone()
    }
    pub(crate) fn healthy(&self) -> bool {
        !self.factory.unhealthy.load(Ordering::Acquire) && !self.task.is_finished()
    }
    pub(crate) fn stopping(&self) {
        self.factory.stopping.store(true, Ordering::Release);
    }
    pub(crate) async fn shutdown(self) -> io::Result<()> {
        self.stop.send_replace(true);
        self.task.await.map_err(io::Error::other)?
    }
}
async fn persist(client: &Client, event: &Event) -> io::Result<()> {
    let deadline = tokio::time::Instant::now() + Duration::from_secs(5);
    loop {
        let attempt = match client.try_persist(event.job()) {
            Ok(p) => p.wait().await,
            Err(e) => Err(e),
        };
        match attempt {
            Ok(PersistenceResult::Match(record)) => {
                let correct = match event {
                    Event::Start(start) => record.start == *start && record.finish.is_none(),
                    Event::Finish(key, finish) => {
                        record.start.key == *key && record.finish.as_ref() == Some(finish)
                    }
                };
                return if correct && record.reward_per_player == 0 {
                    Ok(())
                } else {
                    Err(err("match journal readback differs"))
                };
            }
            Err(Error::Busy | Error::QueueExpired | Error::Backend(accounts::Error::Busy))
                if tokio::time::Instant::now() < deadline =>
            {
                tokio::time::sleep(Duration::from_millis(20)).await
            }
            other => {
                return Err(err(format!(
                    "match journal persistence failed: {}",
                    match other {
                        Err(e) => format!("{e:?}"),
                        Ok(_) => "unexpected result".into(),
                    }
                )));
            }
        }
    }
}
struct Round {
    start: MatchStart,
    exits: BTreeMap<u8, MatchExit>,
    finished: bool,
}
pub(crate) struct Owner {
    factory: Factory,
    current: Option<Round>,
    ticket: Option<Ticket>,
}
impl Owner {
    pub(crate) fn epoch(&self) -> i64 {
        self.factory.epoch()
    }
    pub(crate) fn ticket(&self) -> Option<Ticket> {
        self.ticket.clone()
    }
    pub(crate) fn ready(&self) -> io::Result<bool> {
        self.ticket.as_ref().map_or(Ok(true), Ticket::ready)
    }
    pub(crate) fn begin(&mut self, start: MatchStart) -> io::Result<()> {
        if self.factory.stopping.load(Ordering::Acquire)
            || self.current.as_ref().is_some_and(|r| !r.finished)
            || !self.ready()?
        {
            return Err(err("match journal round not retired"));
        }
        let ticket = self.factory.submit(Event::Start(start.clone()))?;
        self.current = Some(Round {
            start,
            exits: BTreeMap::new(),
            finished: false,
        });
        self.ticket = Some(ticket);
        Ok(())
    }
    pub(crate) fn departed(&mut self, slot: u8, exit: MatchExit) {
        if let Some(round) = &mut self.current
            && !round.finished
        {
            round.exits.entry(slot).or_insert(exit);
        }
    }
    pub(crate) fn finish(&mut self, winner: u8) -> io::Result<()> {
        let Some(round) = &self.current else {
            return Err(err("match journal has no start"));
        };
        if round.finished {
            return Err(err("match journal duplicate transition"));
        }
        let reason = if round.exits.is_empty() {
            MatchEndReason::Elimination
        } else {
            MatchEndReason::Forfeit
        };
        let finish = MatchFinish {
            ended_at: utc()?.max(round.start.started_at),
            reason,
            players: round
                .start
                .players
                .iter()
                .map(|p| MatchPlayerFinish {
                    slot: p.slot,
                    outcome: if p.team == winner {
                        MatchOutcome::Win
                    } else {
                        MatchOutcome::Loss
                    },
                    exit: *round.exits.get(&p.slot).unwrap_or(&MatchExit::Present),
                })
                .collect(),
        };
        self.complete(finish)
    }
    pub(crate) fn cancel(&mut self, reason: MatchEndReason) -> io::Result<()> {
        let Some(round) = &self.current else {
            return Ok(());
        };
        if round.finished {
            return Ok(());
        }
        let reason = if self.factory.stopping.load(Ordering::Acquire) {
            MatchEndReason::ServerShutdown
        } else {
            reason
        };
        let finish = MatchFinish {
            ended_at: utc()?.max(round.start.started_at),
            reason,
            players: round
                .start
                .players
                .iter()
                .map(|p| MatchPlayerFinish {
                    slot: p.slot,
                    outcome: MatchOutcome::Cancelled,
                    exit: *round.exits.get(&p.slot).unwrap_or(&MatchExit::Present),
                })
                .collect(),
        };
        self.complete(finish)
    }
    fn complete(&mut self, finish: MatchFinish) -> io::Result<()> {
        let round = self
            .current
            .as_ref()
            .ok_or_else(|| err("match journal has no start"))?;
        let ticket = self
            .factory
            .submit(Event::Finish(round.start.key.clone(), finish))?;
        self.current.as_mut().expect("current round").finished = true;
        self.ticket = Some(ticket);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        account_worker::Worker,
        accounts::{Admin, MatchParticipant, Store},
    };
    use std::{fs, os::unix::fs::PermissionsExt, path::PathBuf, sync::atomic::AtomicU64};
    static NEXT: AtomicU64 = AtomicU64::new(0);
    struct Temp(PathBuf, bool);
    impl Drop for Temp {
        fn drop(&mut self) {
            if !self.1 {
                let _ = fs::remove_dir_all(&self.0);
            }
        }
    }
    async fn setup() -> (Temp, Worker, Manager, MatchStart) {
        let retained = std::env::var_os("KR_MATCH_JOURNAL_TEST_RAW").map(PathBuf::from);
        let root = retained
            .clone()
            .unwrap_or_else(std::env::temp_dir)
            .join(format!(
                "kr-journal-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
        fs::create_dir(&root).unwrap();
        fs::set_permissions(&root, fs::Permissions::from_mode(0o700)).unwrap();
        let db = root.join("accounts.sqlite");
        let (epoch, players) = {
            let _serial = crate::accounts::TEST_SERIAL.lock().unwrap();
            let mut store = Store::open(&db).unwrap();
            let admin = Admin {
                actor: "journal-test",
                reason: "isolated durable gates",
            };
            let now = utc().unwrap();
            let epoch = store.begin_instance(&admin, now).unwrap();
            let mut players = Vec::new();
            for (slot, name, team) in [(0, "Alice", 0), (6, "Bob", 3)] {
                let p = store
                    .create_account(&admin, name, name, b"journal-test-password", now)
                    .unwrap();
                players.push(MatchParticipant {
                    account_id: p.account_id,
                    player_id: p.player_id,
                    slot,
                    character_key: 0,
                    team,
                });
            }
            (epoch, players)
        };
        let worker = Worker::open(db, 4, Duration::from_secs(2)).await.unwrap();
        let manager = Manager::open(worker.client(), epoch);
        let start = MatchStart {
            key: MatchKey {
                boot_epoch: epoch,
                room_session: 5,
                room_id: 7,
                room_generation: 9,
                round: 100,
            },
            map_code: 281,
            map_name: "Factory07".into(),
            started_at: utc().unwrap(),
            players,
        };
        (Temp(root, retained.is_some()), worker, manager, start)
    }
    fn history(temp: &Temp) -> Vec<accounts::MatchRecord> {
        let reader = rusqlite::Connection::open_with_flags(
            temp.0.join("accounts.sqlite"),
            rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
        )
        .unwrap();
        reader
            .prepare("SELECT id,start_json,finish_json FROM matches ORDER BY id DESC")
            .unwrap()
            .query_map([], |r| {
                Ok((
                    r.get::<_, i64>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, Option<String>>(2)?,
                ))
            })
            .unwrap()
            .map(|r| {
                let (id, start, finish) = r.unwrap();
                accounts::MatchRecord {
                    id,
                    start: serde_json::from_str(&start).unwrap(),
                    finish: finish.map(|f| serde_json::from_str(&f).unwrap()),
                    reward_per_player: 0,
                }
            })
            .collect()
    }
    #[tokio::test]
    async fn real_writer_locks_gate_start_finish_and_exact_duplicate_recovery() {
        let (temp, worker, manager, start) = setup().await;
        let mut owner = manager.factory().owner();
        let writer = rusqlite::Connection::open(temp.0.join("accounts.sqlite")).unwrap();
        writer.execute_batch("BEGIN IMMEDIATE").unwrap();
        owner.begin(start.clone()).unwrap();
        let first = owner.ticket().unwrap();
        assert!(
            tokio::time::timeout(Duration::from_millis(150), first.wait())
                .await
                .is_err()
        );
        assert!(history(&temp).is_empty());
        writer.execute_batch("COMMIT").unwrap();
        first.wait().await.unwrap();
        assert_eq!(history(&temp)[0].start, start);
        writer.execute_batch("BEGIN IMMEDIATE").unwrap();
        owner.departed(6, MatchExit::Left);
        owner.finish(0).unwrap();
        let end = owner.ticket().unwrap();
        assert!(
            tokio::time::timeout(Duration::from_millis(150), end.wait())
                .await
                .is_err()
        );
        assert!(history(&temp)[0].finish.is_none());
        writer.execute_batch("COMMIT").unwrap();
        end.wait().await.unwrap();
        let records = history(&temp);
        assert_eq!(records.len(), 1);
        let finish = records[0].finish.as_ref().unwrap();
        assert_eq!(finish.reason, MatchEndReason::Forfeit);
        assert_eq!(finish.players[1].exit, MatchExit::Left);
        assert_eq!(records[0].reward_per_player, 0);
        manager
            .factory()
            .submit(Event::Finish(start.key, finish.clone()))
            .unwrap()
            .wait()
            .await
            .unwrap();
        assert_eq!(history(&temp), records);
        manager.shutdown().await.unwrap();
        worker.shutdown().await.unwrap();
    }
    #[tokio::test]
    async fn shutdown_cancels_loading_and_old_ticket_cannot_open_a_new_round() {
        let (temp, worker, manager, start) = setup().await;
        let mut owner = manager.factory().owner();
        owner.begin(start.clone()).unwrap();
        let old = owner.ticket().unwrap();
        old.wait().await.unwrap();
        manager.stopping();
        owner.departed(6, MatchExit::Disconnected);
        owner.cancel(MatchEndReason::LoadingAborted).unwrap();
        owner.ticket().unwrap().wait().await.unwrap();
        let mut next = start;
        next.key.round += 1;
        assert!(owner.begin(next).is_err());
        assert!(old.ready().unwrap());
        manager.shutdown().await.unwrap();
        worker.shutdown().await.unwrap();
        let result = history(&temp);
        assert_eq!(result.len(), 1);
        let finish = result[0].finish.as_ref().unwrap();
        assert_eq!(finish.reason, MatchEndReason::ServerShutdown);
        assert!(
            finish
                .players
                .iter()
                .all(|p| p.outcome == MatchOutcome::Cancelled)
        );
        assert_eq!(finish.players[1].exit, MatchExit::Disconnected);
    }
    #[tokio::test]
    async fn conflicting_result_poisoning_is_visible_and_never_rewrites_committed_match() {
        let (temp, worker, manager, start) = setup().await;
        let mut owner = manager.factory().owner();
        owner.begin(start.clone()).unwrap();
        owner.ticket().unwrap().wait().await.unwrap();
        owner.finish(3).unwrap();
        owner.ticket().unwrap().wait().await.unwrap();
        let before = history(&temp);
        let mut conflict = before[0].finish.clone().unwrap();
        conflict.players[0].outcome = MatchOutcome::Win;
        conflict.players[1].outcome = MatchOutcome::Loss;
        assert!(
            manager
                .factory()
                .submit(Event::Finish(start.key, conflict))
                .unwrap()
                .wait()
                .await
                .is_err()
        );
        assert!(!manager.healthy());
        assert_eq!(history(&temp), before);
        assert!(manager.shutdown().await.is_err());
        worker.shutdown().await.unwrap();
    }
    #[tokio::test]
    async fn no_players_cancellation_does_not_award_last_disconnect_or_leak_to_next_round() {
        let (temp, worker, manager, mut start) = setup().await;
        let mut owner = manager.factory().owner();
        owner.begin(start.clone()).unwrap();
        owner.ticket().unwrap().wait().await.unwrap();
        owner.departed(0, MatchExit::Disconnected);
        owner.departed(6, MatchExit::Disconnected);
        owner.cancel(MatchEndReason::NoPlayers).unwrap();
        owner.ticket().unwrap().wait().await.unwrap();
        start.key.round += 100;
        start.started_at = utc().unwrap();
        owner.begin(start.clone()).unwrap();
        owner.ticket().unwrap().wait().await.unwrap();
        owner.finish(0).unwrap();
        owner.ticket().unwrap().wait().await.unwrap();
        manager.shutdown().await.unwrap();
        worker.shutdown().await.unwrap();
        let records = history(&temp);
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].start, start);
        assert_eq!(
            records[0].finish.as_ref().unwrap().reason,
            MatchEndReason::Elimination
        );
        assert!(
            records[0]
                .finish
                .as_ref()
                .unwrap()
                .players
                .iter()
                .all(|p| p.exit == MatchExit::Present)
        );
        let cancelled = records[1].finish.as_ref().unwrap();
        assert_eq!(cancelled.reason, MatchEndReason::NoPlayers);
        assert!(
            cancelled
                .players
                .iter()
                .all(|p| p.outcome == MatchOutcome::Cancelled && p.exit == MatchExit::Disconnected)
        );
    }
}
