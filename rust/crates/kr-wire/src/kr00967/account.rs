//! Zero-profile account projection from retained KR templates. This module
//! builds bytes; backend ownership and native admission are service concerns.
use super::{PlayerTemplate, decode_bytes, encode_bytes, put, uint};
use crate::Width;
use std::io;

pub struct AccountTemplate {
    auth_prefix: Vec<u8>,
    auth_tail: Vec<u8>,
    player: PlayerTemplate,
}
pub struct AccountPackets {
    pub auth: Vec<u8>,
    pub own291: Vec<u8>,
    pub player106: Vec<u8>,
}
impl AccountTemplate {
    /// Source templates stay on the independently measured name5/name7 branch.
    /// Only identity/name/echo are projected; every other auth byte is retained.
    pub fn parse(auth: &[u8], own: &[u8], player: &[u8]) -> io::Result<Self> {
        if !matches!(auth.len(), 135 | 137) || uint(&auth[..1], Width::U8)? != 0 {
            return Err(io::Error::other("unsupported account auth template"));
        }
        let n = uint(&auth[29..31], Width::U16)? as usize;
        if !matches!(n, 5 | 7) || auth.len() != 130 + n {
            return Err(io::Error::other("auth template name branch"));
        }
        let name = decode_bytes(&auth[31..31 + n])?;
        super::player::account_name(&name)?;
        let identity = uint(&auth[25..29], Width::U32)? as u32;
        if identity == 0
            || uint(
                player
                    .get(1..5)
                    .ok_or_else(|| io::Error::other("short player template"))?,
                Width::U32,
            )? != u64::from(identity)
        {
            return Err(io::Error::other("account template identity mismatch"));
        }
        let template = PlayerTemplate::parse(player)?;
        if template.project(identity, &name)? != player || own291(&name)? != own {
            return Err(io::Error::other(
                "account templates disagree or nonzero profile branch",
            ));
        }
        Ok(Self {
            auth_prefix: auth[..25].to_vec(),
            auth_tail: auth[31 + n..].to_vec(),
            player: template,
        })
    }
    pub fn project(&self, identity: u32, name: &[u8], echo: u32) -> io::Result<AccountPackets> {
        super::player::account_name(name)?;
        if identity == 0 {
            return Err(io::Error::other("zero account wire identity"));
        }
        let mut auth = self.auth_prefix.clone();
        auth.extend(put(u64::from(identity), Width::U32)?);
        auth.extend(put(name.len() as u64, Width::U16)?);
        auth.extend(encode_bytes(name)?);
        auth.extend(&self.auth_tail);
        let offset = 86 + name.len();
        auth[offset..offset + 4].copy_from_slice(&put(u64::from(echo), Width::U32)?);
        Ok(AccountPackets {
            auth,
            own291: own291(name)?,
            player106: self.player.project_account(identity, name)?,
        })
    }

    /// Project only the login reply for one or two local accounts.
    /// The two-player layout was consumed by the declared KR compatibility
    /// copy in r55: common13-byte prefix, per-player record, common19-byte tail.
    /// This does not authorize accounts or define multi-player291/channel data.
    pub fn project_local_auth(&self, players: &[(u32, &[u8], u32)]) -> io::Result<Vec<u8>> {
        if !(1..=2).contains(&players.len()) {
            return Err(io::Error::other("unsupported local account count"));
        }
        let mut rows = Vec::with_capacity(players.len());
        for (index, &(identity, name, echo)) in players.iter().enumerate() {
            if players[..index].iter().any(|p| p.0 == identity) {
                return Err(io::Error::other("duplicate local account identity"));
            }
            rows.push(self.project(identity, name, echo)?.auth);
        }
        let first = &rows[0];
        let mut auth = first[..13].to_vec();
        auth[4..5].copy_from_slice(&put(players.len() as u64, Width::U8)?);
        let tail = &first[first.len() - 19..];
        for row in &rows {
            if row[..13] != first[..13] || row[row.len() - 19..] != *tail {
                return Err(io::Error::other("inconsistent local account auth profile"));
            }
            auth.extend_from_slice(&row[13..row.len() - 19]);
        }
        auth.extend_from_slice(tail);
        Ok(auth)
    }
}
/// Diagnostic test accounts only: fill the empty per-player identity string that
/// follows the name and one u8 in the auth reply (client 0x10FD540 stores it at
/// CSelfInfo+0x21C; an empty value blocks room-title editing). Only the string
/// value changes; field order and encoding stay as measured.
pub fn with_identity_string(auth: &[u8], name_len: usize, id: &[u8]) -> io::Result<Vec<u8>> {
    let at = 31 + name_len + 1;
    if id.is_empty()
        || id.len() > 40
        || !id.iter().all(|b| b.is_ascii_graphic())
        || auth.len() < at + 2
        || uint(&auth[at..at + 2], Width::U16)? != 0
    {
        return Err(io::Error::other("unsupported identity string patch"));
    }
    let mut out = auth[..at].to_vec();
    out.extend(put(id.len() as u64, Width::U16)?);
    out.extend(encode_bytes(id)?);
    out.extend(&auth[at + 2..]);
    Ok(out)
}
/// CP183/184 zero-profile branch: name plus98 zero bytes, zero trailing fields.
pub fn own291(name: &[u8]) -> io::Result<Vec<u8>> {
    super::player::account_name(name)?;
    let size = 108 + name.len();
    let mut raw = vec![24];
    raw.extend(((size - 5) as u32).to_le_bytes());
    raw.push(0);
    raw.extend((name.len() as u32).to_le_bytes());
    raw.extend(name);
    raw.extend([0; 98]);
    let mut out = put(1, Width::U8)?;
    out.extend(put(size as u64, Width::U32)?);
    out.extend(encode_bytes(&raw)?);
    out.extend(put(0, Width::U32)?);
    for _ in 0..4 {
        out.extend(put(0, Width::U8)?);
    }
    Ok(out)
}

/// D2FEB0 loops over the authenticated local roster and reads one active/blob
/// record for each slot; there is no count field in291 itself. CP183 measured
/// the single-slot record. Multi-slot native acceptance remains separate from
/// this projection of the cached consumer contract.
pub fn local_own291(names: &[&[u8]]) -> io::Result<Vec<u8>> {
    if !(1..=2).contains(&names.len()) {
        return Err(io::Error::other("unsupported local own-profile roster"));
    }
    let mut out = Vec::new();
    for name in names {
        out.extend(own291(name)?);
    }
    Ok(out)
}

#[cfg(test)]
mod identity_string_tests {
    use super::*;
    #[test]
    fn identity_string_replaces_only_the_empty_field() {
        let name = b"KRLocal";
        let mut auth = vec![0x5a; 31];
        auth.extend(encode_bytes(name).unwrap());
        auth.extend(put(0, Width::U8).unwrap());
        auth.extend(put(0, Width::U16).unwrap());
        auth.extend([1, 2, 3, 4]);
        let out = with_identity_string(&auth, name.len(), b"id@test").unwrap();
        let at = 31 + name.len() + 1;
        assert_eq!(&out[..at], &auth[..at]);
        assert_eq!(uint(&out[at..at + 2], Width::U16).unwrap(), 7);
        assert_eq!(decode_bytes(&out[at + 2..at + 9]).unwrap(), b"id@test");
        assert_eq!(&out[at + 9..], &[1, 2, 3, 4]);
        assert!(with_identity_string(&out, name.len(), b"again").is_err());
        assert!(with_identity_string(&auth, name.len(), b"bad id").is_err());
    }
}
