use super::{SocialAction as Action, SocialReply as Reply, *};
use crate::{
    account_bridge::{self as bridge, local::LocalSession},
    account_worker::Worker,
    accounts::{CHAT_WINDOW_CAP, GuildRole, TEST_SERIAL},
};
use std::{net::Ipv4Addr, time::Duration};
use tokio::time::{sleep, timeout};

use bridge::{
    local::tests::{channel, login},
    tests::{PASSWORD, Temp, channel_body, login_body, now, rt, worker},
};

async fn call(client: &Client, player: i64, action: Action) -> Result<Reply> {
    client.try_social(player, action)?.wait().await
}

fn chat(recipient: Option<i64>, message_bytes: usize) -> Action {
    Action::CheckChatPermission {
        recipient,
        message_bytes,
    }
}

#[test]
fn local_bridge_and_worker_cover_friend_block_and_chat_lifecycle() {
    let _serial = TEST_SERIAL.lock().unwrap();
    let (temp, alice, bob) = Temp::new();
    let (a, b) = (alice.player_id, bob.player_id);
    rt().block_on(async {
        let worker = worker(&temp).await;
        let client = worker.client();
        timeout(Duration::from_secs(20), async {
            let mut session = LocalSession::new(client.clone(), 1).unwrap();
            let rows = session
                .login(&login(&["Alice", "Bob"]), 60, None)
                .await
                .unwrap();
            session.redirected(1).unwrap();
            let handles = session.channel(&channel(&rows), 2).await.unwrap();
            let (first, second) = (&handles[0], &handles[1]);
            assert_eq!(first.player(), &alice);
            assert_eq!(second.player(), &bob);
            assert_eq!(
                first.social(Action::SendFriendRequest(a)).await,
                Err(bridge::Error::Backend(Error::Backend(
                    accounts::Error::InvalidInput
                )))
            );
            for rejection in [
                Action::DeclineFriendRequest(a),
                Action::AcceptFriendRequest(a),
            ] {
                assert_eq!(
                    first.social(Action::SendFriendRequest(b)).await.unwrap(),
                    Reply::Changed(true)
                );
                assert_eq!(
                    first.social(Action::AcceptFriendRequest(b)).await.unwrap(),
                    Reply::Changed(false)
                );
                let Reply::FriendRequests(pending) =
                    second.social(Action::ListFriendRequests).await.unwrap()
                else {
                    panic!("wrong projection");
                };
                assert_eq!(pending.len(), 1);
                assert_eq!((pending[0].sender_id, pending[0].recipient_id), (a, b));
                assert_eq!(
                    second.social(rejection).await.unwrap(),
                    Reply::Changed(true)
                );
            }
            assert_eq!(
                first.social(Action::ListFriends).await.unwrap(),
                Reply::Friends(vec![b])
            );
            assert_eq!(
                second.social(Action::ListFriends).await.unwrap(),
                Reply::Friends(vec![a])
            );
            assert_eq!(
                call(&client, a, Action::RemoveFriend(b)).await.unwrap(),
                Reply::Changed(true)
            );
            assert_eq!(
                call(&client, b, Action::SendFriendRequest(a))
                    .await
                    .unwrap(),
                Reply::Changed(true)
            );
            assert_eq!(
                call(&client, b, Action::CancelFriendRequest(a))
                    .await
                    .unwrap(),
                Reply::Changed(true)
            );
            call(&client, a, Action::SendFriendRequest(b))
                .await
                .unwrap();
            assert_eq!(
                second.social(Action::BlockPlayer(a)).await.unwrap(),
                Reply::Changed(true)
            );
            assert_eq!(
                second.social(Action::ListBlocks).await.unwrap(),
                Reply::Blocks(vec![a])
            );
            assert_eq!(
                first.social(Action::ListFriendRequests).await.unwrap(),
                Reply::FriendRequests(vec![])
            );
            assert_eq!(
                call(&client, a, Action::SendFriendRequest(b)).await,
                Err(Error::Backend(accounts::Error::Conflict))
            );
            assert_eq!(
                second.social(chat(Some(a), 11)).await.unwrap(),
                Reply::ChatPermission(ChatPermission::Blocked)
            );
            assert_eq!(
                first.social(Action::CanReceivePublicChat(b)).await.unwrap(),
                Reply::CanReceivePublicChat(false)
            );
            assert_eq!(
                second.social(Action::UnblockPlayer(a)).await.unwrap(),
                Reply::Changed(true)
            );
            assert_eq!(
                first.social(Action::CanReceivePublicChat(b)).await.unwrap(),
                Reply::CanReceivePublicChat(true)
            );
            assert_eq!(
                first.social(chat(None, usize::MAX)).await.unwrap(),
                Reply::ChatPermission(ChatPermission::InvalidLength)
            );
            let mut allowed = 0;
            // Permit a clock boundary during the loop without sleeping for an
            // entire window. Two windows still cannot admit 2*cap+1 sends.
            for _ in 0..=2 * CHAT_WINDOW_CAP {
                match call(&client, a, chat(Some(b), 27)).await.unwrap() {
                    Reply::ChatPermission(ChatPermission::Allowed) => allowed += 1,
                    Reply::ChatPermission(ChatPermission::RateLimited) => break,
                    _ => panic!("unexpected chat permission"),
                }
            }
            assert!((CHAT_WINDOW_CAP..=2 * CHAT_WINDOW_CAP).contains(&allowed));
            assert!(matches!(
                client.try_mute_social_player(a, 0, ChatMuteReason::Spam),
                Err(Error::InvalidInput)
            ));
            let earliest_expiry = now() + 60;
            let expiry = client
                .try_mute_social_player(a, 60, ChatMuteReason::Spam)
                .unwrap()
                .wait()
                .await
                .unwrap();
            assert!((earliest_expiry..=now() + 60).contains(&expiry));
            assert_eq!(
                first.social(chat(Some(b), 33)).await.unwrap(),
                Reply::ChatPermission(ChatPermission::Muted)
            );
            session.finish().await.unwrap();
            assert!(matches!(
                first.social(Action::BlockPlayer(b)).await,
                Err(bridge::Error::Stale)
            ));
            assert_eq!(
                call(&client, a, Action::ListBlocks).await.unwrap(),
                Reply::Blocks(vec![])
            );
        })
        .await
        .expect("worker/bridge deadlock");
        worker.shutdown().await.unwrap();
    });
    let db = rusqlite::Connection::open(temp.db()).unwrap();
    let events: Vec<(String, i64)> = db
        .prepare("SELECT reason, message_bytes FROM chat_abuse_events ORDER BY rowid")
        .unwrap()
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
        .unwrap()
        .collect::<rusqlite::Result<_>>()
        .unwrap();
    assert_eq!(
        events,
        [
            ("blocked".into(), 11),
            ("invalid_length".into(), i64::MAX),
            ("rate_limited".into(), 27),
            ("muted".into(), 33)
        ]
    );
}

#[test]
fn launcher_bridge_binds_actor_and_worker_persists_guild_and_ranking() {
    let _serial = TEST_SERIAL.lock().unwrap();
    let (temp, alice, bob) = Temp::new();
    let (a, b) = (alice.player_id, bob.player_id);
    rt().block_on(async {
        let worker = worker(&temp).await;
        let client = worker.client();
        let live = Worker::open_live(temp.db(), 4, Duration::from_secs(3))
            .await
            .unwrap();
        let bridge = bridge::Bridge::new(client.clone(), 2, Duration::from_secs(30))
            .unwrap()
            .with_live_authorization(live.client());
        let grant = client
            .try_login("Alice", PASSWORD, 60, now())
            .unwrap()
            .wait()
            .await
            .unwrap();
        let peer = Ipv4Addr::LOCALHOST;
        let launch = bridge.issue(&grant.token, a, peer).await.unwrap();
        let body = login_body(launch.token().expose_secret(), peer);
        launch.delivered();
        let login = bridge.login(&body, peer, 1).await.unwrap();
        let body = channel_body(&alice, login.echo);
        login.redirected().unwrap();
        let mut channel = bridge.channel(&body, peer, 2).await.unwrap();
        assert!(matches!(
            client.try_social(a, Action::CreateGuild("x".repeat(65))),
            Err(Error::InvalidInput)
        ));
        assert_eq!(
            channel.social(Action::CreateGuild("a?".into())).await,
            Err(bridge::Error::Backend(Error::Backend(
                accounts::Error::InvalidInput
            )))
        );
        channel.player = bob.clone();
        assert_eq!(
            channel
                .social(Action::CreateGuild("Wrong Actor".into()))
                .await,
            Err(bridge::Error::Denied)
        );
        channel.player = alice.clone();
        let Reply::Guild(guild) = channel
            .social(Action::CreateGuild("  Alpha   Team  ".into()))
            .await
            .unwrap()
        else {
            panic!("wrong projection");
        };
        assert_eq!(guild.name, "Alpha Team");
        assert_eq!(guild.leader_player_id, a);
        let id = guild.id;
        assert_eq!(
            call(&client, b, Action::JoinGuild(id)).await.unwrap(),
            Reply::Changed(true)
        );
        assert_eq!(
            call(&client, b, Action::LeaveGuild(id)).await.unwrap(),
            Reply::Changed(true)
        );
        assert_eq!(
            call(&client, b, Action::JoinGuild(id)).await.unwrap(),
            Reply::Changed(true)
        );
        assert_eq!(
            call(&client, b, Action::DisbandGuild(id)).await,
            Err(Error::Backend(accounts::Error::Conflict))
        );
        assert_eq!(
            call(
                &client,
                b,
                Action::TransferGuildLeadership {
                    guild_id: id,
                    successor: a
                }
            )
            .await,
            Err(Error::Backend(accounts::Error::Conflict))
        );
        assert_eq!(
            channel
                .social(Action::SetGuildOfficer {
                    guild_id: id,
                    player: b,
                    officer: true
                })
                .await
                .unwrap(),
            Reply::Changed(true)
        );
        let Reply::GuildMembers(members) = call(&client, b, Action::ListGuildMembers(id))
            .await
            .unwrap()
        else {
            panic!("wrong projection");
        };
        assert_eq!(
            members
                .iter()
                .map(|member| (member.player_id, member.role))
                .collect::<Vec<_>>(),
            [(a, GuildRole::Leader), (b, GuildRole::Officer)]
        );
        assert_eq!(
            channel
                .social(Action::TransferGuildLeadership {
                    guild_id: id,
                    successor: b
                })
                .await
                .unwrap(),
            Reply::LeadershipTransferred
        );
        assert_eq!(
            channel.social(Action::DisbandGuild(id)).await,
            Err(bridge::Error::Backend(Error::Backend(
                accounts::Error::Conflict
            )))
        );
        assert_eq!(
            live.client()
                .try_social(a, Action::ListFriends)
                .unwrap()
                .wait()
                .await,
            Err(Error::InvalidInput)
        );
        client
            .try_logout_current(&grant.token, a)
            .unwrap()
            .wait()
            .await
            .unwrap();
        assert_eq!(
            channel.social(Action::LeaveGuild(id)).await,
            Err(bridge::Error::Backend(Error::Backend(
                accounts::Error::InvalidSession
            )))
        );
        live.shutdown().await.unwrap();
        worker.shutdown().await.unwrap();
        let reopened = Worker::open(temp.db(), 4, Duration::from_secs(3))
            .await
            .unwrap();
        let client = reopened.client();
        let Reply::GuildMembers(members) = call(&client, b, Action::ListGuildMembers(id))
            .await
            .unwrap()
        else {
            panic!("wrong projection");
        };
        assert_eq!(
            members
                .iter()
                .map(|member| (member.player_id, member.role))
                .collect::<Vec<_>>(),
            [(b, GuildRole::Leader), (a, GuildRole::Member)]
        );
        assert_eq!(
            call(&client, b, Action::DisbandGuild(id)).await.unwrap(),
            Reply::Changed(true)
        );
        assert_eq!(
            call(&client, b, Action::ListGuildMembers(id))
                .await
                .unwrap(),
            Reply::GuildMembers(vec![])
        );
        let page = |offset, limit| Action::Ranking { offset, limit };
        assert_eq!(
            call(&client, b, page(0, 0)).await,
            Err(Error::Backend(accounts::Error::InvalidInput))
        );
        let Reply::Ranking(all) = call(&client, b, page(0, 2)).await.unwrap() else {
            panic!("wrong projection");
        };
        assert_eq!(
            all.iter().map(|entry| entry.player_id).collect::<Vec<_>>(),
            [a, b]
        );
        for (offset, entry) in all.into_iter().enumerate() {
            let expected = Reply::Ranking(vec![entry]);
            assert_eq!(
                call(&client, b, page(offset as u32, 1)).await.unwrap(),
                expected
            );
            assert_eq!(
                call(&client, b, page(offset as u32, 1)).await.unwrap(),
                expected
            );
        }
        assert_eq!(
            call(&client, b, page(2, 1)).await.unwrap(),
            Reply::Ranking(vec![])
        );
        reopened.shutdown().await.unwrap();
    });
}

#[test]
fn social_admission_is_bounded_and_cancelled_or_expired_writes_do_not_run() {
    let _serial = TEST_SERIAL.lock().unwrap();
    let (temp, alice, bob) = Temp::new();
    let (a, b) = (alice.player_id, bob.player_id);
    rt().block_on(async {
        let worker = Worker::open(temp.db(), 2, Duration::from_millis(250))
            .await
            .unwrap();
        let client = worker.client();
        timeout(Duration::from_secs(5), async {
            let (entered, waiting) = oneshot::channel();
            let (resume, paused) = std::sync::mpsc::channel();
            client
                .submit(Command::Pause {
                    entered,
                    resume: paused,
                })
                .unwrap();
            waiting.await.unwrap();
            let cancelled = client.try_social(b, Action::BlockPlayer(a)).unwrap();
            let barrier = client.try_social(b, Action::ListBlocks).unwrap();
            assert!(matches!(
                client.try_social(a, Action::ListFriends),
                Err(Error::Busy)
            ));
            assert!(matches!(
                client.try_mute_social_player(a, 60, ChatMuteReason::Spam),
                Err(Error::Busy)
            ));
            drop(cancelled);
            resume.send(()).unwrap();
            assert_eq!(barrier.wait().await.unwrap(), Reply::Blocks(vec![]));

            let (entered, waiting) = oneshot::channel();
            let (resume, paused) = std::sync::mpsc::channel();
            client
                .submit(Command::Pause {
                    entered,
                    resume: paused,
                })
                .unwrap();
            waiting.await.unwrap();
            let expired = client.try_social(a, Action::BlockPlayer(b)).unwrap();
            sleep(Duration::from_millis(300)).await;
            resume.send(()).unwrap();
            assert_eq!(expired.wait().await, Err(Error::QueueExpired));
            assert_eq!(
                call(&client, a, Action::ListBlocks).await.unwrap(),
                Reply::Blocks(vec![])
            );
        })
        .await
        .expect("worker deadlock");
        worker.shutdown().await.unwrap();
        assert!(matches!(
            client.try_social(a, Action::ListFriends),
            Err(Error::Closed)
        ));
    });
}
