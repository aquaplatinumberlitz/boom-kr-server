use super::{Config, F4ExtendedPolicyBundle};
use crate::platform;
use serde_json::json;
use std::{
    fs, io,
    path::{Path, PathBuf},
};

const DISABLED_PROFILE: &str = "../../../server/profiles/f4-extended-disabled.json";

struct TestDir(PathBuf);

impl TestDir {
    fn new() -> Self {
        let path = fs::canonicalize(std::env::temp_dir())
            .unwrap()
            .join(format!("kr-f4-policy-{:016x}", getrandom::u64().unwrap()));
        platform::provision_private_dir(&path).unwrap();
        let dir = Self(path);
        platform::private_options()
            .write(true)
            .create_new(true)
            .open(dir.0.join("accounts.sqlite"))
            .unwrap();
        dir
    }

    fn config(&self, policy_path: Option<&Path>) -> Config {
        let mut value = json!({
            "database": self.0.join("accounts.sqlite"),
            "launch_socket": self.0.join("launch.sock"),
            "launch_ttl_seconds": 60,
        });
        if let Some(path) = policy_path {
            value["f4_extended_policy"] = json!(path);
        }
        serde_json::from_value(value).unwrap()
    }
}

impl Drop for TestDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn assert_all_disabled(policy: &F4ExtendedPolicyBundle) {
    assert!(!policy.powerup_kits.enabled);
    assert!(!policy.booster_tickets.enabled);
    assert!(!policy.titles.enabled);
    assert!(!policy.level_rewards.enabled);
    assert!(!policy.lucci_vault.enabled);
    assert!(!policy.record_erasers.enabled);
    assert!(!policy.gift_coupons.enabled);
    assert!(!policy.second_password.enabled);
    assert!(!policy.upgrades.enabled);
}

#[test]
fn omitted_profile_disables_all_nine_policies() {
    let dir = TestDir::new();
    let config = dir.config(None);
    assert!(config.f4_extended_policy.is_none());
    config.validate().unwrap();
    assert_all_disabled(&config.f4_extended_policy().unwrap());
}

#[test]
fn checked_in_profile_matches_omitted_profile() {
    let dir = TestDir::new();
    let path =
        fs::canonicalize(Path::new(env!("CARGO_MANIFEST_DIR")).join(DISABLED_PROFILE)).unwrap();
    let config = dir.config(Some(&path));
    config.validate().unwrap();
    assert_eq!(
        config.f4_extended_policy().unwrap(),
        dir.config(None).f4_extended_policy().unwrap(),
    );
}

#[test]
fn valid_opt_in_loads_with_other_policies_disabled() {
    let dir = TestDir::new();
    let path = dir.0.join("policy.json");
    fs::write(
        &path,
        r#"{
            "powerup_kits": {
                "enabled": true,
                "max_equipped_slots": 1,
                "kits": [{"sku": "kit.test", "bonus": {"bombs": 1, "blast": 0, "speed": 0}}]
            }
        }"#,
    )
    .unwrap();

    let config = dir.config(Some(&path));
    config.validate().unwrap();
    let policy = config.f4_extended_policy().unwrap();
    assert!(policy.powerup_kits.enabled);
    assert_eq!(policy.powerup_kits.max_equipped_slots, 1);
    assert_eq!(policy.powerup_kits.kits[0].sku, "kit.test");
    assert_eq!(policy.powerup_kits.kits[0].bonus.bombs, 1);
    assert_all_disabled(&F4ExtendedPolicyBundle {
        powerup_kits: Default::default(),
        ..policy
    });
}

#[test]
fn empty_malformed_and_non_object_json_are_rejected() {
    let dir = TestDir::new();
    let path = dir.0.join("policy.json");
    let config = dir.config(Some(&path));
    for contents in ["", " \n", "{", "[]", "null", "{} {}"] {
        fs::write(&path, contents).unwrap();
        let error = config.validate().unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::InvalidData, "{contents:?}");
    }
}

#[test]
fn unknown_and_duplicate_fields_are_rejected_at_each_level() {
    let dir = TestDir::new();
    let path = dir.0.join("policy.json");
    let config = dir.config(Some(&path));
    for (contents, reason) in [
        (r#"{"unknown": {}}"#, "unknown field"),
        (r#"{"titles": {"unknown": false}}"#, "unknown field"),
        (
            r#"{"powerup_kits":{"kits":[{"sku":"kit","bonus":{"bombs":1,"blast":0,"speed":0,"extra":0}}]}}"#,
            "unknown field",
        ),
        (r#"{"titles": {}, "titles": {}}"#, "duplicate field"),
        (
            r#"{"titles": {"enabled": true, "enabl\u0065d": false}}"#,
            "duplicate field",
        ),
        (
            r#"{"powerup_kits":{"kits":[{"sku":"kit","bonus":{"bombs":1,"blast":0,"speed":0,"bombs":2}}]}}"#,
            "duplicate field",
        ),
    ] {
        fs::write(&path, contents).unwrap();
        let error = config.validate().unwrap_err();
        assert_eq!(
            error.kind(),
            io::ErrorKind::InvalidData,
            "{contents}: {error}"
        );
        assert!(error.to_string().contains(reason), "{contents}: {error}");
    }
}

#[test]
fn invalid_settings_in_each_policy_reject_the_profile() {
    let dir = TestDir::new();
    let path = dir.0.join("policy.json");
    let config = dir.config(Some(&path));
    for (section, settings) in [
        ("powerup_kits", json!({"enabled": true})),
        ("booster_tickets", json!({"enabled": true})),
        ("titles", json!({"enabled": true})),
        ("level_rewards", json!({"enabled": true})),
        ("lucci_vault", json!({"enabled": true})),
        ("record_erasers", json!({"require_audit": false})),
        ("gift_coupons", json!({"enabled": true})),
        ("second_password", json!({"max_failed_attempts": 0})),
        ("upgrades", json!({"enabled": true})),
    ] {
        fs::write(&path, json!({(section): settings}).to_string()).unwrap();
        let error = config.validate().unwrap_err();
        assert_eq!(
            error.kind(),
            io::ErrorKind::InvalidData,
            "{section}: {error}"
        );
        assert!(
            error
                .to_string()
                .contains("invalid F4 policy configuration")
        );
    }
}

#[test]
fn empty_traversing_directory_and_missing_paths_are_rejected() {
    let dir = TestDir::new();
    let policy = dir.0.join("policy.json");
    fs::write(&policy, "{}").unwrap();
    let subdir = dir.0.join("profiles");
    fs::create_dir(&subdir).unwrap();
    for (path, kind) in [
        (PathBuf::new(), io::ErrorKind::InvalidInput),
        (
            PathBuf::from("profiles/../policy.json"),
            io::ErrorKind::InvalidInput,
        ),
        (subdir.join("../policy.json"), io::ErrorKind::InvalidInput),
        (subdir, io::ErrorKind::InvalidInput),
        (dir.0.join("missing.json"), io::ErrorKind::NotFound),
    ] {
        let error = dir.config(Some(&path)).validate().unwrap_err();
        assert_eq!(error.kind(), kind, "{path:?}: {error}");
    }
}

#[cfg(unix)]
#[test]
fn symlink_files_and_directories_are_rejected() {
    use std::os::unix::fs::symlink;

    let dir = TestDir::new();
    let profiles = dir.0.join("profiles");
    let nested = profiles.join("nested");
    fs::create_dir_all(&nested).unwrap();
    let target = nested.join("policy.json");
    fs::write(&target, "{}").unwrap();
    let file_link = dir.0.join("policy-link.json");
    let parent_link = dir.0.join("nested-link");
    let ancestor_link = dir.0.join("profiles-link");
    symlink(&target, &file_link).unwrap();
    symlink(&nested, &parent_link).unwrap();
    symlink(&profiles, &ancestor_link).unwrap();

    for path in [
        file_link,
        parent_link.join("policy.json"),
        ancestor_link.join("nested/policy.json"),
    ] {
        let error = dir.config(Some(&path)).validate().unwrap_err();
        assert_eq!(
            error.kind(),
            io::ErrorKind::InvalidInput,
            "{path:?}: {error}"
        );
        assert!(error.to_string().contains("without symlinks"), "{error}");
    }
}
