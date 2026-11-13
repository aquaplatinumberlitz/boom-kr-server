//! Pack-exported map identity and bounded packet consistency before bind.
//! This validates input agreement; a profile is not native gameplay acceptance.
use crate::{ActorProfile, FixturePin, Reply, room::PlayerId};
use kr_wire::{Width, kr00967 as wire, read_uint};
use serde::Deserialize;
use std::{collections::HashMap, io};

#[derive(Deserialize)]
pub(crate) struct Profile {
    name: String,
    map_code: u16,
    width: u8,
    height: u8,
    spawns: Vec<[u8; 2]>,
    scripted: bool,
    payload_sha256: String,
}
fn error() -> io::Error {
    io::Error::other("inconsistent or unsupported pack map profile")
}
fn number(bytes: &[u8], width: Width) -> io::Result<u64> {
    read_uint(bytes, width).map_err(io::Error::other)
}
fn stream(body: &[u8]) -> io::Result<(Vec<u8>, usize)> {
    let size = number(body.get(..4).ok_or_else(error)?, Width::U32)? as usize;
    if !(6..=4096).contains(&size) {
        return Err(error());
    }
    let end = 4 + size;
    let decoded = wire::decode_bytes(body.get(4..end).ok_or_else(error)?)?;
    if decoded[0] != 24
        || u32::from_le_bytes(decoded[1..5].try_into().expect("bounded")) as usize != size - 5
    {
        return Err(error());
    }
    Ok((decoded, end))
}
fn adler_zero(bytes: &[u8]) -> u32 {
    let (mut a, mut b) = (0_u32, 0_u32);
    for byte in bytes {
        a = (a + u32::from(*byte)) % 65521;
        b = (b + a) % 65521;
    }
    (b << 16) | a
}
impl Profile {
    pub(crate) fn load(pin: &FixturePin) -> io::Result<Self> {
        let value: Self = serde_json::from_slice(&pin.load()?).map_err(io::Error::other)?;
        if value.scripted
            || (value.width, value.height) != (15, 13)
            || !(2..=8).contains(&value.spawns.len())
            || value
                .spawns
                .iter()
                .any(|[x, y]| *x >= value.width || *y >= value.height)
            || !(1..=64).contains(&value.name.len())
            || !value
                .name
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'_')
            || value.payload_sha256.len() != 64
            || !value.payload_sha256.bytes().all(|b| b.is_ascii_hexdigit())
        {
            return Err(error());
        }
        Ok(value)
    }
    pub(crate) fn validate(
        &self,
        expected_count: usize,
        packets: &HashMap<String, Reply>,
        actors: &HashMap<PlayerId, ActorProfile>,
    ) -> io::Result<()> {
        self.validate_selected(expected_count, packets, actors, 0, 1)
    }
    pub(crate) fn validate_selected(
        &self,
        expected_count: usize,
        packets: &HashMap<String, Reply>,
        actors: &HashMap<PlayerId, ActorProfile>,
        selected: usize,
        count: usize,
    ) -> io::Result<()> {
        let start = &packets["start"].payload;
        let (code, name) = wire::start121_map(start)?;
        if code != self.map_code
            || name != self.name.as_bytes()
            || number(&packets["map"].payload[2..4], Width::U16)? != u64::from(code)
            || number(&packets["play"].payload[5..7], Width::U16)? != u64::from(code)
        {
            return Err(error());
        }
        // Native selects coordinates by this exported spawn ordinal; do not
        // replace pack coordinates with fixture coordinates from another map.
        let (start_stream, _) = stream(&start[1..])?;
        let rows = 43 + name.len();
        if usize::from(start_stream[rows - 5]) != expected_count
            || number(&packets["map"].payload[4..8], Width::U32)? != selected as u64
            || start_stream[32 + name.len()..36 + name.len()] != (selected as u32).to_le_bytes()
        {
            return Err(error());
        }
        if start_stream[rows..]
            .as_chunks::<26>()
            .0
            .iter()
            .any(|row| usize::from(row[2]) >= self.spawns.len())
        {
            return Err(error());
        }
        let (table, end) = stream(&packets["table"].payload)?;
        if end != packets["table"].payload.len() {
            return Err(error());
        }
        if table[5] == 0 {
            let inner = &table[6..];
            let rows = table_rows(inner)?;
            if rows.len() != count
                || rows.get(selected).is_none_or(|row| {
                    row.code != code
                        || row.name != self.name.as_bytes()
                        || row.id != selected as u32
                })
            {
                return Err(error());
            }
        } else if crate::sha256_hex(&table)
            != "9b30734b4feccf44fba7432f65137d7125f13e2580aea761749cca62543d98eb"
            || (code, self.name.as_str()) != (278, "ExVillage01")
            || selected != 0
            || count != 1
        {
            // Preserve the independently measured compressed baseline exactly.
            // New map tables use the already measured raw-selector0 schema;
            // no generic compressed decoder or inferred checksum is introduced.
            return Err(error());
        }
        let checksum = adler_zero(&table[5..]);
        for actor in actors.values() {
            let ready = &actor.packets["ready"].payload;
            let (_, end) = stream(ready)?;
            if ready.len() != end + 13
                || number(&ready[end..end + 4], Width::U32)? != u64::from(checksum)
            {
                return Err(error());
            }
        }
        Ok(())
    }
}

struct TableRow<'a> {
    id: u32,
    code: u16,
    name: &'a [u8],
}
fn table_rows(inner: &[u8]) -> io::Result<Vec<TableRow<'_>>> {
    let count = u16::from_le_bytes(
        inner
            .get(..2)
            .ok_or_else(error)?
            .try_into()
            .expect("bounded"),
    ) as usize;
    if !(1..=16).contains(&count) {
        return Err(error());
    }
    let mut at = 2;
    let mut rows = Vec::new();
    for _ in 0..count {
        let id = u32::from_le_bytes(
            inner
                .get(at..at + 4)
                .ok_or_else(error)?
                .try_into()
                .expect("bounded"),
        );
        at += 5;
        let mut names = Vec::new();
        for _ in 0..2 {
            for _ in 0..2 {
                let n = u32::from_le_bytes(
                    inner
                        .get(at..at + 4)
                        .ok_or_else(error)?
                        .try_into()
                        .expect("bounded"),
                ) as usize;
                if n > 64 {
                    return Err(error());
                }
                let value = inner.get(at + 4..at + 4 + n).ok_or_else(error)?;
                names.push(value);
                at += 4 + n;
            }
            at += 2;
        }
        at += 7;
        let code = u16::from_le_bytes(
            inner
                .get(at..at + 2)
                .ok_or_else(error)?
                .try_into()
                .expect("bounded"),
        );
        at += 7;
        rows.push(TableRow {
            id,
            code,
            name: names[1],
        });
    }
    if at != inner.len() {
        return Err(error());
    }
    Ok(rows)
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ChoiceConfig {
    pub map_profile: FixturePin,
    pub packets: HashMap<String, crate::LobbyPacketPin>,
    #[serde(default)]
    pub terrain: Option<crate::terrain::Config>,
}
pub(crate) struct Choice {
    pub code: u16,
    pub index: u32,
    pub bodies: HashMap<crate::room_packets::Template, Vec<u8>>,
    pub floor: crate::pickups::Catalog,
    pub terrain: Option<crate::terrain::Terrain>,
}
pub(crate) struct Catalog {
    pub choices: Vec<Choice>,
}
impl Catalog {
    pub(crate) fn load(
        configs: &[ChoiceConfig],
        count: usize,
        root: &HashMap<String, Reply>,
        actors: &HashMap<PlayerId, ActorProfile>,
        needle: bool,
        basic_items: bool,
    ) -> io::Result<Self> {
        use crate::room_packets::Template;
        if !(2..=16).contains(&configs.len()) {
            return Err(error());
        }
        let mut choices = Vec::new();
        let mut codes = std::collections::HashSet::new();
        for (index, cfg) in configs.iter().enumerate() {
            if cfg.terrain.is_some() && !basic_items {
                return Err(io::Error::other("map terrain requires basic-item policy"));
            }
            if cfg.packets.len() != 4 {
                return Err(error());
            }
            let profile = Profile::load(&cfg.map_profile)?;
            let mut packets = root.clone();
            let mut bodies = HashMap::new();
            for (name, key) in [
                ("map", Template::Map117),
                ("start", Template::Start121),
                ("play", Template::Play196),
                ("floor", Template::Floor582),
            ] {
                let pin = cfg.packets.get(name).ok_or_else(error)?;
                let raw = pin.fixture.load()?;
                let frame = kr_wire::decode(&raw, pin.sequence.ok_or_else(error)?)
                    .map_err(io::Error::other)?;
                if pin.opcode != key.opcode()
                    || frame.opcode != key.opcode()
                    || !wire::header_matches(&raw[..37])
                {
                    return Err(error());
                }
                bodies.insert(key, frame.payload.clone());
                packets.insert(
                    name.into(),
                    Reply {
                        header: raw[..37].try_into().expect("decoded"),
                        opcode: frame.opcode,
                        payload: frame.payload,
                    },
                );
            }
            profile.validate_selected(count, &packets, actors, index, configs.len())?;
            if !codes.insert(profile.map_code) {
                return Err(error());
            }
            let floor = crate::pickups::Catalog::from_floor(&packets["floor"].payload)?;
            floor.validate_needle(needle)?;
            let terrain = cfg
                .terrain
                .as_ref()
                .map(|terrain| {
                    terrain.validate_for_map(
                        &cfg.packets["map"].fixture.load()?,
                        &cfg.packets["start"].fixture.load()?,
                        &profile.payload_sha256,
                    )
                })
                .transpose()?;
            choices.push(Choice {
                code: profile.map_code,
                index: index as u32,
                bodies,
                floor,
                terrain,
            });
        }
        for (name, key) in [
            ("map", Template::Map117),
            ("start", Template::Start121),
            ("play", Template::Play196),
            ("floor", Template::Floor582),
        ] {
            if root[name].payload != choices[0].bodies[&key] {
                return Err(error());
            }
        }
        Ok(Self { choices })
    }
    pub(crate) fn select(&self, code: u16, index: u32) -> io::Result<usize> {
        self.choices
            .iter()
            .position(|c| c.code == code && c.index == index)
            .ok_or_else(error)
    }
}
