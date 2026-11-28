//! KR 00967 F6 event anchors from p233 (`kr_static`, pending live Wine capture).
//! See `docs/protocol/f5-f6-f7-ingame-wire-re.md` and
//! `docs/evidence/content/f5-f6-f7-ingame-wire-re.json` (IDA DB SHA-256
//! `6be15a1fccea0837ee95c50893345e9f13dc217b0fa570975b5c284f0d547fd1`).
//! These entries establish neither a live dispatch route nor server policy.

#![deny(unsafe_code)]

use crate::{CodecError, Width, read_uint, write_uint};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum F6EventWireStatus {
    /// A bounded body or prefix codec follows the static reader order.
    StaticCandidatePendingCapture,
    /// Static anchor only; this entry has no payload codec.
    UnmeasuredPendingCapture,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum F6StageKind {
    GameStage,
    LobbyStage,
}

impl F6StageKind {
    /// `CGameStage::OnPacket` or the `CStage::OnPacket` lobby dispatch path.
    pub const fn dispatcher_va(self) -> u32 {
        match self {
            Self::GameStage => 0xE5D3A0,
            Self::LobbyStage => 0xD68C90,
        }
    }
}

/// p233 names both tables for `CGameStage::OnPacket` without mapping them per opcode.
pub const C_GAME_STAGE_JUMP_TABLE_VAS: [u32; 2] = [0xE5DF9C, 0xE5E0DC];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct F6EventAnchor {
    pub stage: F6StageKind,
    pub opcode: u16,
    pub handler_va: u32,
    /// `None` when p233 does not pin a jump table for this opcode.
    pub jump_table_va: Option<u32>,
    pub status: F6EventWireStatus,
    pub note: &'static str,
}

impl F6EventAnchor {
    const fn unmeasured(
        stage: F6StageKind,
        opcode: u16,
        handler_va: u32,
        note: &'static str,
    ) -> Self {
        Self {
            stage,
            opcode,
            handler_va,
            jump_table_va: match stage {
                F6StageKind::GameStage => Some(C_GAME_STAGE_JUMP_TABLE_VAS[0]),
                F6StageKind::LobbyStage => None,
            },
            status: F6EventWireStatus::UnmeasuredPendingCapture,
            note,
        }
    }
}

use F6StageKind::{GameStage, LobbyStage};

pub static F6_EVENT_ANCHORS: &[F6EventAnchor] = &[
    F6EventAnchor::unmeasured(
        GameStage,
        321,
        0xE6AA90,
        "Event ID <= 0x14E; accepting stage entity triggers C->S 174.",
    ),
    F6EventAnchor::unmeasured(GameStage, 322, 0xE6AEB0, "Shared event reward reader."),
    F6EventAnchor::unmeasured(GameStage, 323, 0xE6AEB0, "Shared event reward reader."),
    F6EventAnchor::unmeasured(GameStage, 548, 0xE69440, "Conditional reward fields."),
    F6EventAnchor::unmeasured(
        GameStage,
        549,
        0xE69D30,
        "Delegates to event state routine.",
    ),
    F6EventAnchor {
        stage: GameStage,
        opcode: Stage553EventCandidate::OPCODE,
        handler_va: 0xE69D80,
        jump_table_va: Some(C_GAME_STAGE_JUMP_TABLE_VAS[0]),
        status: Stage553EventCandidate::STATUS,
        note: "Reads u8, u16, u8.",
    },
    F6EventAnchor::unmeasured(GameStage, 566, 0xE6D830, "Conditional RPG reward fields."),
    F6EventAnchor::unmeasured(GameStage, 568, 0xE75C90, "Stage status depends on branch."),
    F6EventAnchor::unmeasured(GameStage, 569, 0xE76ED0, "Stage status depends on branch."),
    F6EventAnchor::unmeasured(GameStage, 574, 0xE75CB0, "Stage status depends on branch."),
    F6EventAnchor::unmeasured(GameStage, 576, 0xE763A0, "Stage status depends on branch."),
    F6EventAnchor::unmeasured(GameStage, 577, 0xE760B0, "Stage status depends on branch."),
    F6EventAnchor::unmeasured(LobbyStage, 408, 0xD7B2B0, "Status branches 0..=5."),
    F6EventAnchor::unmeasured(LobbyStage, 409, 0xD7B5D0, "Leading u8 must be nonzero."),
    F6EventAnchor::unmeasured(LobbyStage, 413, 0xD7B2B0, "Status branches 0..=5."),
    F6EventAnchor::unmeasured(LobbyStage, 414, 0xD7C2C0, "Mode gate: 11, 14, or 21."),
    F6EventAnchor {
        stage: LobbyStage,
        opcode: Lobby418PrefixCandidate::OPCODE,
        handler_va: 0xD852C0,
        jump_table_va: None,
        status: Lobby418PrefixCandidate::STATUS,
        note: "Reads a u8, u16, u32 prefix; remaining fields depend on the branch.",
    },
    F6EventAnchor::unmeasured(
        LobbyStage,
        419,
        0xD852C0,
        "Shares the u8, u16, u32 prefix reader; remaining fields depend on the branch.",
    ),
];

/// Looks up static metadata by stage and opcode without enabling dispatch.
pub fn f6_event_anchor(stage: F6StageKind, opcode: u16) -> Option<&'static F6EventAnchor> {
    F6_EVENT_ANCHORS
        .iter()
        .find(|anchor| anchor.stage == stage && anchor.opcode == opcode)
}

fn exact_body<const N: usize>(body: &[u8]) -> Result<&[u8; N], CodecError> {
    body.try_into().map_err(|_| {
        if body.len() < N {
            CodecError::TruncatedEof
        } else {
            CodecError::InvalidLength
        }
    })
}

/// Static body candidate for opcode 553; field meanings remain unverified.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Stage553EventCandidate {
    pub flag_a: u8,
    pub value: u16,
    pub flag_b: u8,
}

impl Stage553EventCandidate {
    pub const OPCODE: u16 = 553;
    pub const STATUS: F6EventWireStatus = F6EventWireStatus::StaticCandidatePendingCapture;

    pub fn encode(self) -> Result<Vec<u8>, CodecError> {
        let mut body = Vec::with_capacity(4);
        body.extend(write_uint(self.flag_a.into(), Width::U8)?);
        body.extend(write_uint(self.value.into(), Width::U16)?);
        body.extend(write_uint(self.flag_b.into(), Width::U8)?);
        Ok(body)
    }

    pub fn decode(body: &[u8]) -> Result<Self, CodecError> {
        let body = exact_body::<4>(body)?;
        Ok(Self {
            flag_a: read_uint(&body[..1], Width::U8)? as u8,
            value: read_uint(&body[1..3], Width::U16)? as u16,
            flag_b: read_uint(&body[3..], Width::U8)? as u8,
        })
    }
}

/// Static prefix candidate for opcode 418; branch-dependent fields are opaque.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Lobby418PrefixCandidate {
    pub kind: u8,
    pub param: u16,
    pub value: u32,
}

impl Lobby418PrefixCandidate {
    pub const OPCODE: u16 = 418;
    pub const STATUS: F6EventWireStatus = F6EventWireStatus::StaticCandidatePendingCapture;

    pub fn encode_prefix(self) -> Result<Vec<u8>, CodecError> {
        let mut prefix = Vec::with_capacity(7);
        prefix.extend(write_uint(self.kind.into(), Width::U8)?);
        prefix.extend(write_uint(self.param.into(), Width::U16)?);
        prefix.extend(write_uint(self.value.into(), Width::U32)?);
        Ok(prefix)
    }

    /// Expects exactly the seven prefix bytes, excluding any branch fields.
    pub fn decode_prefix(prefix: &[u8]) -> Result<Self, CodecError> {
        let prefix = exact_body::<7>(prefix)?;
        Ok(Self {
            kind: read_uint(&prefix[..1], Width::U8)? as u8,
            param: read_uint(&prefix[1..3], Width::U16)? as u16,
            value: read_uint(&prefix[3..], Width::U32)? as u32,
        })
    }
}
