//! Bounded fixture allocator. CP028/037/044/171/174/188–189 reference policies.
//! No general KR authority, chain, terrain, life, inventory, or result semantics.
use crate::room::{Membership, wire_key};
use kr_wire::{Width, read_uint};
use std::{
    collections::{BTreeMap, BTreeSet},
    io,
    sync::Arc,
    time::{Duration, Instant},
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
    pub(crate) fn cell(&self) -> (u16, u16) {
        (u16::from(self.row), u16::from(self.column))
    }
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
        kr_wire::kr00967::spawn159(
            self.owner,
            self.serial,
            self.row,
            self.column,
            self.power,
            id,
        )
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
#[derive(Clone, Copy)]
pub(crate) struct Limits {
    pub capacity: usize,
    pub maximum_power: u8,
}
#[derive(Clone, Debug)]
struct Active {
    ticket: Ticket,
    request: Request,
    pending: BTreeSet<Membership>,
    deadline: Instant,
    motion: Option<kr_wire::kr00967::BombMotion162>,
    preparing: BTreeSet<Membership>,
}
impl Active {
    /// Occupancy checked by new Shoe actions includes the complete path of a
    /// prior Punch, whose original source may still contain its moving sprite.
    /// Placement uses this corridor only for Shoe allocations below. Native stop
    /// has no wire acknowledgement; conservatively retain until retirement.
    fn reserves(&self, cell: (u8, u8)) -> bool {
        (self.request.row, self.request.column) == cell
            || self.motion.is_some_and(|m| {
                matches!(m.selector, 1 | 3 | 5)
                    && m.source_column == cell.1
                    && (m.source_row.min(m.target_row)..=m.source_row.max(m.target_row))
                        .contains(&cell.0)
            })
    }
}
#[derive(Clone, Debug)]
pub(crate) struct Ledger {
    scope: Arc<()>,
    generation: Option<u64>,
    next_wire: u32,
    next_allocation: u64,
    active: BTreeMap<u16, Active>,
    serials: BTreeMap<(Membership, u32), (Request, Option<u16>)>,
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
pub(crate) enum Placement {
    Allocated(Ticket),
    Cancelled,
}
pub(crate) struct Admission {
    pub limits: Limits,
    pub cancel_on_map: bool,
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
    pub(crate) fn forget_recipient(&mut self, key: Membership) -> Vec<Ticket> {
        let mut completed = Vec::new();
        for active in self.active.values_mut() {
            active.pending.remove(&key);
            if active.pending.is_empty() {
                completed.push(active.ticket.clone());
            }
        }
        self.active.retain(|_, active| !active.pending.is_empty());
        completed
    }
    /// Operates on the candidate room clone; reserve ALL spawn+deadline queues
    /// before publishing this allocation/serial/reservation to the real owner.
    #[cfg(test)]
    pub fn place(
        &mut self,
        round: u64,
        owner: Membership,
        slot: u8,
        request: &Request,
        roster: &[Membership],
        capacity: usize,
    ) -> io::Result<std::result::Result<Ticket, &'static str>> {
        self.place_measured(
            round,
            owner,
            slot,
            request,
            roster,
            Limits {
                capacity,
                maximum_power: 2,
            },
        )
    }
    #[cfg(test)]
    pub(crate) fn place_measured(
        &mut self,
        round: u64,
        owner: Membership,
        slot: u8,
        request: &Request,
        roster: &[Membership],
        limits: Limits,
    ) -> io::Result<std::result::Result<Ticket, &'static str>> {
        match self.admit(
            round,
            owner,
            slot,
            request,
            roster,
            Admission {
                limits,
                cancel_on_map: false,
            },
        )? {
            Ok(Placement::Allocated(ticket)) => Ok(Ok(ticket)),
            Ok(Placement::Cancelled) => {
                Err(io::Error::other("unexpected ordinary bomb cancellation"))
            }
            Err(reason) => Ok(Err(reason)),
        }
    }
    /// Runs on the candidate room clone. Cancelled requests consume their
    /// serial but allocate no wire ID, quota, recipient set, or deadline.
    pub(crate) fn admit(
        &mut self,
        round: u64,
        owner: Membership,
        slot: u8,
        request: &Request,
        roster: &[Membership],
        admission: Admission,
    ) -> io::Result<std::result::Result<Placement, &'static str>> {
        let limits = admission.limits;
        if !(1..=10).contains(&limits.capacity) || !(1..=9).contains(&limits.maximum_power) {
            return Err(io::Error::other("unsupported measured bomb limits"));
        }
        let reject = if self.generation != Some(round) {
            Some("inactive_round")
        } else if request.key != wire_key(round)? {
            Some("round_key_mismatch")
        } else if request.owner != slot || !roster.contains(&owner) {
            Some("owner_mismatch")
        } else if request.row >= 13
            || request.column >= 15
            || !(1..=limits.maximum_power).contains(&request.power)
        {
            Some("unsupported_cell_or_power")
        } else if self.serials.contains_key(&(owner, request.serial)) {
            Some(if self.serials[&(owner, request.serial)].0 == *request {
                "duplicate_serial"
            } else {
                "conflicting_serial"
            })
        } else if admission.cancel_on_map
            && self
                .serials
                .iter()
                .filter(|((member, _), (_, id))| *member == owner && id.is_none())
                .count()
                >= 64
        {
            Some("map_cancellation_budget")
        } else if self
            .active
            .values()
            .filter(|a| a.ticket.owner == owner)
            .count()
            >= limits.capacity
        {
            Some("owner_capacity_reserved")
        } else if self.active.values().any(|a| {
            a.motion.is_some_and(|m| m.selector == 5) && a.reserves((request.row, request.column))
        }) {
            Some("motion_path_reserved")
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
        let recipients: BTreeSet<_> = roster.iter().copied().collect();
        if !(2..=8).contains(&recipients.len()) || recipients.len() != roster.len() {
            return Ok(Err("unsupported_roster"));
        }
        if admission.cancel_on_map {
            self.serials
                .insert((owner, request.serial), (request.clone(), None));
            return Ok(Ok(Placement::Cancelled));
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
        self.next_wire += 1;
        self.next_allocation = allocation;
        self.serials
            .insert((owner, request.serial), (request.clone(), Some(id)));
        self.active.insert(
            id,
            Active {
                ticket: ticket.clone(),
                request: request.clone(),
                pending: recipients,
                deadline: Instant::now() + Duration::from_secs(6),
                motion: None,
                preparing: BTreeSet::new(),
            },
        );
        Ok(Ok(Placement::Allocated(ticket)))
    }
    /// Candidate owner-only vertical Punch policy, not original KR authority.
    /// Selector1 emission was captured at CP327; native execution acceptance
    /// for down-one remains pending in KRE-G1-PUNCH-DOWN (#456).
    /// Wire119 has no epoch: current non-reused ID and Membership fence it.
    pub(crate) fn move_punch(
        &mut self,
        round: u64,
        actor: Membership,
        slot: u8,
        request: kr_wire::kr00967::BombAction119,
        now: Instant,
    ) -> Result<Ticket, &'static str> {
        self.move_action(round, actor, slot, request, now, false)
    }
    /// CP328 measured three-cell upward contact; all further geometry stays off.
    pub(crate) fn move_shoe(
        &mut self,
        round: u64,
        actor: Membership,
        slot: u8,
        request: kr_wire::kr00967::BombAction119,
        now: Instant,
    ) -> Result<Ticket, &'static str> {
        self.move_action(round, actor, slot, request, now, true)
    }
    fn move_action(
        &mut self,
        round: u64,
        actor: Membership,
        slot: u8,
        request: kr_wire::kr00967::BombAction119,
        now: Instant,
        shoe: bool,
    ) -> Result<Ticket, &'static str> {
        let motion = request.motion;
        if self.generation != Some(round) {
            return Err("inactive_round");
        }
        if motion.context != 0 || motion.slot != slot {
            return Err("action_actor_mismatch");
        }
        // Explicit captured direction/cell relationships only. The down-one
        // branch is a candidate until the matching native execution is accepted.
        let target_row = match (shoe, motion.selector) {
            (true, 5) => motion.source_row.checked_sub(3),
            (false, 3) => motion.source_row.checked_sub(1),
            (false, 1) => motion.source_row.checked_add(1),
            _ => None,
        };
        if target_row != Some(motion.target_row)
            || motion.source_column != motion.target_column
            || motion.source_row >= 13
            || motion.target_row >= 13
            || motion.source_column >= 15
        {
            return Err("unmeasured_motion_geometry");
        }
        if self.active.values().any(|other| {
            other.ticket.id != motion.bomb_id
                && if shoe {
                    (motion.target_row..=motion.source_row)
                        .any(|row| other.reserves((row, motion.target_column)))
                } else {
                    other.motion.is_some_and(|m| m.selector == 5)
                        && other.reserves((motion.target_row, motion.target_column))
                }
        }) {
            return Err("motion_path_occupied");
        }
        let active = self
            .active
            .get_mut(&motion.bomb_id)
            .ok_or("inactive_bomb")?;
        if active.ticket.owner != actor {
            return Err("foreign_bomb");
        }
        if active.motion.is_some() {
            return Err("bomb_already_moved");
        }
        if !active.preparing.is_empty()
            || active.deadline.saturating_duration_since(now) < Duration::from_secs(3)
        {
            return Err("motion_deadline_closed");
        }
        if (active.request.row, active.request.column) != (motion.source_row, motion.source_column)
        {
            return Err("motion_source_mismatch");
        }
        active.request.row = motion.target_row;
        active.request.column = motion.target_column;
        active.motion = Some(motion);
        Ok(active.ticket.clone())
    }
    /// Close action admission before the first recipient starts its blast write.
    /// The stored deadline, owner, quota and allocation remain unchanged.
    pub(crate) fn prepare_blast(
        &mut self,
        ticket: &Ticket,
        recipient: Membership,
    ) -> io::Result<bool> {
        if !self.deliverable(ticket, recipient) {
            return Err(io::Error::other("stale bomb preparation"));
        }
        let active = self.active.get_mut(&ticket.id).expect("current ticket");
        if !active.preparing.insert(recipient) {
            return Err(io::Error::other("duplicate bomb preparation"));
        }
        Ok(active.motion.is_some())
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
pub(crate) use kr_wire::kr00967::explode158;
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
    fn shoe_corridor_blocks_peer_placement_and_crossing_punch_until_all_receipts() {
        use kr_wire::kr00967::{BombAction119, BombMotion162};
        let a = member(70000, 11);
        let b = member(80000, 22);
        let roster = [a, b];
        let mut ledger = Ledger::default();
        ledger.begin(1);
        let own = Request {
            row: 6,
            ..request(0)
        };
        let peer = Request {
            owner: 1,
            row: 7,
            ..request(0)
        };
        let first = ledger.place(1, a, 0, &own, &roster, 2).unwrap().unwrap();
        let second = ledger.place(1, b, 1, &peer, &roster, 2).unwrap().unwrap();
        let action = BombAction119 {
            motion: BombMotion162 {
                context: 0,
                slot: 0,
                selector: 5,
                bomb_id: first.id,
                source_row: 6,
                source_column: 3,
                target_row: 3,
                target_column: 3,
            },
            opaque_tail: [1000, 3],
        };
        let deadline = ledger.active[&first.id].deadline;
        assert_eq!(
            ledger
                .move_shoe(1, a, 0, action, deadline - Duration::from_secs(2))
                .unwrap_err(),
            "motion_deadline_closed"
        );
        ledger
            .move_shoe(1, a, 0, action, deadline - Duration::from_secs(5))
            .unwrap();
        let crossing = BombAction119 {
            motion: BombMotion162 {
                context: 0,
                slot: 1,
                selector: 3,
                bomb_id: second.id,
                source_row: 7,
                source_column: 3,
                target_row: 6,
                target_column: 3,
            },
            opaque_tail: [1715, 1],
        };
        assert_eq!(
            ledger
                .move_punch(1, b, 1, crossing, Instant::now())
                .unwrap_err(),
            "motion_path_occupied"
        );
        let placement = Request {
            row: 4,
            serial: 1,
            ..peer
        };
        assert_eq!(
            ledger
                .place(1, b, 1, &placement, &roster, 2)
                .unwrap()
                .unwrap_err(),
            "motion_path_reserved"
        );
        assert!(ledger.delivered(&first, a));
        assert_eq!(
            ledger
                .place(1, b, 1, &placement, &roster, 2)
                .unwrap()
                .unwrap_err(),
            "motion_path_reserved"
        );
        assert!(ledger.delivered(&first, b));
        assert!(
            ledger
                .place(1, b, 1, &placement, &roster, 2)
                .unwrap()
                .is_ok()
        );
        // A new round clears every old corridor, but old wire IDs stay retired.
        ledger.begin(2);
        assert_eq!(
            ledger
                .move_shoe(2, a, 0, action, Instant::now())
                .unwrap_err(),
            "inactive_bomb"
        );
    }
    #[test]
    fn shoe_cannot_enter_the_vacated_source_of_a_still_active_punch() {
        use kr_wire::kr00967::{BombAction119, BombMotion162};
        let a = member(70000, 11);
        let b = member(80000, 22);
        let roster = [a, b];
        let mut ledger = Ledger::default();
        ledger.begin(1);
        let first = ledger
            .place(
                1,
                a,
                0,
                &Request {
                    row: 6,
                    ..request(0)
                },
                &roster,
                2,
            )
            .unwrap()
            .unwrap();
        let second = ledger
            .place(
                1,
                b,
                1,
                &Request {
                    row: 9,
                    owner: 1,
                    ..request(0)
                },
                &roster,
                2,
            )
            .unwrap()
            .unwrap();
        ledger
            .move_punch(
                1,
                a,
                0,
                BombAction119 {
                    motion: BombMotion162 {
                        context: 0,
                        slot: 0,
                        selector: 3,
                        bomb_id: first.id,
                        source_row: 6,
                        source_column: 3,
                        target_row: 5,
                        target_column: 3,
                    },
                    opaque_tail: [1715, 1],
                },
                Instant::now(),
            )
            .unwrap();
        let action = BombAction119 {
            motion: BombMotion162 {
                context: 0,
                slot: 1,
                selector: 5,
                bomb_id: second.id,
                source_row: 9,
                source_column: 3,
                target_row: 6,
                target_column: 3,
            },
            opaque_tail: [1000, 3],
        };
        assert_eq!(
            ledger
                .move_shoe(1, b, 1, action, Instant::now())
                .unwrap_err(),
            "motion_path_occupied"
        );
        assert!(ledger.delivered(&first, a));
        assert!(ledger.delivered(&first, b));
        assert!(ledger.move_shoe(1, b, 1, action, Instant::now()).is_ok());
    }
    #[test]
    fn downward_punch_keeps_vacated_source_reserved_for_new_shoe_and_rejects_outside_map() {
        use kr_wire::kr00967::{BombAction119, BombMotion162};
        let a = member(70000, 11);
        let b = member(80000, 22);
        let mut ledger = Ledger::default();
        ledger.begin(1);
        let ticket = ledger
            .place(
                1,
                a,
                0,
                &Request {
                    row: 6,
                    ..request(0)
                },
                &[a, b],
                2,
            )
            .unwrap()
            .unwrap();
        let action = BombAction119 {
            motion: BombMotion162 {
                context: 0,
                slot: 0,
                selector: 1,
                bomb_id: ticket.id,
                source_row: 6,
                source_column: 3,
                target_row: 7,
                target_column: 3,
            },
            opaque_tail: [1715, 1],
        };
        for (selector, source, target) in
            [(1, 12, 13), (1, 255, 0), (1, 6, 8), (0, 6, 7), (2, 6, 7)]
        {
            let mut invalid = action;
            invalid.motion.selector = selector;
            invalid.motion.source_row = source;
            invalid.motion.target_row = target;
            assert_eq!(
                ledger
                    .move_punch(1, a, 0, invalid, Instant::now())
                    .unwrap_err(),
                "unmeasured_motion_geometry"
            );
        }
        ledger.move_punch(1, a, 0, action, Instant::now()).unwrap();
        // Source6 is now free in the ledger root projection; a new Shoe may not
        // enter it while the previous Punch could still be crossing it.
        let second = ledger
            .place(
                1,
                b,
                1,
                &Request {
                    row: 6,
                    owner: 1,
                    ..request(0)
                },
                &[a, b],
                2,
            )
            .unwrap()
            .unwrap();
        let shoe = BombAction119 {
            motion: BombMotion162 {
                context: 0,
                slot: 1,
                selector: 5,
                bomb_id: second.id,
                source_row: 6,
                source_column: 3,
                target_row: 3,
                target_column: 3,
            },
            opaque_tail: [1000, 3],
        };
        assert_eq!(
            ledger.move_shoe(1, b, 1, shoe, Instant::now()).unwrap_err(),
            "motion_path_occupied"
        );
        assert!(ledger.delivered(&ticket, a));
        assert_eq!(
            ledger.move_shoe(1, b, 1, shoe, Instant::now()).unwrap_err(),
            "motion_path_occupied"
        );
        assert!(ledger.delivered(&ticket, b));
        assert!(ledger.move_shoe(1, b, 1, shoe, Instant::now()).is_ok());
    }
    #[test]
    fn punch_admission_closes_before_deadline_or_first_blast_write_and_never_reuses_ids() {
        use kr_wire::kr00967::{BombAction119, BombMotion162};
        let a = member(70000, 11);
        let b = member(80000, 22);
        let mut ledger = Ledger::default();
        ledger.begin(1);
        let mut original = request(1);
        original.row = 6;
        let ticket = ledger
            .place(1, a, 0, &original, &[a, b], 1)
            .unwrap()
            .unwrap();
        let action = BombAction119 {
            motion: BombMotion162 {
                context: 0,
                slot: 0,
                selector: 3,
                bomb_id: ticket.id,
                source_row: 6,
                source_column: 3,
                target_row: 5,
                target_column: 3,
            },
            opaque_tail: [1715, 1],
        };
        let deadline = ledger.active[&ticket.id].deadline;
        assert!(matches!(
            ledger.move_punch(1, a, 0, action, deadline - Duration::from_secs(2)),
            Err("motion_deadline_closed")
        ));
        assert_eq!(ledger.active[&ticket.id].request, original);
        assert!(!ledger.prepare_blast(&ticket, b).unwrap());
        assert!(ledger.prepare_blast(&ticket, b).is_err());
        assert!(matches!(
            ledger.move_punch(1, a, 0, action, deadline - Duration::from_secs(5)),
            Err("motion_deadline_closed")
        ));
        ledger.begin(2);
        assert!(matches!(
            ledger.move_punch(2, a, 0, action, Instant::now()),
            Err("inactive_bomb")
        ));
        original.key += 1;
        let fresh = ledger
            .place(2, a, 0, &original, &[a, b], 1)
            .unwrap()
            .unwrap();
        assert!(fresh.id > ticket.id);
        let mut fresh_action = action;
        fresh_action.motion.bomb_id = fresh.id;
        let replacement = Membership {
            connection: 12,
            visit: 2,
            ..a
        };
        assert!(matches!(
            ledger.move_punch(2, replacement, 0, fresh_action, Instant::now()),
            Err("foreign_bomb")
        ));
        let fresh_deadline = ledger.active[&fresh.id].deadline;
        assert!(
            ledger
                .move_punch(2, a, 0, fresh_action, Instant::now())
                .is_ok()
        );
        assert_eq!(ledger.active[&fresh.id].deadline, fresh_deadline);
        assert_eq!(ledger.active[&fresh.id].ticket.owner, a);
        assert_eq!(ledger.active[&fresh.id].request.power, original.power);
        assert_eq!(ledger.serials[&(a, original.serial)].0, original);
    }
    #[test]
    fn cancelled_serial_is_bounded_without_wire_id_or_quota_and_fenced_by_round_and_visit() {
        let a = member(1, 11);
        let b = member(2, 22);
        let roster = [a, b];
        let mut ledger = Ledger::default();
        ledger.begin(1);
        let cancel = || Admission {
            limits: Limits {
                capacity: 1,
                maximum_power: 2,
            },
            cancel_on_map: true,
        };
        for serial in 1..=64 {
            let mut req = request(1);
            req.serial = serial;
            assert!(matches!(
                ledger.admit(1, a, 0, &req, &roster, cancel()).unwrap(),
                Ok(Placement::Cancelled)
            ));
        }
        assert!(!ledger.pending());
        assert_eq!(ledger.next_wire, 1);
        let mut req = request(1);
        req.serial = 65;
        assert!(matches!(
            ledger.admit(1, a, 0, &req, &roster, cancel()).unwrap(),
            Err("map_cancellation_budget")
        ));
        req.serial = 1;
        assert!(matches!(
            ledger.admit(1, a, 0, &req, &roster, cancel()).unwrap(),
            Err("duplicate_serial")
        ));
        req.column += 1;
        assert!(matches!(
            ledger.admit(1, a, 0, &req, &roster, cancel()).unwrap(),
            Err("conflicting_serial")
        ));
        req.serial = 65;
        let ticket = ledger.place(1, a, 0, &req, &roster, 1).unwrap().unwrap();
        assert_eq!(ticket.id, 1);
        assert!(ledger.deliverable(&ticket, b));
        ledger.finish();
        ledger.begin(2);
        assert!(!ledger.deliverable(&ticket, b));
        assert!(matches!(
            ledger.admit(2, a, 0, &req, &roster, cancel()).unwrap(),
            Err("round_key_mismatch")
        ));
        let new_visit = Membership {
            visit: a.visit + 1,
            ..a
        };
        let mut current = request(1);
        current.key = wire_key(2).unwrap();
        assert!(matches!(
            ledger
                .admit(2, a, 0, &current, &[new_visit, b], cancel())
                .unwrap(),
            Err("owner_mismatch")
        ));
        assert!(matches!(
            ledger
                .admit(2, new_visit, 0, &current, &[new_visit, b], cancel())
                .unwrap(),
            Ok(Placement::Cancelled)
        ));
    }

    #[test]
    fn eight_completion_requires_all_distinct_current_recipients_and_fences_old_ticket() {
        let roster: [Membership; 8] = std::array::from_fn(|i| member(i as u32 + 1, i as u64 + 11));
        let mut ledger = Ledger::default();
        ledger.begin(1);
        let t = ledger
            .place(1, roster[0], 0, &request(1), &roster, 1)
            .unwrap()
            .unwrap();
        for recipient in &roster[..7] {
            assert!(ledger.delivered(&t, *recipient));
            assert!(!ledger.delivered(&t, *recipient));
            assert!(ledger.pending());
        }
        let mut wrong = roster[7];
        wrong.visit += 1;
        assert!(!ledger.delivered(&t, wrong));
        assert!(ledger.pending());
        assert!(ledger.delivered(&t, roster[7]));
        assert!(!ledger.pending());
        ledger.finish();
        ledger.begin(2);
        assert!(!ledger.delivered(&t, roster[7]));
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
