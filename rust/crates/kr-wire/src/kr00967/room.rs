//! Measured room body layouts; no session, authority, timer or membership state.
use super::{decode_bytes, encode_bytes, put, uint};
use crate::{Width, read_uint};
use std::io;

pub fn player106(
    template: &[u8],
    identity: u32,
    slot: u8,
    team: u8,
    ready: bool,
    chief: bool,
) -> io::Result<Vec<u8>> {
    if ![200, 204].contains(&template.len())
        || uint(&template[1..5], Width::U32)? != u64::from(identity)
    {
        return Err(io::Error::other("wrong pinned player106 identity"));
    }
    player106_state(template, slot, team, ready, chief)
}

pub fn player106_state(
    template: &[u8],
    slot: u8,
    team: u8,
    ready: bool,
    chief: bool,
) -> io::Result<Vec<u8>> {
    if ![200, 204].contains(&template.len()) || slot >= 26 || team > 1 {
        return Err(io::Error::other("unsupported player106 slot/team"));
    }
    let mut body = template.to_vec();
    body[0] = put(u64::from(slot), Width::U8)?[0];
    let prior = uint(&body[12..16], Width::U32)?;
    let low = if chief {
        2
    } else if ready {
        4
    } else {
        0
    };
    body[12..16].copy_from_slice(&put(
        (prior & 0xffff0000) | low | (u64::from(team) << 5),
        Width::U32,
    )?);
    Ok(body)
}

pub fn create81_title(body: &[u8]) -> io::Result<Vec<u8>> {
    if !(57..=88).contains(&body.len()) {
        return Err(io::Error::other("create81 size"));
    }
    let n = uint(&body[..2], Width::U16)? as usize;
    if !(1..=32).contains(&n) || body.len() != 56 + n || uint(&body[2 + n..4 + n], Width::U16)? != 0
    {
        return Err(io::Error::other("create81 title/password branch"));
    }
    let title = decode_bytes(&body[2..2 + n])?;
    // Opaque client title bytes: bounded/control-free CP949 carrier, not UTF-8.
    // No Unicode normalization or account name semantics are inferred.
    if title.iter().any(|b| *b < 32 || *b == 127) {
        return Err(io::Error::other("title control bytes"));
    }
    for i in 0..12 {
        uint(&body[4 + n + i * 4..8 + n + i * 4], Width::U32)?;
    }
    for i in 0..4 {
        uint(&body[52 + n + i..53 + n + i], Width::U8)?;
    }
    if uint(&body[8 + n..12 + n], Width::U32)? > 1 {
        return Err(io::Error::other("unsupported fixed room selector"));
    }
    Ok(title)
}

pub fn join82_context(body: &[u8]) -> io::Result<()> {
    if body.len() != 11 {
        return Err(io::Error::other("join82 size"));
    }
    let widths = [Width::U16, Width::U32, Width::U16, Width::U8, Width::U16];
    let expected = [1, 1, 0, 1, 5];
    let mut at = 0;
    for (w, expected) in widths.into_iter().zip(expected) {
        let n = match w {
            Width::U8 => 1,
            Width::U16 => 2,
            Width::U32 => 4,
            Width::U64 => 8,
        };
        if uint(&body[at..at + n], w)? != expected {
            return Err(io::Error::other("wrong fixed room join context"));
        }
        at += n;
    }
    Ok(())
}

pub fn zero585_setup(body: &[u8]) -> io::Result<bool> {
    Ok(body.len() == 4 && uint(body, Width::U32)? == 0)
}

pub fn start121_teams(body: &[u8], teams: &[u8]) -> io::Result<Vec<u8>> {
    if teams.len() != 8 || body.len() != 285 || uint(&body[..1], Width::U8)? != 0 {
        return Err(io::Error::other("expected measured eight start121"));
    }
    let size = uint(&body[1..5], Width::U32)? as usize;
    if size != 262 || teams.iter().any(|team| *team > 1) {
        return Err(io::Error::other("eight start121 envelope/teams"));
    }
    let mut stream = decode_bytes(&body[5..5 + size])?;
    if stream[..6] != [24, 1, 1, 0, 0, 0] || stream[6 + 43] != 8 {
        return Err(io::Error::other("eight start121 stream/count"));
    }
    for (slot, team) in teams.iter().enumerate() {
        let at = 6 + 48 + 26 * slot;
        if stream[at] != slot as u8
            || stream[at + 2] != slot as u8
            || u16::from_le_bytes([stream[at + 3], stream[at + 4]]) != slot as u16
        {
            return Err(io::Error::other("eight start121 row slot"));
        }
        stream[at + 5] = *team;
    }
    let mut out = body[..5].to_vec();
    out.extend(encode_bytes(&stream)?);
    out.extend_from_slice(&body[5 + size..]);
    Ok(out)
}

pub fn peer153(
    template: &[u8],
    key: u32,
    recipient: u8,
    endpoints: &[(u32, u16)],
) -> io::Result<Vec<u8>> {
    if template.len() < 35
        || !matches!(endpoints.len(), 2 | 4 | 8)
        || usize::from(recipient) >= endpoints.len()
    {
        return Err(io::Error::other("unsupported diagnostic peer153"));
    }
    let mut out = template[..35].to_vec();
    out[15..19].copy_from_slice(&put(u64::from(key), Width::U32)?);
    for (slot, remote_endpoint) in endpoints.iter().enumerate() {
        let active = true;
        out.extend(put(u64::from(active), Width::U8)?);
        if active {
            let (ip, port) = if slot != usize::from(recipient) {
                *remote_endpoint
            } else {
                (0, 0)
            };
            if slot != usize::from(recipient) && (ip == 0 || port == 0) {
                return Err(io::Error::other("missing current peer endpoint"));
            }
            for (address, endpoint) in [(ip, port), (0, 0), (0, 0)] {
                out.extend(put(u64::from(address), Width::U32)?);
                out.extend(put(u64::from(endpoint), Width::U16)?);
            }
        }
    }
    for _ in endpoints.len()..26 {
        out.extend(put(0, Width::U8)?);
    }
    Ok(out)
}

pub fn listing91_title(body: &[u8]) -> io::Result<Vec<u8>> {
    if body.len() < 16 || uint(&body[..2], Width::U16)? != 1 {
        return Err(io::Error::other("listing count"));
    }
    let n = uint(&body[2..6], Width::U32)? as usize;
    if n != body.len() - 6 {
        return Err(io::Error::other("listing blob size"));
    }
    let stream = decode_bytes(&body[6..])?;
    if stream.len() < 16
        || stream[0] != 0x18
        || u32::from_le_bytes(stream[1..5].try_into().expect("checked stream")) as usize != n - 5
        || stream[5] != 0
    {
        return Err(io::Error::other("listing envelope"));
    }
    let row = &stream[6..];
    if row.len() < 32
        || u16::from_le_bytes(row[..2].try_into().expect("checked row")) != 1
        || u32::from_le_bytes(row[2..6].try_into().expect("checked row")) != 1
    {
        return Err(io::Error::other("fixed listing tuple"));
    }
    let len = u32::from_le_bytes(row[6..10].try_into().expect("checked row")) as usize;
    if !(1..=32).contains(&len) || row.len() != 32 + len {
        return Err(io::Error::other("listing title size"));
    }
    Ok(row[10..10 + len].to_vec())
}

pub fn create81_selector(body: &[u8]) -> io::Result<u8> {
    let title = create81_title(body)?;
    Ok(uint(&body[8 + title.len()..12 + title.len()], Width::U32)? as u8)
}

pub fn selector_reply(body: &[u8], opcode: u32, selector: u8) -> io::Result<Vec<u8>> {
    let offset = match (opcode, body.len()) {
        (86, 11) => 7,
        (87, 7) => 5,
        (180, 3)
            if read_uint(&body[..1], Width::U8).map_err(io::Error::other)? == 0
                && read_uint(&body[2..], Width::U8).map_err(io::Error::other)? == 0 =>
        {
            1
        }
        _ => return Err(io::Error::other("unsupported mode reply")),
    };
    if selector > 1 {
        return Err(io::Error::other("unsupported mode selector"));
    }
    let mut out = body.to_vec();
    out[offset] = put(u64::from(selector), Width::U8)?[0];
    Ok(out)
}

pub fn listing91_replace_title(body: &[u8], title: &[u8]) -> io::Result<Vec<u8>> {
    listing91_title(body)?;
    if !(1..=32).contains(&title.len()) || title.iter().any(|b| *b < 32 || *b == 127) {
        return Err(io::Error::other("listing title bytes"));
    }
    let stream = decode_bytes(&body[6..])?;
    let row = &stream[6..];
    let old = u32::from_le_bytes(row[6..10].try_into().expect("validated row")) as usize;
    let mut updated = row[..6].to_vec();
    updated.extend_from_slice(&(title.len() as u32).to_le_bytes());
    updated.extend_from_slice(title);
    updated.extend_from_slice(&row[10 + old..]);
    let mut blob = vec![0x18];
    blob.extend_from_slice(&((updated.len() + 1) as u32).to_le_bytes());
    blob.push(0);
    blob.extend(updated);
    let mut out = body[..2].to_vec();
    out.extend(put(blob.len() as u64, Width::U32)?);
    out.extend(encode_bytes(&blob)?);
    Ok(out)
}

pub fn join87(template: &[u8], slot: u8) -> io::Result<Vec<u8>> {
    if template.len() != 7 || uint(&template[..1], Width::U8)? != 0 {
        return Err(io::Error::other("wrong join87 template"));
    }
    let mut out = template.to_vec();
    out[1] = put(u64::from(slot), Width::U8)?[0];
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn four_peer_endpoints_are_distinct_self_zero_and_all_inactive_slots_explicit() {
        let endpoints = [
            (0x0a4d0002, 32001),
            (0x0a4d0003, 32002),
            (0x0a4d0004, 32003),
            (0x0a4d0005, 32004),
        ];
        for recipient in 0..4 {
            let body = peer153(&[0; 97], 12346, recipient, &endpoints).unwrap();
            assert_eq!(body.len(), 133);
            let mut at = 35;
            for (slot, endpoint) in endpoints.iter().enumerate() {
                assert_eq!(uint(&body[at..at + 1], Width::U8).unwrap(), 1);
                at += 1;
                let expected = if slot == usize::from(recipient) {
                    (0, 0)
                } else {
                    *endpoint
                };
                assert_eq!(
                    uint(&body[at..at + 4], Width::U32).unwrap(),
                    u64::from(expected.0)
                );
                assert_eq!(
                    uint(&body[at + 4..at + 6], Width::U16).unwrap(),
                    u64::from(expected.1)
                );
                for offset in [6, 12] {
                    assert_eq!(
                        uint(&body[at + offset..at + offset + 4], Width::U32).unwrap(),
                        0
                    );
                    assert_eq!(
                        uint(&body[at + offset + 4..at + offset + 6], Width::U16).unwrap(),
                        0
                    );
                }
                at += 18;
            }
            for _ in 4..26 {
                assert_eq!(uint(&body[at..at + 1], Width::U8).unwrap(), 0);
                at += 1;
            }
            assert_eq!(at, body.len());
        }
        assert!(peer153(&[0; 97], 12346, 4, &endpoints).is_err());
        assert!(peer153(&[0; 97], 12346, 0, &endpoints[..3]).is_err());
        let mut missing = endpoints;
        missing[1] = (0, 0);
        assert!(peer153(&[0; 97], 12346, 0, &missing).is_err());
        assert_eq!(
            peer153(&[0; 97], 12346, 0, &endpoints[..2]).unwrap().len(),
            97
        );
    }
}
