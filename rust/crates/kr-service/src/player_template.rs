//! Measured zero-profile106 transformation, preserving opaque fields.
//! CP018/020/026/199 layout and Python four_player_profile reference.
use kr_wire::{Width, read_uint, write_uint};
use std::io;

fn uint(data: &[u8], width: Width) -> io::Result<u64> {
    read_uint(data, width).map_err(io::Error::other)
}
fn put(value: u64, width: Width) -> io::Result<Vec<u8>> {
    write_uint(value, width).map_err(io::Error::other)
}
fn encode_bytes(data: &[u8]) -> io::Result<Vec<u8>> {
    let mut out = Vec::with_capacity(data.len());
    let mut at = 0;
    while at < data.len() {
        let (size, width) = if data.len() - at >= 4 {
            (4, Width::U32)
        } else if data.len() - at >= 2 {
            (2, Width::U16)
        } else {
            (1, Width::U8)
        };
        let mut value = [0; 8];
        value[..size].copy_from_slice(&data[at..at + size]);
        out.extend(put(u64::from_le_bytes(value), width)?);
        at += size;
    }
    Ok(out)
}
/// Retain the opaque prefix/suffix from one pinned106. The measured profile
/// branch has a name and 98 zero bytes; both106 names must agree. No optional
/// accessory/profile branch is inferred or silently projected.
pub(crate) struct PlayerTemplate {
    prefix: Vec<u8>,
    before_outer_name: Vec<u8>,
    suffix: Vec<u8>,
}
impl PlayerTemplate {
    pub(crate) fn parse(body: &[u8]) -> io::Result<Self> {
        if ![200, 204].contains(&body.len()) {
            return Err(io::Error::other("unsupported player106 template size"));
        }
        let size = uint(&body[16..20], Width::U32)? as usize;
        if !matches!(size, 113 | 115) || 25 + size > body.len() {
            return Err(io::Error::other("unsupported player106 profile size"));
        }
        let stream = crate::lobby::decode_bytes(&body[20..20 + size])?;
        if stream[0] != 0x18
            || u32::from_le_bytes(stream[1..5].try_into().expect("bounded profile")) as usize
                != size - 5
            || stream[5] != 0
        {
            return Err(io::Error::other("unsupported player106 envelope"));
        }
        let length =
            u32::from_le_bytes(stream[6..10].try_into().expect("bounded profile")) as usize;
        if !matches!(length, 5 | 7) || size != 108 + length || stream[10 + length..] != [0; 98] {
            return Err(io::Error::other("unsupported player106 zero-profile"));
        }
        let tail = &body[20 + size..];
        let outer = uint(&tail[3..5], Width::U16)? as usize;
        if outer != length
            || tail.len() < 5 + outer
            || crate::lobby::decode_bytes(&tail[5..5 + outer])? != stream[10..10 + length]
        {
            return Err(io::Error::other("player106 names disagree"));
        }
        Ok(Self {
            prefix: body[..16].to_vec(),
            before_outer_name: tail[..3].to_vec(),
            suffix: tail[5 + outer..].to_vec(),
        })
    }
    pub(crate) fn project(&self, identity: u32, name: &[u8]) -> io::Result<Vec<u8>> {
        if identity == 0
            || !matches!(name.len(), 5 | 7)
            || !name.is_ascii()
            || name.iter().any(|c| !(32..=126).contains(c))
        {
            return Err(io::Error::other("unsupported player106 identity/name"));
        }
        let mut inner = (name.len() as u32).to_le_bytes().to_vec();
        inner.extend_from_slice(name);
        inner.extend_from_slice(&[0; 98]);
        let mut stream = vec![0x18];
        stream.extend_from_slice(&((inner.len() + 1) as u32).to_le_bytes());
        stream.push(0);
        stream.extend(inner);
        let mut body = self.prefix.clone();
        body[1..5].copy_from_slice(&put(u64::from(identity), Width::U32)?);
        body.extend(put(stream.len() as u64, Width::U32)?);
        body.extend(encode_bytes(&stream)?);
        body.extend_from_slice(&self.before_outer_name);
        body.extend(put(name.len() as u64, Width::U16)?);
        body.extend(encode_bytes(name)?);
        body.extend_from_slice(&self.suffix);
        Ok(body)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn example() -> Vec<u8> {
        let name = b"KRLocal";
        let mut stream = vec![0x18];
        stream.extend_from_slice(&110u32.to_le_bytes());
        stream.push(0);
        stream.extend_from_slice(&7u32.to_le_bytes());
        stream.extend(name);
        stream.extend([0; 98]);
        let mut body = vec![0; 16];
        body[1..5].copy_from_slice(&put(1, Width::U32).unwrap());
        body[12..16].copy_from_slice(&put(0x12340002, Width::U32).unwrap());
        body.extend(put(stream.len() as u64, Width::U32).unwrap());
        body.extend(encode_bytes(&stream).unwrap());
        body.extend([1, 2, 3]); // opaque bytes retained, no semantics assigned
        body.extend(put(7, Width::U16).unwrap());
        body.extend(encode_bytes(name).unwrap());
        body.extend([0x17; 57]); // opaque accessory suffix
        assert_eq!(body.len(), 204);
        body
    }
    #[test]
    fn shorter_name_and_unrelated_identity_keep_opaque_fields_and_both_names() {
        let base = example();
        let template = PlayerTemplate::parse(&base).unwrap();
        assert_eq!(template.project(1, b"KRLocal").unwrap(), base);
        let projected = template.project(70000, b"KRNew").unwrap();
        assert_eq!(projected.len(), 200);
        assert_eq!(uint(&projected[1..5], Width::U32).unwrap(), 70000);
        assert_eq!(&projected[5..16], &base[5..16]);
        assert_eq!(&projected[projected.len() - 57..], &[0x17; 57]);
        let reparsed = PlayerTemplate::parse(&projected).unwrap();
        assert_eq!(reparsed.project(1, b"KRLocal").unwrap(), base);
        for (identity, name) in [
            (0, b"KRNew".as_slice()),
            (1, b"short!".as_slice()),
            (1, b"bad\x01xxx".as_slice()),
        ] {
            assert!(template.project(identity, name).is_err());
        }
    }
    #[test]
    fn malformed_envelope_profile_variant_and_disagreeing_names_fail_closed() {
        let base = example();
        for size in [0, 16, 199, 201, 203] {
            assert!(PlayerTemplate::parse(&base[..size]).is_err());
        }
        for encoded_size in [0, 112, 114, 116, u32::MAX] {
            let mut bad = base.clone();
            bad[16..20].copy_from_slice(&put(encoded_size as u64, Width::U32).unwrap());
            assert!(PlayerTemplate::parse(&bad).is_err());
        }
        for kind in 0..4 {
            let mut bad = base.clone();
            let mut stream = crate::lobby::decode_bytes(&bad[20..135]).unwrap();
            match kind {
                0 => stream[0] = 0x19,
                1 => stream[5] = 1,
                2 => stream[10 + 7] = 1, // nonzero optional profile content
                _ => stream[10] ^= 1,    // inner name disagrees with outer name
            }
            bad[20..135].copy_from_slice(&encode_bytes(&stream).unwrap());
            assert!(PlayerTemplate::parse(&bad).is_err());
        }
    }
}
