//! CP341 native manual/trial requests. Parsing never authenticates an account.
//!
//! The caller must admit this route only inside the companion TLS tunnel and
//! erase its input buffer after parsing. KR string obfuscation provides no
//! credential protection. Length/ASCII bounds here are company admission
//! policy, not recovered maximum lengths of the native edit controls.
use super::{Error, Result};
use kr_wire::{Width, crypt, read_uint};
use std::net::Ipv4Addr;
use zeroize::Zeroizing;

const LOGIN_LIMIT: usize = 32;
const PASSWORD_LIMIT: usize = 128;
const CREDENTIAL_TAIL: usize = 25;
const TRIAL_TAIL: usize = 24;

/// No Debug, Clone or serialization: credentials must not become diagnostics.
pub struct Credentials {
    login: Zeroizing<String>,
    password: Zeroizing<Vec<u8>>,
}
impl Credentials {
    pub fn login(&self) -> &str {
        &self.login
    }
    pub fn password(&self) -> &[u8] {
        &self.password
    }
}

/// Reported metadata is untrusted and must never choose account or transport.
pub struct Request {
    pub players: Players,
    pub reported_address: Ipv4Addr,
}
pub enum Players {
    Accounts(Vec<Credentials>),
    Trial { count: u8 },
}

fn field(bytes: &[u8], width: Width) -> Result<u64> {
    read_uint(bytes, width).map_err(|_| Error::UnsupportedWire)
}

fn string(
    records: &[u8],
    cursor: &mut usize,
    limit: usize,
    key: u32,
) -> Result<Zeroizing<Vec<u8>>> {
    let end = cursor.checked_add(2).ok_or(Error::UnsupportedWire)?;
    let length = field(
        records.get(*cursor..end).ok_or(Error::UnsupportedWire)?,
        Width::U16,
    )? as usize;
    if !(1..=limit).contains(&length) {
        return Err(Error::UnsupportedWire);
    }
    let next = end.checked_add(length).ok_or(Error::UnsupportedWire)?;
    let encoded = records.get(end..next).ok_or(Error::UnsupportedWire)?;
    let decoded = Zeroizing::new(crypt(encoded, key, true));
    *cursor = next;
    Ok(decoded)
}

/// Narrow CP341 count1/2 manual opcode1 and trial opcode62 profiles.
/// Password policy/KDF, duplicate principals, rate limits and multi-account
/// publication belong to the backend. A valid packet grants no authority.
pub fn parse(opcode: u32, body: &[u8]) -> Result<Request> {
    let (players, tail) = match opcode {
        1 => {
            // Two players, each at most two bounded strings, plus exact tail.
            const MAX: usize = 2 + 2 * (4 + LOGIN_LIMIT + PASSWORD_LIMIT) + CREDENTIAL_TAIL;
            if !(2 + CREDENTIAL_TAIL..=MAX).contains(&body.len())
                || field(&body[..1], Width::U8)? != 0
            {
                return Err(Error::UnsupportedWire);
            }
            let count = field(&body[1..2], Width::U8)? as usize;
            if !(1..=2).contains(&count) {
                return Err(Error::UnsupportedWire);
            }
            let records_end = body.len() - CREDENTIAL_TAIL;
            let records = &body[2..records_end];
            let mut cursor = 0;
            let mut credentials = Vec::with_capacity(count);
            for _ in 0..count {
                let login = string(records, &mut cursor, LOGIN_LIMIT, 0x1122_3344)?;
                if !login
                    .iter()
                    .all(|c| c.is_ascii_alphanumeric() || *c == b'_')
                {
                    return Err(Error::UnsupportedWire);
                }
                let login = Zeroizing::new(
                    std::str::from_utf8(&login)
                        .map_err(|_| Error::UnsupportedWire)?
                        .to_owned(),
                );
                let password = string(records, &mut cursor, PASSWORD_LIMIT, 0x4433_2211)?;
                if !password.iter().all(|c| (32..=126).contains(c)) {
                    return Err(Error::UnsupportedWire);
                }
                credentials.push(Credentials { login, password });
            }
            if cursor != records.len() || field(&body[body.len() - 1..], Width::U8)? != 0 {
                return Err(Error::UnsupportedWire);
            }
            (
                Players::Accounts(credentials),
                &body[records_end..body.len() - 1],
            )
        }
        62 if body.len() == 1 + TRIAL_TAIL => {
            let count = field(&body[..1], Width::U8)? as u8;
            if !(1..=2).contains(&count) {
                return Err(Error::UnsupportedWire);
            }
            (Players::Trial { count }, &body[1..])
        }
        _ => return Err(Error::UnsupportedWire),
    };
    // First16 bytes remain opaque. The measured zero is a profile guard only.
    if field(&tail[16..20], Width::U32)? != 0 {
        return Err(Error::UnsupportedWire);
    }
    let reported = field(&tail[20..24], Width::U32)? as u32;
    Ok(Request {
        players,
        reported_address: Ipv4Addr::from(reported.to_le_bytes()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use kr_wire::write_uint;

    fn hex(text: &str) -> Vec<u8> {
        text.as_bytes()
            .as_chunks::<2>()
            .0
            .iter()
            .map(|b| u8::from_str_radix(std::str::from_utf8(b).unwrap(), 16).unwrap())
            .collect()
    }
    fn tail() -> Vec<u8> {
        // Opaque/machine fields are synthetic, never copied from a user's VM.
        let mut out = vec![0; 16];
        out.extend(write_uint(0, Width::U32).unwrap());
        out.extend(write_uint(u32::from_le_bytes([192, 0, 2, 1]).into(), Width::U32).unwrap());
        out
    }
    fn captured_records(count: u8) -> Vec<u8> {
        // CP341 actual native record bytes for declared synthetic inputs.
        // Capture SHA/recipe is in m8-login-stage-manual.json. Tail is synthetic.
        let a = "a56195164730d7db79eda561c007567694d95be3";
        let b = "a56195164730d7db79eea561c007567694d95be0";
        let mut body = write_uint(0, Width::U8).unwrap();
        body.extend(write_uint(count.into(), Width::U8).unwrap());
        body.extend(hex(a));
        if count == 2 {
            body.extend(hex(b));
        }
        body.extend(tail());
        body.extend(write_uint(0, Width::U8).unwrap());
        body
    }

    #[test]
    fn native_records_decode_exactly_for_one_and_two_accounts() {
        for count in [1, 2] {
            let request = parse(1, &captured_records(count)).unwrap();
            assert_eq!(request.reported_address, Ipv4Addr::new(192, 0, 2, 1));
            let Players::Accounts(players) = request.players else {
                panic!("manual route")
            };
            assert_eq!(players.len(), count as usize);
            assert_eq!(players[0].login(), "M8ProbeA");
            assert_eq!(players[0].password(), b"M8Pass01");
            if count == 2 {
                assert_eq!(players[1].login(), "M8ProbeB");
                assert_eq!(players[1].password(), b"M8Pass02");
            }
        }
    }

    #[test]
    fn native_trial_count_is_distinct_from_account_credentials() {
        for count in [1u8, 2] {
            let mut body = write_uint(count.into(), Width::U8).unwrap();
            body.extend(tail());
            assert!(
                matches!(parse(62, &body).unwrap().players, Players::Trial { count: n } if n == count)
            );
            assert!(parse(1, &body).is_err());
        }
    }

    #[test]
    fn malformed_or_other_auth_branches_cannot_borrow_manual_route() {
        let body = captured_records(2);
        for cut in 0..body.len() {
            assert!(parse(1, &body[..cut]).is_err(), "truncation {cut}");
        }
        for count in [0, 3, 255] {
            let mut changed = body.clone();
            changed[1] = write_uint(count, Width::U8).unwrap()[0];
            assert!(parse(1, &changed).is_err());
        }
        for tag in [1, 4, 255] {
            let mut changed = body.clone();
            changed[0] = write_uint(tag, Width::U8).unwrap()[0];
            assert!(parse(1, &changed).is_err());
        }
        for length in [0, 33, 65535] {
            let mut changed = body.clone();
            changed[2..4].copy_from_slice(&write_uint(length, Width::U16).unwrap());
            assert!(parse(1, &changed).is_err());
        }
        for length in [0, 129, 65535] {
            let mut changed = body.clone();
            changed[12..14].copy_from_slice(&write_uint(length, Width::U16).unwrap());
            assert!(parse(1, &changed).is_err());
        }
        let mut control_character = body.clone();
        control_character[14..22].copy_from_slice(&crypt(b"bad\0pass", 0x4433_2211, false));
        assert!(parse(1, &control_character).is_err());
        let mut bad_login = body.clone();
        bad_login[4..12].copy_from_slice(&crypt(b"bad name", 0x1122_3344, false));
        assert!(parse(1, &bad_login).is_err());
        let mut extra = body.clone();
        extra.insert(2, 0);
        assert!(parse(1, &extra).is_err());
        let mut changed = body.clone();
        let n = changed.len();
        changed[n - 9..n - 5].copy_from_slice(&write_uint(1, Width::U32).unwrap());
        assert!(parse(1, &changed).is_err());
        changed = body;
        let n = changed.len();
        changed[n - 1] = write_uint(1, Width::U8).unwrap()[0];
        assert!(parse(1, &changed).is_err());
    }
}
