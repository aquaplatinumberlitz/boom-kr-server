//! Independent actual119 bodies from g1-ability-flags-native-r2, two rounds.
//! The nine-byte 162 prefix was independently consumed in both views in
//! g1-ability-motion-source-native-r3 (9/9, errors 0, secure root changed).
//! This tests the codec only; r3 did not complete results/reset or next118.
use kr_wire::kr00967::{BombAction119, BombMotion162};

fn hex(s: &str) -> Vec<u8> {
    s.as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
        .collect()
}

#[test]
fn captured_requests_preserve_wire_id_coordinates_and_opaque_tail() {
    for (captured, id, row, opaque) in [
        ("5a5a59a5685c595f59a3daa568", 1, 6, 1715),
        ("5a5a59a56b52595f59a374a568", 2, 8, 1565),
    ] {
        let body = hex(captured);
        let decoded = BombAction119::decode(&body).unwrap();
        assert_eq!(
            decoded.motion,
            BombMotion162 {
                context: 0,
                slot: 0,
                selector: 3,
                bomb_id: id,
                source_row: row,
                source_column: 3,
                target_row: 5,
                target_column: 3,
            }
        );
        assert_eq!(decoded.opaque_tail, [opaque, 1]);
    }
}

#[test]
fn captured_selector1_request_preserves_down_one_without_claiming_native_execution() {
    // CP327 Rust native R3 round2, C2S sequence352. The service refused it;
    // this independent capture establishes emission/codec, not a consumed162.
    let decoded = BombAction119::decode(&hex("5a5a5ba56c5c595d59a3daa568")).unwrap();
    assert_eq!(decoded.motion.selector, 1);
    assert_eq!(decoded.motion.bomb_id, 5);
    assert_eq!(
        (
            decoded.motion.source_row,
            decoded.motion.source_column,
            decoded.motion.target_row,
            decoded.motion.target_column
        ),
        (6, 3, 7, 3)
    );
    assert_eq!(decoded.opaque_tail, [1715, 1]);
}

#[test]
fn native_accepted_motion_body_has_its_own_exact_boundary() {
    let captured = hex("5a5a59a5685c595f59");
    let decoded = BombMotion162::decode(&captured).unwrap();
    assert_eq!(decoded.bomb_id, 1);
    assert_eq!((decoded.source_row, decoded.target_row), (6, 5));
    assert_eq!(decoded.encode().unwrap(), captured);
    let request = hex("5a5a59a5685c595f59a3daa568");
    for n in 0..request.len() {
        assert!(BombAction119::decode(&request[..n]).is_err());
    }
    for n in 0..captured.len() {
        assert!(BombMotion162::decode(&captured[..n]).is_err());
    }
    assert!(BombMotion162::decode(&request).is_err());
    let mut trailing = request;
    trailing.push(0);
    assert!(BombAction119::decode(&trailing).is_err());
    let mut trailing = captured;
    trailing.push(0);
    assert!(BombMotion162::decode(&trailing).is_err());
}

#[test]
fn captured_short158_rebind_is_distinct_from_thirteen_byte_explosion() {
    // CP325 R11: these exact bodies were consumed4/4, errors0 by both clients.
    for (id, captured) in [(1, "5ba5685b"), (3, "5ba56a5b")] {
        assert_eq!(kr_wire::kr00967::rebind158(id).unwrap(), hex(captured));
        assert_eq!(kr_wire::kr00967::explode158(id).unwrap().len(), 13);
    }
    assert!(kr_wire::kr00967::rebind158(0).is_err());
}
