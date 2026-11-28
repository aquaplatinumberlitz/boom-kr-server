use kr_service::custom_map_store::{CustomMapPolicy, CustomMapStore};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output, Stdio},
};

struct TestDir(PathBuf);

impl TestDir {
    fn new() -> Self {
        let path = std::env::temp_dir()
            .canonicalize()
            .unwrap()
            .join(format!("kr-f3-f7-admin-{}", getrandom::u64().unwrap()));
        let mut builder = fs::DirBuilder::new();
        #[cfg(unix)]
        {
            use std::os::unix::fs::DirBuilderExt;
            builder.mode(0o700);
        }
        builder.create(&path).unwrap();
        Self(path)
    }

    fn write(&self, name: &str, contents: &str) -> PathBuf {
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

fn admin(command: &str) -> Command {
    let mut cli = Command::new(env!("CARGO_BIN_EXE_kr-admin"));
    cli.arg(command).stdin(Stdio::null());
    cli
}

fn dry_run(path: &Path, mode: &str, extra: &[&str]) -> Output {
    admin("f3-pve-dry-run")
        .arg("--f3-policy")
        .arg(path)
        .args(["--mode", mode])
        .args(extra)
        .output()
        .unwrap()
}

fn list_maps(policy: &Path, root: &Path, owner: Option<&str>) -> Output {
    let mut cli = admin("f7-custom-maps-list");
    cli.arg("--f7-policy")
        .arg(policy)
        .arg("--custom-maps-dir")
        .arg(root);
    if let Some(owner) = owner {
        cli.args(["--owner", owner]);
    }
    cli.output().unwrap()
}

fn assert_rejected(output: Output, diagnostic: &str) -> String {
    assert_eq!(output.status.code(), Some(1), "{output:?}");
    assert!(output.stdout.is_empty(), "{output:?}");
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains(diagnostic), "{stderr}");
    stderr
}

fn summary(output: &Output) -> Value {
    assert!(output.status.success(), "{output:?}");
    assert!(output.stderr.is_empty(), "{output:?}");
    serde_json::from_slice(&output.stdout).unwrap()
}

fn enabled_store(dir: &TestDir) -> (PathBuf, PathBuf, CustomMapStore) {
    let policy = dir.write("f7-policy.json", r#"{"enabled":true}"#);
    let root = dir.0.join("maps");
    let store = CustomMapStore::new(&root, CustomMapPolicy::from_path(&policy).unwrap()).unwrap();
    (policy, root, store)
}

fn map_payload() -> Vec<u8> {
    serde_json::to_vec(&json!({
        "schema": "kre-f7-normalized-map-v1",
        "width": 15,
        "height": 13,
        "spawns": [[0, 0], [14, 12]],
        "tiles": vec![0; 15 * 13],
    }))
    .unwrap()
}

#[test]
fn checked_in_disabled_profiles_fail_closed() {
    let dir = TestDir::new();
    let root = dir.0.join("maps");
    assert_rejected(
        dry_run(&profile("f3-pve-disabled.json"), "MonsterS1", &[]),
        "F3 PvE policy is disabled",
    );
    assert_rejected(
        list_maps(&profile("f7-custom-map-disabled.json"), &root, None),
        "custom map store is disabled",
    );
    assert!(!root.exists());
}

#[test]
fn f3_enabled_fixture_checks_host_and_sequence_without_claiming_wire_dispatch() {
    let dir = TestDir::new();
    let policy = dir.write("f3.json", r#"{"enabled":true}"#);
    let extra = ["--host-player-id", "42", "--guest-player-id", "7"];
    let first = dry_run(&policy, "monster_s1", &extra);
    assert_eq!(
        summary(&first),
        json!({
            "mode": "MonsterS1",
            "host_player_id": 42,
            "guest_player_id": 7,
            "checks": {
                "session_start": "accepted",
                "host_attack": "accepted",
                "guest_attack": "not_host",
                "non_monotonic_sequence": "non_increasing_sequence",
                "missing_dispatch_evidence": "wine_dispatch_evidence_required",
            },
            "dispatch": {
                "fixture_evidence": "synthetic",
                "status": "static_candidate_pending_live_wine_capture",
                "jump_table_va": "0xE5DE9C",
                "opcode_index_subtrahend": 20,
            },
        }),
    );
    let second = dry_run(&policy, "MonsterS1", &extra);
    assert_eq!(summary(&first), summary(&second));
    assert_eq!(first.stdout, second.stdout);
    let defaults = summary(&dry_run(&policy, "DragonPalace", &[]));
    assert_eq!(defaults["mode"], "DragonPalace");
    assert_eq!(defaults["host_player_id"], 1);
    assert_eq!(defaults["guest_player_id"], 2);
}

#[test]
fn f3_rejects_invalid_policy_mode_and_player_ids() {
    let dir = TestDir::new();
    let policy = dir.write("f3.json", r#"{"enabled":false,"enabled":true}"#);
    assert_rejected(dry_run(&policy, "MonsterS1", &[]), "invalid f3_pve_policy");
    fs::write(&policy, r#"{"enabled":true}"#).unwrap();
    assert_rejected(dry_run(&policy, "CustomMap", &[]), "invalid --mode");
    for extra in [
        &["--host-player-id", "0"][..],
        &["--guest-player-id", "1"],
        &["--host-player-id", "7", "--guest-player-id", "7"],
    ] {
        assert_rejected(dry_run(&policy, "MonsterS1", extra), "distinct, nonzero");
    }
    assert_rejected(
        dry_run(&policy, "MonsterS1", &["--host-player-id", "4294967296"]),
        "invalid --host-player-id",
    );
}

#[test]
fn new_commands_require_their_policy_and_selector_arguments() {
    for (subcommand, flag) in [
        ("f3-pve-dry-run", "--mode"),
        ("f7-custom-maps-list", "--custom-maps-dir"),
    ] {
        assert_rejected(
            admin(subcommand).output().unwrap(),
            &format!("usage: kr-admin {subcommand}"),
        );
        assert_rejected(
            admin(subcommand).args([flag, "example"]).output().unwrap(),
            &format!("usage: kr-admin {subcommand}"),
        );
    }
}

#[test]
fn f7_lists_verified_maps_in_owner_and_id_order_and_honors_filter() {
    let dir = TestDir::new();
    let (policy, root, store) = enabled_store(&dir);
    assert_eq!(
        summary(&list_maps(&policy, &root, None)),
        json!({"maps": []})
    );
    assert!(!root.exists());

    let mut rows = Vec::new();
    for (i, (owner, id)) in [("zoe", "z"), ("alice", "q"), ("alice", "a")]
        .into_iter()
        .enumerate()
    {
        let mut payload = map_payload();
        payload.extend(std::iter::repeat_n(b'\n', i));
        store.save(owner, id, &payload).unwrap();
        let sha256: String = Sha256::digest(&payload)
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect();
        rows.push(json!({
            "id": id,
            "owner": owner,
            "name": id,
            "bytes": payload.len(),
            "sha256": sha256,
        }));
    }
    rows.reverse();
    let global = list_maps(&policy, &root, None);
    assert_eq!(summary(&global), json!({"maps": rows}));
    assert_eq!(global.stdout, list_maps(&policy, &root, None).stdout);
    assert_eq!(
        summary(&list_maps(&policy, &root, Some("alice"))),
        json!({"maps": &rows[..2]}),
    );
    assert_eq!(
        summary(&list_maps(&policy, &root, Some("missing"))),
        json!({"maps": []}),
    );
    assert_rejected(
        list_maps(&policy, &root, Some("../maps")),
        "invalid custom map owner ID",
    );
}

#[test]
fn f7_corrupt_sha256_fails_the_entire_listing_without_printing_paths() {
    let dir = TestDir::new();
    let (policy, root, store) = enabled_store(&dir);
    let payload = map_payload();
    store.save("alice", "valid", &payload).unwrap();
    store.save("zoe", "corrupt", &payload).unwrap();
    let record = root.join("zoe/corrupt.json");
    let mut contents: Value = serde_json::from_slice(&fs::read(&record).unwrap()).unwrap();
    contents["metadata"]["sha256"] = json!("0".repeat(64));
    fs::write(&record, serde_json::to_vec(&contents).unwrap()).unwrap();

    for owner in [None, Some("zoe")] {
        let stderr = assert_rejected(
            list_maps(&policy, &root, owner),
            "stored custom map failed integrity verification",
        );
        assert!(!stderr.contains(dir.0.to_str().unwrap()));
    }
    assert_eq!(
        summary(&list_maps(&policy, &root, Some("alice")))["maps"][0]["id"],
        "valid",
    );
}

#[test]
fn f7_rejects_invalid_policy_without_echoing_policy_values_or_paths() {
    let dir = TestDir::new();
    let root = dir.0.join("maps");
    let policy = dir.write("f7.json", r#"{"enabled":true,"max_maps_per_owner":0}"#);
    assert_rejected(list_maps(&policy, &root, None), "f7_custom_map_policy");
    fs::write(
        &policy,
        json!({"enabled": root.to_str().unwrap()}).to_string(),
    )
    .unwrap();
    let stderr = assert_rejected(list_maps(&policy, &root, None), "f7_custom_map_policy");
    assert!(!stderr.contains(dir.0.to_str().unwrap()));
    assert!(!root.exists());
}

#[cfg(unix)]
#[test]
fn f7_rejects_symlinks_at_store_owner_and_map_levels() {
    use std::os::unix::fs::symlink;

    let dir = TestDir::new();
    let (policy, root, store) = enabled_store(&dir);
    store.save("alice", "real", &map_payload()).unwrap();

    let root_link = dir.0.join("root-link");
    symlink(&root, &root_link).unwrap();
    let owner_link = root.join("owner-link");
    symlink(root.join("alice"), &owner_link).unwrap();
    for (root, owner) in [
        (root_link.as_path(), None),
        (root.as_path(), None),
        (root.as_path(), Some("owner-link")),
    ] {
        let stderr = assert_rejected(list_maps(&policy, root, owner), "symlinks are not allowed");
        assert!(!stderr.contains(dir.0.to_str().unwrap()));
    }
    fs::remove_file(owner_link).unwrap();
    symlink(root.join("alice/real.json"), root.join("alice/alias.json")).unwrap();
    for owner in [None, Some("alice")] {
        let stderr = assert_rejected(list_maps(&policy, &root, owner), "symlinks are not allowed");
        assert!(!stderr.contains(dir.0.to_str().unwrap()));
    }
}
