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
