//! Captured KR00967 mode2/selector3 framing profile only.
//! R1d foundation; limits: docs/PLAN.md#rust-r1 and existing codec evidence.
//! This measured profile does not establish other modes or original-server policy.
use crate::{CodecError, HEADER_LEN};

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
