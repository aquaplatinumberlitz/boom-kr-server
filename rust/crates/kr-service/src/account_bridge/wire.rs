//! CP214–217/225 measured count1 branches. Opaque fields are not credentials.
use super::{Error, FrontendToken, Result};
use crate::lobby::decode_bytes;
use kr_wire::{Width, crypt, read_uint};
use std::net::Ipv4Addr;
use zeroize::Zeroizing;

fn field(bytes: &[u8], width: Width) -> Result<u64> {
    read_uint(bytes, width).map_err(|_| Error::UnsupportedWire)
}

/// Untrusted request metadata. This does not select an account or authorize a peer.
pub(crate) fn reported_address(body: &[u8]) -> Result<Ipv4Addr> {
    if body.len() != 61 {
        return Err(Error::UnsupportedWire);
    }
    let value = field(&body[56..60], Width::U32)? as u32;
    Ok(Ipv4Addr::from(value.to_le_bytes()))
}

/// Narrow network profile: token32/body61, opaque16, zero, reported IPv4, zero.
/// The expected address constrains client metadata; it does not identify an
/// account or establish the transport address observed beyond NAT.
pub fn login_token(body: &[u8], peer: Ipv4Addr) -> Result<FrontendToken> {
    if body.len() != 61
        || field(&body[..1], Width::U8)? != 4
        || field(&body[1..2], Width::U8)? != 1
        || field(&body[2..4], Width::U16)? != 32
        || field(&body[52..56], Width::U32)? != 0
        || field(&body[56..60], Width::U32)? != u64::from(u32::from_le_bytes(peer.octets()))
        || field(&body[60..], Width::U8)? != 0
    {
        return Err(Error::UnsupportedWire);
    }
    FrontendToken::from_bytes(Zeroizing::new(crypt(&body[4..36], 0x1122_3344, true)))
}

#[derive(Debug, PartialEq, Eq)]
pub struct ChannelIdentity {
    pub wire_id: u32,
    pub name: Vec<u8>,
    /// Legacy field36 echo: only a correlator, not a strong credential.
    pub echo: u32,
}

/// Bounded backend ASCII name3..15, count1/no-optional profile projection.
/// The optional-string flag must be off; the other 23 trailing bytes stay opaque.
pub fn channel_identity(body: &[u8]) -> Result<ChannelIdentity> {
    if body.len() < 4 || field(&body[..1], Width::U8)? != 1 || field(&body[1..2], Width::U8)? != 255
    {
        return Err(Error::UnsupportedWire);
    }
    let size = field(&body[2..4], Width::U16)? as usize;
    if !(3..=15).contains(&size) || body.len() != 36 + size {
        return Err(Error::UnsupportedWire);
    }
    if field(&body[31 + size..32 + size], Width::U8)? != 0 {
        return Err(Error::UnsupportedWire);
    }
    let name = decode_bytes(&body[4..4 + size]).map_err(|_| Error::UnsupportedWire)?;
    if !name.iter().all(|b| b.is_ascii_alphanumeric() || *b == b'_') {
        return Err(Error::UnsupportedWire);
    }
    Ok(ChannelIdentity {
        wire_id: field(&body[4 + size..8 + size], Width::U32)? as u32,
        name,
        echo: field(&body[8 + size..12 + size], Width::U32)? as u32,
    })
}
