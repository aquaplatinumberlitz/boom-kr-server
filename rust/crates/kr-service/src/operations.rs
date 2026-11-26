//! Local operator controls. The endpoint lives in the private DB directory.
use crate::{
    accounts::{self, Admin, Store},
    platform::{
        self,
        local_ipc::{self, CONTROL_IO, ExchangeError},
    },
    room_registry::{RoomId, RoomKey},
    room_service::Hub,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    fs::File,
    io,
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    sync::watch,
    time::timeout,
};

pub use crate::room_modes::adapter::{Command as F2Command, Event as F2Event, Setup as F2Setup};

mod backup;
pub use backup::BackupSchedule;
pub(crate) use backup::Scheduled;

/// The SQLite instance lock is also used by offline CLI operations.
pub fn lock_database(database: &Path) -> io::Result<File> {
    let parent = database
        .parent()
        .ok_or_else(|| io::Error::other("database parent missing"))?;
    platform::private_dir(parent)?;
    platform::lock_private_file(&database.with_extension("instance.lock"))
}

fn socket_path(database: &Path) -> PathBuf {
    database.with_extension("admin.sock")
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct RoomTarget {
    pub instance: String,
    pub epoch: i64,
    pub room_id: u32,
    pub session: u16,
    pub generation: u64,
}
impl RoomTarget {
    fn key(&self) -> RoomKey {
        RoomKey {
            id: RoomId(self.room_id),
            session: self.session,
            generation: self.generation,
        }
    }
    fn audit_id(&self) -> String {
        format!(
            "{}/{}/{}/{}/{}",
            self.instance, self.epoch, self.room_id, self.session, self.generation
        )
    }
}
#[derive(Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case", deny_unknown_fields)]
pub enum LiveCommand {
    ListRooms,
    CloseRoom {
        target: RoomTarget,
    },
    F2Runtime {
        target: RoomTarget,
        command: F2Command,
    },
}
impl LiveCommand {
    fn audit_fields(&self) -> (&'static str, &'static str, String) {
        match self {
            Self::ListRooms => ("list_rooms", "service", "rooms".into()),
            Self::CloseRoom { target } | Self::F2Runtime { target, .. } => {
                let op = if matches!(self, Self::CloseRoom { .. }) {
                    "close_room"
                } else {
                    "f2_runtime"
                };
                if target.instance.len() != 32
                    || !target.instance.bytes().all(|b| b.is_ascii_hexdigit())
                {
                    (op, "room", "invalid-instance".into())
                } else {
                    (op, "room", target.audit_id())
                }
            }
        }
    }
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Request {
    actor: String,
    reason: String,
    command: LiveCommand,
}

/// A lost reply after delivery is unknown; the audit is the source of truth.
pub fn live_command(db: &Path, admin: &Admin<'_>, command: LiveCommand) -> io::Result<Value> {
    admin.validate().map_err(io::Error::other)?;
    let request = Request {
        actor: admin.actor.into(),
        reason: admin.reason.into(),
        command,
    };
    let bytes = match local_ipc::exchange(&socket_path(db), &serde_json::to_vec(&request)?) {
        Ok(bytes) => bytes,
        Err(ExchangeError::Unavailable) => {
            let (op, kind, id) = request.command.audit_fields();
            Store::open_existing(db)
                .and_then(|mut s| s.operator_event(admin, op, (kind, &id), "unavailable"))
                .map_err(io::Error::other)?;
            return Err(io::Error::other("live admin unavailable"));
        }
        Err(ExchangeError::Unknown) => {
            return Err(io::Error::other("live admin response unknown; query audit"));
        }
    };
    let response: Value = serde_json::from_slice(&bytes)
        .map_err(|_| io::Error::other("live admin response unknown; query audit"))?;
    if let Some(error) = response.get("error").and_then(Value::as_str) {
        return Err(io::Error::other(error.to_owned()));
    }
    Ok(response)
}

pub(crate) struct Listener {
    listener: local_ipc::Listener,
    db: PathBuf,
    epoch: i64,
    instance: String,
    hub: Arc<Hub>,
}
impl Listener {
    pub(crate) fn bind(db: &Path, epoch: i64, hub: Arc<Hub>) -> io::Result<Self> {
        let mut nonce = [0; 16];
        getrandom::fill(&mut nonce).map_err(io::Error::other)?;
        let instance = format!("{:032x}", u128::from_be_bytes(nonce));
        let listener = local_ipc::Listener::bind(&socket_path(db))?;
        Ok(Self {
            listener,
            db: db.to_owned(),
            epoch,
            instance,
            hub,
        })
    }
    pub(crate) async fn run(self, mut stop: watch::Receiver<bool>) -> io::Result<()> {
        loop {
            let mut socket = tokio::select! {
                biased;
                _ = stop.changed() => return Ok(()),
                incoming = self.listener.accept() => incoming?,
            };
            let mut bytes = Vec::new();
            let read = async {
                self.listener.authenticate(&mut socket).await?;
                (&mut socket).take(4097).read_to_end(&mut bytes).await
            };
            match timeout(CONTROL_IO, read).await {
                Ok(Ok(_)) if bytes.len() <= 4096 => {}
                _ => continue,
            }
            let Ok(request) = serde_json::from_slice(&bytes) else {
                continue;
            };
            let (db, hub, epoch) = (self.db.clone(), self.hub.clone(), self.epoch);
            let result = handle(db, hub, epoch, self.instance.clone(), request)
                .await
                .unwrap_or_else(|e| json!({"error":e.to_string()}));
            let _ = timeout(CONTROL_IO, socket.write_all(&serde_json::to_vec(&result)?)).await;
        }
    }
}

async fn handle(
    db: PathBuf,
    hub: Arc<Hub>,
    epoch: i64,
    instance: String,
    request: Request,
) -> accounts::Result<Value> {
    let (mut store, audit_id, response, ticket) = tokio::task::spawn_blocking(move || {
        let admin = Admin {
            actor: &request.actor,
            reason: &request.reason,
        };
        let mut store = Store::open_existing(&db)?;
        let (op, kind, id) = request.command.audit_fields();
        let audit_id = store.operator_event(&admin, op, (kind, &id), "pending")?;
        let (response, ticket) = match request.command {
            LiveCommand::ListRooms => match hub.operator_rooms() {
                Ok(keys) => (
                    json!({"rooms":keys.into_iter().map(|key| RoomTarget {
                    instance: instance.clone(), epoch, room_id: key.id.0, session: key.session, generation: key.generation,
                }).collect::<Vec<_>>()}),
                    None,
                ),
                Err(_) => (json!({"error":"room service unavailable"}), None),
            },
            LiveCommand::CloseRoom { target } if target.epoch == epoch && target.instance == instance => {
                match hub.operator_close(target.key()) {
                    Ok(Some(ticket)) => (json!({"ok":true}), ticket),
                    Ok(None) => (json!({"error":"room not found"}), None),
                    Err(_) => (json!({"error":"room service unavailable"}), None),
                }
            }
            LiveCommand::F2Runtime { target, command } if target.epoch == epoch && target.instance == instance => {
                match hub.operator_f2(target.key(), command) {
                    Ok(view) => (json!({"f2_runtime":view}), None),
                    Err(error) => (json!({"error":error.to_string()}), None),
                }
            }
            _ => (json!({"error":"stale service epoch"}), None),
        };
        Ok::<_, accounts::Error>((store, audit_id, response, ticket))
    })
    .await
    .map_err(|_| accounts::Error::Storage)??;
    let mut result = if response.get("error").is_some() {
        "rejected"
    } else {
        "ok"
    };
    let response = if let Some(ticket) = ticket
        && !matches!(
            timeout(Duration::from_secs(7), ticket.wait()).await,
            Ok(Ok(()))
        ) {
        result = "journal_unconfirmed";
        json!({"error":"room closed; match journal not confirmed"})
    } else {
        response
    };
    tokio::task::spawn_blocking(move || store.finish_operator_event(&audit_id, result))
        .await
        .map_err(|_| accounts::Error::Storage)??;
    Ok(response)
}
