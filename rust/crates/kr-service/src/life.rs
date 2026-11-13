//! Report-derived Battle compatibility policy from CP023/102/103/188/189.
//! A forward client tick is not authenticated life, location, or a wire epoch.
#[cfg(test)]
use kr_wire::{Width, write_uint};
use std::{
    collections::{BTreeMap, BTreeSet},
    io,
    time::Instant,
};
#[derive(Clone, Debug)]
pub(crate) struct Report {
    pub subtype: u8,
    pub slot: u8,
    pub tick: u32,
    pub digest: String,
    pub forward: Option<Vec<u8>>,
}

impl Report {
    pub fn parse(body: &[u8]) -> io::Result<Self> {
        let parsed = kr_wire::kr00967::report127(body)?;
        Ok(Self {
            subtype: parsed.subtype,
            slot: parsed.slot,
            tick: parsed.tick,
            digest: crate::sha256_hex(body),
            forward: parsed.forward,
        })
    }
}
#[derive(Clone, Debug)]
struct Cursor {
    tick: u32,
    bodies: BTreeSet<String>,
}
#[derive(Clone, Debug)]
pub(crate) struct Trap {
    pub digest: String,
    pub at: Instant,
    pub subtype: u8,
}
#[derive(Clone, Debug, Default)]
pub(crate) struct Player {
    pub(crate) actions: crate::life_actions::State,
    cursor: Option<Cursor>,
    pub trap: Option<Trap>,
    pub death: Option<Instant>,
    pub survivor: Option<Instant>,
    forwarded21: Option<String>,
    pending21: Option<(String, Option<BTreeSet<crate::room::Membership>>)>,
    reports: u32,
    deliveries: BTreeMap<String, BTreeSet<crate::room::Membership>>,
}
#[derive(Clone, Debug, serde::Serialize)]
pub struct Decision {
    pub accepted: bool,
    pub reason: &'static str,
    pub subtype: u8,
    pub slot: u8,
    pub tick: u32,
    pub forward: bool,
}
impl Report {
    pub fn decision(&self, reason: &'static str, forward: bool) -> Decision {
        Decision {
            accepted: reason == "accepted",
            reason,
            subtype: self.subtype,
            slot: self.slot,
            tick: self.tick,
            forward,
        }
    }
}
impl Player {
    pub(crate) fn check_report(&self, slot: u8, report: &Report) -> Option<&'static str> {
        let mut reason = "accepted";
        if report.slot != slot {
            reason = "slot_mismatch";
        } else if self.reports >= 4096 {
            reason = "report_budget";
        } else if let Some(c) = &self.cursor {
            let delta = report.tick.wrapping_sub(c.tick);
            if delta >= 0x8000_0000 {
                reason = "report_tick_not_forward";
            } else if delta == 0 && c.bodies.contains(&report.digest) {
                reason = "duplicate_report_body";
            } else if delta == 0 && c.bodies.len() >= 64 {
                reason = "same_tick_report_budget";
            }
        }
        (reason != "accepted").then_some(reason)
    }
    pub(crate) fn record_report(&mut self, report: &Report) {
        if self.cursor.as_ref().is_none_or(|c| c.tick != report.tick) {
            self.cursor = Some(Cursor {
                tick: report.tick,
                bodies: BTreeSet::new(),
            });
        }
        self.cursor
            .as_mut()
            .expect("cursor")
            .bodies
            .insert(report.digest.clone());
        self.reports += 1;
    }
    pub(crate) fn clear_result_witness(&mut self) {
        self.pending21 = None;
        self.forwarded21 = None;
    }
    pub fn begin(&mut self) {
        let cursor = self.cursor.take();
        *self = Self {
            cursor,
            ..Self::default()
        };
    }
    pub fn blocked_pickup(&self) -> bool {
        self.trap.is_some() || self.death.is_some()
    }
    pub fn apply(
        &mut self,
        slot: u8,
        report: &Report,
        now: Instant,
        last_blast: Option<Instant>,
    ) -> Decision {
        let mut reason = self.check_report(slot, report).unwrap_or("accepted");
        if self.actions.clear_pending() && !matches!(report.subtype, 17 | 19) {
            reason = "life_action_delivery_pending";
        }
        if reason == "accepted" && report.subtype == 19 {
            if self.death.is_some() {
                reason = "death_already_accepted";
            } else if self.trap.as_ref().is_none_or(|t| t.subtype != 17) {
                reason = "death_no_current_trap";
            }
        }
        if reason != "accepted" {
            return report.decision(reason, false);
        }
        self.record_report(report);
        let forward = match report.subtype {
            17 | 19 => {
                self.actions.interrupt();
                self.trap = Some(Trap {
                    digest: report.digest.clone(),
                    at: now,
                    subtype: report.subtype,
                });
                self.pending21 = None;
                if report.subtype == 19 {
                    self.death = Some(now);
                }
                true
            }
            21 => {
                self.survivor = Some(now);
                if self.current_survivor(now, last_blast)
                    && self
                        .trap
                        .as_ref()
                        .is_some_and(|t| self.forwarded21.as_ref() != Some(&t.digest))
                    && self.pending21.is_none()
                {
                    self.pending21 = Some((report.digest.clone(), None));
                    true
                } else {
                    false
                }
            }
            _ => false,
        };
        report.decision("accepted", forward)
    }
    pub fn current_survivor(&self, at: Instant, last_blast: Option<Instant>) -> bool {
        self.death.is_none()
            && self
                .trap
                .as_ref()
                .is_none_or(|t| t.subtype == 17 && at > t.at)
            && last_blast.is_none_or(|b| at > b)
    }
    pub fn forward21_valid(&self, digest: &str, at: Instant, last_blast: Option<Instant>) -> bool {
        self.pending21.as_ref().is_some_and(|p| p.0 == digest)
            && self.current_survivor(at, last_blast)
    }
    /// Bind completion to every distinct current recipient, not merely one send.
    pub fn forward21_targets(
        &mut self,
        digest: &str,
        targets: &[crate::room::Membership],
    ) -> io::Result<()> {
        let unique: BTreeSet<_> = targets.iter().copied().collect();
        if !(1..=7).contains(&targets.len()) || unique.len() != targets.len() {
            return Err(io::Error::other("unsupported life clear recipient roster"));
        }
        let pending = self
            .pending21
            .as_mut()
            .filter(|p| p.0 == digest && p.1.is_none())
            .ok_or_else(|| io::Error::other("missing current life clear"))?;
        pending.1 = Some(unique);
        Ok(())
    }
    pub fn forward_report_targets(
        &mut self,
        digest: &str,
        targets: &[crate::room::Membership],
    ) -> io::Result<()> {
        let unique: BTreeSet<_> = targets.iter().copied().collect();
        if !(1..=7).contains(&targets.len())
            || unique.len() != targets.len()
            || self.deliveries.contains_key(digest)
        {
            return Err(io::Error::other(
                "unsupported or duplicate life recipient roster",
            ));
        }
        self.deliveries.insert(digest.to_owned(), unique);
        Ok(())
    }
    pub fn report_delivered(&mut self, digest: &str, recipient: crate::room::Membership) {
        if let Some(targets) = self.deliveries.get_mut(digest) {
            if targets.remove(&recipient) && targets.is_empty() {
                self.deliveries.remove(digest);
            }
        } else {
            self.forward21_delivered(digest, recipient);
        }
    }
    pub fn delivery_pending(&self) -> bool {
        !self.deliveries.is_empty() || self.pending21.is_some() || self.actions.delivery_pending()
    }
    pub(crate) fn forget_recipient(&mut self, key: crate::room::Membership) {
        self.action_forget_recipient(key);
        for targets in self.deliveries.values_mut() {
            targets.remove(&key);
        }
        self.deliveries.retain(|_, targets| !targets.is_empty());
        if let Some((digest, Some(targets))) = &mut self.pending21 {
            targets.remove(&key);
            if targets.is_empty() {
                self.forwarded21 = self.trap.as_ref().map(|t| t.digest.clone());
                let _ = digest;
                self.pending21 = None;
            }
        }
    }
    pub fn forward21_delivered(&mut self, digest: &str, recipient: crate::room::Membership) {
        let complete = self
            .pending21
            .as_mut()
            .filter(|p| p.0 == digest)
            .and_then(|p| p.1.as_mut())
            .is_some_and(|targets| targets.remove(&recipient) && targets.is_empty());
        if complete {
            self.forward21_complete(digest);
        }
    }
    pub fn forward21_complete(&mut self, digest: &str) {
        if self.pending21.as_ref().is_some_and(|p| p.0 == digest) {
            self.forwarded21 = self.trap.as_ref().map(|t| t.digest.clone());
            self.pending21 = None;
        }
    }
}
impl Player {
    pub fn blast_committed(&mut self) {
        self.pending21 = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    fn report(subtype: u8, tick: u32) -> Report {
        Report {
            subtype,
            slot: 0,
            tick,
            digest: format!("{subtype}:{tick}"),
            forward: None,
        }
    }
    #[test]
    fn malformed_blob_and_unmeasured21_variant_fail_before_admission() {
        fn body(subtype: u8, size: usize) -> Vec<u8> {
            let mut out = Vec::new();
            for (v, w) in [
                (subtype as u64, Width::U8),
                (0, Width::U8),
                (120, Width::U16),
                (159, Width::U16),
                (1, Width::U32),
                (0, Width::U32),
            ] {
                out.extend(write_uint(v, w).unwrap());
            }
            let mut blob = vec![24];
            blob.extend_from_slice(&((size - 5) as u32).to_le_bytes());
            blob.resize(size, 0);
            out.extend(write_uint(size as u64, Width::U32).unwrap());
            for chunk in blob.chunks(4) {
                // The death remainder is2+1, unlike a single3-byte integer.
                if chunk.len() == 3 {
                    out.extend(
                        write_uint(u16::from_le_bytes([chunk[0], chunk[1]]) as u64, Width::U16)
                            .unwrap(),
                    );
                    out.extend(write_uint(chunk[2] as u64, Width::U8).unwrap());
                } else {
                    let mut value = [0; 8];
                    value[..chunk.len()].copy_from_slice(chunk);
                    out.extend(
                        write_uint(
                            u64::from_le_bytes(value),
                            if chunk.len() == 4 {
                                Width::U32
                            } else {
                                Width::U16
                            },
                        )
                        .unwrap(),
                    );
                }
            }
            out
        }
        for (subtype, size, forward_size) in [(17, 14, 10), (19, 11, 21)] {
            let valid = body(subtype, size);
            let parsed = Report::parse(&valid).unwrap();
            assert_eq!(parsed.forward.unwrap().len(), forward_size);
            assert!(Report::parse(&valid[..valid.len() - 1]).is_err());
            let mut wrong = valid.clone();
            wrong[18..22].copy_from_slice(&write_uint(0, Width::U32).unwrap());
            assert!(Report::parse(&wrong).is_err());
            let mut trailing = valid;
            trailing.push(0);
            assert!(Report::parse(&trailing).is_err());
        }
        let mut alive = body(21, 11);
        alive.truncate(14);
        assert!(Report::parse(&alive).is_ok());
        alive.push(0);
        assert!(Report::parse(&alive).is_err());
        assert!(Report::parse(&[0; 13]).is_err());
        assert!(Report::parse(&[0; 4097]).is_err());
    }
    #[test]
    fn invalid_death_cannot_poison_cursor_and_death_cannot_be_hidden_by_new_trap() {
        let now = Instant::now();
        let mut p = Player::default();
        assert_eq!(
            p.apply(0, &report(19, 0x7fff_ffff), now, None).reason,
            "death_no_current_trap"
        );
        assert!(p.apply(0, &report(17, 10), now, None).accepted);
        assert!(p.apply(0, &report(19, 11), now, None).accepted);
        assert!(p.apply(0, &report(17, 12), now, None).accepted);
        assert_eq!(
            p.apply(0, &report(19, 13), now, None).reason,
            "death_already_accepted"
        );
        assert!(!p.current_survivor(now + Duration::from_secs(1), None));
    }
    #[test]
    fn reset_keeps_connection_cursor_and_wrap_is_serial_arithmetic() {
        let now = Instant::now();
        let mut p = Player::default();
        assert!(p.apply(0, &report(17, u32::MAX - 1), now, None).accepted);
        p.begin();
        assert_eq!(
            p.apply(0, &report(17, u32::MAX - 1), now, None).reason,
            "duplicate_report_body"
        );
        assert!(p.apply(0, &report(17, 1), now, None).accepted);
        assert_eq!(
            p.apply(0, &report(19, 0x8000_0001), now, None).reason,
            "report_tick_not_forward"
        );
        assert!(p.apply(0, &report(19, 2), now, None).accepted);
        assert!(!p.apply(1, &report(21, 3), now, None).accepted);
    }
    #[test]
    fn result_clear_does_not_unblock_pickup_and_once_marker_needs_delivery() {
        let now = Instant::now();
        let later = now + Duration::from_secs(1);
        let mut p = Player::default();
        p.apply(0, &report(17, 1), now, None);
        assert!(p.apply(0, &report(21, 2), later, Some(now)).forward);
        assert!(p.blocked_pickup());
        assert!(p.forward21_valid("21:2", later, Some(now)));
        assert!(!p.forward21_valid("21:2", later, Some(later)));
        assert!(!p.apply(0, &report(21, 3), later, None).forward);
        p.forward21_complete("21:2");
        assert!(!p.apply(0, &report(21, 4), later, None).forward);
        p.apply(0, &report(17, 5), later, None);
        assert!(
            p.apply(0, &report(21, 6), later + Duration::from_secs(1), None)
                .forward
        );
        p.apply(0, &report(19, 7), later, None);
        assert!(!p.forward21_valid("21:6", later + Duration::from_secs(2), None));
    }
    #[test]
    fn clear21_waits_for_all_three_memberships_and_old_completion_cannot_finish_new_clear() {
        let now = Instant::now();
        let later = now + Duration::from_secs(1);
        let targets = [2, 3, 4].map(|id| crate::room::Membership {
            identity: id,
            connection: id as u64 * 10,
            visit: 1,
        });
        let mut p = Player::default();
        p.apply(0, &report(17, 1), now, None);
        assert!(p.apply(0, &report(21, 2), later, None).forward);
        assert!(p.forward21_targets("21:2", &[]).is_err());
        assert!(p.forward21_targets("21:2", &[targets[0]; 3]).is_err());
        p.forward21_targets("21:2", &targets).unwrap();
        assert!(p.forward21_targets("21:2", &targets).is_err());
        for target in &targets[..2] {
            p.forward21_delivered("21:2", *target);
            p.forward21_delivered("21:2", *target); // duplicate cannot consume another peer
            assert!(p.forward21_valid("21:2", later, None));
        }
        let reused = crate::room::Membership {
            connection: 99,
            visit: 2,
            ..targets[2]
        };
        p.forward21_delivered("21:2", reused);
        assert!(p.forward21_valid("21:2", later, None));
        assert!(!p.apply(0, &report(21, 3), later, None).forward);
        p.forward21_delivered("21:2", targets[2]);
        assert!(!p.forward21_valid("21:2", later, None));
        assert!(!p.apply(0, &report(21, 4), later, None).forward);
        p.apply(0, &report(17, 5), later, None);
        let fresh = later + Duration::from_secs(1);
        assert!(p.apply(0, &report(21, 6), fresh, None).forward);
        p.forward21_targets("21:6", &targets).unwrap();
        p.forward21_delivered("21:2", targets[2]);
        assert!(p.forward21_valid("21:6", fresh, None));
        p.blast_committed();
        assert!(!p.forward21_valid("21:6", fresh, None));
        p.forward21_delivered("21:6", targets[0]);
        assert!(p.apply(0, &report(21, 7), fresh, None).forward);
        p.forward21_targets("21:7", &targets).unwrap();
        p.begin();
        for target in targets {
            p.forward21_delivered("21:7", target);
        }
        assert!(!p.forward21_valid("21:7", fresh, None));
    }
}

#[cfg(test)]
mod variable_recipient_tests {
    use super::*;
    use crate::room::Membership;
    #[test]
    fn three_player_delivery_waits_for_both_distinct_current_recipients() {
        let mut player = Player::default();
        let a = Membership {
            identity: 42,
            connection: 11,
            visit: 1,
        };
        let b = Membership {
            identity: 900,
            connection: 22,
            visit: 2,
        };
        assert!(player.forward_report_targets("duplicate", &[a, a]).is_err());
        player.forward_report_targets("death", &[a, b]).unwrap();
        assert!(player.delivery_pending());
        player.report_delivered("death", a);
        player.report_delivered("death", a);
        assert!(player.delivery_pending());
        player.report_delivered("death", Membership { visit: 3, ..b });
        assert!(player.delivery_pending());
        player.report_delivered("death", b);
        assert!(!player.delivery_pending());
    }
}
