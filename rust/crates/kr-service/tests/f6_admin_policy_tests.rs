use serde_json::{Value, json};
#[cfg(unix)]
use std::os::unix::fs::DirBuilderExt;
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
    process::{Command, Output, Stdio},
};

const DISABLED_PROFILE: &str = include_str!("../../../../server/profiles/f6-events-disabled.json");

const SYSTEMS: [&str; 19] = [
    "CAPass",
    "Câu cá",
    "DIYBomb",
    "DTT",
    "Exchange",
    "GrowSprout",
    "HoneyButterChip",
    "LineFriends",
    "LuckyCard",
    "MagicMarket",
    "MineExplore",
    "MonWars",
    "Nhiệm vụ",
    "Partners",
    "Pet",
    "RetroWriggle",
    "SelectionBox",
    "Sự kiện TakeCareOfEvent",
    "Sự kiện theo tiền tố Event",
];

struct TestDir(PathBuf);

impl TestDir {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!("kr-f6-admin-{}", getrandom::u64().unwrap()));
        let mut builder = fs::DirBuilder::new();
        #[cfg(unix)]
        builder.mode(0o700);
        builder.create(&path).unwrap();
        Self(path)
    }

    fn db(&self) -> PathBuf {
        self.0.join("accounts.sqlite")
    }
}

impl Drop for TestDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn command(db: &Path, command: Value) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_kr-admin"))
        .arg(db)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let request = json!({
        "actor": "on-call",
        "reason": "inspect F6 policy",
        "command": command,
    });
    child
        .stdin
        .take()
        .unwrap()
        .write_all(&serde_json::to_vec(&request).unwrap())
        .unwrap();
    child.wait_with_output().unwrap()
}

fn setup() -> TestDir {
    let dir = TestDir::new();
    let db = dir.db();
    let init = command(&db, json!({"op":"init"}));
    assert!(
        init.status.success(),
        "{}",
        String::from_utf8_lossy(&init.stderr)
    );
    let begin = command(&db, json!({"op":"begin-instance"}));
    assert!(
        begin.status.success(),
        "{}",
        String::from_utf8_lossy(&begin.stderr)
    );
    dir
}

fn check(dir: &TestDir, request: Value) -> Value {
    let output = command(&dir.db(), request);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).unwrap()
}

fn assert_all_disabled(result: &Value, at: i64) {
    assert_eq!(result["schema"], "kre.f6-events-policy.v1");
    assert_eq!(result["default_enabled"], false);
    assert_eq!(result["excluded_systems"], json!(["PCRoomBenefit"]));
    assert_eq!(result["evaluated_at_unix_secs"], at);
    assert_eq!(result["enabled_systems"], json!([]));
    let systems = result["systems"].as_array().unwrap();
    assert_eq!(systems.len(), SYSTEMS.len());
    for (row, name) in systems.iter().zip(SYSTEMS) {
        assert_eq!(row, &json!({"name": name, "enabled": false}));
    }
}

#[test]
fn omitted_policy_path_reports_all_systems_disabled_and_audits_read() {
    let dir = setup();
    let at = 1_798_761_600;
    let result = check(&dir, json!({"op":"f6-policy-check","at_unix_secs":at}));
    assert_all_disabled(&result, at);

    let audit = check(&dir, json!({"op":"audit-list","filter":{"limit":20}}));
    assert!(audit["audit"].as_array().unwrap().iter().any(|row| {
        row["operation"] == "f6_policy_check"
            && row["target_kind"] == "service"
            && row["target_id"] == "f6_policy"
            && row["result"] == "ok"
    }));
}

#[test]
fn checked_in_disabled_profile_reports_all_systems_disabled() {
    let dir = setup();
    let path = fs::canonicalize(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../../server/profiles/f6-events-disabled.json"),
    )
    .unwrap();
    let at = 1_798_761_600;
    let result = check(
        &dir,
        json!({
            "op":"f6-policy-check",
            "policy_path":path,
            "at_unix_secs":at,
        }),
    );
    assert_all_disabled(&result, at);
}

#[test]
fn opted_in_policy_reports_configured_caps_and_inclusive_active_window() {
    let dir = setup();
    let path = dir.0.join("policy.json");
    let mut profile: Value = serde_json::from_str(DISABLED_PROFILE).unwrap();
    profile["systems"]["CAPass"] = json!({
        "enabled": true,
        "start_utc": "2027-01-01T00:00:00Z",
        "end_utc": "2027-01-01T00:01:00Z",
        "rate_caps": {"per_account_per_hour": 6, "global_per_minute": 43},
    });
    profile["systems"]["Pet"] = profile["systems"]["CAPass"].clone();
    profile["systems"]["Pet"]["enabled"] = json!(false);
    fs::write(&path, serde_json::to_vec(&profile).unwrap()).unwrap();

    let start = 1_798_761_600;
    let end = start + 60;
    for (at, active) in [
        (start - 1, false),
        (start, true),
        (end, true),
        (end + 1, false),
    ] {
        let result = check(
            &dir,
            json!({"op":"f6-policy-check", "policy_path":path, "at_unix_secs":at}),
        );
        assert_eq!(result["evaluated_at_unix_secs"], at);
        assert_eq!(
            result["enabled_systems"],
            json!([{
                "name": "CAPass",
                "start_unix_secs": start,
                "end_unix_secs": end,
                "active": active,
                "rate_caps": {"per_account_per_hour": 6, "global_per_minute": 43},
            }])
        );
        let systems = result["systems"].as_array().unwrap();
        assert_eq!(systems.len(), SYSTEMS.len());
        for (row, name) in systems.iter().zip(SYSTEMS) {
            assert_eq!(
                row,
                &json!({"name": name, "enabled": name == "CAPass" && active})
            );
        }
    }
}

#[test]
fn tampered_profiles_fail_closed() {
    let dir = setup();
    let path = dir.0.join("policy.json");
    let original: Value = serde_json::from_str(DISABLED_PROFILE).unwrap();
    let mut tampered = Vec::new();

    let mut excluded = original.clone();
    excluded["systems"]["PCRoomBenefit"] = excluded["systems"]["CAPass"].clone();
    tampered.push(excluded);

    let mut default_enabled = original.clone();
    default_enabled["default_enabled"] = json!(true);
    tampered.push(default_enabled);

    let mut unknown = original;
    unknown["unknown_key"] = json!(true);
    tampered.push(unknown);

    for profile in tampered {
        fs::write(&path, serde_json::to_vec(&profile).unwrap()).unwrap();
        let output = command(
            &dir.db(),
            json!({
                "op":"f6-policy-check",
                "policy_path":path,
                "at_unix_secs":1_798_761_600,
            }),
        );
        assert!(!output.status.success());
        assert!(output.stdout.is_empty());
        assert_eq!(
            String::from_utf8_lossy(&output.stderr).trim(),
            "kr-admin: InvalidInput"
        );
    }
}

#[test]
fn missing_and_symlink_profiles_fail_closed() {
    let dir = setup();
    let missing = dir.0.join("missing.json");
    let missing_output = command(
        &dir.db(),
        json!({"op":"f6-policy-check","policy_path":missing}),
    );
    assert!(!missing_output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&missing_output.stderr).trim(),
        "kr-admin: InvalidInput"
    );

    #[cfg(unix)]
    {
        use std::os::unix::fs::symlink;
        let target = dir.0.join("target.json");
        let link = dir.0.join("policy-link.json");
        fs::write(&target, DISABLED_PROFILE).unwrap();
        symlink(&target, &link).unwrap();
        let output = command(
            &dir.db(),
            json!({"op":"f6-policy-check","policy_path":link}),
        );
        assert!(!output.status.success());
        assert_eq!(
            String::from_utf8_lossy(&output.stderr).trim(),
            "kr-admin: InvalidInput"
        );
    }
}
