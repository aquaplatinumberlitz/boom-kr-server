use kr_service::FixturePin;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{fmt::Write as _, fs, io, path::PathBuf};

struct FixtureDir(PathBuf);

impl FixtureDir {
    fn new() -> Self {
        let mut seed = [0; 8];
        getrandom::fill(&mut seed).unwrap();
        let path = std::env::temp_dir().join(format!(
            "kr-map-profile-probe-{:016x}",
            u64::from_ne_bytes(seed)
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }

    fn validate(&self, profile: &Value) -> io::Result<()> {
        let path = self.0.join("map.json");
        let body = serde_json::to_vec(profile).unwrap();
        fs::write(&path, &body).unwrap();
        let mut sha256 = String::with_capacity(64);
        for byte in Sha256::digest(&body) {
            write!(sha256, "{byte:02x}").unwrap();
        }
        FixturePin {
            path,
            bytes: body.len(),
            sha256,
        }
        .validate_map_profile()
    }
}

impl Drop for FixtureDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn profile(name: &str, code: u16, scripted: bool, spawn_count: u8) -> Value {
    json!({
        "adapter": "f2f3-probe-r1",
        "name": name,
        "map_code": code,
        "width": 15,
        "height": 13,
        "scripted": scripted,
        "disable_bombs": false,
        "spawns": (0..spawn_count).map(|x| [x, 0]).collect::<Vec<_>>(),
        "payload_sha256": "0".repeat(64),
    })
}

#[test]
fn map_profile_loads_verified_f2_probes() {
    let files = FixtureDir::new();
    for (name, code, scripted, spawn_count) in [
        ("Coloring01", 213, true, 4),
        ("Hamster01", 355, true, 8),
        ("Gumiho01", 344, true, 8),
        ("SuperFighter16", 1075, false, 8),
        ("CAPong01", 192, false, 4),
        ("Rabbit01", 920, true, 8),
        ("Wriggle01", 1115, true, 4),
        ("Zzolman01", 1146, true, 4),
        ("PopcadeB01", 910, true, 2),
        ("PopcadeC01", 915, true, 4),
    ] {
        files
            .validate(&profile(name, code, scripted, spawn_count))
            .unwrap_or_else(|err| panic!("{name}: {err}"));
    }
}

#[test]
fn map_profile_loads_verified_f3_probes() {
    let files = FixtureDir::new();
    for (name, code) in [
        ("MonSnowBear07", 666),
        ("MonPen01", 536),
        ("MonTurtle01", 737),
        ("MonGolem01", 501),
    ] {
        files
            .validate(&profile(name, code, true, 4))
            .unwrap_or_else(|err| panic!("{name}: {err}"));
    }
}

#[test]
fn map_profile_rejects_probe_metadata_drift() {
    let files = FixtureDir::new();
    let mon_pen = profile("MonPen01", 536, true, 4);
    let capong = profile("CAPong01", 192, false, 4);
    let popcade = profile("PopcadeB01", 910, true, 2);
    for (base, field, value) in [
        (&mon_pen, "map_code", json!(501)),
        (&mon_pen, "name", json!("MonPen02")),
        (&mon_pen, "name", json!("UnknownFamily01")),
        (&mon_pen, "scripted", json!(false)),
        (&capong, "scripted", json!(true)),
        (&capong, "width", json!(16)),
        (&capong, "disable_bombs", json!(true)),
        (&popcade, "spawns", json!([[0, 0], [1, 0], [2, 0], [3, 0]])),
        (&popcade, "spawns", json!([[0, 0], [0, 0]])),
    ] {
        let mut drift = base.clone();
        drift[field] = value;
        let err = files.validate(&drift).unwrap_err();
        assert_eq!(
            err.to_string(),
            "inconsistent or unsupported pack map profile",
            "{} with {field} = {}",
            base["name"],
            drift[field]
        );
    }
}
