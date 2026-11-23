use serde_json::{Value, json};
use std::{
    fs,
    io::Write,
    os::unix::fs::DirBuilderExt,
    path::{Path, PathBuf},
    process::{Command, Output, Stdio},
};

struct TestDir(PathBuf);
impl TestDir {
    fn new() -> Self {
        let mut seed = [0u8; 8];
        getrandom::fill(&mut seed).unwrap();
        let path =
            std::env::temp_dir().join(format!("kr-admin-test-{:016x}", u64::from_ne_bytes(seed)));
        fs::DirBuilder::new().mode(0o700).create(&path).unwrap();
        Self(path)
    }
}
impl Drop for TestDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn command(db: &Path, body: Value) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_kr-admin"))
        .arg(db)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let request = json!({"actor":"on-call","reason":"test rotation","command":body});
    child
        .stdin
        .take()
        .unwrap()
        .write_all(&serde_json::to_vec(&request).unwrap())
        .unwrap();
    child.wait_with_output().unwrap()
}

fn success(db: &Path, cmd: Value) -> Value {
    let run = command(db, cmd);
    assert!(
        run.status.success(),
        "{}",
        String::from_utf8_lossy(&run.stderr)
    );
    serde_json::from_slice(&run.stdout).unwrap()
}
fn failure(db: &Path, cmd: Value, error: &str) {
    let run = command(db, cmd);
    assert!(!run.status.success());
    assert_eq!(
        String::from_utf8_lossy(&run.stderr).trim(),
        format!("kr-admin: {error}")
    );
    assert!(run.stdout.is_empty());
}

#[test]
fn administrative_mutations_and_failures_are_queryable_without_secrets() {
    let path = TestDir::new();
    let db = path.0.join("accounts.sqlite");
    let original = "generated-for-cli-test";
    let changed = "another-cli-test-secret";
    failure(&db, json!({"op":"lookup","login":"alice"}), "Storage");
    assert!(!db.exists());
    success(&db, json!({"op":"init"}));
    success(&db, json!({"op":"begin-instance"}));
    let create = json!({"op":"create","login":"Alice","name":"Alice","password":original});
    let account = success(&db, create.clone());
    let id = &account["player"]["account_id"];
    let player = &account["player"]["player_id"];
    let id_text = id.to_string();
    failure(&db, create, "Conflict");
    let login = json!({"op":"login","login":"alice","password":original,"ttl_seconds":60});
    let grant = success(&db, login.clone());
    let authorize = json!({"op":"authorize","player_id":player,"token":grant["token"]});
    success(&db, authorize.clone());
    success(&db, json!({"op":"kick","account_id":id}));
    failure(&db, authorize, "InvalidSession");
    success(&db, json!({"op":"disable","account_id":id}));
    failure(&db, login, "InvalidCredentials");
    success(&db, json!({"op":"enable","account_id":id}));
    let code = success(&db, json!({"op":"recovery-reset","account_id":id}));
    assert_eq!(code["recovery_code"].as_str().unwrap().len(), 64);
    success(
        &db,
        json!({"op":"password-reset","account_id":id,"password":changed}),
    );
    let sql = rusqlite::Connection::open(&db).unwrap();
    let active_codes: i64 = sql
        .query_row("SELECT count(*) FROM account_recovery", [], |r| r.get(0))
        .unwrap();
    assert_eq!(active_codes, 0);
    success(&db, json!({"op":"delete","account_id":id}));
    failure(&db, json!({"op":"enable","account_id":id}), "InvalidInput");
    failure(
        &db,
        json!({"op":"lookup","login":"alice"}),
        "InvalidCredentials",
    );
    let new = success(
        &db,
        json!({"op":"create","login":"Alice","name":"Alice","password":changed}),
    );
    assert_ne!(new["player"]["player_id"], *player);

    let audit = success(&db, json!({"op":"audit-list","filter":{"limit":100}}));
    let rows = audit["audit"].as_array().unwrap();
    for (op, result) in [
        ("create_account", "ok"),
        ("revoke", "ok"),
        ("disable", "ok"),
        ("enable", "ok"),
        ("reset_recovery", "ok"),
        ("reset_password", "ok"),
        ("delete_account", "ok"),
        ("enable", "InvalidInput"),
    ] {
        assert!(
            rows.iter().any(|r| r["operation"] == op
                && r["result"] == result
                && r["target_kind"] == "account"
                && r["target_id"] == id_text
                && r["actor"] == "on-call"
                && r["at_utc"].as_i64().unwrap() > 0),
            "{op} {result}"
        );
    }
    let filtered = success(
        &db,
        json!({"op":"audit-list","filter":{
            "actor":"on-call","operation":"create_account","target_kind":"login",
            "target_id":"alice","result":"Conflict","limit":1,
        }}),
    );
    assert_eq!(filtered["audit"].as_array().unwrap().len(), 1);
    let text = audit.to_string();
    assert!(
        ![
            original,
            changed,
            code["recovery_code"].as_str().unwrap(),
            grant["token"].as_str().unwrap()
        ]
        .iter()
        .any(|secret| text.contains(secret))
    );
}

#[test]
fn f4_profile_ledger_inventory_and_audited_grants_over_stdin_json() {
    let path = TestDir::new();
    let db = path.0.join("accounts.sqlite");
    success(&db, json!({"op":"init"}));
    success(&db, json!({"op":"begin-instance"}));
    let account = success(
        &db,
        json!({"op":"create","login":"Alice","name":"Alice","password":"generated-for-f4-test"}),
    );
    let player = account["player"]["player_id"].as_i64().unwrap();
    let profile = success(&db, json!({"op":"profile-get","player_id":player}));
    assert_eq!(
        profile,
        json!({"profile":{"player_id":player,"exp":0,"level":1,"lucci":0,"version":1},"equipped":[]})
    );

    let grant = json!({"op":"admin-grant-lucci","player_id":player,"lucci_delta":250,"idempotency_key":"ops-1"});
    let first = success(&db, grant.clone());
    assert_eq!(first["replayed"], false);
    assert_eq!(first["profile"]["lucci"], 250);
    let replay = success(&db, grant);
    assert_eq!(replay["replayed"], true);
    assert_eq!(replay["ledger_id"], first["ledger_id"]);
    assert_eq!(replay["profile"]["lucci"], 250);
    failure(
        &db,
        json!({"op":"admin-grant-lucci","player_id":player,"lucci_delta":251,"idempotency_key":"ops-1"}),
        "Conflict",
    );
    failure(
        &db,
        json!({"op":"admin-grant-lucci","player_id":player,"lucci_delta":-1000,"idempotency_key":"ops-2"}),
        "InsufficientFunds",
    );
    failure(
        &db,
        json!({"op":"admin-grant-lucci","player_id":player,"lucci_delta":1,"idempotency_key":"ops-3","extra":1}),
        "InvalidInput",
    );

    let item = json!({"op":"admin-grant-item","player_id":player,"sku_id":"hat.red","quantity":2,
        "grant":{"kind":"timed","duration_seconds":3600},"idempotency_key":"item-1"});
    let granted = success(&db, item.clone());
    assert_eq!(granted["replayed"], false);
    let expires = granted["item"]["expires_at"].as_i64().unwrap();
    assert_eq!(
        expires - granted["item"]["acquired_at"].as_i64().unwrap(),
        7200
    );
    assert_eq!(success(&db, item)["item"]["expires_at"], expires);
    failure(
        &db,
        json!({"op":"admin-grant-item","player_id":player,"sku_id":"hat.red","quantity":1,
            "grant":{"kind":"timed","duration_seconds":3600},"idempotency_key":"item-1"}),
        "Conflict",
    );
    failure(
        &db,
        json!({"op":"admin-grant-item","player_id":player,"sku_id":"bad sku","quantity":1,
            "grant":{"kind":"permanent"},"idempotency_key":"item-2"}),
        "InvalidInput",
    );
    success(
        &db,
        json!({"op":"admin-grant-item","player_id":player,"sku_id":"char.bazzi","quantity":1,
            "grant":{"kind":"permanent"},"idempotency_key":"item-3"}),
    );

    let inventory = success(&db, json!({"op":"inventory-list","player_id":player}));
    let items = inventory["inventory"].as_array().unwrap();
    assert_eq!(items.len(), 2);
    assert_eq!(items[0]["sku_id"], "char.bazzi");
    assert_eq!(items[1]["grant_kind"], "timed");
    assert_eq!(items[1]["active"], true);
    assert_eq!(items[1]["equipped_slots"], json!([]));
    let ledger = success(
        &db,
        json!({"op":"ledger-list","player_id":player,"limit":10}),
    );
    let entries = ledger["ledger"].as_array().unwrap();
    assert_eq!(entries.len(), 3);
    assert!(entries.iter().all(|e| e["kind"] == "admin_grant"));
    assert_eq!(entries[2]["lucci_delta"], 250);
    failure(
        &db,
        json!({"op":"ledger-list","player_id":player,"limit":0}),
        "InvalidInput",
    );
    failure(
        &db,
        json!({"op":"profile-get","player_id":999}),
        "InvalidInput",
    );
    assert_eq!(
        success(&db, json!({"op":"profile-get","player_id":player}))["profile"]["version"],
        4
    );

    let audit = success(
        &db,
        json!({"op":"audit-list","filter":{"target_kind":"player","target_id":player.to_string(),"limit":100}}),
    );
    let rows = audit["audit"].as_array().unwrap();
    let has = |op: &str, result: &str| {
        rows.iter()
            .filter(|r| r["operation"] == op && r["result"] == result && r["actor"] == "on-call")
            .count()
    };
    assert_eq!(has("admin_grant_lucci", "ok"), 1);
    assert_eq!(has("admin_grant_lucci", "replayed"), 1);
    assert_eq!(has("admin_grant_lucci", "Conflict"), 1);
    assert_eq!(has("admin_grant_lucci", "InsufficientFunds"), 1);
    assert_eq!(has("admin_grant_item", "ok"), 2);
    assert_eq!(has("admin_grant_item", "replayed"), 1);
    assert_eq!(has("admin_grant_item", "Conflict"), 1);
    assert_eq!(has("admin_grant_item", "InvalidInput"), 1);
    assert_eq!(has("profile_get", "ok"), 2);
    assert_eq!(has("profile_get", "InvalidInput"), 0);
    assert_eq!(has("ledger_list", "ok"), 1);
    assert_eq!(has("inventory_list", "ok"), 1);
    let sql = rusqlite::Connection::open(&db).unwrap();
    let applied: i64 = sql
        .query_row(
            "SELECT count(*) FROM admin_audit WHERE operation LIKE 'admin_grant_%' AND result='ok' AND account_id IS NOT NULL",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(applied, 3);
    let lucci: i64 = sql
        .query_row("SELECT lucci FROM player_profiles", [], |r| r.get(0))
        .unwrap();
    assert_eq!(lucci, 250);
}
