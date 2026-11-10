//! Bounded fixture allocator. CP028/037/044/171/174/188–189 reference policies.
//! No general KR authority, chain, terrain, life, inventory, or result semantics.
use crate::room::{Membership, wire_key};
use kr_wire::{Width, read_uint, write_uint};
use std::{
    collections::{BTreeMap, BTreeSet},
    io,
    sync::Arc,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Request {
    key: u32,
    owner: u8,
    serial: u32,
    row: u8,
    column: u8,
    power: u8,
}
impl Request {
    pub fn power(&self) -> u8 {
        self.power
    }
    pub fn parse(body: &[u8]) -> io::Result<Self> {
        let widths = [
            Width::U32,
            Width::U8,
            Width::U8,
            Width::U32,
            Width::U8,
            Width::U8,
            Width::U8,
            Width::U8,
            Width::U8,
            Width::U8,
            Width::U8,
            Width::U32,
            Width::U8,
        ];
        let lens = [4, 1, 1, 4, 1, 1, 1, 1, 1, 1, 1, 4, 1];
        if body.len() != 22 {
            return Err(io::Error::other("bomb118 size"));
        }
        let mut at = 0;
        let mut v = Vec::new();
        for (w, n) in widths.into_iter().zip(lens) {
            v.push(read_uint(&body[at..at + n], w).map_err(io::Error::other)?);
            at += n;
        }
        if [v[1], v[6], v[7], v[8], v[9], v[11], v[12]] != [0, 1, 0, 0, 0, 1, 0] {
            return Err(io::Error::other("unsupported basic bomb118 variant"));
        }
        Ok(Self {
            key: v[0] as u32,
            owner: v[2] as u8,
            serial: v[3] as u32,
            row: v[4] as u8,
            column: v[5] as u8,
            power: v[10] as u8,
        })
    }
    pub fn terrain_bomb(&self, id: u16) -> crate::terrain::Bomb {
        crate::terrain::Bomb {
            id,
            owner: self.owner,
            row: self.row,
            column: self.column,
            power: self.power,
        }
    }
    pub fn spawn159(&self, id: u16) -> io::Result<Vec<u8>> {
        let fields = [
            (0, Width::U8),
            (u64::from(self.owner), Width::U8),
            (0, Width::U16),
            (u64::from(self.serial), Width::U32),
            (0, Width::U32),
            (0, Width::U32),
            (u64::from(self.row), Width::U8),
            (u64::from(self.column), Width::U8),
            (u64::from(self.power), Width::U8),
            (1, Width::U32),
            (0, Width::U8),
            (u64::from(id), Width::U16),
            (0, Width::U16),
            (1, Width::U8),
            (0, Width::U64),
            (0, Width::U8),
            (0, Width::U8),
        ];
        let mut out = Vec::with_capacity(39);
        for (v, w) in fields {
            out.extend(write_uint(v, w).map_err(io::Error::other)?);
        }
        Ok(out)
    }
}
#[derive(Clone, Debug)]
pub(crate) struct Ticket {
    scope: Arc<()>,
    pub round: u64,
    allocation: u64,
    pub id: u16,
    pub owner: Membership,
}
#[derive(Clone, Debug)]
struct Active {
    ticket: Ticket,
    request: Request,
    pending: BTreeSet<Membership>,
}
#[derive(Clone, Debug)]
pub(crate) struct Ledger {
    scope: Arc<()>,
    generation: Option<u64>,
    next_wire: u32,
    next_allocation: u64,
    active: BTreeMap<u16, Active>,
    serials: BTreeMap<(Membership, u32), (Request, u16)>,
}
impl Default for Ledger {
    fn default() -> Self {
        Self {
            scope: Arc::new(()),
            generation: None,
            next_wire: 1,
            next_allocation: 0,
            active: BTreeMap::new(),
            serials: BTreeMap::new(),
        }
    }
}
impl Ledger {
    pub fn begin(&mut self, round: u64) {
        self.generation = Some(round);
        self.active.clear();
        self.serials.clear();
    }
    pub fn finish(&mut self) {
        self.generation = None;
        self.active.clear();
        self.serials.clear();
    }
    /// Operates on the candidate room clone; reserve ALL spawn+deadline queues
    /// before publishing this allocation/serial/reservation to the real owner.
    pub fn place(
        &mut self,
        round: u64,
        owner: Membership,
        slot: u8,
        request: &Request,
        roster: &[Membership],
        capacity: usize,
    ) -> io::Result<std::result::Result<Ticket, &'static str>> {
        let reject = if self.generation != Some(round) {
            Some("inactive_round")
        } else if request.key != wire_key(round)? {
            Some("round_key_mismatch")
        } else if request.owner != slot || !roster.contains(&owner) {
            Some("owner_mismatch")
        } else if request.row >= 13 || request.column >= 15 || !(1..=2).contains(&request.power) {
            Some("unsupported_cell_or_power")
        } else if self.serials.contains_key(&(owner, request.serial)) {
            Some(if self.serials[&(owner, request.serial)].0 == *request {
                "duplicate_serial"
            } else {
                "conflicting_serial"
            })
        } else if self
            .active
            .values()
            .filter(|a| a.ticket.owner == owner)
            .count()
            >= capacity
        {
            Some("owner_capacity_reserved")
        } else if self
            .active
            .values()
            .any(|a| (a.request.row, a.request.column) == (request.row, request.column))
        {
            Some("occupied_cell")
        } else {
            None
        };
        if let Some(reason) = reject {
            return Ok(Err(reason));
        }
        let id = u16::try_from(self.next_wire)
            .map_err(|_| io::Error::other("diagnostic bomb ID space exhausted"))?;
        if id == 0 {
            return Err(io::Error::other("diagnostic bomb ID zero"));
        }
        let allocation = self
            .next_allocation
            .checked_add(1)
            .ok_or_else(|| io::Error::other("bomb allocation exhausted"))?;
        let ticket = Ticket {
            scope: self.scope.clone(),
            round,
            allocation,
            id,
            owner,
        };
        let recipients: BTreeSet<_> = roster.iter().copied().collect();
        if !matches!(recipients.len(), 2 | 4) || recipients.len() != roster.len() {
            return Ok(Err("unsupported_roster"));
        }
        self.next_wire += 1;
        self.next_allocation = allocation;
        self.serials
            .insert((owner, request.serial), (request.clone(), id));
        self.active.insert(
            id,
            Active {
                ticket: ticket.clone(),
                request: request.clone(),
                pending: recipients,
            },
        );
        Ok(Ok(ticket))
    }
    pub fn deliverable(&self, ticket: &Ticket, recipient: Membership) -> bool {
        Arc::ptr_eq(&self.scope, &ticket.scope)
            && self.generation == Some(ticket.round)
            && self.active.get(&ticket.id).is_some_and(|a| {
                a.ticket.allocation == ticket.allocation
                    && a.ticket.owner == ticket.owner
                    && a.pending.contains(&recipient)
            })
    }
    /// Each actual TCP recipient consumes its own deadline once. Release the
    /// reservation only after ALL roster writes succeed; not a native-execution ACK.
    pub fn pending(&self) -> bool {
        !self.active.is_empty()
    }
    pub fn active(&self, ticket: &Ticket) -> bool {
        Arc::ptr_eq(&self.scope, &ticket.scope)
            && self.generation == Some(ticket.round)
            && self.active.get(&ticket.id).is_some_and(|a| {
                a.ticket.allocation == ticket.allocation && a.ticket.owner == ticket.owner
            })
    }
    pub fn delivered(&mut self, ticket: &Ticket, recipient: Membership) -> bool {
        if !self.deliverable(ticket, recipient) {
            return false;
        }
        let active = self.active.get_mut(&ticket.id).expect("current ticket");
        active.pending.remove(&recipient);
        if active.pending.is_empty() {
            self.active.remove(&ticket.id);
        }
        true
    }
}
pub(crate) fn explode158(id: u16) -> io::Result<Vec<u8>> {
    let mut out = Vec::with_capacity(13);
    for (v, w) in [
        (1, Width::U8),
        (u64::from(id), Width::U16),
        (0, Width::U8),
        (2, Width::U32),
        (u64::from(u32::MAX), Width::U32),
        (255, Width::U8),
    ] {
        out.extend(write_uint(v, w).map_err(io::Error::other)?);
    }
    Ok(out)
}
#[cfg(test)]
mod tests {
    use super::*;
    fn member(id: u32, conn: u64) -> Membership {
        Membership {
            identity: id,
            connection: conn,
            visit: 1,
        }
    }
    fn request(serial: u32) -> Request {
        Request {
            key: 12346,
            owner: 0,
            serial,
            row: 3,
            column: 3,
            power: 1,
        }
    }
    #[test]
    fn duplicate_conflict_owner_key_capacity_and_two_recipient_completion() {
        let (a, b) = (member(1, 11), member(2, 22));
        let roster = [a, b];
        let mut l = Ledger::default();
        l.begin(1);
        let mut bad = request(1);
        bad.key -= 1;
        assert!(matches!(
            l.place(1, a, 0, &bad, &roster, 1).unwrap(),
            Err("round_key_mismatch")
        ));
        assert!(matches!(
            l.place(1, a, 1, &request(1), &roster, 1).unwrap(),
            Err("owner_mismatch")
        ));
        let t = l.place(1, a, 0, &request(1), &roster, 1).unwrap().unwrap();
        assert_eq!(t.id, 1);
        assert!(matches!(
            l.place(1, a, 0, &request(1), &roster, 1).unwrap(),
            Err("duplicate_serial")
        ));
        let mut changed = request(1);
        changed.column = 4;
        assert!(matches!(
            l.place(1, a, 0, &changed, &roster, 1).unwrap(),
            Err("conflicting_serial")
        ));
        changed.serial = 2;
        assert!(matches!(
            l.place(1, a, 0, &changed, &roster, 1).unwrap(),
            Err("owner_capacity_reserved")
        ));
        assert!(l.delivered(&t, a));
        assert!(!l.delivered(&t, a));
        assert!(l.deliverable(&t, b));
        assert!(matches!(
            l.place(1, a, 0, &changed, &roster, 1).unwrap(),
            Err("owner_capacity_reserved")
        ));
        assert!(l.delivered(&t, b));
        assert!(!l.deliverable(&t, b));
        let next = l.place(1, a, 0, &changed, &roster, 1).unwrap().unwrap();
        assert_eq!(next.id, 2);
        assert_eq!(request(1).spawn159(t.id).unwrap().len(), 39);
        assert_eq!(explode158(t.id).unwrap().len(), 13);
    }
    #[test]
    fn closed_round_foreign_owner_and_old_deadline_cannot_adopt_new_allocation() {
        let (a, b) = (member(1, 11), member(2, 22));
        let roster = [a, b];
        let mut l = Ledger::default();
        l.begin(1);
        let old = l.place(1, a, 0, &request(1), &roster, 1).unwrap().unwrap();
        let mut other = Ledger::default();
        other.begin(1);
        let foreign = other
            .place(1, a, 0, &request(1), &roster, 1)
            .unwrap()
            .unwrap();
        assert_eq!(old.id, foreign.id);
        assert!(!other.deliverable(&old, a));
        l.finish();
        assert!(!l.deliverable(&old, a));
        l.begin(2);
        let mut current = request(1);
        current.key += 1;
        assert!(matches!(
            l.place(2, a, 0, &request(1), &roster, 1).unwrap(),
            Err("round_key_mismatch")
        ));
        let next = l.place(2, a, 0, &current, &roster, 1).unwrap().unwrap();
        assert_eq!(next.id, 2);
        assert!(!l.deliverable(&old, a));
        assert!(!l.delivered(&old, a));
        assert!(l.deliverable(&next, a));
        let reused = Membership {
            connection: 12,
            visit: 2,
            ..a
        };
        assert!(!l.deliverable(&next, reused));
    }
    #[test]
    fn four_owner_capacity_serial_and_all_recipient_completion_are_independent() {
        let roster = [member(1, 11), member(2, 22), member(3, 33), member(4, 44)];
        let mut ledger = Ledger::default();
        ledger.begin(1);
        let mut tickets = Vec::new();
        for (slot, owner) in roster.into_iter().enumerate() {
            let mut req = request(7);
            req.owner = slot as u8;
            req.column = 2 + slot as u8;
            let capacity = [1, 2, 1, 1][slot];
            let ticket = ledger
                .place(1, owner, slot as u8, &req, &roster, capacity)
                .unwrap()
                .unwrap();
            assert_eq!(ticket.id, 1 + slot as u16);
            assert!(matches!(
                ledger
                    .place(1, owner, slot as u8, &req, &roster, capacity)
                    .unwrap(),
                Err("duplicate_serial")
            ));
            tickets.push((ticket, req));
        }
        let mut d_next = tickets[3].1.clone();
        d_next.serial += 1;
        d_next.row += 1;
        assert!(matches!(
            ledger.place(1, roster[3], 3, &d_next, &roster, 1).unwrap(),
            Err("owner_capacity_reserved")
        ));
        let mut b_next = tickets[1].1.clone();
        b_next.serial += 1;
        b_next.row += 1;
        let b_extra = ledger
            .place(1, roster[1], 1, &b_next, &roster, 2)
            .unwrap()
            .unwrap();
        assert_eq!(b_extra.id, 5);
        let d = &tickets[3].0;
        for recipient in &roster[..3] {
            assert!(ledger.delivered(d, *recipient));
            assert!(!ledger.delivered(d, *recipient));
        }
        assert!(ledger.active(d));
        assert!(matches!(
            ledger.place(1, roster[3], 3, &d_next, &roster, 1).unwrap(),
            Err("owner_capacity_reserved")
        ));
        let replacement = Membership {
            connection: 45,
            visit: 2,
            ..roster[3]
        };
        assert!(!ledger.delivered(d, replacement));
        assert!(ledger.delivered(d, roster[3]));
        assert!(!ledger.active(d));
        let next = ledger
            .place(1, roster[3], 3, &d_next, &roster, 1)
            .unwrap()
            .unwrap();
        assert_eq!(next.id, 6);
        for (ticket, _) in &tickets[..3] {
            assert!(ledger.active(ticket));
        }
        ledger.finish();
        for recipient in roster {
            assert!(!ledger.deliverable(&next, recipient));
            assert!(!ledger.delivered(&b_extra, recipient));
        }
        ledger.begin(2);
        d_next.key += 1;
        let new = ledger
            .place(2, roster[3], 3, &d_next, &roster, 1)
            .unwrap()
            .unwrap();
        assert_eq!(new.id, 7);
        assert!(!ledger.delivered(&next, roster[3]));
        assert!(ledger.deliverable(&new, roster[3]));
    }
}
