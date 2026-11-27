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
        let path = std::env::temp_dir().join(format!("kr-f5-admin-{}", getrandom::u64().unwrap()));
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

fn profile_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(3)
        .unwrap()
        .join("server/profiles/f5-social-chat-disabled.json")
}

fn command(args: &[&OsStr]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_kr-admin"))
        .arg("f5-policy-check")
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

fn expected_disabled_summary() -> Value {
    json!({
        "whisper": false,
        "guild_chat": false,
        "megaphone": false,
        "note_inbox_cap": null,
        "couple": false,
        "plaza_zones": {"count": 0, "names": []},
        "all_disabled": true,
    })
}

#[test]
fn omitted_default_and_checked_in_profile_match() {
    let expected = expected_disabled_summary();
    assert_eq!(read_summary(check(None)), expected);
    let profile = profile_path();
    let contents: Value = serde_json::from_slice(&fs::read(&profile).unwrap()).unwrap();
    assert_eq!(
        contents,
        json!({
            "whisper": false,
            "guild_chat": false,
            "megaphone": null,
            "note_inbox_cap": null,
            "couple": false,
            "plaza_zones": {},
        })
    );
    assert_eq!(read_summary(check(Some(&profile))), expected);
}

#[test]
fn valid_opt_in_reports_enabled_surfaces_and_caps() {
    let dir = TestDir::new();
    let path = dir.0.join("policy.json");
    fs::write(
        &path,
        r#"{
            "whisper": true,
            "guild_chat": true,
            "megaphone": {"global": true, "channel": false, "cooldown_seconds": 2, "window_seconds": 60, "quota": 3},
            "note_inbox_cap": 7,
            "couple": true,
            "plaza_zones": {
                "lobby_square": {"max_occupants": 10, "seats": 4},
                "square": {"max_occupants": 20, "seats": 2}
            }
        }"#,
    )
    .unwrap();
    assert_eq!(
        read_summary(check(Some(&path))),
        json!({
            "whisper": true,
            "guild_chat": true,
            "megaphone": true,
            "note_inbox_cap": 7,
            "couple": true,
            "plaza_zones": {"count": 2, "names": ["lobby_square", "square"]},
            "all_disabled": false,
        })
    );
}

#[test]
fn malformed_unknown_and_out_of_bounds_policies_fail_closed() {
    let dir = TestDir::new();
    let path = dir.0.join("policy.json");
    for contents in [
        "",
        "{",
        "[]",
        "{} {}",
        r#"{"unknown": false}"#,
        r#"{"whisper": "true"}"#,
        r#"{"note_inbox_cap": 101}"#,
        r#"{"megaphone": {"global": false, "channel": false, "cooldown_seconds": 1, "window_seconds": 1, "quota": 1}}"#,
        r#"{"megaphone": {"global": true, "channel": false, "cooldown_seconds": 0, "window_seconds": 1, "quota": 1}}"#,
        r#"{"plaza_zones": {"square": {"max_occupants": 257, "seats": 1}}}"#,
    ] {
        fs::write(&path, contents).unwrap();
        assert_rejected(check(Some(&path)), "f5_social_policy");
    }
}

#[test]
fn invalid_options_and_missing_paths_fail_closed() {
    let dir = TestDir::new();
    assert_rejected(command(&[OsStr::new("--policy")]), "usage:");
    assert_rejected(command(&[OsStr::new("--unknown")]), "usage:");
    assert_rejected(check(Some(Path::new(""))), "invalid f5_social_policy path");
    assert_rejected(check(Some(&dir.0)), "must be a regular file");
    assert_rejected(
        check(Some(&dir.0.join("../policy.json"))),
        "invalid f5_social_policy path",
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
    fs::write(&target, "{}").unwrap();
    assert_eq!(
        read_summary(check(Some(&target))),
        expected_disabled_summary()
    );

    let file_link = dir.0.join("policy-link.json");
    symlink(&target, &file_link).unwrap();
    assert_rejected(
        check(Some(&file_link)),
        "must be a regular file without symlinks",
    );

    let parent_link = dir.0.join("parent-link");
    symlink(&real, &parent_link).unwrap();
    assert_rejected(
        check(Some(&parent_link.join("nested/policy.json"))),
        "parents must be directories without symlinks",
    );
}

fn manage(database: &Path, command: Value) -> Value {
    let mut child = Command::new(env!("CARGO_BIN_EXE_kr-admin"))
        .arg(database)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let request = json!({
        "actor": "on-call",
        "reason": "inspect F5 social policy",
        "command": command,
    });
    serde_json::to_writer(child.stdin.take().unwrap(), &request).unwrap();
    read_summary(child.wait_with_output().unwrap())
}

#[test]
fn database_command_matches_summary_and_records_read_audit() {
    let dir = TestDir::new();
    let database = dir.0.join("accounts.sqlite");
    manage(&database, json!({"op": "init"}));
    let result = manage(&database, json!({"op": "f5-policy-check"}));
    assert_eq!(result, read_summary(check(None)));
    let audit = manage(
        &database,
        json!({"op": "audit-list", "filter": {"limit": 20}}),
    );
    assert!(audit["audit"].as_array().unwrap().iter().any(|row| {
        row["operation"] == "f5_policy_check"
            && row["target_kind"] == "service"
            && row["target_id"] == "f5_policy"
            && row["result"] == "ok"
    }));
}
