//! CP226/227 room selection carriers. These bytes identify a requested public
//! tuple; only the service can authorize its account, room, membership and epoch.
use super::{encode_bytes, put, uint};
use crate::Width;
use std::io;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct JoinSelection {
    pub session: u16,
    pub room_id: u32,
}

/// Exact measured 82 branch: u16 session, u32 room ID, then opaque 0/1/5.
/// Session zero was observed in CP028. Registry admission policy is separate.
/// Nonzero opaque fields and other branches remain unsupported, not normalized.
pub fn join82_selection(body: &[u8]) -> io::Result<JoinSelection> {
    if body.len() != 11
        || uint(&body[6..8], Width::U16)? != 0
        || uint(&body[8..9], Width::U8)? != 1
        || uint(&body[9..11], Width::U16)? != 5
    {
        return Err(io::Error::other("unsupported measured join82 branch"));
    }
    Ok(JoinSelection {
        session: uint(&body[..2], Width::U16)? as u16,
        room_id: uint(&body[2..6], Width::U32)? as u32,
    })
}

/// CP017 successful create86: only the measured session field is projected.
/// The following u32 and remaining bytes stay opaque; it is not a room ID echo.
pub fn create86_session(template: &[u8], session: u16) -> io::Result<Vec<u8>> {
    if template.len() != 11 || uint(&template[..1], Width::U8)? != 0 {
        return Err(io::Error::other("unsupported successful create86"));
    }
    let mut out = template.to_vec();
    out[1..3].copy_from_slice(&put(u64::from(session), Width::U16)?);
    Ok(out)
}

/// CP227 successful87 binds session and slot, with the existing opaque tail.
/// Client derives the selected room ID/title from its list row, not this body.
pub fn join87_session(template: &[u8], slot: u8, session: u16) -> io::Result<Vec<u8>> {
    if slot >= 26 || template.len() != 7 || uint(&template[..1], Width::U8)? != 0 {
        return Err(io::Error::other("unsupported successful join87"));
    }
    let mut out = super::room::join87(template, slot)?;
    out[2..4].copy_from_slice(&put(u64::from(session), Width::U16)?);
    Ok(out)
}

/// CP226 measured deletion branch only: count1=0, count2=1, tag0/session/ID.
/// A zero-row91 did not clear the client list; deletion needs this guarded path.
/// Sending it does not retire the server room or prove stale82 rejection.
pub fn removal90_one(selection: JoinSelection) -> io::Result<Vec<u8>> {
    let mut inner = vec![0];
    inner.extend(selection.session.to_le_bytes());
    inner.extend(selection.room_id.to_le_bytes());
    let mut stream = vec![0x18];
    stream.extend(8u32.to_le_bytes());
    stream.push(0);
    stream.extend(inner);
    let mut body = put(0, Width::U16)?;
    body.extend(put(1, Width::U16)?);
    body.extend(put(stream.len() as u64, Width::U32)?);
    body.extend(encode_bytes(&stream)?);
    Ok(body)
}

#[cfg(test)]
mod tests {
    use super::*;
    // CP226 actual client82 and CP227 native-accepted87 inputs; provenance in
    // local/research/codex/m7-room-source-r1/wire-vectors.json. No Rust encoder
    // generated these expected bytes. Captures remain local.
    const JOIN_A: &[u8] = &[165, 104, 150, 202, 83, 240, 165, 105, 91, 165, 108];
    const JOIN_B: &[u8] = &[165, 107, 150, 202, 83, 95, 165, 105, 91, 165, 108];
    const REPLY_A: &[u8] = &[90, 90, 165, 104, 90, 90, 90];
    const REPLY_B: &[u8] = &[90, 91, 165, 107, 90, 90, 90];
    const DELETE: &[u8] = &[
        165, 105, 165, 104, 150, 202, 83, 152, 150, 202, 91, 141, 148, 202, 83, 149, 150, 202, 153,
        149, 90,
    ];

    #[test]
    fn captured_selection_tuple_is_preserved_and_not_a_fixed_room_alias() {
        assert_eq!(
            join82_selection(JOIN_A).unwrap(),
            JoinSelection {
                session: 1,
                room_id: 101
            }
        );
        assert_eq!(
            join82_selection(JOIN_B).unwrap(),
            JoinSelection {
                session: 2,
                room_id: 202
            }
        );
        for length in 0..JOIN_A.len() {
            assert!(join82_selection(&JOIN_A[..length]).is_err());
        }
        let mut trailing = JOIN_A.to_vec();
        trailing.push(0);
        assert!(join82_selection(&trailing).is_err());
        for offset in 6..11 {
            let mut malformed = JOIN_A.to_vec();
            malformed[offset] ^= 1;
            assert!(join82_selection(&malformed).is_err());
        }
    }
    #[test]
    fn successful_reply_projects_the_native_measured_session_and_slot_only() {
        assert_eq!(join87_session(REPLY_A, 1, 2).unwrap(), REPLY_B);
        assert_eq!(join87_session(REPLY_B, 0, 1).unwrap(), REPLY_A);
        assert!(join87_session(&REPLY_A[..6], 0, 1).is_err());
        assert!(join87_session(REPLY_A, 26, 1).is_err());
        assert!(join87_session(&[0x58], 0, 1).is_err());
        // CP017 measured successful86, preserving the still-opaque u32/tail.
        let baseline = [
            0x5a, 0xa5, 0x68, 0x96, 0xca, 0x53, 0x95, 0x5a, 0x5a, 0x5a, 0x5a,
        ];
        let projected = create86_session(&baseline, 2).unwrap();
        assert_eq!(uint(&projected[1..3], Width::U16).unwrap(), 2);
        assert_eq!(&projected[3..], &baseline[3..]);
        assert_eq!(create86_session(&baseline, 1).unwrap(), baseline);
    }
    #[test]
    fn guarded_deletion_matches_the_independently_native_accepted_input() {
        assert_eq!(
            removal90_one(JoinSelection {
                session: 2,
                room_id: 202
            })
            .unwrap(),
            DELETE
        );
    }
}
