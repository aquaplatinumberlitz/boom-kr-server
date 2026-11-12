//! Measured body serializers; result/death/pickup authority stays in service.
use super::uint;
use crate::{Width, write_uint};
use std::io;
pub fn rank175(teams: &[u8], winner: u8) -> io::Result<Vec<u8>> {
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
    rank175_players(
        &teams
            .iter()
            .enumerate()
            .map(|(slot, team)| (slot as u8, *team))
            .collect::<Vec<_>>(),
        winner,
    )
}

/// Slot-indexed rank rows for an explicit current roster. Service decides the
/// outcome. Sparse/variable native acceptance is independent of serialization.
pub fn rank175_players(players: &[(u8, u8)], winner: u8) -> io::Result<Vec<u8>> {
    if !(2..=8).contains(&players.len()) || winner > 31 {
        return Err(io::Error::other("invalid player rank roster"));
    }
    let mut slots = std::collections::BTreeMap::new();
    let mut teams = std::collections::BTreeSet::new();
    for &(slot, team) in players {
        if slot >= 8 || team > 31 || slots.insert(slot, team).is_some() {
            return Err(io::Error::other("invalid or duplicate player rank slot"));
        }
        teams.insert(team);
    }
    if teams.len() < 2 || !teams.contains(&winner) {
        return Err(io::Error::other(
            "rank requires two teams and a current winner",
        ));
    }
    let mut inner = vec![0; 16];
    inner[9] = 255;
    for slot in 0..16 {
        if let Some(team) = slots.get(&slot) {
            let mut row = vec![0; 131];
            row[0] = 1;
            row[5] = if *team == winner { 1 } else { 2 };
            inner.extend(row);
        } else {
            inner.push(0);
        }
    }
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
pub fn return180(selector: u8) -> io::Result<Vec<u8>> {
    if selector > 1 {
        return Err(io::Error::other("unsupported room selector"));
    }
    let mut body = Vec::new();
    for value in [0, selector, 0] {
        body.extend(write_uint(value as u64, Width::U8).map_err(io::Error::other)?);
    }
    Ok(body)
}
pub fn spawn159(
    owner: u8,
    serial: u32,
    row: u8,
    column: u8,
    power: u8,
    id: u16,
) -> io::Result<Vec<u8>> {
    let fields = [
        (0, Width::U8),
        (u64::from(owner), Width::U8),
        (0, Width::U16),
        (u64::from(serial), Width::U32),
        (0, Width::U32),
        (0, Width::U32),
        (u64::from(row), Width::U8),
        (u64::from(column), Width::U8),
        (u64::from(power), Width::U8),
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
pub fn explode158(id: u16) -> io::Result<Vec<u8>> {
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
pub fn pickup165(
    context: u8,
    slot: u8,
    row: u8,
    column: u8,
    index: u32,
    kind: u16,
) -> io::Result<Vec<u8>> {
    let mut result = Vec::with_capacity(12);
    for (v, w) in [
        (1, Width::U8),
        (u64::from(context), Width::U8),
        (u64::from(slot), Width::U8),
        (u64::from(row), Width::U8),
        (u64::from(column), Width::U8),
        (u64::from(index), Width::U32),
        (u64::from(kind), Width::U16),
        (0, Width::U8),
    ] {
        result.extend(write_uint(v, w).map_err(io::Error::other)?);
    }
    Ok(result)
}
pub struct LifeReport127 {
    pub subtype: u8,
    pub slot: u8,
    pub tick: u32,
    pub forward: Option<Vec<u8>>,
}
/// Candidate disconnect policy using the measured death19/empty serialized
/// branch. Zero coordinates are explicit forfeit policy, not claimed as
/// authoritative position or a fabricated client report/ACK.
pub fn departure170(slot: u8) -> io::Result<Vec<u8>> {
    if slot >= 8 {
        return Err(io::Error::other("departure slot outside measured bound"));
    }
    let mut out = Vec::new();
    for (v, w) in [
        (19, Width::U8),
        (u64::from(slot), Width::U8),
        (0, Width::U16),
        (0, Width::U16),
    ] {
        out.extend(write_uint(v, w).map_err(io::Error::other)?);
    }
    let mut raw = vec![24, 6, 0, 0, 0, 0];
    raw.extend(u32::MAX.to_le_bytes());
    raw.push(0);
    out.extend(write_uint(raw.len() as u64, Width::U32).map_err(io::Error::other)?);
    out.extend(super::encode_bytes(&raw)?);
    Ok(out)
}
pub fn report127(body: &[u8]) -> io::Result<LifeReport127> {
    if !(14..=4096).contains(&body.len()) {
        return Err(io::Error::other("report127 bounded prefix"));
    }
    let subtype = uint(&body[..1], Width::U8)? as u8;
    let forward = if matches!(subtype, 17 | 19) {
        let size = if subtype == 17 { 14usize } else { 11 };
        if body.len() != 18 + size || uint(&body[14..18], Width::U32)? != size as u64 {
            return Err(io::Error::other("report127 supported blob size"));
        }
        let mut raw = Vec::new();
        let mut offset = 18;
        while offset < body.len() {
            let (width, n) = match body.len() - offset {
                4..=usize::MAX => (Width::U32, 4),
                2..=3 => (Width::U16, 2),
                _ => (Width::U8, 1),
            };
            raw.extend_from_slice(&uint(&body[offset..offset + n], width)?.to_le_bytes()[..n]);
            offset += n;
        }
        let mut envelope = vec![24];
        envelope.extend_from_slice(&((size - 5) as u32).to_le_bytes());
        envelope.push(0);
        if raw[..6] != envelope {
            return Err(io::Error::other("report127 supported raw envelope"));
        }
        let mut out = body[..6].to_vec();
        if subtype == 17 {
            out.extend(
                write_uint(
                    u32::from_le_bytes(raw[10..14].try_into().expect("trap size")) as u64,
                    Width::U32,
                )
                .map_err(io::Error::other)?,
            );
        } else {
            out.extend_from_slice(&body[14..]);
        }
        Some(out)
    } else if subtype == 21 {
        if body.len() != 14 {
            return Err(io::Error::other("report21 measured prefix size"));
        }
        Some(body[..6].to_vec())
    } else {
        None
    };
    Ok(LifeReport127 {
        subtype,
        slot: uint(&body[1..2], Width::U8)? as u8,
        tick: uint(&body[6..10], Width::U32)? as u32,
        forward,
    })
}
