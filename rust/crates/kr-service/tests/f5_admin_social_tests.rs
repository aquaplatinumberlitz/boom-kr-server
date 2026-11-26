use kr_service::accounts::{self, ChatPermission, Store};
use serde_json::{Value, json};
use std::{
    fs,
    io::Write,
    os::unix::fs::DirBuilderExt,
    path::PathBuf,
    process::{Command, Output, Stdio},
};

struct Fixture {
    dir: PathBuf,
    db: PathBuf,
}

impl Fixture {
    fn new() -> Self {
        let mut seed = [0; 8];
        getrandom::fill(&mut seed).unwrap();
        let nonce = u64::from_ne_bytes(seed);
        let dir = std::env::temp_dir().join(format!("kr-admin-social-{nonce:016x}"));
        fs::DirBuilder::new().mode(0o700).create(&dir).unwrap();
        let fixture = Self {
            db: dir.join("accounts.sqlite"),
            dir,
        };
        assert_eq!(fixture.success(json!({"op":"init"})), json!({"schema":9}));
        fixture
    }

    fn player(&self, name: &str) -> (i64, i64) {
        let mut seed = [0; 16];
        getrandom::fill(&mut seed).unwrap();
        let password = format!("{:032x}", u128::from_ne_bytes(seed));
        let created = self.success(json!({
            "op":"create","login":name,"name":name,"password":password
        }));
        let player = &created["player"];
        (
            player["player_id"].as_i64().unwrap(),
            player["account_id"].as_i64().unwrap(),
        )
    }

    fn run(&self, command: Value) -> Output {
        let request = json!({"actor":"on-call","reason":"social review","command":command});
        let mut child = Command::new(env!("CARGO_BIN_EXE_kr-admin"))
            .arg(&self.db)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        child
            .stdin
            .take()
            .unwrap()
            .write_all(&serde_json::to_vec(&request).unwrap())
            .unwrap();
        child.wait_with_output().unwrap()
    }

    fn success(&self, command: Value) -> Value {
        let op = command["op"].as_str().unwrap().to_owned();
        let run = self.run(command);
        assert!(
            run.status.success(),
            "{op}: {}",
            String::from_utf8_lossy(&run.stderr)
        );
        serde_json::from_slice(&run.stdout).unwrap()
    }

    fn failure(&self, command: Value, error: &str) {
        let run = self.run(command.clone());
        assert_eq!(run.status.code(), Some(1), "{command}");
        assert_eq!(
            String::from_utf8_lossy(&run.stderr),
            format!("kr-admin: {error}\n"),
            "{command}"
        );
        assert!(run.stdout.is_empty(), "{command}");
    }

    fn audit(&self, operation: &str, result: &str) -> Vec<Value> {
        let mut reply = self.success(json!({
            "op":"audit-list","filter":{"operation":operation,"result":result}
        }));
        serde_json::from_value(reply["audit"].take()).unwrap()
    }

    fn assert_audit(
        &self,
        operation: &str,
        result: &str,
        kind: &str,
        id: i64,
        account: Option<i64>,
    ) {
        let rows = self.audit(operation, result);
        assert_eq!(rows.len(), 1, "{operation}: {rows:?}");
        let row = &rows[0];
        assert_eq!(row["actor"], "on-call");
        assert_eq!(row["reason"], "social review");
        assert_eq!(row["target_kind"], kind);
        assert_eq!(row["target_id"], id.to_string());
        assert_eq!(row["account_id"], json!(account));
        assert!(row["at_utc"].as_i64().unwrap() > 0);
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.dir);
    }
}

#[test]
fn friend_block_and_ranking_readbacks_use_the_persistent_social_store() {
    let f = Fixture::new();
    let (alice, _) = f.player("Alice");
    let (betty, _) = f.player("Betty");
    let (chris, chris_account) = f.player("Chris");
    {
        let mut s = Store::open_existing(&f.db).unwrap();
        assert!(
            s.send_friend_request(alice, betty, accounts::utc().unwrap())
                .unwrap()
        );
        assert!(
            s.accept_friend_request(betty, alice, accounts::utc().unwrap())
                .unwrap()
        );
        assert!(
            s.block_player(alice, chris, accounts::utc().unwrap())
                .unwrap()
        );
    }
    assert_eq!(
        f.success(json!({"op":"social-friend-list","player_id":alice})),
        json!({"friends":[betty]})
    );
    assert_eq!(
        f.success(json!({"op":"social-block-list","player_id":alice})),
        json!({"blocks":[chris]})
    );
    f.success(json!({"op":"disable","account_id":chris_account}));
    assert_eq!(
        f.success(json!({"op":"social-ranking-list","offset":1,"limit":2})),
        json!({"ranking":[{"player_id":betty,"display_name":"Betty","level":1,"exp":0,"wins":0}]})
    );
    f.assert_audit("social_friend_list", "ok", "player", alice, None);
    f.assert_audit("social_block_list", "ok", "player", alice, None);
    let ranking = f.audit("social_ranking_list", "ok");
    assert_eq!(ranking.len(), 1);
    assert_eq!(ranking[0]["target_kind"], "service");
    assert_eq!(ranking[0]["target_id"], "ranking");
}

#[test]
fn mute_clear_preserves_abuse_history_and_both_commands_are_audited() {
    let f = Fixture::new();
    let (alice, account) = f.player("Alice");
    let (betty, _) = f.player("Betty");
    let before = accounts::utc().unwrap();
    let set = f.success(json!({
        "op":"social-mute-set","player_id":alice,"duration_seconds":3600,"mute_reason":"spam"
    }));
    let expires = set["expires_at"].as_i64().unwrap();
    assert!((before + 3600..=accounts::utc().unwrap() + 3600).contains(&expires));
    let mut s = Store::open_existing(&f.db).unwrap();
    assert_eq!(
        s.check_chat_permission(alice, None, 12, accounts::utc().unwrap())
            .unwrap(),
        ChatPermission::Muted
    );
    assert_eq!(
        s.check_chat_permission(alice, Some(betty), 24, accounts::utc().unwrap())
            .unwrap(),
        ChatPermission::Muted
    );
    assert_eq!(
        s.check_chat_permission(betty, None, 0, accounts::utc().unwrap())
            .unwrap(),
        ChatPermission::InvalidLength
    );
    assert_eq!(
        f.success(json!({"op":"social-mute-clear","player_id":alice})),
        json!({"cleared":1})
    );
    assert_eq!(
        s.check_chat_permission(alice, None, 12, accounts::utc().unwrap())
            .unwrap(),
        ChatPermission::Allowed
    );
    let events = f.success(json!({"op":"social-abuse-list","sender_id":alice,"limit":1}));
    let rows = events["events"].as_array().unwrap();
    assert_eq!(rows.len(), 1);
    let row = &rows[0];
    assert_eq!(
        *row,
        json!({"id":row["id"],"sender_id":alice,"recipient_id":betty,"reason":"muted",
            "message_bytes":24,"accepted_in_window":0,"at_utc":row["at_utc"]})
    );
    assert!(row["at_utc"].as_i64().unwrap() >= before);
    let page = f.success(json!({
        "op":"social-abuse-list","sender_id":alice,"before_id":row["id"],"limit":2
    }));
    let previous = page["events"].as_array().unwrap();
    assert_eq!(previous.len(), 1);
    assert!(previous[0]["recipient_id"].is_null());
    assert_eq!(previous[0]["message_bytes"], 12);
    let all = f.success(json!({"op":"social-abuse-list","limit":3}));
    assert_eq!(all["events"].as_array().unwrap().len(), 3);
    assert_eq!(all["events"][0]["sender_id"], betty);
    f.assert_audit("social_mute_set", "ok", "player", alice, Some(account));
    f.assert_audit("social_mute_clear", "ok", "player", alice, Some(account));
}

#[test]
fn guild_lifecycle_preserves_the_leader_and_audits_each_mutation() {
    let f = Fixture::new();
    let (leader, leader_account) = f.player("Alice");
    let (member, member_account) = f.player("Betty");
    let created = f.success(json!({
        "op":"social-guild-create","leader_player_id":leader,"name":"  Blue   Team "
    }));
    let guild = created["guild"]["id"].as_i64().unwrap();
    assert_eq!(created["guild"]["name"], "Blue Team");
    assert_eq!(created["guild"]["leader_player_id"], leader);
    let get = || f.success(json!({"op":"social-guild-get","guild_id":guild}));
    let initial = get();
    assert_eq!(initial["guild"], created["guild"]);
    assert_eq!(
        initial["members"],
        json!([{"player_id":leader,"role":"leader","joined_at":created["guild"]["created_at"]}])
    );
    assert_eq!(
        f.success(json!({"op":"social-guild-member-add","guild_id":guild,"player_id":member})),
        json!({"changed":true})
    );
    let updated = get();
    assert_eq!(updated["members"].as_array().unwrap().len(), 2);
    assert_eq!(updated["members"][1]["player_id"], member);
    assert_eq!(updated["members"][1]["role"], "member");
    f.failure(
        json!({"op":"social-guild-member-remove","guild_id":guild,"player_id":leader}),
        "Conflict",
    );
    assert_eq!(
        f.success(json!({"op":"social-guild-member-remove","guild_id":guild,"player_id":member})),
        json!({"changed":true})
    );
    assert_eq!(get(), initial);
    // The local operator can disband even when the original leader cannot log in.
    f.success(json!({"op":"disable","account_id":leader_account}));
    assert_eq!(
        f.success(json!({"op":"social-guild-disband","guild_id":guild})),
        json!({"ok":true})
    );
    f.failure(
        json!({"op":"social-guild-get","guild_id":guild}),
        "InvalidInput",
    );
    f.assert_audit(
        "social_guild_create",
        "ok",
        "player",
        leader,
        Some(leader_account),
    );
    f.assert_audit(
        "social_guild_member_add",
        "ok",
        "guild",
        guild,
        Some(member_account),
    );
    f.assert_audit(
        "social_guild_member_remove",
        "ok",
        "guild",
        guild,
        Some(member_account),
    );
    f.assert_audit(
        "social_guild_member_remove",
        "Conflict",
        "guild",
        guild,
        None,
    );
    f.assert_audit("social_guild_disband", "ok", "guild", guild, None);
}

#[test]
fn malformed_requests_fail_closed_and_parsed_denials_are_audited() {
    let f = Fixture::new();
    let (player, _) = f.player("Alice");
    for command in [
        json!({"op":"social-friend-list","player_id":0}),
        json!({"op":"social-block-list","player_id":player,"account_id":player}),
        json!({"op":"social-mute-set","player_id":player,"duration_seconds":0,"mute_reason":"spam"}),
        json!({"op":"social-mute-set","player_id":player,"duration_seconds":3600,"mute_reason":"other"}),
        json!({"op":"social-mute-clear","player_id":-1}),
        json!({"op":"social-abuse-list","limit":101}),
        json!({"op":"social-abuse-list","limit":1,"before_id":0}),
        json!({"op":"social-guild-create","leader_player_id":player,"name":"a\nbc"}),
        json!({"op":"social-guild-member-add","guild_id":1,"player_id":0}),
        json!({"op":"social-guild-disband","guild_id":1}),
        json!({"op":"social-ranking-list","offset":10001,"limit":1}),
        json!({"op":"social-ranking-list","offset":0,"limit":0}),
    ] {
        f.failure(command, "InvalidInput");
    }
    f.assert_audit("social_mute_set", "InvalidInput", "player", player, None);
    f.assert_audit(
        "social_guild_create",
        "InvalidInput",
        "player",
        player,
        None,
    );
    assert!(f.audit("social_block_list", "InvalidInput").is_empty());
    assert!(f.audit("social_mute_set", "ok").is_empty());
    let mut s = Store::open_existing(&f.db).unwrap();
    assert_eq!(
        s.check_chat_permission(player, None, 12, accounts::utc().unwrap())
            .unwrap(),
        ChatPermission::Allowed
    );
}

#[test]
fn failed_success_audit_rolls_back_the_guild_and_its_members() {
    let f = Fixture::new();
    let (leader, account) = f.player("Alice");
    let sql = rusqlite::Connection::open(&f.db).unwrap();
    sql.execute_batch(
        "CREATE TRIGGER refuse_success BEFORE INSERT ON admin_audit
         WHEN NEW.operation='social_guild_create' AND NEW.result='ok'
         BEGIN SELECT RAISE(ABORT, 'test audit failure'); END;",
    )
    .unwrap();
    let create = json!({"op":"social-guild-create","leader_player_id":leader,"name":"Blue Team"});
    f.failure(create.clone(), "Conflict");
    f.assert_audit("social_guild_create", "Conflict", "player", leader, None);
    assert!(f.audit("social_guild_create", "ok").is_empty());
    sql.execute_batch("DROP TRIGGER refuse_success").unwrap();
    // Both the name and the leader's membership must still be available.
    f.success(create);
    f.assert_audit("social_guild_create", "ok", "player", leader, Some(account));
}
