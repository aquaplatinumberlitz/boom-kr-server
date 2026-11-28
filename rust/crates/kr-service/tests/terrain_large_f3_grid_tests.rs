// Terrain is crate-private; exercise the same source without exposing the module.
#[path = "../src/terrain.rs"]
#[allow(dead_code)]
mod terrain;

use serde::Deserialize;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::fmt::Write as _;
use terrain::{
    BIG_BATTLE_SIZE, Bomb, Config, F3_LARGE_GRID_SIZES, STANDARD_SIZE, is_pinned_f3_large_grid,
};

const MAP: &[u8] = b"map fixture";
const START: &[u8] = b"start fixture";
const PAYLOAD_SHA256: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

fn sha256_hex(data: &[u8]) -> String {
    let mut out = String::with_capacity(64);
    for byte in Sha256::digest(data) {
        write!(&mut out, "{byte:02x}").unwrap();
    }
    out
}

fn profile(width: u8, height: u8) -> Value {
    let kinds = vec![0u8; usize::from(width) * usize::from(height)];
    json!({
        "width": width,
        "height": height,
        "seed_sha256": sha256_hex(&kinds),
        "kinds": kinds,
        "map_sha256": PAYLOAD_SHA256,
        "fixtures": {"map": sha256_hex(MAP), "start": sha256_hex(START)},
        "maximum_power": 2,
    })
}

#[test]
fn default_grid_validation_is_unchanged() {
    assert_eq!(STANDARD_SIZE, (15, 13));
    assert_eq!(BIG_BATTLE_SIZE, (19, 17));
    for (width, height) in [STANDARD_SIZE, BIG_BATTLE_SIZE] {
        let cfg: Config = serde_json::from_value(profile(width, height)).unwrap();
        assert_eq!(
            cfg.validate(MAP, START).unwrap().grid_size(),
            (width, height)
        );
        assert_eq!(
            cfg.validate_for_map(MAP, START, PAYLOAD_SHA256)
                .unwrap()
                .grid_size(),
            (width, height)
        );
        assert!(!is_pinned_f3_large_grid(width, height));
        assert!(
            cfg.validate_large_f3_grid(MAP, START, PAYLOAD_SHA256)
                .is_err()
        );
    }
}

#[test]
fn every_grid_in_the_kr_census_validates_only_with_opt_in() {
    #[derive(Deserialize)]
    struct Census {
        dimensions: Vec<Dimension>,
    }
    #[derive(Deserialize)]
    struct Dimension {
        width: u8,
        height: u8,
        large: bool,
    }
    let census: Census = serde_json::from_str(include_str!(
        "../../../../docs/evidence/content/f3-pve-large-grid-re.json"
    ))
    .unwrap();
    let expected: Vec<_> = census
        .dimensions
        .into_iter()
        .filter(|dim| dim.large)
        .map(|dim| (dim.width, dim.height))
        .collect();
    assert_eq!(expected.len(), 35);
    assert_eq!(F3_LARGE_GRID_SIZES.as_slice(), expected);

    for (width, height) in expected {
        let cfg: Config = serde_json::from_value(profile(width, height)).unwrap();
        assert!(is_pinned_f3_large_grid(width, height), "{width}x{height}");
        assert!(cfg.validate(MAP, START).is_err(), "{width}x{height}");
        assert!(cfg.validate_for_map(MAP, START, PAYLOAD_SHA256).is_err());
        let mut grid = cfg
            .validate_large_f3_grid(MAP, START, PAYLOAD_SHA256)
            .unwrap_or_else(|err| panic!("{width}x{height}: {err}"));
        assert_eq!(grid.grid_size(), (width, height));

        grid.begin(1);
        let edge = Bomb {
            id: 1,
            owner: 0,
            row: height - 1,
            column: width - 1,
            power: 1,
        };
        assert!(
            grid.place(Bomb {
                row: height,
                ..edge
            })
            .is_err()
        );
        assert!(
            grid.place(Bomb {
                column: width,
                ..edge
            })
            .is_err()
        );
        grid.place(edge).unwrap();
        assert_eq!(grid.blast_committed(1, 1).processed, [1]);
    }
}

#[test]
fn unpinned_grid_dimensions_fail_closed() {
    for (width, height) in [
        (0, 0),
        (0, 51),
        (100, 0),
        (101, 10),
        (15, 52),
        (u8::MAX, 10),
        (15, u8::MAX),
        (1, 1),
        (20, 14),
        (24, 25),
        (29, 37),
        (50, 20), // Same cell count as 100x10.
    ] {
        let cfg: Config = serde_json::from_value(profile(width, height)).unwrap();
        assert!(!is_pinned_f3_large_grid(width, height), "{width}x{height}");
        assert!(
            cfg.validate_large_f3_grid(MAP, START, PAYLOAD_SHA256)
                .is_err(),
            "{width}x{height}"
        );
    }
}

#[test]
fn opt_in_still_requires_matching_fixtures_and_grid_layers() {
    let mut data = profile(24, 24);
    let cfg: Config = serde_json::from_value(data.clone()).unwrap();
    for (map, start, payload) in [
        (b"other".as_slice(), START, PAYLOAD_SHA256),
        (MAP, b"other".as_slice(), PAYLOAD_SHA256),
        (MAP, START, &"0".repeat(64)),
    ] {
        assert!(cfg.validate_large_f3_grid(map, start, payload).is_err());
    }

    data["ray_barriers"] = json!([0]);
    let cfg: Config = serde_json::from_value(data).unwrap();
    assert!(
        cfg.validate_large_f3_grid(MAP, START, PAYLOAD_SHA256)
            .is_err()
    );

    let mut data = profile(24, 24);
    let short_kinds = vec![0u8; 24 * 24 - 1];
    data["seed_sha256"] = json!(sha256_hex(&short_kinds));
    data["kinds"] = json!(short_kinds);
    let cfg: Config = serde_json::from_value(data).unwrap();
    assert!(
        cfg.validate_large_f3_grid(MAP, START, PAYLOAD_SHA256)
            .is_err()
    );
}
