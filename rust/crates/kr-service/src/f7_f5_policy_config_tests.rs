use super::{Config, F5SocialPolicy};
use crate::{custom_map_store::CustomMapPolicy, platform};
use serde_json::json;
use std::{
    fs, io,
    path::{Path, PathBuf},
};

const F7_PROFILE: &str = "../../../server/profiles/f7-custom-map-disabled.json";
const F5_PROFILE: &str = "../../../server/profiles/f5-social-chat-disabled.json";

struct TestDir(PathBuf);

impl TestDir {
    fn new() -> Self {
        let path = fs::canonicalize(std::env::temp_dir())
            .unwrap()
            .join(format!(
                "kr-f7-f5-policy-{:016x}",
                getrandom::u64().unwrap()
            ));
        platform::provision_private_dir(&path).unwrap();
        let dir = Self(path);
        platform::private_options()
            .write(true)
            .create_new(true)
            .open(dir.0.join("accounts.sqlite"))
            .unwrap();
        dir
    }

    fn config(&self, f7: Option<&Path>, f5: Option<&Path>) -> Config {
        let mut value = json!({
            "database": self.0.join("accounts.sqlite"),
            "launch_socket": self.0.join("launch.sock"),
            "launch_ttl_seconds": 60,
        });
        if let Some(path) = f7 {
            value["f7_custom_map_policy"] = json!(path);
        }
        if let Some(path) = f5 {
            value["f5_social_policy"] = json!(path);
        }
        serde_json::from_value(value).unwrap()
    }
}

impl Drop for TestDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn omitted_fields_default_to_disabled() {
    let dir = TestDir::new();
    let config = dir.config(None, None);
    config.validate().unwrap();
    assert_eq!(
        config.f7_custom_map_policy().unwrap(),
        CustomMapPolicy::default()
    );
    assert_eq!(
        config.f5_social_policy().unwrap(),
        F5SocialPolicy::default()
    );
}

#[test]
fn checked_in_profiles_match_disabled_defaults() {
    let dir = TestDir::new();
    let f7 = fs::canonicalize(Path::new(env!("CARGO_MANIFEST_DIR")).join(F7_PROFILE)).unwrap();
    let f5 = fs::canonicalize(Path::new(env!("CARGO_MANIFEST_DIR")).join(F5_PROFILE)).unwrap();
    let config = dir.config(Some(&f7), Some(&f5));
    config.validate().unwrap();
    assert_eq!(
        config.f7_custom_map_policy().unwrap(),
        CustomMapPolicy::default()
    );
    assert_eq!(
        config.f5_social_policy().unwrap(),
        F5SocialPolicy::default()
    );
}

#[test]
fn valid_opt_in_configs_load_accurately() {
    let dir = TestDir::new();
    let f7 = dir.0.join("f7.json");
    fs::write(
        &f7,
        r#"{"enabled":true,"max_maps_per_owner":3,"max_payload_bytes":1024}"#,
    )
    .unwrap();
    let f5 = dir.0.join("f5.json");
    fs::write(&f5, r#"{
        "whisper": true, "guild_chat": true, "couple": true,
        "megaphone": {"global": true, "channel": false, "cooldown_seconds": 2, "window_seconds": 60, "quota": 3},
        "note_inbox_cap": 7,
        "plaza_zones": {"lobby_square": {"max_occupants": 10, "seats": 4}}
    }"#).unwrap();
    let config = dir.config(Some(&f7), Some(&f5));
    config.validate().unwrap();
    assert_eq!(config.f7_custom_map_policy().unwrap().max_maps_per_owner, 3);
    let policy = config.f5_social_policy().unwrap();
    assert!(policy.whisper && policy.guild_chat && policy.couple);
    assert_eq!(policy.megaphone.unwrap().quota, 3);
    assert_eq!(policy.note_inbox_cap.unwrap().get(), 7);
    assert_eq!(policy.plaza_zones.len(), 1);
}

#[cfg(unix)]
#[test]
fn paths_and_policy_errors_fail_closed() {
    use std::os::unix::fs::symlink;
    let dir = TestDir::new();
    let f7 = dir.0.join("f7.json");
    let f5 = dir.0.join("f5.json");
    let cases = [
        ("{", "{}", "invalid json"),
        (r#"{"unknown":false}"#, "{}", "unknown field"),
        (r#"{"enabled":true,"max_maps_per_owner":0}"#, "{}", "bounds"),
        ("{}", "{", "invalid json"),
        ("{}", r#"{"unknown":false}"#, "unknown field"),
        (
            "{}",
            r#"{"megaphone":{"global":false,"channel":false,"cooldown_seconds":1,"window_seconds":1,"quota":1}}"#,
            "bounds",
        ),
    ];
    for (a, b, _) in cases {
        fs::write(&f7, a).unwrap();
        fs::write(&f5, b).unwrap();
        assert_eq!(
            dir.config(Some(&f7), Some(&f5))
                .validate()
                .unwrap_err()
                .kind(),
            io::ErrorKind::InvalidData
        );
    }
    let missing = dir.0.join("missing.json");
    assert_eq!(
        dir.config(Some(&missing), None)
            .validate()
            .unwrap_err()
            .kind(),
        io::ErrorKind::NotFound
    );
    fs::write(&f7, "{}").unwrap();
    fs::write(&f5, "{}").unwrap();
    let link = dir.0.join("link.json");
    symlink(&f7, &link).unwrap();
    assert_eq!(
        dir.config(Some(&link), None).validate().unwrap_err().kind(),
        io::ErrorKind::InvalidInput
    );
}
