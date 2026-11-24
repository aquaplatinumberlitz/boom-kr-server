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
/// C2S 122 reason written by the RemoteBomb detonation path `B4C0C0 -> B7A8A0 -> B731D0`.
pub const REMOTE_DETONATION_REASON: u32 = 2;
/// `delay = max(0, 200 - 2 * latency)` on the same path, so it never exceeds 200.
pub const REMOTE_DETONATION_MAX_DELAY: u32 = 200;

/// Static o8 C2S 122 RemoteBomb detonation (`[u32 bomb_id, u16 row, u16 col, u32 reason=2,
/// u8 slot, u32 delay]`). Codec bound only; the server reply is unknown and no
/// service handler consumes it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RemoteDetonation122 {
    pub bomb_id: u32,
    pub row: u16,
    pub column: u16,
    pub slot: u8,
    pub delay: u32,
}

pub fn remote_detonation122(body: &[u8]) -> io::Result<RemoteDetonation122> {
    let report = bomb_report122(body)?;
    if report.reason != REMOTE_DETONATION_REASON || report.trailing > REMOTE_DETONATION_MAX_DELAY {
        return Err(io::Error::other("unsupported remote detonation122 shape"));
    }
    Ok(RemoteDetonation122 {
        bomb_id: report.property,
        row: report.row,
        column: report.column,
        slot: report.auxiliary,
        delay: report.trailing,
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ItemUseKind {
    Dart = 143,
    Needle = 145,
    Scanner = 146,
    Spring = 148,
    /// Group 29 Ctrl case `11910F8`; shares the 127/sub0 writer (o8 `shield_g29`).
    Shield = 151,
    /// Group 30 Ctrl case `119114A`; sub0 is its only report (o8 `sensor_g30`).
    Sensor = 152,
    Bond = 224,
    Trap = 225,
    Pin = 226,
    Banana1 = 227,
    Banana2 = 228,
    Banana3 = 229,
    Banana8 = 230,
}

pub const DART_KIND: u16 = ItemUseKind::Dart as u16;
pub const SCANNER_KIND: u16 = ItemUseKind::Scanner as u16;
pub const SPRING_KIND: u16 = ItemUseKind::Spring as u16;
pub const BOND_KIND: u16 = ItemUseKind::Bond as u16;
pub const TRAP_KIND: u16 = ItemUseKind::Trap as u16;
pub const PIN_KIND: u16 = ItemUseKind::Pin as u16;
pub const BANANA1_KIND: u16 = ItemUseKind::Banana1 as u16;
pub const BANANA8_KIND: u16 = ItemUseKind::Banana8 as u16;
pub const SHIELD_KIND: u16 = ItemUseKind::Shield as u16;
pub const SENSOR_KIND: u16 = ItemUseKind::Sensor as u16;
/// Group 28 RemoteBomb row. No item-use variant: its 118 flag byte offset is unpinned.
pub const REMOTE_BOMB_KIND: u16 = 150;

pub const SPRING_SUBTYPE: u8 = 2;
pub const DRILL_SUBTYPE: u8 = 3;
pub const SHIELD_SUBTYPE: u8 = 5;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ItemUse127 {
    pub slot: u8,
    pub x: u16,
    pub y: u16,
    pub tick: u32,
    /// Common prefix field, not a validated round key.
    pub auxiliary: u32,
    pub kind: ItemUseKind,
}

/// The observed 22-byte C2S 127/sub0 body (Needle CP037, Scanner p93, active/group11 p100b).
/// This report does not provide a peer response layout.
pub fn item_use127(body: &[u8]) -> io::Result<ItemUse127> {
    let error = || io::Error::other("unsupported item-use127/sub0 shape");
    if body.len() != 22
        || uint(&body[..1], Width::U8)? != 0
        || uint(&body[14..15], Width::U8)? != 1
        || uint(&body[17..18], Width::U8)? != 0
        || uint(&body[18..22], Width::U32)? != 0
    {
        return Err(error());
    }
    let kind = match uint(&body[15..17], Width::U16)? as u16 {
        DART_KIND => ItemUseKind::Dart,
        145 => ItemUseKind::Needle,
        SCANNER_KIND => ItemUseKind::Scanner,
        SPRING_KIND => ItemUseKind::Spring,
        SHIELD_KIND => ItemUseKind::Shield,
        SENSOR_KIND => ItemUseKind::Sensor,
        BOND_KIND => ItemUseKind::Bond,
        TRAP_KIND => ItemUseKind::Trap,
        PIN_KIND => ItemUseKind::Pin,
        BANANA1_KIND => ItemUseKind::Banana1,
        228 => ItemUseKind::Banana2,
        229 => ItemUseKind::Banana3,
        BANANA8_KIND => ItemUseKind::Banana8,
        _ => return Err(error()),
    };
    Ok(ItemUse127 {
        slot: uint(&body[1..2], Width::U8)? as u8,
        x: uint(&body[2..4], Width::U16)? as u16,
        y: uint(&body[4..6], Width::U16)? as u16,
        tick: uint(&body[6..10], Width::U32)? as u32,
        auxiliary: uint(&body[10..14], Width::U32)? as u32,
        kind,
    })
}

/// Measured C2S 138 / S2C 172 9-byte Dart projectile report (E2EB30 / E64570).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Dart138 {
    pub world_index: u8,
    pub x_px: u16,
    pub y_px: u16,
    pub dir: u8,
    pub distance_px: u16,
}

pub fn dart138(body: &[u8]) -> io::Result<Dart138> {
    let error = || io::Error::other("unsupported dart138 shape");
    if body.len() != 9 || uint(&body[1..2], Width::U8)? != 0 {
        return Err(error());
    }
    Ok(Dart138 {
        world_index: uint(&body[..1], Width::U8)? as u8,
        x_px: uint(&body[2..4], Width::U16)? as u16,
        y_px: uint(&body[4..6], Width::U16)? as u16,
        dir: uint(&body[6..7], Width::U8)? as u8,
        distance_px: uint(&body[7..9], Width::U16)? as u16,
    })
}

pub fn dart172(dart: Dart138) -> io::Result<Vec<u8>> {
    let mut out = Vec::with_capacity(9);
    for (v, w) in [
        (u64::from(dart.world_index), Width::U8),
        (0, Width::U8),
        (u64::from(dart.x_px), Width::U16),
        (u64::from(dart.y_px), Width::U16),
        (u64::from(dart.dir), Width::U8),
        (u64::from(dart.distance_px), Width::U16),
    ] {
        out.extend(super::put(v, w)?);
    }
    Ok(out)
}

/// Measured C2S 139 4-byte Group 11 cell placement report (E781B0 -> E646E0 S2C 173).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PlaceItem139 {
    pub slot: u8,
    pub kind_flags: u8,
    pub row: u8,
    pub col: u8,
}

impl PlaceItem139 {
    pub fn selector(self) -> u8 {
        self.kind_flags & 0x0f
    }
}

pub fn place_item139(body: &[u8]) -> io::Result<PlaceItem139> {
    if body.len() != 4 {
        return Err(io::Error::other("unsupported place139 shape"));
    }
    Ok(PlaceItem139 {
        slot: uint(&body[..1], Width::U8)? as u8,
        kind_flags: uint(&body[1..2], Width::U8)? as u8,
        row: uint(&body[2..3], Width::U8)? as u8,
        col: uint(&body[3..4], Width::U8)? as u8,
    })
}

/// S2C 173 strips the leading slot byte from C2S 139.
pub fn place_item173(kind_flags: u8, row: u8, col: u8) -> io::Result<Vec<u8>> {
    let mut out = Vec::with_capacity(3);
    for v in [kind_flags, row, col] {
        out.extend(super::put(u64::from(v), Width::U8)?);
    }
    Ok(out)
}

/// Measured C2S 126 4-byte Group 11 trigger report (E2E610 -> E63F10 S2C 168).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TriggerItem126 {
    pub world_index: u8,
    pub slot: u8,
    pub row: u8,
    pub col: u8,
}

pub fn trigger_item126(body: &[u8]) -> io::Result<TriggerItem126> {
    if body.len() != 4 {
        return Err(io::Error::other("unsupported trigger126 shape"));
    }
    Ok(TriggerItem126 {
        world_index: uint(&body[..1], Width::U8)? as u8,
        slot: uint(&body[1..2], Width::U8)? as u8,
        row: uint(&body[2..3], Width::U8)? as u8,
        col: uint(&body[3..4], Width::U8)? as u8,
    })
}

pub fn trigger_item168(
    result: u8,
    world_index: u8,
    slot: u8,
    row: u8,
    col: u8,
) -> io::Result<Vec<u8>> {
    let mut out = Vec::with_capacity(5);
    for v in [result, world_index, slot, row, col] {
        out.extend(super::put(u64::from(v), Width::U8)?);
    }
    Ok(out)
}

/// Measured 27-byte C2S 127/sub2 Spring jump report (B4B5A0 -> E86DD0 S2C 170/sub2).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Spring127 {
    pub slot: u8,
    pub x: u16,
    pub y: u16,
    pub tick: u32,
    pub extra: u32,
    pub row: u8,
    pub col: u8,
    pub packed_step: u8,
}

pub fn spring127(body: &[u8]) -> io::Result<Spring127> {
    let error = || io::Error::other("unsupported spring127/sub2 shape");
    if body.len() != 27
        || uint(&body[..1], Width::U8)? != u64::from(SPRING_SUBTYPE)
        || uint(&body[14..18], Width::U32)? != 9
    {
        return Err(error());
    }
    let raw = super::decode_bytes(&body[18..27])?;
    if raw.len() != 9 || raw[..6] != [24, 4, 0, 0, 0, 0] {
        return Err(error());
    }
    Ok(Spring127 {
        slot: uint(&body[1..2], Width::U8)? as u8,
        x: uint(&body[2..4], Width::U16)? as u16,
        y: uint(&body[4..6], Width::U16)? as u16,
        tick: uint(&body[6..10], Width::U32)? as u32,
        extra: uint(&body[10..14], Width::U32)? as u32,
        row: raw[6],
        col: raw[7],
        packed_step: raw[8],
    })
}

/// S2C 170/sub2 unpacks the serialized blob into 3 plain inline bytes after the 6-byte header.
pub fn spring170(
    slot: u8,
    x: u16,
    y: u16,
    row: u8,
    col: u8,
    packed_step: u8,
) -> io::Result<Vec<u8>> {
    let mut out = Vec::with_capacity(9);
    for (v, w) in [
        (u64::from(SPRING_SUBTYPE), Width::U8),
        (u64::from(slot), Width::U8),
        (u64::from(x), Width::U16),
        (u64::from(y), Width::U16),
        (u64::from(row), Width::U8),
        (u64::from(col), Width::U8),
        (u64::from(packed_step), Width::U8),
    ] {
        out.extend(super::put(v, w)?);
    }
    Ok(out)
}

/// Static 31-byte C2S 127/sub3 Drill report (B4BB40 -> E86520 S2C 170/sub3).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Drill127 {
    pub slot: u8,
    pub x: u16,
    pub y: u16,
    pub tick: u32,
    pub extra: u32,
    pub box_id: u32,
    pub row: u8,
    pub col: u8,
    pub dir: u8,
}

pub fn drill127(body: &[u8]) -> io::Result<Drill127> {
    let error = || io::Error::other("unsupported drill127/sub3 shape");
    if body.len() != 31
        || uint(&body[..1], Width::U8)? != u64::from(DRILL_SUBTYPE)
        || uint(&body[14..18], Width::U32)? != 13
    {
        return Err(error());
    }
    let raw = super::decode_bytes(&body[18..31])?;
    if raw.len() != 13 || raw[..6] != [24, 8, 0, 0, 0, 0] {
        return Err(error());
    }
    Ok(Drill127 {
        slot: uint(&body[1..2], Width::U8)? as u8,
        x: uint(&body[2..4], Width::U16)? as u16,
        y: uint(&body[4..6], Width::U16)? as u16,
        tick: uint(&body[6..10], Width::U32)? as u32,
        extra: uint(&body[10..14], Width::U32)? as u32,
        box_id: u32::from_le_bytes(raw[6..10].try_into().expect("bounded")),
        row: raw[10],
        col: raw[11],
        dir: raw[12],
    })
}

pub fn drill170(drill: Drill127) -> io::Result<Vec<u8>> {
    let mut raw = vec![24, 8, 0, 0, 0, 0];
    raw.extend_from_slice(&drill.box_id.to_le_bytes());
    raw.extend_from_slice(&[drill.row, drill.col, drill.dir]);
    let mut out = Vec::with_capacity(23);
    for (v, w) in [
        (u64::from(DRILL_SUBTYPE), Width::U8),
        (u64::from(drill.slot), Width::U8),
        (u64::from(drill.x), Width::U16),
        (u64::from(drill.y), Width::U16),
        (raw.len() as u64, Width::U32),
    ] {
        out.extend(super::put(v, w)?);
    }
    out.extend(super::encode_bytes(&raw)?);
    Ok(out)
}

/// Static 14-byte C2S 127/sub5 Shield report (B73DC0 -> E86670 S2C 170/sub5).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Shield127 {
    pub slot: u8,
    pub x: u16,
    pub y: u16,
    pub tick: u32,
    pub extra: u32,
}

pub fn shield127(body: &[u8]) -> io::Result<Shield127> {
    if body.len() != 14 || uint(&body[..1], Width::U8)? != u64::from(SHIELD_SUBTYPE) {
        return Err(io::Error::other("unsupported shield127/sub5 shape"));
    }
    Ok(Shield127 {
        slot: uint(&body[1..2], Width::U8)? as u8,
        x: uint(&body[2..4], Width::U16)? as u16,
        y: uint(&body[4..6], Width::U16)? as u16,
        tick: uint(&body[6..10], Width::U32)? as u32,
        extra: uint(&body[10..14], Width::U32)? as u32,
    })
}

pub fn shield170(slot: u8, x: u16, y: u16) -> io::Result<Vec<u8>> {
    let mut out = Vec::with_capacity(6);
    for (v, w) in [
        (u64::from(SHIELD_SUBTYPE), Width::U8),
        (u64::from(slot), Width::U8),
        (u64::from(x), Width::U16),
        (u64::from(y), Width::U16),
    ] {
        out.extend(super::put(v, w)?);
    }
    Ok(out)
}

pub const VEHICLE_DISMOUNT_SUBTYPE: u8 = 16;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct VehicleDismount127 {
    pub slot: u8,
    pub x: u16,
    pub y: u16,
    pub tick: u32,
    /// The trailing client field is not a validated round key.
    pub extra: u32,
}

/// Exact 14-byte C2S 127/sub16 body measured in p97b.
pub fn vehicle_dismount127(body: &[u8]) -> io::Result<VehicleDismount127> {
    if body.len() != 14 || uint(&body[..1], Width::U8)? != u64::from(VEHICLE_DISMOUNT_SUBTYPE) {
        return Err(io::Error::other(
            "unsupported vehicle dismount127/sub16 shape",
        ));
    }
    Ok(VehicleDismount127 {
        slot: uint(&body[1..2], Width::U8)? as u8,
        x: uint(&body[2..4], Width::U16)? as u16,
        y: uint(&body[4..6], Width::U16)? as u16,
        tick: uint(&body[6..10], Width::U32)? as u32,
        extra: uint(&body[10..14], Width::U32)? as u32,
    })
}

/// The peer receives only the six-byte header, without the tick and extra.
pub fn vehicle_dismount170(slot: u8, x: u16, y: u16) -> io::Result<Vec<u8>> {
    let mut out = Vec::with_capacity(6);
    for (value, width) in [
        (u64::from(VEHICLE_DISMOUNT_SUBTYPE), Width::U8),
        (u64::from(slot), Width::U8),
        (u64::from(x), Width::U16),
        (u64::from(y), Width::U16),
    ] {
        out.extend(super::put(value, width)?);
    }
    Ok(out)
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

/// Wire `S->C 159` owner byte assigned to PangLand `CGOBombSprinkler` spawns (`p108` `0xE5E8F0`).
pub const SPRINKLER_BOMB_OWNER: u8 = 25;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SpaceChangeTile228 {
    pub strings: [Vec<u8>; 4],
    pub danger_time: u32,
    pub danger_alpha: u32,
    pub danger_speed: u32,
    pub danger_color: u32,
    pub cells: Vec<(u32, u32)>,
    pub flag_a: u8,
    pub flag_b: u8,
}

/// Decodes host `C->S 228` tile-change broadcast (`p108` `0x483060` / `CaJJ_Space02`).
pub fn space_change_tile228(body: &[u8]) -> io::Result<SpaceChangeTile228> {
    let err = || io::Error::other("unsupported Space C->S 228 payload");
    let mut at = 0usize;
    let take = |at: &mut usize, n: usize| -> io::Result<&[u8]> {
        let end = at.checked_add(n).ok_or_else(err)?;
        let slice = body.get(*at..end).ok_or_else(err)?;
        *at = end;
        Ok(slice)
    };
    let mut strings = [Vec::new(), Vec::new(), Vec::new(), Vec::new()];
    for slot in &mut strings {
        let len = uint(take(&mut at, 2)?, Width::U16)? as usize;
        if len > 64 {
            return Err(err());
        }
        *slot = super::decode_bytes(take(&mut at, len)?)?;
    }
    let danger_time = uint(take(&mut at, 4)?, Width::U32)? as u32;
    let danger_alpha = uint(take(&mut at, 4)?, Width::U32)? as u32;
    let danger_speed = uint(take(&mut at, 4)?, Width::U32)? as u32;
    let danger_color = uint(take(&mut at, 4)?, Width::U32)? as u32;
    let count = uint(take(&mut at, 4)?, Width::U32)? as usize;
    if !(1..=195).contains(&count) {
        return Err(err());
    }
    let mut cells = Vec::with_capacity(count);
    for _ in 0..count {
        let x = uint(take(&mut at, 4)?, Width::U32)? as u32;
        let y = uint(take(&mut at, 4)?, Width::U32)? as u32;
        if (x != u32::MAX && x >= 15)
            || (y != u32::MAX && y >= 13)
            || (x == u32::MAX && y == u32::MAX)
        {
            return Err(err());
        }
        cells.push((x, y));
    }
    let flag_a = uint(take(&mut at, 1)?, Width::U8)? as u8;
    let flag_b = uint(take(&mut at, 1)?, Width::U8)? as u8;
    if at != body.len() {
        return Err(err());
    }
    Ok(SpaceChangeTile228 {
        strings,
        danger_time,
        danger_alpha,
        danger_speed,
        danger_color,
        cells,
        flag_a,
        flag_b,
    })
}

/// Validates host `C->S 228` and mirrors its exact body as `S->C 350` (`p108` `0xE6A3C0`).
pub fn space_change_tile350(body: &[u8]) -> io::Result<Vec<u8>> {
    space_change_tile228(body)?;
    Ok(body.to_vec())
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SprinklerReport148 {
    pub script_source: u32,
    pub x: u16,
    pub y: u16,
    pub power: u8,
    pub damage: u32,
    pub state: u16,
    pub cells: Vec<(u8, u8)>,
}

/// Decodes host `C->S 148` sprinkler bomb request (`p108` `0xE733D0` / `PangLand`).
pub fn sprinkler148(body: &[u8]) -> io::Result<SprinklerReport148> {
    let err = || io::Error::other("unsupported PangLand C->S 148 payload");
    if body.len() < 16 {
        return Err(err());
    }
    let script_source = uint(&body[..4], Width::U32)? as u32;
    let count = uint(&body[4..5], Width::U8)? as usize;
    if !(1..=64).contains(&count) || body.len() != 16 + 2 * count {
        return Err(err());
    }
    let y = uint(&body[5..7], Width::U16)? as u16;
    let x = uint(&body[7..9], Width::U16)? as u16;
    let power = uint(&body[9..10], Width::U8)? as u8;
    if !(1..=10).contains(&power) {
        return Err(err());
    }
    let damage = uint(&body[10..14], Width::U32)? as u32;
    let state = uint(&body[14..16], Width::U16)? as u16;
    let mut cells = Vec::with_capacity(count);
    for i in 0..count {
        let base = 16 + 2 * i;
        let row = uint(&body[base..base + 1], Width::U8)? as u8;
        let col = uint(&body[base + 1..base + 2], Width::U8)? as u8;
        if row >= 13 || col >= 15 {
            return Err(err());
        }
        cells.push((row, col));
    }
    Ok(SprinklerReport148 {
        script_source,
        x,
        y,
        power,
        damage,
        state,
        cells,
    })
}

/// Serializes the 39-byte `S->C 159` sprinkler bomb spawn (`owner = 25`, `p108` `pang-159`).
#[allow(clippy::too_many_arguments)]
pub fn sprinkler_spawn159(
    serial: u32,
    x: u32,
    y: u32,
    row: u8,
    column: u8,
    power: u8,
    damage: u32,
    id: u16,
) -> io::Result<Vec<u8>> {
    if row >= 13 || column >= 15 || !(1..=10).contains(&power) {
        return Err(io::Error::other("unsupported sprinkler spawn159 bounds"));
    }
    let mut out = Vec::with_capacity(39);
    for (v, w) in [
        (0, Width::U8),
        (u64::from(SPRINKLER_BOMB_OWNER), Width::U8),
        (0, Width::U16),
        (u64::from(serial), Width::U32),
        (u64::from(x), Width::U32),
        (u64::from(y), Width::U32),
        (u64::from(row), Width::U8),
        (u64::from(column), Width::U8),
        (u64::from(power), Width::U8),
        (u64::from(damage), Width::U32),
        (0, Width::U8),
        (u64::from(id), Width::U16),
        (0, Width::U16),
        (1, Width::U8),
        (0, Width::U64),
        (0, Width::U8),
        (0, Width::U8),
    ] {
        out.extend(write_uint(v, w).map_err(io::Error::other)?);
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    // p97b Cookie07, docs/evidence/item-families/f1d-vehicles-wine.json.
    const DISMOUNT_HOLD: [u8; 14] = [
        0x4a, 0x5a, 0xa5, 0xc9, 0xa7, 0x6e, 0x96, 0xca, 0x5d, 0xf7, 0x96, 0xca, 0x53, 0x94,
    ];
    const DISMOUNT_FORWARD: [u8; 14] = [
        0x4a, 0x5a, 0xa5, 0xc9, 0xa7, 0x6e, 0x96, 0xca, 0x4f, 0x1c, 0x96, 0xca, 0x53, 0x94,
    ];
    const DISMOUNT_PEER: [u8; 6] = [0x4a, 0x5a, 0xa5, 0xc9, 0xa7, 0x6e];

    #[test]
    fn captured_vehicle_dismount_reports_and_peer_body() {
        for (body, tick) in [(DISMOUNT_HOLD, 3682), (DISMOUNT_FORWARD, 7305)] {
            let dismount = vehicle_dismount127(&body).unwrap();
            assert_eq!(
                dismount,
                VehicleDismount127 {
                    slot: 0,
                    x: 160,
                    y: 519,
                    tick,
                    extra: 1,
                }
            );
            assert_eq!(
                vehicle_dismount170(dismount.slot, dismount.x, dismount.y).unwrap(),
                DISMOUNT_PEER
            );
        }
        assert!(vehicle_dismount127(&DISMOUNT_PEER).is_err());
        assert!(vehicle_dismount127(&[DISMOUNT_FORWARD.as_slice(), &[0]].concat()).is_err());
        let mut wrong_subtype = DISMOUNT_FORWARD;
        wrong_subtype[0] = 0x5a;
        assert!(vehicle_dismount127(&wrong_subtype).is_err());
    }

    // p93 run-2 wire.jsonl, CRC-verified C2S payloads A/430 and B/376.
    const SCANNER_A: [u8; 22] = [
        0x5a, 0x5a, 0xa5, 0x69, 0xa4, 0x0e, 0x96, 0xca, 0x4c, 0x30, 0x96, 0xca, 0x53, 0x97, 0x5b,
        0xa5, 0xfb, 0x5a, 0x96, 0xca, 0x53, 0x95,
    ];
    const SCANNER_B: [u8; 22] = [
        0x5a, 0x5b, 0xa5, 0x5d, 0xa4, 0xde, 0x96, 0xca, 0x4c, 0x44, 0x96, 0xca, 0x53, 0x97, 0x5b,
        0xa5, 0xfb, 0x5a, 0x96, 0xca, 0x53, 0x95,
    ];

    #[test]
    fn captured_scanner_reports_decode_without_peer_layout() {
        for (body, slot, x, y, tick) in
            [(SCANNER_A, 0, 0, 359, 8101), (SCANNER_B, 1, 52, 439, 8145)]
        {
            assert_eq!(
                item_use127(&body).unwrap(),
                ItemUse127 {
                    slot,
                    x,
                    y,
                    tick,
                    auxiliary: 2,
                    kind: ItemUseKind::Scanner
                }
            );
            assert!(report127(&body).unwrap().forward.is_none());
        }
    }

    #[test]
    fn captured_needle_still_decodes_and_bad_item_use_is_rejected() {
        // CP037 Needle consume body, also used by the service life-action tests.
        let needle = [
            0x5a, 0x5a, 0xa5, 0x11, 0xa4, 0x64, 0x96, 0xca, 0x59, 0x35, 0x96, 0xca, 0x53, 0x90,
            0x5b, 0xa5, 0xf8, 0x5a, 0x96, 0xca, 0x53, 0x95,
        ];
        assert_eq!(item_use127(&needle).unwrap().kind, ItemUseKind::Needle);
        for len in 0..SCANNER_A.len() {
            assert!(item_use127(&SCANNER_A[..len]).is_err());
        }
        assert!(item_use127(&[SCANNER_A.as_slice(), &[0]].concat()).is_err());
        for (offset, invalid) in [(0, 0x5b), (14, 0x5a), (16, 0x96), (17, 0x5b), (21, 0x94)] {
            let mut body = SCANNER_A;
            body[offset] = invalid;
            assert!(item_use127(&body).is_err());
        }
    }

    #[test]
    fn captured_dart_group11_spring_drill_and_shield_codecs() {
        // p100b Dart C2S 138 live payloads
        let dart_left = [0x5a, 0x5a, 0xa5, 0xc9, 0xa7, 0x61, 0x5a, 0xa5, 0xc9];
        let parsed_left = dart138(&dart_left).unwrap();
        assert_eq!(
            parsed_left,
            Dart138 {
                world_index: 0,
                x_px: 160,
                y_px: 520,
                dir: 0,
                distance_px: 160,
            }
        );
        assert_eq!(dart172(parsed_left).unwrap(), dart_left);

        let dart_down = [0x5a, 0x5a, 0xa5, 0xc9, 0xa7, 0x61, 0x59, 0xa5, 0x41];
        let parsed_down = dart138(&dart_down).unwrap();
        assert_eq!(
            parsed_down,
            Dart138 {
                world_index: 0,
                x_px: 160,
                y_px: 520,
                dir: 3,
                distance_px: 40,
            }
        );
        assert_eq!(dart172(parsed_down).unwrap(), dart_down);
        assert!(dart138(&[0x5a, 0x5b, 0xa5, 0xc9, 0xa7, 0x61, 0x59, 0xa5, 0x41]).is_err());
        assert!(dart138(&dart_left[..8]).is_err());

        // p100b Group 11 C2S 139 live payloads
        for (bytes, slot, kind_flags, sel, row, col) in [
            ([0x5a, 0x5b, 0x51, 0x58], 0, 1, 1, 11, 2),
            ([0x5a, 0x5b, 0x56, 0x5d], 0, 1, 1, 12, 7),
            ([0x5a, 0xde, 0x51, 0x5a], 0, 132, 4, 11, 0),
            ([0x5a, 0x59, 0x50, 0x5a], 0, 3, 3, 10, 0),
        ] {
            let p = place_item139(&bytes).unwrap();
            assert_eq!(p.slot, slot);
            assert_eq!(p.kind_flags, kind_flags);
            assert_eq!(p.selector(), sel);
            assert_eq!(p.row, row);
            assert_eq!(p.col, col);
            assert_eq!(
                place_item173(p.kind_flags, p.row, p.col).unwrap(),
                bytes[1..]
            );
        }
        assert!(place_item139(&[0x5a, 0x5b, 0x51]).is_err());

        // Trigger 126 <-> 168 round-trip
        let trig_bytes = [
            super::super::put(0, Width::U8).unwrap()[0],
            super::super::put(1, Width::U8).unwrap()[0],
            super::super::put(11, Width::U8).unwrap()[0],
            super::super::put(2, Width::U8).unwrap()[0],
        ];
        let trig = trigger_item126(&trig_bytes).unwrap();
        assert_eq!(
            trig,
            TriggerItem126 {
                world_index: 0,
                slot: 1,
                row: 11,
                col: 2,
            }
        );
        let reply168 = trigger_item168(1, trig.world_index, trig.slot, trig.row, trig.col).unwrap();
        assert_eq!(reply168.len(), 5);
        assert_eq!(uint(&reply168[0..1], Width::U8).unwrap(), 1);
        assert_eq!(reply168[1..], trig_bytes);

        // p100b Spring 127/sub2 live payload
        let spring_bytes = [
            0x58, 0x5a, 0xa5, 0x41, 0xa4, 0xde, 0x96, 0xca, 0x55, 0xee, 0x96, 0xca, 0x53, 0x9d,
            0x96, 0xca, 0x53, 0x9c, 0x96, 0xca, 0x57, 0x8d, 0x97, 0xc2, 0x53, 0x95, 0x4b,
        ];
        let s = spring127(&spring_bytes).unwrap();
        assert_eq!(
            s,
            Spring127 {
                slot: 0,
                x: 40,
                y: 439,
                tick: 1659,
                extra: 8,
                row: 8,
                col: 1,
                packed_step: 17,
            }
        );
        let s170 = spring170(s.slot, s.x, s.y, s.row, s.col, s.packed_step).unwrap();
        assert_eq!(s170.len(), 9);
        assert_eq!(uint(&s170[6..7], Width::U8).unwrap(), 8);
        assert_eq!(uint(&s170[7..8], Width::U8).unwrap(), 1);
        assert_eq!(uint(&s170[8..9], Width::U8).unwrap(), 17);

        // Drill 127/sub3 -> 170/sub3 round-trip
        let mut raw_drill = vec![24, 8, 0, 0, 0, 0];
        raw_drill.extend_from_slice(&0x1122_3344u32.to_le_bytes());
        raw_drill.extend_from_slice(&[5, 6, 2]);
        let mut drill_wire = Vec::new();
        for (v, w) in [
            (u64::from(DRILL_SUBTYPE), Width::U8),
            (1, Width::U8),
            (120, Width::U16),
            (240, Width::U16),
            (555, Width::U32),
            (3, Width::U32),
            (raw_drill.len() as u64, Width::U32),
        ] {
            drill_wire.extend(super::super::put(v, w).unwrap());
        }
        drill_wire.extend(super::super::encode_bytes(&raw_drill).unwrap());
        let d = drill127(&drill_wire).unwrap();
        assert_eq!(d.box_id, 0x1122_3344);
        assert_eq!((d.row, d.col, d.dir), (5, 6, 2));
        let d170 = drill170(d).unwrap();
        assert_eq!(d170[..6], drill_wire[..6]);
        assert_eq!(d170[6..], drill_wire[14..]);

        // Shield 127/sub5 -> 170/sub5 round-trip
        let mut shield_wire = Vec::new();
        for (v, w) in [
            (u64::from(SHIELD_SUBTYPE), Width::U8),
            (0, Width::U8),
            (80, Width::U16),
            (160, Width::U16),
            (999, Width::U32),
            (1, Width::U32),
        ] {
            shield_wire.extend(super::super::put(v, w).unwrap());
        }
        let sh = shield127(&shield_wire).unwrap();
        assert_eq!(
            (sh.slot, sh.x, sh.y, sh.tick, sh.extra),
            (0, 80, 160, 999, 1)
        );
        assert_eq!(shield170(sh.slot, sh.x, sh.y).unwrap(), shield_wire[..6]);

        // p100b live 127/sub0 payloads for Dart 143, Bond 224, Pin 226, Banana1 227, Spring 148
        let live_sub0 = [
            (
                [
                    0x5a, 0x5a, 0xa5, 0xff, 0xa7, 0x6e, 0x96, 0xca, 0x55, 0x2f, 0x96, 0xca, 0x53,
                    0x97, 0x5b, 0xa5, 0xe6, 0x5a, 0x96, 0xca, 0x53, 0x95,
                ],
                ItemUseKind::Dart,
            ),
            (
                [
                    0x5a, 0x5a, 0xa5, 0x39, 0xa4, 0xb6, 0x96, 0xca, 0x5f, 0x11, 0x96, 0xca, 0x53,
                    0x97, 0x5b, 0xa5, 0x89, 0x5a, 0x96, 0xca, 0x53, 0x95,
                ],
                ItemUseKind::Bond,
            ),
            (
                [
                    0x5a, 0x5a, 0xa5, 0x69, 0xa4, 0xb6, 0x96, 0xca, 0x5d, 0x69, 0x96, 0xca, 0x53,
                    0x97, 0x5b, 0xa5, 0x8b, 0x5a, 0x96, 0xca, 0x53, 0x95,
                ],
                ItemUseKind::Pin,
            ),
            (
                [
                    0x5a, 0x5a, 0xa5, 0x69, 0xa4, 0xde, 0x96, 0xca, 0x43, 0xac, 0x96, 0xca, 0x53,
                    0x97, 0x5b, 0xa5, 0x8a, 0x5a, 0x96, 0xca, 0x53, 0x95,
                ],
                ItemUseKind::Banana1,
            ),
            (
                [
                    0x5a, 0x5a, 0xa5, 0x41, 0xa4, 0xde, 0x96, 0xca, 0x55, 0xee, 0x96, 0xca, 0x53,
                    0x9d, 0x5b, 0xa5, 0xfd, 0x5a, 0x96, 0xca, 0x53, 0x95,
                ],
                ItemUseKind::Spring,
            ),
        ];
        for (bytes, expected) in live_sub0 {
            assert_eq!(item_use127(&bytes).unwrap().kind, expected);
        }
    }

    #[test]
    fn captured_space228_and_pangland148_159_codecs() {
        // p108 CaJJ_Space02 run space-dual, C->S 228 seq 175 -> exact mirror S->C 350.
        let space228 = [
            0xa5, 0x64, 0xf3, 0xa6, 0x3a, 0xe1, 0xe5, 0x95, 0x65, 0xa6, 0xff, 0xbe, 0x32, 0xe1,
            0x39, 0xa5, 0x6f, 0xf3, 0xa6, 0x3a, 0xe1, 0x95, 0x59, 0xa5, 0x6f, 0xf3, 0xa6, 0x3a,
            0xe1, 0x94, 0x59, 0xa5, 0x6d, 0xfa, 0xa6, 0x32, 0xf3, 0x96, 0xca, 0x54, 0x45, 0x96,
            0xca, 0x53, 0xcf, 0x96, 0xca, 0x53, 0x91, 0x96, 0x35, 0x53, 0x95, 0x96, 0xca, 0x53,
            0x94, 0x69, 0x35, 0xac, 0x6a, 0x96, 0xca, 0x53, 0x95, 0x5b, 0x5a,
        ];
        let parsed228 = space_change_tile228(&space228).unwrap();
        assert_eq!(
            parsed228,
            SpaceChangeTile228 {
                strings: [
                    b"tile36_static".to_vec(),
                    b"tile00".to_vec(),
                    b"tile01".to_vec(),
                    b"fall".to_vec(),
                ],
                danger_time: 2000,
                danger_alpha: 90,
                danger_speed: 4,
                danger_color: 16_711_680,
                cells: vec![(u32::MAX, 0)],
                flag_a: 1,
                flag_b: 0,
            }
        );
        assert_eq!(space_change_tile350(&space228).unwrap(), space228);
        assert!(space_change_tile228(&space228[..66]).is_err());
        let mut extra228 = space228.to_vec();
        extra228.push(0);
        assert!(space_change_tile228(&extra228).is_err());

        // p108 NXCL_PangLand15 (18B) & PangLand03 (28B, 30B) C->S 148 captures.
        let pang_early = [
            0x69, 0x35, 0xac, 0x6a, 0x5b, 0xa5, 0xdb, 0xa4, 0x75, 0x5b, 0x96, 0xca, 0x53, 0x94,
            0xa5, 0x6a, 0x5a, 0x5b,
        ];
        assert_eq!(
            sprinkler148(&pang_early).unwrap(),
            SprinklerReport148 {
                script_source: u32::MAX,
                x: 284,
                y: 178,
                power: 1,
                damage: 1,
                state: 3,
                cells: vec![(0, 1)],
            }
        );

        let pang_repeat_6 = [
            0x69, 0x35, 0xac, 0x6a, 0x5c, 0xa5, 0x78, 0xa4, 0x4b, 0x5b, 0x96, 0xca, 0x53, 0x94,
            0xa5, 0x6a, 0x5d, 0x5e, 0x51, 0x57, 0x56, 0x50, 0x5d, 0x51, 0x5a, 0x5f, 0x59, 0x5b,
        ];
        assert_eq!(
            sprinkler148(&pang_repeat_6).unwrap().cells,
            vec![(7, 4), (11, 13), (12, 10), (7, 11), (0, 5), (3, 1)]
        );

        let pang_repeat_7 = [
            0x69, 0x35, 0xac, 0x6a, 0x5d, 0xa5, 0x78, 0xa4, 0x4b, 0x58, 0x96, 0xca, 0x53, 0x94,
            0xa5, 0x6a, 0x5f, 0x5d, 0x5d, 0x53, 0x5d, 0x50, 0x5d, 0x5f, 0x56, 0x52, 0x56, 0x5e,
            0x5b, 0x5e,
        ];
        let parsed_7 = sprinkler148(&pang_repeat_7).unwrap();
        assert_eq!(parsed_7.power, 2);
        assert_eq!(parsed_7.cells.len(), 7);
        assert!(sprinkler148(&pang_early[..17]).is_err());

        // Exact 39B body decoded from p108 followup S->C 159 capture (frame seq 115, owner 25, id 901).
        let expected_159 = [
            0x5a, 0x43, 0xa5, 0x69, 0x69, 0x35, 0xac, 0x6a, 0x96, 0xca, 0x53, 0x9f, 0x96, 0xca,
            0x52, 0x80, 0x5d, 0x58, 0x5b, 0x96, 0xca, 0x53, 0x94, 0x5a, 0xa6, 0xec, 0xa5, 0x69,
            0x5b, 0x00, 0x00, 0x00, 0x00, 0x71, 0xb1, 0x85, 0x89, 0x5a, 0x5a,
        ];
        assert_eq!(
            sprinkler_spawn159(u32::MAX, 10, 277, 7, 2, 1, 1, 901).unwrap(),
            expected_159
        );
        assert!(sprinkler_spawn159(u32::MAX, 10, 277, 13, 2, 1, 1, 901).is_err());
    }
}
