//! Captured KR00967 mode2/selector3 framing profile only.
//! R1d foundation; limits: docs/PLAN.md#rust-r1 and existing codec evidence.
//! This measured profile does not establish other modes or original-server policy.
use crate::{CodecError, HEADER_LEN, Width, read_uint, write_uint};
use std::io;
mod battle;
mod player;
mod room;
mod room_selection;
pub use battle::*;
pub use player::PlayerTemplate;
pub use room::*;
pub use room_selection::*;

/// Captured header with opcode/length fields zeroed; encode recomputes both.
pub const HEADER: [u8; HEADER_LEN] = [
    0x69, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xff, 0xff, 0xff, 0xff, 0x00, 0x00, 0x00,
    0x00, 0x02, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x01, 0x00, 0x00, 0x00,
];

pub fn header_matches(captured: &[u8]) -> bool {
    let Ok(mut normalized) = <[u8; HEADER_LEN]>::try_from(captured) else {
        return false;
    };
    normalized[5..9].fill(0);
    normalized[13..17].fill(0);
    normalized == HEADER
}

pub fn encode(opcode: u32, payload: &[u8], sequence: u32) -> Result<Vec<u8>, CodecError> {
    crate::encode(&HEADER, opcode, payload, sequence)
}

/// Measured107 body; transport/membership authority belongs to the service.
pub fn removal107(slot: u8, identity: u32) -> Result<Vec<u8>, CodecError> {
    let mut body = crate::write_uint(u64::from(slot), crate::Width::U8)?;
    body.extend(crate::write_uint(u64::from(identity), crate::Width::U32)?);
    body.extend(crate::write_uint(0, crate::Width::U8)?);
    Ok(body)
}

/// Measured110 body; the service selects the existing projected player state.
pub fn slot110(slot: u8, state: u32) -> Result<Vec<u8>, CodecError> {
    let mut body = crate::write_uint(u64::from(slot), crate::Width::U8)?;
    body.extend(crate::write_uint(u64::from(state), crate::Width::U32)?);
    body.extend(crate::write_uint(0, crate::Width::U8)?);
    Ok(body)
}

fn uint(b: &[u8], w: Width) -> io::Result<u64> {
    read_uint(b, w).map_err(io::Error::other)
}
fn put(v: u64, w: Width) -> io::Result<Vec<u8>> {
    write_uint(v, w).map_err(io::Error::other)
}

pub fn encode_bytes(data: &[u8]) -> io::Result<Vec<u8>> {
    let mut at = 0;
    let mut out = Vec::new();
    while at < data.len() {
        let (n, w) = if data.len() - at >= 4 {
            (4, Width::U32)
        } else if data.len() - at >= 2 {
            (2, Width::U16)
        } else {
            (1, Width::U8)
        };
        let mut raw = [0; 8];
        raw[..n].copy_from_slice(&data[at..at + n]);
        out.extend(put(u64::from_le_bytes(raw), w)?);
        at += n;
    }
    Ok(out)
}

pub fn decode_bytes(mut encoded: &[u8]) -> io::Result<Vec<u8>> {
    let mut out = Vec::with_capacity(encoded.len());
    while !encoded.is_empty() {
        let (n, w) = if encoded.len() >= 4 {
            (4, Width::U32)
        } else if encoded.len() >= 2 {
            (2, Width::U16)
        } else {
            (1, Width::U8)
        };
        let value = read_uint(&encoded[..n], w).map_err(io::Error::other)?;
        out.extend_from_slice(&value.to_le_bytes()[..n]);
        encoded = &encoded[n..];
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn donor_opcode_and_length_are_not_part_of_the_pinned_header_profile() {
        let mut captured = HEADER;
        captured[5..9].copy_from_slice(&175u32.to_le_bytes());
        captured[13..17].copy_from_slice(&1082u32.to_le_bytes());
        assert!(header_matches(&captured));
        assert!(!header_matches(&captured[..36]));
        for at in (0..HEADER_LEN).filter(|at| !(5..9).contains(at) && !(13..17).contains(at)) {
            let mut unsupported = captured;
            unsupported[at] ^= 1;
            assert!(
                !header_matches(&unsupported),
                "unexpectedly normalized byte {at}"
            );
        }
    }
}
