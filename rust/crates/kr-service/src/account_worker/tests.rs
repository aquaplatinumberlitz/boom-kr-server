use super::*;
use crate::accounts::{Admin, TEST_SERIAL};
use std::{
    fs,
    os::unix::fs::PermissionsExt,
    sync::atomic::{AtomicU64, Ordering},
};
static NEXT: AtomicU64 = AtomicU64::new(0);
const PASSWORD: &[u8] = b"worker-test-password";
struct Temp(PathBuf, bool);
impl Temp {
    fn new() -> Self {
        let retained = std::env::var_os("KR_ACCOUNT_WORKER_TEST_RAW").map(PathBuf::from);
        let retain = retained.is_some();
        let parent = retained.unwrap_or_else(std::env::temp_dir);
        if retain {
            let meta = fs::symlink_metadata(&parent).unwrap();
            assert!(meta.is_dir() && meta.permissions().mode() & 0o077 == 0);
        }
        let current = std::thread::current();
        let case = current
            .name()
            .unwrap_or("unnamed")
            .rsplit("::")
            .next()
            .unwrap();
        let p = parent.join(format!(
            "kr-worker-{}-{}-{case}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&p).unwrap();
        fs::set_permissions(&p, fs::Permissions::from_mode(0o700)).unwrap();
        let temp = Self(p, retain);
        let mut store = Store::open(&temp.db()).unwrap();
        store.begin_instance(&admin(), 100).unwrap();
        store
            .create_account(&admin(), "Alice", "Alice", PASSWORD, 100)
            .unwrap();
        temp
    }
    fn db(&self) -> PathBuf {
        self.0.join("accounts.sqlite")
    }
    fn sql(&self) -> rusqlite::Connection {
        rusqlite::Connection::open(self.db()).unwrap()
    }
    fn active(&self) -> i64 {
        self.sql()
            .query_row(
                "SELECT count(*) FROM sessions WHERE revoked_at IS NULL",
                [],
                |r| r.get(0),
            )
            .unwrap()
    }
}
impl Drop for Temp {
    fn drop(&mut self) {
        if !self.1 {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
}
fn admin() -> Admin<'static> {
    Admin {
        actor: "test-admin",
        reason: "bounded account worker tests",
    }
}
fn rt() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
}
pub(crate) async fn pause(client: &Client) -> std::sync::mpsc::Sender<()> {
    tokio::time::timeout(Duration::from_secs(3), async {
        loop {
            let (entered, ready) = oneshot::channel();
            let (resume, rx) = std::sync::mpsc::channel();
            match client.submit(Command::Pause {
                entered,
                resume: rx,
            }) {
                Ok(()) => {
                    ready.await.unwrap();
                    return resume;
                }
                Err(Error::Busy) => tokio::task::yield_now().await,
                other => panic!("unexpected admission result: {other:?}"),
            }
        }
    })
    .await
    .unwrap()
}
#[test]
fn real_sqlite_login_authorize_and_blocked_worker_leave_tokio_responsive() {
    let _serial = TEST_SERIAL.lock().unwrap();
    let temp = Temp::new();
    rt().block_on(async {
        let worker = Worker::open(temp.db(), 4, Duration::from_secs(5))
            .await
            .unwrap();
        let client = worker.client();
        let resume = pause(&client).await;
        let pending = client.try_login("ALICE", PASSWORD, 60, 101).unwrap();
        // This single-thread Tokio executor remains responsive while the account
        // thread is held at a deterministic barrier and the real login is queued.
        tokio::time::timeout(
            Duration::from_secs(1),
            tokio::time::sleep(Duration::from_millis(5)),
        )
        .await
        .unwrap();
        assert_eq!(temp.active(), 0);
        resume.send(()).unwrap();
        let grant = pending.wait().await.unwrap();
        assert_eq!(grant.player.display_name, "Alice");
        assert_eq!(
            client
                .try_authorize(&grant.token, grant.player.player_id, 102)
                .unwrap()
                .wait()
                .await,
            Ok(grant.player.clone())
        );
        assert_eq!(
            client
                .try_authorize(&grant.token, grant.player.player_id + 1, 102)
                .unwrap()
                .wait()
                .await,
            Err(Error::Backend(accounts::Error::InvalidSession))
        );
        assert!(matches!(
            client
                .try_login("alice", PASSWORD, 60, 103)
                .unwrap()
                .wait()
                .await,
            Err(Error::Backend(accounts::Error::ActiveSession))
        ));
        assert!(matches!(
            client
                .try_login("unknown", PASSWORD, 60, 103)
                .unwrap()
                .wait()
                .await,
            Err(Error::Backend(accounts::Error::InvalidCredentials))
        ));
        worker.shutdown().await.unwrap();
        assert!(client.is_closed());
        assert!(matches!(
            client.try_login("alice", PASSWORD, 60, 104),
            Err(Error::Closed)
        ));
        let mut store = Store::open(&temp.db()).unwrap();
        assert_eq!(
            store
                .authorize(&grant.token, grant.player.player_id, 104)
                .unwrap(),
            grant.player
        );
        store
            .revoke(&admin(), grant.player.account_id, 105)
            .unwrap();
        assert_eq!(
            store.authorize(&grant.token, grant.player.player_id, 106),
            Err(accounts::Error::InvalidSession)
        );
    });
}
#[test]
fn admission_is_bounded_cancelled_and_queue_expired_work_never_issues_a_token() {
    let _serial = TEST_SERIAL.lock().unwrap();
    let temp = Temp::new();
    rt().block_on(async {
        assert!(matches!(
            Worker::open(temp.db(), 0, Duration::from_secs(1)).await,
            Err(Error::InvalidInput)
        ));
        let worker = Worker::open(temp.db(), 1, Duration::from_secs(5))
            .await
            .unwrap();
        let client = worker.client();
        let resume = pause(&client).await;
        let cancelled = client.try_login("alice", PASSWORD, 60, 101).unwrap();
        assert!(matches!(
            client.try_login("alice", PASSWORD, 60, 101),
            Err(Error::Busy)
        ));
        assert!(matches!(
            client.try_login(&"a".repeat(10000), PASSWORD, 60, 101),
            Err(Error::InvalidInput)
        ));
        drop(cancelled);
        resume.send(()).unwrap();
        let resume = pause(&client).await;
        assert_eq!(temp.active(), 0);
        let (reply, rx) = oneshot::channel();
        let handoff = Handoff::new();
        client
            .submit(Command::Login {
                login: "alice".into(),
                password: Zeroizing::new(PASSWORD.to_vec()),
                ttl: 60,
                idle: None,
                now: 101,
                deadline: Instant::now() - Duration::from_secs(1),
                reply,
                handoff: handoff.clone(),
                handoff_ttl: Duration::from_secs(1),
            })
            .unwrap();
        resume.send(()).unwrap();
        assert!(matches!(
            Pending {
                reply: Some(rx),
                handoff: Some(handoff)
            }
            .wait()
            .await,
            Err(Error::QueueExpired)
        ));
        assert_eq!(temp.active(), 0);
        let grant = client
            .try_login("alice", PASSWORD, 60, 102)
            .unwrap()
            .wait()
            .await
            .unwrap();
        let authorized = client
            .try_authorize(&grant.token, grant.player.player_id, 103)
            .unwrap();
        worker.shutdown().await.unwrap();
        assert!(client.is_closed());
        assert_eq!(authorized.wait().await, Ok(grant.player));
    });
}
#[test]
fn unconsumed_committed_result_is_revoked_atomically_without_revoking_a_newer_session() {
    let _serial = TEST_SERIAL.lock().unwrap();
    let temp = Temp::new();
    let mut store = Store::open(&temp.db()).unwrap();
    let grant = store.login("alice", PASSWORD, 60, 101).unwrap();
    let old = Grant {
        player: grant.player.clone(),
        token: Token::parse(grant.token.expose_secret()).unwrap(),
        expires_at: grant.expires_at,
    };
    // oneshot.send succeeds, but the result has not been polled by its caller.
    let (reply, rx) = oneshot::channel();
    let handoff = Handoff::new();
    deliver_login(&mut store, reply, Ok(grant), 101, &handoff, Duration::ZERO).unwrap();
    assert_eq!(temp.active(), 0);
    assert!(matches!(
        rt().block_on(
            Pending {
                reply: Some(rx),
                handoff: Some(handoff)
            }
            .wait()
        ),
        Err(Error::QueueExpired)
    ));
    let audit: i64 = temp
        .sql()
        .query_row(
            "SELECT count(*) FROM admin_audit WHERE operation='abandoned_login'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(audit, 1);
    let current = store.login("alice", PASSWORD, 60, 102).unwrap();
    store
        .authorize(&current.token, current.player.player_id, 110)
        .unwrap();
    store.release_undelivered(&old, 101).unwrap();
    assert_eq!(temp.active(), 1);
    assert_eq!(
        store.authorize(&current.token, current.player.player_id, 109),
        Err(accounts::Error::ClockRollback)
    );
    assert!(
        store
            .authorize(&current.token, current.player.player_id, 111)
            .is_ok()
    );
    let handoff = Handoff::new();
    handoff.claim().unwrap();
    assert!(handoff.accepted(Duration::ZERO).unwrap());
    handoff.abandon();
    assert!(handoff.accepted(Duration::ZERO).unwrap());
}
#[test]
fn compensation_fault_and_panic_close_admission_and_shutdown_preserves_root_error() {
    let _serial = TEST_SERIAL.lock().unwrap();
    let temp = Temp::new();
    rt().block_on(async {
        let worker=Worker::open(temp.db(),2,Duration::from_secs(5)).await.unwrap();let client=worker.client();
        temp.sql().execute_batch("CREATE TRIGGER fail_abandoned_audit BEFORE INSERT ON admin_audit WHEN NEW.operation='abandoned_login' BEGIN SELECT RAISE(ABORT,'injected audit failure'); END;").unwrap();
        let pending=client.try_login("alice",PASSWORD,60,101).unwrap();
        tokio::time::timeout(Duration::from_secs(5),async {while temp.active()==0{tokio::time::sleep(Duration::from_millis(5)).await;}}).await.unwrap();
        drop(pending);
        tokio::time::timeout(Duration::from_secs(3),async {while !client.is_closed(){tokio::task::yield_now().await;}}).await.unwrap();
        assert_eq!(worker.shutdown().await,Err(Error::Backend(accounts::Error::Conflict)));
        // Audit failure rolls back compensation; worker closes instead of claiming
        // cleanup succeeded. Operator revoke/begin-instance recovery is required.
        assert_eq!(temp.active(),1);
        temp.sql().execute_batch("DROP TRIGGER fail_abandoned_audit;").unwrap();
        let worker=Worker::open(temp.db(),1,Duration::from_secs(1)).await.unwrap();let client=worker.client();client.submit(Command::Panic).unwrap();
        tokio::time::timeout(Duration::from_secs(3),async {while !client.is_closed(){tokio::task::yield_now().await;}}).await.unwrap();
        assert_eq!(worker.shutdown().await,Err(Error::WorkerFault));
        let path=temp.0.join("missing-parent/db");assert!(matches!(Worker::open(path,1,Duration::from_secs(1)).await,Err(Error::Backend(accounts::Error::Storage))));
    });
}

#[test]
fn unpolled_handoff_expires_on_real_worker_and_account_can_login_again() {
    let _serial = TEST_SERIAL.lock().unwrap();
    let temp = Temp::new();
    rt().block_on(async {
        let worker = Worker::open(temp.db(), 2, Duration::from_secs(1))
            .await
            .unwrap();
        let client = worker.client();
        let pending = client.try_login("alice", PASSWORD, 60, 101).unwrap();
        tokio::time::timeout(Duration::from_secs(10), async {
            // The job is committed and waiting for result acceptance. Do not poll
            // Pending: a successful oneshot send must not count as acceptance.
            while temp.active() == 0 {
                tokio::time::sleep(Duration::from_millis(5)).await;
            }
            while temp.active() != 0 {
                tokio::time::sleep(Duration::from_millis(5)).await;
            }
        })
        .await
        .unwrap();
        assert!(matches!(pending.wait().await, Err(Error::QueueExpired)));
        let next = client
            .try_login("alice", PASSWORD, 60, 102)
            .unwrap()
            .wait()
            .await
            .unwrap();
        assert_eq!(temp.active(), 1);
        assert_eq!(
            client
                .try_authorize(&next.token, next.player.player_id, 103)
                .unwrap()
                .wait()
                .await,
            Ok(next.player)
        );
        worker.shutdown().await.unwrap();
    });
}

#[test]
fn cloned_principals_share_rate_but_full_queue_does_not_charge_and_cancelled_admission_does() {
    let _serial = TEST_SERIAL.lock().unwrap();
    let temp = Temp::new();
    rt().block_on(async {
        let policy = RatePolicy {
            attempts: 2,
            interval: Duration::from_secs(60),
            principals: 2,
        };
        let worker = Worker::open_with_rate(temp.db(), 1, Duration::from_secs(5), policy)
            .await
            .unwrap();
        let c = worker.client();
        let clone = c.clone();
        let resume = pause(&c).await;
        let canceled = c.try_login("ALICE", PASSWORD, 60, 101).unwrap();
        assert!(matches!(
            clone.try_login("unknown", PASSWORD, 60, 101),
            Err(Error::Busy)
        ));
        drop(canceled);
        resume.send(()).unwrap();
        let resume = pause(&c).await;
        assert_eq!(temp.active(), 0);
        resume.send(()).unwrap();
        let grant = clone
            .try_login("alice", PASSWORD, 60, 102)
            .unwrap()
            .wait()
            .await
            .unwrap();
        assert!(matches!(
            c.try_login("ALICE", PASSWORD, 60, 102),
            Err(Error::RateLimited)
        ));
        for _ in 0..2 {
            assert!(matches!(
                clone
                    .try_login("unknown", PASSWORD, 60, 103)
                    .unwrap()
                    .wait()
                    .await,
                Err(Error::Backend(accounts::Error::InvalidCredentials))
            ));
        }
        assert!(matches!(
            c.try_login("UNKNOWN", PASSWORD, 60, 103),
            Err(Error::RateLimited)
        ));
        assert!(matches!(
            c.try_login("another", PASSWORD, 60, 103),
            Err(Error::RateLimited)
        ));
        assert_eq!(
            c.try_authorize(&grant.token, grant.player.player_id, 104)
                .unwrap()
                .wait()
                .await,
            Ok(grant.player)
        );
        worker.shutdown().await.unwrap();
        let worker = Worker::open_with_rate(temp.db(), 1, Duration::from_secs(5), policy)
            .await
            .unwrap();
        // Ephemeral admission counters reset on worker reopen. Session persistence
        // does not: this reaches Store and returns ActiveSession, not RateLimited.
        assert!(matches!(
            worker
                .client()
                .try_login("alice", PASSWORD, 60, 105)
                .unwrap()
                .wait()
                .await,
            Err(Error::Backend(accounts::Error::ActiveSession))
        ));
        worker.shutdown().await.unwrap();
    });
}
#[test]
fn worker_idle_renews_only_correct_owner_and_external_revoke_still_wins() {
    let _serial = TEST_SERIAL.lock().unwrap();
    let temp = Temp::new();
    rt().block_on(async {
        let worker = Worker::open(temp.db(), 2, Duration::from_secs(5))
            .await
            .unwrap();
        let c = worker.client();
        let g = c
            .try_login_with_idle("alice", PASSWORD, 100, 10, 101)
            .unwrap()
            .wait()
            .await
            .unwrap();
        assert!(
            c.try_authorize(&g.token, g.player.player_id, 109)
                .unwrap()
                .wait()
                .await
                .is_ok()
        );
        assert_eq!(
            c.try_authorize(&g.token, g.player.player_id + 1, 118)
                .unwrap()
                .wait()
                .await,
            Err(Error::Backend(accounts::Error::InvalidSession))
        );
        let last: i64 = temp
            .sql()
            .query_row("SELECT last_seen_at FROM sessions", [], |r| r.get(0))
            .unwrap();
        assert_eq!(last, 109);
        assert_eq!(
            c.try_authorize(&g.token, g.player.player_id, 119)
                .unwrap()
                .wait()
                .await,
            Err(Error::Backend(accounts::Error::InvalidSession))
        );
        let g = c
            .try_login_with_idle("alice", PASSWORD, 100, 10, 119)
            .unwrap()
            .wait()
            .await
            .unwrap();
        let mut store = Store::open(&temp.db()).unwrap();
        store.revoke(&admin(), g.player.account_id, 120).unwrap();
        assert_eq!(
            c.try_authorize(&g.token, g.player.player_id, 121)
                .unwrap()
                .wait()
                .await,
            Err(Error::Backend(accounts::Error::InvalidSession))
        );
        worker.shutdown().await.unwrap();
        assert_eq!(temp.active(), 0);
    });
}

#[test]
fn persistence_jobs_are_bounded_cancel_queued_work_retry_committed_events_and_keep_live_lane_separate()
 {
    use accounts::{
        MatchEndReason, MatchExit, MatchFinish, MatchKey, MatchOutcome, MatchParticipant,
        MatchPlayerFinish, MatchStart,
    };
    let _serial = TEST_SERIAL.lock().unwrap();
    let temp = Temp::new();
    rt().block_on(async {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        let mut store = Store::open(&temp.db()).unwrap();
        let a = store.lookup("alice").unwrap();
        let b = store
            .create_account(&admin(), "bob", "Bob", PASSWORD, now)
            .unwrap();
        let begun = MatchStart {
            key: MatchKey {
                boot_epoch: 1,
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
                    account_id: a.account_id,
                    player_id: a.player_id,
                    slot: 0,
                    character_key: 0,
                    team: 0,
                },
                MatchParticipant {
                    account_id: b.account_id,
                    player_id: b.player_id,
                    slot: 1,
                    character_key: 1,
                    team: 3,
                },
            ],
        };
        let worker = Worker::open(temp.db(), 1, Duration::from_secs(2))
            .await
            .unwrap();
        let c = worker.client();
        let resume = pause(&c).await;
        let pending = c
            .try_persist(PersistenceJob::BeginMatch(begun.clone()))
            .unwrap();
        assert!(matches!(
            c.try_persist(PersistenceJob::BeginMatch(begun.clone())),
            Err(Error::Busy)
        ));
        drop(pending);
        resume.send(()).unwrap();
        let ready = pause(&c).await;
        assert_eq!(
            temp.sql()
                .query_row("SELECT count(*) FROM matches", [], |r| r.get::<_, i64>(0))
                .unwrap(),
            0
        );
        ready.send(()).unwrap();
        let pending = c
            .try_persist(PersistenceJob::BeginMatch(begun.clone()))
            .unwrap();
        // Do not consume the successful result: the same durable key recovers it.
        tokio::time::timeout(Duration::from_secs(3), async {
            loop {
                if temp
                    .sql()
                    .query_row("SELECT count(*) FROM matches", [], |r| r.get::<_, i64>(0))
                    .unwrap()
                    == 1
                {
                    break;
                }
                tokio::time::sleep(Duration::from_millis(5)).await;
            }
        })
        .await
        .unwrap();
        drop(pending);
        let PersistenceResult::Match(first) = c
            .try_persist(PersistenceJob::BeginMatch(begun.clone()))
            .unwrap()
            .wait()
            .await
            .unwrap()
        else {
            panic!("wrong persistence result")
        };
        let finish = MatchFinish {
            ended_at: now,
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
        };
        let mut writer = temp.sql();
        let lock = writer
            .transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)
            .unwrap();
        assert!(matches!(
            c.try_persist(PersistenceJob::FinishMatch(
                begun.key.clone(),
                finish.clone()
            ))
            .unwrap()
            .wait()
            .await,
            Err(Error::Backend(accounts::Error::Busy))
        ));
        lock.rollback().unwrap();
        let PersistenceResult::Match(ended) = c
            .try_persist(PersistenceJob::FinishMatch(
                begun.key.clone(),
                finish.clone(),
            ))
            .unwrap()
            .wait()
            .await
            .unwrap()
        else {
            panic!("wrong persistence result")
        };
        assert_eq!(ended.id, first.id);
        let PersistenceResult::Match(duplicate) = c
            .try_persist(PersistenceJob::FinishMatch(begun.key.clone(), finish))
            .unwrap()
            .wait()
            .await
            .unwrap()
        else {
            panic!("wrong persistence result")
        };
        assert_eq!(ended, duplicate);
        let live = Worker::open_live(temp.db(), 1, Duration::from_secs(1))
            .await
            .unwrap();
        assert!(matches!(
            live.client()
                .try_persist(PersistenceJob::BeginMatch(begun.clone()))
                .unwrap()
                .wait()
                .await,
            Err(Error::InvalidInput)
        ));
        live.shutdown().await.unwrap();
        worker.shutdown().await.unwrap();
    });
}

#[test]
fn persistence_preference_queue_expiry_and_revocation_cannot_publish_late_character() {
    let _serial = TEST_SERIAL.lock().unwrap();
    let temp = Temp::new();
    rt().block_on(async {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        let mut store = Store::open(&temp.db()).unwrap();
        let g = store.login("alice", PASSWORD, 60, now).unwrap();
        let worker = Worker::open(temp.db(), 1, Duration::from_millis(30))
            .await
            .unwrap();
        let c = worker.client();
        let token = || Token::parse(g.token.expose_secret()).unwrap();
        let resume = pause(&c).await;
        let pending = c
            .try_persist(PersistenceJob::SelectCharacter {
                token: token(),
                player: g.player.player_id,
                character: 1,
                expected_revision: 0,
            })
            .unwrap();
        tokio::time::sleep(Duration::from_millis(50)).await;
        resume.send(()).unwrap();
        assert!(matches!(pending.wait().await, Err(Error::QueueExpired)));
        let PersistenceResult::Character(initial) = c
            .try_persist(PersistenceJob::Character {
                token: token(),
                player: g.player.player_id,
            })
            .unwrap()
            .wait()
            .await
            .unwrap()
        else {
            panic!("wrong persistence result")
        };
        assert_eq!(initial.revision, 0);
        let resume = pause(&c).await;
        let pending = c
            .try_persist(PersistenceJob::SelectCharacter {
                token: token(),
                player: g.player.player_id,
                character: 1,
                expected_revision: 0,
            })
            .unwrap();
        store
            .revoke(
                &admin(),
                g.player.account_id,
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs() as i64,
            )
            .unwrap();
        resume.send(()).unwrap();
        assert!(matches!(
            pending.wait().await,
            Err(Error::Backend(accounts::Error::InvalidSession))
        ));
        assert_eq!(
            temp.sql()
                .query_row("SELECT count(*) FROM player_preferences", [], |r| r
                    .get::<_, i64>(0))
                .unwrap(),
            0
        );
        worker.shutdown().await.unwrap();
    });
}
