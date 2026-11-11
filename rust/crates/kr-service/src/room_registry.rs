//! Synchronous room registry container (R1e foundation).
//! Session/room authority and retirement eligibility stay with the caller.
//! Public tuples are not reused in-process: KR82 has no measured epoch.
use std::collections::{BTreeMap, BTreeSet};
use std::io;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct RoomId(pub u32);
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RoomKey {
    pub id: RoomId,
    pub session: u16,
    pub generation: u64,
}
pub struct Entry<T> {
    pub key: RoomKey,
    pub value: T,
}
pub struct Registry<T> {
    rooms: BTreeMap<RoomId, Entry<T>>,
    public_tuples: BTreeSet<(u16, RoomId)>,
    generation: u64,
}
impl<T> Default for Registry<T> {
    fn default() -> Self {
        Self {
            rooms: BTreeMap::new(),
            public_tuples: BTreeSet::new(),
            generation: 0,
        }
    }
}
impl<T> Registry<T> {
    pub fn insert(&mut self, session: u16, id: RoomId, value: T) -> io::Result<RoomKey> {
        if session == 0
            || id.0 == 0
            || self.rooms.contains_key(&id)
            || self
                .rooms
                .values()
                .any(|entry| entry.key.session == session)
            || self.public_tuples.contains(&(session, id))
        {
            return Err(io::Error::other("invalid, duplicate or retired room tuple"));
        }
        let generation = self
            .generation
            .checked_add(1)
            .ok_or_else(|| io::Error::other("room generation exhausted"))?;
        let key = RoomKey {
            id,
            session,
            generation,
        };
        self.rooms.insert(id, Entry { key, value });
        self.public_tuples.insert((session, id));
        self.generation = generation;
        Ok(key)
    }
    pub fn selection(&self, session: u16, id: RoomId) -> io::Result<RoomKey> {
        self.rooms
            .get(&id)
            .filter(|entry| entry.key.session == session)
            .map(|entry| entry.key)
            .ok_or_else(|| io::Error::other("unknown room selection"))
    }
    pub fn get(&self, key: RoomKey) -> io::Result<&T> {
        self.rooms
            .get(&key.id)
            .filter(|entry| entry.key == key)
            .map(|entry| &entry.value)
            .ok_or_else(|| io::Error::other("stale room key"))
    }
    // Removal policy must first verify room occupancy/current work under a
    // consistent lock order. This generic container supplies no authority.
    pub fn remove(&mut self, key: RoomKey) -> io::Result<T> {
        self.get(key)?;
        Ok(self.rooms.remove(&key.id).expect("current entry").value)
    }
    pub fn remove_if(&mut self, key: RoomKey, removable: impl FnOnce(&T) -> bool) -> io::Result<T> {
        if !removable(self.get(key)?) {
            return Err(io::Error::other("room still has live users or work"));
        }
        self.remove(key)
    }
    pub fn entries(&self) -> impl Iterator<Item = &Entry<T>> {
        self.rooms.values()
    }
}

impl Registry<crate::room_io::Shared> {
    pub(crate) fn listing91(&self) -> io::Result<Vec<u8>> {
        let mut rows = Vec::new();
        for entry in self.entries() {
            let mut row = crate::room_io::listing_row(&entry.value)?;
            row.session = entry.key.session;
            row.room_id = entry.key.id.0;
            rows.push(row);
        }
        kr_wire::kr00967::listing91_from_rows(&rows)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn selection_and_removal_require_exact_current_key_and_never_reuse_public_tuple() {
        let mut registry = Registry::default();
        let alpha = registry.insert(1, RoomId(101), "alpha").unwrap();
        let beta = registry.insert(2, RoomId(202), "beta").unwrap();
        assert_eq!(*registry.get(alpha).unwrap(), "alpha");
        assert_eq!(*registry.get(beta).unwrap(), "beta");
        assert!(registry.selection(1, RoomId(202)).is_err());
        assert!(registry.insert(1, RoomId(303), "collision").is_err());
        let stale = RoomKey {
            generation: beta.generation + 1,
            ..beta
        };
        assert!(registry.remove(stale).is_err());
        assert_eq!(registry.entries().count(), 2);
        assert_eq!(registry.remove(beta).unwrap(), "beta");
        assert!(registry.get(beta).is_err());
        assert!(registry.insert(2, RoomId(202), "ABA").is_err());
        let gamma = registry.insert(2, RoomId(303), "gamma").unwrap();
        assert!(gamma.generation > beta.generation);
        assert_eq!(*registry.get(alpha).unwrap(), "alpha");
        assert_eq!(*registry.get(gamma).unwrap(), "gamma");
    }
}
