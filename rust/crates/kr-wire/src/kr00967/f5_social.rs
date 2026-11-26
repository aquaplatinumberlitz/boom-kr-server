//! KR 00967 social payloads, `kr_static/confirmed` only. See
//! docs/protocol/f2-f5-f3-ida-anchors.md and the source hashes in its evidence.
//! Friend, CHOCOPacket framing, guild management and ranking wire stay unknown.

use super::{decode_bytes, encode_bytes, put, uint};
use crate::{FRAME_OVERHEAD, MAX_FRAME_LEN, Width};
use encoding_rs::EUC_KR;
use std::{io, num::NonZeroU8};

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
    pub const OPCODE: u32 = 474;

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
    pub const OPCODE: u32 = 633;

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

#[cfg(test)]
mod tests {
    use super::*;

    fn reject_partial_or_trailing<T>(body: &[u8], decode: impl Fn(&[u8]) -> io::Result<T>) {
        for n in 0..body.len() {
            assert!(decode(&body[..n]).is_err(), "accepted {n} bytes");
        }
        let mut extra = body.to_vec();
        extra.push(0);
        assert!(decode(&extra).is_err());
    }

    #[test]
    fn guild_invite_and_blocked_refusal_follow_ida_field_order() {
        // Synthetic body literals from the static field order and KR primitive
        // masks. They check the codecs, not runtime behavior of the client.
        let invite_body = [
            0xa5, 0x68, 0x1b, // "A"
            0xa5, 0x6b, 0x74, 0xae, // "한" (C7 D1)
            0x97, 0xc8, 0x50, 0x91, // first u32
            0x67, 0xc8, 0x50, 0x91, // second u32
            0xa5, 0x6b, 0x84, 0x33, // "Z!"
        ];
        let invite = GuildInvite {
            first: "A".into(),
            second: "한".into(),
            first_number: 0x0102_0304,
            second_number: 0xf102_0304,
            third: "Z!".into(),
        };
        assert_eq!(GuildInvite::decode(&invite_body).unwrap(), invite);
        assert_eq!(invite.encode().unwrap(), invite_body);
        reject_partial_or_trailing(&invite_body, GuildInvite::decode);

        let refuse_body = [0x5a, 0xa5, 0x6b, 0x84, 0x33, 0xa5, 0x68, 0x1b];
        let refusal = GuildInviteAutoRefuse {
            invite_third: invite.third,
            invite_first: invite.first,
        };
        assert_eq!(
            GuildInviteAutoRefuse::decode(&refuse_body).unwrap(),
            refusal
        );
        assert_eq!(refusal.encode().unwrap(), refuse_body);
        reject_partial_or_trailing(&refuse_body, GuildInviteAutoRefuse::decode);
        let mut manual = refuse_body;
        manual[0] = 0x5b;
        assert!(GuildInviteAutoRefuse::decode(&manual).is_err());
    }

    #[test]
    fn guild_codecs_reject_bad_lengths_invalid_text_and_oversized_output() {
        let mut body = GuildInvite {
            first: "A".into(),
            second: String::new(),
            first_number: 0,
            second_number: u32::MAX,
            third: String::new(),
        };
        let valid = body.encode().unwrap();
        assert_eq!(GuildInvite::decode(&valid).unwrap(), body);
        let mut bad = valid;
        bad[..2].copy_from_slice(&put(u16::MAX.into(), Width::U16).unwrap());
        assert!(GuildInvite::decode(&bad).is_err());
        bad[..2].copy_from_slice(&[0xa5, 0x68]);
        bad[2] = 0xa5; // Encoded native byte 0xff.
        assert!(GuildInvite::decode(&bad).is_err());
        assert!(GuildInviteAutoRefuse::decode(&[0x5a, 0xa5, 0x68, 0xa5, 0xa5, 0x69]).is_err());
        body.second = "🦀".into();
        assert!(body.encode().is_err());
        body.second = "a".repeat(MAX_BODY);
        assert!(body.encode().is_err());
        assert!(GuildInvite::decode(&vec![0; MAX_BODY + 1]).is_err());
        let mut refuse = GuildInviteAutoRefuse {
            invite_third: String::new(),
            invite_first: "a".repeat(MAX_BODY - 5),
        };
        let edge = refuse.encode().unwrap();
        assert_eq!(edge.len(), MAX_BODY);
        assert_eq!(GuildInviteAutoRefuse::decode(&edge).unwrap(), refuse);
        refuse.invite_first.push('a');
        assert!(refuse.encode().is_err());
    }

    #[test]
    fn chat_connect_reply_has_status_dependent_length_and_no_inferred_framing() {
        let failed = [0x5a];
        let endpoint = [
            0x58, 0x5d, 0x97, 0xc8, 0x50, 0x91, 0x94, 0xca, 0x53, 0xea, 0xae, 0xd1,
        ];
        for (body, reply) in [
            (&failed[..], ChatServerConnectReply::Failed),
            (
                &endpoint[..],
                ChatServerConnectReply::Endpoint {
                    ok: NonZeroU8::new(2).unwrap(),
                    second_byte: 7,
                    first_number: 0x0102_0304,
                    ip: 0x0200_007f,
                    port: 3000,
                },
            ),
        ] {
            assert_eq!(ChatServerConnectReply::decode(body).unwrap(), reply);
            assert_eq!(reply.encode().unwrap(), body);
            reject_partial_or_trailing(body, ChatServerConnectReply::decode);
        }
        let mut extra_on_failure = endpoint;
        extra_on_failure[0] = failed[0];
        assert!(ChatServerConnectReply::decode(&extra_on_failure).is_err());
    }
}
