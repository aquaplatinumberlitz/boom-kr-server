//! Server choice, authenticated S1 bridge and per-server account hints.
use crate::game::Config;
use kr_p2p::{
    Endpoint, Invite,
    self_host::{ClientSession, Network},
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs, io,
    net::{Ipv4Addr, SocketAddr, SocketAddrV4},
    path::{Path, PathBuf},
    sync::Arc,
};

pub(crate) type Id = String;

#[derive(Clone)]
pub(crate) struct Route {
    pub account: SocketAddrV4,
    pub account_cert: PathBuf,
    pub game: SocketAddrV4,
    pub game_cert: PathBuf,
}

impl Route {
    fn id(&self) -> std::io::Result<Id> {
        let mut hash = Sha256::new();
        for (address, certificate) in [
            (self.account, &self.account_cert),
            (self.game, &self.game_cert),
        ] {
            hash.update(address.ip().octets());
            hash.update(address.port().to_be_bytes());
            // Paths can change when the bundle is moved. Trust bytes identify the route.
            hash.update(Sha256::digest(fs::read(certificate)?));
        }
        Ok(hash.finalize().iter().map(|b| format!("{b:02x}")).collect())
    }
}

pub(crate) struct Entry {
    pub id: Id,
    pub name: Option<String>,
    pub route: Option<Route>,
    pub invite: Option<Invite>,
}

impl Entry {
    pub fn bundled(config: &Config) -> std::io::Result<Self> {
        let route = config
            .route()
            .ok_or_else(|| std::io::Error::other("no bundled server"))?;
        Ok(Self {
            id: route.id()?,
            name: config
                .server_name
                .as_deref()
                .map(str::trim)
                .filter(|name| {
                    !name.is_empty()
                        && name.chars().count() <= 64
                        && !name.chars().any(char::is_control)
                })
                .map(str::to_owned),
            route: Some(route),
            invite: None,
        })
    }
}

const LIMIT: usize = 32;

impl Entry {
    pub fn invited(code: &str, name: String) -> Option<Self> {
        let invite @ Invite::Server { .. } = code.parse().ok()? else {
            return None;
        };
        Some(Self {
            id: invite.to_string(),
            invite: Some(invite),
            name: Some(name),
            route: None,
        })
    }
}

pub(crate) struct Bridge {
    // The bridge must outlive account requests and the native companion.
    session: ClientSession,
    _endpoint: Endpoint,
    pub route: Route,
    cert: PathBuf,
}

impl Bridge {
    pub fn name(&self) -> &str {
        &self.session.name
    }

    pub async fn open(state: &Path, invite: &Invite) -> io::Result<Arc<Self>> {
        Self::dial(state, invite, Network::Public, None).await
    }

    async fn dial(
        state: &Path,
        invite: &Invite,
        network: Network,
        hint: Option<SocketAddr>,
    ) -> io::Result<Arc<Self>> {
        let endpoint = network
            .endpoint(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 0), None)
            .await?;
        let session = ClientSession::open(&endpoint, &network, invite, hint).await?;
        let account = session
            .launcher
            .ok_or_else(|| io::Error::other("S1 launcher service unavailable"))?;
        // A unique file avoids cross-instance overwrite and is removed after
        // the final use by the launcher. Certificate bytes were verified by S1.
        let cert = state.join(format!("s1-{}-{}.der", std::process::id(), account.port()));
        {
            use std::io::Write;
            let mut file = fs::File::create_new(&cert)?;
            if let Err(error) = file.write_all(&session.certificate) {
                drop(file);
                let _ = fs::remove_file(&cert);
                return Err(error);
            }
        }
        let route = Route {
            account,
            account_cert: cert.clone(),
            game: session.prelogin,
            game_cert: cert.clone(),
        };
        Ok(Arc::new(Self {
            session,
            _endpoint: endpoint,
            route,
            cert,
        }))
    }
}

impl Drop for Bridge {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.cert);
    }
}

#[derive(Default, Serialize, Deserialize)]
pub(crate) struct Preferences {
    pub selected: Option<Id>,
    logins: BTreeMap<Id, String>,
    #[serde(default)]
    invites: BTreeMap<Id, String>,
}

impl Preferences {
    pub fn entries(&self) -> impl Iterator<Item = Entry> + '_ {
        self.invites.iter().take(LIMIT).filter_map(|(code, name)| {
            if !(1..=128).contains(&name.len()) || name.chars().any(char::is_control) {
                return None;
            }
            Entry::invited(code, name.clone())
        })
    }

    pub fn has_space(&self) -> bool {
        self.invites.len() < LIMIT
    }

    pub fn add(&mut self, code: Id, name: String) {
        if self.invites.contains_key(&code) || self.has_space() {
            self.invites.insert(code, name);
        }
    }

    pub fn remove(&mut self, id: &str) {
        self.invites.remove(id);
        self.logins.remove(id);
    }

    pub fn login(&self, id: &str) -> Option<&str> {
        self.logins.get(id).map(String::as_str)
    }

    pub fn remember(&mut self, id: Id, login: &str, deleted: bool) {
        if deleted {
            if self
                .login(&id)
                .is_some_and(|old| old.eq_ignore_ascii_case(login))
            {
                self.logins.remove(&id);
            }
        } else {
            self.logins.insert(id, login.to_owned());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn invite_bridge_checks_pin_and_keeps_the_saved_selection_reconnectable() {
        use kr_p2p::{
            SecretKey,
            self_host::{Services, serve},
        };
        use tokio::net::{TcpListener, UdpSocket};

        let local = SocketAddrV4::new(Ipv4Addr::LOCALHOST, 0);
        let host = Network::Local
            .endpoint(local, Some(SecretKey::generate()))
            .await
            .unwrap();
        let prelogin = UdpSocket::bind(local).await.unwrap();
        let relay = UdpSocket::bind(local).await.unwrap();
        let launcher = TcpListener::bind(local).await.unwrap();
        let ipv4 = |addr: SocketAddr| match addr {
            SocketAddr::V4(addr) => addr,
            _ => panic!("expected IPv4"),
        };
        let certificate = vec![11, 22, 33];
        let endpoint = host.clone();
        let services = Services {
            prelogin: ipv4(prelogin.local_addr().unwrap()),
            relay: ipv4(relay.local_addr().unwrap()),
            launcher: Some(ipv4(launcher.local_addr().unwrap())),
            name: "Local club".into(),
            certificate: certificate.clone(),
        };
        let host_task = tokio::spawn(async move { serve(&endpoint, services).await });
        let invite = Invite::server(host.id(), &certificate);
        let wrong = Invite::server(host.id(), b"wrong pin");
        let hint = host.bound_sockets().into_iter().find(SocketAddr::is_ipv4);
        let path = std::env::temp_dir().join(format!("lanboom-s1-test-{}", std::process::id()));
        fs::create_dir(&path).unwrap();
        assert!(
            Bridge::dial(&path, &wrong, Network::Local, hint)
                .await
                .is_err()
        );
        assert_eq!(fs::read_dir(&path).unwrap().count(), 0);
        let bridge = Bridge::dial(&path, &invite, Network::Local, hint)
            .await
            .unwrap();
        assert_eq!(fs::read(&bridge.route.account_cert).unwrap(), certificate);
        assert!(bridge.route.account.ip().is_loopback() && bridge.route.game.ip().is_loopback());
        let id = invite.to_string();
        let mut saved = Preferences {
            selected: Some(id.clone()),
            ..Default::default()
        };
        saved.add(id.clone(), bridge.name().into());
        drop(bridge);
        fs::remove_dir(&path).unwrap();
        let mut saved: Preferences =
            serde_json::from_slice(&serde_json::to_vec(&saved).unwrap()).unwrap();
        let entry = saved.entries().next().unwrap();
        assert_eq!(entry.invite, Some(invite));
        assert_eq!(entry.id, id);
        saved.remove(&id);
        assert!(saved.entries().next().is_none());
        host_task.abort();
        host.close().await;
    }

    #[test]
    fn account_hints_round_trip_and_stay_scoped_to_their_server() {
        let mut prefs = Preferences {
            selected: Some("first".into()),
            ..Default::default()
        };
        prefs.remember("first".into(), "alice", false);
        prefs.remember("second".into(), "bob", false);
        prefs.remember("first".into(), "someone_else", true);
        let mut prefs: Preferences =
            serde_json::from_slice(&serde_json::to_vec(&prefs).unwrap()).unwrap();
        assert_eq!(prefs.selected.as_deref(), Some("first"));
        assert_eq!(prefs.login("first"), Some("alice"));
        prefs.remember("first".into(), "ALICE", true);
        assert_eq!(prefs.login("first"), None);
        assert_eq!(prefs.login("second"), Some("bob"));
    }
}
