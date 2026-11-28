use kr_wire::kr00967::cosmetics::{
    CosmeticTarget, F4_SHOP_ANCHORS, F4AnchorSite, ProjectionError, ProjectionStatus,
    ProjectionSurface, f4_shop_anchor_for_opcode, slots_conflict, validate_mutual_exclusion,
};
use std::{collections::BTreeSet, ptr};

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
