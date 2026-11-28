//! F7 normalized map validation for KR 00967.
//! Policy source: docs/evidence/content/f7-custom-map-spec.json (design).
//! Normalized JSON and content pins are for server storage. KR upload and
//! selection remain unmeasured, and validation grants no approval or authority.
//! Static anchors come from p233: docs/evidence/content/f5-f6-f7-ingame-wire-re.json.

use std::{
    collections::{BTreeMap, VecDeque},
    fmt::{self, Write as _},
};

pub const MAX_CUSTOM_MAP_BYTES: usize = 8192;
pub const MIN_CUSTOM_MAP_SPAWNS: usize = 2;
pub const MAX_CUSTOM_MAP_SPAWNS: usize = 8;
pub const CUSTOM_MAP_SCHEMA: &str = "kre-f7-normalized-map-v1";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CustomMapError {
    EmptyPayload,
    PayloadTooLarge,
    InvalidNormalizedJson,
    UnsupportedGrid,
    WrongTileCount,
    MissingTrustedPalette,
    UnknownTile(u16),
    InvalidSpawnCount,
    SpawnOutOfBounds([u16; 2]),
    DuplicateSpawn([u16; 2]),
    SolidSpawn([u16; 2]),
    UnreachableSpawn([u16; 2]),
    InsufficientSpawns,
    PinMismatch,
    UnmeasuredWireContract,
}

impl fmt::Display for CustomMapError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyPayload => f.write_str("empty custom map payload"),
            Self::PayloadTooLarge => write!(f, "custom map exceeds {MAX_CUSTOM_MAP_BYTES} bytes"),
            Self::InvalidNormalizedJson => f.write_str("invalid normalized custom map JSON"),
            Self::UnsupportedGrid => f.write_str("unsupported custom map grid"),
            Self::WrongTileCount => f.write_str("tile count does not match custom map grid"),
            Self::MissingTrustedPalette => f.write_str("no trusted custom map tile palette"),
            Self::UnknownTile(id) => {
                write!(f, "custom map tile {id} is not in the trusted palette")
            }
            Self::InvalidSpawnCount => write!(
                f,
                "custom map must have {MIN_CUSTOM_MAP_SPAWNS}–{MAX_CUSTOM_MAP_SPAWNS} spawns"
            ),
            Self::SpawnOutOfBounds([x, y]) => {
                write!(f, "custom map spawn ({x}, {y}) is out of bounds")
            }
            Self::DuplicateSpawn([x, y]) => write!(f, "duplicate custom map spawn ({x}, {y})"),
            Self::SolidSpawn([x, y]) => write!(f, "custom map spawn ({x}, {y}) is on a solid tile"),
            Self::UnreachableSpawn([x, y]) => {
                write!(f, "custom map spawn ({x}, {y}) is unreachable")
            }
            Self::InsufficientSpawns => f.write_str("custom map does not cover the active roster"),
            Self::PinMismatch => f.write_str("custom map content pin does not match"),
            Self::UnmeasuredWireContract => f.write_str("unmeasured_wire_contract"),
        }
    }
}

impl std::error::Error for CustomMapError {}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CustomMapGrid {
    width: u16,
    height: u16,
}

impl CustomMapGrid {
    pub const fn new(width: u16, height: u16) -> Result<Self, CustomMapError> {
        match (width, height) {
            (15, 13) | (21, 21) | (31, 13) => Ok(Self { width, height }),
            _ => Err(CustomMapError::UnsupportedGrid),
        }
    }

    pub const fn dimensions(self) -> (u16, u16) {
        (self.width, self.height)
    }

    pub fn tile_count(self) -> usize {
        usize::from(self.width) * usize::from(self.height)
    }

    fn index(self, [x, y]: [u16; 2]) -> Option<usize> {
        (x < self.width && y < self.height)
            .then(|| usize::from(y) * usize::from(self.width) + usize::from(x))
    }
}

/// An immutable validated snapshot. A consumer must check authorization and
/// verify an approved pin against a fresh validation with its current palette.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CustomMap {
    grid: CustomMapGrid,
    tiles: Vec<u16>,
    spawns: Vec<[u16; 2]>,
    content_pin: String,
}

impl CustomMap {
    /// Validates server storage JSON with the server's trusted palette (tile ID
    /// to walkability). The caller must also bound file reads before buffering.
    pub fn parse_normalized(
        payload: &[u8],
        trusted_palette: &BTreeMap<u16, bool>,
    ) -> Result<Self, CustomMapError> {
        check_payload_size(payload.len())?;
        let (grid, tiles, spawns) = Parser { rest: payload }.map()?;
        if tiles.len() != grid.tile_count() {
            return Err(CustomMapError::WrongTileCount);
        }
        if trusted_palette.is_empty() {
            return Err(CustomMapError::MissingTrustedPalette);
        }
        let walkable = tiles
            .iter()
            .map(|id| {
                trusted_palette
                    .get(id)
                    .copied()
                    .ok_or(CustomMapError::UnknownTile(*id))
            })
            .collect::<Result<Vec<_>, _>>()?;
        validate_spawns(grid, &walkable, &spawns)?;

        let mut map = Self {
            grid,
            tiles,
            spawns,
            content_pin: String::new(),
        };
        let canonical = map.canonical_map();
        // The normalized map itself has one trailing LF at the storage boundary.
        check_payload_size(canonical.len() + 1)?;
        let palette_pin = palette_sha256(trusted_palette);
        let bundle = format!("{{\"map\":{canonical},\"palette_sha256\":\"{palette_pin}\"}}\n");
        map.content_pin = sha256_hex(bundle.as_bytes());
        Ok(map)
    }

    pub const fn grid(&self) -> CustomMapGrid {
        self.grid
    }

    pub fn tiles(&self) -> &[u16] {
        &self.tiles
    }

    pub fn spawns(&self) -> &[[u16; 2]] {
        &self.spawns
    }

    /// Lowercase SHA-256 over canonical JSON binding the map and full palette.
    pub fn content_pin(&self) -> &str {
        &self.content_pin
    }

    pub fn verify_pin(&self, approved_pin: &str) -> Result<(), CustomMapError> {
        if self.content_pin == approved_pin {
            Ok(())
        } else {
            Err(CustomMapError::PinMismatch)
        }
    }

    /// Check the number of current players, independently of room capacity.
    pub fn validate_roster(&self, active_players: usize) -> Result<(), CustomMapError> {
        if active_players > self.spawns.len() {
            Err(CustomMapError::InsufficientSpawns)
        } else {
            Ok(())
        }
    }

    fn canonical_map(&self) -> String {
        let (width, height) = self.grid.dimensions();
        let mut json =
            format!("{{\"height\":{height},\"schema\":\"{CUSTOM_MAP_SCHEMA}\",\"spawns\":[");
        for (i, [x, y]) in self.spawns.iter().enumerate() {
            if i != 0 {
                json.push(',');
            }
            write!(json, "[{x},{y}]").expect("writing to a String cannot fail");
        }
        json.push_str("],\"tiles\":[");
        for (i, tile) in self.tiles.iter().enumerate() {
            if i != 0 {
                json.push(',');
            }
            write!(json, "{tile}").expect("writing to a String cannot fail");
        }
        write!(json, "],\"width\":{width}}}").expect("writing to a String cannot fail");
        json
    }
}

fn check_payload_size(len: usize) -> Result<(), CustomMapError> {
    match len {
        0 => Err(CustomMapError::EmptyPayload),
        1..=MAX_CUSTOM_MAP_BYTES => Ok(()),
        _ => Err(CustomMapError::PayloadTooLarge),
    }
}

fn validate_spawns(
    grid: CustomMapGrid,
    walkable: &[bool],
    spawns: &[[u16; 2]],
) -> Result<(), CustomMapError> {
    if !(MIN_CUSTOM_MAP_SPAWNS..=MAX_CUSTOM_MAP_SPAWNS).contains(&spawns.len()) {
        return Err(CustomMapError::InvalidSpawnCount);
    }

    let mut visited = vec![false; grid.tile_count()];
    let mut indices = Vec::with_capacity(spawns.len());
    for &spawn in spawns {
        let at = grid
            .index(spawn)
            .ok_or(CustomMapError::SpawnOutOfBounds(spawn))?;
        if indices.contains(&at) {
            return Err(CustomMapError::DuplicateSpawn(spawn));
        }
        if !walkable[at] {
            return Err(CustomMapError::SolidSpawn(spawn));
        }
        indices.push(at);
    }

    let width = usize::from(grid.width);
    let start = indices[0];
    visited[start] = true;
    let mut queue = VecDeque::from([start]);
    while let Some(at) = queue.pop_front() {
        let x = at % width;
        let neighbors = [
            (x > 0).then(|| at - 1),
            (x + 1 < width).then_some(at + 1),
            at.checked_sub(width),
            (at + width < walkable.len()).then_some(at + width),
        ];
        for next in neighbors.into_iter().flatten() {
            if walkable[next] && !visited[next] {
                visited[next] = true;
                queue.push_back(next);
            }
        }
    }

    for (at, &spawn) in indices.into_iter().zip(spawns) {
        if !visited[at] {
            return Err(CustomMapError::UnreachableSpawn(spawn));
        }
    }
    Ok(())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CustomMapWireStatus {
    Unmeasured,
}

impl CustomMapWireStatus {
    /// Name used by static anchors; it intentionally retains the existing
    /// unmeasured status and does not promote live wire acceptance.
    #[allow(non_upper_case_globals)]
    pub const UnmeasuredWireContract: Self = Self::Unmeasured;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CustomMapWireRoute {
    Upload,
    Select,
}

impl CustomMapWireRoute {
    pub const fn status(self) -> CustomMapWireStatus {
        match self {
            Self::Upload | Self::Select => CustomMapWireStatus::Unmeasured,
        }
    }

    /// Reject before a caller dispatches a request or sends an acknowledgement.
    pub const fn validate_dispatch(self) -> Result<(), CustomMapError> {
        match self.status() {
            CustomMapWireStatus::Unmeasured => Err(CustomMapError::UnmeasuredWireContract),
        }
    }

    /// A valid storage snapshot still cannot be serialized onto the KR wire.
    pub const fn encode(self, _map: &CustomMap) -> Result<Vec<u8>, CustomMapError> {
        match self.status() {
            CustomMapWireStatus::Unmeasured => Err(CustomMapError::UnmeasuredWireContract),
        }
    }
}

/// Static provenance for the KR 00967 CustomMap packet family.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CustomMapType666Anchors {
    pub dispatcher_va: u32,
    pub packet_type: u32,
    /// Inclusive bounds of the client switch; gaps can take the default branch.
    pub subtype_range: (u8, u8),
    pub upload_ctor_va: u32,
    pub upload_handler_va: u32,
    pub usermap_rtti_va: u32,
    pub usermap_vtable_va: u32,
    pub room_meta_opcode: u32,
    pub room_meta_reader_va: u32,
    pub status: CustomMapWireStatus,
}

/// The p233 static dispatcher and object anchors for packet type 666.
/// The local editor and object serializers (`0x508370`, `0x5091F0`,
/// `0x4621C0`, `0x57FD80`) do not establish a wire layout.
pub const CUSTOM_MAP_TYPE_666_ANCHORS: CustomMapType666Anchors = CustomMapType666Anchors {
    dispatcher_va: 0x5A3BF0,
    packet_type: 666,
    subtype_range: (19, 38),
    upload_ctor_va: 0x9BFC80,
    upload_handler_va: 0x5B11D0,
    usermap_rtti_va: 0x19C7E80,
    usermap_vtable_va: 0x175E38C,
    room_meta_opcode: 273,
    room_meta_reader_va: 0xD72FD0,
    status: CustomMapWireStatus::UnmeasuredWireContract,
};

/// Static reader/writer locations by class, without subtype or wire opcode mappings.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CustomMapPacketAnchor {
    pub name: &'static str,
    pub reader_va: u32,
    pub writer_va: u32,
    pub status: CustomMapWireStatus,
}

pub static CUSTOM_MAP_PACKET_ANCHORS: &[CustomMapPacketAnchor] = &[
    CustomMapPacketAnchor {
        name: "P_REQ_CHECK_HASH_VALUE",
        reader_va: 0x4D6980,
        writer_va: 0x4D68F0,
        status: CustomMapWireStatus::UnmeasuredWireContract,
    },
    CustomMapPacketAnchor {
        name: "P_REQ_PUBLISH",
        reader_va: 0x4D6C10,
        writer_va: 0x4D6B80,
        status: CustomMapWireStatus::UnmeasuredWireContract,
    },
    CustomMapPacketAnchor {
        name: "P_REQ_LOAD",
        reader_va: 0x4D6EC0,
        writer_va: 0x4D6E60,
        status: CustomMapWireStatus::UnmeasuredWireContract,
    },
    CustomMapPacketAnchor {
        name: "P_REQ_LOAD_SEARCH",
        reader_va: 0x4D7080,
        writer_va: 0x4D6FC0,
        status: CustomMapWireStatus::UnmeasuredWireContract,
    },
    CustomMapPacketAnchor {
        name: "P_REQ_GOOD",
        reader_va: 0x4D73E0,
        writer_va: 0x4D7350,
        status: CustomMapWireStatus::UnmeasuredWireContract,
    },
];

/// Finds a static packet anchor by its RTTI class name.
pub fn custom_map_anchor(name: &str) -> Option<&'static CustomMapPacketAnchor> {
    CUSTOM_MAP_PACKET_ANCHORS
        .iter()
        .find(|anchor| anchor.name == name)
}

type ParsedMap = (CustomMapGrid, Vec<u16>, Vec<[u16; 2]>);

/// Only the normalized F7 schema is accepted. Its only strings are known ASCII
/// keys and the schema tag, and its numbers are unsigned 16-bit integers.
struct Parser<'a> {
    rest: &'a [u8],
}

impl Parser<'_> {
    fn map(mut self) -> Result<ParsedMap, CustomMapError> {
        use CustomMapError::InvalidNormalizedJson;

        let (mut width, mut height, mut tiles, mut spawns) = (None, None, None, None);
        let mut schema_seen = false;
        self.expect(b'{')?;
        loop {
            let key = self.string()?;
            self.expect(b':')?;
            match key.as_slice() {
                b"schema" if !schema_seen => {
                    if self.string()? != CUSTOM_MAP_SCHEMA.as_bytes() {
                        return Err(InvalidNormalizedJson);
                    }
                    schema_seen = true;
                }
                b"width" if width.is_none() => width = Some(self.number()?),
                b"height" if height.is_none() => height = Some(self.number()?),
                b"tiles" if tiles.is_none() => tiles = Some(self.array(Self::number)?),
                b"spawns" if spawns.is_none() => {
                    spawns = Some(self.array(|parser| {
                        parser.expect(b'[')?;
                        let x = parser.number()?;
                        parser.expect(b',')?;
                        let y = parser.number()?;
                        parser.expect(b']')?;
                        Ok([x, y])
                    })?);
                }
                _ => return Err(InvalidNormalizedJson),
            }
            if self.take(b'}') {
                break;
            }
            self.expect(b',')?;
        }
        self.skip_space();
        let (Some(width), Some(height), Some(tiles), Some(spawns)) = (width, height, tiles, spawns)
        else {
            return Err(InvalidNormalizedJson);
        };
        if !schema_seen || !self.rest.is_empty() {
            return Err(InvalidNormalizedJson);
        }
        Ok((CustomMapGrid::new(width, height)?, tiles, spawns))
    }

    fn array<T>(
        &mut self,
        mut element: impl FnMut(&mut Self) -> Result<T, CustomMapError>,
    ) -> Result<Vec<T>, CustomMapError> {
        self.expect(b'[')?;
        let mut values = Vec::new();
        if !self.take(b']') {
            loop {
                values.push(element(self)?);
                if self.take(b']') {
                    break;
                }
                self.expect(b',')?;
            }
        }
        Ok(values)
    }

    fn skip_space(&mut self) {
        while matches!(self.rest.first(), Some(b' ' | b'\t' | b'\r' | b'\n')) {
            self.rest = &self.rest[1..];
        }
    }

    fn take(&mut self, byte: u8) -> bool {
        self.skip_space();
        if let Some(rest) = self.rest.strip_prefix(&[byte]) {
            self.rest = rest;
            true
        } else {
            false
        }
    }

    fn expect(&mut self, byte: u8) -> Result<(), CustomMapError> {
        if self.take(byte) {
            Ok(())
        } else {
            Err(CustomMapError::InvalidNormalizedJson)
        }
    }

    fn number(&mut self) -> Result<u16, CustomMapError> {
        use CustomMapError::InvalidNormalizedJson;

        self.skip_space();
        let digits = self.rest.iter().take_while(|b| b.is_ascii_digit()).count();
        let (number, rest) = self.rest.split_at(digits);
        if digits > 1 && number[0] == b'0' {
            return Err(InvalidNormalizedJson);
        }
        self.rest = rest;
        std::str::from_utf8(number)
            .ok()
            .and_then(|number| number.parse().ok())
            .ok_or(InvalidNormalizedJson)
    }

    fn string(&mut self) -> Result<Vec<u8>, CustomMapError> {
        use CustomMapError::InvalidNormalizedJson;

        self.expect(b'"')?;
        let mut bytes = Vec::new();
        while let Some((&byte, rest)) = self.rest.split_first() {
            self.rest = rest;
            match byte {
                b'"' => return Ok(bytes),
                b'\\' => {
                    // No recognized key or tag needs any escape besides \uXXXX.
                    let escape = rest.get(..5).ok_or(InvalidNormalizedJson)?;
                    if escape[0] != b'u' || !escape[1..].iter().all(u8::is_ascii_hexdigit) {
                        return Err(InvalidNormalizedJson);
                    }
                    let hex =
                        std::str::from_utf8(&escape[1..]).map_err(|_| InvalidNormalizedJson)?;
                    bytes.push(u8::from_str_radix(hex, 16).map_err(|_| InvalidNormalizedJson)?);
                    self.rest = &rest[5..];
                }
                b' '..=b'~' => bytes.push(byte),
                _ => return Err(InvalidNormalizedJson),
            }
        }
        Err(InvalidNormalizedJson)
    }
}

fn palette_sha256(palette: &BTreeMap<u16, bool>) -> String {
    let mut json = String::from("[");
    for (i, (&id, &walkable)) in palette.iter().enumerate() {
        if i != 0 {
            json.push(',');
        }
        write!(json, "[{id},{walkable}]").expect("writing to a String cannot fail");
    }
    json.push_str("]\n");
    sha256_hex(json.as_bytes())
}

// SHA-256 per FIPS 180-4. Kept private because kr-wire has no digest dependency.
fn sha256_hex(data: &[u8]) -> String {
    let mut state: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab,
        0x5be0cd19,
    ];
    let (blocks, tail) = data.as_chunks::<64>();
    for block in blocks {
        sha256_block(&mut state, block);
    }
    let mut padding = [0; 128];
    let end = if tail.len() < 56 { 64 } else { 128 };
    padding[..tail.len()].copy_from_slice(tail);
    padding[tail.len()] = 0x80;
    padding[end - 8..end].copy_from_slice(&(data.len() as u64 * 8).to_be_bytes());
    for block in padding[..end].as_chunks::<64>().0 {
        sha256_block(&mut state, block);
    }

    let mut hex = String::with_capacity(64);
    for word in state {
        write!(hex, "{word:08x}").expect("writing to a String cannot fail");
    }
    hex
}

fn sha256_block(state: &mut [u32; 8], block: &[u8]) {
    const K: [u32; 64] = [
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4,
        0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe,
        0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f,
        0x4a7484aa, 0x5cb0a9dc, 0x76f988da, 0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
        0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc,
        0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b,
        0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070, 0x19a4c116,
        0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
        0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7,
        0xc67178f2,
    ];
    let mut words = [0u32; 64];
    for (word, bytes) in words.iter_mut().zip(block.as_chunks::<4>().0) {
        *word = u32::from_be_bytes(*bytes);
    }
    for i in 16..64 {
        let x = words[i - 15];
        let y = words[i - 2];
        let s0 = x.rotate_right(7) ^ x.rotate_right(18) ^ (x >> 3);
        let s1 = y.rotate_right(17) ^ y.rotate_right(19) ^ (y >> 10);
        words[i] = words[i - 16]
            .wrapping_add(s0)
            .wrapping_add(words[i - 7])
            .wrapping_add(s1);
    }

    let [mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut h] = *state;
    for (k, w) in K.into_iter().zip(words) {
        let sum1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
        let choose = (e & f) ^ (!e & g);
        let t1 = h
            .wrapping_add(sum1)
            .wrapping_add(choose)
            .wrapping_add(k)
            .wrapping_add(w);
        let sum0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
        let majority = (a & b) ^ (a & c) ^ (b & c);
        let t2 = sum0.wrapping_add(majority);
        [a, b, c, d, e, f, g, h] = [t1.wrapping_add(t2), a, b, c, d.wrapping_add(t1), e, f, g];
    }
    for (out, word) in state.iter_mut().zip([a, b, c, d, e, f, g, h]) {
        *out = out.wrapping_add(word);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn payload(width: u16, height: u16, tiles: &str, spawns: &str) -> Vec<u8> {
        format!(
            r#"{{"schema":"{CUSTOM_MAP_SCHEMA}","width":{width},"height":{height},"tiles":{tiles},"spawns":{spawns}}}"#
        )
        .into_bytes()
    }

    fn palette() -> BTreeMap<u16, bool> {
        BTreeMap::from([(0, true), (1, false)])
    }

    fn open_tiles(count: usize) -> String {
        format!("[{}]", vec!["0"; count].join(","))
    }

    #[test]
    fn only_the_three_committed_grids_are_accepted() {
        for dimensions in [(15, 13), (21, 21), (31, 13)] {
            let (width, height) = dimensions;
            let tiles = open_tiles(usize::from(width) * usize::from(height));
            let bytes = payload(width, height, &tiles, "[[0,0],[1,0]]");
            let map = CustomMap::parse_normalized(&bytes, &palette()).unwrap();
            assert_eq!(map.grid().dimensions(), dimensions);
        }
        for (width, height) in [
            (0, 0),
            (15, 27),
            (31, 27),
            (16, 13),
            (13, 15),
            (u16::MAX, u16::MAX),
        ] {
            assert_eq!(
                CustomMapGrid::new(width, height),
                Err(CustomMapError::UnsupportedGrid)
            );
        }
    }

    #[test]
    fn parser_bounds_payload_and_requires_exact_tile_count() {
        assert_eq!(
            CustomMap::parse_normalized(&[], &palette()),
            Err(CustomMapError::EmptyPayload)
        );
        assert_eq!(
            CustomMap::parse_normalized(&vec![b' '; MAX_CUSTOM_MAP_BYTES + 1], &palette()),
            Err(CustomMapError::PayloadTooLarge)
        );
        for count in [194, 196] {
            let bytes = payload(15, 13, &open_tiles(count), "[[0,0],[1,0]]");
            assert_eq!(
                CustomMap::parse_normalized(&bytes, &palette()),
                Err(CustomMapError::WrongTileCount)
            );
        }
        let mut bytes = payload(15, 13, &open_tiles(195), "[[0,0],[1,0]]");
        bytes.resize(MAX_CUSTOM_MAP_BYTES, b' ');
        assert!(CustomMap::parse_normalized(&bytes, &palette()).is_ok());
        assert_eq!(
            CustomMap::parse_normalized(&bytes, &BTreeMap::new()),
            Err(CustomMapError::MissingTrustedPalette)
        );
        assert_eq!(
            CustomMap::parse_normalized(&bytes, &BTreeMap::from([(1, true)])),
            Err(CustomMapError::UnknownTile(0))
        );
    }

    #[test]
    fn rejects_invalid_json_and_schema() {
        let bytes = payload(15, 13, &open_tiles(195), "[[0,0],[1,0]]");
        let json = std::str::from_utf8(&bytes).unwrap();
        for invalid in [
            json.replace("\"width\":15", "\"width\":15,\"wid\\u0074h\":15"),
            json.replace("\"width\":15,", ""),
            json.replace("\"width\":15", "\"width\":15,\"script\":null"),
            json.replace(CUSTOM_MAP_SCHEMA, "kre-map-v2"),
            json.replace("\"width\":15", "\"width\":015"),
            json.replace("[0,0]", "[0,0.0]"),
            json.replace("[0,0]", "[0,-1]"),
            json.replace("[0,0]", "[0,0,0]"),
            json.replace("[0,0]", "[0,0,]"),
            json.replacen("\"tiles\":[0", "\"tiles\":[65536", 1),
            format!("{json}null"),
        ] {
            assert_eq!(
                CustomMap::parse_normalized(invalid.as_bytes(), &palette()),
                Err(CustomMapError::InvalidNormalizedJson),
                "accepted {invalid}"
            );
        }
        let mut invalid_utf8 = bytes;
        invalid_utf8[3] = 0xff;
        assert_eq!(
            CustomMap::parse_normalized(&invalid_utf8, &palette()),
            Err(CustomMapError::InvalidNormalizedJson)
        );
    }

    #[test]
    fn spawn_count_bounds_and_active_roster() {
        let tiles = open_tiles(195);
        let spawns = [
            "[0,0]", "[1,0]", "[2,0]", "[3,0]", "[4,0]", "[5,0]", "[6,0]", "[7,0]", "[8,0]",
        ];
        let parse = |count| {
            CustomMap::parse_normalized(
                &payload(15, 13, &tiles, &format!("[{}]", spawns[..count].join(","))),
                &palette(),
            )
        };
        for count in [0, 1, 9] {
            assert_eq!(parse(count), Err(CustomMapError::InvalidSpawnCount));
        }
        let map = parse(8).unwrap();
        assert_eq!(map.spawns().len(), 8);
        assert_eq!(map.validate_roster(8), Ok(()));
        assert_eq!(
            map.validate_roster(9),
            Err(CustomMapError::InsufficientSpawns)
        );
        for (spawns, coordinate) in [("[[0,0],[15,0]]", [15, 0]), ("[[0,0],[0,13]]", [0, 13])] {
            let bytes = payload(15, 13, &tiles, spawns);
            assert_eq!(
                CustomMap::parse_normalized(&bytes, &palette()),
                Err(CustomMapError::SpawnOutOfBounds(coordinate))
            );
        }
    }

    #[test]
    fn spawn_rules_reject_duplicates_solids_and_disconnected_components() {
        let tiles = open_tiles(195);
        let duplicate = payload(15, 13, &tiles, "[[0,0],[0,0]]");
        assert!(matches!(
            CustomMap::parse_normalized(&duplicate, &palette()),
            Err(CustomMapError::DuplicateSpawn([0, 0]))
        ));

        let mut solid_tiles = vec!["0"; 195];
        solid_tiles[1] = "1";
        let solid = payload(
            15,
            13,
            &format!("[{}]", solid_tiles.join(",")),
            "[[1,0],[0,0]]",
        );
        assert!(matches!(
            CustomMap::parse_normalized(&solid, &palette()),
            Err(CustomMapError::SolidSpawn([1, 0]))
        ));

        let mut split_tiles = vec!["0"; 195];
        for y in 0..13 {
            split_tiles[y * 15 + 7] = "1";
        }
        let disconnected = payload(
            15,
            13,
            &format!("[{}]", split_tiles.join(",")),
            "[[0,0],[14,0]]",
        );
        assert!(matches!(
            CustomMap::parse_normalized(&disconnected, &palette()),
            Err(CustomMapError::UnreachableSpawn([14, 0]))
        ));
    }

    #[test]
    fn reachability_uses_cardinal_edges_and_does_not_wrap_rows() {
        for (a, b, spawns, target) in [
            (0, 16, "[[0,0],[1,1]]", [1, 1]),
            (14, 15, "[[14,0],[0,1]]", [0, 1]),
        ] {
            let mut tiles = vec!["1"; 195];
            tiles[a] = "0";
            tiles[b] = "0";
            let bytes = payload(15, 13, &format!("[{}]", tiles.join(",")), spawns);
            assert_eq!(
                CustomMap::parse_normalized(&bytes, &palette()),
                Err(CustomMapError::UnreachableSpawn(target))
            );
        }
    }

    #[test]
    fn pin_matches_canonical_sha256_and_binds_order_and_full_palette() {
        let palette = BTreeMap::from([(65535, true), (2, false), (1, true)]);
        let tiles = format!("[1,2{}]", ",1".repeat(193));
        let bytes = payload(15, 13, &tiles, "[[0,0],[14,12]]");
        let map = CustomMap::parse_normalized(&bytes, &palette).unwrap();
        // Independent canonical JSON vectors from Python json and hashlib.
        let pin = "98e7efa8523bb17b8954c0967b81350b3589847cb5250e381fea70928d349b9e";
        assert_eq!(map.verify_pin(pin), Ok(()));
        let reordered = format!(
            "{{\"tiles\": {tiles}, \"spawns\": [[0,0],[14,12]], \"height\": 13, \"wid\\u0074h\": 15, \"schema\": \"{CUSTOM_MAP_SCHEMA}\"}}\n"
        );
        assert_eq!(
            CustomMap::parse_normalized(reordered.as_bytes(), &palette)
                .unwrap()
                .content_pin(),
            pin
        );

        let spawn_order = payload(15, 13, &tiles, "[[14,12],[0,0]]");
        let tile_order = payload(
            15,
            13,
            &tiles.replacen("1,2,1", "1,1,2", 1),
            "[[0,0],[14,12]]",
        );
        for changed in [&spawn_order, &tile_order] {
            let changed = CustomMap::parse_normalized(changed, &palette).unwrap();
            assert_eq!(changed.verify_pin(pin), Err(CustomMapError::PinMismatch));
        }
        let mut changed_palette = palette;
        changed_palette.insert(65535, false); // Unused tile still changes the pin.
        let changed = CustomMap::parse_normalized(&bytes, &changed_palette).unwrap();
        assert_eq!(changed.verify_pin(pin), Err(CustomMapError::PinMismatch));
    }

    #[test]
    fn sha256_matches_standard_vectors_including_two_block_padding() {
        assert_eq!(
            sha256_hex(b""),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
        assert_eq!(
            sha256_hex(b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq"),
            "248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1"
        );
    }

    #[test]
    fn valid_map_cannot_encode_or_dispatch_either_wire_route() {
        let bytes = payload(15, 13, &open_tiles(195), "[[0,0],[14,12]]");
        let map = CustomMap::parse_normalized(&bytes, &palette()).unwrap();
        for route in [CustomMapWireRoute::Upload, CustomMapWireRoute::Select] {
            assert_eq!(route.status(), CustomMapWireStatus::Unmeasured);
            assert_eq!(
                route.validate_dispatch(),
                Err(CustomMapError::UnmeasuredWireContract)
            );
            assert_eq!(
                route.encode(&map),
                Err(CustomMapError::UnmeasuredWireContract)
            );
        }
    }
}
