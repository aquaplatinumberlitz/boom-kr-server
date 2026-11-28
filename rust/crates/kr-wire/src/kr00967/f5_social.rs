//! KR 00967 F5 anchors and body codecs from `kr_static` evidence. See
//! `docs/protocol/f2-f5-f3-ida-anchors.md` and
//! `docs/evidence/content/f2-f5-f3-ida-anchors.json` (IDA DB SHA-256
//! `6be15a1fccea0837ee95c50893345e9f13dc217b0fa570975b5c284f0d547fd1`).
//! An opcode anchor does not confirm a body layout, live route, or server policy.

use super::{decode_bytes, encode_bytes, put, uint};
use crate::{FRAME_OVERHEAD, MAX_FRAME_LEN, Width};
use encoding_rs::EUC_KR;
use std::{io, num::NonZeroU8};

pub const F5_S2C_GUILD_467: u32 = 467;
pub const F5_S2C_GUILD_INVITE: u32 = 474;
pub const F5_S2C_REFUSE_GUILD_INVITE: u32 = 478;
pub const F5_C2S_GUILD_INVITE_AUTO_REFUSE: u32 = 633;
pub const F5_S2C_COUPLE_257: u32 = 257;
pub const F5_S2C_COUPLE_273: u32 = 273;
pub const F5_S2C_COUPLE_284: u32 = 284;
pub const F5_S2C_COUPLE_640: u32 = 640;
pub const F5_C2S_COUPLE_670: u32 = 670;
pub const F5_S2C_COUPLE_674: u32 = 674;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum F5SocialGroup {
    Guild,
    Couple,
    Chat,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum F5Direction {
    ServerToClient,
    ClientToServer,
}

/// IDA virtual addresses in the KR 00967 image.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum F5AnchorSite {
    /// Handler dispatched by `CStage::OnPacket` at `0xD68C90`.
    CStageHandler(u32),
    /// Callers of the opcode builder at `0x1164C10`.
    RequestBuilderCallers(&'static [u32]),
    ChatStageHandler {
        handler_va: u32,
        connect_callee_va: u32,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum F5ConfirmedBody {
    GuildInvite,
    /// Only the blocked-invites branch of C->S 633.
    GuildInviteAutoRefuse,
    ChatServerConnectReply,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum F5WireContract {
    KrStaticConfirmed(F5ConfirmedBody),
    /// Anchor only: there is no payload codec for this entry.
    UnmeasuredWireContract,
}

/// Static metadata only. Presence in the registry does not enable dispatch.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct F5SocialAnchor {
    pub group: F5SocialGroup,
    pub direction: F5Direction,
    /// `None` when only the handler and its body are known.
    pub opcode: Option<u32>,
    /// Descriptive name when available in the evidence.
    pub name: Option<&'static str>,
    pub site: F5AnchorSite,
    pub wire_contract: F5WireContract,
}

impl F5SocialAnchor {
    const fn unmeasured_handler(group: F5SocialGroup, opcode: u32, handler_va: u32) -> Self {
        Self {
            group,
            direction: F5Direction::ServerToClient,
            opcode: Some(opcode),
            name: None,
            site: F5AnchorSite::CStageHandler(handler_va),
            wire_contract: F5WireContract::UnmeasuredWireContract,
        }
    }
}

pub static F5_SOCIAL_ANCHORS: &[F5SocialAnchor] = &[
    F5SocialAnchor::unmeasured_handler(F5SocialGroup::Guild, F5_S2C_GUILD_467, 0x6D9340),
    F5SocialAnchor {
        group: F5SocialGroup::Guild,
        direction: F5Direction::ServerToClient,
        opcode: Some(F5_S2C_GUILD_INVITE),
        name: Some("GuildInvite"),
        site: F5AnchorSite::CStageHandler(0xD769D0),
        wire_contract: F5WireContract::KrStaticConfirmed(F5ConfirmedBody::GuildInvite),
    },
    F5SocialAnchor {
        name: Some("RefuseGuildInvite"),
        ..F5SocialAnchor::unmeasured_handler(
            F5SocialGroup::Guild,
            F5_S2C_REFUSE_GUILD_INVITE,
            0xD76DA0,
        )
    },
    F5SocialAnchor {
        group: F5SocialGroup::Guild,
        direction: F5Direction::ClientToServer,
        opcode: Some(F5_C2S_GUILD_INVITE_AUTO_REFUSE),
        name: Some("GuildInviteAutoRefuse"),
        site: F5AnchorSite::RequestBuilderCallers(&[0xD769D0]),
        wire_contract: F5WireContract::KrStaticConfirmed(F5ConfirmedBody::GuildInviteAutoRefuse),
    },
    F5SocialAnchor::unmeasured_handler(F5SocialGroup::Couple, F5_S2C_COUPLE_257, 0x128D3C0),
    F5SocialAnchor::unmeasured_handler(F5SocialGroup::Couple, F5_S2C_COUPLE_273, 0xD87250),
    F5SocialAnchor::unmeasured_handler(F5SocialGroup::Couple, F5_S2C_COUPLE_284, 0x128D120),
    F5SocialAnchor::unmeasured_handler(F5SocialGroup::Couple, F5_S2C_COUPLE_640, 0xD88830),
    F5SocialAnchor {
        group: F5SocialGroup::Couple,
        direction: F5Direction::ClientToServer,
        opcode: Some(F5_C2S_COUPLE_670),
        name: None,
        site: F5AnchorSite::RequestBuilderCallers(&[0x583530, 0x583E80]),
        wire_contract: F5WireContract::UnmeasuredWireContract,
    },
    F5SocialAnchor::unmeasured_handler(F5SocialGroup::Couple, F5_S2C_COUPLE_674, 0xD7FA60),
    F5SocialAnchor {
        group: F5SocialGroup::Chat,
        direction: F5Direction::ServerToClient,
        opcode: None,
        name: Some("CChattingStage::OnResConnectToSvr"),
        site: F5AnchorSite::ChatStageHandler {
            handler_va: 0xD58760,
            connect_callee_va: 0x13AD4D0,
        },
        wire_contract: F5WireContract::KrStaticConfirmed(F5ConfirmedBody::ChatServerConnectReply),
    },
];

/// Looks up static opcode metadata in the specified direction, including anchor-only entries.
pub fn f5_social_opcode_anchor(
    direction: F5Direction,
    opcode: u32,
) -> Option<&'static F5SocialAnchor> {
    F5_SOCIAL_ANCHORS
        .iter()
        .find(|anchor| anchor.direction == direction && anchor.opcode == Some(opcode))
}

const MAX_BODY: usize = MAX_FRAME_LEN - FRAME_OVERHEAD;

fn invalid() -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, "invalid KR social payload")
}

struct Reader<'a>(&'a [u8]);

impl<'a> Reader<'a> {
    fn new(body: &'a [u8]) -> io::Result<Self> {
        if body.len() > MAX_BODY {
            return Err(invalid());
        }
        Ok(Self(body))
    }

    fn take(&mut self, len: usize) -> io::Result<&'a [u8]> {
        let (head, tail) = self
            .0
            .split_at_checked(len)
            .ok_or(io::ErrorKind::UnexpectedEof)?;
        self.0 = tail;
        Ok(head)
    }

    fn u8(&mut self) -> io::Result<u8> {
        Ok(uint(self.take(1)?, Width::U8)? as u8)
    }

    fn u16(&mut self) -> io::Result<u16> {
        Ok(uint(self.take(2)?, Width::U16)? as u16)
    }

    fn u32(&mut self) -> io::Result<u32> {
        Ok(uint(self.take(4)?, Width::U32)? as u32)
    }

    fn text(&mut self) -> io::Result<String> {
        let len = usize::from(self.u16()?);
        let bytes = decode_bytes(self.take(len)?)?;
        let value = EUC_KR
            .decode_without_bom_handling_and_without_replacement(&bytes)
            .ok_or_else(invalid)?;
        if EUC_KR.encode(&value).0 != bytes {
            return Err(invalid());
        }
        Ok(value.into_owned())
    }

    fn finish(self) -> io::Result<()> {
        if self.0.is_empty() {
            Ok(())
        } else {
            Err(invalid())
        }
    }
}

fn append_text(body: &mut Vec<u8>, value: &str) -> io::Result<()> {
    // Bound conversion before allocating the native CP949 bytes.
    if value.len() > MAX_BODY * 3 {
        return Err(invalid());
    }
    let (bytes, _, lossy) = EUC_KR.encode(value);
    if lossy || body.len() + 2 + bytes.len() > MAX_BODY {
        return Err(invalid());
    }
    body.extend(put(bytes.len() as u64, Width::U16)?);
    body.extend(encode_bytes(&bytes)?);
    Ok(())
}

/// S->C 474, read by `0xD769D0`. Field names follow read order where their
/// precise meanings are unverified.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GuildInvite {
    pub first: String,
    pub second: String,
    pub first_number: u32,
    pub second_number: u32,
    pub third: String,
}

impl GuildInvite {
    pub const OPCODE: u32 = F5_S2C_GUILD_INVITE;

    pub fn decode(body: &[u8]) -> io::Result<Self> {
        let mut reader = Reader::new(body)?;
        let invite = Self {
            first: reader.text()?,
            second: reader.text()?,
            first_number: reader.u32()?,
            second_number: reader.u32()?,
            third: reader.text()?,
        };
        reader.finish()?;
        Ok(invite)
    }

    pub fn encode(&self) -> io::Result<Vec<u8>> {
        let mut body = Vec::new();
        append_text(&mut body, &self.first)?;
        append_text(&mut body, &self.second)?;
        body.extend(put(self.first_number.into(), Width::U32)?);
        body.extend(put(self.second_number.into(), Width::U32)?);
        append_text(&mut body, &self.third)?;
        Ok(body)
    }
}

/// C->S 633, automatic refusal only. At `0xD769D0`, the blocked-invites
/// branch writes 0 with `0x14EBC50` and then invite strings 3 and 1 with
/// `0x14EC5E0`. The other branches using opcode 633 are out of scope.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GuildInviteAutoRefuse {
    pub invite_third: String,
    pub invite_first: String,
}

impl GuildInviteAutoRefuse {
    pub const OPCODE: u32 = F5_C2S_GUILD_INVITE_AUTO_REFUSE;

    pub fn decode(body: &[u8]) -> io::Result<Self> {
        let mut reader = Reader::new(body)?;
        if reader.u8()? != 0 {
            return Err(invalid());
        }
        let refusal = Self {
            invite_third: reader.text()?,
            invite_first: reader.text()?,
        };
        reader.finish()?;
        Ok(refusal)
    }

    pub fn encode(&self) -> io::Result<Vec<u8>> {
        let mut body = put(0, Width::U8)?;
        append_text(&mut body, &self.invite_third)?;
        append_text(&mut body, &self.invite_first)?;
        Ok(body)
    }
}

/// Payload read by `CChattingStage::OnResConnectToSvr` at `0xD58760`.
/// A zero status consumes only that byte. Any nonzero status reads all fields;
/// the client then requires nonzero IP and port before connecting. No opcode,
/// IP conversion, or CHOCO transport framing is inferred here.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChatServerConnectReply {
    Failed,
    Endpoint {
        ok: NonZeroU8,
        second_byte: u8,
        first_number: u32,
        ip: u32,
        port: u16,
    },
}

impl ChatServerConnectReply {
    pub fn decode(body: &[u8]) -> io::Result<Self> {
        let mut reader = Reader::new(body)?;
        let reply = match NonZeroU8::new(reader.u8()?) {
            None => Self::Failed,
            Some(ok) => Self::Endpoint {
                ok,
                second_byte: reader.u8()?,
                first_number: reader.u32()?,
                ip: reader.u32()?,
                port: reader.u16()?,
            },
        };
        reader.finish()?;
        Ok(reply)
    }

    pub fn encode(self) -> io::Result<Vec<u8>> {
        match self {
            Self::Failed => put(0, Width::U8),
            Self::Endpoint {
                ok,
                second_byte,
                first_number,
                ip,
                port,
            } => {
                let mut body = put(ok.get().into(), Width::U8)?;
                body.extend(put(second_byte.into(), Width::U8)?);
                body.extend(put(first_number.into(), Width::U32)?);
                body.extend(put(ip.into(), Width::U32)?);
                body.extend(put(port.into(), Width::U16)?);
                Ok(body)
            }
        }
    }
}
