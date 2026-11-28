use kr_service::custom_map_store::{
    CustomMapPolicy, CustomMapStore, CustomMapStoreError as Error, CustomMapWireContract,
};
use kr_wire::kr00967::custom_map::{
    CUSTOM_MAP_SCHEMA, CustomMapError, CustomMapWireRoute, CustomMapWireStatus,
    MAX_CUSTOM_MAP_BYTES,
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    sync::Barrier,
    thread,
};

struct Scratch(PathBuf);

impl Scratch {
    fn new() -> Self {
        let mut nonce = [0; 16];
        getrandom::fill(&mut nonce).unwrap();
        let path = std::env::temp_dir().join(format!("kr-f7-{:032x}", u128::from_ne_bytes(nonce)));
        fs::create_dir(&path).unwrap();
        Self(path)
    }

    fn path(&self, name: &str) -> PathBuf {
        self.0.join(name)
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn policy() -> CustomMapPolicy {
    CustomMapPolicy {
        enabled: true,
        max_maps_per_owner: 2,
        max_payload_bytes: MAX_CUSTOM_MAP_BYTES,
    }
}

fn palette() -> BTreeMap<u16, bool> {
    BTreeMap::from([(0, true), (1, false)])
}

fn store(root: &Path, policy: CustomMapPolicy) -> CustomMapStore {
    CustomMapStore::with_palette(root, policy, palette()).unwrap()
}

fn payload(tiles: &[u16]) -> Vec<u8> {
    serde_json::to_vec(&json!({
        "schema": CUSTOM_MAP_SCHEMA,
        "width": 15,
        "height": 13,
        "tiles": tiles,
        "spawns": [[0, 0], [14, 12]],
    }))
    .unwrap()
}

fn valid_payload() -> Vec<u8> {
    payload(&[0; 15 * 13])
}

fn record_path(root: &Path) -> PathBuf {
    root.join("owner/map.json")
}

#[test]
fn disabled_policy_rejects_every_operation_without_creating_a_root() {
    let scratch = Scratch::new();
    let root = scratch.path("maps");
    let path = scratch.path("policy.json");
    fs::write(&path, "{}").unwrap();
    let disabled = CustomMapPolicy::from_path(&path).unwrap();
    assert_eq!(disabled, CustomMapPolicy::default());
    assert!(!disabled.enabled);
    let store = store(&root, disabled);

    assert!(matches!(
        store.save("owner", "map", &valid_payload()),
        Err(Error::Disabled)
    ));
    assert!(matches!(store.get("owner", "map"), Err(Error::Disabled)));
    assert!(matches!(store.list_by_owner("owner"), Err(Error::Disabled)));
    assert!(matches!(store.delete("owner", "map"), Err(Error::Disabled)));
    assert!(matches!(
        store.quota_remaining("owner"),
        Err(Error::Disabled)
    ));
    assert!(!root.exists());
}

#[test]
fn roundtrip_persists_verified_payload_metadata_and_unmeasured_marker() {
    let scratch = Scratch::new();
    let root = scratch.path("maps");
    let mut payload = valid_payload();
    payload.resize(MAX_CUSTOM_MAP_BYTES, b' ');
    let first = store(&root, policy());
    let saved = first.save("owner", "map", &payload).unwrap();
    let expected_sha256 = Sha256::digest(&payload)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    assert_eq!(saved.sha256, expected_sha256);
    assert_eq!((saved.width, saved.height, saved.spawn_count), (15, 13, 2));
    assert_eq!(saved.schema, CUSTOM_MAP_SCHEMA);
    assert_eq!(
        saved.wire_contract,
        CustomMapWireContract::UnmeasuredWireContract
    );
    assert_eq!(
        saved.wire_contract.status(),
        CustomMapWireStatus::Unmeasured
    );

    let on_disk: Value = serde_json::from_slice(&fs::read(record_path(&root)).unwrap()).unwrap();
    assert_eq!(
        on_disk["metadata"]["wire_contract"],
        "unmeasured_wire_contract"
    );
    for route in [CustomMapWireRoute::Upload, CustomMapWireRoute::Select] {
        assert_eq!(
            route.validate_dispatch(),
            Err(CustomMapError::UnmeasuredWireContract)
        );
    }
    drop(first);

    let reopened = store(&root, policy());
    let loaded = reopened.get("owner", "map").unwrap().unwrap();
    assert_eq!(loaded.payload, payload);
    assert_eq!(loaded.metadata, saved);
    assert_eq!(reopened.list_by_owner("owner").unwrap(), [saved]);
    assert!(reopened.list_by_owner("other").unwrap().is_empty());
    assert!(reopened.get("other", "map").unwrap().is_none());

    assert!(reopened.delete("owner", "map").unwrap());
    assert!(!reopened.delete("owner", "map").unwrap());
    assert!(reopened.get("owner", "map").unwrap().is_none());
    assert!(reopened.list_by_owner("owner").unwrap().is_empty());
    assert_eq!(reopened.quota_remaining("owner").unwrap(), 2);
}

#[test]
fn changed_payload_metadata_and_record_identity_fail_closed() {
    let scratch = Scratch::new();
    let root = scratch.path("maps");
    let store = store(&root, policy());
    let bytes = valid_payload();
    store.save("owner", "map", &bytes).unwrap();
    let path = record_path(&root);
    let original: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();

    // Extra whitespace still gives a valid normalized map, but changes its raw digest.
    let mut changed = original.clone();
    changed["payload"] = json!(format!("{} ", std::str::from_utf8(&bytes).unwrap()));
    fs::write(&path, serde_json::to_vec(&changed).unwrap()).unwrap();
    assert!(matches!(store.get("owner", "map"), Err(Error::Integrity)));
    assert!(matches!(
        store.list_by_owner("owner"),
        Err(Error::Integrity)
    ));

    for (field, value) in [("width", json!(21)), ("wire_contract", json!("measured"))] {
        let mut changed = original.clone();
        changed["metadata"][field] = value;
        fs::write(&path, serde_json::to_vec(&changed).unwrap()).unwrap();
        assert!(matches!(store.get("owner", "map"), Err(Error::Integrity)));
    }

    fs::write(
        root.join("owner/other.json"),
        serde_json::to_vec(&original).unwrap(),
    )
    .unwrap();
    assert!(matches!(store.get("owner", "other"), Err(Error::Integrity)));
}

#[test]
fn quota_is_per_owner_and_replacing_a_map_does_not_consume_a_slot() {
    let scratch = Scratch::new();
    let root = scratch.path("maps");
    let store = store(&root, policy());
    let bytes = valid_payload();
    assert_eq!(store.quota_remaining("owner").unwrap(), 2);
    store.save("owner", "b", &bytes).unwrap();
    store.save("owner", "a", &bytes).unwrap();
    assert_eq!(store.quota_remaining("owner").unwrap(), 0);
    assert!(matches!(
        store.save("owner", "c", &bytes),
        Err(Error::QuotaExceeded)
    ));
    let replacement = [bytes.as_slice(), b"\n"].concat();
    store.save("owner", "a", &replacement).unwrap();
    assert_eq!(
        store.get("owner", "a").unwrap().unwrap().payload,
        replacement
    );
    let ids: Vec<_> = store
        .list_by_owner("owner")
        .unwrap()
        .into_iter()
        .map(|meta| meta.map_id)
        .collect();
    assert_eq!(ids, ["a", "b"]);
    store.save("other", "c", &bytes).unwrap();
    assert_eq!(store.quota_remaining("other").unwrap(), 1);
    store.delete("owner", "b").unwrap();
    store.save("owner", "c", &bytes).unwrap();
}

#[test]
fn concurrent_store_handles_cannot_exceed_the_owner_quota() {
    let scratch = Scratch::new();
    let root = scratch.path("maps");
    let policy = CustomMapPolicy {
        max_maps_per_owner: 1,
        ..policy()
    };
    let first = store(&root, policy.clone());
    let second = store(&root, policy);
    let gate = Barrier::new(2);
    let results = thread::scope(|scope| {
        let first_write = scope.spawn(|| {
            gate.wait();
            first.save("owner", "first", &valid_payload())
        });
        gate.wait();
        let second_write = second.save("owner", "second", &valid_payload());
        [first_write.join().unwrap(), second_write]
    });
    assert_eq!(results.iter().filter(|result| result.is_ok()).count(), 1);
    assert_eq!(
        results
            .iter()
            .filter(|result| matches!(result, Err(Error::QuotaExceeded)))
            .count(),
        1
    );
    assert_eq!(first.quota_remaining("owner").unwrap(), 0);
}

#[test]
fn rejects_over_limit_payloads_and_maps_without_reachable_spawns() {
    let scratch = Scratch::new();
    let root = scratch.path("maps");
    let store = store(&root, policy());
    assert!(matches!(
        store.save("owner", "large", &vec![b' '; MAX_CUSTOM_MAP_BYTES + 1]),
        Err(Error::PayloadTooLarge)
    ));
    let limited = CustomMapStore::with_palette(
        &root,
        CustomMapPolicy {
            max_payload_bytes: 32,
            ..policy()
        },
        palette(),
    )
    .unwrap();
    assert!(matches!(
        limited.save("owner", "large", &valid_payload()),
        Err(Error::PayloadTooLarge)
    ));

    let mut tiles = [0; 15 * 13];
    for row in tiles.as_chunks_mut::<15>().0 {
        row[7] = 1;
    }
    assert!(matches!(
        store.save("owner", "disconnected", &payload(&tiles)),
        Err(Error::InvalidMap(CustomMapError::UnreachableSpawn([
            14, 12
        ])))
    ));

    let no_palette = CustomMapStore::with_palette(&root, policy(), BTreeMap::new()).unwrap();
    assert!(matches!(
        no_palette.save("owner", "no-palette", &valid_payload()),
        Err(Error::InvalidMap(CustomMapError::MissingTrustedPalette))
    ));
    assert!(!root.exists());
}

#[test]
fn rejects_invalid_owner_and_map_ids() {
    let scratch = Scratch::new();
    let root = scratch.path("maps");
    let store = store(&root, policy());
    let bytes = valid_payload();
    for id in [
        "",
        ".",
        "..",
        "../evil",
        "has/slash",
        "has\\slash",
        "nul\0",
        "line\n",
        "control\u{85}",
    ] {
        assert!(
            matches!(store.save(id, "map", &bytes), Err(Error::InvalidOwnerId)),
            "{id:?}"
        );
        assert!(
            matches!(store.get("owner", id), Err(Error::InvalidMapId)),
            "{id:?}"
        );
        assert!(
            matches!(store.save("owner", id, &bytes), Err(Error::InvalidMapId)),
            "{id:?}"
        );
    }
    assert!(!root.exists());
    assert!(!scratch.path("evil").exists());
}

#[test]
fn policy_requires_an_object_and_rejects_unknown_duplicate_and_invalid_limits() {
    let scratch = Scratch::new();
    let path = scratch.path("policy.json");

    fs::write(
        &path,
        r#"{"enabled":true,"max_maps_per_owner":2,"max_payload_bytes":8192}"#,
    )
    .unwrap();
    assert_eq!(CustomMapPolicy::from_path(&path).unwrap(), policy());
    for invalid in [
        r#"{"enabled":false,"enabled":true}"#,
        r#"{"max_maps_per_owner":1,"max_maps_per_owner":2}"#,
        r#"{"max_payload_bytes":8192,"max_payload_byt\u0065s":1}"#,
        r#"{"unknown":true}"#,
        r#"{"enabled":null}"#,
        r#"{"max_maps_per_owner":0}"#,
        r#"{"max_maps_per_owner":1000000}"#,
        r#"{"max_payload_bytes":0}"#,
        r#"{"max_payload_bytes":8193}"#,
        r#"[true,2,8192]"#,
    ] {
        fs::write(&path, invalid).unwrap();
        assert!(
            matches!(CustomMapPolicy::from_path(&path), Err(Error::PolicyJson(_))),
            "{invalid}"
        );
    }
    assert!(matches!(
        CustomMapStore::with_palette(
            scratch.path("maps"),
            CustomMapPolicy {
                max_payload_bytes: 8193,
                ..policy()
            },
            palette()
        ),
        Err(Error::InvalidPolicyLimits)
    ));
    assert!(matches!(
        CustomMapPolicy::from_path(&scratch.0),
        Err(Error::InvalidPath)
    ));
}

#[cfg(unix)]
#[test]
fn rejects_symlinks_in_policy_store_and_record_paths() {
    use std::os::unix::fs::symlink;

    let scratch = Scratch::new();
    let target = scratch.path("target");
    fs::create_dir(&target).unwrap();
    let linked_dir = scratch.path("linked-dir");
    symlink(&target, &linked_dir).unwrap();
    let policy_path = target.join("policy.json");
    fs::write(&policy_path, "{}").unwrap();
    let linked_policy = scratch.path("linked-policy");
    symlink(&policy_path, &linked_policy).unwrap();

    assert!(matches!(
        CustomMapPolicy::from_path(&linked_policy),
        Err(Error::Symlink)
    ));
    assert!(matches!(
        CustomMapPolicy::from_path(linked_dir.join("policy.json")),
        Err(Error::Symlink)
    ));
    for path in [&linked_dir, &linked_dir.join("nonexistent")] {
        assert!(matches!(
            CustomMapStore::with_palette(path, policy(), palette()),
            Err(Error::Symlink)
        ));
    }

    let root = scratch.path("maps");
    fs::create_dir(&root).unwrap();
    let store = store(&root, policy());
    let owner = root.join("owner");
    symlink(&target, &owner).unwrap();
    assert!(matches!(
        store.save("owner", "map", &valid_payload()),
        Err(Error::Symlink)
    ));
    assert!(matches!(store.get("owner", "map"), Err(Error::Symlink)));
    assert!(matches!(store.list_by_owner("owner"), Err(Error::Symlink)));
    assert!(matches!(
        store.quota_remaining("owner"),
        Err(Error::Symlink)
    ));
    assert!(!target.join("map.json").exists());

    fs::remove_file(&owner).unwrap();
    fs::create_dir(&owner).unwrap();
    let record_link = record_path(&root);
    symlink(&policy_path, &record_link).unwrap();
    assert!(matches!(store.get("owner", "map"), Err(Error::Symlink)));
    assert!(matches!(
        store.save("owner", "map", &valid_payload()),
        Err(Error::Symlink)
    ));
    assert!(matches!(store.list_by_owner("owner"), Err(Error::Symlink)));
    assert!(matches!(
        store.quota_remaining("owner"),
        Err(Error::Symlink)
    ));
    assert!(matches!(store.delete("owner", "map"), Err(Error::Symlink)));
    assert_eq!(fs::read(&policy_path).unwrap(), b"{}");
    fs::remove_file(record_link).unwrap();

    fs::rename(&root, scratch.path("old-maps")).unwrap();
    symlink(&target, &root).unwrap();
    assert!(matches!(store.get("owner", "map"), Err(Error::Symlink)));
}

#[test]
fn disabled_policy_rejects_new_helpers_before_validation() {
    let scratch = Scratch::new();
    let root = scratch.path("maps");
    let mut store = store(&root, CustomMapPolicy::default());

    assert!(matches!(
        store.check_hash_value("bad"),
        Err(Error::Disabled)
    ));
    assert!(matches!(store.search_maps("", 0), Err(Error::Disabled)));
    assert!(matches!(
        store.record_good_vote("../bad", 1),
        Err(Error::Disabled)
    ));
    assert!(matches!(
        store.validate_publish_draft("", "", &[]),
        Err(Error::Disabled)
    ));
    assert!(!root.exists());
}

#[test]
fn hash_lookup_requires_a_canonical_digest_and_a_verified_record() {
    let scratch = Scratch::new();
    let root = scratch.path("maps");
    let store = store(&root, policy());
    let miss = "0".repeat(64);
    assert!(!store.check_hash_value(&miss).unwrap());
    assert!(!root.exists());

    let bytes = valid_payload();
    let saved = store.save("owner", "map", &bytes).unwrap();
    assert!(store.check_hash_value(&saved.sha256).unwrap());
    assert!(!store.check_hash_value(&miss).unwrap());
    for invalid in [
        "a".repeat(63),
        "a".repeat(65),
        "g".repeat(64),
        "A".repeat(64),
        "é".repeat(32),
    ] {
        assert!(matches!(
            store.check_hash_value(&invalid),
            Err(Error::InvalidHash)
        ));
    }

    let path = record_path(&root);
    let mut record: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    record["metadata"]["sha256"] = json!(miss);
    fs::write(&path, serde_json::to_vec(&record).unwrap()).unwrap();
    assert!(matches!(
        store.check_hash_value(&miss),
        Err(Error::Integrity)
    ));
}

#[test]
fn search_returns_only_matching_metadata_in_bounded_sorted_order() {
    let scratch = Scratch::new();
    let root = scratch.path("maps");
    let store = store(&root, policy());
    assert!(store.search_maps("", 1).unwrap().is_empty());
    assert!(!root.exists());

    let bytes = valid_payload();
    for i in (0..33).rev() {
        let owner = format!("Maker-{i:02}");
        store.save(&owner, "Beta", &bytes).unwrap();
        store.save(&owner, "Alpha", &bytes).unwrap();
    }

    let first = store.search_maps("", 3).unwrap();
    let keys: Vec<_> = first
        .iter()
        .map(|meta| (meta.owner_id.as_str(), meta.map_id.as_str()))
        .collect();
    assert_eq!(
        keys,
        [
            ("Maker-00", "Alpha"),
            ("Maker-00", "Beta"),
            ("Maker-01", "Alpha")
        ]
    );
    assert_eq!(store.search_maps("MAKER-00", 2).unwrap(), first[..2]);
    assert_eq!(store.search_maps("ALPh", 1).unwrap(), first[..1]);
    assert_eq!(store.search_maps("", usize::MAX).unwrap().len(), 64);
    assert!(store.search_maps("missing", 64).unwrap().is_empty());
    assert!(store.search_maps("", 0).unwrap().is_empty());
    assert!(store.search_maps(&"é".repeat(32), 1).unwrap().is_empty());
    assert!(matches!(
        store.search_maps(&"é".repeat(33), 1),
        Err(Error::QueryTooLong)
    ));
}

#[test]
fn query_budget_counts_ignored_directory_entries() {
    let scratch = Scratch::new();
    let root = scratch.path("maps");
    let store = store(&root, policy());
    let saved = store.save("owner", "map", &valid_payload()).unwrap();
    let owner_dir = root.join("owner");
    for i in 0..4095 {
        fs::write(owner_dir.join(format!(".f7-tmp-{i}")), b"").unwrap();
    }
    assert!(matches!(
        store.search_maps("", 1),
        Err(Error::QueryScanLimitExceeded)
    ));
    assert!(matches!(
        store.check_hash_value(&saved.sha256),
        Err(Error::QueryScanLimitExceeded)
    ));
    assert_eq!(store.list_by_owner("owner").unwrap(), [saved]);
}

#[test]
fn votes_require_an_unambiguous_verified_map_and_deduplicate_accounts() {
    let scratch = Scratch::new();
    let root = scratch.path("maps");
    let mut store = store(&root, policy());
    assert!(matches!(
        store.record_good_vote("map", 7),
        Err(Error::NotFound)
    ));
    assert!(matches!(
        store.record_good_vote("../map", 7),
        Err(Error::InvalidMapId)
    ));
    assert!(!root.exists());

    let mut bytes = valid_payload();
    store.save("owner", "map", &bytes).unwrap();
    let path = record_path(&root);
    let on_disk = fs::read(&path).unwrap();
    assert_eq!(store.record_good_vote("map", 7).unwrap(), 1);
    assert_eq!(store.record_good_vote("map", 7).unwrap(), 1);
    assert_eq!(store.record_good_vote("map", 8).unwrap(), 2);
    assert_eq!(fs::read(&path).unwrap(), on_disk);

    store.save("other", "map", &bytes).unwrap();
    assert!(matches!(
        store.record_good_vote("map", 9),
        Err(Error::AmbiguousMapId)
    ));
    store.delete("other", "map").unwrap();
    assert_eq!(store.record_good_vote("map", 9).unwrap(), 3);

    fs::write(&path, b"{}").unwrap();
    assert!(matches!(
        store.record_good_vote("map", 7),
        Err(Error::Integrity)
    ));
    bytes.push(b'\n');
    store.save("owner", "map", &bytes).unwrap();
    assert_eq!(store.record_good_vote("map", 7).unwrap(), 1);
    store.delete("owner", "map").unwrap();
    assert!(matches!(
        store.record_good_vote("map", 7),
        Err(Error::NotFound)
    ));
}

#[test]
fn vote_capacity_rejects_new_accounts_and_preserves_duplicate_counts() {
    let scratch = Scratch::new();
    let mut store = store(&scratch.path("maps"), policy());
    store.save("owner", "map", &valid_payload()).unwrap();

    for id in 1..=256 {
        assert_eq!(store.record_good_vote("map", id).unwrap(), id);
    }
    assert!(matches!(
        store.record_good_vote("map", 257),
        Err(Error::VoteLimitExceeded)
    ));
    assert_eq!(store.record_good_vote("map", 1).unwrap(), 256);
}

#[test]
fn publish_draft_checks_ids_payload_and_palette_without_saving() {
    let scratch = Scratch::new();
    let root = scratch.path("maps");
    let mut bytes = valid_payload();
    bytes.resize(MAX_CUSTOM_MAP_BYTES, b' ');
    let limited = CustomMapPolicy {
        max_payload_bytes: MAX_CUSTOM_MAP_BYTES - 1,
        ..policy()
    };
    assert!(matches!(
        store(&root, limited).validate_publish_draft("owner", "map", &bytes),
        Err(Error::PayloadTooLarge)
    ));
    let store = store(&root, policy());
    let draft = store
        .validate_publish_draft("owner", "map", &bytes)
        .unwrap();
    assert_eq!(
        draft.wire_contract,
        CustomMapWireContract::UnmeasuredWireContract
    );
    assert!(matches!(
        store.validate_publish_draft(&"é".repeat(33), "map", &bytes),
        Err(Error::InvalidOwnerId)
    ));
    assert!(matches!(
        store.validate_publish_draft("owner", "../map", &bytes),
        Err(Error::InvalidMapId)
    ));
    assert!(matches!(
        store.validate_publish_draft("owner", "map", &[]),
        Err(Error::InvalidMap(CustomMapError::EmptyPayload))
    ));
    assert!(matches!(
        store.validate_publish_draft("owner", "map", &payload(&[2; 15 * 13])),
        Err(Error::InvalidMap(CustomMapError::UnknownTile(2)))
    ));
    assert!(!root.exists());
    assert_eq!(store.save("owner", "map", &bytes).unwrap(), draft);
}
