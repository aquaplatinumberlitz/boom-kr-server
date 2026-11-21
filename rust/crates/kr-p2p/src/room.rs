//! O1 discovery and admission. DNS-SD supplies only an untrusted local hint;
//! the code commits to the full node key verified by the iroh handshake.
use crate::{Connection, Endpoint, EndpointAddr, EndpointId, Invite, RelayMode, RelayUrl};
use mdns_sd::{ServiceDaemon, ServiceEvent, ServiceInfo};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    io,
    net::{Ipv4Addr, SocketAddr},
    time::Duration,
};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

const SERVICE: &str = "_boom-kr._udp.local.";
const MAGIC: &[u8; 8] = b"BOOM-O1\0";
const ROOM_CODE: usize = 24;
const MAX_NAME: usize = 64;
const TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Route {
    pub node: EndpointId,
    pub addresses: Vec<SocketAddr>,
    pub relay: Option<RelayUrl>,
}

impl Route {
    fn address(&self) -> io::Result<EndpointAddr> {
        if (self.addresses.is_empty() && self.relay.is_none())
            || self.addresses.len() > 16
            || self.addresses.iter().any(|addr| {
                !addr.is_ipv4()
                    || addr.port() == 0
                    || addr.ip().is_unspecified()
                    || addr.ip().is_multicast()
            })
        {
            return Err(io::Error::other("room route bounds"));
        }
        let address = self
            .addresses
            .iter()
            .copied()
            .fold(EndpointAddr::new(self.node), EndpointAddr::with_ip_addr);
        Ok(if let Some(relay) = &self.relay {
            address.with_relay_url(relay.clone())
        } else {
            address
        })
    }
}

pub struct Discovery(ServiceDaemon);
impl Discovery {
    pub fn new() -> io::Result<Self> {
        Ok(Self(ServiceDaemon::new().map_err(io::Error::other)?))
    }

    pub fn advertise(&self, node: EndpointId, port: u16) -> io::Result<()> {
        let code = Invite::room(node).to_string();
        let endpoint = node.to_string();
        let info = ServiceInfo::new(
            SERVICE,
            &code,
            &format!("{code}.local."),
            (),
            port,
            &[("node", endpoint.as_str())][..],
        )
        .map_err(io::Error::other)?
        .enable_addr_auto();
        self.0.register(info).map_err(io::Error::other)
    }

    pub async fn rooms(&self, wait: Duration) -> io::Result<Vec<Route>> {
        let receiver = self.0.browse(SERVICE).map_err(io::Error::other)?;
        let mut routes = BTreeMap::new();
        let _ = tokio::time::timeout(wait, async {
            while let Ok(event) = receiver.recv_async().await {
                match event {
                    ServiceEvent::ServiceResolved(info) => {
                        let Some(code) = info
                            .get_fullname()
                            .strip_suffix(&format!(".{SERVICE}"))
                            .and_then(|name| name.parse::<Invite>().ok())
                        else {
                            continue;
                        };
                        let Some(node) = info
                            .get_property_val_str("node")
                            .and_then(|key| key.parse().ok())
                        else {
                            continue;
                        };
                        if routes.len() >= 64
                            || !matches!(code, Invite::Room(_))
                            || !code.matches_node(node)
                        {
                            continue;
                        }
                        let addresses = info
                            .get_addresses_v4()
                            .into_iter()
                            .filter(|ip| !ip.is_loopback())
                            .take(16)
                            .map(|ip| SocketAddr::from((ip, info.get_port())))
                            .collect();
                        let route = Route {
                            node,
                            addresses,
                            relay: None,
                        };
                        if route.address().is_ok() {
                            routes.insert(info.get_fullname().to_owned(), route);
                        }
                    }
                    ServiceEvent::ServiceRemoved(_, name) => {
                        routes.remove(&name);
                    }
                    _ => {}
                }
            }
        })
        .await;
        let _ = self.0.stop_browse(SERVICE);
        Ok(routes.into_values().collect())
    }
}
impl Drop for Discovery {
    fn drop(&mut self) {
        let _ = self.0.shutdown();
    }
}

pub async fn endpoint(public: bool) -> io::Result<Endpoint> {
    let mode = if public {
        RelayMode::Default
    } else {
        RelayMode::Disabled
    };
    crate::endpoint_builder((Ipv4Addr::UNSPECIFIED, 0).into(), mode, None)?
        .bind()
        .await
        .map_err(io::Error::other)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JoinError {
    Unavailable,
    Version,
    Denied,
}
impl std::fmt::Display for JoinError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Unavailable => "room unavailable",
            Self::Version => "room package differs",
            Self::Denied => "room admission denied",
        })
    }
}
impl std::error::Error for JoinError {}

pub async fn join(
    endpoint: &Endpoint,
    code: &Invite,
    route: &Route,
    package: [u8; 32],
    name: &str,
) -> Result<Connection, JoinError> {
    if !matches!(code, Invite::Room(_)) || !(1..=MAX_NAME).contains(&name.len()) {
        return Err(JoinError::Denied);
    }
    tokio::time::timeout(TIMEOUT, async {
        let address = route.address().map_err(|_| JoinError::Unavailable)?;
        let connection = crate::connect(endpoint, code, address)
            .await
            .map_err(|_| JoinError::Unavailable)?;
        let admission = async {
            let (mut send, mut recv) = connection.open_bi().await.map_err(io::Error::other)?;
            send.write_all(MAGIC).await?;
            send.write_all(code.to_string().as_bytes()).await?;
            send.write_all(&package).await?;
            send.write_u8(name.len() as u8).await?;
            send.write_all(name.as_bytes()).await?;
            send.finish().map_err(io::Error::other)?;
            recv.read_u8().await
        }
        .await
        .map_err(|_| JoinError::Unavailable);
        match admission {
            Ok(0) => Ok(connection),
            status => {
                connection.close(1u32.into(), b"room admission rejected");
                Err(match status {
                    Ok(1) => JoinError::Version,
                    Ok(_) => JoinError::Denied,
                    Err(e) => e,
                })
            }
        }
    })
    .await
    .map_err(|_| JoinError::Unavailable)?
}

pub async fn admit(
    connection: &Connection,
    node: EndpointId,
    package: [u8; 32],
    valid_name: impl Fn(&str) -> bool,
) -> io::Result<String> {
    tokio::time::timeout(TIMEOUT, async {
        let (mut send, mut recv) = connection.accept_bi().await.map_err(io::Error::other)?;
        let mut magic = [0; 8];
        let mut code = [0; ROOM_CODE];
        let mut fingerprint = [0; 32];
        recv.read_exact(&mut magic)
            .await
            .map_err(io::Error::other)?;
        recv.read_exact(&mut code).await.map_err(io::Error::other)?;
        recv.read_exact(&mut fingerprint)
            .await
            .map_err(io::Error::other)?;
        let len = recv.read_u8().await? as usize;
        let mut name = [0; MAX_NAME];
        if len <= name.len() {
            recv.read_exact(&mut name[..len])
                .await
                .map_err(io::Error::other)?;
        }
        let name = name
            .get(..len)
            .and_then(|bytes| std::str::from_utf8(bytes).ok());
        let code = std::str::from_utf8(&code)
            .ok()
            .and_then(|code| code.parse::<Invite>().ok());
        let status =
            if magic != *MAGIC || code != Some(Invite::room(node)) || !name.is_some_and(valid_name)
            {
                2
            } else if package != fingerprint {
                1
            } else {
                0
            };
        send.write_u8(status).await?;
        send.finish().map_err(io::Error::other)?;
        if status == 0 {
            Ok(name.expect("admitted name").into())
        } else {
            let _ = tokio::time::timeout(Duration::from_millis(750), send.stopped()).await;
            Err(io::Error::other("room admission rejected"))
        }
    })
    .await
    .map_err(|_| io::Error::other("room admission deadline"))?
}
