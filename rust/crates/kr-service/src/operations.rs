//! Local Linux operator controls. The socket lives in the private DB directory.
use crate::{
    accounts::{self, Admin, Store},
    room_registry::{RoomId, RoomKey},
    room_service::Hub,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    fs::{self, File, OpenOptions},
    io::{self, Read, Write},
    os::unix::{
        fs::{FileTypeExt, MetadataExt, OpenOptionsExt, PermissionsExt},
        net::UnixStream as BlockingStream,
    },
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::UnixListener,
    sync::watch,
    time::timeout,
};

mod backup;
pub use backup::BackupSchedule;
pub(crate) use backup::Scheduled;

const CONTROL_IO: Duration = Duration::from_secs(2);
const RESPONSE_IO: Duration = Duration::from_secs(15);

/// The SQLite instance lock is also used by offline CLI operations.
pub fn lock_database(database: &Path) -> io::Result<File> {
    let parent = database
        .parent()
        .ok_or_else(|| io::Error::other("database parent missing"))?;
    let metadata = fs::symlink_metadata(parent)?;
    if !metadata.is_dir() || metadata.permissions().mode() & 0o077 != 0 {
        return Err(io::Error::other("database parent must be private"));
    }
    let path = database.with_extension("instance.lock");
    match fs::symlink_metadata(&path) {
        Ok(meta) if meta.is_file() && meta.permissions().mode() & 0o077 == 0 => {}
        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
        _ => {
            return Err(io::Error::other(
                "instance lock must be a private regular file",
            ));
        }
    }
    let lock = OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .mode(0o600)
        .open(path)?;
    lock.try_lock()?;
    Ok(lock)
}

pub(crate) struct PrivateSocket {
    path: PathBuf,
    device: u64,
    inode: u64,
}
impl PrivateSocket {
    pub(crate) fn bind(path: &Path) -> io::Result<(UnixListener, Self)> {
        // The caller holds the instance lock. Never unlink a live listener or
        // any non-socket, even after an unclean process exit.
        match fs::symlink_metadata(path) {
            Ok(meta) if meta.file_type().is_socket() => match BlockingStream::connect(path) {
                Err(e) if e.kind() == io::ErrorKind::ConnectionRefused => fs::remove_file(path)?,
                _ => return Err(io::Error::other("operator socket path already active")),
            },
            Err(e) if e.kind() == io::ErrorKind::NotFound => {}
            _ => return Err(io::Error::other("operator socket path occupied")),
        }
        let listener = UnixListener::bind(path)?;
        let meta = fs::symlink_metadata(path)?;
        let owner = Self {
            path: path.to_owned(),
            device: meta.dev(),
            inode: meta.ino(),
        };
        fs::set_permissions(path, fs::Permissions::from_mode(0o600))?;
        Ok((listener, owner))
    }
}
impl Drop for PrivateSocket {
    fn drop(&mut self) {
        if let Ok(meta) = fs::symlink_metadata(&self.path)
            && meta.file_type().is_socket()
            && meta.dev() == self.device
            && meta.ino() == self.inode
        {
            let _ = fs::remove_file(&self.path);
        }
    }
}

fn socket_path(database: &Path) -> PathBuf {
    database.with_extension("admin.sock")
}

#[derive(Serialize, Deserialize, Clone, Copy)]
#[serde(deny_unknown_fields)]
pub struct RoomTarget {
    pub epoch: i64,
    pub room_id: u32,
    pub session: u16,
    pub generation: u64,
}
impl RoomTarget {
    fn key(self) -> RoomKey {
        RoomKey {
            id: RoomId(self.room_id),
            session: self.session,
            generation: self.generation,
        }
    }
    fn audit_id(self) -> String {
        format!(
            "{}/{}/{}/{}",
            self.epoch, self.room_id, self.session, self.generation
        )
    }
}
#[derive(Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case", deny_unknown_fields)]
pub enum LiveCommand {
    ListRooms,
    CloseRoom { target: RoomTarget },
}
impl LiveCommand {
    fn audit_fields(&self) -> (&'static str, &'static str, String) {
        match self {
            Self::ListRooms => ("list_rooms", "service", "rooms".into()),
            Self::CloseRoom { target } => ("close_room", "room", target.audit_id()),
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
    let mut socket = match BlockingStream::connect(socket_path(db)) {
        Ok(socket) => socket,
        Err(_) => {
            let (op, kind, id) = command.audit_fields();
            Store::open_existing(db)
                .and_then(|mut s| s.operator_event(admin, op, (kind, &id), "unavailable"))
                .map_err(io::Error::other)?;
            return Err(io::Error::other("live admin unavailable"));
        }
    };
    socket.set_write_timeout(Some(CONTROL_IO))?;
    socket.set_read_timeout(Some(RESPONSE_IO))?;
    let unknown = |_| io::Error::other("live admin response unknown; query audit");
    let request = Request {
        actor: admin.actor.into(),
        reason: admin.reason.into(),
        command,
    };
    socket
        .write_all(&serde_json::to_vec(&request)?)
        .map_err(unknown)?;
    socket
        .shutdown(std::net::Shutdown::Write)
        .map_err(unknown)?;
    let mut bytes = Vec::new();
    socket
        .take(32_769)
        .read_to_end(&mut bytes)
        .map_err(unknown)?;
    let response: Value = serde_json::from_slice(&bytes)
        .map_err(|_| io::Error::other("live admin response unknown; query audit"))?;
    if let Some(error) = response.get("error").and_then(Value::as_str) {
        return Err(io::Error::other(error.to_owned()));
    }
    Ok(response)
}

pub(crate) struct Listener {
    listener: UnixListener,
    _owner: PrivateSocket,
    db: PathBuf,
    epoch: i64,
    hub: Arc<Hub>,
}
impl Listener {
    pub(crate) fn bind(db: &Path, epoch: i64, hub: Arc<Hub>) -> io::Result<Self> {
        let (listener, owner) = PrivateSocket::bind(&socket_path(db))?;
        Ok(Self {
            listener,
            _owner: owner,
            db: db.to_owned(),
            epoch,
            hub,
        })
    }
    pub(crate) async fn run(self, mut stop: watch::Receiver<bool>) -> io::Result<()> {
        loop {
            let (socket, _) = tokio::select! {
                biased;
                _ = stop.changed() => return Ok(()),
                incoming = self.listener.accept() => incoming?,
            };
            let mut bytes = Vec::new();
            let mut socket = socket.take(4097);
            match timeout(CONTROL_IO, socket.read_to_end(&mut bytes)).await {
                Ok(Ok(_)) if bytes.len() <= 4096 => {}
                _ => continue,
            }
            let Ok(request) = serde_json::from_slice(&bytes) else {
                continue;
            };
            let mut stream = socket.into_inner();
            let (db, hub, epoch) = (self.db.clone(), self.hub.clone(), self.epoch);
            let result = handle(db, hub, epoch, request)
                .await
                .unwrap_or_else(|e| json!({"error":e.to_string()}));
            let _ = timeout(CONTROL_IO, stream.write_all(&serde_json::to_vec(&result)?)).await;
        }
    }
}

async fn handle(
    db: PathBuf,
    hub: Arc<Hub>,
    epoch: i64,
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
                    epoch, room_id: key.id.0, session: key.session, generation: key.generation,
                }).collect::<Vec<_>>()}),
                    None,
                ),
                Err(_) => (json!({"error":"room service unavailable"}), None),
            },
            LiveCommand::CloseRoom { target } if target.epoch == epoch => {
                match hub.operator_close(target.key()) {
                    Ok(Some(ticket)) => (json!({"ok":true}), ticket),
                    Ok(None) => (json!({"error":"room not found"}), None),
                    Err(_) => (json!({"error":"room service unavailable"}), None),
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
