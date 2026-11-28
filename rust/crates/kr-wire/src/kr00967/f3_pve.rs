//! KR00967 F3 PvE body candidates (`kr_static/candidate`, pending live Wine
//! capture). See `docs/protocol/f2-f5-f3-ida-anchors.md` and
//! `docs/evidence/content/f2-f5-f3-ida-anchors.json` (IDA DB SHA-256
//! `6be15a1fccea0837ee95c50893345e9f13dc217b0fa570975b5c284f0d547fd1`).
//! Existing codecs provide body validation only; see [`F3JumpTableProvenance`]
//! before associating any Hex-Rays case label with a wire opcode. The CP405
//! MonAction descriptors validate metadata and do not encode or decode bytes.

#![deny(unsafe_code)]

use crate::{CodecError, Width, read_uint, write_uint};
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
    pub hex_rays_cases: [u32; 5],
    pub status: F3WireStatus,
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
        if self.opcode != F3MonActionProvenance::CP405_STATIC.delegation_opcode {
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
