use kr_service::custom_map_store::CustomMapPolicy;
use serde_json::{Value, json};
#[cfg(unix)]
use std::os::unix::fs::DirBuilderExt;
use std::{
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
    process::{Command, Output, Stdio},
};

const F6_EXTENDED_SPEC: &str =
    include_str!("../../../../docs/evidence/content/f6-extended-events-spec.json");
const F6_START: i64 = 1_798_761_600;
const F6_END: i64 = F6_START + 60;

struct TestDir(PathBuf);

impl TestDir {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!("kr-f7-admin-{}", getrandom::u64().unwrap()));
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

fn profile_path(filename: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(3)
        .unwrap()
        .join("server/profiles")
        .join(filename)
}

fn f7_command(args: &[&OsStr]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_kr-admin"))
        .arg("f7-policy-check")
        .args(args)
        .stdin(Stdio::null())
        .output()
        .unwrap()
}

fn f7_check(path: &Path) -> Output {
    f7_command(&[OsStr::new("--policy"), path.as_os_str()])
}

fn read_summary(output: Output) -> Value {
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stderr.is_empty());
    serde_json::from_slice(&output.stdout).unwrap()
}

fn assert_rejected(output: Output, diagnostic: &str) {
    assert_eq!(output.status.code(), Some(1), "unexpected exit: {output:?}");
    assert!(output.stdout.is_empty());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(
        stderr.starts_with("kr-admin: ") && stderr.contains(diagnostic),
        "expected {diagnostic:?}: {stderr}"
    );
}

#[test]
fn f7_omitted_policy_and_checked_in_profile_report_disabled_defaults() {
    let defaults = CustomMapPolicy::default();
    let summary = read_summary(f7_command(&[]));
    assert_eq!(
        summary,
        json!({
            "default_enabled": false,
            "enabled": false,
            "max_maps_per_owner": defaults.max_maps_per_owner,
            "max_payload_bytes": defaults.max_payload_bytes,
            "wire_contract": "unmeasured",
        })
    );
    let path = profile_path("f7-custom-map-disabled.json");
    let profile: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    assert_eq!(profile, json!(defaults));
    assert_eq!(read_summary(f7_check(&path)), summary);
}

#[test]
fn f7_opt_in_reports_configured_limits_without_claiming_wire_support() {
    let dir = TestDir::new();
    let path = dir.0.join("policy.json");
    fs::write(
        &path,
        r#"{"enabled":true,"max_maps_per_owner":3,"max_payload_bytes":2048}"#,
    )
    .unwrap();
    assert_eq!(
        read_summary(f7_check(&path)),
        json!({
            "default_enabled": false,
            "enabled": true,
            "max_maps_per_owner": 3,
            "max_payload_bytes": 2048,
            "wire_contract": "unmeasured",
        })
    );
}

#[test]
fn f7_rejects_empty_malformed_unknown_duplicate_and_out_of_range_profiles() {
    let dir = TestDir::new();
    let path = dir.0.join("policy.json");
    for (contents, diagnostic) in [
        ("", "EOF"),
        ("{", "invalid custom map policy JSON"),
        ("[]", "invalid custom map policy JSON"),
        ("{} {}", "trailing characters"),
        (r#"{"enabled":"true"}"#, "invalid type"),
        (r#"{"enable":true}"#, "unknown field"),
        (r#"{"enabled":false,"enabled":true}"#, "duplicate field"),
        (r#"{"max_maps_per_owner":0}"#, "limits are out of bounds"),
        (r#"{"max_maps_per_owner":1025}"#, "limits are out of bounds"),
        (r#"{"max_payload_bytes":0}"#, "limits are out of bounds"),
        (r#"{"max_payload_bytes":8193}"#, "limits are out of bounds"),
    ] {
        fs::write(&path, contents).unwrap();
        assert_rejected(f7_check(&path), diagnostic);
    }
}

#[test]
fn f7_rejects_invalid_options_and_unsafe_paths() {
    let dir = TestDir::new();
    assert_rejected(f7_command(&[OsStr::new("--policy")]), "usage:");
    assert_rejected(f7_command(&[OsStr::new("--unknown")]), "usage:");
    assert_rejected(f7_check(Path::new("")), "invalid f7_custom_map_policy path");
    assert_rejected(
        f7_check(&dir.0.join("../policy.json")),
        "invalid f7_custom_map_policy path",
    );
    assert_rejected(f7_check(&dir.0), "must be a regular file");
    let missing = dir.0.join("missing.json");
    let missing_error = fs::symlink_metadata(&missing).unwrap_err();
    assert_rejected(f7_check(&missing), &missing_error.to_string());
}

#[cfg(unix)]
#[test]
fn f7_rejects_symlink_files_and_parent_directories() {
    use std::os::unix::fs::symlink;

    let dir = TestDir::new();
    let real = dir.0.join("real");
    fs::create_dir_all(real.join("nested")).unwrap();
    let target = real.join("nested/policy.json");
    fs::write(&target, "{}").unwrap();
    assert_eq!(read_summary(f7_check(&target))["enabled"], false);

    let link = dir.0.join("policy-link.json");
    symlink(&target, &link).unwrap();
    assert_rejected(f7_check(&link), "must be a regular file without symlinks");

    let parent_link = dir.0.join("parent-link");
    symlink(&real, &parent_link).unwrap();
    assert_rejected(
        f7_check(&parent_link.join("nested/policy.json")),
        "parents must be directories without symlinks",
    );
}

fn f6_command(db: &Path, command: Value) -> Value {
    let mut child = Command::new(env!("CARGO_BIN_EXE_kr-admin"))
        .arg(db)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let request = json!({
        "actor": "on-call",
        "reason": "inspect F6 extended systems",
        "command": command,
    });
    serde_json::to_writer(child.stdin.take().unwrap(), &request).unwrap();
    read_summary(child.wait_with_output().unwrap())
}

fn assert_f6_extended_summary(db: &Path, path: &Path, at: i64, active: Option<&str>) {
    let result = f6_command(
        db,
        json!({"op":"f6-policy-check", "policy_path":path, "at_unix_secs":at}),
    );
    assert_eq!(result["evaluated_at_unix_secs"], at);
    let spec: Value = serde_json::from_str(F6_EXTENDED_SPEC).unwrap();
    let names = spec["roadmap_only"].as_array().unwrap();
    let rows = result["extended_systems"].as_array().unwrap();
    assert_eq!(rows.len(), 25);
    assert_eq!(rows.len(), names.len());
    for (row, system) in rows.iter().zip(names) {
        let name = system["id"].as_str().unwrap();
        assert_eq!(*row, json!({"name": name, "enabled": active == Some(name)}));
    }
}

#[test]
fn f6_checked_in_profile_reports_all_extended_systems_disabled() {
    let dir = TestDir::new();
    let db = dir.0.join("accounts.sqlite");
    f6_command(&db, json!({"op":"init"}));
    assert_f6_extended_summary(
        &db,
        &profile_path("f6-events-disabled.json"),
        F6_START,
        None,
    );
}

#[test]
fn f6_extended_opt_in_is_enabled_only_during_its_half_open_window() {
    let dir = TestDir::new();
    let db = dir.0.join("accounts.sqlite");
    f6_command(&db, json!({"op":"init"}));

    let disabled_profile = fs::read(profile_path("f6-events-disabled.json")).unwrap();
    let mut profile: Value = serde_json::from_slice(&disabled_profile).unwrap();
    let spec: Value = serde_json::from_str(F6_EXTENDED_SPEC).unwrap();
    let mut disabled = spec["defaults"].clone();
    disabled["max_grants_per_day"] = json!(0);
    disabled["max_grants_per_season"] = json!(0);
    let extended = spec["roadmap_only"]
        .as_array()
        .unwrap()
        .iter()
        .map(|entry| (entry["id"].as_str().unwrap().to_owned(), disabled.clone()))
        .collect();
    profile["extended_systems"] = Value::Object(extended);
    profile["extended_systems"]["attendance"] = json!({
        "enabled": true,
        "start_utc": "2027-01-01T00:00:00Z",
        "end_utc": "2027-01-01T00:01:00Z",
        "rate_caps": {"per_account_per_hour": 6, "global_per_minute": 43},
        "season_key": "winter-2027",
        "max_grants_per_day": 2,
        "max_grants_per_season": 10,
    });
    let path = dir.0.join("f6-policy.json");
    fs::write(&path, serde_json::to_vec(&profile).unwrap()).unwrap();

    for (at, active) in [
        (F6_START - 1, None),
        (F6_START, Some("attendance")),
        (F6_END - 1, Some("attendance")),
        (F6_END, None),
    ] {
        assert_f6_extended_summary(&db, &path, at, active);
    }
}
