//! Measured room body layouts; no session, authority, timer or membership state.
use super::{decode_bytes, encode_bytes, put, uint};
use crate::{Width, read_uint};
use std::io;

fn supported_player_body(body: &[u8]) -> bool {
    // Preserve the original measured fixed-size carriers, including opaque
    // fields. Other name sizes must use the explicit zero-account schema.
    [200, 204].contains(&body.len()) || super::PlayerTemplate::parse_account(body).is_ok()
}

/// Actual KR room UI producers:92/u8+u16 and97/u8. One local player
/// only; this does not establish multi-local-player or entitlement support.
pub fn character92(body: &[u8]) -> io::Result<u16> {
    local_character92(body, 1).map(|(_, character)| character)
}
/// Current KR D61990 writes its local ordinal as u8 and character as u16.
/// Multi-local serving remains separately gated until native room acceptance.
pub fn local_character92(body: &[u8], count: usize) -> io::Result<(usize, u16)> {
    if !(1..=2).contains(&count) || body.len() != 3 {
        return Err(io::Error::other("unsupported character92 envelope"));
    }
    let local = uint(&body[..1], Width::U8)? as usize;
    if local >= count {
        return Err(io::Error::other("unsupported character92 local player"));
    }
    Ok((local, uint(&body[1..], Width::U16)? as u16))
}

pub fn team97(body: &[u8]) -> io::Result<u8> {
    local_team97(body, 1).map(|(_, team)| team)
}
/// D61890 calls678F10(local, team): high/low four bits, then typed u8.
/// This retains the existing supported team choices, not all possible modes.
pub fn local_team97(body: &[u8], count: usize) -> io::Result<(usize, u8)> {
    if !(1..=2).contains(&count) || body.len() != 1 {
        return Err(io::Error::other("unsupported team97 envelope"));
    }
    let packed = uint(body, Width::U8)? as u8;
    let local = usize::from(packed >> 4);
    let team = packed & 15;
    if local >= count || !matches!(team, 0 | 3) {
        return Err(io::Error::other("unsupported team97/local player"));
    }
    Ok((local, team))
}

/// Native DC6700 reads character from bits16..28 of the slot state.
/// Preserve every other bit; choice entitlement belongs to the service.
pub fn player106_character(body: &[u8], character: u16) -> io::Result<Vec<u8>> {
    if !supported_player_body(body) || character > 0x1fff {
        return Err(io::Error::other("unsupported player character projection"));
    }
    let mut out = body.to_vec();
    let prior = uint(&body[12..16], Width::U32)?;
    out[12..16].copy_from_slice(&put(
        (prior & !0x1fff0000) | (u64::from(character) << 16),
        Width::U32,
    )?);
    Ok(out)
}

pub fn player106(
    template: &[u8],
    identity: u32,
    slot: u8,
    team: u8,
    ready: bool,
    chief: bool,
) -> io::Result<Vec<u8>> {
    if !supported_player_body(template) || uint(&template[1..5], Width::U32)? != u64::from(identity)
    {
        return Err(io::Error::other("wrong pinned player106 identity"));
    }
    player106_state(template, slot, team, ready, chief)
}

pub fn player106_state(
    template: &[u8],
    slot: u8,
    team: u8,
    ready: bool,
    chief: bool,
) -> io::Result<Vec<u8>> {
    if !supported_player_body(template) || slot >= 26 || !matches!(team, 0 | 1 | 3) {
        return Err(io::Error::other("unsupported player106 slot/team"));
    }
    let mut body = template.to_vec();
    body[0] = put(u64::from(slot), Width::U8)?[0];
    let prior = uint(&body[12..16], Width::U32)?;
    let low = if chief {
        2
    } else if ready {
        4
    } else {
        0
    };
    body[12..16].copy_from_slice(&put(
        (prior & 0xffff0000) | low | (u64::from(team) << 5),
        Width::U32,
    )?);
    Ok(body)
}

/// Measured Free/Manner team selector, echoed by 86/87/180. It has no
/// established relationship to Attr/MapGroup or to Start121's roomtype.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum RoomTeamSelector {
    Free = 0,
    Manner = 1,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Create81 {
    pub title: Vec<u8>,
    pub team_selector: RoomTeamSelector,
}

pub fn create81(body: &[u8]) -> io::Result<Create81> {
    if !(57..=88).contains(&body.len()) {
        return Err(io::Error::other("create81 size"));
    }
    let n = uint(&body[..2], Width::U16)? as usize;
    if !(1..=32).contains(&n) || body.len() != 56 + n || uint(&body[2 + n..4 + n], Width::U16)? != 0
    {
        return Err(io::Error::other("create81 title/password branch"));
    }
    let title = decode_bytes(&body[2..2 + n])?;
    // Opaque client title bytes: bounded/control-free CP949 carrier, not UTF-8.
    // No Unicode normalization or account name semantics are inferred.
    if title.iter().any(|b| *b < 32 || *b == 127) {
        return Err(io::Error::other("title control bytes"));
    }
    for i in 0..12 {
        uint(&body[4 + n + i * 4..8 + n + i * 4], Width::U32)?;
    }
    for i in 0..4 {
        uint(&body[52 + n + i..53 + n + i], Width::U8)?;
    }
    let team_selector = match uint(&body[8 + n..12 + n], Width::U32)? {
        0 => RoomTeamSelector::Free,
        1 => RoomTeamSelector::Manner,
        _ => return Err(io::Error::other("unsupported fixed room selector")),
    };
    Ok(Create81 {
        title,
        team_selector,
    })
}

pub fn create81_title(body: &[u8]) -> io::Result<Vec<u8>> {
    Ok(create81(body)?.title)
}

pub fn join82_context(body: &[u8]) -> io::Result<()> {
    if body.len() != 11 {
        return Err(io::Error::other("join82 size"));
    }
    let widths = [Width::U16, Width::U32, Width::U16, Width::U8, Width::U16];
    let expected = [1, 1, 0, 1, 5];
    let mut at = 0;
    for (w, expected) in widths.into_iter().zip(expected) {
        let n = match w {
            Width::U8 => 1,
            Width::U16 => 2,
            Width::U32 => 4,
            Width::U64 => 8,
        };
        if uint(&body[at..at + n], w)? != expected {
            return Err(io::Error::other("wrong fixed room join context"));
        }
        at += n;
    }
    Ok(())
}

pub fn zero585_setup(body: &[u8]) -> io::Result<bool> {
    Ok(body.len() == 4 && uint(body, Width::U32)? == 0)
}

/// Parsed offsets of the measured raw Start profile-list branch. The map name
/// is length-prefixed; offsets after it must not depend on ExVillage01's length.
struct StartLayout {
    stream: Vec<u8>,
    size: usize,
    rows: usize,
    count: usize,
}
fn start_layout(body: &[u8]) -> io::Result<StartLayout> {
    let error = || io::Error::other("unsupported Start map/profile layout");
    if !(119..=2048).contains(&body.len()) || uint(&body[..1], Width::U8)? != 0 {
        return Err(error());
    }
    let size = uint(&body[1..5], Width::U32)? as usize;
    if size.checked_add(23) != Some(body.len()) {
        return Err(error());
    }
    let stream = decode_bytes(&body[5..5 + size])?;
    if stream.len() < 44
        || stream[0] != 24
        || stream[5] != 0
        || u32::from_le_bytes(stream[1..5].try_into().expect("bounded")) as usize != size - 5
    {
        return Err(error());
    }
    let name_size = u32::from_le_bytes(stream[28..32].try_into().expect("bounded")) as usize;
    if !(1..=64).contains(&name_size) {
        return Err(error());
    }
    let rows = 43 + name_size;
    if rows > size
        || !stream[32..32 + name_size]
            .iter()
            .all(|b| b.is_ascii_alphanumeric() || *b == b'_')
    {
        return Err(error());
    }
    let count = usize::from(stream[rows - 5]);
    if !(2..=8).contains(&count) || size != rows + count * 26 {
        return Err(error());
    }
    Ok(StartLayout {
        stream,
        size,
        rows,
        count,
    })
}

/// CP020 raw profile-list prefix. The roomtype byte is carried as read;
/// only roomtype 0 has a measured contract here. No F2 IDs are inferred.
#[derive(Debug, PartialEq, Eq)]
pub struct Start121Map {
    pub map_code: u16,
    pub name: Vec<u8>,
    pub dynamic_index: u32,
    pub roomtype: u8,
}

pub fn start121_map_context(body: &[u8]) -> io::Result<Start121Map> {
    let parsed = start_layout(body)?;
    let name_end = parsed.rows - 11;
    Ok(Start121Map {
        map_code: u16::from_le_bytes(parsed.stream[26..28].try_into().expect("bounded")),
        name: parsed.stream[32..name_end].to_vec(),
        dynamic_index: u32::from_le_bytes(
            parsed.stream[name_end..name_end + 4]
                .try_into()
                .expect("bounded"),
        ),
        roomtype: parsed.stream[name_end + 4],
    })
}

pub fn start121_map(body: &[u8]) -> io::Result<(u16, Vec<u8>)> {
    let map = start121_map_context(body)?;
    Ok((map.map_code, map.name))
}

/// Map code and dynamic index of the measured eight byte S→C117 branch.
pub fn map117_selection(body: &[u8]) -> io::Result<(u16, u32)> {
    if body.len() != 8 {
        return Err(io::Error::other("unsupported map117 size"));
    }
    Ok((
        uint(&body[2..4], Width::U16)? as u16,
        uint(&body[4..8], Width::U32)? as u32,
    ))
}

/// Select the independently retained two-row branch from a measured expanded
/// Start. Identity is absent from these rows; only contiguous slots0/1 qualify.
pub fn start121_duel(body: &[u8]) -> io::Result<Vec<u8>> {
    start121_roster(body, &[0, 1])
}

/// Select measured contiguous rows without expanding or changing their teams.
/// Admission decides which profiles have native acceptance; this is wire only.
pub fn start121_roster(body: &[u8], teams: &[u8]) -> io::Result<Vec<u8>> {
    if !matches!(teams.len(), 2 | 4 | 8) || teams.iter().any(|team| *team > 1) {
        return Err(io::Error::other("unsupported roster Start teams"));
    }
    let parsed = start_layout(body)?;
    let StartLayout {
        stream,
        size,
        rows,
        count,
    } = parsed;
    if teams.len() > count {
        return Err(io::Error::other("unsupported roster Start count"));
    }
    for (slot, team) in teams.iter().enumerate() {
        let row = &stream[rows + slot * 26..rows + (slot + 1) * 26];
        if row[0] != slot as u8
            || row[2] != slot as u8
            || u16::from_le_bytes([row[3], row[4]]) != slot as u16
            || row[5] != *team
        {
            return Err(io::Error::other("unsupported roster Start slot/team"));
        }
    }
    let selected_size = rows + teams.len() * 26;
    let mut out_stream = stream[..selected_size].to_vec();
    out_stream[1..5].copy_from_slice(&((selected_size - 5) as u32).to_le_bytes());
    out_stream[rows - 5] = teams.len() as u8;
    let mut out = body[..1].to_vec();
    out.extend(put(selected_size as u64, Width::U32)?);
    out.extend(encode_bytes(&out_stream)?);
    out.extend_from_slice(&body[5 + size..]);
    Ok(out)
}

/// One current player in a Start. Character Attr key is independent of slot
/// and spawn: native M7e control reverses keys0/1 while retaining both others.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StartPlayer {
    pub slot: u8,
    pub spawn: u8,
    pub character: u16,
    pub team: u8,
}

/// Read character keys by slot from the measured Start roster layout.
pub fn start121_characters(body: &[u8]) -> io::Result<Vec<(u8, u16)>> {
    let StartLayout { stream, rows, .. } = start_layout(body)?;
    Ok(stream[rows..]
        .as_chunks::<26>()
        .0
        .iter()
        .map(|row| (row[0], u16::from_le_bytes([row[3], row[4]])))
        .collect())
}

/// Build current player rows from explicit fields, retaining the measured map
/// prefix and outer tail. Service admission owns supported characters/rosters;
/// accepting a wire shape here does not establish native compatibility.
pub fn start121_players(template: &[u8], players: &[StartPlayer]) -> io::Result<Vec<u8>> {
    if !(2..=8).contains(&players.len()) {
        return Err(io::Error::other("Start player count outside bound"));
    }
    // Validate the retained source profile, not newly supplied player values.
    let base = start121_duel(template)?;
    let StartLayout {
        stream, size, rows, ..
    } = start_layout(&base)?;
    const ROW_TAIL: [u8; 20] = [
        0, 0, 255, 0, 0, 0, 0, 255, 255, 255, 255, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ];
    // Opaque flags/partner/u32s remain exactly the measured row branch. Do not
    // silently transplant an unmeasured profile into this serializer.
    for row in stream[rows..].as_chunks::<26>().0 {
        if row[1] != 0 || row[6..] != ROW_TAIL {
            return Err(io::Error::other("unsupported Start player row branch"));
        }
    }
    let mut ordered = players.to_vec();
    ordered.sort_by_key(|player| player.slot);
    let mut slots = std::collections::BTreeSet::new();
    let mut spawns = std::collections::BTreeSet::new();
    let mut selected = stream[..rows].to_vec();
    selected[rows - 5] = ordered.len() as u8;
    for player in ordered {
        if player.slot >= 8
            || player.spawn >= 8
            || player.character > 0x1fff
            || player.team > 31
            || !slots.insert(player.slot)
            || !spawns.insert(player.spawn)
        {
            return Err(io::Error::other("invalid or duplicate Start player fields"));
        }
        selected.extend([player.slot, 0, player.spawn]);
        selected.extend(player.character.to_le_bytes());
        selected.push(player.team);
        selected.extend(ROW_TAIL);
    }
    let selected_size = selected.len();
    selected[1..5].copy_from_slice(&((selected_size - 5) as u32).to_le_bytes());
    let mut out = base[..1].to_vec();
    out.extend(put(selected_size as u64, Width::U32)?);
    out.extend(encode_bytes(&selected)?);
    out.extend_from_slice(&base[5 + size..]);
    Ok(out)
}

pub fn start121_teams(body: &[u8], teams: &[u8]) -> io::Result<Vec<u8>> {
    if teams.len() != 8 || teams.iter().any(|team| *team > 1) {
        return Err(io::Error::other("expected measured eight start121 teams"));
    }
    let StartLayout {
        mut stream,
        size,
        rows,
        count,
    } = start_layout(body)?;
    if count != teams.len() {
        return Err(io::Error::other("eight start121 count mismatch"));
    }
    for (slot, team) in teams.iter().enumerate() {
        let at = rows + 26 * slot;
        if stream[at] != slot as u8
            || stream[at + 2] != slot as u8
            || u16::from_le_bytes([stream[at + 3], stream[at + 4]]) != slot as u16
        {
            return Err(io::Error::other("eight start121 row slot"));
        }
        stream[at + 5] = *team;
    }
    let mut out = body[..5].to_vec();
    out.extend(encode_bytes(&stream)?);
    out.extend_from_slice(&body[5 + size..]);
    Ok(out)
}

/// Read the UDP port shared by the native process. One local actor uses
/// the 23-byte shape; two actors use 24 bytes. The per-actor words do not
/// identify a DB owner or room slot (both were zero in the 2P capture).
pub fn loading99_port(body: &[u8], local_count: usize) -> io::Result<u16> {
    if !matches!((local_count, body.len()), (1, 23) | (2, 24)) {
        return Err(io::Error::other("unsupported loading99 size"));
    }
    let port = uint(&body[8..10], Width::U16)? as u16;
    if port == 0 {
        return Err(io::Error::other("loading UDP port zero"));
    }
    Ok(port)
}

pub fn peer153(
    template: &[u8],
    key: u32,
    recipient: u8,
    endpoints: &[(u32, u16)],
) -> io::Result<Vec<u8>> {
    if template.len() < 35
        || !matches!(endpoints.len(), 2 | 4 | 8)
        || usize::from(recipient) >= endpoints.len()
    {
        return Err(io::Error::other("unsupported diagnostic peer153"));
    }
    let mut out = template[..35].to_vec();
    out[15..19].copy_from_slice(&put(u64::from(key), Width::U32)?);
    for (slot, remote_endpoint) in endpoints.iter().enumerate() {
        let active = true;
        out.extend(put(u64::from(active), Width::U8)?);
        if active {
            let (ip, port) = if slot != usize::from(recipient) {
                *remote_endpoint
            } else {
                (0, 0)
            };
            if slot != usize::from(recipient) && (ip == 0 || port == 0) {
                return Err(io::Error::other("missing current peer endpoint"));
            }
            for (address, endpoint) in [(ip, port), (0, 0), (0, 0)] {
                out.extend(put(u64::from(address), Width::U32)?);
                out.extend(put(u64::from(endpoint), Width::U16)?);
            }
        }
    }
    for _ in endpoints.len()..26 {
        out.extend(put(0, Width::U8)?);
    }
    Ok(out)
}

/// Explicit slot table: absent slots get only the inactive byte. The first35
/// bytes are the measured branch; self endpoints remain zero.
pub fn peer153_players(
    template: &[u8],
    key: u32,
    recipient: u8,
    players: &[(u8, u32, u16)],
) -> io::Result<Vec<u8>> {
    peer153_for_local_slots(template, key, &[recipient], players)
}

/// Keep each local actor present with zero endpoints so the client retains
/// both actors. A Windows 2P diagnostic with two occupied local rows and no
/// remote peers reached the running game clock. Mixed routing is separate.
pub fn peer153_for_local_slots(
    template: &[u8],
    key: u32,
    local_slots: &[u8],
    players: &[(u8, u32, u16)],
) -> io::Result<Vec<u8>> {
    let local_pair = match local_slots {
        [_] => false,
        [a, b] if a != b => true,
        _ => return Err(io::Error::other("invalid local peer slots")),
    };
    if template.len() < 35 || !(2..=8).contains(&players.len()) {
        return Err(io::Error::other("unsupported player peer roster"));
    }
    let mut slots = std::collections::BTreeMap::new();
    for &(slot, ip, port) in players {
        let valid_endpoint = (ip != 0 && port != 0)
            || (local_pair && local_slots.contains(&slot) && ip == 0 && port == 0);
        if slot >= 8 || !valid_endpoint || slots.insert(slot, (ip, port)).is_some() {
            return Err(io::Error::other("invalid player peer slot/endpoint"));
        }
    }
    if local_slots.iter().any(|slot| !slots.contains_key(slot)) {
        return Err(io::Error::other("peer recipient outside roster"));
    }
    let mut out = template[..35].to_vec();
    out[15..19].copy_from_slice(&put(u64::from(key), Width::U32)?);
    for slot in 0..26 {
        out.extend(put(u64::from(slots.contains_key(&slot)), Width::U8)?);
        if let Some(&(ip, port)) = slots.get(&slot) {
            let (ip, port) = if local_slots.contains(&slot) {
                (0, 0)
            } else {
                (ip, port)
            };
            for (ip, port) in [(ip, port), (0, 0), (0, 0)] {
                out.extend(put(u64::from(ip), Width::U32)?);
                out.extend(put(u64::from(port), Width::U16)?);
            }
        }
    }
    Ok(out)
}

pub fn listing91_title(body: &[u8]) -> io::Result<Vec<u8>> {
    if body.len() < 16 || uint(&body[..2], Width::U16)? != 1 {
        return Err(io::Error::other("listing count"));
    }
    let n = uint(&body[2..6], Width::U32)? as usize;
    if n != body.len() - 6 {
        return Err(io::Error::other("listing blob size"));
    }
    let stream = decode_bytes(&body[6..])?;
    if stream.len() < 16
        || stream[0] != 0x18
        || u32::from_le_bytes(stream[1..5].try_into().expect("checked stream")) as usize != n - 5
        || stream[5] != 0
    {
        return Err(io::Error::other("listing envelope"));
    }
    let row = &stream[6..];
    if row.len() < 32
        || u16::from_le_bytes(row[..2].try_into().expect("checked row")) != 1
        || u32::from_le_bytes(row[2..6].try_into().expect("checked row")) != 1
    {
        return Err(io::Error::other("fixed listing tuple"));
    }
    let len = u32::from_le_bytes(row[6..10].try_into().expect("checked row")) as usize;
    if !(1..=32).contains(&len) || row.len() != 32 + len {
        return Err(io::Error::other("listing title size"));
    }
    Ok(row[10..10 + len].to_vec())
}

pub fn create81_selector(body: &[u8]) -> io::Result<u8> {
    Ok(create81(body)?.team_selector as u8)
}

pub fn selector_reply(body: &[u8], opcode: u32, selector: u8) -> io::Result<Vec<u8>> {
    let offset = match (opcode, body.len()) {
        (86, 11) => 7,
        (87, 7) => 5,
        (180, 3)
            if read_uint(&body[..1], Width::U8).map_err(io::Error::other)? == 0
                && read_uint(&body[2..], Width::U8).map_err(io::Error::other)? == 0 =>
        {
            1
        }
        _ => return Err(io::Error::other("unsupported mode reply")),
    };
    if selector > 1 {
        return Err(io::Error::other("unsupported mode selector"));
    }
    let mut out = body.to_vec();
    out[offset] = put(u64::from(selector), Width::U8)?[0];
    Ok(out)
}

pub fn listing91_replace_title(body: &[u8], title: &[u8]) -> io::Result<Vec<u8>> {
    listing91_title(body)?;
    if !(1..=32).contains(&title.len()) || title.iter().any(|b| *b < 32 || *b == 127) {
        return Err(io::Error::other("listing title bytes"));
    }
    let stream = decode_bytes(&body[6..])?;
    let row = &stream[6..];
    let old = u32::from_le_bytes(row[6..10].try_into().expect("validated row")) as usize;
    let mut updated = row[..6].to_vec();
    updated.extend_from_slice(&(title.len() as u32).to_le_bytes());
    updated.extend_from_slice(title);
    updated.extend_from_slice(&row[10 + old..]);
    let mut blob = vec![0x18];
    blob.extend_from_slice(&((updated.len() + 1) as u32).to_le_bytes());
    blob.push(0);
    blob.extend(updated);
    let mut out = body[..2].to_vec();
    out.extend(put(blob.len() as u64, Width::U32)?);
    out.extend(encode_bytes(&blob)?);
    Ok(out)
}

pub fn join87(template: &[u8], slot: u8) -> io::Result<Vec<u8>> {
    if template.len() != 7 || uint(&template[..1], Width::U8)? != 0 {
        return Err(io::Error::other("wrong join87 template"));
    }
    let mut out = template.to_vec();
    out[1] = put(u64::from(slot), Width::U8)?[0];
    Ok(out)
}

/// CP226 measured list rows, retaining22 opaque bytes without interpretation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListingRow {
    pub session: u16,
    pub room_id: u32,
    pub title: Vec<u8>,
    pub opaque: [u8; 22],
}
impl ListingRow {
    /// KR D9D720 reader sites D9D7E3/D9D817: measured current/capacity.
    /// Native Wine proof: room-membership.md#rust-room-population.
    /// The other20 opaque bytes stay exact.
    pub fn set_population(&mut self, current: usize, capacity: usize) -> io::Result<()> {
        if capacity == 0 || capacity > 26 || current > capacity {
            return Err(io::Error::other("invalid room list population"));
        }
        self.opaque[5] = u8::try_from(current).map_err(io::Error::other)?;
        self.opaque[7] = u8::try_from(capacity).map_err(io::Error::other)?;
        Ok(())
    }
}
fn row_read(data: &[u8], width: Width) -> io::Result<u64> {
    read_uint(data, width).map_err(io::Error::other)
}
fn row_put(value: u64, width: Width) -> io::Result<Vec<u8>> {
    crate::write_uint(value, width).map_err(io::Error::other)
}
pub fn listing91_from_rows(rows: &[ListingRow]) -> io::Result<Vec<u8>> {
    let count = u16::try_from(rows.len()).map_err(io::Error::other)?;
    let mut out = row_put(u64::from(count), Width::U16)?;
    if rows.is_empty() {
        return Ok(out);
    }
    let mut inner = Vec::new();
    for row in rows {
        if row.session == 0
            || row.room_id == 0
            || !(1..=32).contains(&row.title.len())
            || row.title.iter().any(|c| *c < 32 || *c == 127)
        {
            return Err(io::Error::other("unsupported measured listing row"));
        }
        inner.extend_from_slice(&row.session.to_le_bytes());
        inner.extend_from_slice(&row.room_id.to_le_bytes());
        inner.extend_from_slice(&(row.title.len() as u32).to_le_bytes());
        inner.extend_from_slice(&row.title);
        inner.extend_from_slice(&row.opaque);
    }
    let mut stream = vec![0x18];
    stream.extend_from_slice(&((inner.len() + 1) as u32).to_le_bytes());
    stream.push(0);
    stream.extend(inner);
    out.extend(row_put(stream.len() as u64, Width::U32)?);
    out.extend(encode_bytes(&stream)?);
    Ok(out)
}
pub fn listing91_parse_rows(body: &[u8]) -> io::Result<Vec<ListingRow>> {
    if body.len() < 2 {
        return Err(io::Error::other("count"));
    }
    let count = row_read(&body[..2], Width::U16)? as usize;
    if count == 0 {
        if body.len() != 2 {
            return Err(io::Error::other("empty tail"));
        }
        return Ok(Vec::new());
    }
    if body.len() < 6 {
        return Err(io::Error::other("blob size"));
    }
    let size = row_read(&body[2..6], Width::U32)? as usize;
    if body.len() != 6 + size {
        return Err(io::Error::other("blob length"));
    }
    let stream = decode_bytes(&body[6..])?;
    if size < 6
        || stream[0] != 0x18
        || stream[5] != 0
        || u32::from_le_bytes(stream[1..5].try_into().unwrap()) as usize != size - 5
    {
        return Err(io::Error::other("envelope"));
    }
    let mut cursor = 6;
    let mut rows = Vec::new();
    for _ in 0..count {
        if stream.len() - cursor < 32 {
            return Err(io::Error::other("row prefix"));
        }
        let b = &stream[cursor..];
        let session = u16::from_le_bytes(b[..2].try_into().unwrap());
        let room_id = u32::from_le_bytes(b[2..6].try_into().unwrap());
        let len = u32::from_le_bytes(b[6..10].try_into().unwrap()) as usize;
        if !(1..=32).contains(&len) || b.len() < 32 + len {
            return Err(io::Error::other("row title"));
        }
        rows.push(ListingRow {
            session,
            room_id,
            title: b[10..10 + len].to_vec(),
            opaque: b[10 + len..32 + len].try_into().unwrap(),
        });
        cursor += 32 + len;
    }
    if cursor != stream.len() {
        return Err(io::Error::other("unparsed row bytes"));
    }
    Ok(rows)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn local_choice_selectors_preserve_single_player_and_reject_foreign_ordinals() {
        // Raw bytes follow the current static D61890/D61990 writers and
        // 678F10 packer. Second-local vectors are not native runtime evidence.
        for (raw, local, team) in [(0x5a, 0, 0), (0x59, 0, 3), (0x4a, 1, 0), (0x49, 1, 3)] {
            assert_eq!(local_team97(&[raw], 2).unwrap(), (local, team));
            if local == 0 {
                assert_eq!(team97(&[raw]).unwrap(), team);
            } else {
                assert!(team97(&[raw]).is_err());
            }
        }
        assert_eq!(local_character92(&[0x5a, 0xa5, 0x68], 2).unwrap(), (0, 1));
        assert_eq!(local_character92(&[0x5b, 0xa5, 0x68], 2).unwrap(), (1, 1));
        assert!(character92(&[0x5b, 0xa5, 0x68]).is_err());
        for count in [0, 1, 2, 3] {
            assert!(local_team97(&[0x79], count).is_err()); // foreign local ordinal2
            assert!(local_character92(&[0x58, 0xa5, 0x68], count).is_err());
        }
        assert!(local_team97(&[0x48], 2).is_err()); // unsupported team2
        assert!(local_team97(&[0x49, 0], 2).is_err());
        assert!(local_character92(&[0x5b, 0xa5], 2).is_err());
    }
    #[test]
    fn create81_keeps_the_measured_team_selector_and_cp949_title_bytes() {
        // Existing Windows two-local create81 input, also used by the room
        // service tests. Mutations exercise the parser, not native behavior.
        let captured = "a5796d7ca955561ee3b52b059446e81ae952a56996ca539596ca539496ca539096ca539096ca539596ca539596ca53956935ac6a96ca539596ca539596ca539596ca53955a5a5a5a";
        let mut body: Vec<u8> = captured
            .as_bytes()
            .as_chunks::<2>()
            .0
            .iter()
            .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
            .collect();
        let request = create81(&body).unwrap();
        assert_eq!(request.team_selector, RoomTeamSelector::Manner);
        assert_eq!(request.title.len(), 16);
        assert!(request.title.iter().any(|byte| !byte.is_ascii()));
        assert_eq!(create81_title(&body).unwrap(), request.title);
        let offset = 8 + request.title.len();
        body[offset..offset + 4].copy_from_slice(&put(0, Width::U32).unwrap());
        assert_eq!(
            create81(&body).unwrap(),
            Create81 {
                title: request.title,
                team_selector: RoomTeamSelector::Free,
            }
        );
        body[offset..offset + 4].copy_from_slice(&put(2, Width::U32).unwrap());
        assert!(create81(&body).is_err());
        assert!(create81_selector(&body).is_err());
    }

    #[test]
    fn native_six_player_start_retains_map_and_rejects_inconsistent_count() {
        // CP283 native run eca76a98, not an encoder-generated oracle.
        // docs/evidence/rust-service/m7e-selected-rosters.json.
        let captured = "5a96ca534796ca9e8d96ca539596ca539596ca539596ca5395a6f3539597dc539596ca539eff9c2bd0f1ab3ff996fb63f096ca539596ca559596ca539596ca52959635539569ca53959635ac6a96ca539596ca539596cb539496ca539596ca536a6935ac9596ca536a96ca539596c8539596ca52979635539569ca53959635ac6a96ca539596ca539596c9539696ca509596ca536a6935ac9596ca536a96ca539596ce539595ca53919635539569ca53959635ac6a96ca539596ca539597cf539096ca509596ca536a6935ac9596ca536a96ca5395a5695a22a55aa56996ca539396ca528da5695a5a";
        let body: Vec<u8> = captured
            .as_bytes()
            .as_chunks::<2>()
            .0
            .iter()
            .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
            .collect();
        let parsed = start_layout(&body).unwrap();
        assert_eq!(parsed.count, 6);
        assert_eq!(
            start121_map_context(&body).unwrap(),
            Start121Map {
                map_code: 278,
                name: b"ExVillage01".to_vec(),
                dynamic_index: 0,
                roomtype: 0,
            }
        );
        assert_eq!(start121_map(&body).unwrap(), (278, b"ExVillage01".to_vec()));
        // Actual chosen characters/teams are not the retained diagnostic rows.
        assert!(start121_duel(&body).is_err());
        let mut stream = parsed.stream;
        stream[parsed.rows - 5] = 7;
        let mut malformed = body[..5].to_vec();
        malformed.extend(encode_bytes(&stream).unwrap());
        malformed.extend_from_slice(&body[5 + parsed.size..]);
        assert!(start121_map(&malformed).is_err());
    }

    #[test]
    fn four_peer_endpoints_are_distinct_self_zero_and_all_inactive_slots_explicit() {
        let endpoints = [
            (0x0a4d0002, 32001),
            (0x0a4d0003, 32002),
            (0x0a4d0004, 32003),
            (0x0a4d0005, 32004),
        ];
        for recipient in 0..4 {
            let body = peer153(&[0; 97], 12346, recipient, &endpoints).unwrap();
            assert_eq!(body.len(), 133);
            let mut at = 35;
            for (slot, endpoint) in endpoints.iter().enumerate() {
                assert_eq!(uint(&body[at..at + 1], Width::U8).unwrap(), 1);
                at += 1;
                let expected = if slot == usize::from(recipient) {
                    (0, 0)
                } else {
                    *endpoint
                };
                assert_eq!(
                    uint(&body[at..at + 4], Width::U32).unwrap(),
                    u64::from(expected.0)
                );
                assert_eq!(
                    uint(&body[at + 4..at + 6], Width::U16).unwrap(),
                    u64::from(expected.1)
                );
                for offset in [6, 12] {
                    assert_eq!(
                        uint(&body[at + offset..at + offset + 4], Width::U32).unwrap(),
                        0
                    );
                    assert_eq!(
                        uint(&body[at + offset + 4..at + offset + 6], Width::U16).unwrap(),
                        0
                    );
                }
                at += 18;
            }
            for _ in 4..26 {
                assert_eq!(uint(&body[at..at + 1], Width::U8).unwrap(), 0);
                at += 1;
            }
            assert_eq!(at, body.len());
        }
        assert!(peer153(&[0; 97], 12346, 4, &endpoints).is_err());
        assert!(peer153(&[0; 97], 12346, 0, &endpoints[..3]).is_err());
        let mut missing = endpoints;
        missing[1] = (0, 0);
        assert!(peer153(&[0; 97], 12346, 0, &missing).is_err());
        assert_eq!(
            peer153(&[0; 97], 12346, 0, &endpoints[..2]).unwrap().len(),
            97
        );
    }
}
