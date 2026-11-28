use kr_wire::{
    CodecError, FRAME_OVERHEAD, MAX_FRAME_LEN, Width,
    kr00967::f3_pve::{
        C_GAME_STAGE_JUMP_ENTRIES, F3JumpTableProvenance, F3MonActionProvenance, F3TimerField,
        F3WireError, F3WireStatus, MonActionDelegationCandidate, MonActionStep132Candidate,
        MonActionStep132Payload, MonActionStep180Candidate, MonActionStep180Payload,
        MonActionStepReportCandidate, MonsterAttackReqCandidate, ObjectEffectPairCandidate,
        TimedStageEventCandidate, f3_stage_jump_entry,
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
fn stage_jump_entries_preserve_case_and_wire_opcode_as_distinct_values() {
    let cases = [
        0, 5, 9, 64, 80, 90, 91, 103, 112, 119, 120, 121, 122, 132, 133, 134, 135, 136, 137, 138,
        139, 140, 142, 144, 145, 146, 147, 148, 150, 151, 152, 153, 155, 156, 157, 158, 160, 161,
        163, 164, 165, 171, 176, 177, 178, 179, 180, 181, 184, 189, 190, 195, 200, 201, 202, 254,
    ];
    assert_eq!(
        C_GAME_STAGE_JUMP_ENTRIES
            .iter()
            .map(|entry| entry.case)
            .collect::<Vec<_>>(),
        cases
    );
    for opcode in (0..=306).chain([u32::MAX]) {
        let expected = cases.iter().position(|&case| case + 20 == opcode);
        assert_eq!(
            f3_stage_jump_entry(opcode),
            expected.map(|index| &C_GAME_STAGE_JUMP_ENTRIES[index]),
            "opcode {opcode}"
        );
    }
    for (case, opcode, selector, dispatch_va, handler_va) in [
        (9, 29, 53, 0xE5DCA9, None),
        (80, 100, 53, 0xE5DCA9, None),
        (103, 123, 5, 0xE5D47A, None),
        (112, 132, 6, 0xE5D691, Some(0xE610C0)),
        (135, 155, 14, 0xE5D777, Some(0xE67640)),
        (136, 156, 15, 0xE5D789, Some(0xE67770)),
        (145, 165, 22, 0xE5D570, Some(0xE63260)),
        (160, 180, 34, 0xE5D610, Some(0xE72490)),
        (165, 185, 38, 0xE5D765, Some(0xE67470)),
        (177, 197, 41, 0xE5D79B, Some(0xE751F0)),
        (184, 204, 53, 0xE5DCA9, None),
        (254, 274, 52, 0xE5DAAB, Some(0xD73100)),
    ] {
        let entry = f3_stage_jump_entry(opcode).unwrap();
        assert_eq!(entry.case, case);
        assert_eq!(entry.wire_opcode, opcode);
        assert_eq!(entry.table_selector, selector);
        assert_eq!(entry.dispatch_va, dispatch_va);
        assert_eq!(entry.handler_va, handler_va);
    }
}

#[test]
fn mon_action_provenance_pins_cp405_and_keeps_capture_pending() {
    let provenance = F3MonActionProvenance::CP405_STATIC;
    assert_eq!(provenance.owner_gate_va, 0xB91F60);
    assert_eq!(provenance.delegation_opcode, 204);
    assert_eq!(provenance.step_report_opcodes, [132, 180]);
    assert_eq!(
        provenance.status,
        F3WireStatus::StaticCandidatePendingCapture
    );
    assert_eq!(MonActionDelegationCandidate::STATUS, provenance.status);
    assert_eq!(MonActionStepReportCandidate::STATUS, provenance.status);
    assert_eq!(MonActionStep132Candidate::STATUS, provenance.status);
    assert_eq!(MonActionStep180Candidate::STATUS, provenance.status);
    assert_eq!(MonActionDelegationCandidate::OPCODE, 204);
    assert_eq!(MonActionStep132Candidate::OPCODE, 132);
    assert_eq!(MonActionStep180Candidate::OPCODE, 180);
}

#[test]
fn mon_action_delegation_checks_opcode_object_and_slot() {
    for (object_id, owner_slot) in [(1, 0), (u16::MAX, 7)] {
        let candidate = MonActionDelegationCandidate {
            opcode: 204,
            object_id,
            owner_slot,
        };
        assert_eq!(candidate.validate(), Ok(()));
        assert_eq!(
            MonActionDelegationCandidate::new(204, object_id, owner_slot),
            Ok(candidate)
        );
    }

    let assert_rejected = |opcode, object_id, owner_slot, error| {
        let candidate = MonActionDelegationCandidate {
            opcode,
            object_id,
            owner_slot,
        };
        assert_eq!(candidate.validate(), Err(error));
        assert_eq!(candidate.encode(), Err(error));
        assert_eq!(
            MonActionDelegationCandidate::new(opcode, object_id, owner_slot),
            Err(error)
        );
    };
    for opcode in [0, 132, 180, 203, 205, 0x1_00cc, u32::MAX] {
        assert_rejected(opcode, 1, 0, F3WireError::InvalidMonActionOpcode(opcode));
    }
    assert_rejected(204, 0, 0, F3WireError::ZeroMonActionObjectId);
    for slot in [8, u8::MAX] {
        assert_rejected(
            204,
            1,
            slot,
            F3WireError::MonActionOwnerSlotOutOfRange(slot),
        );
    }
}

#[test]
fn mon_action_delegation_204_uses_one_word_then_one_byte() {
    for (object_id, owner_slot, body) in [
        (1, 0, [0xa5, 0x68, 0x5a]),
        (0x1234, 3, [0xb7, 0x5d, 0x59]),
        (u16::MAX, 7, [0x5a, 0x96, 0x5d]),
    ] {
        let candidate = MonActionDelegationCandidate::new(204, object_id, owner_slot).unwrap();
        assert_eq!(candidate.encode().unwrap(), body);
        assert_eq!(MonActionDelegationCandidate::decode(&body), Ok(candidate));
        assert_exact(&body, MonActionDelegationCandidate::decode);
    }
    assert_eq!(
        MonActionDelegationCandidate::decode(&[0xa5, 0x69, 0x5a]),
        Err(F3WireError::ZeroMonActionObjectId)
    );
    for slot in 8..=u8::MAX {
        assert_eq!(
            MonActionDelegationCandidate::decode(&[0xb7, 0x5d, slot ^ 0x5a]),
            Err(F3WireError::MonActionOwnerSlotOutOfRange(slot))
        );
    }
}

#[test]
fn mon_action_step_report_checks_opcode_object_and_catalog_bounds() {
    for opcode in [132, 180] {
        for (object_id, step_index, template_id) in [(1, 0, 0), (u16::MAX, 214, 363)] {
            let candidate = MonActionStepReportCandidate {
                opcode,
                object_id,
                step_index,
                template_id,
            };
            assert_eq!(candidate.validate(), Ok(()));
            assert_eq!(
                MonActionStepReportCandidate::new(opcode, object_id, step_index, template_id),
                Ok(candidate)
            );
        }
    }

    let assert_rejected = |opcode, object_id, step_index, template_id, error| {
        let candidate = MonActionStepReportCandidate {
            opcode,
            object_id,
            step_index,
            template_id,
        };
        assert_eq!(candidate.validate(), Err(error));
        assert_eq!(
            MonActionStepReportCandidate::new(opcode, object_id, step_index, template_id),
            Err(error)
        );
    };
    for opcode in [0, 131, 133, 179, 181, 204, 0x1_0084, u32::MAX] {
        assert_rejected(opcode, 1, 0, 0, F3WireError::InvalidMonActionOpcode(opcode));
    }
    assert_rejected(132, 0, 0, 0, F3WireError::ZeroMonActionObjectId);
    for index in [215, u16::MAX] {
        assert_rejected(
            132,
            1,
            index,
            0,
            F3WireError::MonActionStepIndexOutOfRange(index),
        );
    }
    for id in [364, u16::MAX] {
        assert_rejected(
            180,
            1,
            0,
            id,
            F3WireError::MonActionTemplateIdOutOfRange(id),
        );
    }
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
fn mon_action_132_matches_each_static_writer_shape() {
    use MonActionStep132Payload::*;

    let vectors: [(MonActionStep132Payload, u16, &[u8]); 16] = [
        (
            Kind0(vec![1, 2, 3, 4, 5, 6, 7]),
            0,
            &[0xa5, 0x6e, 0x92, 0xc9, 0x51, 0x94, 0xa3, 0x6c, 0x5d],
        ),
        (Kind1(0xff), 1, &[0xa5]),
        (Kind9(0xffff, 0x0102), 9, &[0x5a, 0x96, 0xa4, 0x6b]),
        (Kind10(0x0102, 0xffff), 10, &[0xa4, 0x6b, 0x5a, 0x96]),
        (Kind11(0, 0x1234), 11, &[0xa5, 0x69, 0xb7, 0x5d]),
        (
            Kind12 {
                selector: 3,
                value: 0xff,
            },
            12,
            &[0x59, 0xa5],
        ),
        (Kind13(0x1234), 13, &[0xb7, 0x5d]),
        (Kind14, 14, &[]),
        (Kind15(0), 15, &[0x5a]),
        (Kind16, 16, &[]),
        (Kind17(0x12), 17, &[0x48]),
        (Kind18, 18, &[]),
        (Kind19, 19, &[]),
        (
            Kind21(vec![0, 0xff, 0x81]),
            21,
            &[0xa5, 0x6a, 0x5a, 0x69, 0xdb],
        ),
        (Kind21(vec![]), 21, &[0xa5, 0x69]),
        (Kind22(0xffff), 22, &[0x5a, 0x96]),
    ];
    for (payload, kind, tail) in vectors {
        let candidate = MonActionStep132Candidate {
            session_or_owner: 0x0102_0304,
            object_id: 0x1234,
            payload,
        };
        let mut body = vec![0x97, 0xc8, 0x50, 0x91, 0xb7, 0x5d, 0xa5, 0x69 ^ kind as u8];
        body.extend(tail);
        assert_eq!(candidate.action_kind(), kind);
        assert_eq!(candidate.encode().unwrap(), body);
        assert_eq!(MonActionStep132Candidate::decode(&body), Ok(candidate));
        assert_exact(&body, MonActionStep132Candidate::decode);
    }
}

#[test]
fn mon_action_180_matches_each_static_writer_shape() {
    use MonActionStep180Payload::*;

    let vectors: [(MonActionStep180Payload, u16, &[u8]); 8] = [
        (Kind0(0xff), 0, &[0xa5]),
        (Kind3(0xffff, 0x0102), 3, &[0x5a, 0x96, 0xa4, 0x6b]),
        (Kind4(0x0102, 0xffff), 4, &[0xa4, 0x6b, 0x5a, 0x96]),
        (Kind5(0, 0x1234), 5, &[0xa5, 0x69, 0xb7, 0x5d]),
        (
            Kind6 {
                selector: 14,
                value: 0x12,
            },
            6,
            &[0x54, 0x48],
        ),
        (Kind8, 8, &[]),
        (
            Kind9(vec![0, 0xff, 0x81]),
            9,
            &[0xa5, 0x6a, 0x5a, 0x69, 0xdb],
        ),
        (
            Kind10 {
                target: vec![0, 0xff, 0x81],
                first: 0x0102_0304,
                second: 0x1122_3344,
            },
            10,
            &[
                0xa5, 0x6a, 0x5a, 0x69, 0xdb, 0x97, 0xc8, 0x50, 0x91, 0x87, 0xe8, 0x60, 0xd1,
            ],
        ),
    ];
    for (payload, kind, tail) in vectors {
        let candidate = MonActionStep180Candidate {
            object_id: 0x1234,
            payload,
        };
        let mut body = vec![0xb7, 0x5d, 0xa5, 0x69 ^ kind as u8];
        body.extend(tail);
        assert_eq!(candidate.action_kind(), kind);
        assert_eq!(candidate.encode().unwrap(), body);
        assert_eq!(MonActionStep180Candidate::decode(&body), Ok(candidate));
        assert_exact(&body, MonActionStep180Candidate::decode);
    }
}

#[test]
fn mon_action_step_decoders_reject_unrecorded_kind_words() {
    for kind in [2, 3, 8, 20, 23, 0x100, 0x109, u16::MAX] {
        let mut body = vec![0x97, 0xc8, 0x50, 0x91, 0xb7, 0x5d];
        body.extend(write_uint(kind.into(), Width::U16).unwrap());
        assert_eq!(
            MonActionStep132Candidate::decode(&body),
            Err(F3WireError::UnsupportedMonActionKind { opcode: 132, kind })
        );
    }
    for kind in [1, 2, 7, 11, 14, 0x100, 0x103, u16::MAX] {
        let mut body = vec![0xb7, 0x5d];
        body.extend(write_uint(kind.into(), Width::U16).unwrap());
        assert_eq!(
            MonActionStep180Candidate::decode(&body),
            Err(F3WireError::UnsupportedMonActionKind { opcode: 180, kind })
        );
    }
}

#[test]
fn mon_action_step_selectors_are_bounded_on_both_codec_paths() {
    for selector in 0..=u8::MAX {
        let step = MonActionStep132Candidate {
            session_or_owner: 0x0102_0304,
            object_id: 0x1234,
            payload: MonActionStep132Payload::Kind12 { selector, value: 0 },
        };
        let body = [
            0x97,
            0xc8,
            0x50,
            0x91,
            0xb7,
            0x5d,
            0xa5,
            0x65,
            selector ^ 0x5a,
            0x5a,
        ];
        if selector <= 3 {
            assert_eq!(step.encode().unwrap(), body);
            assert_eq!(MonActionStep132Candidate::decode(&body), Ok(step));
        } else {
            let error = F3WireError::InvalidMonActionSelector {
                opcode: 132,
                selector,
            };
            assert_eq!(step.encode(), Err(error));
            assert_eq!(MonActionStep132Candidate::decode(&body), Err(error));
        }

        let step = MonActionStep180Candidate {
            object_id: 0x1234,
            payload: MonActionStep180Payload::Kind6 {
                selector,
                value: 0xff,
            },
        };
        let body = [0xb7, 0x5d, 0xa5, 0x6f, selector ^ 0x5a, 0xa5];
        if (14..=16).contains(&selector) {
            assert_eq!(step.encode().unwrap(), body);
            assert_eq!(MonActionStep180Candidate::decode(&body), Ok(step));
        } else {
            let error = F3WireError::InvalidMonActionSelector {
                opcode: 180,
                selector,
            };
            assert_eq!(step.encode(), Err(error));
            assert_eq!(MonActionStep180Candidate::decode(&body), Err(error));
        }
    }
}

#[test]
fn mon_action_step_byte_strings_respect_declared_lengths_and_body_limits() {
    const MAX_BODY: usize = MAX_FRAME_LEN - FRAME_OVERHEAD;
    let frame_too_large = F3WireError::Codec(CodecError::FrameTooLarge);
    // Both bodies declare 65,535 string bytes and supply one.
    assert_eq!(
        MonActionStep132Candidate::decode(&[
            0x97, 0xc8, 0x50, 0x91, 0xb7, 0x5d, 0xa5, 0x69, 0x5a, 0x96, 0x5a,
        ]),
        Err(F3WireError::TruncatedEof)
    );
    assert_eq!(
        MonActionStep180Candidate::decode(&[0xb7, 0x5d, 0xa5, 0x63, 0x5a, 0x96, 0x5a]),
        Err(F3WireError::TruncatedEof)
    );

    let mut step132 = MonActionStep132Candidate {
        session_or_owner: u32::MAX,
        object_id: u16::MAX,
        payload: MonActionStep132Payload::Kind0(vec![0xff; MAX_BODY - 10]),
    };
    let mut body = step132.encode().unwrap();
    assert_eq!(body.len(), MAX_BODY);
    assert_eq!(
        MonActionStep132Candidate::decode(&body),
        Ok(step132.clone())
    );
    body.push(0);
    assert_eq!(
        MonActionStep132Candidate::decode(&body),
        Err(frame_too_large)
    );
    step132.payload = MonActionStep132Payload::Kind21(vec![0; MAX_BODY - 9]);
    assert_eq!(step132.encode(), Err(frame_too_large));

    let step180 = |len| MonActionStep180Candidate {
        object_id: u16::MAX,
        payload: MonActionStep180Payload::Kind10 {
            target: vec![0x81; len],
            first: 0,
            second: u32::MAX,
        },
    };
    let limit = step180(MAX_BODY - 14);
    let mut body = limit.encode().unwrap();
    assert_eq!(body.len(), MAX_BODY);
    assert_eq!(MonActionStep180Candidate::decode(&body), Ok(limit));
    body.push(0);
    assert_eq!(
        MonActionStep180Candidate::decode(&body),
        Err(frame_too_large)
    );
    assert_eq!(step180(MAX_BODY - 13).encode(), Err(frame_too_large));
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
