use kr_wire::kr00967::custom_map::{
    CUSTOM_MAP_PACKET_ANCHORS, CUSTOM_MAP_TYPE_666_ANCHORS, CustomMap, CustomMapError,
    CustomMapWireRoute, CustomMapWireStatus, custom_map_anchor,
};
use std::collections::BTreeMap;

fn payload() -> Vec<u8> {
    let tiles = vec!["0"; 15 * 13].join(",");
    format!(
        r#"{{"schema":"kre-f7-normalized-map-v1","width":15,"height":13,"tiles":[{tiles}],"spawns":[[0,0],[1,0]]}}"#
    )
    .into_bytes()
}

#[test]
fn existing_custom_map_validation_and_wire_guard_remain_intact() {
    let palette = BTreeMap::from([(0_u16, true)]);
    let payload = payload();
    let map = CustomMap::parse_normalized(&payload, &palette).expect("valid normalized map");
    assert_eq!(map.grid().dimensions(), (15, 13));
    assert_eq!(map.spawns(), &[[0, 0], [1, 0]]);
    assert_eq!(
        CustomMap::parse_normalized(&payload, &BTreeMap::from([(0, false)])),
        Err(CustomMapError::SolidSpawn([0, 0]))
    );
    assert_eq!(map.validate_roster(2), Ok(()));
    assert_eq!(
        map.validate_roster(3),
        Err(CustomMapError::InsufficientSpawns)
    );
    for route in [CustomMapWireRoute::Upload, CustomMapWireRoute::Select] {
        assert_eq!(route.status(), CustomMapWireStatus::Unmeasured);
        assert_eq!(
            route.validate_dispatch(),
            Err(CustomMapError::UnmeasuredWireContract)
        );
        assert_eq!(
            route.encode(&map),
            Err(CustomMapError::UnmeasuredWireContract)
        );
    }
}

#[test]
fn type_666_anchor_values_and_status_are_static_only() {
    let anchors = CUSTOM_MAP_TYPE_666_ANCHORS;
    assert_eq!(anchors.dispatcher_va, 0x5A3BF0);
    assert_eq!(anchors.packet_type, 666);
    assert_eq!(anchors.subtype_range, (19, 38));
    assert_eq!(anchors.upload_ctor_va, 0x9BFC80);
    assert_eq!(anchors.upload_handler_va, 0x5B11D0);
    assert_eq!(anchors.usermap_rtti_va, 0x19C7E80);
    assert_eq!(anchors.usermap_vtable_va, 0x175E38C);
    assert_eq!(anchors.room_meta_opcode, 273);
    assert_eq!(anchors.room_meta_reader_va, 0xD72FD0);
    assert_eq!(anchors.status, CustomMapWireStatus::UnmeasuredWireContract);
}

#[test]
fn packet_anchor_lookup_covers_all_reader_writer_pairs() {
    let expected = [
        ("P_REQ_CHECK_HASH_VALUE", 0x4D68F0, 0x4D6980),
        ("P_REQ_PUBLISH", 0x4D6B80, 0x4D6C10),
        ("P_REQ_LOAD", 0x4D6E60, 0x4D6EC0),
        ("P_REQ_LOAD_SEARCH", 0x4D6FC0, 0x4D7080),
        ("P_REQ_GOOD", 0x4D7350, 0x4D73E0),
    ];
    assert_eq!(CUSTOM_MAP_PACKET_ANCHORS.len(), expected.len());
    for (name, writer_va, reader_va) in expected {
        let anchor = custom_map_anchor(name).expect("known packet anchor");
        assert_eq!(anchor.name, name);
        assert_eq!((anchor.writer_va, anchor.reader_va), (writer_va, reader_va));
        assert_eq!(anchor.status, CustomMapWireStatus::UnmeasuredWireContract);
    }
    assert!(custom_map_anchor("P_RES_ERROR").is_none());
    assert!(
        CUSTOM_MAP_PACKET_ANCHORS
            .iter()
            .all(|anchor| anchor.status == CustomMapWireStatus::UnmeasuredWireContract)
    );
}
