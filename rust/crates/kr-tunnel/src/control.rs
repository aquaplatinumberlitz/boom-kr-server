//! Company control protocol, carried only by an authenticated QUIC stream.
//! Bounded length framing is independent of native KR TCP/UDP codecs.
use crate::router::{Assignment, Binding, Scope};
use quinn::{RecvStream, SendStream};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::io;
pub const MAX_CONTROL: usize = 4096;
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum Server {
    Configure {
        scope: Scope,
        routes: Vec<Assignment>,
    },
    Retire {
        scope: Scope,
    },
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum Client {
    Ready {
        scope: Scope,
        bindings: Vec<Binding>,
    },
}
pub async fn write<T: Serialize>(send: &mut SendStream, value: &T) -> io::Result<()> {
    let bytes = serde_json::to_vec(value).map_err(io::Error::other)?;
    if bytes.is_empty() || bytes.len() > MAX_CONTROL {
        return Err(io::Error::other("peer control write bound"));
    }
    send.write_all(&(bytes.len() as u32).to_le_bytes())
        .await
        .map_err(io::Error::other)?;
    send.write_all(&bytes).await.map_err(io::Error::other)
}
pub async fn read<T: DeserializeOwned>(recv: &mut RecvStream) -> io::Result<T> {
    let mut size = [0; 4];
    recv.read_exact(&mut size).await.map_err(io::Error::other)?;
    let size = u32::from_le_bytes(size) as usize;
    if size == 0 || size > MAX_CONTROL {
        return Err(io::Error::other("peer control read bound"));
    }
    let mut bytes = vec![0; size];
    recv.read_exact(&mut bytes)
        .await
        .map_err(io::Error::other)?;
    serde_json::from_slice(&bytes).map_err(io::Error::other)
}
