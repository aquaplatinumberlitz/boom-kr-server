use kr_wire::kr00967::cosmetics::{
    CosmeticTarget, F4_SHOP_ANCHORS, F4AnchorSite, ProjectionError, ProjectionStatus,
    ProjectionSurface, SHOP_STAGE_GATE_ANCHORS, ShopStageGateAnchors, USER_SIMPLE_INFO_TAIL_GUARDS,
    UserSimpleInfoTailGuards, f4_shop_anchor_for_opcode, slots_conflict, validate_mutual_exclusion,
};
use kr_wire::kr00967::inventory::{Stage20ProfileSetterCandidate, Stage20SubD95C30};
use kr_wire::{FRAME_OVERHEAD, MAX_FRAME_LEN};
use std::{collections::BTreeSet, io::ErrorKind, ptr};

#[test]
fn f4_shop_gate_and_user_simple_info_tail_are_additive_static_anchors() {
    assert_eq!(F4_SHOP_ANCHORS.len(), 16); // Existing branch baseline.
    assert_eq!(
        SHOP_STAGE_GATE_ANCHORS,
        ShopStageGateAnchors {
            dispatcher_va: 0x1072B10,
            gate_va: 0x125E540,
            states: [72, 73, 74],
            input_opcode: 121,
            from_stage: 11,
            to_stage: 12,
            status: ProjectionStatus::StaticCandidate,
        }
    );
    assert_eq!(
        USER_SIMPLE_INFO_TAIL_GUARDS,
        UserSimpleInfoTailGuards {
            deserializer_va: 0xAE6330,
            guard_vas: [0x7036F0, 0x7A0130, 0x48D8D0],
            status: ProjectionStatus::StaticCandidate,
        }
    );
}

#[test]
fn stage20_round_trips_with_and_without_the_client_guard() {
    // Conformance vectors assembled from the static layout and KR field primitives.
    let prefix = [
        0xa4, 0x7b, // Two independent u8 fields.
        0x97, 0xc8, 0x50, 0x91, 0x87, 0xe8, 0x60, 0xd1, 0x68, 0x16, 0xe9, 0x0d, 0x96, 0xca, 0x53,
        0x95, // Four u32 values.
    ];
    let extension = [
        0x5a, // Zero tag still appears when the guard is true.
        0x96, 0xca, 0x53, 0x95, 0x68, 0x16, 0xe9, 0x0d, 0x97, 0xc8, 0x50, 0x91, 0x87, 0xe8, 0x60,
        0xd1, // Four u32 words.
        0xa5, 0x6b, 0x21, 0xbb, // Two native bytes for "ệ".
    ];
    let mut candidate = Stage20ProfileSetterCandidate {
        flags_or_a: 0xfe,
        b: 0x21,
        values: [0x0102_0304, 0x1122_3344, 0xfedc_ba98, 0],
        sub_d95c30: None,
    };
    assert_eq!(Stage20ProfileSetterCandidate::OPCODE, 20);
    assert_eq!(candidate.encode().unwrap(), prefix);
    let decode = Stage20ProfileSetterCandidate::decode;
    assert_eq!(decode(&prefix, false).unwrap(), candidate);
    assert_eq!(
        decode(&prefix, true).unwrap_err().kind(),
        ErrorKind::UnexpectedEof
    );

    candidate.sub_d95c30 = Some(Stage20SubD95C30 {
        tag: 0,
        words: [0, 0xfedc_ba98, 0x0102_0304, 0x1122_3344],
        name: "ệ".into(),
    });
    let mut extended = [prefix.as_slice(), &extension].concat();
    assert_eq!(candidate.encode().unwrap(), extended);
    assert_eq!(decode(&extended, true).unwrap(), candidate);
    assert_eq!(
        decode(&extended, false).unwrap_err().kind(),
        ErrorKind::InvalidData
    );
    assert!(decode(&extended[..extended.len() - 1], true).is_err());
    extended.push(0);
    assert_eq!(
        decode(&extended, true).unwrap_err().kind(),
        ErrorKind::InvalidData
    );
}

#[test]
fn stage20_extension_name_is_bounded_by_the_frame_body() {
    let max_body = MAX_FRAME_LEN - FRAME_OVERHEAD;
    let fixed_bytes = 2 + 4 * 4 + 1 + 4 * 4 + 2;
    let mut candidate = Stage20ProfileSetterCandidate {
        flags_or_a: 0,
        b: 0,
        values: [0; 4],
        sub_d95c30: Some(Stage20SubD95C30 {
            tag: 0,
            words: [0; 4],
            name: "a".repeat(max_body - fixed_bytes),
        }),
    };
    let mut body = candidate.encode().unwrap();
    assert_eq!(body.len(), max_body);
    let decode = Stage20ProfileSetterCandidate::decode;
    assert_eq!(decode(&body, true).unwrap(), candidate);

    candidate.sub_d95c30.as_mut().unwrap().name.push('a');
    assert_eq!(
        candidate.encode().unwrap_err().kind(),
        ErrorKind::InvalidInput
    );
    body[fixed_bytes - 2..fixed_bytes].copy_from_slice(&[0x5a, 0x96]); // u16::MAX length.
    assert_eq!(
        decode(&body, true).unwrap_err().kind(),
        ErrorKind::UnexpectedEof
    );
    body.push(0);
    assert_eq!(
        decode(&body, true).unwrap_err().kind(),
        ErrorKind::InvalidData
    );
}

#[test]
fn f4_anchor_keys_are_unique_and_lookup_returns_the_original_entry() {
    let mut names = BTreeSet::new();
    let mut opcodes = BTreeSet::new();
    for anchor in F4_SHOP_ANCHORS {
        assert!(names.insert(anchor.name), "duplicate name: {}", anchor.name);
        if let Some(opcode) = anchor.opcode {
            assert!(opcodes.insert(opcode), "duplicate opcode: {opcode}");
            assert!(ptr::eq(f4_shop_anchor_for_opcode(opcode).unwrap(), anchor));
        }
    }
    for unknown in [0, 134, 136, 236, 312, 318, u32::MAX] {
        assert_eq!(f4_shop_anchor_for_opcode(unknown), None, "opcode {unknown}");
    }
}

#[test]
fn f4_shop_functions_are_static_sites_without_packet_opcodes() {
    use F4AnchorSite::{Function, ShopStageDispatcher};
    let sites = F4_SHOP_ANCHORS
        .iter()
        .filter(|anchor| anchor.opcode.is_none())
        .map(|anchor| (anchor.name, (anchor.site, anchor.status)))
        .collect::<std::collections::BTreeMap<_, _>>();
    use ProjectionStatus::StaticCandidate;
    assert_eq!(
        sites,
        [
            (
                "CShopStageNew",
                (ShopStageDispatcher(0x1072B10), StaticCandidate)
            ),
            ("sub_11CE930", (Function(0x11CE930), StaticCandidate)),
            ("sub_125D190", (Function(0x125D190), StaticCandidate)),
            ("sub_AE6330", (Function(0xAE6330), StaticCandidate)),
        ]
        .into()
    );
}

#[test]
fn f4_measured_account_packets_do_not_promote_static_candidates() {
    let measured = F4_SHOP_ANCHORS
        .iter()
        .filter(|anchor| anchor.status == ProjectionStatus::Measured)
        .map(|anchor| (anchor.opcode, anchor.name))
        .collect::<BTreeSet<_>>();
    assert_eq!(
        measured,
        [(Some(135), "lucci135"), (Some(291), "own291")].into()
    );

    for opcode in [20, 219, 224, 225, 313, 314, 315, 316, 317] {
        let anchor = f4_shop_anchor_for_opcode(opcode).unwrap();
        assert_eq!(anchor.status, ProjectionStatus::StaticCandidate, "{opcode}");
        assert!(matches!(
            anchor.site,
            F4AnchorSite::ServerToClientHandler(_)
        ));
    }
    let outgoing = f4_shop_anchor_for_opcode(566).unwrap();
    assert_eq!(outgoing.status, ProjectionStatus::StaticCandidate);
    assert_eq!(
        outgoing.site,
        F4AnchorSite::ClientToServerBuilder(0x11AFA5D)
    );

    let surface = ProjectionSurface::Own291;
    for &target in CosmeticTarget::ALL {
        assert_eq!(
            target.projection_status(surface),
            ProjectionStatus::Unmeasured
        );
        assert_eq!(
            target.validate_projection(surface),
            Err(ProjectionError::Unmeasured { target, surface })
        );
    }
}

#[test]
fn cosmetic_mutual_exclusion_uses_exact_slot_tokens_and_input_order() {
    assert!(slots_conflict("cosmetic.avatar", "cosmetic.avatar"));
    assert!(!slots_conflict("character", "cosmetic.char"));
    assert_eq!(validate_mutual_exclusion(&[]), Ok(()));
    assert_eq!(
        validate_mutual_exclusion(&["head", "Head", "head ", "character", "cosmetic.char"]),
        Ok(())
    );
    assert_eq!(
        validate_mutual_exclusion(&["head", "quick", "quick", "head"]),
        Err("quick")
    );
    assert_eq!(
        validate_mutual_exclusion(&["quick", "head", "head", "quick"]),
        Err("head")
    );
}
