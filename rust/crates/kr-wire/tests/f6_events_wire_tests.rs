use kr_wire::CodecError;
use kr_wire::kr00967::f6_events::{
    C_GAME_STAGE_JUMP_TABLE_VAS, F6_EVENT_ANCHORS, F6EventWireStatus, F6StageKind,
    Lobby418PrefixCandidate, Stage553EventCandidate, f6_event_anchor,
};

#[test]
fn lookup_pins_p233_handlers_and_dispatch_tables() {
    assert_eq!(F6StageKind::GameStage.dispatcher_va(), 0xE5D3A0);
    assert_eq!(F6StageKind::LobbyStage.dispatcher_va(), 0xD68C90);
    assert_eq!(C_GAME_STAGE_JUMP_TABLE_VAS, [0xE5DF9C, 0xE5E0DC]);
    // All 18 slots are accounted for by the two stage allowlists below.
    assert_eq!(F6_EVENT_ANCHORS.len(), 18);
    for (opcode, handler) in [
        (321, 0xE6AA90),
        (322, 0xE6AEB0),
        (323, 0xE6AEB0),
        (548, 0xE69440),
        (549, 0xE69D30),
        (553, 0xE69D80),
        (566, 0xE6D830),
        (568, 0xE75C90),
        (569, 0xE76ED0),
        (574, 0xE75CB0),
        (576, 0xE763A0),
        (577, 0xE760B0),
    ] {
        let anchor = f6_event_anchor(F6StageKind::GameStage, opcode).unwrap();
        assert_eq!(anchor.handler_va, handler);
        assert_eq!(anchor.jump_table_va, Some(0xE5DF9C));
        assert_eq!(f6_event_anchor(F6StageKind::LobbyStage, opcode), None);
    }
    for (opcode, handler) in [
        (408, 0xD7B2B0),
        (409, 0xD7B5D0),
        (413, 0xD7B2B0),
        (414, 0xD7C2C0),
        (418, 0xD852C0),
        (419, 0xD852C0),
    ] {
        let anchor = f6_event_anchor(F6StageKind::LobbyStage, opcode).unwrap();
        assert_eq!(anchor.handler_va, handler);
        assert_eq!(anchor.jump_table_va, None);
        assert_eq!(f6_event_anchor(F6StageKind::GameStage, opcode), None);
    }
    assert_eq!(f6_event_anchor(F6StageKind::GameStage, 174), None);
}

#[test]
fn statuses_keep_unmeasured_and_candidate_entries_separate() {
    for anchor in F6_EVENT_ANCHORS {
        let expected = match (anchor.stage, anchor.opcode) {
            (F6StageKind::GameStage, 553) | (F6StageKind::LobbyStage, 418) => {
                F6EventWireStatus::StaticCandidatePendingCapture
            }
            _ => F6EventWireStatus::UnmeasuredPendingCapture,
        };
        assert_eq!(anchor.status, expected, "opcode {}", anchor.opcode);
    }
}

// Synthetic vectors exercise the KR primitive masks and reader order.
#[test]
fn stage_553_round_trips_exact_four_byte_candidate() {
    let value = Stage553EventCandidate {
        flag_a: 0x12,
        value: 0x3456,
        flag_b: 0x78,
    };
    let encoded = value.encode().unwrap();
    assert_eq!(encoded, [0x48, 0x91, 0x3f, 0x22]);
    assert_eq!(Stage553EventCandidate::decode(&encoded), Ok(value));
    assert_eq!(Stage553EventCandidate::OPCODE, 553);
    assert_eq!(
        Stage553EventCandidate::STATUS,
        F6EventWireStatus::StaticCandidatePendingCapture
    );
    assert_exact(&encoded, Stage553EventCandidate::decode);
}

#[test]
fn lobby_418_round_trips_exact_seven_byte_prefix() {
    let value = Lobby418PrefixCandidate {
        kind: 0x12,
        param: 0x3456,
        value: 0x789A_BCDE,
    };
    let encoded = value.encode_prefix().unwrap();
    assert_eq!(encoded, [0x48, 0x91, 0x3f, 0xee, 0x50, 0xef, 0x4b]);
    assert_eq!(Lobby418PrefixCandidate::decode_prefix(&encoded), Ok(value));
    assert_eq!(Lobby418PrefixCandidate::OPCODE, 418);
    assert_eq!(
        Lobby418PrefixCandidate::STATUS,
        F6EventWireStatus::StaticCandidatePendingCapture
    );
    assert_exact(&encoded, Lobby418PrefixCandidate::decode_prefix);
}

fn assert_exact<T>(body: &[u8], decode: impl Fn(&[u8]) -> Result<T, CodecError>) {
    for end in 0..body.len() {
        assert!(matches!(
            decode(&body[..end]),
            Err(CodecError::TruncatedEof)
        ));
    }
    let mut trailing = body.to_vec();
    trailing.push(0x5A);
    assert!(matches!(decode(&trailing), Err(CodecError::InvalidLength)));
}
