//! Measured two-player field adapters; opaque template bytes stay opaque.
use crate::room::Member;
use kr_wire::{Width, read_uint, write_uint};
use std::io;
fn uint(data: &[u8], w: Width) -> io::Result<u64> {
    read_uint(data, w).map_err(io::Error::other)
}
fn put(v: u64, w: Width) -> io::Result<Vec<u8>> {
    write_uint(v, w).map_err(io::Error::other)
}
pub fn player(template: &[u8], member: &Member, chief: bool) -> io::Result<Vec<u8>> {
    if template.len() != 204 || uint(&template[1..5], Width::U32)? != u64::from(member.key.identity)
    {
        return Err(io::Error::other("wrong pinned player106 identity"));
    }
    let mut body = template.to_vec();
    body[0] = put(u64::from(member.slot), Width::U8)?[0];
    let prior = uint(&body[12..16], Width::U32)?;
    let low = if chief {
        2
    } else if member.ready {
        4
    } else {
        0
    };
    let team = u64::from(member.slot); // fixed two-player opposing-team policy
    body[12..16].copy_from_slice(&put((prior & 0xffff0000) | low | (team << 5), Width::U32)?);
    Ok(body)
}
pub fn slot(template: &[u8], member: &Member, chief: bool) -> io::Result<Vec<u8>> {
    let p = player(template, member, chief)?;
    let mut out = vec![p[0]];
    out.extend_from_slice(&p[12..16]);
    out.extend(put(0, Width::U8)?);
    Ok(out)
}
pub fn removal(member: &Member) -> io::Result<Vec<u8>> {
    let mut out = put(u64::from(member.slot), Width::U8)?;
    out.extend(put(u64::from(member.key.identity), Width::U32)?);
    out.extend(put(0, Width::U8)?);
    Ok(out)
}
pub fn join(template: &[u8], member: &Member) -> io::Result<Vec<u8>> {
    if template.len() != 7 || uint(&template[..1], Width::U8)? != 0 {
        return Err(io::Error::other("wrong join87 template"));
    }
    let mut out = template.to_vec();
    out[1] = put(u64::from(member.slot), Width::U8)?[0];
    Ok(out)
}
pub fn create_title(body: &[u8]) -> io::Result<Vec<u8>> {
    if !(57..=88).contains(&body.len()) {
        return Err(io::Error::other("create81 size"));
    }
    let n = uint(&body[..2], Width::U16)? as usize;
    if !(1..=32).contains(&n) || body.len() != 56 + n || uint(&body[2 + n..4 + n], Width::U16)? != 0
    {
        return Err(io::Error::other("create81 title/password branch"));
    }
    let title = crate::lobby::decode_bytes(&body[2..2 + n])?;
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
pub fn join_context(body: &[u8]) -> io::Result<()> {
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
pub fn is_zero_setup(body: &[u8]) -> io::Result<bool> {
    Ok(body.len() == 4 && uint(body, Width::U32)? == 0)
}
pub fn peer(
    template: &[u8],
    key: u32,
    recipient: u8,
    endpoints: [(u32, u16); 2],
) -> io::Result<Vec<u8>> {
    if template.len() < 35 || recipient > 1 {
        return Err(io::Error::other("unsupported two-player peer153"));
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
    for _ in 2..26 {
        out.extend(put(0, Width::U8)?);
    }
    Ok(out)
}

pub fn listing_title(body: &[u8]) -> io::Result<Vec<u8>> {
    if body.len() < 16 || uint(&body[..2], Width::U16)? != 1 {
        return Err(io::Error::other("listing count"));
    }
    let n = uint(&body[2..6], Width::U32)? as usize;
    if n != body.len() - 6 {
        return Err(io::Error::other("listing blob size"));
    }
    let stream = crate::lobby::decode_bytes(&body[6..])?;
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

fn encode_bytes(data: &[u8]) -> io::Result<Vec<u8>> {
    let mut at = 0;
    let mut out = Vec::new();
    while at < data.len() {
        let (n, w) = if data.len() - at >= 4 {
            (4, Width::U32)
        } else if data.len() - at >= 2 {
            (2, Width::U16)
        } else {
            (1, Width::U8)
        };
        let mut raw = [0; 8];
        raw[..n].copy_from_slice(&data[at..at + n]);
        out.extend(put(u64::from_le_bytes(raw), w)?);
        at += n;
    }
    Ok(out)
}
pub fn selector(body: &[u8]) -> io::Result<u8> {
    let title = create_title(body)?;
    Ok(uint(&body[8 + title.len()..12 + title.len()], Width::U32)? as u8)
}
pub fn room_selector(body: &[u8], opcode: u32, selector: u8) -> io::Result<Vec<u8>> {
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
pub fn replace_listing_title(body: &[u8], title: &[u8]) -> io::Result<Vec<u8>> {
    listing_title(body)?;
    if !(1..=32).contains(&title.len()) || title.iter().any(|b| *b < 32 || *b == 127) {
        return Err(io::Error::other("listing title bytes"));
    }
    let stream = crate::lobby::decode_bytes(&body[6..])?;
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
