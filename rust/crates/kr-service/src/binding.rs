//! Operator-bound diagnostic endpoints, not account authentication.
//! Source IPv4 picks a pinned lab identity; destination port picks transport role.
use std::{
    collections::HashMap,
    io,
    net::Ipv4Addr,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Wcp,
    Login,
    Channel,
}
#[derive(Default)]
struct Actor {
    wcp: u8,
    generation: u64,
    login: Option<u64>,
    channel: Option<u64>,
    redirected: Option<Instant>,
    channel_bound: bool,
}
#[derive(Default)]
pub struct Registry {
    actors: HashMap<Ipv4Addr, Actor>,
}
pub type Shared = Arc<Mutex<Registry>>;
pub struct Lease {
    shared: Shared,
    peer: Ipv4Addr,
    connection: u64,
    generation: u64,
    pub kind: Kind,
}
impl Registry {
    pub fn claim(
        shared: &Shared,
        peer: Ipv4Addr,
        connection: u64,
        kind: Kind,
    ) -> io::Result<Lease> {
        let mut registry = shared
            .lock()
            .map_err(|_| io::Error::other("binding registry unavailable"))?;
        let a = registry.actors.entry(peer).or_default();
        let kind = match kind {
            Kind::Wcp => {
                if a.wcp >= 1 {
                    return Err(io::Error::other("diagnostic WCP request budget exhausted"));
                }
                a.wcp += 1;
                Kind::Wcp
            }
            Kind::Login => {
                if a.login.is_some()
                    || a.channel.is_some()
                    || a.redirected
                        .is_some_and(|t| t.elapsed() <= Duration::from_secs(10))
                {
                    return Err(io::Error::other("diagnostic identity already reserved"));
                }
                a.generation = a
                    .generation
                    .checked_add(1)
                    .ok_or_else(|| io::Error::other("binding generation exhausted"))?;
                a.redirected = None;
                a.channel_bound = false;
                a.login = Some(connection);
                Kind::Login
            }
            Kind::Channel => {
                if a.channel.is_some()
                    || !a
                        .redirected
                        .is_some_and(|t| t.elapsed() <= Duration::from_secs(10))
                {
                    return Err(io::Error::other(
                        "missing/expired diagnostic redirect or channel occupied",
                    ));
                }
                a.channel = Some(connection);
                Kind::Channel
            }
        };
        Ok(Lease {
            shared: shared.clone(),
            peer,
            connection,
            generation: a.generation,
            kind,
        })
    }
}
impl Lease {
    pub fn redirected(&self) -> io::Result<()> {
        let mut r = self
            .shared
            .lock()
            .map_err(|_| io::Error::other("binding registry unavailable"))?;
        let a = r
            .actors
            .get_mut(&self.peer)
            .ok_or_else(|| io::Error::other("missing binding"))?;
        if self.kind != Kind::Login
            || a.generation != self.generation
            || a.login != Some(self.connection)
        {
            return Err(io::Error::other("stale diagnostic login"));
        }
        a.redirected = Some(Instant::now());
        Ok(())
    }
    pub fn bind_channel(&self) -> io::Result<()> {
        let mut r = self
            .shared
            .lock()
            .map_err(|_| io::Error::other("binding registry unavailable"))?;
        let a = r
            .actors
            .get_mut(&self.peer)
            .ok_or_else(|| io::Error::other("missing binding"))?;
        if self.kind != Kind::Channel
            || a.generation != self.generation
            || a.channel != Some(self.connection)
            || a.channel_bound
            || !a
                .redirected
                .is_some_and(|t| t.elapsed() <= Duration::from_secs(10))
        {
            return Err(io::Error::other("stale diagnostic channel"));
        }
        a.channel_bound = true;
        a.redirected = None;
        Ok(())
    }
}
impl Drop for Lease {
    fn drop(&mut self) {
        if let Ok(mut r) = self.shared.lock()
            && let Some(a) = r.actors.get_mut(&self.peer)
        {
            if a.generation != self.generation {
                return;
            }
            if self.kind == Kind::Login && a.login == Some(self.connection) {
                a.login = None;
            }
            if self.kind == Kind::Channel && a.channel == Some(self.connection) {
                a.channel = None;
                if a.channel_bound {
                    a.redirected = None;
                    a.channel_bound = false;
                }
            }
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn interleaved_actors_have_independent_endpoint_leases() {
        let s = Shared::default();
        let a = Ipv4Addr::new(127, 0, 0, 2);
        let b = Ipv4Addr::new(127, 0, 0, 3);
        assert!(Registry::claim(&s, a, 1, Kind::Channel).is_err());
        let la = Registry::claim(&s, a, 2, Kind::Login).unwrap();
        let lb = Registry::claim(&s, b, 3, Kind::Login).unwrap();
        assert!(Registry::claim(&s, a, 4, Kind::Login).is_err());
        lb.redirected().unwrap();
        la.redirected().unwrap();
        let cb = Registry::claim(&s, b, 5, Kind::Channel).unwrap();
        cb.bind_channel().unwrap();
        assert!(Registry::claim(&s, b, 6, Kind::Channel).is_err());
        assert!(cb.bind_channel().is_err());
        drop(lb);
        drop(cb);
        assert!(Registry::claim(&s, b, 7, Kind::Channel).is_err());
        let ca = Registry::claim(&s, a, 8, Kind::Channel).unwrap();
        ca.bind_channel().unwrap();
        assert!(Registry::claim(&s, a, 9, Kind::Login).is_err());
        drop(la);
        drop(ca);
        assert!(Registry::claim(&s, a, 10, Kind::Login).is_ok());
    }
    #[test]
    fn unbound_channel_failure_does_not_consume_redirect_but_expiry_does() {
        let s = Shared::default();
        let a = Ipv4Addr::LOCALHOST;
        let l = Registry::claim(&s, a, 1, Kind::Login).unwrap();
        l.redirected().unwrap();
        drop(l);
        drop(Registry::claim(&s, a, 2, Kind::Channel).unwrap());
        let c = Registry::claim(&s, a, 3, Kind::Channel).unwrap();
        s.lock().unwrap().actors.get_mut(&a).unwrap().redirected =
            Some(Instant::now() - Duration::from_secs(11));
        assert!(c.bind_channel().is_err());
        drop(c);
        assert!(Registry::claim(&s, a, 4, Kind::Channel).is_err());
        assert!(Registry::claim(&s, a, 5, Kind::Login).is_ok());
    }
}
