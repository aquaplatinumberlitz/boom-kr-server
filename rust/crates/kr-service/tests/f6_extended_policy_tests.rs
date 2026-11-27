use kr_service::f6_policy::{F6EventPolicy, F6PolicyError, F6QuotaTracker, QuotaRejection};
use serde_json::{Value, json};
use std::{fs, io, path::PathBuf};

const DISABLED: &str = include_str!("../../../../server/profiles/f6-events-disabled.json");
const SPEC: &str = include_str!("../../../../docs/evidence/content/f6-extended-events-spec.json");
const START_UTC: &str = "2027-01-01T00:00:00Z";
const END_UTC: &str = "2027-01-04T00:00:00Z";
const START: i64 = 1_798_761_600;
const DAY: i64 = 86_400;

fn extended_profile() -> Value {
    let mut profile: Value = serde_json::from_str(DISABLED).unwrap();
    let spec: Value = serde_json::from_str(SPEC).unwrap();
    let mut extended = serde_json::Map::new();
    for row in spec["roadmap_only"].as_array().unwrap() {
        assert_eq!(row["scope"], "roadmap_only");
        let mut disabled = spec["defaults"].clone();
        disabled["max_grants_per_day"] = json!(0);
        disabled["max_grants_per_season"] = json!(0);
        extended.insert(row["id"].as_str().unwrap().into(), disabled);
    }
    profile["extended_systems"] = Value::Object(extended);
    profile
}

fn opted_in_system(day: u32, season: u32) -> Value {
    json!({
        "enabled": true,
        "start_utc": START_UTC,
        "end_utc": END_UTC,
        "rate_caps": {"per_account_per_hour": 10, "global_per_minute": 10},
        "season_key": "winter-2027",
        "max_grants_per_day": day,
        "max_grants_per_season": season
    })
}

fn load(profile: &Value) -> F6EventPolicy {
    F6EventPolicy::from_json_str(&profile.to_string()).unwrap()
}

fn assert_rejected(input: &str, reason: &str) {
    let err = F6EventPolicy::from_json_str(input).unwrap_err();
    assert_eq!(err.kind(), io::ErrorKind::InvalidData);
    assert!(err.get_ref().unwrap().is::<F6PolicyError>());
    assert!(
        err.to_string().contains(reason),
        "expected {reason:?}: {err}"
    );
}

#[test]
fn checked_in_profile_and_explicit_defaults_disable_all_44_audited_systems() {
    let checked_in = fs::canonicalize(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../../server/profiles/f6-events-disabled.json"),
    )
    .unwrap();
    let profile = extended_profile();
    let core = profile["systems"].as_object().unwrap();
    let extended = profile["extended_systems"].as_object().unwrap();
    assert_eq!(core.len(), 19);
    assert_eq!(extended.len(), 25);
    assert_eq!(F6EventPolicy::system_names().len(), core.len());
    assert_eq!(F6EventPolicy::extended_system_names().len(), extended.len());
    assert!(
        F6EventPolicy::system_names()
            .iter()
            .all(|id| core.contains_key(*id))
    );
    assert!(
        F6EventPolicy::extended_system_names()
            .iter()
            .all(|id| extended.contains_key(*id))
    );
    for policy in [
        F6EventPolicy::from_path(checked_in).unwrap(),
        F6EventPolicy::disabled_default(),
        load(&profile),
    ] {
        let mut tracker = F6QuotaTracker::default();
        assert_eq!(policy.enabled_systems().count(), 0);
        assert_eq!(policy.enabled_extended_systems().count(), 0);
        for name in core.keys().chain(extended.keys()) {
            assert!(!policy.is_system_enabled_at(name, START), "{name}");
            assert_eq!(
                policy.check_operation(&mut tracker, name, 1, START),
                Err(QuotaRejection::Disabled),
                "{name}",
            );
        }
    }
}

#[test]
fn opt_ins_use_independent_half_open_windows() {
    let mut profile = extended_profile();
    profile["extended_systems"]["pet-care"] = opted_in_system(2, 3);
    let policy = load(&profile);
    assert_eq!(policy.enabled_systems().count(), 0);
    let (name, system) = policy.enabled_extended_systems().next().unwrap();
    assert_eq!(name, "pet-care");
    assert_eq!(system.season_key, "winter-2027");
    assert_eq!((system.start_utc, system.end_utc), (START, START + 3 * DAY));
    assert_eq!(
        (system.max_grants_per_day, system.max_grants_per_season),
        (2, 3)
    );
    for (at_unix_secs, active) in [
        (START - 1, false),
        (START, true),
        (START + 3 * DAY - 1, true),
        (START + 3 * DAY, false),
    ] {
        assert_eq!(
            policy.is_system_enabled_at("pet-care", at_unix_secs),
            active
        );
        assert!(!policy.is_system_enabled_at("Pet", at_unix_secs));
        assert!(!policy.is_system_enabled_at("pet-intimacy", at_unix_secs));
    }
    let mut tracker = F6QuotaTracker::default();
    assert_eq!(
        policy.check_operation(&mut tracker, "pet-care", 1, START + 3 * DAY),
        Err(QuotaRejection::OutsideSchedule),
    );
    assert_eq!(
        policy.check_operation(&mut tracker, name, 0, START),
        Err(QuotaRejection::InvalidAccountId),
    );
    assert_eq!(
        policy.check_operation(&mut tracker, "pet-care", 1, START),
        Ok(()),
    );
}

#[test]
fn daily_resets_at_utc_midnight_and_seasonal_grants_remain_per_account_and_system() {
    use QuotaRejection::{
        ClockRegression, ExtendedDailyQuotaExceeded, ExtendedSeasonQuotaExceeded,
    };

    let mut profile = extended_profile();
    profile["extended_systems"]["attendance"] = opted_in_system(2, 3);
    profile["extended_systems"]["bingo"] = opted_in_system(2, 3);
    let policy = load(&profile);
    let mut tracker = F6QuotaTracker::default();
    let mut grant = |system, account, offset| {
        policy.check_operation(&mut tracker, system, account, START + offset)
    };
    assert_eq!(grant("attendance", 1, DAY - 1), Ok(()));
    assert_eq!(grant("attendance", 1, DAY - 1), Ok(()));
    assert_eq!(
        grant("attendance", 1, DAY - 1),
        Err(ExtendedDailyQuotaExceeded)
    );
    assert_eq!(grant("attendance", 2, DAY - 1), Ok(()));
    assert_eq!(grant("bingo", 1, DAY - 1), Ok(()));
    assert_eq!(grant("attendance", 1, DAY), Ok(()));
    assert_eq!(
        grant("attendance", 1, DAY),
        Err(ExtendedSeasonQuotaExceeded)
    );
    assert_eq!(grant("attendance", 2, DAY - 1), Err(ClockRegression));
    assert_eq!(
        grant("attendance", 1, 2 * DAY),
        Err(ExtendedSeasonQuotaExceeded)
    );
    assert_eq!(grant("attendance", 2, 2 * DAY), Ok(()));
    assert_eq!(grant("bingo", 1, 2 * DAY), Ok(()));
}

#[test]
fn rate_rejections_do_not_consume_grants_and_grant_rejections_do_not_consume_rate() {
    use QuotaRejection::{
        ExtendedDailyQuotaExceeded, ExtendedSeasonQuotaExceeded, GlobalQuotaExceeded,
    };

    let mut profile = extended_profile();
    profile["extended_systems"]["attendance"] = opted_in_system(1, 2);
    profile["extended_systems"]["attendance"]["rate_caps"]["global_per_minute"] = json!(1);
    let policy = load(&profile);
    let mut tracker = F6QuotaTracker::default();
    let mut grant = |account, offset| {
        policy.check_operation(&mut tracker, "attendance", account, START + offset)
    };
    assert_eq!(grant(1, 0), Ok(()));
    assert_eq!(grant(2, 0), Err(GlobalQuotaExceeded));
    assert_eq!(grant(1, 60), Err(ExtendedDailyQuotaExceeded));
    assert_eq!(grant(2, 60), Ok(()));
    assert_eq!(grant(2, DAY), Ok(()));
    assert_eq!(grant(2, 2 * DAY), Err(ExtendedSeasonQuotaExceeded));
    assert_eq!(grant(1, 2 * DAY), Ok(()));
}

#[test]
fn a_new_season_key_resets_season_usage_but_keeps_the_current_day() {
    use QuotaRejection::{ExtendedDailyQuotaExceeded, ExtendedSeasonQuotaExceeded};

    let mut profile = extended_profile();
    profile["extended_systems"]["attendance"] = opted_in_system(1, 1);
    let winter = load(&profile);
    profile["extended_systems"]["attendance"]["season_key"] = json!("special-2027");
    let special = load(&profile);
    let mut tracker = F6QuotaTracker::default();
    let mut grant = |policy: &F6EventPolicy, offset| {
        policy.check_operation(&mut tracker, "attendance", 1, START + offset)
    };
    assert_eq!(grant(&winter, 0), Ok(()));
    assert_eq!(grant(&special, 60), Err(ExtendedDailyQuotaExceeded));
    assert_eq!(grant(&winter, DAY), Err(ExtendedSeasonQuotaExceeded));
    assert_eq!(grant(&special, DAY), Ok(()));
    assert_eq!(grant(&winter, 2 * DAY), Err(ExtendedSeasonQuotaExceeded));
}

#[test]
fn rejects_excluded_unknown_missing_and_malformed_extended_entries() {
    let profile = extended_profile();
    for (section, source) in [("systems", "Pet"), ("extended_systems", "pet-care")] {
        let mut invalid = profile.clone();
        invalid[section]["PCRoomBenefit"] = invalid[section][source].clone();
        assert_rejected(
            &invalid.to_string(),
            &format!("{section}: PCRoomBenefit is excluded"),
        );
    }
    for name in ["Pet", "unknown-system", "PET-CARE", ""] {
        let mut invalid = profile.clone();
        invalid["extended_systems"][name] = invalid["extended_systems"]["pet-care"].clone();
        assert_rejected(&invalid.to_string(), "extended_systems: unknown F6 system");
    }
    let mut invalid = profile.clone();
    invalid["extended_systems"]
        .as_object_mut()
        .unwrap()
        .remove("bingo");
    assert_rejected(&invalid.to_string(), "missing extended system \"bingo\"");
    for section in [json!(null), json!([]), json!(false)] {
        invalid["extended_systems"] = section;
        assert_rejected(&invalid.to_string(), "a JSON object for extended_systems");
    }
    invalid = profile;
    invalid["extended_systems"]["bingo"] = json!([]);
    assert_rejected(&invalid.to_string(), "a JSON object");
}

#[test]
fn rejects_unknown_and_duplicate_fields_without_normalizing_json_first() {
    let input = extended_profile().to_string();
    for (field, replacement) in [
        (
            r#""extended_systems":"#,
            r#""extended_systems":{},"extended_systems":"#,
        ),
        (
            r#""bingo":"#,
            r#""\u0062ingo":{"enabled":false,"start_utc":null,"end_utc":null,"rate_caps":{"global_per_minute":0,"per_account_per_hour":0},"max_grants_per_day":0,"max_grants_per_season":0},"bingo":"#,
        ),
        (
            r#""max_grants_per_day":"#,
            r#""max_grants_per_day":0,"max_grants_per_day":"#,
        ),
        (
            r#""rate_caps":{"global_per_minute":"#,
            r#""rate_caps":{"global_per_minute":0,"global_per_minute":"#,
        ),
    ] {
        assert_rejected(&input.replacen(field, replacement, 1), "duplicate field");
    }
    for (field, replacement) in [
        (
            r#""max_grants_per_day":"#,
            r#""grant_everyone":true,"max_grants_per_day":"#,
        ),
        (
            r#""rate_caps":{"global_per_minute":"#,
            r#""rate_caps":{"extra":0,"global_per_minute":"#,
        ),
    ] {
        assert_rejected(&input.replacen(field, replacement, 1), "unknown field");
    }
}

#[test]
fn rejects_invalid_windows_season_keys_and_caps() {
    let mut profile = extended_profile();
    profile["extended_systems"]["bingo"] = opted_in_system(2, 3);
    for (field, value, diagnostic) in [
        ("start_utc", json!(END_UTC), "start_utc < end_utc"),
        (
            "end_utc",
            json!("2026-12-31T00:00:00Z"),
            "start_utc < end_utc",
        ),
        ("end_utc", Value::Null, "both timestamps"),
        (
            "start_utc",
            json!("2027-02-29T00:00:00Z"),
            "expected a UTC timestamp",
        ),
        ("season_key", Value::Null, "season_key"),
        ("season_key", json!(""), "season_key"),
        ("season_key", json!("winter 2027"), "season_key"),
        ("season_key", json!("x".repeat(65)), "season_key"),
        (
            "season_key",
            json!("PCRoomBenefit"),
            "PCRoomBenefit is excluded",
        ),
        ("max_grants_per_day", json!(0), "positive grant caps"),
        ("max_grants_per_season", json!(0), "max_grants_per_season"),
        ("max_grants_per_day", json!(1441), "grant caps exceed"),
        (
            "max_grants_per_season",
            json!(1_000_001),
            "grant caps exceed",
        ),
        ("max_grants_per_day", json!(4), "must not exceed"),
        ("max_grants_per_season", json!(-1), "invalid value"),
        (
            "rate_caps",
            json!({"per_account_per_hour": 0, "global_per_minute": 1}),
            "positive rate caps",
        ),
    ] {
        let mut invalid = profile.clone();
        invalid["extended_systems"]["bingo"][field] = value;
        assert_rejected(&invalid.to_string(), diagnostic);
    }
    for field in [
        "start_utc",
        "season_key",
        "max_grants_per_day",
        "max_grants_per_season",
    ] {
        let mut invalid = profile.clone();
        invalid["extended_systems"]["bingo"]
            .as_object_mut()
            .unwrap()
            .remove(field);
        assert_rejected(&invalid.to_string(), field);
    }
}

struct TestDir(PathBuf);

impl Drop for TestDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn path_loader_accepts_regular_profiles_and_rejects_unsafe_paths() {
    let path = fs::canonicalize(std::env::temp_dir())
        .unwrap()
        .join(format!("kr-f6-extended-{:016x}", getrandom::u64().unwrap()));
    kr_service::platform::provision_private_dir(&path).unwrap();
    let dir = TestDir(path);
    let target = dir.0.join("extended.json");
    fs::write(&target, extended_profile().to_string()).unwrap();
    assert_eq!(
        F6EventPolicy::from_path(&target)
            .unwrap()
            .enabled_extended_systems()
            .count(),
        0
    );

    let subdir = dir.0.join("subdir");
    fs::create_dir(&subdir).unwrap();
    let mut unsafe_paths = vec![
        PathBuf::new(),
        subdir.clone(),
        subdir.join("../extended.json"),
    ];
    #[cfg(unix)]
    {
        use std::os::unix::fs::symlink;
        let file_link = dir.0.join("file-link");
        let dir_link = dir.0.join("dir-link");
        symlink(&target, &file_link).unwrap();
        symlink(&dir.0, &dir_link).unwrap();
        unsafe_paths.extend([file_link, dir_link.join("extended.json")]);
    }
    for path in unsafe_paths {
        let error = F6EventPolicy::from_path(&path).unwrap_err();
        assert_eq!(
            error.kind(),
            io::ErrorKind::InvalidInput,
            "{path:?}: {error}"
        );
    }
    assert_eq!(
        F6EventPolicy::from_path(dir.0.join("missing.json"))
            .unwrap_err()
            .kind(),
        io::ErrorKind::NotFound,
    );
}
