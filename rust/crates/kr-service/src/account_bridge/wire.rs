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

/// Bounded name3..15 native bytes, count1/no-optional profile projection.
/// 949x validates the bytes; the bridge separately fences them to the issued
/// name and its configured ASCII/patched-client policy.
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
    kr_wire::kr00967::text949x::validate_name(&name).map_err(|_| Error::UnsupportedWire)?;
    Ok(ChannelIdentity {
        wire_id: field(&body[4 + size..8 + size], Width::U32)? as u32,
        name,
        echo: field(&body[8 + size..12 + size], Width::U32)? as u32,
    })
}

/// CP342 actual local2P channel request, with the cached native serializer
/// matched to the current executable. Only the bounded no-optional-string
/// profile is admitted. Names/IDs/echo are claims until the transport adapter
/// matches every slot to its issued DB roster; parsing grants no authority.
pub fn local_channel_identities(body: &[u8]) -> Result<Vec<ChannelIdentity>> {
    if !(36..=85).contains(&body.len()) || field(&body[1..2], Width::U8)? != 255 {
        return Err(Error::UnsupportedWire);
    }
    let count = field(&body[..1], Width::U8)? as usize;
    if !(1..=2).contains(&count) {
        return Err(Error::UnsupportedWire);
    }
    let mut cursor = 2;
    let mut players: Vec<ChannelIdentity> = Vec::with_capacity(count);
    for _ in 0..count {
        let raw_size = body.get(cursor..cursor + 2).ok_or(Error::UnsupportedWire)?;
        let size = field(raw_size, Width::U16)? as usize;
        if !(3..=15).contains(&size) {
            return Err(Error::UnsupportedWire);
        }
        cursor += 2;
        let record = body
            .get(cursor..cursor + size + 17)
            .ok_or(Error::UnsupportedWire)?;
        let name = decode_bytes(&record[..size]).map_err(|_| Error::UnsupportedWire)?;
        kr_wire::kr00967::text949x::validate_name(&name).map_err(|_| Error::UnsupportedWire)?;
        let wire_id = field(&record[size..size + 4], Width::U32)? as u32;
        let echo = field(&record[size + 4..size + 8], Width::U32)? as u32;
        if wire_id == 0 || players.iter().any(|p| p.wire_id == wire_id) {
            return Err(Error::UnsupportedWire);
        }
        players.push(ChannelIdentity {
            wire_id,
            name,
            echo,
        });
        // The remaining u64/u8 per record retain opaque semantics.
        cursor += size + 17;
    }
    let tail = body.get(cursor..).ok_or(Error::UnsupportedWire)?;
    if tail.len() != 15 || field(&tail[10..11], Width::U8)? != 0 {
        return Err(Error::UnsupportedWire);
    }
    Ok(players)
}

#[cfg(test)]
mod local_channel_tests {
    use super::*;

    // Retained r60 native C2S6 body, independently CRC-verified at sequence40.
    const NATIVE: &str = "58a5a56ef98601dec40a3696ca539496ca53950000000071b185885aa56ee38d01ded60c2e96ca539796ca53950000000071b185885a5a5b96ca539596ca53955a96ca5395";
    fn native() -> Vec<u8> {
        (0..NATIVE.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&NATIVE[i..i + 2], 16).unwrap())
            .collect()
    }
    #[test]
    fn measured_two_player_request_is_bounded_and_preserves_order() {
        let body = native();
        let players = local_channel_identities(&body).unwrap();
        assert_eq!(
            players,
            vec![
                ChannelIdentity {
                    wire_id: 1,
                    name: b"KRLocal".to_vec(),
                    echo: 0
                },
                ChannelIdentity {
                    wire_id: 2,
                    name: b"KRGuest".to_vec(),
                    echo: 0
                }
            ]
        );
        for end in 0..body.len() {
            assert!(local_channel_identities(&body[..end]).is_err());
        }
        let mut extra = body.clone();
        extra.push(0);
        assert!(local_channel_identities(&extra).is_err());
        // Derived negative controls: duplicate/zero identity, unsupported count,
        // optionalstring, and a name length extending beyond retained input.
        for (at, width, value) in [
            (37, Width::U32, 1),
            (37, Width::U32, 0),
            (0, Width::U8, 3),
            (64, Width::U8, 1),
            (2, Width::U16, 128),
        ] {
            let mut bad = body.clone();
            let raw = kr_wire::write_uint(value, width).unwrap();
            bad[at..at + raw.len()].copy_from_slice(&raw);
            assert!(local_channel_identities(&bad).is_err());
        }
        // The old one-player API keeps rejecting count2 until its serving
        // owner is explicitly replaced by the multi-principal adapter.
        assert!(channel_identity(&body).is_err());
    }
}
