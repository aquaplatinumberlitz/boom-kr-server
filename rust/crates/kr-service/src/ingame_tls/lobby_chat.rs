//! One directory per prelogin listener. A visit exists only after lobby entry
//! and is dropped at room selection or disconnect.
use crate::account_bridge::local::{LocalAuthority, native::Driver};
use std::{
    collections::HashMap,
    io,
    sync::{
        Arc, Mutex, Weak,
        atomic::{AtomicBool, Ordering},
    },
};
use tokio::sync::mpsc;

pub(super) const QUEUE: usize = 8;
const VISITS: usize = 128;

#[derive(Default)]
pub(super) struct Directory(Mutex<HashMap<u64, mpsc::Sender<Message>>>);

struct Scope {
    current: AtomicBool,
    owner: LocalAuthority,
}

pub(super) struct Visit {
    directory: Arc<Directory>,
    scope: Arc<Scope>,
    incoming: mpsc::Receiver<Message>,
}

pub(super) struct Message {
    source: Weak<Scope>,
    pub(super) body: Arc<[u8]>,
}

impl Directory {
    pub(super) fn enter(self: &Arc<Self>, owner: LocalAuthority) -> io::Result<Visit> {
        let mut entries = self
            .0
            .lock()
            .map_err(|_| io::Error::other("lobby chat directory unavailable"))?;
        if !owner.current() || entries.contains_key(&owner.connection()) || entries.len() >= VISITS
        {
            return Err(io::Error::other("lobby chat visit denied"));
        }
        let (outgoing, incoming) = mpsc::channel(QUEUE);
        entries.insert(owner.connection(), outgoing);
        Ok(Visit {
            directory: self.clone(),
            scope: Arc::new(Scope {
                current: AtomicBool::new(true),
                owner,
            }),
            incoming,
        })
    }
}

impl Scope {
    fn current(&self) -> bool {
        self.current.load(Ordering::Acquire) && self.owner.current()
    }
}

impl Visit {
    pub(super) async fn send(&self, request: &[u8]) -> io::Result<()> {
        let body =
            kr_wire::kr00967::chat28_from18(&self.scope.owner.player().display_name, request)?;
        self.scope
            .owner
            .authorize_activity()
            .await
            .map_err(Driver::authority_error)?;
        let entries = self
            .directory
            .0
            .lock()
            .map_err(|_| io::Error::other("lobby chat directory unavailable"))?;
        if !self.scope.current() {
            return Err(io::Error::other("lobby chat sender retired"));
        }
        let permits = entries
            .values()
            .map(|sender| sender.try_reserve())
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| io::Error::other("lobby chat recipient queue unavailable"))?;
        let body: Arc<[u8]> = body.into();
        for permit in permits {
            permit.send(Message {
                source: Arc::downgrade(&self.scope),
                body: body.clone(),
            });
        }
        Ok(())
    }

    pub(super) fn receive(&mut self) -> Option<Message> {
        self.incoming.try_recv().ok()
    }
}

impl Message {
    pub(super) async fn source_is_current(&self) -> bool {
        let Some(scope) = self.source.upgrade() else {
            return false;
        };
        scope.current() && scope.owner.inspect().await.is_ok() && scope.current()
    }

    pub(super) fn source_still_current(&self) -> bool {
        self.source.upgrade().is_some_and(|scope| scope.current())
    }
}

impl Drop for Visit {
    fn drop(&mut self) {
        self.scope.current.store(false, Ordering::Release);
        if let Ok(mut entries) = self.directory.0.lock() {
            entries.remove(&self.scope.owner.connection());
        }
    }
}
