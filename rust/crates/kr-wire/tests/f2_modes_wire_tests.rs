use kr_wire::kr00967::{
    BulaGroupPoints, BulaGroupRecordTail, BulaTailRecord, F2_MODE_ANCHORS, F2_MODE_FAMILIES,
    F2AnchorSite, F2AnchorStatus, GameTypeId,
};
use std::io::ErrorKind;

// Raw tail_hex from docs/evidence/content/f2-bula-tail-and-mode-wire-re.json (#810).
const BULA00: &str = concat!(
    "01000000040000000e0000000c000000000000000c0000000000000000000000",
    "0e00000000000000040000000a0000004f75746572536f756c000e0c00000000",
    "03000000010000000a000000496e6e6572536f756c0005020200000002000000",
    "000000000a000000496e6e6572536f756c000308020000000200000000000000",
    "0a000000496e6e6572536f756c00040400000000020000000000000000000000",
);
const BULA16: &str = concat!(
    "02000000040000000000000000000000000000000c0000000e0000000c000000",
    "0e00000000000000040000000e000000000000000e0000000c00000000000000",
    "0c0000000000000000000000020000000a0000004f75746572536f756c000000",
    "0000000003000000010000000a0000004f75746572536f756c000e0000000000",
    "030000000200000000000000",
);

fn hex_bytes(hex: &str) -> Vec<u8> {
    assert_eq!(hex.len() % 2, 0);
    (0..hex.len())
        .step_by(2)
        .map(|at| u8::from_str_radix(&hex[at..at + 2], 16).unwrap())
        .collect()
}

fn record(name: &[u8], x: u8, y: u8, fields: [i32; 3]) -> BulaTailRecord {
    BulaTailRecord {
        raw_cp949_name: name.to_vec(),
        x,
        y,
        fields,
    }
}

#[test]
fn shipped_bula_tails_round_trip_byte_for_byte() {
    let outer = b"OuterSoul\0";
    let inner = b"InnerSoul\0";
    for (hex, groups, records) in [
        (
            BULA00,
            vec![BulaGroupPoints(vec![(14, 12), (0, 12), (0, 0), (14, 0)])],
            vec![
                record(outer, 14, 12, [0, 3, 1]),
                record(inner, 5, 2, [2, 2, 0]),
                record(inner, 3, 8, [2, 2, 0]),
                record(inner, 4, 4, [0, 2, 0]),
            ],
        ),
        (
            BULA16,
            vec![
                BulaGroupPoints(vec![(0, 0), (0, 12), (14, 12), (14, 0)]),
                BulaGroupPoints(vec![(14, 0), (14, 12), (0, 12), (0, 0)]),
            ],
            vec![
                record(outer, 0, 0, [0, 3, 1]),
                record(outer, 14, 0, [0, 3, 2]),
            ],
        ),
    ] {
        let raw = hex_bytes(hex);
        let tail = BulaGroupRecordTail::decode(&raw).unwrap();
        assert_eq!(tail.groups, groups);
        assert_eq!(tail.records, records);
        assert!(tail.records.iter().all(|record| {
            record
                .raw_cp949_name
                .strip_suffix(b"\0")
                .is_some_and(|name| name == b"OuterSoul" || name == b"InnerSoul")
        }));
        assert!(tail.script.is_empty());
        assert_eq!(tail.encode().unwrap(), raw);
    }
}

#[test]
fn decode_rejects_every_truncated_prefix_and_trailing_bytes() {
    for hex in [BULA00, BULA16] {
        let mut raw = hex_bytes(hex);
        for end in 0..raw.len() {
            assert_eq!(
                BulaGroupRecordTail::decode(&raw[..end]).unwrap_err().kind(),
                ErrorKind::UnexpectedEof,
                "prefix with {end} bytes"
            );
        }
        raw.push(0);
        assert_eq!(
            BulaGroupRecordTail::decode(&raw).unwrap_err().kind(),
            ErrorKind::InvalidData
        );
    }
}

#[test]
fn decode_rejects_invalid_counts_and_script_lengths() {
    let raw = hex_bytes(BULA00);
    for (offset, cap) in [
        (0, BulaGroupRecordTail::MAX_GROUPS),
        (4, BulaGroupRecordTail::MAX_POINTS_PER_GROUP),
        (40, BulaGroupRecordTail::MAX_RECORDS),
        (44, BulaGroupRecordTail::MAX_NAME_BYTES),
    ] {
        for value in [-1, (cap + 1) as i32] {
            let mut bad = raw.clone();
            bad[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
            assert_eq!(
                BulaGroupRecordTail::decode(&bad).unwrap_err().kind(),
                ErrorKind::InvalidData,
                "count {value} at offset {offset}"
            );
        }
    }

    let mut empty_name = raw.clone();
    empty_name[44..48].fill(0);
    assert_eq!(
        BulaGroupRecordTail::decode(&empty_name).unwrap_err().kind(),
        ErrorKind::InvalidData
    );

    let script_at = raw.len() - 4;
    for (length, error) in [
        (1, ErrorKind::UnexpectedEof),
        (
            BulaGroupRecordTail::MAX_SCRIPT_BYTES as u32 + 1,
            ErrorKind::InvalidData,
        ),
        (u32::MAX, ErrorKind::InvalidData),
    ] {
        let mut bad = raw.clone();
        bad[script_at..].copy_from_slice(&length.to_le_bytes());
        assert_eq!(BulaGroupRecordTail::decode(&bad).unwrap_err().kind(), error);
    }
}

#[test]
fn codec_accepts_the_caps_and_encoder_rejects_larger_values() {
    let mut tail = BulaGroupRecordTail {
        groups: vec![BulaGroupPoints::default(); BulaGroupRecordTail::MAX_GROUPS],
        records: vec![
            record(
                &b"\xc7\xd1".repeat(BulaGroupRecordTail::MAX_NAME_BYTES / 2),
                u8::MAX,
                0,
                [i32::MIN, -1, i32::MAX],
            );
            BulaGroupRecordTail::MAX_RECORDS
        ],
        script: vec![0xa5; BulaGroupRecordTail::MAX_SCRIPT_BYTES],
    };
    tail.groups[0] = BulaGroupPoints(vec![
        (i32::MIN, i32::MAX);
        BulaGroupRecordTail::MAX_POINTS_PER_GROUP
    ]);
    assert_eq!(
        BulaGroupRecordTail::decode(&tail.encode().unwrap()).unwrap(),
        tail
    );

    let mut bad = BulaGroupRecordTail {
        groups: vec![BulaGroupPoints::default(); BulaGroupRecordTail::MAX_GROUPS + 1],
        ..Default::default()
    };
    assert_eq!(bad.encode().unwrap_err().kind(), ErrorKind::InvalidInput);
    bad.groups.truncate(1);
    bad.groups[0]
        .0
        .resize(BulaGroupRecordTail::MAX_POINTS_PER_GROUP + 1, (0, 0));
    assert_eq!(bad.encode().unwrap_err().kind(), ErrorKind::InvalidInput);
    bad.groups.clear();
    bad.records = vec![record(b"x", 0, 0, [0; 3]); BulaGroupRecordTail::MAX_RECORDS + 1];
    assert_eq!(bad.encode().unwrap_err().kind(), ErrorKind::InvalidInput);
    bad.records.truncate(1);
    bad.records[0]
        .raw_cp949_name
        .resize(BulaGroupRecordTail::MAX_NAME_BYTES + 1, b'x');
    assert_eq!(bad.encode().unwrap_err().kind(), ErrorKind::InvalidInput);
    bad.records[0].raw_cp949_name.clear();
    assert_eq!(bad.encode().unwrap_err().kind(), ErrorKind::InvalidInput);
    bad.records.clear();
    bad.script
        .resize(BulaGroupRecordTail::MAX_SCRIPT_BYTES + 1, 0);
    assert_eq!(bad.encode().unwrap_err().kind(), ErrorKind::InvalidInput);
}

#[test]
fn ida_locators_have_only_the_documented_static_bindings() {
    use F2AnchorSite::{Function, Rtti, ServerToClientHandler};
    let expected = [
        ("sub_57F520", Function(0x57F520)),
        ("sub_C6C080", Function(0xC6C080)),
        ("MapGroupVisible writer", Function(0xE04CB0)),
        ("MapGroupVisible reader", Function(0xE04DD0)),
        ("CGamePlayMap Soul reader", Function(0x115C770)),
        ("CGamePlayMap Soul record consumer", Function(0x115DCF0)),
        (
            "RebirthToGhost",
            ServerToClientHandler {
                va: 0xE67470,
                opcode: 185,
            },
        ),
        ("WeeklyLadderPacket", Rtti),
        ("ConfusedFightPacket", Rtti),
        ("PTimebombClient", Rtti),
        ("PTimebombServer", Rtti),
        ("PWeeklyLadderUserInfoResult", Rtti),
    ];
    assert_eq!(F2_MODE_ANCHORS.len(), expected.len());
    for (name, site) in expected {
        let anchor = F2_MODE_ANCHORS
            .iter()
            .find(|anchor| anchor.name == name)
            .unwrap();
        assert_eq!(anchor.site, site, "{name}");
        assert_eq!(anchor.status, F2AnchorStatus::KrStaticConfirmed, "{name}");
    }
}

#[test]
fn all_eighteen_families_still_require_wine_capture() {
    let families = [
        "Baseball",
        "Bula",
        "CAPong",
        "Clover",
        "Coloring",
        "Gumiho",
        "Hamster",
        "Kart",
        "PopcadeA",
        "PopcadeB",
        "PopcadeC",
        "Rabbit",
        "RescueET",
        "SnowyFields",
        "SuperFighter",
        "Wriggle",
        "Zombie",
        "Zzolman",
    ];
    assert_eq!(F2_MODE_FAMILIES.len(), families.len());
    let pending = F2AnchorStatus::UnmeasuredPendingWineCapture;
    for family in families {
        let row = F2_MODE_FAMILIES
            .iter()
            .find(|row| row.family == family)
            .unwrap();
        assert_eq!(row.wire_mode, pending, "{family}");
        assert_eq!(row.round_objective_wire, pending, "{family}");
        let ui_ids: &[GameTypeId] = match family {
            "PopcadeA" | "PopcadeB" | "PopcadeC" => &[GameTypeId::Popcade],
            "SuperFighter" => &[
                GameTypeId::SuperFighter,
                GameTypeId::SuperFighter21,
                GameTypeId::SuperFighter25,
            ],
            _ => &[],
        };
        assert_eq!(row.static_game_type_candidates, ui_ids, "{family}");
    }
}
