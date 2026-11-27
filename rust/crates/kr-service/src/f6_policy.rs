//! Fail-closed seasonal event policy for KR 00967.

use serde::de::{self, MapAccess, Visitor, value::MapAccessDeserializer};
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::{BTreeMap, HashMap, hash_map::Entry};
use std::fmt;
use std::io;
use std::marker::PhantomData;
use std::path::Path;

const SCHEMA: &str = "kre.f6-events-policy.v1";
const EXCLUDED_SYSTEM: &str = "PCRoomBenefit";
const MAX_PER_ACCOUNT_PER_HOUR: u16 = 60;
const MAX_GLOBAL_PER_MINUTE: u16 = 600;
const SECONDS_PER_HOUR: i64 = 3600;
const SECONDS_PER_MINUTE: i64 = 60;

// The in_scope set from docs/evidence/content/f6-events-catalog.json.
const SYSTEM_NAMES: [&str; 19] = [
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

/// A validated policy. Systems are off unless explicitly opted in.
#[derive(Debug)]
pub struct F6EventPolicy {
    enabled_systems: HashMap<&'static str, F6EnabledSystem>,
}

/// The configured schedule and limits for an opted-in system.
#[derive(Debug)]
pub struct F6EnabledSystem {
    pub start_utc: i64,
    pub end_utc: i64,
    pub rate_caps: RateCaps,
}

impl F6EnabledSystem {
    fn contains(&self, now_utc: i64) -> bool {
        (self.start_utc..=self.end_utc).contains(&now_utc)
    }
}

/// Keep one tracker for all F6 operations on the server. Counters reset on
/// UTC hour and minute boundaries respectively, with no wall clock access.
#[derive(Debug, Default)]
pub struct F6QuotaTracker {
    latest_utc: i64,
    account_counts: BTreeMap<(&'static str, i64), u16>,
    system_counts: BTreeMap<&'static str, u16>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum QuotaRejection {
    UnknownSystem,
    ExcludedSystem,
    Disabled,
    OutsideSchedule,
    InvalidTimestamp,
    InvalidAccountId,
    ClockRegression,
    AccountQuotaExceeded,
    GlobalQuotaExceeded,
}

impl F6QuotaTracker {
    fn accept(
        &mut self,
        system: &'static str,
        caps: RateCaps,
        account_id: i64,
        now_utc: i64,
    ) -> Result<(), QuotaRejection> {
        if now_utc < self.latest_utc {
            return Err(QuotaRejection::ClockRegression);
        }
        // Clear expired buckets for every system, including idle ones.
        if now_utc / SECONDS_PER_HOUR != self.latest_utc / SECONDS_PER_HOUR {
            self.account_counts.clear();
        }
        if now_utc / SECONDS_PER_MINUTE != self.latest_utc / SECONDS_PER_MINUTE {
            self.system_counts.clear();
        }
        self.latest_utc = now_utc;

        let account = (system, account_id);
        let account_count = self.account_counts.get(&account).copied().unwrap_or(0);
        if account_count >= caps.per_account_per_hour {
            return Err(QuotaRejection::AccountQuotaExceeded);
        }
        let system_count = self.system_counts.get(system).copied().unwrap_or(0);
        if system_count >= caps.global_per_minute {
            return Err(QuotaRejection::GlobalQuotaExceeded);
        }

        // Only accepted operations allocate entries or increment either quota.
        self.account_counts.insert(account, account_count + 1);
        self.system_counts.insert(system, system_count + 1);
        Ok(())
    }

    #[cfg(test)]
    pub(crate) fn tracked_accounts(&self) -> usize {
        self.account_counts.len()
    }
}

impl F6EventPolicy {
    /// Load a complete profile. Any invalid field rejects the entire profile.
    pub fn from_json_str(json: &str) -> io::Result<Self> {
        let Object(profile): Object<Profile> = serde_json::from_str(json)
            .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
        if profile.schema != SCHEMA {
            return Err(invalid_data(format!("schema must be {SCHEMA}")));
        }
        if profile.default_enabled {
            return Err(invalid_data("default_enabled must be false"));
        }

        let mut policy = Self::disabled_default();
        for name in SYSTEM_NAMES {
            let system = profile
                .systems
                .0
                .get(name)
                .ok_or_else(|| invalid_data(format!("missing system {name:?}")))?;
            let caps = system.rate_caps.0;
            if caps.per_account_per_hour > MAX_PER_ACCOUNT_PER_HOUR
                || caps.global_per_minute > MAX_GLOBAL_PER_MINUTE
            {
                return Err(invalid_data(format!(
                    "systems.{name}: rate caps exceed policy limits"
                )));
            }
            if system.enabled && (caps.per_account_per_hour == 0 || caps.global_per_minute == 0) {
                return Err(invalid_data(format!(
                    "systems.{name}: enabled systems require positive rate caps"
                )));
            }

            match (system.start_utc, system.end_utc) {
                (Some(start_utc), Some(end_utc)) if start_utc < end_utc => {
                    if system.enabled {
                        policy.enabled_systems.insert(
                            name,
                            F6EnabledSystem {
                                start_utc,
                                end_utc,
                                rate_caps: caps,
                            },
                        );
                    }
                }
                (None, None) if !system.enabled => {}
                _ => {
                    return Err(invalid_data(format!(
                        "systems.{name}: a schedule requires both timestamps and start_utc < end_utc"
                    )));
                }
            }
        }
        Ok(policy)
    }

    /// Load a complete UTF-8 JSON profile from disk.
    pub fn from_path(path: impl AsRef<Path>) -> io::Result<Self> {
        Self::from_json_str(&std::fs::read_to_string(path)?)
    }

    /// Disable all events without relying on an external profile.
    pub fn disabled_default() -> Self {
        Self {
            enabled_systems: HashMap::new(),
        }
    }

    /// Return the catalog systems in their stable policy order.
    pub fn system_names() -> &'static [&'static str] {
        &SYSTEM_NAMES
    }

    /// Report explicit opt-ins in catalog order, including inactive schedules.
    pub fn enabled_systems(&self) -> impl Iterator<Item = (&'static str, &F6EnabledSystem)> {
        SYSTEM_NAMES
            .into_iter()
            .filter_map(|name| self.enabled_systems.get(name).map(|system| (name, system)))
    }

    /// Return whether the system is opted in at the given Unix second.
    /// Both endpoints of a validated schedule are included.
    pub fn is_system_enabled_at(&self, name: &str, utc_unix_secs: i64) -> bool {
        self.enabled_systems
            .get(name)
            .is_some_and(|system| system.contains(utc_unix_secs))
    }

    /// Check and charge both quotas for one operation. Supply timestamps in
    /// nondecreasing order for a shared tracker; clock regressions fail closed.
    pub fn check_operation(
        &self,
        tracker: &mut F6QuotaTracker,
        system: &str,
        account_id: i64,
        now_utc: i64,
    ) -> Result<(), QuotaRejection> {
        if system == EXCLUDED_SYSTEM {
            return Err(QuotaRejection::ExcludedSystem);
        }
        let Some((&name, policy)) = self.enabled_systems.get_key_value(system) else {
            return Err(if SYSTEM_NAMES.contains(&system) {
                QuotaRejection::Disabled
            } else {
                QuotaRejection::UnknownSystem
            });
        };
        if !policy.contains(now_utc) {
            return Err(QuotaRejection::OutsideSchedule);
        }
        if now_utc < 0 {
            return Err(QuotaRejection::InvalidTimestamp);
        }
        if account_id <= 0 {
            return Err(QuotaRejection::InvalidAccountId);
        }
        tracker.accept(name, policy.rate_caps, account_id, now_utc)
    }
}

fn invalid_data(message: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message.into())
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Profile {
    schema: String,
    default_enabled: bool,
    systems: Systems,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SystemPolicy {
    enabled: bool,
    #[serde(deserialize_with = "nullable_utc")]
    start_utc: Option<i64>,
    #[serde(deserialize_with = "nullable_utc")]
    end_utc: Option<i64>,
    rate_caps: Object<RateCaps>,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RateCaps {
    pub per_account_per_hour: u16,
    pub global_per_minute: u16,
}

// Serde's derived struct deserializer also accepts positional arrays. These
// fields must be objects with explicit keys, as in the offline validator.
struct Object<T>(T);

impl<'de, T: Deserialize<'de>> Deserialize<'de> for Object<T> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct ObjectVisitor<T>(PhantomData<T>);
        impl<'de, T: Deserialize<'de>> Visitor<'de> for ObjectVisitor<T> {
            type Value = Object<T>;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("a JSON object")
            }

            fn visit_map<M: MapAccess<'de>>(self, map: M) -> Result<Self::Value, M::Error> {
                T::deserialize(MapAccessDeserializer::new(map)).map(Object)
            }
        }
        deserializer.deserialize_map(ObjectVisitor(PhantomData))
    }
}

struct Systems(HashMap<&'static str, SystemPolicy>);

impl<'de> Deserialize<'de> for Systems {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct SystemsVisitor;
        impl<'de> Visitor<'de> for SystemsVisitor {
            type Value = Systems;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("a JSON object of F6 systems")
            }

            fn visit_map<M: MapAccess<'de>>(self, mut map: M) -> Result<Systems, M::Error> {
                let mut systems = HashMap::new();
                while let Some(name) = map.next_key::<String>()? {
                    if name == EXCLUDED_SYSTEM {
                        return Err(de::Error::custom("PCRoomBenefit is excluded by policy"));
                    }
                    let name = SYSTEM_NAMES
                        .iter()
                        .copied()
                        .find(|allowed| *allowed == name)
                        .ok_or_else(|| de::Error::custom(format!("unknown F6 system {name:?}")))?;
                    let Entry::Vacant(slot) = systems.entry(name) else {
                        return Err(de::Error::duplicate_field(name));
                    };
                    let Object(system) = map.next_value()?;
                    slot.insert(system);
                }
                Ok(Systems(systems))
            }
        }
        deserializer.deserialize_map(SystemsVisitor)
    }
}

fn nullable_utc<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Option<i64>, D::Error> {
    Option::<String>::deserialize(deserializer)?
        .map(|value| {
            utc_unix_secs(&value).ok_or_else(|| {
                de::Error::custom("expected a UTC timestamp in YYYY-MM-DDTHH:MM:SSZ")
            })
        })
        .transpose()
}

fn utc_unix_secs(value: &str) -> Option<i64> {
    let bytes = value.as_bytes();
    if bytes.len() != 20
        || [
            bytes[4], bytes[7], bytes[10], bytes[13], bytes[16], bytes[19],
        ] != *b"--T::Z"
    {
        return None;
    }
    let number = |digits: &[u8]| -> Option<u32> {
        digits.iter().try_fold(0, |n, &digit| {
            digit
                .is_ascii_digit()
                .then(|| n * 10 + u32::from(digit - b'0'))
        })
    };
    let year = number(&bytes[0..4])?;
    let month = number(&bytes[5..7])?;
    let day = number(&bytes[8..10])?;
    let hour = number(&bytes[11..13])?;
    let minute = number(&bytes[14..16])?;
    let second = number(&bytes[17..19])?;
    if year == 0 || !(1..=12).contains(&month) || hour > 23 || minute > 59 || second > 59 {
        return None;
    }

    let leap_year =
        year.is_multiple_of(4) && (!year.is_multiple_of(100) || year.is_multiple_of(400));
    let month_lengths = [
        31,
        28 + u32::from(leap_year),
        31,
        30,
        31,
        30,
        31,
        31,
        30,
        31,
        30,
        31,
    ];
    let month_index = (month - 1) as usize;
    if !(1..=month_lengths[month_index]).contains(&day) {
        return None;
    }

    // Count days in complete Gregorian years, then in the current year.
    let days_before_year = |year: i64| {
        let y = year - 1;
        y * 365 + y / 4 - y / 100 + y / 400
    };
    let days_this_year = month_lengths[..month_index].iter().sum::<u32>() + day - 1;
    let unix_days =
        days_before_year(i64::from(year)) - days_before_year(1970) + i64::from(days_this_year);
    Some(unix_days * 86_400 + i64::from(hour * 3600 + minute * 60 + second))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Value, json};

    const DISABLED: &str = include_str!("../../../../server/profiles/f6-events-disabled.json");
    const START: &str = "2027-01-01T00:00:00Z";
    const END: &str = "2027-01-01T00:01:00Z";

    fn profile() -> Value {
        serde_json::from_str(DISABLED).unwrap()
    }

    fn opted_in_profile() -> Value {
        let mut profile = profile();
        profile["systems"]["CAPass"] = json!({
            "enabled": true,
            "start_utc": START,
            "end_utc": END,
            "rate_caps": {"per_account_per_hour": 60, "global_per_minute": 600}
        });
        profile
    }

    fn assert_rejected(json: &str, message: &str) {
        let error = F6EventPolicy::from_json_str(json).unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::InvalidData);
        assert!(error.to_string().contains(message), "{error}");
    }

    #[test]
    fn checked_in_profile_and_fallback_disable_every_catalog_system() {
        let catalog: Value = serde_json::from_str(include_str!(
            "../../../../docs/evidence/content/f6-events-catalog.json"
        ))
        .unwrap();
        let systems = catalog["systems"].as_array().unwrap();
        let mut in_scope: Vec<_> = systems
            .iter()
            .filter(|row| row["scope"] == "in_scope")
            .map(|row| row["system"].as_str().unwrap())
            .collect();
        in_scope.sort_unstable();
        let mut allowlist = SYSTEM_NAMES;
        allowlist.sort_unstable();
        assert_eq!(in_scope, allowlist);
        assert!(
            systems.iter().any(|row| {
                row["system"] == EXCLUDED_SYSTEM && row["scope"] == "excluded_policy"
            })
        );

        for policy in [
            F6EventPolicy::from_json_str(DISABLED).unwrap(),
            F6EventPolicy::from_path("../../../server/profiles/f6-events-disabled.json").unwrap(),
            F6EventPolicy::disabled_default(),
        ] {
            for name in in_scope.iter().copied().chain([EXCLUDED_SYSTEM, "unknown"]) {
                assert!(!policy.is_system_enabled_at(name, 1_798_761_600), "{name}");
            }
        }
    }

    #[test]
    fn rejects_unknown_schema_and_default_enablement() {
        let mut profile = profile();
        profile["schema"] = json!("kre.f6-events-policy.v2");
        assert_rejected(
            &profile.to_string(),
            "schema must be kre.f6-events-policy.v1",
        );
        profile["schema"] = json!(SCHEMA);
        profile["default_enabled"] = json!(true);
        assert_rejected(&profile.to_string(), "default_enabled must be false");
    }

    #[test]
    fn rejects_excluded_unknown_and_missing_systems() {
        let mut profile = profile();
        let systems = profile["systems"].as_object_mut().unwrap();
        let disabled = systems["CAPass"].clone();
        for (name, message) in [
            (EXCLUDED_SYSTEM, "PCRoomBenefit is excluded"),
            ("Antarctic", "unknown F6 system"),
            ("capass", "unknown F6 system"),
        ] {
            let systems = profile["systems"].as_object_mut().unwrap();
            systems.insert(name.into(), disabled.clone());
            assert_rejected(&profile.to_string(), message);
            profile["systems"].as_object_mut().unwrap().remove(name);
        }
        profile["systems"].as_object_mut().unwrap().remove("Pet");
        assert_rejected(&profile.to_string(), "missing system \"Pet\"");
    }

    #[test]
    fn rejects_unknown_and_duplicate_keys_at_every_level() {
        for (field, replacement) in [
            ("\"schema\":", "\"extra\": false, \"schema\":"),
            ("\"enabled\":", "\"extra\": false, \"enabled\":"),
            (
                "\"per_account_per_hour\":",
                "\"extra\": 0, \"per_account_per_hour\":",
            ),
        ] {
            assert_rejected(&DISABLED.replacen(field, replacement, 1), "unknown field");
        }
        for (field, replacement) in [
            (
                "\"default_enabled\":",
                "\"default_enabled\": false, \"default_enabled\":",
            ),
            ("\"Câu cá\":", "\"\\u0043APass\": {}, \"Câu cá\":"),
            ("\"enabled\":", "\"enabled\": false, \"enabled\":"),
            (
                "\"global_per_minute\":",
                "\"global_per_minute\": 0, \"global_per_minute\":",
            ),
        ] {
            assert_rejected(&DISABLED.replacen(field, replacement, 1), "duplicate field");
        }
    }

    #[test]
    fn rejects_missing_fields_and_nonobject_shapes() {
        for (name, input) in [
            (
                "default_enabled",
                DISABLED.replacen("\"default_enabled\": false,", "", 1),
            ),
            (
                "start_utc",
                DISABLED.replacen("\"start_utc\": null,", "", 1),
            ),
            ("end_utc", DISABLED.replacen("\"end_utc\": null,", "", 1)),
            (
                "per_account_per_hour",
                DISABLED.replacen("\"per_account_per_hour\": 0,", "", 1),
            ),
        ] {
            assert_rejected(&input, &format!("missing field `{name}`"));
        }

        let mut profile = profile();
        let root_array = json!([
            profile["schema"],
            profile["default_enabled"],
            profile["systems"]
        ]);
        assert_rejected(&root_array.to_string(), "invalid type");
        profile["systems"]["CAPass"]["rate_caps"] = json!([0, 0]);
        assert_rejected(&profile.to_string(), "invalid type");
        profile["systems"]["CAPass"] =
            json!([false, null, null, {"per_account_per_hour": 0, "global_per_minute": 0}]);
        assert_rejected(&profile.to_string(), "invalid type");
        profile["systems"] = json!([]);
        assert_rejected(&profile.to_string(), "invalid type");
    }

    #[test]
    fn rejects_invalid_or_unbounded_rate_caps() {
        for (name, over_limit) in [("per_account_per_hour", 61), ("global_per_minute", 601)] {
            let mut profile = opted_in_profile();
            for (cap, error) in [
                (json!(0), "positive rate caps"),
                (json!(over_limit), "rate caps exceed"),
                (json!(-1), "invalid value"),
                (json!(1.5), "invalid type"),
                (json!(true), "invalid type"),
            ] {
                profile["systems"]["CAPass"]["rate_caps"][name] = cap;
                assert_rejected(&profile.to_string(), error);
            }
        }

        let mut profile = profile();
        profile["systems"]["CAPass"]["rate_caps"]["global_per_minute"] = json!(601);
        assert_rejected(&profile.to_string(), "rate caps exceed");
    }

    #[test]
    fn rejects_missing_inverted_and_equal_schedule_bounds() {
        let mut profile = opted_in_profile();
        for (start, end) in [
            (json!(END), json!(START)),
            (json!(START), json!(START)),
            (Value::Null, json!(END)),
            (json!(START), Value::Null),
            (Value::Null, Value::Null),
        ] {
            profile["systems"]["CAPass"]["start_utc"] = start;
            profile["systems"]["CAPass"]["end_utc"] = end;
            assert_rejected(&profile.to_string(), "a schedule requires both timestamps");
        }
        profile["systems"]["CAPass"]["enabled"] = json!(false);
        profile["systems"]["CAPass"]["end_utc"] = json!(END);
        assert_rejected(&profile.to_string(), "a schedule requires both timestamps");
    }

    #[test]
    fn validates_exact_utc_format_and_gregorian_dates() {
        for invalid in [
            "0000-01-01T00:00:00Z",
            "1900-02-29T00:00:00Z",
            "2027-04-31T00:00:00Z",
            "2027-01-00T00:00:00Z",
            "2027-13-01T00:00:00Z",
            "2027-01-01T24:00:00Z",
            "2027-01-01T00:60:00Z",
            "2027-01-01T00:00:60Z",
            "2027-01-01T00:00:00+00:00",
            "2027-01-01T00:00:00.5Z",
            "2027-01-01T00:00:00z",
            "2027-01-01 00:00:00Z",
        ] {
            let mut profile = opted_in_profile();
            profile["systems"]["CAPass"]["start_utc"] = json!(invalid);
            assert_rejected(&profile.to_string(), "expected a UTC timestamp");
        }
        for (value, expected) in [
            ("0001-01-01T00:00:00Z", -62_135_596_800),
            ("1969-12-31T23:59:59Z", -1),
            ("1970-01-01T00:00:00Z", 0),
            ("2000-02-29T00:00:00Z", 951_782_400),
            ("9999-12-31T23:59:59Z", 253_402_300_799),
        ] {
            assert_eq!(utc_unix_secs(value), Some(expected), "{value}");
        }
    }

    #[test]
    fn opt_in_only_enables_the_named_system_within_the_inclusive_window() {
        let mut profile = opted_in_profile();
        profile["systems"]["Pet"] = profile["systems"]["CAPass"].clone();
        profile["systems"]["Pet"]["enabled"] = json!(false);
        let policy = F6EventPolicy::from_json_str(&profile.to_string()).unwrap();

        let start = 1_798_761_600;
        let end = start + 60;
        assert!(!policy.is_system_enabled_at("CAPass", start - 1));
        assert!(policy.is_system_enabled_at("CAPass", start));
        assert!(policy.is_system_enabled_at("CAPass", start + 30));
        assert!(policy.is_system_enabled_at("CAPass", end));
        assert!(!policy.is_system_enabled_at("CAPass", end + 1));
        for name in ["Pet", "LuckyCard", "capass", EXCLUDED_SYSTEM, "unknown"] {
            assert!(!policy.is_system_enabled_at(name, start + 30));
        }
    }
}
