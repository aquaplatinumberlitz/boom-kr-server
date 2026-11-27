use serde_json::{Value, json};
#[cfg(unix)]
use std::os::unix::fs::DirBuilderExt;
use std::{
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
    process::{Command, Output, Stdio},
};

struct TestDir(PathBuf);

impl TestDir {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!("kr-f3-admin-{}", getrandom::u64().unwrap()));
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
        .arg("f3-policy-check")
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

fn manage(db: &Path, command: Value) -> Value {
    let mut child = Command::new(env!("CARGO_BIN_EXE_kr-admin"))
        .arg(db)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let request = json!({
        "actor": "on-call",
        "reason": "inspect F3 policy",
        "command": command,
    });
    serde_json::to_writer(child.stdin.take().unwrap(), &request).unwrap();
    read_summary(child.wait_with_output().unwrap())
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

fn assert_summary(result: &Value, enabled: bool, dispatch: &str) {
    assert_eq!(result["default_enabled"], false);
    assert_eq!(result["enabled"], enabled);
    assert_eq!(result["authority_default"], "unmeasured");
    assert_eq!(result["dispatch"]["monster_s1"], dispatch);
    assert_eq!(
        result["monster_attack_guard"],
        json!({
            "authority": "host_candidate",
            "requires_current_host": true,
            "requires_current_round_epoch": true,
            "requires_increasing_attack_sequence": true,
        })
    );
}

fn assert_rejected(output: Output, diagnostic: &str) {
    assert_eq!(output.status.code(), Some(1), "unexpected exit: {output:?}");
    assert!(output.stdout.is_empty());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(
        stderr.starts_with("kr-admin: ") && stderr.contains(diagnostic),
        "{stderr}"
    );
}

#[test]
fn omitted_policy_reports_disabled() {
    assert_summary(&read_summary(check(None)), false, "disabled");
}

#[test]
fn checked_in_disabled_profile_matches_default_summary() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(3)
        .unwrap()
        .join("server/profiles/f3-pve-disabled.json");
    let profile: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    assert_eq!(profile, json!({"enabled": false}));
    assert_eq!(read_summary(check(Some(&path))), read_summary(check(None)));
}

#[test]
fn valid_opt_in_still_requires_dispatch_evidence() {
    let dir = TestDir::new();
    let path = dir.0.join("policy.json");
    fs::write(&path, r#"{"enabled": true}"#).unwrap();
    assert_summary(
        &read_summary(check(Some(&path))),
        true,
        "wine_evidence_required",
    );
}

#[test]
fn invalid_policy_contents_fail_closed() {
    let dir = TestDir::new();
    let path = dir.0.join("policy.json");
    for invalid in [
        "",
        "{",
        "[]",
        "{} {}",
        r#"{"enabled": "true"}"#,
        r#"{"enabled": false, "extra": true}"#,
        r#"{"enabled": false, "enabled": true}"#,
    ] {
        fs::write(&path, invalid).unwrap();
        assert_rejected(check(Some(&path)), "invalid f3_pve_policy configuration");
    }
}

#[test]
fn invalid_options_and_unsafe_paths_fail_closed() {
    let dir = TestDir::new();
    assert_rejected(command(&[OsStr::new("--policy")]), "usage:");
    assert_rejected(command(&[OsStr::new("--unknown")]), "usage:");
    assert_rejected(check(Some(Path::new(""))), "invalid f3_pve_policy path");
    assert_rejected(check(Some(&dir.0)), "must be a regular file");
    assert_rejected(
        check(Some(&dir.0.join("../policy.json"))),
        "invalid f3_pve_policy path",
    );
    let missing = dir.0.join("missing.json");
    let not_found = fs::symlink_metadata(&missing).unwrap_err().to_string();
    assert_rejected(check(Some(&missing)), &not_found);
}

#[cfg(unix)]
#[test]
fn symlink_file_and_parent_fail_closed() {
    use std::os::unix::fs::symlink;

    let dir = TestDir::new();
    let real = dir.0.join("real");
    fs::create_dir_all(real.join("nested")).unwrap();
    let target = real.join("nested/policy.json");
    fs::write(&target, r#"{"enabled": false}"#).unwrap();
    assert_summary(&read_summary(check(Some(&target))), false, "disabled");

    let file_link = dir.0.join("policy-link.json");
    symlink(&target, &file_link).unwrap();
    assert_rejected(
        check(Some(&file_link)),
        "must be a regular file without symlinks",
    );

    let dir_link = dir.0.join("dir-link");
    symlink(&real, &dir_link).unwrap();
    assert_rejected(
        check(Some(&dir_link.join("nested/policy.json"))),
        "parents must be directories without symlinks",
    );
}

#[test]
fn database_command_reports_same_summary_and_audits_read() {
    let dir = TestDir::new();
    let db = dir.0.join("accounts.sqlite");
    manage(&db, json!({"op": "init"}));
    let summary = manage(&db, json!({"op": "f3-policy-check"}));
    assert_eq!(summary, read_summary(check(None)));

    let audit = manage(&db, json!({"op": "audit-list", "filter": {"limit": 20}}));
    assert!(audit["audit"].as_array().unwrap().iter().any(|row| {
        row["operation"] == "f3_policy_check"
            && row["target_kind"] == "service"
            && row["target_id"] == "f3_policy"
            && row["result"] == "ok"
    }));
}
