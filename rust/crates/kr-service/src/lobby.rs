//! Pinned diagnostic identity projections, not accounts or general lobby/rooms.
//! CP058/183/186; immutable templates keep unmeasured fields opaque.
use kr_wire::{Width, read_uint};
use std::io;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stage {
    Login,
    Info,
    Redirect,
    LoginDone,
    Channel,
    Ready,
    Lobby,
}

pub struct Flow {
    pub stage: Stage,
    ignored: usize,
    identity: u32,
    name: Vec<u8>,
}
impl Flow {
    pub fn login() -> Self {
        Self::for_identity(false, 1, b"KRLocal")
    }
    pub fn channel() -> Self {
        Self::for_identity(true, 1, b"KRLocal")
    }
    pub fn for_identity(channel: bool, identity: u32, name: &[u8]) -> Self {
        Self {
            stage: if channel {
                Stage::Channel
            } else {
                Stage::Login
            },
            ignored: 0,
            identity,
            name: name.to_vec(),
        }
    }

    pub fn packet(&mut self, opcode: u32, body: &[u8]) -> io::Result<Vec<&'static str>> {
        let (next, responses) = match (self.stage, opcode) {
            (Stage::Login, 1) => {
                if body.len() < 4 || body[0] != 0x5e || body[1] != 0x5b {
                    return Err(io::Error::other("unsupported diagnostic login branch"));
                }
                let size = read_uint(&body[2..4], Width::U16).map_err(io::Error::other)? as usize;
                if !(1..=64).contains(&size) || body.len() != 4 + size + 25 {
                    return Err(io::Error::other("unsupported diagnostic login size"));
                }
                (Stage::Info, vec!["auth_A"])
            }
            (Stage::Info, 9) if body.len() == 8 => (Stage::Redirect, vec!["netinfo", "flags"]),
            (Stage::Redirect, 5) if body.len() == 9 => (Stage::LoginDone, vec!["redirect"]),
            (Stage::Channel, 6) => {
                if body.len() != 36 + self.name.len()
                    || read_uint(&body[2..4], Width::U16).map_err(io::Error::other)?
                        != self.name.len() as u64
                    || decode_bytes(&body[4..4 + self.name.len()])? != self.name
                    || read_uint(&body[4 + self.name.len()..8 + self.name.len()], Width::U32)
                        .map_err(io::Error::other)?
                        != u64::from(self.identity)
                {
                    return Err(io::Error::other("diagnostic channel identity mismatch"));
                }
                (Stage::Ready, vec!["channel"])
            }
            (Stage::Ready, 67) if body.len() == 2 => (
                Stage::Lobby,
                vec!["ready", "roomlist_A", "channel_list", "own291"],
            ),
            _ if matches!(
                (opcode, body.len()),
                (60, 20)
                    | (15, 31)
                    | (744, 4)
                    | (745, 0)
                    | (40, 1)
                    | (544, 10)
                    | (549, 3)
                    | (567, 0)
                    | (742, 0)
                    | (741, 0)
                    | (715, 0)
            ) =>
            {
                self.ignored += 1;
                if self.ignored > 256 {
                    return Err(io::Error::other("diagnostic ancillary budget"));
                }
                return Ok(vec![]);
            }
            _ => {
                return Err(io::Error::other(
                    "diagnostic lobby wrong stage/opcode/shape",
                ));
            }
        };
        self.stage = next;
        Ok(responses)
    }
}

pub(crate) fn decode_bytes(mut encoded: &[u8]) -> io::Result<Vec<u8>> {
    let mut out = Vec::with_capacity(encoded.len());
    while !encoded.is_empty() {
        let (n, w) = if encoded.len() >= 4 {
            (4, Width::U32)
        } else if encoded.len() >= 2 {
            (2, Width::U16)
        } else {
            (1, Width::U8)
        };
        let value = read_uint(&encoded[..n], w).map_err(io::Error::other)?;
        out.extend_from_slice(&value.to_le_bytes()[..n]);
        encoded = &encoded[n..];
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use kr_wire::write_uint;
    #[test]
    fn wrong_stage_never_advances_and_duplicate_login_is_not_accepted() {
        let mut flow = Flow::login();
        assert!(flow.packet(9, &[0; 8]).is_err());
        assert_eq!(flow.stage, Stage::Login);
        let mut input = vec![0x5e, 0x5b];
        input.extend(write_uint(13, Width::U16).unwrap());
        input.extend([0; 38]);
        assert_eq!(flow.packet(1, &input).unwrap(), vec!["auth_A"]);
        assert_eq!(flow.stage, Stage::Info);
        assert!(flow.packet(1, &input).is_err());
        assert_eq!(flow.stage, Stage::Info);
        assert!(flow.packet(9, &[0; 7]).is_err());
        assert_eq!(flow.stage, Stage::Info);
        assert_eq!(flow.packet(9, &[0; 8]).unwrap(), vec!["netinfo", "flags"]);
        assert_eq!(flow.packet(5, &[0; 9]).unwrap(), vec!["redirect"]);
        assert_eq!(flow.stage, Stage::LoginDone);
    }
    #[test]
    fn diagnostic_channel_binding_rejects_another_name_or_identity() {
        let mut body = vec![0; 43];
        body[2..4].copy_from_slice(&write_uint(7, Width::U16).unwrap());
        let first = u32::from_le_bytes(*b"KRLo");
        body[4..8].copy_from_slice(&write_uint(first.into(), Width::U32).unwrap());
        body[8..10]
            .copy_from_slice(&write_uint(u16::from_le_bytes(*b"ca").into(), Width::U16).unwrap());
        body[10] = write_uint(b'l'.into(), Width::U8).unwrap()[0];
        body[11..15].copy_from_slice(&write_uint(2, Width::U32).unwrap());
        let mut flow = Flow::channel();
        assert!(flow.packet(6, &body).is_err());
        assert_eq!(flow.stage, Stage::Channel);
        body[11..15].copy_from_slice(&write_uint(1, Width::U32).unwrap());
        assert_eq!(flow.packet(6, &body).unwrap(), vec!["channel"]);
        assert!(flow.packet(67, &[]).is_err());
        assert_eq!(flow.stage, Stage::Ready);
        assert_eq!(
            flow.packet(67, &[0; 2]).unwrap(),
            vec!["ready", "roomlist_A", "channel_list", "own291"]
        );
        assert!(flow.packet(67, &[0; 2]).is_err());
        assert_eq!(flow.stage, Stage::Lobby);
    }

    #[test]
    fn guest_identity_is_distinct_and_cannot_claim_host_channel() {
        let mut body = vec![0; 43];
        body[2..4].copy_from_slice(&write_uint(7, Width::U16).unwrap());
        body[4..8]
            .copy_from_slice(&write_uint(u32::from_le_bytes(*b"KRGu").into(), Width::U32).unwrap());
        body[8..10]
            .copy_from_slice(&write_uint(u16::from_le_bytes(*b"es").into(), Width::U16).unwrap());
        body[10] = write_uint(b't'.into(), Width::U8).unwrap()[0];
        body[11..15].copy_from_slice(&write_uint(2, Width::U32).unwrap());
        let mut host = Flow::channel();
        assert!(host.packet(6, &body).is_err());
        assert_eq!(host.stage, Stage::Channel);
        let mut guest = Flow::for_identity(true, 2, b"KRGuest");
        assert_eq!(guest.packet(6, &body).unwrap(), vec!["channel"]);
        assert_eq!(guest.stage, Stage::Ready);
    }
}
