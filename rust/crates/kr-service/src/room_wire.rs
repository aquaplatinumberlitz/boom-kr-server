//! Thin membership adapters; bytes are built by kr-wire::kr00967.
use crate::room::Member;
use kr_wire::{Width, kr00967 as wire, read_uint};
use std::io;
pub fn player(template: &[u8], member: &Member, chief: bool) -> io::Result<Vec<u8>> {
    let body = wire::player106(
        template,
        member.key.identity,
        member.slot,
        member.team,
        member.ready,
        chief,
    )?;
    if let Some(character) = member.character {
        wire::player106_character(&body, character)
    } else {
        Ok(body)
    }
}
pub fn slot(template: &[u8], member: &Member, chief: bool) -> io::Result<Vec<u8>> {
    let p = player(template, member, chief)?;
    wire::slot110(
        read_uint(&p[..1], Width::U8).map_err(io::Error::other)? as u8,
        read_uint(&p[12..16], Width::U32).map_err(io::Error::other)? as u32,
    )
    .map_err(io::Error::other)
}
pub fn removal(member: &Member) -> io::Result<Vec<u8>> {
    wire::removal107(member.slot, member.key.identity).map_err(io::Error::other)
}
pub fn join(template: &[u8], member: &Member) -> io::Result<Vec<u8>> {
    wire::join87(template, member.slot)
}
#[cfg(test)]
mod tests {
    use super::*;
    fn put(v: u64, w: Width) -> io::Result<Vec<u8>> {
        kr_wire::write_uint(v, w).map_err(io::Error::other)
    }
    fn uint(b: &[u8], w: Width) -> io::Result<u64> {
        kr_wire::read_uint(b, w).map_err(io::Error::other)
    }
    #[test]
    fn projected_slot_is_decoded_before_crossing_the_typed_wire_builder_boundary() {
        let mut room = crate::room::Room::default();
        room.enter(1, 11, true).unwrap();
        let key = room.enter(2, 22, false).unwrap();
        let member = &room.members[&key.identity];
        let mut template = vec![0; 204];
        template[1..5].copy_from_slice(&put(2, Width::U32).unwrap());
        template[12..16].copy_from_slice(&put(0x12340000, Width::U32).unwrap());
        let body = slot(&template, member, false).unwrap();
        assert_eq!(body.len(), 6);
        assert_eq!(uint(&body[..1], Width::U8).unwrap(), 1);
        assert_eq!(uint(&body[1..5], Width::U32).unwrap(), 0x12340020);
        assert_eq!(uint(&body[5..], Width::U8).unwrap(), 0);
    }
}
