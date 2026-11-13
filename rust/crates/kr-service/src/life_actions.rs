//! Company report-derived needle/rescue policy. KR wire from CP037/038/113–116.
//! This does not authenticate contact distance, inventory or original authority.
use crate::{
    life::{Decision, Player, Report},
    room::Membership,
};
use kr_wire::{Width, read_uint, write_uint};
use std::{collections::BTreeSet, io};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Policy {
    #[default]
    Disabled,
    Needle,
    Rescue,
    NeedleRescue,
}
impl Policy {
    pub(crate) fn needle(self) -> bool {
        matches!(self, Self::Needle | Self::NeedleRescue)
    }
    pub(crate) fn rescue(self) -> bool {
        matches!(self, Self::Rescue | Self::NeedleRescue)
    }
}
#[derive(Debug, Clone)]
pub(crate) enum Action {
    Stage([u8; 6]),
    Consume([u8; 5]),
    Rescue(u8, Vec<u8>),
}
impl Action {
    pub(crate) fn parse(report: &Report, body: &[u8]) -> io::Result<Self> {
        let err = || io::Error::other("unsupported measured life-action127 shape");
        match report.subtype {
            1 if body.len() == 14 => Ok(Self::Stage(body[..6].try_into().expect("prefix"))),
            0 if body.len() == 22 => {
                let mut tail = Vec::new();
                for (value, width) in [
                    (1, Width::U8),
                    (145, Width::U16),
                    (0, Width::U8),
                    (0, Width::U32),
                ] {
                    tail.extend(write_uint(value, width).map_err(io::Error::other)?);
                }
                if body[14..] != tail {
                    return Err(err());
                }
                Ok(Self::Consume(body[1..6].try_into().expect("prefix")))
            }
            18 if body.len() == 28 => {
                if read_uint(&body[14..18], Width::U32).map_err(io::Error::other)? != 10 {
                    return Err(err());
                }
                let raw = kr_wire::kr00967::decode_bytes(&body[18..])?;
                if raw.len() != 10 || raw[..6] != [24, 5, 0, 0, 0, 0] {
                    return Err(err());
                }
                let aux = u32::from_le_bytes(raw[6..10].try_into().expect("bounded"));
                if aux >= 8 {
                    return Err(err());
                }
                let mut forward = body[..6].to_vec();
                forward.extend(write_uint(u64::from(aux), Width::U8).map_err(io::Error::other)?);
                Ok(Self::Rescue(aux as u8, forward))
            }
            _ => Err(err()),
        }
    }
}
#[derive(Clone, Debug)]
struct Stage {
    trap: String,
    prefix: [u8; 6],
    tick: u32,
}
#[derive(Clone, Debug)]
struct Grant {
    digest: String,
    targets: BTreeSet<Membership>,
}
#[derive(Clone, Debug)]
struct Clear {
    trap: String,
    report: Report,
    targets: BTreeSet<Membership>,
    consume: bool,
}
#[derive(Clone, Debug, Default)]
pub(crate) struct State {
    credit: bool,
    credited: bool,
    grant: Option<Grant>,
    stage: Option<Stage>,
    clear: Option<Clear>,
}
impl State {
    pub(crate) fn interrupt(&mut self) {
        self.stage = None;
        self.clear = None;
    }
    pub(crate) fn clear_pending(&self) -> bool {
        self.clear.is_some()
    }
    pub(crate) fn delivery_pending(&self) -> bool {
        self.clear.is_some() || self.grant.is_some()
    }
}
impl Player {
    pub(crate) fn stage_needle_credit(
        &mut self,
        digest: String,
        targets: &[Membership],
    ) -> io::Result<()> {
        let unique: BTreeSet<_> = targets.iter().copied().collect();
        if self.actions.credited
            || self.actions.credit
            || self.actions.grant.is_some()
            || !(2..=8).contains(&targets.len())
            || unique.len() != targets.len()
        {
            return Err(io::Error::other("unsupported duplicate needle grant"));
        }
        self.actions.grant = Some(Grant {
            digest,
            targets: unique,
        });
        Ok(())
    }
    pub(crate) fn needle_credit_delivered(&mut self, digest: &str, recipient: Membership) {
        if self
            .actions
            .grant
            .as_mut()
            .filter(|g| g.digest == digest)
            .is_some_and(|g| g.targets.remove(&recipient) && g.targets.is_empty())
        {
            self.actions.grant = None;
            self.actions.credit = true;
            self.actions.credited = true;
        }
    }
    pub(crate) fn clear_deliverable(&self, digest: &str) -> bool {
        self.actions.clear.as_ref().is_some_and(|c| {
            c.report.digest == digest
                && self.death.is_none()
                && self
                    .trap
                    .as_ref()
                    .is_some_and(|t| t.subtype == 17 && t.digest == c.trap)
        })
    }
    pub(crate) fn action(
        &mut self,
        slot: u8,
        report: &Report,
        action: Action,
        policy: Policy,
        rescuer_eligible: bool,
        targets: &[Membership],
    ) -> (Decision, Option<Vec<u8>>) {
        if let Some(reason) = self.check_report(slot, report) {
            return (report.decision(reason, false), None);
        }
        if self.actions.clear.is_some() {
            return (report.decision("life_action_delivery_pending", false), None);
        }
        if self.death.is_some() {
            return (report.decision("life_action_actor_dead", false), None);
        }
        let Some(trap) = self
            .trap
            .as_ref()
            .filter(|t| t.subtype == 17)
            .map(|t| t.digest.clone())
        else {
            return (report.decision("life_action_no_current_trap", false), None);
        };
        let result =
            match action {
                Action::Stage(prefix) if policy.needle() => {
                    if !self.actions.credit {
                        return (report.decision("needle_no_credit", false), None);
                    }
                    if self.actions.stage.is_some() {
                        return (report.decision("needle_pair_already_pending", false), None);
                    }
                    self.actions.stage = Some(Stage {
                        trap,
                        prefix,
                        tick: report.tick,
                    });
                    self.record_report(report);
                    return (report.decision("accepted", false), None);
                }
                Action::Consume(prefix) if policy.needle() => {
                    if !self.actions.credit {
                        return (report.decision("needle_no_credit", false), None);
                    }
                    let Some(stage) = self.actions.stage.as_ref().filter(|s| {
                        s.trap == trap && s.tick == report.tick && s.prefix[1..] == prefix
                    }) else {
                        return (report.decision("needle_pair_mismatch", false), None);
                    };
                    (stage.prefix.to_vec(), true)
                }
                Action::Rescue(_, forward) if policy.rescue() => {
                    if !rescuer_eligible {
                        return (
                            report.decision("rescuer_not_current_friendly_eligible", false),
                            None,
                        );
                    }
                    (forward, false)
                }
                _ => return (report.decision("life_action_disabled", false), None),
            };
        let unique: BTreeSet<_> = targets.iter().copied().collect();
        if !(1..=7).contains(&targets.len()) || unique.len() != targets.len() {
            return (report.decision("life_action_recipient_roster", false), None);
        }
        self.actions.clear = Some(Clear {
            trap,
            report: report.clone(),
            targets: unique,
            consume: result.1,
        });
        (report.decision("accepted", true), Some(result.0))
    }
    pub(crate) fn action_delivered(&mut self, digest: &str, recipient: Membership) {
        if !self.clear_deliverable(digest) {
            return;
        }
        let complete = self
            .actions
            .clear
            .as_mut()
            .is_some_and(|c| c.targets.remove(&recipient) && c.targets.is_empty());
        if complete {
            self.complete_action();
        }
    }
    fn complete_action(&mut self) {
        if let Some(clear) = self.actions.clear.take() {
            if clear.consume {
                self.actions.credit = false;
            }
            self.actions.stage = None;
            self.trap = None;
            self.survivor = None;
            self.clear_result_witness();
            self.record_report(&clear.report);
        }
    }
    pub(crate) fn action_forget_recipient(&mut self, key: Membership) {
        if let Some(grant) = &self.actions.grant {
            let digest = grant.digest.clone();
            self.needle_credit_delivered(&digest, key);
        }
        if let Some(clear) = &self.actions.clear {
            let digest = clear.report.digest.clone();
            self.action_delivered(&digest, key);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;
    const TRAP: &str = "4b5aa511a46496ca59fb96ca539096ca539b96ca5a8d96c8539596ca5395a569";
    const ONE: &str = "5b5aa511a46496ca593596ca5390";
    const ZERO: &str = "5a5aa511a46496ca593596ca53935ba5f85a96ca5395";
    const RESCUE: &str = "485aa511a5f696ca591c96ca539096ca539f96ca568d96cb5395a569";
    fn bytes(s: &str) -> Vec<u8> {
        s.as_bytes()
            .as_chunks::<2>()
            .0
            .iter()
            .map(|x| u8::from_str_radix(std::str::from_utf8(x).unwrap(), 16).unwrap())
            .collect()
    }
    fn keys() -> [Membership; 3] {
        [
            Membership {
                identity: 70000,
                connection: 11,
                visit: 8,
            },
            Membership {
                identity: 42,
                connection: 12,
                visit: 9,
            },
            Membership {
                identity: 505,
                connection: 13,
                visit: 10,
            },
        ]
    }
    fn trapped() -> Player {
        let mut p = Player::default();
        let report = Report::parse(&bytes(TRAP)).unwrap();
        assert!(p.apply(0, &report, Instant::now(), None).accepted);
        p
    }
    fn credit(p: &mut Player) {
        let members = keys();
        p.stage_needle_credit("claim".into(), &members).unwrap();
        for key in members {
            p.needle_credit_delivered("claim", key);
        }
        assert!(p.actions.credit);
    }
    fn act(p: &mut Player, body: &[u8], eligible: bool) -> (Decision, Option<Vec<u8>>) {
        let r = Report::parse(body).unwrap();
        p.action(
            0,
            &r,
            Action::parse(&r, body).unwrap(),
            Policy::NeedleRescue,
            eligible,
            &keys()[1..],
        )
    }
    #[test]
    fn exact_old_native_vectors_decode_without_using_encoder_as_oracle() {
        // Exact captured CP037/CP036 bodies, retained source SHA in reuse ledger.
        assert_eq!(
            crate::sha256_hex(&bytes(ONE)),
            "f7fd28cd9b8f0867ca981dcedd3f9f36acc8973655505ffa8397fe4c1be434ff"
        );
        assert_eq!(
            crate::sha256_hex(&bytes(ZERO)),
            "51ba3db323df9c6b2cf93d92682d6a947df9dfa0d5d6f4349632c79d1d30e53a"
        );
        assert_eq!(
            crate::sha256_hex(&bytes(RESCUE)),
            "425736c6aea6201cf1860e4edbc0a250faa0c121d4af74e5a5c3394ac690c187"
        );
        let one = Report::parse(&bytes(ONE)).unwrap();
        let zero = Report::parse(&bytes(ZERO)).unwrap();
        assert_eq!(one.tick, zero.tick);
        assert!(matches!(
            Action::parse(&one, &bytes(ONE)).unwrap(),
            Action::Stage(_)
        ));
        assert!(matches!(
            Action::parse(&zero, &bytes(ZERO)).unwrap(),
            Action::Consume(_)
        ));
        let r = Report::parse(&bytes(RESCUE)).unwrap();
        let Action::Rescue(aux, forward) = Action::parse(&r, &bytes(RESCUE)).unwrap() else {
            panic!("rescue")
        };
        assert_eq!(aux, 1);
        assert_eq!(&forward[..6], &bytes(RESCUE)[..6]);
        assert_eq!(forward[6..], write_uint(1, Width::U8).unwrap());
    }
    #[test]
    fn credit_and_escape_commit_wait_for_all_exact_recipients_and_current_trap() {
        let mut p = trapped();
        let members = keys();
        p.stage_needle_credit("claim".into(), &members).unwrap();
        p.needle_credit_delivered("claim", members[0]);
        p.needle_credit_delivered("claim", members[1]);
        assert!(!p.actions.credit);
        assert!(!act(&mut p, &bytes(ONE), false).0.accepted);
        p.needle_credit_delivered(
            "claim",
            Membership {
                visit: 999,
                ..members[2]
            },
        );
        assert!(!p.actions.credit);
        p.needle_credit_delivered("claim", members[2]);
        assert!(p.actions.credit);
        assert!(act(&mut p, &bytes(ONE), false).0.accepted);
        let (d, forward) = act(&mut p, &bytes(ZERO), false);
        assert!(d.accepted && d.forward);
        assert_eq!(forward.unwrap(), bytes(ONE)[..6]);
        let r = Report::parse(&bytes(ZERO)).unwrap();
        assert!(p.clear_deliverable(&r.digest));
        p.action_delivered(&r.digest, members[1]);
        assert!(p.trap.is_some() && p.actions.credit);
        assert_eq!(
            act(&mut p, &bytes(ZERO), false).0.reason,
            "life_action_delivery_pending"
        );
        p.action_delivered(&r.digest, members[2]);
        assert!(p.trap.is_none() && !p.actions.credit && !p.delivery_pending());
        assert_eq!(
            act(&mut p, &bytes(ZERO), false).0.reason,
            "duplicate_report_body"
        );
        assert!(
            p.stage_needle_credit("duplicate-new-claim".into(), &members)
                .is_err()
        );
    }
    #[test]
    fn malformed_or_no_credit_future_report_cannot_poison_cursor_or_consume() {
        let mut p = trapped();
        let mut future = bytes(ONE);
        future[6..10].copy_from_slice(&write_uint(1_000_000, Width::U32).unwrap());
        assert_eq!(act(&mut p, &future, false).0.reason, "needle_no_credit");
        credit(&mut p);
        assert!(act(&mut p, &bytes(ONE), false).0.accepted);
        let mut bad = bytes(ZERO);
        bad[2] ^= 1;
        assert_eq!(act(&mut p, &bad, false).0.reason, "needle_pair_mismatch");
        assert!(p.actions.credit && p.trap.is_some());
        let mut bad_tail = bytes(ZERO);
        bad_tail[16] ^= 1;
        assert!(Action::parse(&Report::parse(&bad_tail).unwrap(), &bad_tail).is_err());
        assert!(act(&mut p, &bytes(ZERO), false).0.accepted);
    }
    #[test]
    fn new_trap_invalidates_staged_or_queued_escape_and_reset_clears_credit_preserves_cursor() {
        let mut p = trapped();
        credit(&mut p);
        assert!(act(&mut p, &bytes(ONE), false).0.accepted);
        assert!(act(&mut p, &bytes(ZERO), false).0.forward);
        let digest = Report::parse(&bytes(ZERO)).unwrap().digest;
        let mut next_trap = bytes(TRAP);
        next_trap[6..10].copy_from_slice(&write_uint(2721, Width::U32).unwrap());
        let r = Report::parse(&next_trap).unwrap();
        assert!(p.apply(0, &r, Instant::now(), None).accepted);
        assert!(!p.clear_deliverable(&digest));
        for k in &keys()[1..] {
            p.action_delivered(&digest, *k);
        }
        assert!(p.trap.is_some() && p.actions.credit);
        p.begin();
        assert!(
            p.trap.is_none() && !p.actions.credit && !p.actions.credited && !p.delivery_pending()
        );
        assert_eq!(
            act(&mut p, &bytes(ONE), false).0.reason,
            "report_tick_not_forward"
        );
    }
    #[test]
    fn friendly_rescue_clears_current_trap_after_delivery_without_consuming_needle() {
        let mut p = trapped();
        credit(&mut p);
        assert_eq!(
            act(&mut p, &bytes(RESCUE), false).0.reason,
            "rescuer_not_current_friendly_eligible"
        );
        let (d, body) = act(&mut p, &bytes(RESCUE), true);
        assert!(d.accepted && d.forward);
        assert_eq!(body.unwrap().len(), 7);
        let digest = Report::parse(&bytes(RESCUE)).unwrap().digest;
        p.action_delivered(&digest, keys()[1]);
        assert!(p.trap.is_some());
        p.action_forget_recipient(keys()[2]);
        assert!(p.trap.is_none() && p.actions.credit);
        assert_eq!(
            act(&mut p, &bytes(RESCUE), true).0.reason,
            "duplicate_report_body"
        );
    }
}
