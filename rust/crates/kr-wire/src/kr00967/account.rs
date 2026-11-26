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
        self.project_local_auth_with_identity_strings(players, |_| None)
    }

    /// Reuse the measured optional local string within each individual profile
    /// row before combining the roster. The echo and tail move with the row.
    pub fn project_local_auth_with_identity_strings<'a>(
        &self,
        players: &[(u32, &[u8], u32)],
        identity_string: impl Fn(usize) -> Option<&'a [u8]>,
    ) -> io::Result<Vec<u8>> {
        if !(1..=2).contains(&players.len()) {
            return Err(io::Error::other("unsupported local account count"));
        }
        let mut rows = Vec::with_capacity(players.len());
        for (index, &(identity, name, echo)) in players.iter().enumerate() {
            if players[..index].iter().any(|p| p.0 == identity) {
                return Err(io::Error::other("duplicate local account identity"));
            }
            let auth = self.project(identity, name, echo)?.auth;
            rows.push(match identity_string(index) {
                Some(tag) => with_identity_string(&auth, name.len(), tag)?,
                None => auth,
            });
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
/// Static-confirmed KR 00967 `UserSimpleInfo` EXP permille upper bound (`sub_902A50`
/// formats `0..=999` as `00.0%..99.9%` and `1000` as `100%`).
pub const EXP_PERMILLE_MAX: u32 = 1000;

/// Static-confirmed KR 00967 `UserSimpleInfoPacket` (`0xAE6330` -> `0x9022D0`
/// `"Lv.%3d"`): bounds DB `i64` level into the wire `u16` domain, rejecting
/// negative levels fail-closed.
pub fn wire_level_u16(level: i64) -> io::Result<u16> {
    if level < 0 {
        return Err(io::Error::other("negative profile level"));
    }
    Ok(u16::try_from(level).unwrap_or(u16::MAX))
}

/// Bounds a calculated EXP permille into KR 00967 `sub_902A50` `[0, 1000]`.
pub fn wire_exp_permille_u32(permille: u32) -> u32 {
    permille.min(EXP_PERMILLE_MAX)
}

/// CP183/184 `own291` template with optional `UserSimpleInfo` level at
/// `tail[11..13]` (`sub_AE6330` -> `sub_9022D0` `"Lv.%3d"`) and outer EXP permille
/// `u32` (`sub_D2FEB0` `0xd2ffc4` -> `UserSimpleInfo + 0x208` -> `sub_902A50`
/// `00.0%..100%`). When `(level, exp_permille) == (0, 0)`, output is
/// byte-identical to `own291`.
pub fn own291_with_progression(name: &[u8], level: u16, exp_permille: u32) -> io::Result<Vec<u8>> {
    super::player::account_name(name)?;
    let size = 108 + name.len();
    let mut raw = vec![24];
    raw.extend(((size - 5) as u32).to_le_bytes());
    raw.push(0);
    raw.extend((name.len() as u32).to_le_bytes());
    raw.extend(name);
    let mut tail = [0u8; 98];
    tail[11..13].copy_from_slice(&level.to_le_bytes());
    raw.extend(tail);
    let mut out = put(1, Width::U8)?;
    out.extend(put(size as u64, Width::U32)?);
    out.extend(encode_bytes(&raw)?);
    out.extend(put(
        u64::from(wire_exp_permille_u32(exp_permille)),
        Width::U32,
    )?);
    for _ in 0..4 {
        out.extend(put(0, Width::U8)?);
    }
    Ok(out)
}

/// Wrapper keeping byte-identical `exp_permille = 0` for level-only callers.
pub fn own291_with_level(name: &[u8], level: u16) -> io::Result<Vec<u8>> {
    own291_with_progression(name, level, 0)
}

/// CP183/184 zero-profile branch: name plus98 zero bytes, zero trailing fields.
pub fn own291(name: &[u8]) -> io::Result<Vec<u8>> {
    own291_with_level(name, 0)
}

/// D2FEB0 loops over the authenticated local roster and reads one active/blob
/// record for each slot; there is no count field in291 itself.
pub fn local_own291_with_progression(players: &[(&[u8], u16, u32)]) -> io::Result<Vec<u8>> {
    if !(1..=2).contains(&players.len()) {
        return Err(io::Error::other("unsupported local own-profile roster"));
    }
    let mut out = Vec::new();
    for &(name, level, exp_permille) in players {
        out.extend(own291_with_progression(name, level, exp_permille)?);
    }
    Ok(out)
}

/// Zero-EXP wrapper around `local_own291_with_progression`.
pub fn local_own291_with_levels(players: &[(&[u8], u16)]) -> io::Result<Vec<u8>> {
    let triples: Vec<(&[u8], u16, u32)> = players
        .iter()
        .map(|&(name, level)| (name, level, 0))
        .collect();
    local_own291_with_progression(&triples)
}

/// Zero-level wrapper around `local_own291_with_levels` keeping byte-identical
/// M5/M7/F1 output when `project_own291_level` is disabled.
pub fn local_own291(names: &[&[u8]]) -> io::Result<Vec<u8>> {
    let pairs: Vec<(&[u8], u16)> = names.iter().map(|&name| (name, 0)).collect();
    local_own291_with_levels(&pairs)
}

/// Static-confirmed KR 00967 `CStage` `S->C 135` (`0xD6CED0`): bounds DB `i64`
/// Lucci into the wire `u32` domain, rejecting negative balances fail-closed.
pub fn wire_lucci_u32(balance: i64) -> io::Result<u32> {
    if balance < 0 {
        return Err(io::Error::other("negative lucci balance"));
    }
    Ok(u32::try_from(balance).unwrap_or(u32::MAX))
}

/// Static-confirmed KR 00967 `CStage` `S->C 135` (`0xD6CED0` -> `0x13E4950`):
/// `{ u8 local_idx, u32 remain_lucci }` encoded with standard `Width::U8` and
/// `Width::U32` typed primitives.
pub fn lucci135(local_idx: u8, remain_lucci: u32) -> io::Result<Vec<u8>> {
    if local_idx > 1 {
        return Err(io::Error::other("unsupported local slot for lucci135"));
    }
    let mut out = put(u64::from(local_idx), Width::U8)?;
    out.extend(put(u64::from(remain_lucci), Width::U32)?);
    Ok(out)
}

/// Decode `CStage` `S->C 135` `{ u8 local_idx, u32 remain_lucci }` with strict
/// 5-byte length and local slot validation.
pub fn decode_lucci135(payload: &[u8]) -> io::Result<(u8, u32)> {
    if payload.len() != 5 {
        return Err(io::Error::other("unsupported lucci135 payload length"));
    }
    let local_idx = uint(&payload[..1], Width::U8)? as u8;
    if local_idx > 1 {
        return Err(io::Error::other("unsupported local slot for lucci135"));
    }
    let remain_lucci = uint(&payload[1..5], Width::U32)? as u32;
    Ok((local_idx, remain_lucci))
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

    #[test]
    fn lucci135_round_trip_and_bounds() {
        assert_eq!(wire_lucci_u32(0).unwrap(), 0);
        assert_eq!(wire_lucci_u32(123_456).unwrap(), 123_456);
        assert_eq!(wire_lucci_u32(i64::from(u32::MAX) + 99).unwrap(), u32::MAX);
        assert!(wire_lucci_u32(-1).is_err());

        for (slot, amount) in [(0, 0), (0, 123_456), (1, u32::MAX)] {
            let body = lucci135(slot, amount).unwrap();
            assert_eq!(body.len(), 5);
            assert_eq!(decode_lucci135(&body).unwrap(), (slot, amount));
        }
        assert!(lucci135(2, 100).is_err());
        assert!(decode_lucci135(&[0x5a; 4]).is_err());
        assert!(decode_lucci135(&[0x5a; 6]).is_err());
        let mut bad_slot = lucci135(1, 50).unwrap();
        bad_slot[0] = put(2, Width::U8).unwrap()[0];
        assert!(decode_lucci135(&bad_slot).is_err());
    }

    #[test]
    fn own291_with_level_projects_offset_11_and_keeps_zero_byte_identical() {
        assert_eq!(wire_level_u16(0).unwrap(), 0);
        assert_eq!(wire_level_u16(720).unwrap(), 720);
        assert_eq!(wire_level_u16(i64::from(u16::MAX) + 5).unwrap(), u16::MAX);
        assert!(wire_level_u16(-1).is_err());

        let name = b"CookieA";
        assert_eq!(own291_with_level(name, 0).unwrap(), own291(name).unwrap());
        let lv2 = own291_with_level(name, 2).unwrap();
        let size = 108 + name.len();
        assert_eq!(uint(&lv2[1..5], Width::U32).unwrap() as usize, size);
        let raw = decode_bytes(&lv2[5..5 + size]).unwrap();
        let tail_start = 10 + name.len();
        assert_eq!(raw.len(), size);
        assert_eq!(
            u16::from_le_bytes([raw[tail_start + 11], raw[tail_start + 12]]),
            2
        );
        assert!(
            raw[tail_start..]
                .iter()
                .enumerate()
                .all(|(idx, &b)| matches!(idx, 11 | 12) || b == 0)
        );
        assert!(local_own291_with_levels(&[]).is_err());
        assert!(local_own291_with_levels(&[(name, 1), (b"CookieB", 2), (b"CookieC", 3)]).is_err());
        let multi = local_own291_with_levels(&[(name, 1), (b"CookieB", 2)]).unwrap();
        assert_eq!(
            multi.len(),
            own291_with_level(name, 1).unwrap().len()
                + own291_with_level(b"CookieB", 2).unwrap().len()
        );

        assert_eq!(wire_exp_permille_u32(0), 0);
        assert_eq!(wire_exp_permille_u32(166), 166);
        assert_eq!(wire_exp_permille_u32(1500), EXP_PERMILLE_MAX);
        assert_eq!(
            own291_with_progression(name, 2, 0).unwrap(),
            own291_with_level(name, 2).unwrap()
        );
        let prog = own291_with_progression(name, 2, 166).unwrap();
        assert_eq!(prog.len(), lv2.len());
        let trailing_u32_at = 5 + size;
        assert_eq!(
            uint(&prog[trailing_u32_at..trailing_u32_at + 4], Width::U32).unwrap(),
            166
        );
        let multi_prog =
            local_own291_with_progression(&[(name, 2, 166), (b"CookieB", 1, 75)]).unwrap();
        assert_eq!(
            multi_prog,
            [
                own291_with_progression(name, 2, 166).unwrap(),
                own291_with_progression(b"CookieB", 1, 75).unwrap(),
            ]
            .concat()
        );
    }
}
