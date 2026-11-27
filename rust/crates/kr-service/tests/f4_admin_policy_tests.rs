use serde_json::{Value, json};
#[cfg(unix)]
use std::os::unix::fs::DirBuilderExt;
use std::{
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
    process::{Command, Output, Stdio},
};

const POLICY_NAMES: [&str; 9] = [
    "powerup_kits",
    "booster_tickets",
    "titles",
    "level_rewards",
    "lucci_vault",
    "record_erasers",
    "gift_coupons",
    "second_password",
    "upgrades",
];

struct TestDir(PathBuf);

impl TestDir {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!("kr-f4-admin-{}", getrandom::u64().unwrap()));
        let mut builder = fs::DirBuilder::new();
        #[cfg(unix)]
        builder.mode(0o700);
        builder.create(&path).unwrap();
        Self(path)
    }
}

impl Drop for TestDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn command(args: &[&OsStr]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_kr-admin"))
        .arg("f4-policy-check")
        .args(args)
        .stdin(Stdio::null())
        .output()
        .unwrap()
}

fn check(path: Option<&Path>) -> Output {
    match path {
        Some(path) => command(&[OsStr::new("--policy"), path.as_os_str()]),
        None => command(&[]),
    }
}

fn assert_summary(output: &Output, enabled: &[&str]) {
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stderr.is_empty());
    let result: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(result["default_enabled"], false);
    let policies = result["policies"].as_object().unwrap();
    assert_eq!(policies.len(), POLICY_NAMES.len());
    for name in POLICY_NAMES {
        let expected = enabled.contains(&name);
        assert_eq!(
            policies[name],
            json!({"enabled": expected, "active": expected}),
            "{name}"
        );
    }
}

fn assert_rejected(output: Output, diagnostic: &str) {
    assert!(!output.status.success(), "unexpected success: {output:?}");
    assert!(output.stdout.is_empty());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains(diagnostic),
        "unexpected diagnostic: {stderr}"
    );
}

#[test]
fn omitted_policy_reports_all_nine_disabled() {
    let output = check(None);
    assert_summary(&output, &[]);
    assert_eq!(output.stdout, check(None).stdout);
}

#[test]
fn checked_in_disabled_profile_matches_default_summary() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(3)
        .unwrap()
        .join("server/profiles/f4-extended-disabled.json");
    let profile: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    for name in POLICY_NAMES {
        assert_eq!(profile[name]["enabled"], false, "{name}");
    }
    let output = check(Some(&path));
    assert_summary(&output, &[]);
    assert_eq!(output.stdout, check(None).stdout);
}

#[test]
fn valid_opt_ins_are_reported_and_unset_policies_stay_disabled() {
    let dir = TestDir::new();
    let path = dir.0.join("policy.json");
    let profile = json!({
        "powerup_kits": {
            "enabled": true,
            "kits": [{"sku": "demo-kit", "bonus": {"bombs": 1, "blast": 0, "speed": 0}}]
        },
        "second_password": {"enabled": true},
        "lucci_vault": {
            "enabled": false,
            "base_lucci_cap": 100,
            "expansion_step": 10,
            "max_expansions": 2,
            "max_lucci_cap": 120
        }
    });
    fs::write(&path, profile.to_string()).unwrap();
    assert_summary(&check(Some(&path)), &["powerup_kits", "second_password"]);
}

#[test]
fn malformed_unknown_and_duplicate_fields_fail_closed() {
    let dir = TestDir::new();
    let path = dir.0.join("policy.json");
    for invalid in [
        "{",
        "[]",
        "{} {}",
        r#"{"powerup_kit":{"enabled":false}}"#,
        r#"{"titles":{"enable":true}}"#,
        r#"{"titles":{},"titles":{}}"#,
        r#"{"second_password":{"enabled":false,"enabled":true}}"#,
    ] {
        fs::write(&path, invalid).unwrap();
        assert_rejected(
            check(Some(&path)),
            "invalid f4_extended_policy configuration",
        );
    }
}

#[test]
fn enabled_conflicts_fail_closed() {
    let dir = TestDir::new();
    let path = dir.0.join("policy.json");
    for invalid in [
        r#"{"booster_tickets":{"enabled":true}}"#,
        r#"{"lucci_vault":{"enabled":true,"base_lucci_cap":100}}"#,
        r#"{"record_erasers":{"enabled":true,"require_audit":false}}"#,
        r#"{"upgrades":{"enabled":true,"recipes":[]}}"#,
    ] {
        fs::write(&path, invalid).unwrap();
        assert_rejected(
            check(Some(&path)),
            "invalid f4_extended_policy configuration",
        );
    }
}

#[test]
fn invalid_options_and_unsafe_paths_fail_closed() {
    let dir = TestDir::new();
    let path = dir.0.join("policy.json");
    fs::write(&path, "{}").unwrap();
    assert_rejected(command(&[OsStr::new("--policy")]), "usage:");
    assert_rejected(
        command(&[OsStr::new("--unknown"), path.as_os_str()]),
        "usage:",
    );
    assert_rejected(
        check(Some(Path::new(""))),
        "invalid f4_extended_policy path",
    );
    assert_rejected(check(Some(&dir.0)), "must be a regular file");
    assert_rejected(
        check(Some(&dir.0.join("../missing.json"))),
        "invalid f4_extended_policy path",
    );
    assert!(!check(Some(&dir.0.join("missing.json"))).status.success());
}

#[cfg(unix)]
#[test]
fn symlink_file_and_parent_directory_fail_closed() {
    use std::os::unix::fs::symlink;

    let dir = TestDir::new();
    let real = dir.0.join("real");
    fs::create_dir_all(real.join("nested")).unwrap();
    let target = real.join("nested/policy.json");
    fs::write(&target, "{}").unwrap();
    assert_summary(&check(Some(&target)), &[]);

    let file_link = dir.0.join("policy-link.json");
    symlink(&target, &file_link).unwrap();
    assert_rejected(
        check(Some(&file_link)),
        "must be a regular file without symlinks",
    );

    let dir_link = dir.0.join("dir-link");
    symlink(&real, &dir_link).unwrap();
    for suffix in ["nested/policy.json", "./nested/policy.json"] {
        assert_rejected(
            check(Some(&dir_link.join(suffix))),
            "parents must be directories without symlinks",
        );
    }
}
