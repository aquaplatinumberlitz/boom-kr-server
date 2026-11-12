//! Actual KR00967 character92/team97 producers, retained under M7c native runs.
//! Expected IDs come from independent UI/capture, not from this encoder.
use kr_wire::{Width, kr00967 as wire, read_uint, write_uint};

#[test]
fn retained_kr_choices_have_exact_widths_and_no_vn_u8_character_alias() {
    assert_eq!(wire::character92(&[0x5a, 0xa5, 0x68]).unwrap(), 1); // Dizni
    assert_eq!(wire::character92(&[0x5a, 0xa5, 0x6d]).unwrap(), 4); // Marid producer only
    assert_eq!(wire::team97(&[0x5a]).unwrap(), 0);
    for body in [&[][..], &[0x5a], &[0x5a, 0xa5], &[0x5a, 0xa5, 0x68, 0]] {
        assert!(wire::character92(body).is_err());
    }
    assert!(wire::character92(&[0x5b, 0xa5, 0x68]).is_err());
    assert!(wire::team97(&[0x5a, 0]).is_err());
    assert!(wire::team97(&[0x58]).is_err()); // decoded2, unsupported team
    assert!(wire::team97(&[0x4a]).is_err()); // decoded16, nonzero local player
}

#[test]
fn character_projection_preserves_unrelated_state_and_rejects_truncation() {
    let mut body = vec![0; 204];
    let prior = 0xe123_0024_u32;
    body[12..16].copy_from_slice(&write_uint(u64::from(prior), Width::U32).unwrap());
    let selected = wire::player106_character(&body, 1).unwrap();
    assert_eq!(
        read_uint(&selected[12..16], Width::U32).unwrap(),
        0xe001_0024
    );
    assert_eq!(&selected[..12], &body[..12]);
    assert_eq!(&selected[16..], &body[16..]);
    assert_eq!(
        read_uint(&body[12..16], Width::U32).unwrap(),
        u64::from(prior)
    );
    assert!(wire::player106_character(&body, 0x2000).is_err());
    assert!(wire::player106_character(&body[..199], 1).is_err());
}
