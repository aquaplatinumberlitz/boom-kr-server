use kr_wire::{
    Width,
    kr00967::f3_pve::{
        F3JumpTableProvenance, F3TimerField, F3WireError, F3WireStatus, MonsterAttackReqCandidate,
        ObjectEffectPairCandidate, TimedStageEventCandidate,
    },
    write_uint,
};

// Static test vectors for typed integer encoding and field order, not captures.
const ATTACK_BODY: [u8; 6] = [0x5b, 0x84, 0xfe, 0x05, 0xed, 0x78];
const EFFECT_BODY: [u8; 2] = [0x48, 0xa5];
const TIMED_BODY: [u8; 14] = [
    0x97, 0xc8, 0x50, 0x91, 0xa4, 0x96, 0xca, 0x53, 0xed, 0x96, 0xca, 0x53, 0x8b, 0x5b,
];

#[test]
fn all_candidates_mark_capture_as_pending_and_keep_dispatch_indices() {
    let status = F3WireStatus::StaticCandidatePendingCapture;
    assert_eq!(MonsterAttackReqCandidate::STATUS, status);
    let provenance = F3JumpTableProvenance::C_GAME_STAGE;
    assert_eq!(TimedStageEventCandidate::PROVENANCE, provenance);
    assert_eq!(ObjectEffectPairCandidate::PROVENANCE, provenance);
    assert_eq!(provenance.status, status);
    assert_eq!(provenance.on_packet_va, 0xE5D3A0);
    assert_eq!(provenance.jump_table_va, 0xE5DE9C);
    assert_eq!(provenance.opcode_index_subtrahend, 20);
    assert_eq!(provenance.hex_rays_cases, [9, 80, 135, 136, 177]);
}

#[test]
fn candidate_layouts_match_fixed_typed_integer_vectors() {
    let attack = MonsterAttackReqCandidate {
        host: true,
        attack_seq: 0x1234_5678,
        state: 34,
    };
    let effect = ObjectEffectPairCandidate {
        first: 0x12,
        second: 0xff,
    };
    let timed = TimedStageEventCandidate {
        first_number: 0x0102_0304,
        first_byte: 0xfe,
        first_timer_seconds: 120,
        second_timer_seconds: 30,
        flag: 1,
    };

    assert_eq!(attack.encode().unwrap(), ATTACK_BODY);
    assert_eq!(MonsterAttackReqCandidate::decode(&ATTACK_BODY), Ok(attack));
    assert_eq!(effect.encode().unwrap(), EFFECT_BODY);
    assert_eq!(ObjectEffectPairCandidate::decode(&EFFECT_BODY), Ok(effect));
    assert_eq!(timed.encode().unwrap(), TIMED_BODY);
    assert_eq!(TimedStageEventCandidate::decode(&TIMED_BODY), Ok(timed));
    assert_eq!(timed.first_timer_ms(), Ok(120_000));
    assert_eq!(timed.second_timer_ms(), Ok(30_000));
}

fn assert_exact<T: std::fmt::Debug>(body: &[u8], decode: impl Fn(&[u8]) -> Result<T, F3WireError>) {
    for end in 0..body.len() {
        assert_eq!(decode(&body[..end]).unwrap_err(), F3WireError::TruncatedEof);
    }
    let mut trailing = body.to_vec();
    trailing.push(0x5a);
    assert_eq!(decode(&trailing).unwrap_err(), F3WireError::TrailingBytes);
}

#[test]
fn all_candidates_reject_every_short_prefix_and_trailing_bytes() {
    assert_exact(&ATTACK_BODY, MonsterAttackReqCandidate::decode);
    assert_exact(&EFFECT_BODY, ObjectEffectPairCandidate::decode);
    assert_exact(&TIMED_BODY, TimedStageEventCandidate::decode);
}

#[test]
fn monster_attack_bounds_host_discriminants_and_states() {
    for byte in 0..=u8::MAX {
        let mut body = ATTACK_BODY;
        body[0] = byte ^ 0x5a;
        match byte {
            0 | 1 => assert_eq!(
                MonsterAttackReqCandidate::decode(&body).unwrap().host,
                byte == 1
            ),
            _ => assert_eq!(
                MonsterAttackReqCandidate::decode(&body),
                Err(F3WireError::InvalidHostFlag(byte))
            ),
        }

        let attack = MonsterAttackReqCandidate {
            host: false,
            attack_seq: u32::MAX,
            state: byte,
        };
        let body = [0x5a, 0x69, 0x35, 0xac, 0x6a, byte ^ 0x5a];
        if (34..=37).contains(&byte) {
            assert_eq!(attack.encode().unwrap(), body);
            assert_eq!(MonsterAttackReqCandidate::decode(&body), Ok(attack));
        } else {
            let error = F3WireError::InvalidAttackState(byte);
            assert_eq!(attack.encode(), Err(error));
            assert_eq!(MonsterAttackReqCandidate::decode(&body), Err(error));
        }
    }
}

#[test]
fn object_effect_preserves_opaque_byte_boundaries_and_order() {
    for (first, second) in [(0, 0), (0, 255), (255, 0), (255, 255)] {
        let effect = ObjectEffectPairCandidate { first, second };
        let body = [first ^ 0x5a, second ^ 0x5a];
        assert_eq!(effect.encode().unwrap(), body);
        assert_eq!(ObjectEffectPairCandidate::decode(&body), Ok(effect));
    }
}

#[test]
fn timed_stage_event_rejects_unsupported_flags_on_encode_and_decode() {
    let event = TimedStageEventCandidate::decode(&TIMED_BODY).unwrap();
    for flag in (0..=u8::MAX).filter(|&flag| flag != 1) {
        let mut body = TIMED_BODY;
        body[13] = flag ^ 0x5a;
        let error = F3WireError::UnsupportedTimedEventFlag(flag);
        assert_eq!(TimedStageEventCandidate::decode(&body), Err(error));
        assert_eq!(
            TimedStageEventCandidate { flag, ..event }.encode(),
            Err(error)
        );
    }
}

#[test]
fn timed_stage_event_checks_millisecond_overflow_for_both_fields() {
    let max_seconds = u32::MAX / 1000;
    let event = TimedStageEventCandidate {
        first_number: u32::MAX,
        first_byte: 0,
        first_timer_seconds: max_seconds,
        second_timer_seconds: max_seconds,
        flag: 1,
    };
    assert_eq!(event.first_timer_ms(), Ok(4_294_967_000));
    assert_eq!(event.second_timer_ms(), Ok(4_294_967_000));
    let valid_body = event.encode().unwrap();
    assert_eq!(TimedStageEventCandidate::decode(&valid_body), Ok(event));

    for too_large in [max_seconds + 1, u32::MAX] {
        let encoded = write_uint(too_large.into(), Width::U32).unwrap();
        let first = TimedStageEventCandidate {
            first_timer_seconds: too_large,
            second_timer_seconds: 0,
            ..event
        };
        let second = TimedStageEventCandidate {
            first_timer_seconds: 0,
            second_timer_seconds: too_large,
            ..event
        };
        let first_error = F3WireError::TimerOverflow(F3TimerField::First);
        let second_error = F3WireError::TimerOverflow(F3TimerField::Second);
        assert_eq!(first.first_timer_ms(), Err(first_error));
        assert_eq!(first.second_timer_ms(), Ok(0));
        assert_eq!(second.first_timer_ms(), Ok(0));
        assert_eq!(second.second_timer_ms(), Err(second_error));
        assert_eq!(first.encode(), Err(first_error));
        assert_eq!(second.encode(), Err(second_error));

        let mut body = valid_body.clone();
        body[5..9].copy_from_slice(&encoded);
        assert_eq!(TimedStageEventCandidate::decode(&body), Err(first_error));
        let mut body = valid_body.clone();
        body[9..13].copy_from_slice(&encoded);
        assert_eq!(TimedStageEventCandidate::decode(&body), Err(second_error));
    }
}
