use super::*;
use crate::accounts::tests::Temp;

// These tests exercise the Store / SQLite boundary, with identities already
// resolved by the caller. No password hashing or network adapter is involved.
fn seed_players(db: &Connection, count: i64) {
    for id in 1..=count {
        let label = format!("Player{id}");
        db.execute(
            "INSERT INTO accounts(id,login,password_phc,created_at) VALUES(?1,?2,?3,0)",
            params![id, label.to_ascii_lowercase(), "x".repeat(64)],
        )
        .unwrap();
        db.execute(
            "INSERT INTO players(id,account_id,wire_id,display_name) VALUES(?1,?1,?1,?2)",
            params![id, label],
        )
        .unwrap();
    }
    db.execute(
        "UPDATE runtime_state SET next_wire_id=?1 WHERE id=1",
        [count + 1],
    )
    .unwrap();
}

fn fixture(count: i64) -> (Temp, Store) {
    let temp = Temp::new();
    let store = temp.store();
    seed_players(&store.db, count);
    (temp, store)
}

#[test]
fn friendship_requires_recipient_acceptance_and_blocks_remove_relationships() {
    let (temp, mut s) = fixture(3);
    assert_eq!(s.send_friend_request(1, 1, 100), Err(Error::InvalidInput));
    assert_eq!(s.send_friend_request(1, 2, 100), Ok(true));
    assert_eq!(s.send_friend_request(1, 2, 101), Ok(false));
    assert_eq!(s.send_friend_request(2, 1, 101), Err(Error::Conflict));
    assert_eq!(s.accept_friend_request(1, 2, 102), Ok(false));
    assert_eq!(
        s.list_friend_requests(2, 102).unwrap(),
        [FriendRequest {
            sender_id: 1,
            recipient_id: 2,
            created_at: 100,
        }]
    );
    assert_eq!(s.decline_friend_request(2, 1, 103), Ok(true));
    assert_eq!(s.decline_friend_request(2, 1, 104), Ok(false));
    assert_eq!(s.send_friend_request(1, 2, 105), Ok(true));
    assert_eq!(s.cancel_friend_request(1, 2, 106), Ok(true));
    assert_eq!(s.cancel_friend_request(1, 2, 107), Ok(false));
    assert_eq!(s.send_friend_request(2, 1, 108), Ok(true));
    drop(s);

    let mut s = temp.store();
    assert_eq!(s.accept_friend_request(1, 2, 109), Ok(true));
    assert_eq!(s.accept_friend_request(1, 2, 110), Ok(false));
    assert_eq!(s.list_friends(1, 110).unwrap(), [2]);
    assert_eq!(s.list_friends(2, 110).unwrap(), [1]);
    assert_eq!(s.send_friend_request(1, 2, 111), Ok(false));
    assert_eq!(s.remove_friend(2, 1, 112), Ok(true));
    assert_eq!(s.remove_friend(1, 2, 113), Ok(false));

    assert_eq!(s.send_friend_request(1, 2, 114), Ok(true));
    assert_eq!(s.block_player(2, 1, 115), Ok(true));
    assert_eq!(s.block_player(2, 1, 116), Ok(false));
    assert!(s.list_friend_requests(1, 116).unwrap().is_empty());
    assert_eq!(s.accept_friend_request(2, 1, 117), Err(Error::Conflict));
    assert_eq!(s.send_friend_request(1, 2, 117), Err(Error::Conflict));
    assert_eq!(s.send_friend_request(2, 1, 117), Err(Error::Conflict));
    assert_eq!(s.can_receive_public_chat(1, 2, 118), Ok(false));
    assert_eq!(
        s.check_chat_permission(2, Some(1), 1, 118),
        Ok(ChatPermission::Blocked)
    );
    assert_eq!(
        s.check_chat_permission(1, Some(2), 1, 118),
        Ok(ChatPermission::Blocked)
    );
    assert_eq!(s.list_blocks(2, 119).unwrap(), [1]);
    assert_eq!(s.unblock_player(1, 2, 119), Ok(false));
    assert_eq!(s.unblock_player(2, 1, 120), Ok(true));
    assert!(s.list_friend_requests(2, 121).unwrap().is_empty());
    assert_eq!(s.can_receive_public_chat(1, 2, 121), Ok(true));

    assert_eq!(s.send_friend_request(2, 1, 122), Ok(true));
    assert_eq!(s.accept_friend_request(1, 2, 122), Ok(true));
    assert_eq!(s.block_player(1, 2, 123), Ok(true));
    assert_eq!(s.unblock_player(1, 2, 124), Ok(true));
    assert!(s.list_friends(1, 125).unwrap().is_empty());
    assert!(s.list_friends(2, 125).unwrap().is_empty());
    assert_eq!(
        s.check_chat_permission(1, Some(2), 1, 125),
        Ok(ChatPermission::Allowed)
    );
}

#[test]
fn strict_friend_response_is_recipient_only_and_guild_lookup_is_player_scoped() {
    let (temp, mut store) = fixture(3);
    assert_eq!(store.guild_for_player(1, 100).unwrap(), None);
    assert_eq!(store.send_friend_request(1, 2, 100), Ok(true));
    assert_eq!(
        store.respond_friend_request(3, 1, true, 101),
        Err(Error::Conflict)
    );
    assert_eq!(
        store.respond_friend_request(1, 1, true, 101),
        Err(Error::InvalidInput)
    );
    assert_eq!(store.respond_friend_request(2, 1, true, 102), Ok(true));
    assert_eq!(store.list_friends(1, 102).unwrap(), [2]);
    assert_eq!(store.list_friends(2, 102).unwrap(), [1]);

    let guild = store.create_guild(1, "Social Test", 103).unwrap();
    assert_eq!(
        store.guild_for_player(1, 103).unwrap(),
        Some((
            guild.clone(),
            vec![GuildMember {
                player_id: 1,
                role: GuildRole::Leader,
                joined_at: 103,
            }]
        ))
    );
    assert_eq!(store.guild_for_player(2, 103).unwrap(), None);
    assert_eq!(store.join_guild(2, guild.id, 104), Ok(true));
    assert_eq!(store.guild_for_player(2, 104).unwrap().unwrap().0, guild);
    assert_eq!(store.leave_guild(2, guild.id, 105), Ok(true));
    assert_eq!(store.guild_for_player(2, 105).unwrap(), None);
    assert_eq!(store.disband_guild(1, guild.id, 106), Ok(true));
    assert_eq!(store.guild_for_player(1, 106).unwrap(), None);
    drop(temp);
}

#[test]
fn friend_and_block_caps_and_pending_rollback_hold_across_connections() {
    let extra = FRIEND_CAP + 2;
    let (temp, mut s) = fixture(extra.max(BLOCK_CAP + 2).max(FRIEND_REQUEST_CAP + 2));
    assert_eq!(s.send_friend_request(extra, 1, 100), Ok(true));
    let mut other = temp.store();
    for player in 2..=FRIEND_CAP + 1 {
        assert_eq!(s.send_friend_request(1, player, 100), Ok(true));
        assert_eq!(other.accept_friend_request(player, 1, 100), Ok(true));
    }
    assert_eq!(s.send_friend_request(1, extra, 101), Err(Error::Conflict));
    assert_eq!(
        other.accept_friend_request(1, extra, 101),
        Err(Error::Exhausted)
    );
    assert_eq!(s.list_friend_requests(extra, 101).unwrap().len(), 1);
    assert_eq!(s.cancel_friend_request(extra, 1, 101), Ok(true));
    assert_eq!(s.send_friend_request(extra, 1, 101), Err(Error::Exhausted));
    assert_eq!(s.send_friend_request(1, extra, 101), Err(Error::Exhausted));
    assert_eq!(s.list_friends(1, 101).unwrap().len() as i64, FRIEND_CAP);
    assert_eq!(s.remove_friend(1, 2, 102), Ok(true));
    assert_eq!(s.send_friend_request(extra, 1, 102), Ok(true));
    assert_eq!(other.accept_friend_request(1, extra, 102), Ok(true));

    for ip in 2..=BLOCK_CAP + 1 {
        assert_eq!(s.block_player(1, ip, 103), Ok(true));
    }
    assert_eq!(other.block_player(1, BLOCK_CAP + 1, 103), Ok(false));
    assert_eq!(
        other.block_player(1, BLOCK_CAP + 2, 103),
        Err(Error::Exhausted)
    );
    assert_eq!(s.unblock_player(1, 2, 104), Ok(true));
    assert_eq!(other.block_player(1, BLOCK_CAP + 2, 104), Ok(true));

    // A separate clean account caps both incoming and outgoing pending lists.
    let (_temp, mut s) = fixture(FRIEND_REQUEST_CAP + 2);
    for player in 2..=FRIEND_REQUEST_CAP + 1 {
        assert_eq!(s.send_friend_request(1, player, 105), Ok(true));
    }
    assert_eq!(
        s.send_friend_request(1, FRIEND_REQUEST_CAP + 2, 105),
        Err(Error::Exhausted)
    );
    for player in 2..=FRIEND_REQUEST_CAP + 1 {
        assert_eq!(s.cancel_friend_request(1, player, 106), Ok(true));
        assert_eq!(s.send_friend_request(player, 1, 106), Ok(true));
    }
    assert_eq!(
        s.send_friend_request(FRIEND_REQUEST_CAP + 2, 1, 106),
        Err(Error::Exhausted)
    );
}

#[test]
fn moderation_survives_reopen_audits_denials_and_fails_closed() {
    let (temp, mut s) = fixture(3);
    assert_eq!(
        s.mute_social_player(1, 0, ChatMuteReason::Spam, 100),
        Err(Error::InvalidInput)
    );
    assert_eq!(
        s.mute_social_player(1, 15, ChatMuteReason::Spam, 100),
        Ok(115)
    );
    assert_eq!(
        s.mute_social_player(1, 2, ChatMuteReason::Harassment, 101),
        Ok(103)
    );
    assert_eq!(
        s.check_chat_permission(1, Some(2), 20, 103),
        Ok(ChatPermission::Muted)
    );
    drop(s);
    let mut s = temp.store();
    assert_eq!(
        s.check_chat_permission(1, None, 10, 114),
        Ok(ChatPermission::Muted)
    );
    assert_eq!(
        s.check_chat_permission(1, None, 0, 115),
        Ok(ChatPermission::InvalidLength)
    );
    assert_eq!(
        s.check_chat_permission(1, None, CHAT_MAX_BYTES + 1, 115),
        Ok(ChatPermission::InvalidLength)
    );
    for n in 0..CHAT_WINDOW_CAP {
        let recipient = (n % 2 == 0).then_some(2);
        assert_eq!(
            s.check_chat_permission(1, recipient, CHAT_MAX_BYTES, 115),
            Ok(ChatPermission::Allowed)
        );
    }
    drop(s);
    let mut s = temp.store();
    assert_eq!(
        s.check_chat_permission(1, Some(2), 8, 119),
        Ok(ChatPermission::RateLimited)
    );
    assert_eq!(
        s.check_chat_permission(2, Some(1), 8, 119),
        Ok(ChatPermission::Allowed)
    );
    assert_eq!(
        s.check_chat_permission(1, Some(2), 8, 120),
        Ok(ChatPermission::Allowed)
    );
    assert_eq!(
        s.check_chat_permission(1, None, 8, 119),
        Err(Error::ClockRollback)
    );
    assert_eq!(
        s.check_chat_permission(1, Some(9), 10, 120),
        Err(Error::InvalidCredentials)
    );
    s.db.execute("UPDATE accounts SET disabled=1 WHERE id=2", [])
        .unwrap();
    assert_eq!(
        s.check_chat_permission(2, None, 8, 120),
        Err(Error::InvalidCredentials)
    );
    assert_eq!(
        s.check_chat_permission(1, Some(2), 8, 120),
        Err(Error::InvalidCredentials)
    );

    let mut other = temp.store();
    let lock = other
        .db
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .unwrap();
    assert_eq!(s.check_chat_permission(1, None, 8, 121), Err(Error::Busy));
    drop(lock);

    let mut query = s.db.prepare(
        "SELECT reason,recipient_id,message_bytes,accepted_in_window,at_utc FROM chat_abuse_events ORDER BY id",
    ).unwrap();
    let rows: Vec<(String, Option<i64>, i64, i64, i64)> = query
        .query_map([], |r| {
            Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?))
        })
        .unwrap()
        .collect::<rusqlite::Result<_>>()
        .unwrap();
    assert_eq!(
        rows,
        [
            ("muted".into(), Some(2), 20, 0, 103),
            ("muted".into(), None, 10, 0, 114),
            ("invalid_length".into(), None, 0, 0, 115),
            (
                "invalid_length".into(),
                None,
                CHAT_MAX_BYTES as i64 + 1,
                0,
                115
            ),
            ("rate_limited".into(), Some(2), 8, CHAT_WINDOW_CAP, 119),
        ]
    );
    let history: (i64, i64) =
        s.db.query_row(
            "SELECT count(*),max(expires_at) FROM social_mutes WHERE player_id=1",
            [],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap();
    assert_eq!(history, (2, 115));
}

#[test]
fn guild_lifecycle_keeps_one_leader_and_transfers_authority() {
    let (temp, mut s) = fixture(4);
    assert_eq!(s.create_guild(1, "a\nbc", 100), Err(Error::InvalidInput));
    let guild = s.create_guild(1, "  Blue   Team ", 100).unwrap();
    let id = guild.id;
    assert_eq!(guild.name, "Blue Team");
    assert_eq!(guild.leader_player_id, 1);
    assert_eq!(s.create_guild(4, "blue team", 101), Err(Error::Conflict));
    let other = s.create_guild(4, "Red Team", 102).unwrap();
    assert_eq!(s.create_guild(1, "New Team", 102), Err(Error::Conflict));
    assert_eq!(s.join_guild(2, id, 103), Ok(true));
    assert_eq!(s.join_guild(2, id, 103), Ok(false));
    assert_eq!(s.join_guild(2, other.id, 103), Err(Error::Conflict));
    assert_eq!(s.join_guild(3, id, 104), Ok(true));
    assert_eq!(s.leave_guild(3, id, 104), Ok(true));
    assert_eq!(s.leave_guild(3, id, 104), Ok(false));
    assert_eq!(s.leave_guild_self_service(3, id, 104), Err(Error::Conflict));
    assert_eq!(s.join_guild(3, id, 104), Ok(true));
    assert_eq!(s.leave_guild_self_service(3, id, 104), Ok(true));
    assert_eq!(s.leave_guild(1, id, 105), Err(Error::Conflict));
    assert_eq!(
        s.set_guild_officer(2, id, 1, true, 105),
        Err(Error::Conflict)
    );
    assert_eq!(s.set_guild_officer(1, id, 2, true, 106), Ok(true));
    assert_eq!(s.set_guild_officer(1, id, 2, true, 106), Ok(false));
    assert_eq!(
        s.transfer_guild_leadership(1, id, 4, 106),
        Err(Error::InvalidInput)
    );
    assert_eq!(
        s.list_guild_members(id, 107).unwrap(),
        [
            GuildMember {
                player_id: 1,
                role: GuildRole::Leader,
                joined_at: 100
            },
            GuildMember {
                player_id: 2,
                role: GuildRole::Officer,
                joined_at: 103
            },
        ]
    );
    // Even direct SQL cannot leave a guild with no leader or with two leaders.
    assert!(
        s.db.execute(
            "DELETE FROM guild_members WHERE guild_id=?1 AND player_id=1",
            [id]
        )
        .is_err()
    );
    assert!(
        s.db.execute(
            "UPDATE guild_members SET role='leader' WHERE guild_id=?1 AND player_id=2",
            [id]
        )
        .is_err()
    );
    assert_eq!(s.transfer_guild_leadership(1, id, 2, 108), Ok(()));
    drop(s);

    let mut s = temp.store();
    assert_eq!(
        s.list_guild_members(id, 109).unwrap(),
        [
            GuildMember {
                player_id: 2,
                role: GuildRole::Leader,
                joined_at: 103
            },
            GuildMember {
                player_id: 1,
                role: GuildRole::Member,
                joined_at: 100
            },
        ]
    );
    assert_eq!(s.disband_guild(1, id, 109), Err(Error::Conflict));
    assert_eq!(s.leave_guild(2, id, 109), Err(Error::Conflict));
    assert_eq!(s.disband_guild(2, id, 110), Ok(true));
    assert_eq!(s.disband_guild(2, id, 110), Ok(false));
    assert!(s.list_guild_members(id, 111).unwrap().is_empty());
    assert_eq!(s.join_guild(1, other.id, 111), Ok(true));
    assert_eq!(s.leave_guild(1, id, 111), Ok(false));
    assert_eq!(
        s.create_guild(2, "blue team", 111)
            .unwrap()
            .leader_player_id,
        2
    );
}

#[test]
fn guild_management_add_requires_leadership_and_eligible_target() {
    use GuildLeaderAction::AddMember;
    let (_temp, mut s) = fixture(6);
    let guild = s.create_guild(1, "Blue Team", 100).unwrap().id;
    let other = s.create_guild(4, "Red Team", 100).unwrap().id;
    assert_eq!(s.manage_guild(1, Some(guild), AddMember(2), 101), Ok(()));
    assert_eq!(s.set_guild_officer(1, guild, 2, true, 101), Ok(true));
    for (actor, expected, target, error) in [
        (2, Some(guild), 3, Error::Conflict), // An officer is not a leader.
        (3, None, 5, Error::Conflict),
        (1, Some(other), 3, Error::Conflict),
        (1, None, 2, Error::Conflict),
        (1, None, 4, Error::Conflict),
        (1, None, 7, Error::InvalidInput),
        (1, None, 0, Error::InvalidInput),
    ] {
        assert_eq!(
            s.manage_guild(actor, expected, AddMember(target), 102),
            Err(error)
        );
    }
    assert_eq!(s.block_player(1, 3, 103), Ok(true));
    assert_eq!(s.block_player(5, 1, 103), Ok(true));
    for target in [3, 5] {
        assert_eq!(
            s.manage_guild(1, None, AddMember(target), 104),
            Err(Error::Conflict)
        );
        assert_eq!(s.guild_for_player(target, 104).unwrap(), None);
    }
    s.db.execute("UPDATE accounts SET disabled=1 WHERE id=6", [])
        .unwrap();
    assert_eq!(
        s.manage_guild(1, None, AddMember(6), 105),
        Err(Error::InvalidInput)
    );

    assert_eq!(s.unblock_player(1, 3, 106), Ok(true));
    assert_eq!(s.manage_guild(1, None, AddMember(3), 107), Ok(()));
    assert_eq!(
        s.list_guild_members(guild, 107).unwrap().last(),
        Some(&GuildMember {
            player_id: 3,
            role: GuildRole::Member,
            joined_at: 107,
        })
    );
}

#[test]
fn guild_management_remove_and_transfer_are_scoped_and_persist() {
    use GuildLeaderAction::{RemoveMember, TransferLeadership};
    let (temp, mut s) = fixture(5);
    let guild = s.create_guild(1, "Blue Team", 100).unwrap().id;
    let other = s.create_guild(4, "Red Team", 100).unwrap().id;
    assert_eq!(s.join_guild(2, guild, 101), Ok(true));
    assert_eq!(s.join_guild(3, guild, 101), Ok(true));
    assert_eq!(s.set_guild_officer(1, guild, 2, true, 101), Ok(true));
    for (actor, expected, action, error) in [
        (2, None, RemoveMember(3), Error::Conflict),
        (2, None, TransferLeadership(3), Error::Conflict),
        (1, None, RemoveMember(1), Error::InvalidInput),
        (1, None, TransferLeadership(1), Error::InvalidInput),
        (1, None, RemoveMember(4), Error::Conflict),
        (1, None, RemoveMember(5), Error::Conflict),
        (1, None, TransferLeadership(4), Error::InvalidInput),
        (1, None, TransferLeadership(5), Error::InvalidInput),
        (1, Some(other), TransferLeadership(2), Error::Conflict),
    ] {
        assert_eq!(s.manage_guild(actor, expected, action, 102), Err(error));
    }
    let (before, roster) = s.get_guild(guild).unwrap();
    assert_eq!(before.leader_player_id, 1);
    assert_eq!(roster.len(), 3);

    // A failure on the final write must roll back both role changes.
    s.db.execute_batch(
        "CREATE TEMP TRIGGER fail_leader_update BEFORE UPDATE OF leader_player_id ON guilds
         BEGIN SELECT RAISE(ABORT, 'test failure'); END;",
    )
    .unwrap();
    assert_eq!(
        s.manage_guild(1, Some(guild), TransferLeadership(2), 103),
        Err(Error::Conflict)
    );
    assert_eq!(s.get_guild(guild).unwrap(), (before, roster));
    s.db.execute_batch("DROP TRIGGER fail_leader_update")
        .unwrap();

    assert_eq!(
        s.manage_guild(1, Some(guild), TransferLeadership(2), 103),
        Ok(())
    );
    drop(s);
    let mut s = temp.store();
    let (after, roster) = s.get_guild(guild).unwrap();
    assert_eq!(after.leader_player_id, 2);
    assert_eq!(
        roster
            .iter()
            .map(|m| (m.player_id, m.role, m.joined_at))
            .collect::<Vec<_>>(),
        [
            (2, GuildRole::Leader, 101),
            (1, GuildRole::Member, 100),
            (3, GuildRole::Member, 101),
        ]
    );
    assert_eq!(
        s.manage_guild(1, None, RemoveMember(3), 104),
        Err(Error::Conflict)
    );
    assert_eq!(s.manage_guild(2, None, RemoveMember(1), 104), Ok(()));
    assert_eq!(s.guild_for_player(1, 104).unwrap(), None);
    assert_eq!(s.get_guild(other).unwrap().0.leader_player_id, 4);
}

#[test]
fn ranking_breaks_ties_deterministically_and_paginates() {
    let (_temp, s) = fixture(10);
    s.db.execute_batch(
        "INSERT INTO player_profiles(player_id,level,exp) VALUES
           (1,2,50),(2,2,50),(3,2,50),(4,1,1000),(5,3,1),(6,2,60),
           (8,99,1),(9,99,1),(10,99,1);
         UPDATE accounts SET disabled=1 WHERE id=8;
         UPDATE accounts SET login='~deleted_9' WHERE id=9;
         INSERT INTO guest_accounts(account_id,boot_epoch,expires_at) VALUES(10,1,1000);
         INSERT INTO matches(id,boot_epoch,room_session,room_id,room_generation,round,
           map_code,map_name,started_at,ended_at,state,reason,start_json,finish_json)
           VALUES(1,1,1,1,1,1,0,'Fixture',100,101,'complete','done','{}','{}'),
                 (2,1,1,1,1,2,0,'Fixture',102,103,'cancelled','cancel','{}','{}');
         INSERT INTO match_players VALUES
           (1,1,1,0,0,0,'loss','present',0),(1,2,2,1,0,1,'win','present',0),
           (1,3,3,2,0,1,'win','present',0),(2,1,1,0,0,0,'win','present',0);",
    )
    .unwrap();
    let ids = |offset, limit| {
        s.ranking(offset, limit)
            .unwrap()
            .into_iter()
            .map(|r| r.player_id)
            .collect::<Vec<_>>()
    };
    assert_eq!(ids(0, 100), [5, 6, 2, 3, 1, 4, 7]);
    assert_eq!(ids(0, 3), [5, 6, 2]);
    assert_eq!(ids(3, 3), [3, 1, 4]);
    assert_eq!(ids(6, 3), [7]);
    assert!(ids(7, 3).is_empty());
    assert_eq!(s.ranking(2, 1).unwrap()[0].wins, 1);
    let default = &s.ranking(6, 1).unwrap()[0];
    assert_eq!((default.level, default.exp, default.wins), (1, 0, 0));
    assert_eq!(s.ranking(0, 0), Err(Error::InvalidInput));
    assert_eq!(s.ranking(0, RANKING_PAGE_MAX + 1), Err(Error::InvalidInput));
    assert_eq!(
        s.ranking(RANKING_OFFSET_MAX + 1, 1),
        Err(Error::InvalidInput)
    );
}

#[test]
fn migrations_from_each_old_schema_preserve_player_rows_and_enable_social() {
    assert_eq!(SCHEMA_VERSION, 9);
    for version in 1..SCHEMA_VERSION {
        let temp = Temp::new();
        let path = temp.0.join("accounts.sqlite");
        platform::private_options()
            .write(true)
            .create_new(true)
            .open(&path)
            .unwrap();
        let db = Connection::open(&path).unwrap();
        for (i, sql) in SCRIPTS.iter().take(version as usize).enumerate() {
            db.execute_batch(sql).unwrap();
            db.execute(
                "INSERT INTO schema_migrations VALUES(?1,?2)",
                params![i as i64 + 1, hex(&Sha256::digest(sql.as_bytes())),],
            )
            .unwrap();
        }
        db.pragma_update(None, "application_id", APPLICATION_ID)
            .unwrap();
        db.pragma_update(None, "user_version", version).unwrap();
        seed_players(&db, 2);
        drop(db);
        assert!(matches!(Store::open_existing(&path), Err(Error::Schema)));
        let mut migrated = Store::open(&path).unwrap();
        let (v, _) = Store::validate_schema(&migrated.db).unwrap();
        assert_eq!(v, 9);
        assert_eq!(migrated.lookup("player1").unwrap().display_name, "Player1");
        assert_eq!(migrated.send_friend_request(1, 2, 0), Ok(true));
        assert_eq!(migrated.accept_friend_request(2, 1, 0), Ok(true));
        assert!(
            !migrated
                .db
                .prepare("PRAGMA foreign_key_check")
                .unwrap()
                .exists([])
                .unwrap()
        );
        drop(migrated);
        assert_eq!(
            Store::open_existing(&path)
                .unwrap()
                .list_friends(1, 0)
                .unwrap(),
            [2]
        );
    }
}
