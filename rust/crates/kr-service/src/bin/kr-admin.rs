//! Local trusted-administrator interface. JSON credentials are stdin-only.
#![forbid(unsafe_code)]
use kr_service::accounts::{Admin, Error, SCHEMA_VERSION, Store, Token};
use serde::Deserialize;
use serde_json::{Value, json};
use std::{
    io::{self, Read},
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
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
    #[serde(rename = "revoke")]
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
}
fn run() -> Result<Value, Error> {
    let args: Vec<_> = std::env::args_os().collect();
    if args.len() != 2 {
        return Err(Error::InvalidInput);
    }
    let mut input = Zeroizing::new(Vec::new());
    io::stdin()
        .lock()
        .take(4097)
        .read_to_end(&mut input)
        .map_err(|_| Error::InvalidInput)?;
    if input.len() > 4096 {
        return Err(Error::InvalidInput);
    }
    let r: Request = serde_json::from_slice(&input).map_err(|_| Error::InvalidInput)?;
    let admin = Admin {
        actor: &r.actor,
        reason: &r.reason,
    };
    let now = i64::try_from(
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| Error::ClockRollback)?
            .as_secs(),
    )
    .map_err(|_| Error::ClockRollback)?;
    let mut s = Store::open(Path::new(&args[1]))?;
    Ok(match r.command {
        Command::Init {} => json!({"schema":SCHEMA_VERSION}),
        Command::BeginInstance {} => json!({"epoch":s.begin_instance(&admin,now)?}),
        Command::Create {
            login,
            name,
            password,
        } => json!({"player":s.create_account(&admin,&login,&name,password.0.as_bytes(),now)?}),
        Command::Lookup { login } => json!({"player":s.lookup(&login)?}),
        Command::Login {
            login,
            password,
            ttl_seconds,
        } => {
            let g = s.login(&login, password.0.as_bytes(), ttl_seconds, now)?;
            json!({"player":g.player,"expires_at":g.expires_at,"token":g.token.expose_secret()})
        }
        Command::Authorize { token, player_id } => {
            json!({"player":s.authorize(&Token::parse(&token.0)?,player_id,now)?})
        }
        Command::Revoke { account_id } => {
            s.revoke(&admin, account_id, now)?;
            json!({"ok":true})
        }
        Command::Disable { account_id } => {
            s.set_disabled(&admin, account_id, true, now)?;
            json!({"ok":true})
        }
        Command::Enable { account_id } => {
            s.set_disabled(&admin, account_id, false, now)?;
            json!({"ok":true})
        }
        Command::Rename {
            account_id,
            name,
            expected_version,
        } => json!({"player":s.rename_player(&admin,account_id,&name,expected_version,now)?}),
        Command::Reset {
            account_id,
            password,
        } => {
            s.reset_password(&admin, account_id, password.0.as_bytes(), now)?;
            json!({"ok":true})
        }
    })
}
fn main() {
    match run() {
        Ok(value) => println!("{value}"),
        Err(e) => {
            eprintln!("kr-admin: {e}");
            std::process::exit(1);
        }
    }
}
