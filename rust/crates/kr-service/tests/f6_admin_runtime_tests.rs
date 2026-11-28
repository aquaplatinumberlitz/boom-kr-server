use kr_service::{
    accounts::{Admin, AuditFilter, Error, Player, Store},
    f6_policy::F6EventPolicy,
};
use rusqlite::Connection;
use serde_json::{Value, json};
#[cfg(unix)]
use std::os::unix::fs::DirBuilderExt;
use std::{
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
    process::{Command, Output, Stdio},
    thread,
    time::Duration,
};

const START: i64 = 1_798_761_600; // 2027-01-01T00:00:00Z
const DAY: i64 = 86_400;
const CORE: &str = "CAPass";
const EXTENDED: &str = "attendance";
const PROFILES: [(&str, &str); 5] = [
    ("--f3-policy", "f3-pve-disabled.json"),
    ("--f4-policy", "f4-extended-disabled.json"),
    ("--f5-policy", "f5-social-chat-disabled.json"),
    ("--f6-policy", "f6-events-disabled.json"),
    ("--f7-policy", "f7-custom-map-disabled.json"),
];
static CREATE_ACCOUNT_GATE: std::sync::Mutex<()> = std::sync::Mutex::new(());

struct TestDir(PathBuf);

impl TestDir {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!("kr-f6-claim-{}", getrandom::u64().unwrap()));
        let mut builder = fs::DirBuilder::new();
        #[cfg(unix)]
        builder.mode(0o700);
        builder.create(&path).unwrap();
        Self(path)
    }

    fn write(&self, name: &str, contents: impl AsRef<[u8]>) -> PathBuf {
        let path = self.0.join(name);
        fs::write(&path, contents).unwrap();
        path
    }
}

impl Drop for TestDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn profile(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(3)
        .unwrap()
        .join("server/profiles")
        .join(name)
}

fn disabled_f6() -> PathBuf {
    profile(PROFILES[3].1)
}

fn opted_in() -> Value {
    let mut policy: Value = serde_json::from_slice(&fs::read(disabled_f6()).unwrap()).unwrap();
    policy["systems"][CORE] = json!({
        "enabled": true,
        "start_utc": "2027-01-01T00:00:00Z",
        "end_utc": "2027-01-04T00:00:00Z",
        "rate_caps": {"per_account_per_hour": 1, "global_per_minute": 1},
    });
    let disabled = json!({
        "enabled": false,
        "start_utc": null,
        "end_utc": null,
        "rate_caps": {"per_account_per_hour": 0, "global_per_minute": 0},
        "max_grants_per_day": 0,
        "max_grants_per_season": 0,
    });
    policy["extended_systems"] = F6EventPolicy::extended_system_names()
        .iter()
        .map(|&event| (event.to_owned(), disabled.clone()))
        .collect::<serde_json::Map<_, _>>()
        .into();
    policy["extended_systems"][EXTENDED] = json!({
        "enabled": true,
        "start_utc": "2027-01-01T00:00:00Z",
        "end_utc": "2027-01-04T00:00:00Z",
        "rate_caps": {"per_account_per_hour": 10, "global_per_minute": 10},
        "season_key": "winter-2027",
        "max_grants_per_day": 1,
        "max_grants_per_season": 2,
    });
    policy
}

fn create_account(store: &mut Store, name: &str) -> Player {
    let _guard = CREATE_ACCOUNT_GATE
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    let admin = Admin {
        actor: "f6-admin-tests",
        reason: "create fixture account",
    };
    for _ in 0..200 {
        match store.create_account(&admin, name, name, b"test-only-password", START - 1) {
            Ok(player) => return player,
            Err(Error::Busy) => thread::sleep(Duration::from_millis(25)),
            Err(error) => panic!("account creation failed: {error:?}"),
        }
    }
    panic!("account creation stayed busy");
}

struct Fixture {
    store: Store,
    observer: Connection,
    db: PathBuf,
    policy: PathBuf,
    alice: Player,
    _dir: TestDir,
}

impl Fixture {
    fn new() -> Self {
        let dir = TestDir::new();
        let db = dir.0.join("accounts.sqlite");
        let mut store = Store::open(&db).unwrap();
        let alice = create_account(&mut store, "Alice");
        Self {
            store,
            observer: Connection::open(&db).unwrap(),
            db,
            policy: dir.write("f6-opted-in.json", opted_in().to_string()),
            alice,
            _dir: dir,
        }
    }

    fn claim(
        &self,
        policy: &Path,
        player: [&str; 2],
        event: &str,
        now: i64,
        reward: &[&str],
    ) -> Output {
        command("f6-claim-event")
            .arg("--db")
            .arg(&self.db)
            .arg("--f6-policy")
            .arg(policy)
            .args(player)
            .args(["--event-id", event, "--now-unix", &now.to_string()])
            .args(["--operator", "on-call", "--reason", "grant event reward"])
            .args(reward)
            .output()
            .unwrap()
    }

    fn claim_alice(&self, event: &str, now: i64, reward: &[&str]) -> Output {
        self.claim(&self.policy, ["--account", "Alice"], event, now, reward)
    }

    fn version(&self) -> i64 {
        self.observer
            .pragma_query_value(None, "data_version", |row| row.get(0))
            .unwrap()
    }

    fn assert_no_write(&self, diagnostic: &str, invoke: impl FnOnce() -> Output) {
        let before = self.version();
        assert_rejected(invoke(), diagnostic);
        // The observer detects any committed write, even an audit or clock
        // update that leaves inventory and Lucci unchanged.
        assert_eq!(self.version(), before);
    }
}

fn command(subcommand: &str) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_kr-admin"));
    command.arg(subcommand).stdin(Stdio::null());
    command
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
    assert_eq!(output.status.code(), Some(1), "{output:?}");
    assert!(output.stdout.is_empty());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(
        stderr.starts_with("kr-admin: ") && stderr.contains(diagnostic),
        "expected {diagnostic:?}: {stderr}"
    );
}

#[test]
fn disabled_f6_profile_rejects_without_writing() {
    let f = Fixture::new();
    f.assert_no_write("Disabled", || {
        f.claim(
            &disabled_f6(),
            ["--account", "Alice"],
            CORE,
            START,
            &["--lucci", "25"],
        )
    });
}

#[test]
fn grants_lucci_and_items_and_persists_runtime_and_operator_audits() {
    let f = Fixture::new();
    let id = f.alice.player_id;
    let lucci = read_summary(f.claim_alice(CORE, START, &["--lucci", "25"]));
    assert_eq!(lucci["granted"], true);
    assert_eq!(lucci["player_id"], id);
    assert_eq!(lucci["event_id"], CORE);
    assert_eq!(lucci["claimed_at_unix"], START);
    assert_eq!(lucci["reward"], json!({"kind": "lucci", "amount": 25}));

    let item = read_summary(f.claim(
        &f.policy,
        ["--player-id", &id.to_string()],
        EXTENDED,
        START,
        &[
            "--sku-id",
            "event-chip",
            "--quantity",
            "2",
            "--max-stack",
            "10",
        ],
    ));
    assert_eq!(item["granted"], true);
    assert_eq!(item["reward"]["sku_id"], "event-chip");

    assert_eq!(f.store.get_profile(id).unwrap().lucci, 25);
    let inventory = f.store.list_inventory(id, START).unwrap();
    assert_eq!(inventory.len(), 1);
    assert_eq!(inventory[0].sku_id, "event-chip");
    assert_eq!(inventory[0].quantity, 2);
    assert_eq!(f.store.list_ledger(id, 10).unwrap().len(), 2);

    let runtime = f
        .store
        .operator_audit(AuditFilter {
            actor: Some("f6-event-runtime".into()),
            ..AuditFilter::default()
        })
        .unwrap();
    assert_eq!(runtime.len(), 2);
    assert_eq!(runtime[0].operation, "admin_grant_item");
    assert_eq!(runtime[1].operation, "admin_grant_lucci");
    assert!(
        runtime
            .iter()
            .all(|row| row.result == "ok" && row.at_utc == START)
    );
    let operator = f
        .store
        .operator_audit(AuditFilter {
            actor: Some("on-call".into()),
            operation: Some("f6_claim_event".into()),
            ..AuditFilter::default()
        })
        .unwrap();
    assert_eq!(operator.len(), 2);
    assert_eq!(operator[0].request_id, item["operator_audit_request_id"]);
    assert_eq!(operator[1].request_id, lucci["operator_audit_request_id"]);
    assert!(
        operator
            .iter()
            .all(|row| row.result == "ok" && row.reason == "grant event reward")
    );
}

#[test]
fn duplicate_and_rate_caps_hold_across_processes_and_reset_at_utc_boundaries() {
    let mut f = Fixture::new();
    let bob = create_account(&mut f.store, "Bobby");
    read_summary(f.claim_alice(CORE, START, &["--lucci", "25"]));
    f.assert_no_write("duplicate F6 claim", || {
        // A different target selector or reward cannot bypass the claim key.
        f.claim(
            &f.policy,
            ["--player-id", &f.alice.player_id.to_string()],
            CORE,
            START,
            &["--lucci", "30"],
        )
    });
    f.assert_no_write("AccountQuotaExceeded", || {
        f.claim_alice(CORE, START + 60, &["--claim-id", "next", "--lucci", "25"])
    });
    f.assert_no_write("GlobalQuotaExceeded", || {
        f.claim(
            &f.policy,
            ["--account", "Bobby"],
            CORE,
            START,
            &["--lucci", "25"],
        )
    });
    read_summary(f.claim(
        &f.policy,
        ["--account", "Bobby"],
        CORE,
        START + 60,
        &["--lucci", "25"],
    ));
    read_summary(f.claim_alice(
        CORE,
        START + 3_600,
        &["--claim-id", "next", "--lucci", "25"],
    ));
    assert_eq!(f.store.get_profile(f.alice.player_id).unwrap().lucci, 50);
    assert_eq!(f.store.get_profile(bob.player_id).unwrap().lucci, 25);
}

#[test]
fn extended_daily_and_season_caps_hold_across_processes() {
    let f = Fixture::new();
    read_summary(f.claim_alice(EXTENDED, START, &["--lucci", "25"]));
    f.assert_no_write("ExtendedDailyQuotaExceeded", || {
        f.claim_alice(
            EXTENDED,
            START + 3_600,
            &["--claim-id", "day-2", "--lucci", "25"],
        )
    });
    read_summary(f.claim_alice(
        EXTENDED,
        START + DAY,
        &["--claim-id", "day-2", "--lucci", "25"],
    ));
    f.assert_no_write("ExtendedSeasonQuotaExceeded", || {
        f.claim_alice(
            EXTENDED,
            START + 2 * DAY,
            &["--claim-id", "day-3", "--lucci", "25"],
        )
    });
    assert_eq!(f.store.get_profile(f.alice.player_id).unwrap().lucci, 50);
}

#[test]
fn invalid_schedule_event_player_and_policy_never_write() {
    let f = Fixture::new();
    for (event, now, diagnostic) in [
        (CORE, START - 1, "OutsideSchedule"),
        (CORE, START + 3 * DAY + 1, "OutsideSchedule"),
        (EXTENDED, START + 3 * DAY, "OutsideSchedule"),
        ("PCRoomBenefit", START, "ExcludedSystem"),
        ("unlisted", START, "UnknownSystem"),
    ] {
        f.assert_no_write(diagnostic, || f.claim_alice(event, now, &["--lucci", "25"]));
    }
    for (selector, diagnostic) in [
        (["--account", "missing"], "account lookup failed"),
        (["--player-id", "999999"], "player lookup failed"),
    ] {
        f.assert_no_write(diagnostic, || {
            f.claim(&f.policy, selector, CORE, START, &["--lucci", "25"])
        });
    }
    let mut invalid = opted_in();
    invalid["systems"][CORE]["rate_caps"]["global_per_minute"] = json!(0);
    let invalid = f._dir.write("invalid-f6.json", invalid.to_string());
    f.assert_no_write("positive rate caps", || {
        f.claim(
            &invalid,
            ["--account", "Alice"],
            CORE,
            START,
            &["--lucci", "25"],
        )
    });
}

#[test]
fn missing_operator_metadata_cannot_reset_quota() {
    let f = Fixture::new();
    read_summary(f.claim_alice(CORE, START, &["--lucci", "25"]));
    // Simulate an interruption after the runtime grant committed.
    assert_eq!(
        f.observer
            .execute(
                "DELETE FROM admin_audit WHERE operation = 'f6_claim_event'",
                []
            )
            .unwrap(),
        1
    );
    f.assert_no_write("incomplete F6 claim audit history", || {
        f.claim_alice(CORE, START + 60, &["--claim-id", "next", "--lucci", "25"])
    });
    assert_eq!(f.store.get_profile(f.alice.player_id).unwrap().lucci, 25);
}

#[test]
fn policy_audit_checked_in_profiles_match_disabled_defaults() {
    let expected = json!({
        "F3": {"enabled": false},
        "F4": {"enabled": false},
        "F5": {"enabled": false},
        "F6": {"enabled": false},
        "F7": {"enabled": false},
    });
    let default = command("policy-audit").output().unwrap();
    let mut explicit = command("policy-audit");
    for (flag, file) in PROFILES.into_iter().rev() {
        explicit.arg(flag).arg(profile(file));
    }
    let explicit = explicit.output().unwrap();
    assert_eq!(explicit.stdout, default.stdout);
    assert_eq!(read_summary(default), expected);
    assert_eq!(read_summary(explicit), expected);
}

#[test]
fn policy_audit_reports_opt_ins_in_every_domain() {
    let dir = TestDir::new();
    let mut f6 = opted_in();
    let disabled: Value = serde_json::from_slice(&fs::read(disabled_f6()).unwrap()).unwrap();
    f6["systems"] = disabled["systems"].clone(); // extended-only opt-in
    let profiles = [
        r#"{"enabled": true}"#.to_owned(),
        r#"{"lucci_vault": {"enabled": true, "base_lucci_cap": 100, "expansion_step": 10, "max_expansions": 1, "max_lucci_cap": 110}}"#.to_owned(),
        r#"{"note_inbox_cap": 3}"#.to_owned(),
        f6.to_string(),
        r#"{"enabled": true}"#.to_owned(),
    ];
    for (i, ((flag, file), input)) in PROFILES.into_iter().zip(profiles).enumerate() {
        let path = dir.write(file, input);
        let summary = read_summary(
            command("policy-audit")
                .arg(flag)
                .arg(path)
                .output()
                .unwrap(),
        );
        for (j, domain) in ["F3", "F4", "F5", "F6", "F7"].into_iter().enumerate() {
            assert_eq!(summary[domain]["enabled"], i == j, "{flag}, {domain}");
        }
    }
}

#[test]
fn policy_audit_rejects_missing_malformed_and_invalid_profiles() {
    let dir = TestDir::new();
    let mut invalid_f6 = opted_in();
    invalid_f6["default_enabled"] = json!(true);
    let invalid = [
        (r#"{"enabled": "true"}"#.to_owned(), "invalid f3_pve_policy"),
        (
            r#"{"lucci_vault": {"enabled": true}}"#.to_owned(),
            "invalid f4_extended_policy",
        ),
        (r#"{"note_inbox_cap": 101}"#.to_owned(), "f5_social_policy"),
        (invalid_f6.to_string(), "default_enabled must be false"),
        (r#"{"max_maps_per_owner": 0}"#.to_owned(), "policy"),
    ];
    let malformed = dir.write("malformed.json", "{");
    let missing = dir.0.join("missing.json");
    let not_found = fs::metadata(&missing).unwrap_err().to_string();
    for ((flag, file), (contents, diagnostic)) in PROFILES.into_iter().zip(invalid) {
        let check = |path: &Path| {
            command("policy-audit")
                .arg(flag)
                .arg(path)
                .output()
                .unwrap()
        };
        assert_rejected(check(&missing), &not_found);
        assert_rejected(check(&malformed), "kr-admin:");
        assert_rejected(check(&dir.write(file, contents)), diagnostic);
    }
    let f3 = profile(PROFILES[0].1);
    assert_rejected(
        command("policy-audit")
            .args([
                OsStr::new("--f3-policy"),
                f3.as_os_str(),
                OsStr::new("--f7-policy"),
            ])
            .output()
            .unwrap(),
        "usage:",
    );
}
