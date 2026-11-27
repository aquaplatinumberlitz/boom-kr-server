use super::{Config, F3PvePolicy};
use crate::platform;
use serde_json::json;
use std::{
    fs, io,
    path::{Path, PathBuf},
};

const DISABLED_PROFILE: &str = "../../../server/profiles/f3-pve-disabled.json";

struct TestDir(PathBuf);

impl TestDir {
    fn new() -> Self {
        let path = fs::canonicalize(std::env::temp_dir())
            .unwrap()
            .join(format!("kr-f3-policy-{:016x}", getrandom::u64().unwrap()));
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
            value["f3_pve_policy"] = json!(path);
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
fn omitted_profile_defaults_to_disabled() {
    let dir = TestDir::new();
    let config = dir.config(None);
    assert!(config.f3_pve_policy.is_none());
    config.validate().unwrap();
    assert_eq!(
        config.f3_pve_policy().unwrap(),
        F3PvePolicy { enabled: false }
    );
}

#[test]
fn checked_in_profile_defaults_to_disabled() {
    let dir = TestDir::new();
    let path =
        fs::canonicalize(Path::new(env!("CARGO_MANIFEST_DIR")).join(DISABLED_PROFILE)).unwrap();
    let config = dir.config(Some(&path));
    config.validate().unwrap();
    assert_eq!(
        config.f3_pve_policy().unwrap(),
        F3PvePolicy { enabled: false }
    );
}

#[test]
fn configured_profile_can_opt_in() {
    let dir = TestDir::new();
    let path = dir.0.join("policy.json");
    fs::write(&path, r#"{"enabled": true}"#).unwrap();

    let config = dir.config(Some(&path));
    config.validate().unwrap();
    assert!(config.f3_pve_policy().unwrap().enabled);
}

#[test]
fn invalid_policy_json_fails_config_validation() {
    let dir = TestDir::new();
    let path = dir.0.join("policy.json");
    let config = dir.config(Some(&path));
    for (contents, reason) in [
        ("", None),
        ("{", None),
        ("[true]", None),
        ("null", None),
        ("{} {}", None),
        (r#"{"enabled": "true"}"#, None),
        (
            r#"{"enabled": true, "unknown": false}"#,
            Some("unknown field"),
        ),
        (
            r#"{"enabled": true, "enabled": false}"#,
            Some("duplicate field"),
        ),
        (
            r#"{"enabled": true, "enabl\u0065d": false}"#,
            Some("duplicate field"),
        ),
    ] {
        fs::write(&path, contents).unwrap();
        let error = config.validate().unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::InvalidData, "{contents:?}");
        if let Some(reason) = reason {
            assert!(error.to_string().contains(reason), "{contents:?}: {error}");
        }
    }
}

#[test]
fn invalid_policy_paths_fail_config_validation() {
    let dir = TestDir::new();
    fs::write(dir.0.join("policy.json"), "{}").unwrap();
    let subdir = dir.0.join("profiles");
    fs::create_dir(&subdir).unwrap();

    for (path, kind) in [
        (PathBuf::new(), io::ErrorKind::InvalidInput),
        (dir.0.join("missing.json"), io::ErrorKind::NotFound),
        (subdir.join("../policy.json"), io::ErrorKind::InvalidInput),
        (subdir, io::ErrorKind::InvalidInput),
    ] {
        let error = dir.config(Some(&path)).validate().unwrap_err();
        assert_eq!(error.kind(), kind, "{path:?}: {error}");
    }
}

#[cfg(unix)]
#[test]
fn symlink_file_and_ancestor_paths_fail_config_validation() {
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
