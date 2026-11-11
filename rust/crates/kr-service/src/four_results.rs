//! CP202/210 measured fixed four/eight rank layout; outcome/zero economy/settle are design.
use crate::room::{Phase, Room};
use kr_wire::{Width, write_uint};
use std::{
    collections::BTreeSet,
    io,
    time::{Duration, Instant},
};

pub(crate) fn winner(room: &Room, now: Instant) -> Option<u8> {
    if !matches!(room.capacity(), 4 | 8)
        || room.members.len() != room.capacity()
        || !matches!(room.phase, Phase::Armed { .. })
        || room.bombs.pending()
        || room.members.values().any(|m| m.life.delivery_pending())
        || !room.fixed_teams_match()
    {
        return None;
    }
    let alive: BTreeSet<_> = room
        .members
        .values()
        .filter(|m| m.life.death.is_none())
        .map(|m| m.team)
        .collect();
    if alive.len() != 1 {
        return None;
    } // Tie/all-dead deliberately unsupported.
    let latest_death = room.members.values().filter_map(|m| m.life.death).max()?;
    let latest = room
        .last_blast
        .map_or(latest_death, |b| b.max(latest_death));
    if now.checked_duration_since(latest)? < Duration::from_secs(8) {
        return None;
    }
    if !room.members.values().any(|m| {
        m.life.death.is_none()
            && m.life
                .survivor
                .is_some_and(|at| at > latest && m.life.current_survivor(at, room.last_blast))
    }) {
        return None;
    }
    alive.first().copied()
}

/// Header16 + count×131 active rows + (16-count) empty slots: inner552/1072.
/// Constructor is an input producer, never a native compatibility oracle.
pub(crate) fn rank_body(teams: &[u8], winner: u8) -> io::Result<Vec<u8>> {
    let count = teams.len();
    if !matches!(count, 4 | 8)
        || winner > 1
        || !(teams
            .iter()
            .copied()
            .eq((0..count).map(|slot| (slot % 2) as u8))
            || teams == [0, 1, 1, 1, 1, 1, 1, 1])
    {
        return Err(io::Error::other("unsupported fixed-four winning team"));
    }
    let mut inner = vec![0; 16];
    inner[9] = 255;
    for team in teams {
        let mut row = vec![0; 131];
        row[0] = 1;
        row[5] = if *team == winner { 1 } else { 2 };
        inner.extend(row);
    }
    inner.resize(inner.len() + 16 - count, 0);
    let mut stream = vec![24];
    stream.extend_from_slice(&((inner.len() + 1) as u32).to_le_bytes());
    stream.push(0);
    stream.extend(inner);
    let mut body = write_uint(stream.len() as u64, Width::U32).map_err(io::Error::other)?;
    let mut at = 0;
    while at < stream.len() {
        let (n, w) = if stream.len() - at >= 4 {
            (4, Width::U32)
        } else if stream.len() - at >= 2 {
            (2, Width::U16)
        } else {
            (1, Width::U8)
        };
        let mut value = [0; 8];
        value[..n].copy_from_slice(&stream[at..at + n]);
        body.extend(write_uint(u64::from_le_bytes(value), w).map_err(io::Error::other)?);
        at += n;
    }
    Ok(body)
}

pub(crate) fn return_body(selector: u8) -> io::Result<Vec<u8>> {
    if selector > 1 {
        return Err(io::Error::other("unsupported room selector"));
    }
    let mut body = Vec::new();
    for value in [0, selector, 0] {
        body.extend(write_uint(value as u64, Width::U8).map_err(io::Error::other)?);
    }
    Ok(body)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{life::Report, room::Membership};
    fn armed() -> (Room, [Membership; 4]) {
        let mut room = Room::with_capacity(4).unwrap();
        let members = std::array::from_fn(|i| {
            let m = room.enter(i as u32 + 1, i as u64 + 10, i == 0).unwrap();
            room.setup(m).unwrap();
            if i > 0 {
                room.ready(m).unwrap();
            }
            m
        });
        let round = room.begin(members[0]).unwrap();
        for m in members {
            room.ack(m, round).unwrap();
        }
        (room, members)
    }
    fn report(slot: u8, kind: u8, tick: u32) -> Report {
        Report {
            subtype: kind,
            slot,
            tick,
            digest: format!("{slot}:{kind}:{tick}"),
            forward: None,
        }
    }
    fn kill(room: &mut Room, slot: u8, at: Instant) {
        let p = &mut room.members.get_mut(&(slot as u32 + 1)).unwrap().life;
        assert!(p.apply(slot, &report(slot, 17, 100), at, None).accepted);
        assert!(p.apply(slot, &report(slot, 19, 101), at, None).accepted);
    }
    #[test]
    fn wire_hashes_match_independent_native_accepted_cp202_fixtures() {
        // CP202 sealed175 bodies, source paths/digests retained in source-first.
        // This is wire regression, never substitutes for Rust native acceptance.
        for (team, expected) in [
            (
                0,
                "e595541e876c726aebf4071a648d2fe8a07028a797971a93249f3d149cb31583",
            ),
            (
                1,
                "794354ab76dda5547052ba3b31a3418a2055a1d664a05883a852d75633d00d66",
            ),
        ] {
            let body = rank_body(&[0, 1, 0, 1], team).unwrap();
            assert_eq!(body.len(), 562);
            assert_eq!(crate::sha256_hex(&body), expected);
        }
        assert!(rank_body(&[0, 1, 0, 1], 2).is_err());
        assert!(return_body(2).is_err());
    }
    #[test]
    fn team_result_waits_for_settle_witness_all_life_writes_and_post_blast_freshness() {
        let (mut room, keys) = armed();
        let at = Instant::now();
        assert_eq!(winner(&room, at + Duration::from_secs(30)), None);
        kill(&mut room, 0, at);
        kill(&mut room, 2, at);
        assert_eq!(winner(&room, at + Duration::from_secs(30)), None); // no witness
        let b = &mut room.members.get_mut(&2).unwrap().life;
        b.apply(1, &report(1, 21, 1), at + Duration::from_secs(1), None);
        assert_eq!(winner(&room, at + Duration::from_millis(7999)), None);
        assert_eq!(winner(&room, at + Duration::from_secs(8)), Some(1));
        let targets = [keys[1], keys[2], keys[3]];
        room.members
            .get_mut(&1)
            .unwrap()
            .life
            .forward_report_targets("deathA", &targets)
            .unwrap();
        for target in &targets[..2] {
            room.members
                .get_mut(&1)
                .unwrap()
                .life
                .report_delivered("deathA", *target);
            assert_eq!(winner(&room, at + Duration::from_secs(8)), None);
        }
        room.members
            .get_mut(&1)
            .unwrap()
            .life
            .report_delivered("deathA", targets[1]);
        assert_eq!(winner(&room, at + Duration::from_secs(8)), None); // duplicate not third
        room.members
            .get_mut(&1)
            .unwrap()
            .life
            .report_delivered("deathA", targets[2]);
        assert_eq!(winner(&room, at + Duration::from_secs(8)), Some(1));
        room.last_blast = Some(at + Duration::from_secs(7));
        assert_eq!(winner(&room, at + Duration::from_secs(30)), None); // old witness
        room.members.get_mut(&2).unwrap().life.apply(
            1,
            &report(1, 21, 2),
            at + Duration::from_secs(8),
            room.last_blast,
        );
        assert_eq!(winner(&room, at + Duration::from_millis(14999)), None);
        assert_eq!(winner(&room, at + Duration::from_secs(15)), Some(1));
        kill(&mut room, 1, at + Duration::from_secs(16));
        kill(&mut room, 3, at + Duration::from_secs(16));
        assert_eq!(winner(&room, at + Duration::from_secs(30)), None); // unsupported all-dead
    }
}

#[cfg(test)]
mod fixed_team_tests {
    use super::*;
    use crate::room::{EightTeamLayout, Room};
    #[test]
    fn fixed_free_one_seven_uses_cp213_native_accepted_rank_vectors() {
        let teams = [0, 1, 1, 1, 1, 1, 1, 1];
        for (winner, expected) in [
            (
                0,
                "238487c646446d32a2b61a5a4e0800e2f164ad7e0ce3c3cc11e19a507281465b",
            ),
            (
                1,
                "d91afbcfe3d01f6b97448e116c196001d39e4a0e4fd2ab1ca6fce97377a68205",
            ),
        ] {
            // Historical accepted frame body hashes, not encoder-generated goldens.
            assert_eq!(
                crate::sha256_hex(&rank_body(&teams, winner).unwrap()),
                expected
            );
        }
        assert!(rank_body(&[0; 8], 0).is_err());
        assert!(rank_body(&[0, 1, 1, 1], 0).is_err());
    }
    #[test]
    fn one_seven_requires_all_seven_deaths_and_rejects_roster_drift() {
        let mut room = Room::with_capacity(8).unwrap();
        room.set_eight_layout(EightTeamLayout::OneSeven).unwrap();
        let keys: [_; 8] = std::array::from_fn(|i| {
            let k = room.enter(i as u32 + 1, i as u64 + 20, i == 0).unwrap();
            room.setup(k).unwrap();
            if i > 0 {
                room.ready(k).unwrap();
            }
            k
        });
        assert!(!room.balanced_teams());
        assert!(room.fixed_teams_match());
        assert!(room.set_eight_layout(EightTeamLayout::Balanced).is_err());
        let round = room.begin(keys[0]).unwrap(); // Free permits unbalanced teams.
        for k in keys {
            room.ack(k, round).unwrap();
        }
        let at = Instant::now();
        for slot in 1..8 {
            let player = &mut room.members.get_mut(&(slot as u32 + 1)).unwrap().life;
            for (kind, tick) in [(17, 1), (19, 2)] {
                let report = crate::life::Report {
                    subtype: kind,
                    slot,
                    tick,
                    digest: format!("{slot}:{kind}"),
                    forward: None,
                };
                assert!(player.apply(slot, &report, at, None).accepted);
            }
            assert_eq!(winner(&room, at + Duration::from_secs(20)), None);
        }
        let witness = crate::life::Report {
            subtype: 21,
            slot: 0,
            tick: 1,
            digest: "current-A".into(),
            forward: None,
        };
        room.members.get_mut(&1).unwrap().life.apply(
            0,
            &witness,
            at + Duration::from_secs(1),
            None,
        );
        assert_eq!(winner(&room, at + Duration::from_secs(8)), Some(0));
        // A roster change must never be accepted as the same configured team family.
        room.members.get_mut(&8).unwrap().team = 0;
        assert_eq!(winner(&room, at + Duration::from_secs(20)), None);
        assert!(
            Room::with_capacity(4)
                .unwrap()
                .set_eight_layout(EightTeamLayout::OneSeven)
                .is_err()
        );
    }
}
