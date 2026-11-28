use kr_service::accounts::{Admin, Error, Player, Store};
use rusqlite::Connection;
use serde_json::{Value, json};
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output, Stdio},
    sync::Mutex,
    thread,
    time::Duration,
};

const COUPON: &str = "f4-redeem-coupon";
const TITLE: &str = "f4-evaluate-title";
const OPERATOR: &str = "on-call";
const REASON: &str = "check F4 operator workflow";
static CREATE_ACCOUNT_GATE: Mutex<()> = Mutex::new(());

struct TestDir(PathBuf);

impl TestDir {
    fn new() -> Self {
        let path = std::env::temp_dir()
            .canonicalize()
            .unwrap()
            .join(format!("kr-f4-admin-{}", getrandom::u64().unwrap()));
        let mut builder = fs::DirBuilder::new();
        #[cfg(unix)]
        {
            use std::os::unix::fs::DirBuilderExt;
            builder.mode(0o700);
        }
        builder.create(&path).unwrap();
        Self(path)
    }

    fn write(&self, name: &str, value: Value) -> PathBuf {
        let path = self.0.join(name);
        fs::write(&path, value.to_string()).unwrap();
        path
    }
}

impl Drop for TestDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn create_account(store: &mut Store, login: &str) -> Player {
    let _guard = CREATE_ACCOUNT_GATE
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let admin = Admin {
        actor: "f4-admin-tests",
        reason: "create fixture account",
    };
    for _ in 0..200 {
        match store.create_account(&admin, login, login, b"test-only-password", 100) {
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
    dir: TestDir,
}

impl Fixture {
    fn new() -> Self {
        let dir = TestDir::new();
        let db = dir.0.join("accounts.sqlite");
        let mut store = Store::open(&db).unwrap();
        let alice = create_account(&mut store, "Alice");
        let policy = dir.write(
            "f4.json",
            json!({
                "gift_coupons": {
                    "enabled": true,
                    "coupon_offers": [
                        {"offer": "ten-lucci", "valid_for_seconds": 60,
                         "grant": {"sku": "lucci", "quantity": 10}},
                        {"offer": "two-chips", "valid_for_seconds": 60,
                         "grant": {"sku": "chip", "quantity": 2}}
                    ]
                },
                "lucci_vault": {"enabled": true, "base_lucci_cap": 15, "expansion_step": 1,
                                "max_expansions": 0, "max_lucci_cap": 15},
                "titles": {
                    "enabled": true,
                    "equip_slot": "title",
                    "notify_on_unlock": true,
                    "unlock_table": [
                        {"title": "newbie", "condition": "level>=1"},
                        {"title": "winner", "condition": "wins>=1"}
                    ]
                }
            }),
        );
        Self {
            store,
            observer: Connection::open(&db).unwrap(),
            db,
            policy,
            alice,
            dir,
        }
    }

    fn invoke(&self, subcommand: &str, policy: Option<&Path>, args: &[(&str, &str)]) -> Output {
        let mut cli = admin(subcommand);
        cli.arg("--db").arg(&self.db);
        if let Some(path) = policy {
            cli.arg("--f4-policy").arg(path);
        }
        for &(flag, value) in args {
            cli.args([flag, value]);
        }
        let output = cli.output().unwrap();
        let root = self.dir.0.to_string_lossy();
        for stream in [&output.stdout, &output.stderr] {
            assert!(!String::from_utf8_lossy(stream).contains(root.as_ref()));
        }
        output
    }

    fn run(&self, subcommand: &str, args: &[(&str, &str)]) -> Output {
        self.invoke(subcommand, Some(&self.policy), args)
    }

    fn version(&self) -> i64 {
        self.observer
            .pragma_query_value(None, "data_version", |row| row.get(0))
            .unwrap()
    }

    fn rejects_without_writing(&self, diagnostic: &str, run: impl FnOnce() -> Output) {
        let before = self.version();
        rejected(run(), diagnostic);
        assert_eq!(self.version(), before);
    }

    fn audit_count(&self, operation: &str) -> i64 {
        self.observer
            .query_row(
                "SELECT count(*) FROM admin_audit WHERE operation=?",
                [operation],
                |row| row.get(0),
            )
            .unwrap()
    }

    fn assert_audit(
        &self,
        summary: &Value,
        operation: &str,
        kind: &str,
        result: &str,
        target: Value,
    ) {
        let request = summary["operator_audit_request_id"].as_str().unwrap();
        self.observer
            .query_row(
                "SELECT actor, reason, operation, target_kind, result, target_id
                 FROM admin_audit WHERE request_id=?",
                [request],
                |row| {
                    for (i, expected) in [OPERATOR, REASON, operation, kind, result]
                        .into_iter()
                        .enumerate()
                    {
                        assert_eq!(row.get::<_, String>(i)?, expected);
                    }
                    assert_eq!(
                        serde_json::from_str::<Value>(&row.get::<_, String>(5)?).unwrap(),
                        target
                    );
                    Ok(())
                },
            )
            .unwrap();
    }
}

fn admin(subcommand: &str) -> Command {
    let mut cli = Command::new(env!("CARGO_BIN_EXE_kr-admin"));
    cli.arg(subcommand).stdin(Stdio::null());
    cli
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

fn summary(output: Output) -> Value {
    assert!(output.status.success(), "{output:?}");
    assert!(output.stderr.is_empty(), "{output:?}");
    serde_json::from_slice(&output.stdout).unwrap()
}

fn coupon_args(issued_to: &str) -> [(&str, &str); 9] {
    [
        ("--account", "Alice"),
        ("--coupon-id", "42"),
        ("--request-id", "17"),
        ("--offer", "ten-lucci"),
        ("--issued-to", issued_to),
        ("--issued-at-unix", "100"),
        ("--now-unix", "101"),
        ("--operator", OPERATOR),
        ("--reason", REASON),
    ]
}

fn title_args(title: &str) -> [(&str, &str); 5] {
    [
        ("--account", "Alice"),
        ("--title", title),
        ("--now-unix", "101"),
        ("--operator", OPERATOR),
        ("--reason", REASON),
    ]
}

fn replace<'a>(
    args: &[(&'a str, &'a str)],
    flag: &'a str,
    value: &'a str,
) -> Vec<(&'a str, &'a str)> {
    args.iter()
        .filter(|(key, _)| *key != flag)
        .copied()
        .chain([(flag, value)])
        .collect()
}

#[test]
fn disabled_and_invalid_profiles_fail_closed() {
    let f = Fixture::new();
    let disabled = Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(3)
        .unwrap()
        .join("server/profiles/f4-extended-disabled.json");
    let defaults = f.dir.write("defaults.json", json!({}));
    let invalid = f
        .dir
        .write("invalid.json", json!({"titles": {"enabled": true}}));
    let issued_to = f.alice.account_id.to_string();
    let coupon = coupon_args(&issued_to);
    let title = title_args("newbie");
    for (subcommand, args, diagnostic) in [
        (COUPON, &coupon[..], "F4 gift coupon policy is disabled"),
        (TITLE, &title[..], "F4 title policy is disabled"),
    ] {
        for path in [&disabled, &defaults] {
            f.rejects_without_writing(diagnostic, || f.invoke(subcommand, Some(path), args));
        }
        f.rejects_without_writing("invalid f4_extended_policy configuration", || {
            f.invoke(subcommand, Some(&invalid), args)
        });
        f.rejects_without_writing("usage: kr-admin", || f.invoke(subcommand, None, args));
    }
}

#[test]
fn coupon_redeems_once_and_audits_apply_and_replay() {
    let mut f = Fixture::new();
    let bob = create_account(&mut f.store, "Bobby");
    let player = f.alice.player_id;
    let issued_to = f.alice.account_id.to_string();
    let mut args = coupon_args(&issued_to);
    let applied = summary(f.run(COUPON, &args));
    assert_eq!(applied["player_id"], player);
    assert_eq!(applied["coupon_id"], 42);
    assert_eq!(applied["replayed"], false);
    assert_eq!(applied["ledger"]["lucci_delta"], 10);
    assert_eq!(applied["ledger"]["created_at"], 101);

    let player_id = player.to_string();
    args[0] = ("--player-id", &player_id);
    let replay = summary(f.run(COUPON, &args));
    assert_eq!(replay["replayed"], true);
    assert_eq!(replay["ledger"], applied["ledger"]);
    assert_ne!(
        replay["operator_audit_request_id"],
        applied["operator_audit_request_id"]
    );
    for (output, result) in [(&applied, "ok"), (&replay, "replayed")] {
        f.assert_audit(
            output,
            "f4_redeem_coupon",
            "f4_coupon",
            result,
            json!({"player_id": player, "coupon_id": 42, "ledger_id": applied["ledger"]["id"]}),
        );
    }

    for (flag, value) in [
        ("--offer", "two-chips"),
        ("--request-id", "18"),
        ("--issued-at-unix", "101"),
    ] {
        f.rejects_without_writing("Conflict", || f.run(COUPON, &replace(&args, flag, value)));
    }
    args[0] = ("--account", "Bobby");
    f.rejects_without_writing("InvalidInput", || f.run(COUPON, &args));
    f.rejects_without_writing("Conflict", || {
        f.run(
            COUPON,
            &replace(&args, "--issued-to", &bob.account_id.to_string()),
        )
    });
    assert_eq!(f.store.get_profile(player).unwrap().lucci, 10);
    assert_eq!(f.store.get_profile(bob.player_id).unwrap().lucci, 0);
    assert_eq!(f.store.list_ledger(player, 10).unwrap().len(), 1);
    assert!(f.store.list_ledger(bob.player_id, 10).unwrap().is_empty());
    assert_eq!(f.audit_count("f4_redeem_coupon"), 2);
}

#[test]
fn coupon_uses_catalog_vault_and_expiry_rules() {
    let f = Fixture::new();
    let issued_to = f.alice.account_id.to_string();
    let args = coupon_args(&issued_to);
    f.rejects_without_writing("InsufficientQuantity", || {
        f.run(COUPON, &replace(&args, "--now-unix", "160"))
    });
    summary(f.run(COUPON, &args));
    let second = replace(&args, "--coupon-id", "43");
    f.rejects_without_writing("Conflict", || f.run(COUPON, &second));
    let mut item = replace(&second, "--offer", "two-chips");
    f.rejects_without_writing("InvalidInput", || f.run(COUPON, &item));
    let catalog = f.dir.write(
        "catalog.json",
        json!({"items": [{
            "sku_id": "chip", "category": "consumable", "price_lucci": 0,
            "grant": {"kind": "stackable", "max_stack": 4}, "equip_slot": null, "enabled": true
        }]}),
    );
    item.push(("--shop-catalog", catalog.to_str().unwrap()));
    let output = summary(f.run(COUPON, &item));
    assert_eq!(output["ledger"]["sku_id"], "chip");
    assert_eq!(output["ledger"]["lucci_delta"], 0);
    let inventory = f.store.list_inventory(f.alice.player_id, 101).unwrap();
    assert_eq!(inventory.len(), 1);
    assert_eq!(inventory[0].sku_id, "chip");
    assert_eq!(inventory[0].quantity, 2);
}

#[test]
fn title_evaluation_repeats_without_unlocking_and_audits_the_decision() {
    let f = Fixture::new();
    let player = f.alice.player_id;
    let before = f.store.get_profile(player).unwrap();
    let mut args = title_args("newbie");
    let first = summary(f.run(TITLE, &args));
    let id = player.to_string();
    args[0] = ("--player-id", &id);
    let repeated = summary(f.run(TITLE, &args));
    assert_ne!(
        first["operator_audit_request_id"],
        repeated["operator_audit_request_id"]
    );
    for output in [&first, &repeated] {
        assert_eq!(output["player_id"], player);
        assert_eq!(output["eligible"], true);
        assert_eq!(output["decision"], "eligible");
        assert_eq!(output["notify_after_commit"], true);
        assert_eq!(output["persisted"], false);
        f.assert_audit(
            output,
            "f4_evaluate_title",
            "f4_title",
            "ok",
            json!({"player_id": player, "title": "newbie", "decision": "eligible"}),
        );
    }
    for title in ["winner", "unknown"] {
        let output = summary(f.run(TITLE, &title_args(title)));
        assert_eq!(output["eligible"], false);
        assert_eq!(output["decision"], "not_eligible");
        assert_eq!(output["notify_after_commit"], false);
        f.assert_audit(
            &output,
            "f4_evaluate_title",
            "f4_title",
            "ok",
            json!({"player_id": player, "title": title, "decision": "not_eligible"}),
        );
    }
    assert_eq!(f.audit_count("f4_evaluate_title"), 4);
    assert_eq!(f.store.get_profile(player).unwrap(), before);
    assert!(f.store.list_ledger(player, 10).unwrap().is_empty());
    assert!(f.store.list_inventory(player, 101).unwrap().is_empty());
}

#[test]
fn malformed_arguments_and_unknown_targets_never_write() {
    let f = Fixture::new();
    let issued_to = f.alice.account_id.to_string();
    let coupon = coupon_args(&issued_to);
    let title = title_args("newbie");
    for (subcommand, args) in [(COUPON, &coupon[..]), (TITLE, &title[..])] {
        let usage = format!("usage: kr-admin {subcommand}");
        rejected(admin(subcommand).output().unwrap(), &usage);
        rejected(admin(subcommand).arg("--db").output().unwrap(), &usage);
        for (required, _) in args {
            let incomplete: Vec<_> = args
                .iter()
                .filter(|(key, _)| key != required)
                .copied()
                .collect();
            f.rejects_without_writing(&usage, || f.run(subcommand, &incomplete));
        }
        for (flag, value, diagnostic) in [
            ("--unknown", "value", usage.as_str()),
            ("--player-id", "0", "invalid --player-id"),
            ("--player-id", "999", "identify different players"),
        ] {
            let mut invalid = args.to_vec();
            invalid.push((flag, value));
            f.rejects_without_writing(diagnostic, || f.run(subcommand, &invalid));
        }
        let mut duplicate = args.to_vec();
        duplicate.push(("--operator", "duplicate"));
        f.rejects_without_writing(&usage, || f.run(subcommand, &duplicate));
        for (flag, value, diagnostic) in [
            ("--now-unix", "-1", "invalid --now-unix"),
            ("--reason", "", "InvalidInput"),
            ("--account", "Nobody", "InvalidCredentials"),
        ] {
            f.rejects_without_writing(diagnostic, || {
                f.run(subcommand, &replace(args, flag, value))
            });
        }
    }
    for (flag, value, diagnostic) in [
        ("--coupon-id", "0", "invalid --coupon-id"),
        ("--request-id", "abc", "invalid --request-id"),
        ("--issued-to", "0", "invalid --issued-to"),
        ("--issued-at-unix", "102", "must not exceed --now-unix"),
        ("--offer", "missing", "InvalidInput"),
    ] {
        f.rejects_without_writing(diagnostic, || f.run(COUPON, &replace(&coupon, flag, value)));
    }
    f.rejects_without_writing("invalid --title", || f.run(TITLE, &title_args("")));
}
