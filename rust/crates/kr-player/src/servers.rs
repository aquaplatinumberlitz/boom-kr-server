//! Server choice and per-server account hints. Saved preferences carry no routes or secrets.
//! S1 can add verified invitations to the trusted catalog without changing the selection model.
use crate::game::Config;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, fs, net::SocketAddrV4, path::PathBuf};

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
    pub route: Route,
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
            route,
        })
    }
}

#[derive(Default, Serialize, Deserialize)]
pub(crate) struct Preferences {
    pub selected: Option<Id>,
    logins: BTreeMap<Id, String>,
}

impl Preferences {
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
