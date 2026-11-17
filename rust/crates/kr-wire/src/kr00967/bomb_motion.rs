//! Measured 119/162 body layouts, not action admission or server authority.
//! Native source: E2E910 / E5EFC0. The 119 tail remains opaque. Service must
//! validate membership, round, ability, bomb, geometry and delivery separately.
use crate::{Width, read_uint, write_uint};
use std::io;

/// CP325 native4/4: one current bomb, variant1. This is a binding/position
/// update, not an explosion or119 ACK. Ordering is a server policy choice.
pub fn rebind158(bomb_id: u16) -> io::Result<Vec<u8>> {
    if bomb_id == 0 {
        return Err(io::Error::other("bomb rebind158 ID zero"));
    }
    let mut body = write_uint(1, Width::U8).map_err(io::Error::other)?;
    body.extend(write_uint(u64::from(bomb_id), Width::U16).map_err(io::Error::other)?);
    body.extend(write_uint(1, Width::U8).map_err(io::Error::other)?);
    Ok(body)
}

/// Nine-byte native 162 record. `selector` chooses native action branches;
/// decoding a value does not declare that branch supported by the service.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BombMotion162 {
    pub context: u8,
    pub slot: u8,
    pub selector: u8,
    pub bomb_id: u16,
    pub source_row: u8,
    pub source_column: u8,
    pub target_row: u8,
    pub target_column: u8,
}

impl BombMotion162 {
    pub fn decode(body: &[u8]) -> io::Result<Self> {
        if body.len() != 9 {
            return Err(io::Error::other("bomb motion162 body size"));
        }
        let byte = |at| read_uint(&body[at..at + 1], Width::U8).map_err(io::Error::other);
        Ok(Self {
            context: byte(0)? as u8,
            slot: byte(1)? as u8,
            selector: byte(2)? as u8,
            bomb_id: read_uint(&body[3..5], Width::U16).map_err(io::Error::other)? as u16,
            source_row: byte(5)? as u8,
            source_column: byte(6)? as u8,
            target_row: byte(7)? as u8,
            target_column: byte(8)? as u8,
        })
    }

    /// Serialize fields only. This does not approve a captured 119 request.
    pub fn encode(self) -> io::Result<Vec<u8>> {
        let mut body = Vec::with_capacity(9);
        for (value, width) in [
            (u64::from(self.context), Width::U8),
            (u64::from(self.slot), Width::U8),
            (u64::from(self.selector), Width::U8),
            (u64::from(self.bomb_id), Width::U16),
            (u64::from(self.source_row), Width::U8),
            (u64::from(self.source_column), Width::U8),
            (u64::from(self.target_row), Width::U8),
            (u64::from(self.target_column), Width::U8),
        ] {
            body.extend(write_uint(value, width).map_err(io::Error::other)?);
        }
        Ok(body)
    }
}

/// Thirteen-byte native 119 request. No epoch or key is invented for the
/// measured layout. Unknown tail values must not become durations or flags
/// without their consumer and runtime evidence.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BombAction119 {
    pub motion: BombMotion162,
    pub opaque_tail: [u16; 2],
}

impl BombAction119 {
    pub fn decode(body: &[u8]) -> io::Result<Self> {
        if body.len() != 13 {
            return Err(io::Error::other("bomb action119 body size"));
        }
        Ok(Self {
            motion: BombMotion162::decode(&body[..9])?,
            opaque_tail: [
                read_uint(&body[9..11], Width::U16).map_err(io::Error::other)? as u16,
                read_uint(&body[11..13], Width::U16).map_err(io::Error::other)? as u16,
            ],
        })
    }
}
