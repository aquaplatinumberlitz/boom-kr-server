//! A per-round endpoint snapshot. This preserves LegacyDirect wire behavior;
//! it is neither a UDP relay nor NAT discovery or account authentication.
use crate::room::Membership;
use std::io;

pub(crate) struct LegacyDirectPlan {
    round: u64,
    members: Vec<Membership>,
    endpoints: Vec<(u32, u16)>,
}
impl LegacyDirectPlan {
    pub(crate) fn new(
        round: u64,
        capacity: usize,
        rows: impl IntoIterator<Item = (Membership, u8, u32, u16)>,
    ) -> io::Result<Self> {
        let mut slots = vec![None; capacity];
        let mut members = Vec::with_capacity(capacity);
        for (member, slot, ip, port) in rows {
            let row = slots
                .get_mut(usize::from(slot))
                .ok_or_else(|| io::Error::other("endpoint slot outside room"))?;
            if row.is_some() || members.contains(&member) || ip == 0 || port == 0 {
                return Err(io::Error::other("invalid legacy endpoint roster"));
            }
            *row = Some((ip, port));
            members.push(member);
        }
        if round == 0 || capacity == 0 || slots.iter().any(Option::is_none) {
            return Err(io::Error::other("incomplete legacy endpoint plan"));
        }
        Ok(Self {
            round,
            members,
            endpoints: slots.into_iter().map(Option::unwrap).collect(),
        })
    }
    pub(crate) fn for_recipient(
        &self,
        round: u64,
        member: Membership,
    ) -> io::Result<&[(u32, u16)]> {
        if self.round != round || !self.members.contains(&member) {
            return Err(io::Error::other("stale endpoint plan recipient or round"));
        }
        Ok(&self.endpoints)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn member(identity: u32, connection: u64, visit: u64) -> Membership {
        Membership {
            identity,
            connection,
            visit,
        }
    }
    #[test]
    fn shared_ip_is_not_identity_and_slot_is_not_player_id() {
        let a = member(70000, 11, 1);
        let b = member(42, 22, 2);
        let ip = u32::from_be_bytes([127, 0, 0, 2]);
        let plan = LegacyDirectPlan::new(3, 2, [(a, 1, ip, 9001), (b, 0, ip, 9002)]).unwrap();
        assert_eq!(plan.for_recipient(3, a).unwrap(), &[(ip, 9002), (ip, 9001)]);
        assert_eq!(
            plan.for_recipient(3, b).unwrap(),
            plan.for_recipient(3, a).unwrap()
        );
        assert!(plan.for_recipient(4, a).is_err());
        assert!(plan.for_recipient(3, member(70000, 33, 1)).is_err());
        assert!(plan.for_recipient(3, member(70000, 11, 3)).is_err());
    }
    #[test]
    fn incomplete_duplicate_and_missing_address_plans_are_rejected_before_publish() {
        let a = member(1, 11, 1);
        let b = member(2, 22, 2);
        let valid = [(a, 0, 1, 9001), (b, 1, 1, 9002)];
        assert!(LegacyDirectPlan::new(1, 2, valid).is_ok());
        assert!(LegacyDirectPlan::new(0, 2, valid).is_err());
        assert!(LegacyDirectPlan::new(1, 2, [valid[0]]).is_err());
        assert!(LegacyDirectPlan::new(1, 2, [valid[0], (b, 0, 1, 9002)]).is_err());
        assert!(LegacyDirectPlan::new(1, 2, [valid[0], (a, 1, 1, 9002)]).is_err());
        assert!(LegacyDirectPlan::new(1, 2, [valid[0], (b, 2, 1, 9002)]).is_err());
        assert!(LegacyDirectPlan::new(1, 2, [valid[0], (b, 1, 0, 9002)]).is_err());
        assert!(LegacyDirectPlan::new(1, 2, [valid[0], (b, 1, 1, 0)]).is_err());
    }
}
