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
            || number(&packets["map"].payload[4..8], Width::U32)? != 0
            || start_stream[32 + name.len()..36 + name.len()] != [0; 4]
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
            if inner.get(..2) != Some(&[1, 0]) {
                return Err(error());
            }
            let mut at = 7;
            let mut names = Vec::new();
            for _ in 0..2 {
                for _ in 0..2 {
                    let length = inner.get(at..at + 4).ok_or_else(error)?;
                    let length = u32::from_le_bytes(length.try_into().expect("bounded")) as usize;
                    if length > 64 {
                        return Err(error());
                    }
                    let value = inner.get(at + 4..at + 4 + length).ok_or_else(error)?;
                    names.push(value);
                    at += 4 + length;
                }
                at += 2;
            }
            at += 7;
            if at + 7 != inner.len()
                || names[1] != self.name.as_bytes()
                || u16::from_le_bytes(inner[at..at + 2].try_into().expect("bounded")) != code
            {
                return Err(error());
            }
        } else if crate::sha256_hex(&table)
            != "9b30734b4feccf44fba7432f65137d7125f13e2580aea761749cca62543d98eb"
            || (code, self.name.as_str()) != (278, "ExVillage01")
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
