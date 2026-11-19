// Source: CP283 retained KR native-accepted zero-profile templates. These are
// independent fixture bytes; generated output alone is not a native oracle.
use kr_wire::{
    Width,
    kr00967::{AccountTemplate, PlayerTemplate},
    read_uint,
};
fn hex(s: &str) -> Vec<u8> {
    s.as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|p| u8::from_str_radix(std::str::from_utf8(p).unwrap(), 16).unwrap())
        .collect()
}
const AUTH: &str = "5a5aa5685b0000000071b1858996ca539596ca5395a569a56996ca5394a56ef98601dec40a365aa569a5695a0000000071b1858996ca539596ca53955a96ca53955a5aa5695a5a96ca539596ca53955a5a5a5a5aa569a5695a96ca539596ca53955a5a96ca53955a5a96ca53955a5aa56996ca53955a5a5a5aa5695a5a96ca539596ca539596ca5395";
const OWN: &str = "5b96ca53e696ca3d8d96cd5395c4815395f7a93cd996ca53f996ca539596ca539596ca539596ca539596ca539596ca539596ca539596ca539596ca539596ca539596ca539596ca539596ca539596ca539596ca539596ca539596ca539596ca539596ca539596ca539596ca539596ca539596ca5395a5695a96ca53955a5a5a5a";
const PLAYER: &str = "5a96ca53945a5a5a96ca539596ca539796ca53e696ca3d8d96cd5395c4815395f7a93cd996ca53f996ca539596ca539596ca539596ca539596ca539596ca539596ca539596ca539596ca539596ca539596ca539596ca539596ca539596ca539596ca539596ca539596ca539596ca539596ca539596ca539596ca539596ca539596ca5395a5695a5a5a5aa56ef98601dec40a36a56996ca539396ca528da5695a96ca539596ca539596ca539596ca5395a5695a5aa55a96ca53955a96ca539596ca53955a96ca5395a569a569";
#[test]
fn projection_preserves_independent_native_templates_and_rejects_disagreement() {
    let (a, o, p) = (hex(AUTH), hex(OWN), hex(PLAYER));
    let template = AccountTemplate::parse(&a, &o, &p).unwrap();
    let got = template.project(1, b"KRLocal", 0).unwrap();
    assert_eq!(got.auth, a);
    assert_eq!(got.own291, o);
    assert_eq!(got.player106, p);
    let mut bad = o.clone();
    bad[10] ^= 1;
    assert!(AccountTemplate::parse(&a, &bad, &p).is_err());
    assert!(AccountTemplate::parse(&a, &o, &[]).is_err());
    for name in [
        b"ab".as_slice(),
        b"sixteen_chars_xx",
        b"bad space",
        b"bad!",
        b"bad\x01",
    ] {
        assert!(template.project(7, name, 1).is_err());
    }
    assert!(template.project(0, b"Valid_1", 1).is_err());
}
#[test]
fn candidate_new_names_and_wire_ids_are_consistent_across_all_three_packets() {
    let t = AccountTemplate::parse(&hex(AUTH), &hex(OWN), &hex(PLAYER)).unwrap();
    for name in [b"Ab3".as_slice(), b"NewPlayer9", b"Fifteen_chars99"] {
        let q = t.project(70000, name, 0x12345678).unwrap();
        let n = name.len();
        assert_eq!(q.auth.len(), 130 + n);
        assert_eq!(read_uint(&q.auth[25..29], Width::U32).unwrap(), 70000);
        assert_eq!(
            read_uint(&q.auth[86 + n..90 + n], Width::U32).unwrap(),
            0x12345678
        );
        assert_eq!(q.own291.len(), 121 + n);
        assert_eq!(q.player106.len(), 190 + 2 * n);
        let p = PlayerTemplate::parse_account(&q.player106).unwrap();
        assert_eq!(p.project_account(70000, name).unwrap(), q.player106);
    }
}

#[test]
fn patched_names_use_native_lengths_and_identical_bytes_in_all_projections() {
    use kr_wire::kr00967::{decode_bytes, text949x};
    let t = AccountTemplate::parse(&hex(AUTH), &hex(OWN), &hex(PLAYER)).unwrap();
    for text in ["Việt01", "한국02", "Việt한국", "ệệệệệệệ"] {
        let name = text949x::encode_name(text).unwrap();
        let n = name.len();
        let q = t.project(70000, &name, 0x12345678).unwrap();
        assert_eq!(read_uint(&q.auth[29..31], Width::U16).unwrap() as usize, n);
        assert_eq!(decode_bytes(&q.auth[31..31 + n]).unwrap(), name);
        assert_eq!(
            read_uint(&q.auth[86 + n..90 + n], Width::U32).unwrap(),
            0x12345678
        );
        let own = decode_bytes(&q.own291[5..5 + 108 + n]).unwrap();
        assert_eq!(
            u32::from_le_bytes(own[6..10].try_into().unwrap()) as usize,
            n
        );
        assert_eq!(&own[10..10 + n], &name);
        let profile = decode_bytes(&q.player106[20..20 + 108 + n]).unwrap();
        assert_eq!(profile, own);
        let outer = 20 + 108 + n + 3;
        assert_eq!(
            read_uint(&q.player106[outer..outer + 2], Width::U16).unwrap() as usize,
            n
        );
        assert_eq!(
            decode_bytes(&q.player106[outer + 2..outer + 2 + n]).unwrap(),
            name
        );
        let parsed = PlayerTemplate::parse_account(&q.player106).unwrap();
        assert_eq!(parsed.project_account(70000, &name).unwrap(), q.player106);
    }
}

// r55: synthetic count2 profile was consumed242/242, errors54/58=0,
// native local slots0/1 set IDs1/2. Retained input, not a generated Rust golden.
const LOCAL_TWO_AUTH: &str = "5a5aa568580000000071b1858996ca539596ca5395a569a56996ca5394a56ef98601dec40a365aa569a5695a0000000071b1858996ca539596ca53955a96ca53955a5aa5695a5a96ca539596ca53955a5a5a5a5aa569a5695a96ca539596ca53955a5a96ca53955a5a96ca53955a5aa56996ca53955a96ca539596ca5395a569a56996ca5397a56ee38d01ded60c2e5aa569a5695a0000000071b1858996ca539596ca53955a96ca53955a5aa5695a5a96ca539596ca53955a5a5a5a5aa569a5695a96ca539596ca53955a5a96ca53955a5a96ca53955a5aa56996ca53955a5a5a5aa5695a5a96ca539596ca539596ca5395";

#[test]
fn two_local_auth_matches_independently_native_accepted_profile() {
    let template = AccountTemplate::parse(&hex(AUTH), &hex(OWN), &hex(PLAYER)).unwrap();
    assert_eq!(
        template
            .project_local_auth(&[(1, b"KRLocal", 0), (2, b"KRGuest", 0)])
            .unwrap(),
        hex(LOCAL_TWO_AUTH)
    );
    assert_eq!(
        template.project_local_auth(&[(1, b"KRLocal", 0)]).unwrap(),
        hex(AUTH)
    );
    assert!(template.project_local_auth(&[]).is_err());
    assert!(
        template
            .project_local_auth(&[(1, b"KRLocal", 0), (1, b"KRGuest", 0)])
            .is_err()
    );
    assert!(
        template
            .project_local_auth(&[(1, b"KRLocal", 0), (2, b"KRGuest", 0), (3, b"KRNew", 0)])
            .is_err()
    );
    assert!(
        template
            .project_local_auth(&[(1, b"KRLocal", 0), (0, b"KRGuest", 0)])
            .is_err()
    );
}

#[test]
fn local_own_profiles_keep_independent_record_boundaries_and_slot_order() {
    use kr_wire::kr00967::{decode_bytes, local_own291};
    assert_eq!(local_own291(&[b"KRLocal"]).unwrap(), hex(OWN));
    let body = local_own291(&[b"KRLocal", b"GuestTwo"]).unwrap();
    assert_eq!(&body[..128], hex(OWN));
    let second = &body[128..];
    assert_eq!(second.len(), 129);
    assert_eq!(read_uint(&second[..1], Width::U8).unwrap(), 1);
    assert_eq!(read_uint(&second[1..5], Width::U32).unwrap(), 116);
    let blob = decode_bytes(&second[5..121]).unwrap();
    assert_eq!(&blob[10..18], b"GuestTwo");
    assert!(local_own291(&[]).is_err());
    assert!(local_own291(&[b"KRLocal", b"GuestTwo", b"Third"]).is_err());
}
