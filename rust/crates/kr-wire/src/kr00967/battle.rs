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
    player_rank_rows(players, Some(winner))
}
/// CP083 measured rank2 for each eliminated player, without a new tie enum.
/// The general roster service policy and native acceptance are separate.
pub fn rank175_all_dead_players(players: &[(u8, u8)]) -> io::Result<Vec<u8>> {
    player_rank_rows(players, None)
}
fn player_rank_rows(players: &[(u8, u8)], winner: Option<u8>) -> io::Result<Vec<u8>> {
    if !(2..=8).contains(&players.len()) || winner.is_some_and(|team| team > 31) {
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
    if teams.len() < 2 || winner.is_some_and(|team| !teams.contains(&team)) {
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
            row[5] = if Some(*team) == winner { 1 } else { 2 };
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

/// Measured C2S122 writer shape, not an authorization or detonation decision.
///
/// Camp08 native evidence: B731D0 writes widths 4/2/2/4/1/4. The first
/// property was FFFFFFFF before the observed local effect; it must not be
/// narrowed to a server bomb ID. The auxiliary fields remain opaque.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BombReport122 {
    pub property: u32,
    pub row: u16,
    pub column: u16,
    pub reason: u32,
    pub auxiliary: u8,
    pub trailing: u32,
}

pub fn bomb_report122(body: &[u8]) -> io::Result<BombReport122> {
    if body.len() != 17 {
        return Err(io::Error::other("bomb122 exact measured body size"));
    }
    Ok(BombReport122 {
        property: uint(&body[..4], Width::U32)? as u32,
        row: uint(&body[4..6], Width::U16)? as u16,
        column: uint(&body[6..8], Width::U16)? as u16,
        reason: uint(&body[8..12], Width::U32)? as u32,
        auxiliary: uint(&body[12..13], Width::U8)? as u8,
        trailing: uint(&body[13..17], Width::U32)? as u32,
    })
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
// Death drops have a u32 followed by a u8 count and eight bytes per entry.
// This bounds inflation even when the client supplies a size of its own.
const DEATH_HEADER: usize = 5;
const DEATH_ENTRY: usize = 8;
const DEATH_MAX: usize = DEATH_HEADER + u8::MAX as usize * DEATH_ENTRY;

fn death_entries_match(inner: &[u8]) -> bool {
    inner
        .get(DEATH_HEADER - 1)
        .is_some_and(|count| inner.len() == DEATH_HEADER + usize::from(*count) * DEATH_ENTRY)
}

fn death_peer_tail(raw: &[u8], encoded: &[u8]) -> io::Result<Vec<u8>> {
    use flate2::{Decompress, FlushDecompress, Status};

    let error = || io::Error::other("report127 death blob layout");
    let mut inflated = [0; DEATH_MAX + 1];
    let inner = match raw[5] {
        0 => &raw[6..],
        1 if raw.len() >= 10 => {
            let size = u32::from_le_bytes(raw[6..10].try_into().expect("bounded")) as usize;
            if !(DEATH_HEADER..=DEATH_MAX).contains(&size)
                || !(size - DEATH_HEADER).is_multiple_of(DEATH_ENTRY)
            {
                return Err(error());
            }
            let compressed = &raw[10..];
            let mut decoder = Decompress::new(true);
            let status = decoder
                .decompress(compressed, &mut inflated[..=size], FlushDecompress::Finish)
                .map_err(|_| error())?;
            if status != Status::StreamEnd
                || decoder.total_in() != compressed.len() as u64
                || decoder.total_out() != size as u64
            {
                return Err(error());
            }
            &inflated[..size]
        }
        _ => return Err(error()),
    };
    if !death_entries_match(inner) {
        return Err(error());
    }
    if inner[4] == 0 {
        return Ok(encoded.to_vec());
    }

    // The client writes index/kind/row/column in 127, but the 170 consumer
    // (1403E90) reads row/column/index/kind. Preserve the header and values.
    let mut stream = Vec::with_capacity(inner.len() + 6);
    stream.push(24);
    stream.extend_from_slice(&((inner.len() + 1) as u32).to_le_bytes());
    stream.push(0);
    stream.extend_from_slice(&inner[..DEATH_HEADER]);
    for entry in inner[DEATH_HEADER..].as_chunks::<DEATH_ENTRY>().0 {
        stream.extend_from_slice(&entry[6..]);
        stream.extend_from_slice(&entry[..6]);
    }
    let mut tail = super::put(stream.len() as u64, Width::U32)?;
    tail.extend(super::encode_bytes(&stream)?);
    Ok(tail)
}

pub fn report127(body: &[u8]) -> io::Result<LifeReport127> {
    if !(14..=4096).contains(&body.len()) {
        return Err(io::Error::other("report127 bounded prefix"));
    }
    let subtype = uint(&body[..1], Width::U8)? as u8;
    let forward = if matches!(subtype, 17 | 19) {
        let size = body.len().saturating_sub(18);
        if size < 6
            || (subtype == 17 && size != 14)
            || uint(&body[14..18], Width::U32)? != size as u64
        {
            return Err(io::Error::other("report127 supported blob size"));
        }
        let raw = super::decode_bytes(&body[18..])?;
        if raw[0] != 24
            || u32::from_le_bytes(raw[1..5].try_into().expect("bounded")) as usize != size - 5
            || (subtype == 17 && raw[5] != 0)
        {
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
            out.extend(death_peer_tail(&raw, &body[14..])?);
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
