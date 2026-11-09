//! KR00967 measured mode2/selector3 profile, not a complete protocol or auth.
//! Source/limits: docs/PROTOCOL.md#login-retry-codec, CP057/180/181.

#![forbid(unsafe_code)]

use std::{error::Error, fmt};

pub const HEADER_LEN: usize = 37;
pub const FRAME_OVERHEAD: usize = HEADER_LEN + 1;
pub const MAX_FRAME_LEN: usize = 65_536; // Diagnostic design bound, not KR maximum.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CodecError {
    InvalidLength,
    FrameTooLarge,
    Checksum,
    BufferFull,
    TruncatedEof,
    FieldOutOfRange,
}
impl fmt::Display for CodecError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}
impl Error for CodecError {}

pub fn crc8(sequence: u32, payload: &[u8]) -> u8 {
    let mut value = sequence as u8;
    for &byte in payload {
        value ^= byte;
        for _ in 0..8 {
            value = (value << 1) ^ if value & 0x80 != 0 { 7 } else { 0 };
        }
    }
    value
}

pub fn crypt(payload: &[u8], sequence: u32, decrypt: bool) -> Vec<u8> {
    let mut out = Vec::with_capacity(payload.len());
    let mut key = sequence ^ 0x5335_1d9c;
    let mut previous = 0u32;
    let (words, tail) = payload.as_chunks::<4>();
    for (i, bytes) in words.iter().enumerate() {
        if i != 0 {
            key = key.wrapping_sub(0x63ca_ace3);
        }
        let word = u32::from_le_bytes(*bytes);
        let converted = word ^ key ^ previous;
        out.extend_from_slice(&converted.to_le_bytes());
        previous = if decrypt { converted } else { word };
    }
    let tail_key = if payload.len() >= 4 { previous } else { key };
    for (i, &byte) in tail.iter().enumerate() {
        out.push(byte ^ (tail_key >> (i * 8)) as u8);
    }
    out
}

pub fn encode(
    header: &[u8; HEADER_LEN],
    opcode: u32,
    payload: &[u8],
    sequence: u32,
) -> Result<Vec<u8>, CodecError> {
    if payload.len() > MAX_FRAME_LEN - FRAME_OVERHEAD {
        return Err(CodecError::FrameTooLarge);
    }
    let mut out = Vec::with_capacity(payload.len() + FRAME_OVERHEAD);
    out.extend_from_slice(header);
    out[5..9].copy_from_slice(&opcode.to_le_bytes());
    out[13..17].copy_from_slice(&(payload.len() as u32).to_le_bytes());
    out.extend_from_slice(&crypt(payload, sequence, false));
    out.push(crc8(sequence, payload));
    Ok(out)
}

#[derive(Debug, PartialEq, Eq)]
pub struct Packet {
    pub opcode: u32,
    pub sequence: u32,
    pub payload: Vec<u8>,
}

pub fn decode(frame: &[u8], sequence: u32) -> Result<Packet, CodecError> {
    if frame.len() < FRAME_OVERHEAD {
        return Err(CodecError::InvalidLength);
    }
    let body_len = u32::from_le_bytes(frame[13..17].try_into().expect("header checked")) as usize;
    if body_len > MAX_FRAME_LEN - FRAME_OVERHEAD {
        return Err(CodecError::FrameTooLarge);
    }
    if frame.len() != body_len + FRAME_OVERHEAD {
        return Err(CodecError::InvalidLength);
    }
    let payload = crypt(&frame[HEADER_LEN..frame.len() - 1], sequence, true);
    if crc8(sequence, &payload) != frame[frame.len() - 1] {
        return Err(CodecError::Checksum);
    }
    Ok(Packet {
        opcode: u32::from_le_bytes(frame[5..9].try_into().expect("header checked")),
        sequence,
        payload,
    })
}

/// Each accepted TCP connection owns a fresh receiver; failure never advances seq.
pub struct Receiver {
    bytes: Vec<u8>,
    sequence: u32,
}
impl Receiver {
    pub fn new(sequence: u32) -> Self {
        Self {
            bytes: Vec::with_capacity(MAX_FRAME_LEN),
            sequence,
        }
    }
    pub fn sequence(&self) -> u32 {
        self.sequence
    }
    pub fn remaining_capacity(&self) -> usize {
        MAX_FRAME_LEN - self.bytes.len()
    }
    pub fn push(&mut self, input: &[u8]) -> Result<(), CodecError> {
        if input.len() > self.remaining_capacity() {
            return Err(CodecError::BufferFull);
        }
        self.bytes.extend_from_slice(input);
        Ok(())
    }
    pub fn next_packet(&mut self) -> Result<Option<Packet>, CodecError> {
        if self.bytes.len() < FRAME_OVERHEAD {
            return Ok(None);
        }
        let body_len =
            u32::from_le_bytes(self.bytes[13..17].try_into().expect("header checked")) as usize;
        if body_len > MAX_FRAME_LEN - FRAME_OVERHEAD {
            return Err(CodecError::FrameTooLarge);
        }
        let total = body_len + FRAME_OVERHEAD;
        if self.bytes.len() < total {
            return Ok(None);
        }
        let packet = decode(&self.bytes[..total], self.sequence)?;
        self.bytes.drain(..total);
        self.sequence = self.sequence.wrapping_add(3);
        Ok(Some(packet))
    }
    pub fn finish(&self) -> Result<(), CodecError> {
        if self.bytes.is_empty() {
            Ok(())
        } else {
            Err(CodecError::TruncatedEof)
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Width {
    U8,
    U16,
    U32,
    U64,
}
impl Width {
    fn properties(self) -> (usize, u64) {
        match self {
            Self::U8 => (1, 0x5a),
            Self::U16 => (2, 0xa569),
            Self::U32 => (4, 0x96ca_5395),
            Self::U64 => (8, 0x71b1_8589),
        }
    }
}
pub fn write_uint(value: u64, width: Width) -> Result<Vec<u8>, CodecError> {
    let (size, mask) = width.properties();
    if size < 8 && value >= (1u64 << (8 * size)) {
        return Err(CodecError::FieldOutOfRange);
    }
    Ok((value ^ mask).to_be_bytes()[8 - size..].to_vec())
}
pub fn read_uint(bytes: &[u8], width: Width) -> Result<u64, CodecError> {
    let (size, mask) = width.properties();
    if bytes.len() != size {
        return Err(CodecError::InvalidLength);
    }
    let mut data = [0; 8];
    data[8 - size..].copy_from_slice(bytes);
    Ok(u64::from_be_bytes(data) ^ mask)
}
