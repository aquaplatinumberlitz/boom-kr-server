use super::{Config, Owner};
use crate::{
    f6_policy::{F6EventPolicy, F6QuotaTracker, QuotaRejection},
    platform,
};
use serde_json::{Value, json};
use std::{
    fs, io,
    path::{Path, PathBuf},
};

const CHECKED_IN: &str = "../../../server/profiles/f6-events-disabled.json";
const DISABLED: &str = include_str!("../../../../server/profiles/f6-events-disabled.json");
const START: i64 = 1_798_761_600;

struct TestDir(PathBuf);

impl TestDir {
    fn new() -> Self {
        let root = fs::canonicalize(std::env::temp_dir())
            .unwrap()
            .join(format!("kr-f6-policy-{:016x}", getrandom::u64().unwrap()));
        platform::provision_private_dir(&root).unwrap();
        let dir = Self(root);
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
            value["f6_events_policy"] = json!(path);
        }
        serde_json::from_value(value).unwrap()
    }
}

impl Drop for TestDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn disabled_profile() -> Value {
    serde_json::from_str(DISABLED).unwrap()
}

fn opted_in_system(account_cap: u16, global_cap: u16) -> Value {
    json!({
        "enabled": true,
        "start_utc": "2027-01-01T00:00:00Z",
        "end_utc": "2027-01-02T00:00:00Z",
        "rate_caps": {"per_account_per_hour": account_cap, "global_per_minute": global_cap},
    })
}

fn assert_all_disabled(policy: &F6EventPolicy) {
    let profile = disabled_profile();
    let systems = profile["systems"].as_object().unwrap();
    assert_eq!(systems.len(), 19);
    for name in systems.keys() {
        assert!(!policy.is_system_enabled_at(name, START), "{name}");
    }
}

#[test]
fn omitted_policy_disables_all_19_systems() {
    let dir = TestDir::new();
    let config = dir.config(None);
    assert!(config.f6_events_policy.is_none());
    config.validate().unwrap();
    assert_all_disabled(&config.f6_event_policy().unwrap());
}

#[test]
fn checked_in_profile_loads_and_disables_all_19_systems() {
    let dir = TestDir::new();
    let path = fs::canonicalize(Path::new(env!("CARGO_MANIFEST_DIR")).join(CHECKED_IN)).unwrap();
    let config = dir.config(Some(&path));
    config.validate().unwrap();
    assert_all_disabled(&config.f6_event_policy().unwrap());
}

#[test]
fn valid_opt_in_is_visible_through_config() {
    let dir = TestDir::new();
    let path = dir.0.join("policy.json");
    let mut profile = disabled_profile();
    profile["systems"]["CAPass"] = opted_in_system(1, 1);
    fs::write(&path, profile.to_string()).unwrap();

    let config = dir.config(Some(&path));
    config.validate().unwrap();
    let policy = config.f6_event_policy().unwrap();
    assert!(policy.is_system_enabled_at("CAPass", START));
    assert!(!policy.is_system_enabled_at("CAPass", START - 1));
    assert!(!policy.is_system_enabled_at("Pet", START));

    let mut tracker = F6QuotaTracker::default();
    assert_eq!(
        policy.check_operation(&mut tracker, "CAPass", 1, START),
        Ok(())
    );
    assert_eq!(
        policy.check_operation(&mut tracker, "CAPass", 2, START),
        Err(QuotaRejection::GlobalQuotaExceeded)
    );
}

#[test]
fn quotas_roll_over_at_utc_boundaries_and_rejections_do_not_consume_either_cap() {
    use QuotaRejection::{AccountQuotaExceeded, ClockRegression, GlobalQuotaExceeded};

    let mut profile = disabled_profile();
    profile["systems"]["CAPass"] = opted_in_system(2, 3);
    let policy = F6EventPolicy::from_json_str(&profile.to_string()).unwrap();
    let mut tracker = F6QuotaTracker::default();
    let mut check = |account_id, elapsed| {
        policy.check_operation(&mut tracker, "CAPass", account_id, START + elapsed)
    };

    // Start one second into the buckets so the test distinguishes UTC
    // boundaries from windows measured since the first operation.
    assert_eq!(check(1, 1), Ok(()));
    assert_eq!(check(1, 1), Ok(()));
    assert_eq!(check(1, 1), Err(AccountQuotaExceeded));
    assert_eq!(check(2, 1), Ok(()));
    assert_eq!(check(3, 1), Err(GlobalQuotaExceeded));
    assert_eq!(check(3, 59), Err(GlobalQuotaExceeded));
    assert_eq!(check(3, 60), Ok(()));
    assert_eq!(check(3, 60), Ok(()));
    assert_eq!(check(1, 3599), Err(AccountQuotaExceeded));
    assert_eq!(check(1, 3600), Ok(()));
    assert_eq!(check(2, 3599), Err(ClockRegression));
    assert_eq!(check(1, 3600), Ok(()));
    assert_eq!(check(1, 3600), Err(AccountQuotaExceeded));
}

#[test]
fn quota_preconditions_fail_closed_in_order_and_include_schedule_endpoints() {
    use QuotaRejection::*;

    let mut profile = disabled_profile();
    profile["systems"]["CAPass"] = opted_in_system(2, 1);
    profile["systems"]["CAPass"]["start_utc"] = json!("1969-12-31T23:59:58Z");
    profile["systems"]["CAPass"]["end_utc"] = json!("1970-01-01T00:01:00Z");
    profile["systems"]["Pet"] = opted_in_system(1, 1);
    let policy = F6EventPolicy::from_json_str(&profile.to_string()).unwrap();
    let mut tracker = F6QuotaTracker::default();
    assert_eq!(
        F6EventPolicy::disabled_default().check_operation(&mut tracker, "CAPass", 1, START),
        Err(Disabled)
    );
    for (system, account, at, rejection) in [
        ("capass", 0, -3, UnknownSystem),
        ("PCRoomBenefit", 0, -3, ExcludedSystem),
        ("DIYBomb", 0, -3, Disabled),
        ("CAPass", 0, -3, OutsideSchedule),
        ("CAPass", 0, -2, InvalidTimestamp),
        ("CAPass", 0, 0, InvalidAccountId),
        ("CAPass", -1, 0, InvalidAccountId),
        ("CAPass", 1, 61, OutsideSchedule),
    ] {
        assert_eq!(
            policy.check_operation(&mut tracker, system, account, at),
            Err(rejection)
        );
    }
    assert_eq!(tracker.tracked_accounts(), 0);
    assert_eq!(policy.check_operation(&mut tracker, "CAPass", 1, 0), Ok(()));
    assert_eq!(
        policy.check_operation(&mut tracker, "CAPass", 1, 60),
        Ok(())
    );
    assert_eq!(
        policy.check_operation(&mut tracker, "Pet", 1, START),
        Ok(())
    );
}

#[test]
fn quotas_are_per_system_and_prune_idle_accounts_across_hours() {
    let mut profile = disabled_profile();
    profile["systems"]["CAPass"] = opted_in_system(1, 1);
    profile["systems"]["Pet"] = opted_in_system(1, 1);
    let policy = F6EventPolicy::from_json_str(&profile.to_string()).unwrap();
    let mut tracker = F6QuotaTracker::default();

    assert_eq!(
        policy.check_operation(&mut tracker, "Pet", 1, START),
        Ok(())
    );
    for hour in 0..8 {
        let now = START + hour * 3600;
        assert_eq!(
            policy.check_operation(&mut tracker, "CAPass", hour + 1, now),
            Ok(())
        );
        for rejected_account in 100..200 {
            assert_eq!(
                policy.check_operation(&mut tracker, "CAPass", rejected_account, now),
                Err(QuotaRejection::GlobalQuotaExceeded)
            );
        }
        assert_eq!(tracker.tracked_accounts(), if hour == 0 { 2 } else { 1 });
    }
}

#[test]
fn invalid_policy_paths_fail_config_validation() {
    let dir = TestDir::new();
    let file = dir.0.join("policy.json");
    fs::write(&file, DISABLED).unwrap();
    let subdir = dir.0.join("subdir");
    fs::create_dir(&subdir).unwrap();
    for (path, kind) in [
        (dir.0.join("missing.json"), io::ErrorKind::NotFound),
        (subdir.clone(), io::ErrorKind::InvalidInput),
        (subdir.join("../policy.json"), io::ErrorKind::InvalidInput),
        (PathBuf::new(), io::ErrorKind::InvalidInput),
    ] {
        let error = dir.config(Some(&path)).validate().unwrap_err();
        assert_eq!(error.kind(), kind, "{path:?}: {error}");
    }
}

#[cfg(unix)]
#[test]
fn symlink_file_and_symlink_parent_fail_config_validation() {
    use std::os::unix::fs::symlink;

    let dir = TestDir::new();
    let target_dir = dir.0.join("profiles");
    fs::create_dir(&target_dir).unwrap();
    let target = target_dir.join("policy.json");
    fs::write(&target, DISABLED).unwrap();
    let file_link = dir.0.join("policy-link.json");
    let dir_link = dir.0.join("profiles-link");
    symlink(&target, &file_link).unwrap();
    symlink(&target_dir, &dir_link).unwrap();

    for path in [file_link, dir_link.join("policy.json")] {
        let error = dir.config(Some(&path)).validate().unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::InvalidInput, "{error}");
        assert!(error.to_string().contains("without symlinks"), "{error}");
    }
}

#[tokio::test]
async fn invalid_policy_contents_fail_validation_and_startup() {
    let dir = TestDir::new();
    let path = dir.0.join("policy.json");
    let config = dir.config(Some(&path));
    let mut schema = disabled_profile();
    schema["schema"] = json!("kre.f6-events-policy.v2");
    let mut default_enabled = disabled_profile();
    default_enabled["default_enabled"] = json!(true);
    let mut unknown = disabled_profile();
    unknown["systems"]["UnknownEvent"] = unknown["systems"]["CAPass"].clone();
    let mut excluded = disabled_profile();
    excluded["systems"]["PCRoomBenefit"] = excluded["systems"]["CAPass"].clone();

    for (policy, reason) in [
        ("{".to_owned(), "EOF while parsing"),
        (schema.to_string(), "schema must be"),
        (default_enabled.to_string(), "default_enabled must be false"),
        (unknown.to_string(), "unknown F6 system \"UnknownEvent\""),
        (excluded.to_string(), "PCRoomBenefit is excluded"),
    ] {
        fs::write(&path, policy).unwrap();
        let validation_error = config.validate().unwrap_err();
        let startup_error = Owner::open(&config, Default::default(), None)
            .await
            .err()
            .expect("invalid F6 policy was accepted at startup");
        for error in [validation_error, startup_error] {
            assert_eq!(error.kind(), io::ErrorKind::InvalidData, "{error}");
            assert!(error.to_string().contains(reason), "{error}");
        }
        assert!(!config.launch_socket.exists());
        assert!(!config.database.with_extension("instance.lock").exists());
    }
}
