//! Owner-selected G1 drop data, not a reconstruction of original RNG rules.
//! Keep every source row (including duplicates), with excluded weights zeroed.
use serde::Deserialize;
use std::io;

#[derive(Deserialize)]
struct Scope {
    maps: Vec<Map>,
}
#[derive(Deserialize)]
struct Map {
    name: String,
    rows: Vec<Row>,
}
#[derive(Deserialize)]
struct Row {
    name: String,
    source_weight: u32,
    weight: u32,
}

pub(crate) fn kept_kind(kind: u32) -> bool {
    matches!(kind, 0..=5 | 145)
}

/// Raw pack profiles remain immutable. The Rust-owned profile uses the scoped
/// table, before any room can serve it. Fixed placements have separate admission.
pub(crate) fn apply(map: &str, drops: &mut Vec<(String, u32)>) -> io::Result<()> {
    let scope: Scope = serde_json::from_str(include_str!("../../../data/g1-item-drops.json"))
        .map_err(io::Error::other)?;
    let Some(table) = scope.maps.into_iter().find(|row| row.name == map) else {
        return Ok(());
    };
    let source: Vec<_> = table
        .rows
        .iter()
        .map(|row| (row.name.clone(), row.source_weight))
        .collect();
    let effective: Vec<_> = table
        .rows
        .into_iter()
        .map(|row| (row.name, row.weight))
        .collect();
    if !drops.is_empty() && *drops != source && *drops != effective {
        return Err(io::Error::other("G1 drop table differs from scoped source"));
    }
    *drops = effective;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_six_tables_keep_source_rows_and_only_measured_groups_can_drop() {
        let data: serde_json::Value =
            serde_json::from_str(include_str!("../../../data/g1-item-drops.json")).unwrap();
        let scope: Scope = serde_json::from_value(data.clone()).unwrap();
        assert_eq!(scope.maps.len(), 6);
        let mut disabled = 0;
        for map in scope.maps {
            let mut drops = map
                .rows
                .iter()
                .map(|r| (r.name.clone(), r.source_weight))
                .collect();
            apply(&map.name, &mut drops).unwrap();
            assert_eq!(drops.len(), map.rows.len());
            for ((name, weight), source) in drops.iter().zip(&map.rows) {
                assert_eq!(name, &source.name);
                if let Some(kind) = data["kept"][name].as_u64() {
                    assert!(kept_kind(kind as u32));
                    assert_eq!(*weight, source.source_weight);
                } else {
                    assert_eq!(*weight, 0, "{}: {name}", map.name);
                    disabled += usize::from(source.source_weight > 0);
                }
            }
            let scoped = drops.clone();
            apply(&map.name, &mut drops).unwrap();
            assert_eq!(drops, scoped);
            drops.push(("Item/ItemBombUnknown".into(), 1));
            assert!(apply(&map.name, &mut drops).is_err());
        }
        assert!(disabled > 50);
        for kind in [6, 7, 144, 146, 65536, u32::MAX] {
            assert!(!kept_kind(kind));
        }
    }
}
