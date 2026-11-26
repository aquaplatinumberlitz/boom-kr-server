//! Local trusted-administrator interface. JSON credentials are stdin-only.
#![forbid(unsafe_code)]
use kr_service::{
    accounts::{
        self, Admin, AuditFilter, ChatMuteReason, Error, HistoryFilter, ItemGrant, SCHEMA_VERSION,
        Store, Token,
    },
    operations::{self, LiveCommand, RoomTarget},
};
use serde::Deserialize;
use serde_json::{Value, json};
use std::{
    ffi::OsString,
    io::{self, Read},
    path::Path,
};
use zeroize::Zeroizing;
struct Secret(Zeroizing<String>);
impl<'de> Deserialize<'de> for Secret {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        String::deserialize(d).map(|s| Self(Zeroizing::new(s)))
    }
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Request {
    actor: String,
    reason: String,
    command: Command,
}
#[derive(Deserialize)]
#[serde(tag = "op", deny_unknown_fields)]
enum Command {
    #[serde(rename = "audit-list")]
    AuditList {
        #[serde(default)]
        filter: AuditFilter,
    },
    #[serde(rename = "list-rooms")]
    ListRooms {},
    #[serde(rename = "close-room")]
    CloseRoom { target: RoomTarget },
    #[serde(rename = "f2-runtime")]
    F2Runtime {
        target: RoomTarget,
        command: operations::F2Command,
    },
    #[serde(rename = "delete")]
    Delete { account_id: i64 },
    #[serde(rename = "recovery-reset")]
    RecoveryReset { account_id: i64 },
    #[serde(rename = "match-history-account")]
    MatchHistoryAccount { account_id: i64, limit: u8 },
    #[serde(rename = "match-history-room")]
    MatchHistoryRoom { room_id: u32, limit: u8 },
    #[serde(rename = "init")]
    Init {},
    #[serde(rename = "begin-instance")]
    BeginInstance {},
    #[serde(rename = "create")]
    Create {
        login: String,
        name: String,
        password: Secret,
    },
    #[serde(rename = "lookup")]
    Lookup { login: String },
    #[serde(rename = "login")]
    Login {
        login: String,
        password: Secret,
        ttl_seconds: i64,
    },
    #[serde(rename = "authorize")]
    Authorize { token: Secret, player_id: i64 },
    #[serde(rename = "revoke", alias = "kick")]
    Revoke { account_id: i64 },
    #[serde(rename = "disable")]
    Disable { account_id: i64 },
    #[serde(rename = "enable")]
    Enable { account_id: i64 },
    #[serde(rename = "rename")]
    Rename {
        account_id: i64,
        name: String,
        expected_version: i64,
    },
    #[serde(rename = "password-reset")]
    Reset { account_id: i64, password: Secret },
    #[serde(rename = "profile-get")]
    ProfileGet { player_id: i64 },
    #[serde(rename = "ledger-list")]
    LedgerList { player_id: i64, limit: u8 },
    #[serde(rename = "inventory-list")]
    InventoryList { player_id: i64 },
    #[serde(rename = "admin-grant-lucci")]
    AdminGrantLucci {
        player_id: i64,
        /// Positive grant or negative correction; never below a zero balance.
        lucci_delta: i64,
        idempotency_key: String,
    },
    #[serde(rename = "admin-grant-item")]
    AdminGrantItem {
        player_id: i64,
        sku_id: String,
        quantity: u32,
        grant: ItemGrant,
        idempotency_key: String,
    },
    #[serde(rename = "social-friend-list")]
    SocialFriendList { player_id: i64 },
    #[serde(rename = "social-block-list")]
    SocialBlockList { player_id: i64 },
    #[serde(rename = "social-mute-set")]
    SocialMuteSet {
        player_id: i64,
        duration_seconds: u32,
        mute_reason: ChatMuteReason,
    },
    #[serde(rename = "social-mute-clear")]
    SocialMuteClear { player_id: i64 },
    #[serde(rename = "social-abuse-list")]
    SocialAbuseList {
        sender_id: Option<i64>,
        before_id: Option<i64>,
        limit: u8,
    },
    #[serde(rename = "social-guild-create")]
    SocialGuildCreate { leader_player_id: i64, name: String },
    #[serde(rename = "social-guild-get")]
    SocialGuildGet { guild_id: i64 },
    #[serde(rename = "social-guild-member-add")]
    SocialGuildMemberAdd { guild_id: i64, player_id: i64 },
    #[serde(rename = "social-guild-member-remove")]
    SocialGuildMemberRemove { guild_id: i64, player_id: i64 },
    #[serde(rename = "social-guild-disband")]
    SocialGuildDisband { guild_id: i64 },
    #[serde(rename = "social-ranking-list")]
    SocialRankingList { offset: u32, limit: u32 },
}
impl Command {
    fn audit_context(&self) -> (&'static str, &'static str, String, bool) {
        use Command::*;
        let account = |op, id: &i64, recorded| (op, "account", id.to_string(), recorded);
        let player = |op, id: &i64, recorded| (op, "player", id.to_string(), recorded);
        let guild = |op, id: &i64, recorded| (op, "guild", id.to_string(), recorded);
        let login = |op, value: &str, recorded| {
            let id = if (3..=32).contains(&value.len())
                && value
                    .bytes()
                    .all(|c| c.is_ascii_alphanumeric() || c == b'_')
            {
                value.to_ascii_lowercase()
            } else {
                "invalid-login".into()
            };
            (op, "login", id, recorded)
        };
        match self {
            MatchHistoryAccount { account_id, .. } => {
                account("match_history_account", account_id, false)
            }
            MatchHistoryRoom { room_id, .. } => {
                ("match_history_room", "room", room_id.to_string(), false)
            }
            Init {} => ("init", "service", "database".into(), false),
            BeginInstance {} => ("begin_instance", "service", "instance".into(), true),
            Create { login: value, .. } => login("create_account", value, true),
            Lookup { login: value } => login("lookup", value, false),
            Login { login: value, .. } => login("login", value, false),
            Authorize { player_id, .. } => ("authorize", "player", player_id.to_string(), false),
            Revoke { account_id } => account("revoke", account_id, true),
            Disable { account_id } => account("disable", account_id, true),
            Enable { account_id } => account("enable", account_id, true),
            Rename { account_id, .. } => account("rename_player", account_id, true),
            Reset { account_id, .. } => account("reset_password", account_id, true),
            RecoveryReset { account_id } => account("reset_recovery", account_id, true),
            Delete { account_id } => account("delete_account", account_id, true),
            AuditList { .. } => ("audit_list", "service", "audit".into(), false),
            ProfileGet { player_id } => player("profile_get", player_id, false),
            LedgerList { player_id, .. } => player("ledger_list", player_id, false),
            InventoryList { player_id } => player("inventory_list", player_id, false),
            AdminGrantLucci { player_id, .. } => player("admin_grant_lucci", player_id, true),
            AdminGrantItem { player_id, .. } => player("admin_grant_item", player_id, true),
            SocialFriendList { player_id } => player("social_friend_list", player_id, false),
            SocialBlockList { player_id } => player("social_block_list", player_id, false),
            SocialMuteSet { player_id, .. } => player("social_mute_set", player_id, true),
            SocialMuteClear { player_id } => player("social_mute_clear", player_id, true),
            SocialAbuseList {
                sender_id: Some(id),
                ..
            } => player("social_abuse_list", id, false),
            SocialAbuseList {
                sender_id: None, ..
            } => ("social_abuse_list", "service", "social_abuse".into(), false),
            SocialGuildCreate {
                leader_player_id, ..
            } => player("social_guild_create", leader_player_id, true),
            SocialGuildGet { guild_id } => guild("social_guild_get", guild_id, false),
            SocialGuildMemberAdd { guild_id, .. } => {
                guild("social_guild_member_add", guild_id, true)
            }
            SocialGuildMemberRemove { guild_id, .. } => {
                guild("social_guild_member_remove", guild_id, true)
            }
            SocialGuildDisband { guild_id } => guild("social_guild_disband", guild_id, true),
            SocialRankingList { .. } => ("social_ranking_list", "service", "ranking".into(), false),
            ListRooms {} | CloseRoom { .. } | F2Runtime { .. } => {
                unreachable!("handled by live admin")
            }
        }
    }
}

fn manage(database: &Path) -> Result<Value, Box<dyn std::error::Error>> {
    let mut input = Zeroizing::new(Vec::new());
    io::stdin()
        .lock()
        .take(4097)
        .read_to_end(&mut input)
        .map_err(|_| Error::InvalidInput)?;
    if input.len() > 4096 {
        return Err(Error::InvalidInput.into());
    }
    let r: Request = serde_json::from_slice(&input).map_err(|_| Error::InvalidInput)?;
    let admin = Admin {
        actor: &r.actor,
        reason: &r.reason,
    };
    admin.validate()?;
    let cmd = match r.command {
        Command::ListRooms {} => {
            return Ok(operations::live_command(
                database,
                &admin,
                LiveCommand::ListRooms,
            )?);
        }
        Command::CloseRoom { target } => {
            return Ok(operations::live_command(
                database,
                &admin,
                LiveCommand::CloseRoom { target },
            )?);
        }
        Command::F2Runtime { target, command } => {
            return Ok(operations::live_command(
                database,
                &admin,
                LiveCommand::F2Runtime { target, command },
            )?);
        }
        command => command,
    };
    let (op, kind, id, recorded) = cmd.audit_context();
    let _instance = matches!(cmd, Command::Init {} | Command::BeginInstance {})
        .then(|| operations::lock_database(database))
        .transpose()?;
    let mut s = if matches!(cmd, Command::Init {}) {
        match std::fs::symlink_metadata(database) {
            Err(error) if error.kind() == io::ErrorKind::NotFound => Store::open(database)?,
            Ok(_) => {
                if let Ok(mut store) = Store::open_existing(database) {
                    store.operator_event(&admin, op, (kind, &id), "Conflict")?;
                }
                return Err(Error::Conflict.into());
            }
            Err(error) => return Err(error.into()),
        }
    } else {
        Store::open_existing(database)?
    };
    let result = execute(&mut s, &admin, cmd);
    match &result {
        // An idempotent replay applied nothing; record the attempt itself.
        Ok(value) if recorded && value["replayed"] == true => {
            s.operator_event(&admin, op, (kind, &id), "replayed")?;
        }
        Ok(_) if !recorded => {
            s.operator_event(&admin, op, (kind, &id), "ok")?;
        }
        Err(e) => {
            s.operator_event(&admin, op, (kind, &id), &e.to_string())?;
        }
        _ => {}
    }
    Ok(result?)
}

fn execute(s: &mut Store, admin: &Admin<'_>, command: Command) -> Result<Value, Error> {
    Ok(match command {
        Command::AuditList { filter } => json!({"audit":s.operator_audit(filter)?}),
        Command::Delete { account_id } => {
            s.delete_account(admin, account_id, None)?;
            json!({"ok":true})
        }
        Command::RecoveryReset { account_id } => {
            let code = s.reset_recovery(admin, account_id, None)?;
            json!({"recovery_code":code.as_str()})
        }
        Command::ListRooms {} | Command::CloseRoom { .. } | Command::F2Runtime { .. } => {
            unreachable!()
        }
        Command::MatchHistoryAccount { account_id, limit } => {
            json!({"matches":s.match_history(admin,HistoryFilter::Account(account_id),limit)?})
        }
        Command::MatchHistoryRoom { room_id, limit } => {
            json!({"matches":s.match_history(admin,HistoryFilter::Room(room_id),limit)?})
        }
        Command::Init {} => json!({"schema":SCHEMA_VERSION}),
        Command::BeginInstance {} => json!({"epoch":s.begin_instance(admin,accounts::utc()?)?}),
        Command::Create {
            login,
            name,
            password,
        } => json!({"player":s.create_account(admin,&login,&name,password.0.as_bytes(),None)?}),
        Command::Lookup { login } => json!({"player":s.lookup(&login)?}),
        Command::Login {
            login,
            password,
            ttl_seconds,
        } => {
            let g = s.login_current(&login, password.0.as_bytes(), ttl_seconds)?;
            json!({"player":g.player,"expires_at":g.expires_at,"token":g.token.expose_secret()})
        }
        Command::Authorize { token, player_id } => {
            json!({"player":s.authorize(&Token::parse(&token.0)?,player_id,accounts::utc()?)?})
        }
        Command::Revoke { account_id } => {
            s.revoke(admin, account_id, None)?;
            json!({"ok":true})
        }
        Command::Disable { account_id } => {
            s.set_disabled(admin, account_id, true, None)?;
            json!({"ok":true})
        }
        Command::Enable { account_id } => {
            s.set_disabled(admin, account_id, false, None)?;
            json!({"ok":true})
        }
        Command::Rename {
            account_id,
            name,
            expected_version,
        } => json!({"player":s.rename_player(admin,account_id,&name,expected_version,None)?}),
        Command::Reset {
            account_id,
            password,
        } => {
            s.reset_password(admin, account_id, password.0.as_bytes(), None)?;
            json!({"ok":true})
        }
        Command::ProfileGet { player_id } => json!({
            "profile": s.get_profile(player_id)?,
            "equipped": s.list_equipped(player_id)?,
        }),
        Command::LedgerList { player_id, limit } => {
            json!({"ledger": s.list_ledger(player_id, limit)?})
        }
        Command::InventoryList { player_id } => {
            json!({"inventory": s.list_inventory(player_id, accounts::utc()?)?})
        }
        Command::AdminGrantLucci {
            player_id,
            lucci_delta,
            idempotency_key,
        } => serde_json::to_value(s.admin_grant_lucci(
            admin,
            player_id,
            lucci_delta,
            &idempotency_key,
            None,
        )?)
        .map_err(|_| Error::Storage)?,
        Command::AdminGrantItem {
            player_id,
            sku_id,
            quantity,
            grant,
            idempotency_key,
        } => serde_json::to_value(s.admin_grant_item(
            admin,
            player_id,
            &sku_id,
            grant,
            quantity,
            &idempotency_key,
            None,
        )?)
        .map_err(|_| Error::Storage)?,
        Command::SocialFriendList { player_id } => {
            json!({"friends": s.list_friends(player_id, accounts::utc()?)?})
        }
        Command::SocialBlockList { player_id } => {
            json!({"blocks": s.list_blocks(player_id, accounts::utc()?)?})
        }
        Command::SocialMuteSet {
            player_id,
            duration_seconds,
            mute_reason,
        } => {
            json!({"expires_at": s.admin_set_social_mute(admin, player_id, duration_seconds, mute_reason)?})
        }
        Command::SocialMuteClear { player_id } => {
            json!({"cleared": s.admin_clear_social_mute(admin, player_id)?})
        }
        Command::SocialAbuseList {
            sender_id,
            before_id,
            limit,
        } => {
            json!({"events": s.list_chat_abuse(sender_id, before_id, limit)?})
        }
        Command::SocialGuildCreate {
            leader_player_id,
            name,
        } => {
            json!({"guild": s.admin_create_guild(admin, leader_player_id, &name)?})
        }
        Command::SocialGuildGet { guild_id } => {
            let (guild, members) = s.get_guild(guild_id)?;
            json!({"guild": guild, "members": members})
        }
        Command::SocialGuildMemberAdd {
            guild_id,
            player_id,
        } => {
            json!({"changed": s.admin_add_guild_member(admin, guild_id, player_id)?})
        }
        Command::SocialGuildMemberRemove {
            guild_id,
            player_id,
        } => {
            json!({"changed": s.admin_remove_guild_member(admin, guild_id, player_id)?})
        }
        Command::SocialGuildDisband { guild_id } => {
            s.admin_disband_guild(admin, guild_id)?;
            json!({"ok": true})
        }
        Command::SocialRankingList { offset, limit } => {
            json!({"ranking": s.ranking(offset, limit)?})
        }
    })
}
fn run(args: &[OsString]) -> Result<Value, Box<dyn std::error::Error>> {
    Ok(match args {
        [flag, dir] if flag == "private-dir" => {
            kr_service::platform::provision_private_dir(Path::new(dir))?;
            json!({"ok":true})
        }
        [flag, run] if flag == "health" => kr_service::monitoring::check(Path::new(run))?,
        [flag, source, destination] if flag == "backup" =>
            json!({"schema": accounts::snapshot(Path::new(source), Path::new(destination))?}),
        [flag, snapshot] if flag == "verify-backup" =>
            json!({"schema": accounts::verify_snapshot(Path::new(snapshot))?}),
        [flag, source, destination] if flag == "restore" => {
            let destination = Path::new(destination);
            let _instance = operations::lock_database(destination)?;
            json!({"schema": accounts::restore(Path::new(source), destination)?})
        }
        [flag, db] if flag == "migrate" => {
            let db = Path::new(db);
            let _instance = operations::lock_database(db)?;
            if !std::fs::symlink_metadata(db)?.is_file() {
                return Err(Error::InvalidInput.into());
            }
            Store::open(db)?;
            json!({"schema": SCHEMA_VERSION})
        }
        [database] => manage(Path::new(database))?,
        _ => return Err("usage: kr-admin DATABASE | backup SOURCE DESTINATION | verify-backup SNAPSHOT | restore SNAPSHOT DATABASE | migrate DATABASE | health RUN_DIRECTORY | private-dir DIRECTORY".into()),
    })
}
fn main() {
    match run(&std::env::args_os().skip(1).collect::<Vec<_>>()) {
        Ok(value) => println!("{value}"),
        Err(e) => {
            eprintln!("kr-admin: {e}");
            std::process::exit(1);
        }
    }
}
