use kr_service::f6_policy::F6EventPolicy;
use serde_json::{Value, json};
use std::{
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
    process::{Command, Output, Stdio},
};

const F2: &str = "f2-modes-status";
const F4: &str = "f4-security-kits-status";
const F6: &str = "f6-events-summary";

struct TestDir(PathBuf);

impl TestDir {
    fn new() -> Self {
        let path = std::env::temp_dir()
            .canonicalize()
            .unwrap()
            .join(format!("kr-admin-status-{}", getrandom::u64().unwrap()));
        fs::create_dir(&path).unwrap();
        Self(path)
    }

    fn write(&self, name: &str, contents: &str) -> PathBuf {
        let path = self.0.join(name);
        fs::write(&path, contents).unwrap();
        path
    }

    fn run_with_policy(&self, subcommand: &str, path: &Path) -> Output {
        let output = run(subcommand, [OsStr::new("--policy"), path.as_os_str()]);
        let root = self.0.to_string_lossy();
        for stream in [&output.stdout, &output.stderr] {
            assert!(!String::from_utf8_lossy(stream).contains(root.as_ref()));
        }
        output
    }
}

impl Drop for TestDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn run(subcommand: &str, args: impl IntoIterator<Item = impl AsRef<OsStr>>) -> Output {
    Command::new(env!("CARGO_BIN_EXE_kr-admin"))
        .arg(subcommand)
        .args(args)
        .stdin(Stdio::null())
        .output()
        .unwrap()
}

fn summary(output: Output) -> Value {
    assert!(output.status.success(), "{output:?}");
    assert!(output.stderr.is_empty(), "{output:?}");
    serde_json::from_slice(&output.stdout).unwrap()
}

fn rejected(output: Output, diagnostic: &str) {
    assert_eq!(output.status.code(), Some(1), "{output:?}");
    assert!(output.stdout.is_empty(), "{output:?}");
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(
        stderr.starts_with("kr-admin: ") && stderr.contains(diagnostic),
        "expected {diagnostic:?}: {stderr}"
    );
}

#[test]
fn f2_reports_catalog_coverage_and_dispatch_status() {
    let value = summary(run(F2, [] as [&str; 0]));
    let families = value["exported_families"].as_array().unwrap();
    assert_eq!(families.len(), 15);
    assert!(
        families
            .windows(2)
            .all(|pair| pair[0].as_str() < pair[1].as_str())
    );
    assert_eq!(families[0], "CAPong");
    assert_eq!(families[14], "Zzolman");
    assert_eq!(value["exported_map_count"], 97);
    assert_eq!(value["total_catalog_families"], 18);
    assert_eq!(value["total_catalog_maps"], 122);
    assert_eq!(value["tentative_family_count"], 21);
    assert_eq!(
        value["live_dispatch_status"],
        "opt_in_fail_closed_pending_wine_capture"
    );
}

#[test]
fn f4_reports_the_disabled_defaults_and_validated_runtime_config() {
    let defaults = summary(run(F4, [] as [&str; 0]));
    assert_eq!(
        defaults,
        json!({
            "default_enabled": false,
            "policy_enabled": {"powerup_kits": false, "second_password": false},
            "powerup_kits_non_empty": false,
            "snapshot": {
                "powerup_kits": {"enabled": false, "tracked_players": 0, "max_players": 1024},
                "second_password": {"enabled": false, "tracked_players": 0, "max_players": 1024},
                "max_unlocked_sessions_per_player": 8,
            },
        })
    );

    let dir = TestDir::new();
    let path = dir.write(
        "f4.json",
        r#"{
            "powerup_kits": {
                "enabled": true,
                "kits": [{"sku": "test-kit", "bonus": {"bombs": 1, "blast": 0, "speed": 0}}]
            },
            "second_password": {"enabled": true}
        }"#,
    );
    let custom = summary(dir.run_with_policy(F4, &path));
    assert_eq!(custom["powerup_kits_non_empty"], true);
    for key in ["powerup_kits", "second_password"] {
        assert_eq!(custom["policy_enabled"][key], true);
        assert_eq!(custom["snapshot"][key]["enabled"], true);
        assert_eq!(custom["snapshot"][key]["tracked_players"], 0);
    }

    let path = dir.write("empty-kits.json", r#"{"powerup_kits":{"enabled":true}}"#);
    rejected(
        dir.run_with_policy(F4, &path),
        "invalid f4_extended_policy configuration",
    );
}

#[test]
fn f6_counts_explicit_opt_ins_and_reports_the_catalog_exclusion() {
    let defaults = summary(run(F6, [] as [&str; 0]));
    assert_eq!(defaults["schema"], "kre.f6-events-policy.v1");
    assert_eq!(defaults["enabled"], 0);
    assert_eq!(defaults["extended"], 0);
    assert_eq!(
        defaults["system_names"],
        json!(F6EventPolicy::system_names())
    );
    assert_eq!(defaults["pc_room_benefit_excluded"], true);
    assert!(
        !defaults["system_names"]
            .as_array()
            .unwrap()
            .contains(&json!("PCRoomBenefit"))
    );

    let mut profile: Value = serde_json::from_str(include_str!(
        "../../../../server/profiles/f6-events-disabled.json"
    ))
    .unwrap();
    profile["systems"]["CAPass"] = json!({
        "enabled": true, "start_utc": "2000-01-01T00:00:00Z", "end_utc": "2000-01-02T00:00:00Z",
        "rate_caps": {"per_account_per_hour": 1, "global_per_minute": 10},
    });
    profile["extended_systems"] = Value::Object(
        F6EventPolicy::extended_system_names()
            .iter()
            .map(|&name| {
                (
                    name.to_owned(),
                    json!({
                        "enabled": false, "start_utc": null, "end_utc": null,
                        "rate_caps": {"per_account_per_hour": 0, "global_per_minute": 0},
                        "max_grants_per_day": 0, "max_grants_per_season": 0,
                    }),
                )
            })
            .collect(),
    );
    profile["extended_systems"]["attendance"] = json!({
        "enabled": true, "start_utc": "2000-01-01T00:00:00Z", "end_utc": "2000-01-02T00:00:00Z",
        "rate_caps": {"per_account_per_hour": 1, "global_per_minute": 10},
        "season_key": "test-season", "max_grants_per_day": 1, "max_grants_per_season": 10,
    });

    let dir = TestDir::new();
    let path = dir.write("f6.json", &profile.to_string());
    let custom = summary(dir.run_with_policy(F6, &path));
    assert_eq!(custom["enabled"], 1);
    assert_eq!(custom["extended"], 1);
    assert_eq!(custom["systems"], json!(["CAPass"]));
    assert_eq!(custom["extended_systems"], json!(["attendance"]));
    assert_eq!(custom["pc_room_benefit_excluded"], true);
}

#[test]
fn rejects_wrong_arguments_with_subcommand_usage() {
    rejected(run(F2, ["extra"]), "usage: kr-admin f2-modes-status");
    for subcommand in [F4, F6] {
        for args in [
            &["--policy"][..],
            &["--unexpected", "value"],
            &["--policy", "file.json", "extra"],
        ] {
            rejected(
                run(subcommand, args),
                &format!("usage: kr-admin {subcommand} [--policy PATH]"),
            );
        }
    }
}

#[test]
fn rejects_missing_and_malformed_policy_files() {
    let dir = TestDir::new();
    let missing = dir.0.join("missing.json");
    let malformed = dir.write("malformed.json", "{");
    for (subcommand, diagnostic) in [
        (F4, "invalid f4_extended_policy configuration"),
        (F6, "EOF while parsing"),
    ] {
        rejected(dir.run_with_policy(subcommand, &missing), "os error");
        rejected(dir.run_with_policy(subcommand, &malformed), diagnostic);
    }
}
