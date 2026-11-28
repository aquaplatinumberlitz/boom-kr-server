//! KR00967 F3 PvE body candidates (`kr_static/candidate`, pending live Wine
//! capture). See `docs/protocol/f3-pve-monaction-wire-re.md` and
//! `docs/evidence/content/f3-pve-monaction-wire-re.json` (IDA DB SHA-256
//! `6be15a1fccea0837ee95c50893345e9f13dc217b0fa570975b5c284f0d547fd1`).
//! These codecs describe packet bodies in the KR mode2 profile. They do not
//! determine ownership, server acceptance, or relay behavior.

#![deny(unsafe_code)]

use super::{decode_bytes, encode_bytes};
use crate::{CodecError, FRAME_OVERHEAD, MAX_FRAME_LEN, Width, read_uint, write_uint};
use std::fmt;

/// Evidence status for the F3 PvE candidates.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum F3WireStatus {
    StaticCandidatePendingCapture,
}

/// Static candidate provenance, pending live Wine capture in F3 PvE.
/// Hex-Rays cases are indices in `CGameStage::OnPacket` (`0xE5D3A0`):
/// the dispatch at `0xE5DE9C` is indexed by `opcode - 20`.
/// Check the assembly and live route before wiring any candidate into dispatch.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct F3JumpTableProvenance {
    pub on_packet_va: u32,
    pub jump_table_va: u32,
    pub opcode_index_subtrahend: u32,
    /// Legacy Hex-Rays labels. Consult the byte table for each actual target.
    pub hex_rays_cases: [u32; 5],
    pub status: F3WireStatus,
}

/// One recorded entry of the client receive dispatcher at `0xE5D3A0`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct F3JumpTableEntry {
    pub case: u32,
    pub wire_opcode: u32,
    /// Byte selected at `0xE5DE9C + case`.
    pub table_selector: u8,
    /// Branch address selected from the address table at `0xE5DDC4`.
    pub dispatch_va: u32,
    /// Named callee in this branch, if one is identified.
    pub handler_va: Option<u32>,
}

impl F3JumpTableEntry {
    const fn new(case: u32, table_selector: u8, dispatch_va: u32, handler_va: Option<u32>) -> Self {
        Self {
            case,
            wire_opcode: case + F3JumpTableProvenance::C_GAME_STAGE.opcode_index_subtrahend,
            table_selector,
            dispatch_va,
            handler_va,
        }
    }

    const fn handler(case: u32, table_selector: u8, dispatch_va: u32, handler_va: u32) -> Self {
        Self::new(case, table_selector, dispatch_va, Some(handler_va))
    }

    const fn fallback(case: u32) -> Self {
        Self::new(case, 53, 0xE5DCA9, None)
    }
}

/// Non-default entries and selected fallback entries in ascending wire opcode
/// order. Cross-checked against the unpacked KR 00967 image (SHA-256
/// `cf72da112308d9f1ca256df8089594c5f787875310933073bf6bd5ba6f70e63b`).
pub static C_GAME_STAGE_JUMP_ENTRIES: &[F3JumpTableEntry] = &[
    F3JumpTableEntry::new(0, 0, 0xE5D8E3, None),
    F3JumpTableEntry::handler(5, 1, 0xE5D8BB, 0xE65A10),
    // The older 9/80 anchors 0xE6AA90 and 0xE6D730 are called in other
    // branches of OnPacket; these two byte-table indices select the fallback.
    F3JumpTableEntry::fallback(9),
    F3JumpTableEntry::handler(64, 2, 0xE5DC34, 0xE6E1A0),
    F3JumpTableEntry::fallback(80),
    F3JumpTableEntry::handler(90, 3, 0xE5D498, 0xE5E2C0),
    F3JumpTableEntry::handler(91, 4, 0xE5DBFE, 0xE6DF60),
    F3JumpTableEntry::new(103, 5, 0xE5D47A, None), // vtable +0x2D4
    F3JumpTableEntry::handler(112, 6, 0xE5D691, 0xE610C0),
    F3JumpTableEntry::handler(119, 7, 0xE5D6D9, 0xE662B0),
    F3JumpTableEntry::handler(120, 8, 0xE5D6FD, 0xE66320),
    F3JumpTableEntry::handler(121, 9, 0xE5D70F, 0xE66360),
    F3JumpTableEntry::handler(122, 10, 0xE5D6EB, 0xE66380),
    F3JumpTableEntry::handler(132, 11, 0xE5D4BC, 0xE5E640),
    F3JumpTableEntry::handler(133, 12, 0xE5D4CE, 0xE5E6D0),
    F3JumpTableEntry::handler(134, 13, 0xE5D4F2, 0xE5E860),
    F3JumpTableEntry::handler(135, 14, 0xE5D777, 0xE67640),
    F3JumpTableEntry::handler(136, 15, 0xE5D789, 0xE67770),
    F3JumpTableEntry::handler(137, 16, 0xE5D4AA, 0xE5E600),
    F3JumpTableEntry::handler(138, 17, 0xE5D55E, 0xE5F450),
    F3JumpTableEntry::handler(139, 18, 0xE5D516, 0xE5E8F0),
    F3JumpTableEntry::handler(140, 19, 0xE5D528, 0xE5EEC0),
    F3JumpTableEntry::handler(142, 20, 0xE5D53A, 0xE5EFC0),
    F3JumpTableEntry::handler(144, 21, 0xE5D54C, 0xE5F2C0),
    F3JumpTableEntry::handler(145, 22, 0xE5D570, 0xE63260),
    F3JumpTableEntry::handler(146, 23, 0xE5D584, 0xE875E0),
    F3JumpTableEntry::handler(147, 24, 0xE5D592, 0xE63E50),
    F3JumpTableEntry::handler(148, 25, 0xE5D5A4, 0xE63F10),
    F3JumpTableEntry::handler(150, 26, 0xE5D5B6, 0xE85B60),
    F3JumpTableEntry::handler(151, 27, 0xE5D5DA, 0xE64310),
    F3JumpTableEntry::handler(152, 28, 0xE5D5EC, 0xE64570),
    F3JumpTableEntry::handler(153, 29, 0xE5D5FE, 0xE646E0),
    F3JumpTableEntry::handler(155, 30, 0xE5D67F, 0xE5F610),
    F3JumpTableEntry::handler(156, 31, 0xE5D807, 0xE77D20),
    F3JumpTableEntry::handler(157, 32, 0xE5DACF, 0xE6DD60),
    F3JumpTableEntry::handler(158, 33, 0xE5D819, 0xE77DD0),
    F3JumpTableEntry::handler(160, 34, 0xE5D610, 0xE72490), // Also vtable +0x2D8.
    F3JumpTableEntry::handler(161, 35, 0xE5D649, 0xE652F0),
    F3JumpTableEntry::handler(163, 36, 0xE5D65B, 0xE65350),
    F3JumpTableEntry::handler(164, 37, 0xE5D66D, 0xE654F0),
    F3JumpTableEntry::handler(165, 38, 0xE5D765, 0xE67470),
    F3JumpTableEntry::handler(171, 39, 0xE5D5C8, 0xE64070),
    F3JumpTableEntry::handler(176, 40, 0xE5D741, 0xE73760),
    F3JumpTableEntry::handler(177, 41, 0xE5D79B, 0xE751F0),
    F3JumpTableEntry::handler(178, 42, 0xE5D7AD, 0xE74410),
    F3JumpTableEntry::handler(179, 43, 0xE5D7F5, 0xE72EC0),
    F3JumpTableEntry::handler(180, 44, 0xE5D82B, 0xE683C0),
    F3JumpTableEntry::handler(181, 45, 0xE5D83D, 0xE678A0),
    // Opcode 204 has a separate reader at 0x115CB10.
    F3JumpTableEntry::fallback(184),
    F3JumpTableEntry::handler(189, 46, 0xE5D861, 0xE68280),
    F3JumpTableEntry::handler(190, 47, 0xE5D84F, 0xE67A60),
    F3JumpTableEntry::handler(195, 48, 0xE5D8E8, 0xE7F500),
    F3JumpTableEntry::handler(200, 49, 0xE5D6A3, 0xE65D40),
    F3JumpTableEntry::handler(201, 50, 0xE5D6C7, 0xE65F20),
    F3JumpTableEntry::handler(202, 51, 0xE5D6B5, 0xE65E40),
    F3JumpTableEntry::handler(254, 52, 0xE5DAAB, 0xD73100),
];

/// Looks up a recorded client receive dispatch entry by wire opcode.
pub fn f3_stage_jump_entry(wire_opcode: u32) -> Option<&'static F3JumpTableEntry> {
    let index = C_GAME_STAGE_JUMP_ENTRIES
        .binary_search_by_key(&wire_opcode, |entry| entry.wire_opcode)
        .ok()?;
    C_GAME_STAGE_JUMP_ENTRIES.get(index)
}

impl F3JumpTableProvenance {
    /// Static candidate dispatch evidence, pending live Wine capture.
    pub const C_GAME_STAGE: Self = Self {
        on_packet_va: 0xE5D3A0,
        jump_table_va: 0xE5DE9C,
        opcode_index_subtrahend: 20,
        hex_rays_cases: [9, 80, 135, 136, 177],
        status: F3WireStatus::StaticCandidatePendingCapture,
    };
}

/// KR 00967 static evidence from CP405 and `client-data-tables.md#monaction-runtime`.
/// The client runs MonAction steps for locally owned objects. The server's choice
/// of owner slot and forwarding of step reports still need a two-client capture.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct F3MonActionProvenance {
    pub owner_gate_va: u32,
    pub delegation_opcode: u32,
    pub step_report_opcodes: [u32; 2],
    pub status: F3WireStatus,
}

impl F3MonActionProvenance {
    pub const CP405_STATIC: Self = Self {
        owner_gate_va: 0xB91F60,
        delegation_opcode: 204,
        step_report_opcodes: [132, 180],
        status: F3WireStatus::StaticCandidatePendingCapture,
    };
}

/// Identifies the timer that overflowed while converting a candidate body.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum F3TimerField {
    First,
    Second,
}

/// Validation errors for the static candidates pending live Wine capture.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum F3WireError {
    TruncatedEof,
    TrailingBytes,
    InvalidHostFlag(u8),
    InvalidAttackState(u8),
    UnsupportedTimedEventFlag(u8),
    TimerOverflow(F3TimerField),
    InvalidMonActionOpcode(u32),
    ZeroMonActionObjectId,
    MonActionOwnerSlotOutOfRange(u8),
    MonActionStepIndexOutOfRange(u16),
    MonActionTemplateIdOutOfRange(u16),
    UnsupportedMonActionKind { opcode: u32, kind: u16 },
    InvalidMonActionSelector { opcode: u32, selector: u8 },
    Codec(CodecError),
}

impl fmt::Display for F3WireError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TruncatedEof => f.write_str("truncated F3 candidate body"),
            Self::TrailingBytes => f.write_str("trailing bytes in F3 candidate body"),
            Self::InvalidHostFlag(flag) => write!(f, "invalid F3 candidate host flag {flag}"),
            Self::InvalidAttackState(state) => {
                write!(f, "invalid F3 candidate attack state {state}")
            }
            Self::UnsupportedTimedEventFlag(flag) => {
                write!(f, "unsupported F3 candidate timed event flag {flag}")
            }
            Self::TimerOverflow(field) => {
                write!(f, "F3 candidate {field:?} timer overflows u32 milliseconds")
            }
            Self::InvalidMonActionOpcode(opcode) => {
                write!(f, "invalid MonAction candidate opcode {opcode}")
            }
            Self::ZeroMonActionObjectId => f.write_str("MonAction candidate object ID is zero"),
            Self::MonActionOwnerSlotOutOfRange(slot) => {
                write!(f, "MonAction candidate owner slot {slot} out of range")
            }
            Self::MonActionStepIndexOutOfRange(index) => {
                write!(f, "MonAction candidate step index {index} out of range")
            }
            Self::MonActionTemplateIdOutOfRange(id) => {
                write!(f, "MonAction candidate template ID {id} out of range")
            }
            Self::UnsupportedMonActionKind { opcode, kind } => {
                write!(f, "unsupported MonAction {opcode} candidate kind {kind}")
            }
            Self::InvalidMonActionSelector { opcode, selector } => {
                write!(
                    f,
                    "invalid MonAction {opcode} candidate selector {selector}"
                )
            }
            Self::Codec(error) => error.fmt(f),
        }
    }
}

impl std::error::Error for F3WireError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Codec(error) => Some(error),
            _ => None,
        }
    }
}

impl From<CodecError> for F3WireError {
    fn from(error: CodecError) -> Self {
        Self::Codec(error)
    }
}

/// Static candidate pending live Wine capture for S→C object ownership delegation.
/// Validation bounds the proposed slot; it does not determine who owns the object.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MonActionDelegationCandidate {
    pub opcode: u32,
    pub object_id: u16,
    pub owner_slot: u8,
}

impl MonActionDelegationCandidate {
    pub const OPCODE: u32 = F3MonActionProvenance::CP405_STATIC.delegation_opcode;
    pub const STATUS: F3WireStatus = F3MonActionProvenance::CP405_STATIC.status;

    /// Bound to the eight supported active player slots; CP405 does not measure
    /// which slot the server selects or what other slots opcode 204 can address.
    pub const OWNER_SLOT_COUNT: u8 = 8;

    pub fn new(opcode: u32, object_id: u16, owner_slot: u8) -> Result<Self, F3WireError> {
        let candidate = Self {
            opcode,
            object_id,
            owner_slot,
        };
        candidate.validate()?;
        Ok(candidate)
    }

    pub fn validate(self) -> Result<(), F3WireError> {
        if self.opcode != Self::OPCODE {
            return Err(F3WireError::InvalidMonActionOpcode(self.opcode));
        }
        if self.object_id == 0 {
            return Err(F3WireError::ZeroMonActionObjectId);
        }
        if self.owner_slot >= Self::OWNER_SLOT_COUNT {
            return Err(F3WireError::MonActionOwnerSlotOutOfRange(self.owner_slot));
        }
        Ok(())
    }

    /// Decodes the S→C 204 candidate body from the reader at `0x115CB10`.
    pub fn decode(body: &[u8]) -> Result<Self, F3WireError> {
        let body = exact_body::<3>(body)?;
        Self::new(
            Self::OPCODE,
            read_uint(&body[..2], Width::U16)? as u16,
            read_uint(&body[2..], Width::U8)? as u8,
        )
    }

    pub fn encode(self) -> Result<Vec<u8>, F3WireError> {
        self.validate()?;
        let mut body = write_uint(self.object_id.into(), Width::U16)?;
        body.extend(write_uint(self.owner_slot.into(), Width::U8)?);
        Ok(body)
    }
}

/// Static candidate pending live Wine capture for a C→S step report.
/// `template_id` is a zero-based catalog entry index and `step_index` is a
/// zero-based index within an action list. They are local descriptor metadata,
/// not observed wire fields. Validation checks global bounds only; it cannot
/// confirm that the object uses that template or that a particular list has the step.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MonActionStepReportCandidate {
    pub opcode: u32,
    pub object_id: u16,
    pub step_index: u16,
    pub template_id: u16,
}

impl MonActionStepReportCandidate {
    pub const STATUS: F3WireStatus = F3MonActionProvenance::CP405_STATIC.status;

    /// Maximum `action_lists[].steps` length in `docs/content/client-tables/mon-action.json`.
    pub const MAX_STEPS_PER_ACTION_LIST: u16 = 215;

    /// Number of `entries` in `docs/content/client-tables/mon-action.json`.
    pub const TEMPLATE_COUNT: u16 = 364;

    pub fn new(
        opcode: u32,
        object_id: u16,
        step_index: u16,
        template_id: u16,
    ) -> Result<Self, F3WireError> {
        let candidate = Self {
            opcode,
            object_id,
            step_index,
            template_id,
        };
        candidate.validate()?;
        Ok(candidate)
    }

    pub fn validate(self) -> Result<(), F3WireError> {
        if !F3MonActionProvenance::CP405_STATIC
            .step_report_opcodes
            .contains(&self.opcode)
        {
            return Err(F3WireError::InvalidMonActionOpcode(self.opcode));
        }
        if self.object_id == 0 {
            return Err(F3WireError::ZeroMonActionObjectId);
        }
        if self.step_index >= Self::MAX_STEPS_PER_ACTION_LIST {
            return Err(F3WireError::MonActionStepIndexOutOfRange(self.step_index));
        }
        if self.template_id >= Self::TEMPLATE_COUNT {
            return Err(F3WireError::MonActionTemplateIdOutOfRange(self.template_id));
        }
        Ok(())
    }
}

const MAX_BODY_LEN: usize = MAX_FRAME_LEN - FRAME_OVERHEAD;

struct MonActionReader<'a>(&'a [u8]);

impl<'a> MonActionReader<'a> {
    fn new(body: &'a [u8]) -> Result<Self, F3WireError> {
        if body.len() > MAX_BODY_LEN {
            return Err(CodecError::FrameTooLarge.into());
        }
        Ok(Self(body))
    }

    fn take(&mut self, len: usize) -> Result<&'a [u8], F3WireError> {
        let (field, rest) = self
            .0
            .split_at_checked(len)
            .ok_or(F3WireError::TruncatedEof)?;
        self.0 = rest;
        Ok(field)
    }

    fn byte(&mut self) -> Result<u8, F3WireError> {
        Ok(read_uint(self.take(1)?, Width::U8)? as u8)
    }

    fn word(&mut self) -> Result<u16, F3WireError> {
        Ok(read_uint(self.take(2)?, Width::U16)? as u16)
    }

    fn dword(&mut self) -> Result<u32, F3WireError> {
        Ok(read_uint(self.take(4)?, Width::U32)? as u32)
    }

    fn byte_string(&mut self) -> Result<Vec<u8>, F3WireError> {
        let len = usize::from(self.word()?);
        Ok(decode_bytes(self.take(len)?).map_err(|_| CodecError::InvalidLength)?)
    }

    fn finish(self) -> Result<(), F3WireError> {
        if self.0.is_empty() {
            Ok(())
        } else {
            Err(F3WireError::TrailingBytes)
        }
    }
}

/// The writer at `0x14EC5E0` uses a u16 byte count and typed byte chunks
/// (`0x14EC600` → `0x14A5D40` in mode2). Keep the bytes opaque.
fn append_mon_action_string(
    body: &mut Vec<u8>,
    value: &[u8],
    suffix_len: usize,
) -> Result<(), F3WireError> {
    if value.len() > MAX_BODY_LEN - body.len() - 2 - suffix_len {
        return Err(CodecError::FrameTooLarge.into());
    }
    body.extend(write_uint(value.len() as u64, Width::U16)?);
    body.extend(encode_bytes(value).map_err(|_| CodecError::FieldOutOfRange)?);
    Ok(())
}

fn validate_selector(
    opcode: u32,
    selector: u8,
    valid: std::ops::RangeInclusive<u8>,
) -> Result<(), F3WireError> {
    if valid.contains(&selector) {
        Ok(())
    } else {
        Err(F3WireError::InvalidMonActionSelector { opcode, selector })
    }
}

/// Typed tails of the C→S 132 writers at `0xF53320`, `0xF50DD0` and
/// the kind-21 write at `0xF51DAD`. Kind 20 is outside the recorded set.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MonActionStep132Payload {
    Kind0(Vec<u8>),
    Kind1(u8),
    Kind9(u16, u16),
    Kind10(u16, u16),
    Kind11(u16, u16),
    Kind12 { selector: u8, value: u8 },
    Kind13(u16),
    Kind14,
    Kind15(u8),
    Kind16,
    Kind17(u8),
    Kind18,
    Kind19,
    Kind21(Vec<u8>),
    Kind22(u16),
}

impl MonActionStep132Payload {
    pub const fn action_kind(&self) -> u16 {
        match self {
            Self::Kind0(_) => 0,
            Self::Kind1(_) => 1,
            Self::Kind9(..) => 9,
            Self::Kind10(..) => 10,
            Self::Kind11(..) => 11,
            Self::Kind12 { .. } => 12,
            Self::Kind13(_) => 13,
            Self::Kind14 => 14,
            Self::Kind15(_) => 15,
            Self::Kind16 => 16,
            Self::Kind17(_) => 17,
            Self::Kind18 => 18,
            Self::Kind19 => 19,
            Self::Kind21(_) => 21,
            Self::Kind22(_) => 22,
        }
    }
}

/// Candidate C→S 132 body. The kind word is determined by the payload variant.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MonActionStep132Candidate {
    pub session_or_owner: u32,
    pub object_id: u16,
    pub payload: MonActionStep132Payload,
}

impl MonActionStep132Candidate {
    pub const OPCODE: u32 = F3MonActionProvenance::CP405_STATIC.step_report_opcodes[0];
    pub const STATUS: F3WireStatus = F3WireStatus::StaticCandidatePendingCapture;

    pub const fn action_kind(&self) -> u16 {
        self.payload.action_kind()
    }

    pub fn decode(body: &[u8]) -> Result<Self, F3WireError> {
        use MonActionStep132Payload::*;

        let mut reader = MonActionReader::new(body)?;
        let session_or_owner = reader.dword()?;
        let object_id = reader.word()?;
        let payload = match reader.word()? {
            0 => Kind0(reader.byte_string()?),
            1 => Kind1(reader.byte()?),
            9 => Kind9(reader.word()?, reader.word()?),
            10 => Kind10(reader.word()?, reader.word()?),
            11 => Kind11(reader.word()?, reader.word()?),
            12 => {
                let (selector, value) = (reader.byte()?, reader.byte()?);
                validate_selector(Self::OPCODE, selector, 0..=3)?;
                Kind12 { selector, value }
            }
            13 => Kind13(reader.word()?),
            14 => Kind14,
            15 => Kind15(reader.byte()?),
            16 => Kind16,
            17 => Kind17(reader.byte()?),
            18 => Kind18,
            19 => Kind19,
            21 => Kind21(reader.byte_string()?),
            22 => Kind22(reader.word()?),
            kind => {
                return Err(F3WireError::UnsupportedMonActionKind {
                    opcode: Self::OPCODE,
                    kind,
                });
            }
        };
        reader.finish()?;
        Ok(Self {
            session_or_owner,
            object_id,
            payload,
        })
    }

    pub fn encode(&self) -> Result<Vec<u8>, F3WireError> {
        use MonActionStep132Payload::*;

        let mut body = write_uint(self.session_or_owner.into(), Width::U32)?;
        body.extend(write_uint(self.object_id.into(), Width::U16)?);
        body.extend(write_uint(self.action_kind().into(), Width::U16)?);
        match &self.payload {
            Kind0(value) | Kind21(value) => append_mon_action_string(&mut body, value, 0)?,
            Kind1(value) | Kind15(value) | Kind17(value) => {
                body.extend(write_uint((*value).into(), Width::U8)?);
            }
            Kind9(first, second) | Kind10(first, second) | Kind11(first, second) => {
                body.extend(write_uint((*first).into(), Width::U16)?);
                body.extend(write_uint((*second).into(), Width::U16)?);
            }
            Kind12 { selector, value } => {
                validate_selector(Self::OPCODE, *selector, 0..=3)?;
                body.extend(write_uint((*selector).into(), Width::U8)?);
                body.extend(write_uint((*value).into(), Width::U8)?);
            }
            Kind13(value) | Kind22(value) => body.extend(write_uint((*value).into(), Width::U16)?),
            Kind14 | Kind16 | Kind18 | Kind19 => {}
        }
        Ok(body)
    }
}

/// Typed tails of the C→S 180 writers at `0x629940`, `0x6292A0`, and `0x627DE0`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MonActionStep180Payload {
    Kind0(u8),
    Kind3(u16, u16),
    Kind4(u16, u16),
    Kind5(u16, u16),
    Kind6 {
        selector: u8,
        value: u8,
    },
    Kind8,
    Kind9(Vec<u8>),
    Kind10 {
        target: Vec<u8>,
        first: u32,
        second: u32,
    },
}

impl MonActionStep180Payload {
    pub const fn action_kind(&self) -> u16 {
        match self {
            Self::Kind0(_) => 0,
            Self::Kind3(..) => 3,
            Self::Kind4(..) => 4,
            Self::Kind5(..) => 5,
            Self::Kind6 { .. } => 6,
            Self::Kind8 => 8,
            Self::Kind9(_) => 9,
            Self::Kind10 { .. } => 10,
        }
    }
}

/// Candidate C→S 180 body. The kind word is determined by the payload variant.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MonActionStep180Candidate {
    pub object_id: u16,
    pub payload: MonActionStep180Payload,
}

impl MonActionStep180Candidate {
    pub const OPCODE: u32 = F3MonActionProvenance::CP405_STATIC.step_report_opcodes[1];
    pub const STATUS: F3WireStatus = F3WireStatus::StaticCandidatePendingCapture;

    pub const fn action_kind(&self) -> u16 {
        self.payload.action_kind()
    }

    pub fn decode(body: &[u8]) -> Result<Self, F3WireError> {
        use MonActionStep180Payload::*;

        let mut reader = MonActionReader::new(body)?;
        let object_id = reader.word()?;
        let payload = match reader.word()? {
            0 => Kind0(reader.byte()?),
            3 => Kind3(reader.word()?, reader.word()?),
            4 => Kind4(reader.word()?, reader.word()?),
            5 => Kind5(reader.word()?, reader.word()?),
            6 => {
                let (selector, value) = (reader.byte()?, reader.byte()?);
                validate_selector(Self::OPCODE, selector, 14..=16)?;
                Kind6 { selector, value }
            }
            8 => Kind8,
            9 => Kind9(reader.byte_string()?),
            10 => Kind10 {
                target: reader.byte_string()?,
                first: reader.dword()?,
                second: reader.dword()?,
            },
            kind => {
                return Err(F3WireError::UnsupportedMonActionKind {
                    opcode: Self::OPCODE,
                    kind,
                });
            }
        };
        reader.finish()?;
        Ok(Self { object_id, payload })
    }

    pub fn encode(&self) -> Result<Vec<u8>, F3WireError> {
        use MonActionStep180Payload::*;

        let mut body = write_uint(self.object_id.into(), Width::U16)?;
        body.extend(write_uint(self.action_kind().into(), Width::U16)?);
        match &self.payload {
            Kind0(value) => body.extend(write_uint((*value).into(), Width::U8)?),
            Kind3(first, second) | Kind4(first, second) | Kind5(first, second) => {
                body.extend(write_uint((*first).into(), Width::U16)?);
                body.extend(write_uint((*second).into(), Width::U16)?);
            }
            Kind6 { selector, value } => {
                validate_selector(Self::OPCODE, *selector, 14..=16)?;
                body.extend(write_uint((*selector).into(), Width::U8)?);
                body.extend(write_uint((*value).into(), Width::U8)?);
            }
            Kind8 => {}
            Kind9(target) => append_mon_action_string(&mut body, target, 0)?,
            Kind10 {
                target,
                first,
                second,
            } => {
                append_mon_action_string(&mut body, target, 8)?;
                body.extend(write_uint((*first).into(), Width::U32)?);
                body.extend(write_uint((*second).into(), Width::U32)?);
            }
        }
        Ok(body)
    }
}

fn exact_body<const N: usize>(body: &[u8]) -> Result<&[u8; N], F3WireError> {
    body.try_into().map_err(|_| {
        if body.len() < N {
            F3WireError::TruncatedEof
        } else {
            F3WireError::TrailingBytes
        }
    })
}

/// Static candidate pending live Wine capture. The `RollAttack::MonsterAttackReq`
/// guards at `0x6C1BC0` and `0xF3B740` check host or matching `attackSeq` before
/// states `34..=37`. The host flag comes from local state (`0x11656A0`); its
/// inclusion in this candidate body neither establishes a wire field nor grants
/// host authority.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MonsterAttackReqCandidate {
    pub host: bool,
    pub attack_seq: u32,
    pub state: u8,
}

impl MonsterAttackReqCandidate {
    /// Static candidate status, pending live Wine capture.
    pub const STATUS: F3WireStatus = F3WireStatus::StaticCandidatePendingCapture;

    pub fn decode(body: &[u8]) -> Result<Self, F3WireError> {
        let body = exact_body::<6>(body)?;
        let host = match read_uint(&body[..1], Width::U8)? as u8 {
            0 => false,
            1 => true,
            flag => return Err(F3WireError::InvalidHostFlag(flag)),
        };
        let attack_seq = read_uint(&body[1..5], Width::U32)? as u32;
        let state = read_uint(&body[5..], Width::U8)? as u8;
        Self::validate_state(state)?;
        Ok(Self {
            host,
            attack_seq,
            state,
        })
    }

    pub fn encode(self) -> Result<Vec<u8>, F3WireError> {
        Self::validate_state(self.state)?;
        let mut body = Vec::with_capacity(6);
        body.extend(write_uint(self.host.into(), Width::U8)?);
        body.extend(write_uint(self.attack_seq.into(), Width::U32)?);
        body.extend(write_uint(self.state.into(), Width::U8)?);
        Ok(body)
    }

    fn validate_state(state: u8) -> Result<(), F3WireError> {
        match state {
            34..=37 => Ok(()),
            _ => Err(F3WireError::InvalidAttackState(state)),
        }
    }
}

/// Static candidate pending live Wine capture. Both bytes remain opaque. The
/// layout is shared by Hex-Rays cases `135` (`0xE67640`) and `136` (`0xE67770`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ObjectEffectPairCandidate {
    pub first: u8,
    pub second: u8,
}

impl ObjectEffectPairCandidate {
    /// Static candidate provenance and status, pending live Wine capture.
    pub const PROVENANCE: F3JumpTableProvenance = F3JumpTableProvenance::C_GAME_STAGE;

    pub fn decode(body: &[u8]) -> Result<Self, F3WireError> {
        let body = exact_body::<2>(body)?;
        Ok(Self {
            first: read_uint(&body[..1], Width::U8)? as u8,
            second: read_uint(&body[1..], Width::U8)? as u8,
        })
    }

    pub fn encode(self) -> Result<Vec<u8>, F3WireError> {
        let mut body = Vec::with_capacity(2);
        body.extend(write_uint(self.first.into(), Width::U8)?);
        body.extend(write_uint(self.second.into(), Width::U8)?);
        Ok(body)
    }
}

/// Static candidate pending live Wine capture in F3 PvE. This is the fixed
/// flag-1 body from Hex-Rays case `177` (`0xE751F0`). Flag 0 requires a count
/// and list (see `docs/protocol/load-phase.md#round-go`) outside this codec.
/// The first two values are opaque; the timers are stored before scaling.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TimedStageEventCandidate {
    pub first_number: u32,
    pub first_byte: u8,
    pub first_timer_seconds: u32,
    pub second_timer_seconds: u32,
    pub flag: u8,
}

impl TimedStageEventCandidate {
    /// Static candidate provenance and status, pending live Wine capture.
    pub const PROVENANCE: F3JumpTableProvenance = F3JumpTableProvenance::C_GAME_STAGE;

    pub fn decode(body: &[u8]) -> Result<Self, F3WireError> {
        let body = exact_body::<14>(body)?;
        let event = Self {
            first_number: read_uint(&body[..4], Width::U32)? as u32,
            first_byte: read_uint(&body[4..5], Width::U8)? as u8,
            first_timer_seconds: read_uint(&body[5..9], Width::U32)? as u32,
            second_timer_seconds: read_uint(&body[9..13], Width::U32)? as u32,
            flag: read_uint(&body[13..], Width::U8)? as u8,
        };
        event.validate()?;
        Ok(event)
    }

    pub fn encode(self) -> Result<Vec<u8>, F3WireError> {
        self.validate()?;
        let mut body = Vec::with_capacity(14);
        body.extend(write_uint(self.first_number.into(), Width::U32)?);
        body.extend(write_uint(self.first_byte.into(), Width::U8)?);
        body.extend(write_uint(self.first_timer_seconds.into(), Width::U32)?);
        body.extend(write_uint(self.second_timer_seconds.into(), Width::U32)?);
        body.extend(write_uint(self.flag.into(), Width::U8)?);
        Ok(body)
    }

    pub fn first_timer_ms(self) -> Result<u32, F3WireError> {
        self.first_timer_seconds
            .checked_mul(1000)
            .ok_or(F3WireError::TimerOverflow(F3TimerField::First))
    }

    pub fn second_timer_ms(self) -> Result<u32, F3WireError> {
        self.second_timer_seconds
            .checked_mul(1000)
            .ok_or(F3WireError::TimerOverflow(F3TimerField::Second))
    }

    fn validate(self) -> Result<(), F3WireError> {
        if self.flag != 1 {
            return Err(F3WireError::UnsupportedTimedEventFlag(self.flag));
        }
        self.first_timer_ms()?;
        self.second_timer_ms()?;
        Ok(())
    }
}
