//! Local storage for normalized F7 maps. A stored map grants no approval,
//! room authority, or KR upload/selection route.
#![deny(unsafe_code)]

use crate::{platform, sha256_hex};
use kr_wire::kr00967::custom_map::{
    CUSTOM_MAP_SCHEMA, CustomMap, CustomMapError, CustomMapWireStatus, MAX_CUSTOM_MAP_BYTES,
};
use serde::de::{MapAccess, Visitor, value::MapAccessDeserializer};
use serde::{Deserialize, Deserializer, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::{self, Write as _},
    fs::{self, DirBuilder, File, Metadata},
    io::{self, Read, Write as _},
    path::{Component, Path, PathBuf},
    sync::{Mutex, MutexGuard},
};

const DEFAULT_MAX_MAPS_PER_OWNER: usize = 16;
const MAX_MAPS_PER_OWNER: usize = 1024;
const MAX_ID_BYTES: usize = 64;
const MAX_POLICY_BYTES: usize = 4096;
// One JSON string can expand to six bytes per input byte when escaped.
const MAX_RECORD_BYTES: usize = 6 * MAX_CUSTOM_MAP_BYTES + 1024;
const SHA256_HEX_BYTES: usize = 64;
const MAX_SEARCH_QUERY_BYTES: usize = 64;
const MAX_SEARCH_RESULTS: usize = 64;
const MAX_QUERY_ENTRIES: usize = 4096;
const MAX_GOOD_VOTE_MAPS: usize = 256;
const MAX_GOOD_VOTERS_PER_MAP: usize = 256;

// Keep the quota check and publication in one critical section, even when
// multiple store handles in this service process point at the same directory.
static STORE_LOCK: Mutex<()> = Mutex::new(());

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct CustomMapPolicy {
    pub enabled: bool,
    pub max_maps_per_owner: usize,
    pub max_payload_bytes: usize,
}

impl Default for CustomMapPolicy {
    fn default() -> Self {
        Self {
            enabled: false,
            max_maps_per_owner: DEFAULT_MAX_MAPS_PER_OWNER,
            max_payload_bytes: MAX_CUSTOM_MAP_BYTES,
        }
    }
}

impl<'de> Deserialize<'de> for CustomMapPolicy {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(default, deny_unknown_fields)]
        struct Fields {
            enabled: bool,
            max_maps_per_owner: usize,
            max_payload_bytes: usize,
        }

        impl Default for Fields {
            fn default() -> Self {
                let policy = CustomMapPolicy::default();
                Self {
                    enabled: policy.enabled,
                    max_maps_per_owner: policy.max_maps_per_owner,
                    max_payload_bytes: policy.max_payload_bytes,
                }
            }
        }

        // Require an object. Deserializing the fields directly would also
        // accept a positional array. Serde rejects repeated named fields.
        struct PolicyObject;
        impl<'de> Visitor<'de> for PolicyObject {
            type Value = CustomMapPolicy;

            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("an F7 custom map policy object")
            }

            fn visit_map<M: MapAccess<'de>>(self, map: M) -> Result<Self::Value, M::Error> {
                let fields = Fields::deserialize(MapAccessDeserializer::new(map))?;
                let policy = CustomMapPolicy {
                    enabled: fields.enabled,
                    max_maps_per_owner: fields.max_maps_per_owner,
                    max_payload_bytes: fields.max_payload_bytes,
                };
                policy.validate().map_err(serde::de::Error::custom)?;
                Ok(policy)
            }
        }

        deserializer.deserialize_map(PolicyObject)
    }
}

impl CustomMapPolicy {
    pub fn from_path(path: impl AsRef<Path>) -> Result<Self, CustomMapStoreError> {
        let path = absolute_path(path.as_ref())?;
        let parent = path.parent().ok_or(CustomMapStoreError::InvalidPath)?;
        if !directory_exists(parent, false)? {
            return Err(CustomMapStoreError::NotFound);
        }
        let bytes = read_regular(&path, MAX_POLICY_BYTES)?.ok_or(CustomMapStoreError::NotFound)?;
        serde_json::from_slice(&bytes).map_err(CustomMapStoreError::PolicyJson)
    }

    fn validate(&self) -> Result<(), CustomMapStoreError> {
        if !(1..=MAX_MAPS_PER_OWNER).contains(&self.max_maps_per_owner)
            || !(1..=MAX_CUSTOM_MAP_BYTES).contains(&self.max_payload_bytes)
        {
            return Err(CustomMapStoreError::InvalidPolicyLimits);
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CustomMapWireContract {
    UnmeasuredWireContract,
}

impl CustomMapWireContract {
    pub const fn status(self) -> CustomMapWireStatus {
        match self {
            Self::UnmeasuredWireContract => CustomMapWireStatus::Unmeasured,
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CustomMapMetadata {
    pub owner_id: String,
    pub map_id: String,
    pub sha256: String,
    pub width: u16,
    pub height: u16,
    pub spawn_count: usize,
    pub schema: String,
    pub wire_contract: CustomMapWireContract,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CustomMapRecord {
    pub metadata: CustomMapMetadata,
    pub payload: Vec<u8>,
}

#[derive(Debug)]
pub enum CustomMapStoreError {
    Disabled,
    InvalidOwnerId,
    InvalidMapId,
    InvalidPath,
    InvalidPolicyLimits,
    PolicyJson(serde_json::Error),
    PayloadTooLarge,
    InvalidHash,
    QueryTooLong,
    QueryScanLimitExceeded,
    AmbiguousMapId,
    VoteLimitExceeded,
    InvalidMap(CustomMapError),
    QuotaExceeded,
    Integrity,
    NotFound,
    Symlink,
    Io(io::Error),
}

impl fmt::Display for CustomMapStoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Disabled => f.write_str("custom map store is disabled"),
            Self::InvalidOwnerId => f.write_str("invalid custom map owner ID"),
            Self::InvalidMapId => f.write_str("invalid custom map ID"),
            Self::InvalidPath => f.write_str("invalid custom map path or filesystem entry type"),
            Self::InvalidPolicyLimits => f.write_str("custom map policy limits are out of bounds"),
            Self::PolicyJson(error) => write!(f, "invalid custom map policy JSON: {error}"),
            Self::PayloadTooLarge => f.write_str("custom map payload exceeds the policy limit"),
            Self::InvalidHash => {
                f.write_str("custom map hash must be 64 lowercase hexadecimal bytes")
            }
            Self::QueryTooLong => f.write_str("custom map search query exceeds the bounded length"),
            Self::QueryScanLimitExceeded => f.write_str("custom map query scan limit exceeded"),
            Self::AmbiguousMapId => f.write_str("custom map ID matches more than one owner"),
            Self::VoteLimitExceeded => f.write_str("custom map good-vote bound exceeded"),
            Self::InvalidMap(error) => write!(f, "invalid custom map: {error}"),
            Self::QuotaExceeded => f.write_str("custom map owner quota exceeded"),
            Self::Integrity => f.write_str("stored custom map failed integrity verification"),
            Self::NotFound => f.write_str("custom map path was not found"),
            Self::Symlink => f.write_str("symlinks are not allowed in custom map paths"),
            Self::Io(error) => error.fmt(f),
        }
    }
}

impl std::error::Error for CustomMapStoreError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::PolicyJson(error) => Some(error),
            Self::InvalidMap(error) => Some(error),
            Self::Io(error) => Some(error),
            _ => None,
        }
    }
}

impl From<io::Error> for CustomMapStoreError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct StoredMap<T> {
    metadata: CustomMapMetadata,
    payload: T,
}

#[derive(Debug)]
struct GoodVotes {
    metadata: CustomMapMetadata,
    voters: BTreeSet<u64>,
}

/// Point this store at a service controlled directory. The service process must
/// be the only writer and untrusted users must not be able to replace path
/// components. Files are checked again on each operation. The persisted digest
/// detects corruption, but is not an external approval pin or signature.
#[derive(Debug)]
pub struct CustomMapStore {
    root: PathBuf,
    policy: CustomMapPolicy,
    trusted_palette: BTreeMap<u16, bool>,
    good_votes: BTreeMap<String, GoodVotes>,
}

impl CustomMapStore {
    /// Open the store with the currently trusted fixture palette.
    pub fn new(
        root: impl AsRef<Path>,
        policy: CustomMapPolicy,
    ) -> Result<Self, CustomMapStoreError> {
        Self::with_palette(root, policy, BTreeMap::from([(0, true), (1, false)]))
    }

    /// Open the store with an explicitly supplied server palette.
    pub fn with_palette(
        root: impl AsRef<Path>,
        policy: CustomMapPolicy,
        trusted_palette: BTreeMap<u16, bool>,
    ) -> Result<Self, CustomMapStoreError> {
        policy.validate()?;
        let root = absolute_path(root.as_ref())?;
        // Check existing components even when disabled, without creating
        // directories until an enabled save needs them.
        directory_exists(&root, false)?;
        Ok(Self {
            root,
            policy,
            trusted_palette,
            good_votes: BTreeMap::new(),
        })
    }

    pub fn policy(&self) -> &CustomMapPolicy {
        &self.policy
    }

    /// Store or replace a normalized map. Replacing an existing ID does not
    /// consume another quota slot.
    pub fn save(
        &self,
        owner_id: &str,
        map_id: &str,
        payload: &[u8],
    ) -> Result<CustomMapMetadata, CustomMapStoreError> {
        self.ensure_enabled()?;
        let owner_dir = self.owner_path(owner_id)?;
        validate_map_id(map_id)?;
        if payload.len() > self.policy.max_payload_bytes {
            return Err(CustomMapStoreError::PayloadTooLarge);
        }
        let map = CustomMap::parse_normalized(payload, &self.trusted_palette)
            .map_err(CustomMapStoreError::InvalidMap)?;
        let body = std::str::from_utf8(payload)
            .map_err(|_| CustomMapStoreError::InvalidMap(CustomMapError::InvalidNormalizedJson))?;
        let metadata = map_metadata(owner_id, map_id, sha256_hex(payload), &map);

        let _lock = lock_store()?;
        directory_exists(&owner_dir, true)?;
        let path = record_path(&owner_dir, map_id);
        let is_replacement = regular_metadata(&path)?.is_some();
        if !is_replacement && self.quota_in(&owner_dir)? == 0 {
            return Err(CustomMapStoreError::QuotaExceeded);
        }

        let bytes = serde_json::to_vec(&StoredMap {
            metadata: metadata.clone(),
            payload: body,
        })
        .map_err(io::Error::other)?;
        atomic_write(&owner_dir, &path, &bytes)?;
        Ok(metadata)
    }

    pub fn get(
        &self,
        owner_id: &str,
        map_id: &str,
    ) -> Result<Option<CustomMapRecord>, CustomMapStoreError> {
        self.ensure_enabled()?;
        let owner_dir = self.owner_path(owner_id)?;
        validate_map_id(map_id)?;
        let _lock = lock_store()?;
        if !directory_exists(&owner_dir, false)? {
            return Ok(None);
        }
        self.load(&owner_dir, owner_id, map_id)
    }

    pub fn list_by_owner(
        &self,
        owner_id: &str,
    ) -> Result<Vec<CustomMapMetadata>, CustomMapStoreError> {
        self.ensure_enabled()?;
        let owner_dir = self.owner_path(owner_id)?;
        let _lock = lock_store()?;
        if !directory_exists(&owner_dir, false)? {
            return Ok(Vec::new());
        }
        let mut records = Vec::new();
        for id in map_ids(&owner_dir)? {
            let record = self
                .load(&owner_dir, owner_id, &id)?
                .ok_or(CustomMapStoreError::Integrity)?;
            records.push(record.metadata);
        }
        Ok(records)
    }

    pub fn delete(&self, owner_id: &str, map_id: &str) -> Result<bool, CustomMapStoreError> {
        self.ensure_enabled()?;
        let owner_dir = self.owner_path(owner_id)?;
        validate_map_id(map_id)?;
        let _lock = lock_store()?;
        if !directory_exists(&owner_dir, false)? {
            return Ok(false);
        }
        let path = record_path(&owner_dir, map_id);
        if regular_metadata(&path)?.is_none() {
            return Ok(false);
        }
        fs::remove_file(path)?;
        platform::sync_directory(&owner_dir)?;
        Ok(true)
    }

    pub fn quota_remaining(&self, owner_id: &str) -> Result<usize, CustomMapStoreError> {
        self.ensure_enabled()?;
        let owner_dir = self.owner_path(owner_id)?;
        let _lock = lock_store()?;
        if !directory_exists(&owner_dir, false)? {
            return Ok(self.policy.max_maps_per_owner);
        }
        self.quota_in(&owner_dir)
    }

    /// Check a stored payload digest without admitting any Type 666 route.
    /// Hash text is deliberately stricter than a generic hex parser so the
    /// lookup has one canonical representation.
    pub fn check_hash_value(&self, sha256_hex: &str) -> Result<bool, CustomMapStoreError> {
        self.ensure_enabled()?;
        if sha256_hex.len() != SHA256_HEX_BYTES
            || !sha256_hex
                .bytes()
                .all(|byte| matches!(byte, b'0'..=b'9' | b'a'..=b'f'))
        {
            return Err(CustomMapStoreError::InvalidHash);
        }
        let _lock = lock_store()?;
        let mut budget = MAX_QUERY_ENTRIES;
        for owner_id in self.query_owner_ids(&mut budget)? {
            let owner_dir = self.root.join(&owner_id);
            for map_id in map_ids_with_budget(&owner_dir, &mut budget)? {
                let record = self
                    .load(&owner_dir, &owner_id, &map_id)?
                    .ok_or(CustomMapStoreError::Integrity)?;
                if record.metadata.sha256 == sha256_hex {
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }

    /// Search verified records by owner or map ID (the local format has no
    /// separate title or author name). Matching ignores ASCII case. An empty
    /// query lists maps in owner then map ID order, like any other search.
    /// Returns metadata only, capped at 64 results.
    pub fn search_maps(
        &self,
        query: &str,
        limit: usize,
    ) -> Result<Vec<CustomMapMetadata>, CustomMapStoreError> {
        self.ensure_enabled()?;
        if query.len() > MAX_SEARCH_QUERY_BYTES {
            return Err(CustomMapStoreError::QueryTooLong);
        }
        let limit = limit.min(MAX_SEARCH_RESULTS);
        if limit == 0 {
            return Ok(Vec::new());
        }
        let query = query.to_ascii_lowercase();
        let _lock = lock_store()?;
        let mut budget = MAX_QUERY_ENTRIES;
        let mut results = Vec::with_capacity(limit);
        for owner_id in self.query_owner_ids(&mut budget)? {
            let owner_dir = self.root.join(&owner_id);
            let owner_matches = owner_id.to_ascii_lowercase().contains(&query);
            for map_id in map_ids_with_budget(&owner_dir, &mut budget)? {
                if !owner_matches && !map_id.to_ascii_lowercase().contains(&query) {
                    continue;
                }
                let record = self
                    .load(&owner_dir, &owner_id, &map_id)?
                    .ok_or(CustomMapStoreError::Integrity)?;
                results.push(record.metadata);
                if results.len() == limit {
                    return Ok(results);
                }
            }
        }
        Ok(results)
    }

    /// Count at most one good vote per account for a map on this store
    /// handle. Votes are tied to the verified owner and payload digest. An
    /// observed change to either resets the count. Votes are not persisted;
    /// the caller supplies the authenticated voter ID.
    pub fn record_good_vote(
        &mut self,
        map_id: &str,
        voter_account_id: u64,
    ) -> Result<u64, CustomMapStoreError> {
        self.ensure_enabled()?;
        validate_map_id(map_id)?;
        let _lock = lock_store()?;
        let mut budget = MAX_QUERY_ENTRIES;
        let mut metadata = None;
        for owner_id in self.query_owner_ids(&mut budget)? {
            let owner_dir = self.root.join(&owner_id);
            if let Some(record) = self.load(&owner_dir, &owner_id, map_id)? {
                if metadata.is_some() {
                    return Err(CustomMapStoreError::AmbiguousMapId);
                }
                metadata = Some(record.metadata);
            }
        }
        let metadata = metadata.ok_or(CustomMapStoreError::NotFound)?;
        if self.good_votes.len() == MAX_GOOD_VOTE_MAPS && !self.good_votes.contains_key(map_id) {
            return Err(CustomMapStoreError::VoteLimitExceeded);
        }
        let votes = self
            .good_votes
            .entry(map_id.to_owned())
            .or_insert_with(|| GoodVotes {
                metadata: metadata.clone(),
                voters: BTreeSet::new(),
            });
        if votes.metadata != metadata {
            votes.metadata = metadata;
            votes.voters.clear();
        }
        if votes.voters.len() == MAX_GOOD_VOTERS_PER_MAP
            && !votes.voters.contains(&voter_account_id)
        {
            return Err(CustomMapStoreError::VoteLimitExceeded);
        }
        votes.voters.insert(voter_account_id);
        Ok(votes.voters.len() as u64)
    }

    /// Validate a publish draft using the same bounded map parser as `save`,
    /// without touching the filesystem or claiming quota.
    pub fn validate_publish_draft(
        &self,
        owner_id: &str,
        map_id: &str,
        payload: &[u8],
    ) -> Result<CustomMapMetadata, CustomMapStoreError> {
        self.ensure_enabled()?;
        if !valid_id(owner_id) {
            return Err(CustomMapStoreError::InvalidOwnerId);
        }
        validate_map_id(map_id)?;
        if payload.len() > self.policy.max_payload_bytes {
            return Err(CustomMapStoreError::PayloadTooLarge);
        }
        let map = CustomMap::parse_normalized(payload, &self.trusted_palette)
            .map_err(CustomMapStoreError::InvalidMap)?;
        std::str::from_utf8(payload)
            .map_err(|_| CustomMapStoreError::InvalidMap(CustomMapError::InvalidNormalizedJson))?;
        Ok(map_metadata(owner_id, map_id, sha256_hex(payload), &map))
    }

    fn quota_in(&self, owner_dir: &Path) -> Result<usize, CustomMapStoreError> {
        Ok(self
            .policy
            .max_maps_per_owner
            .saturating_sub(map_ids(owner_dir)?.len()))
    }

    fn ensure_enabled(&self) -> Result<(), CustomMapStoreError> {
        if self.policy.enabled {
            Ok(())
        } else {
            Err(CustomMapStoreError::Disabled)
        }
    }

    fn query_owner_ids(&self, budget: &mut usize) -> Result<Vec<String>, CustomMapStoreError> {
        if !directory_exists(&self.root, false)? {
            return Ok(Vec::new());
        }
        let mut owners = Vec::new();
        for entry in fs::read_dir(&self.root)? {
            spend_query_entry(budget)?;
            let entry = entry?;
            let file_type = entry.file_type()?;
            if file_type.is_symlink() {
                return Err(CustomMapStoreError::Symlink);
            }
            if !file_type.is_dir() {
                return Err(CustomMapStoreError::InvalidPath);
            }
            let name = entry.file_name();
            let name = name.to_str().ok_or(CustomMapStoreError::Integrity)?;
            if !valid_id(name) {
                return Err(CustomMapStoreError::InvalidOwnerId);
            }
            owners.push(name.to_owned());
        }
        owners.sort_unstable();
        Ok(owners)
    }

    fn owner_path(&self, owner_id: &str) -> Result<PathBuf, CustomMapStoreError> {
        if !valid_id(owner_id) {
            return Err(CustomMapStoreError::InvalidOwnerId);
        }
        Ok(self.root.join(owner_id))
    }

    fn load(
        &self,
        owner_dir: &Path,
        owner_id: &str,
        map_id: &str,
    ) -> Result<Option<CustomMapRecord>, CustomMapStoreError> {
        let path = record_path(owner_dir, map_id);
        let Some(bytes) = read_regular(&path, MAX_RECORD_BYTES)? else {
            return Ok(None);
        };
        let stored: StoredMap<String> =
            serde_json::from_slice(&bytes).map_err(|_| CustomMapStoreError::Integrity)?;
        let payload = stored.payload.into_bytes();
        if payload.len() > self.policy.max_payload_bytes {
            return Err(CustomMapStoreError::Integrity);
        }
        let digest = sha256_hex(&payload);
        if stored.metadata.sha256 != digest {
            return Err(CustomMapStoreError::Integrity);
        }
        let map = CustomMap::parse_normalized(&payload, &self.trusted_palette)
            .map_err(|_| CustomMapStoreError::Integrity)?;
        if stored.metadata != map_metadata(owner_id, map_id, digest, &map) {
            return Err(CustomMapStoreError::Integrity);
        }
        Ok(Some(CustomMapRecord {
            metadata: stored.metadata,
            payload,
        }))
    }
}

fn lock_store() -> Result<MutexGuard<'static, ()>, CustomMapStoreError> {
    STORE_LOCK
        .lock()
        .map_err(|_| io::Error::other("custom map store lock poisoned").into())
}

fn map_metadata(
    owner_id: &str,
    map_id: &str,
    sha256: String,
    map: &CustomMap,
) -> CustomMapMetadata {
    let (width, height) = map.grid().dimensions();
    CustomMapMetadata {
        owner_id: owner_id.to_owned(),
        map_id: map_id.to_owned(),
        sha256,
        width,
        height,
        spawn_count: map.spawns().len(),
        schema: CUSTOM_MAP_SCHEMA.to_owned(),
        wire_contract: CustomMapWireContract::UnmeasuredWireContract,
    }
}

fn valid_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= MAX_ID_BYTES
        && id != "."
        && id != ".."
        && !id.contains('/')
        && !id.contains('\\')
        && !id.chars().any(char::is_control)
}

fn validate_map_id(map_id: &str) -> Result<(), CustomMapStoreError> {
    if valid_id(map_id) {
        Ok(())
    } else {
        Err(CustomMapStoreError::InvalidMapId)
    }
}

fn record_path(owner_dir: &Path, map_id: &str) -> PathBuf {
    owner_dir.join(format!("{map_id}.json"))
}

fn absolute_path(path: &Path) -> Result<PathBuf, CustomMapStoreError> {
    if path.as_os_str().is_empty() || path.components().any(|part| part == Component::ParentDir) {
        return Err(CustomMapStoreError::InvalidPath);
    }
    Ok(std::path::absolute(path)?)
}

// Walk from the filesystem root down. A symlink in any component is rejected
// before inspecting a descendant.
fn directory_exists(path: &Path, create: bool) -> Result<bool, CustomMapStoreError> {
    let mut current = PathBuf::new();
    for part in path.components() {
        current.push(part.as_os_str());
        if matches!(part, Component::Prefix(_)) {
            continue;
        }
        match entry_metadata(&current)? {
            Some(metadata) if metadata.is_dir() => {}
            Some(_) => return Err(CustomMapStoreError::InvalidPath),
            None if !create => return Ok(false),
            None => {
                let mut builder = DirBuilder::new();
                #[cfg(unix)]
                {
                    use std::os::unix::fs::DirBuilderExt;
                    builder.mode(0o700);
                }
                builder.create(&current)?;
                platform::sync_directory(
                    current.parent().ok_or(CustomMapStoreError::InvalidPath)?,
                )?;
            }
        }
    }
    Ok(true)
}

fn entry_metadata(path: &Path) -> Result<Option<Metadata>, CustomMapStoreError> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.is_symlink() => Err(CustomMapStoreError::Symlink),
        Ok(metadata) => Ok(Some(metadata)),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error.into()),
    }
}

fn regular_metadata(path: &Path) -> Result<Option<Metadata>, CustomMapStoreError> {
    match entry_metadata(path)? {
        Some(metadata) if !metadata.is_file() => Err(CustomMapStoreError::InvalidPath),
        other => Ok(other),
    }
}

fn read_regular(path: &Path, limit: usize) -> Result<Option<Vec<u8>>, CustomMapStoreError> {
    let Some(metadata) = regular_metadata(path)? else {
        return Ok(None);
    };
    if metadata.len() > limit as u64 {
        return Err(CustomMapStoreError::Integrity);
    }
    let mut bytes = Vec::new();
    let file = File::open(path)?;
    if !file.metadata()?.is_file() {
        return Err(CustomMapStoreError::InvalidPath);
    }
    file.take(limit as u64 + 1).read_to_end(&mut bytes)?;
    if bytes.len() > limit {
        return Err(CustomMapStoreError::Integrity);
    }
    Ok(Some(bytes))
}

fn map_ids(owner_dir: &Path) -> Result<Vec<String>, CustomMapStoreError> {
    let mut budget = usize::MAX;
    map_ids_with_budget(owner_dir, &mut budget)
}

fn spend_query_entry(budget: &mut usize) -> Result<(), CustomMapStoreError> {
    *budget = budget
        .checked_sub(1)
        .ok_or(CustomMapStoreError::QueryScanLimitExceeded)?;
    Ok(())
}

fn map_ids_with_budget(
    owner_dir: &Path,
    budget: &mut usize,
) -> Result<Vec<String>, CustomMapStoreError> {
    let mut ids = Vec::new();
    for entry in fs::read_dir(owner_dir)? {
        spend_query_entry(budget)?;
        let entry = entry?;
        let file_type = entry.file_type()?;
        if file_type.is_symlink() {
            return Err(CustomMapStoreError::Symlink);
        }
        if !file_type.is_file() {
            return Err(CustomMapStoreError::InvalidPath);
        }
        let name = entry.file_name();
        let name = name.to_str().ok_or(CustomMapStoreError::Integrity)?;
        if let Some(id) = name.strip_suffix(".json") {
            validate_map_id(id)?;
            if ids.len() == MAX_MAPS_PER_OWNER {
                return Err(CustomMapStoreError::Integrity);
            }
            ids.push(id.to_owned());
        }
    }
    ids.sort_unstable();
    Ok(ids)
}

fn atomic_write(owner_dir: &Path, path: &Path, bytes: &[u8]) -> Result<(), CustomMapStoreError> {
    let mut nonce = [0u8; 16];
    getrandom::fill(&mut nonce).map_err(io::Error::other)?;
    let mut name = String::from(".f7-tmp-");
    for byte in nonce {
        write!(&mut name, "{byte:02x}").expect("String formatting");
    }
    let temporary = owner_dir.join(name);
    let mut file = platform::private_options()
        .write(true)
        .create_new(true)
        .open(&temporary)?;
    let write_result = file.write_all(bytes).and_then(|()| file.sync_all());
    drop(file);
    if let Err(error) = write_result.and_then(|()| fs::rename(&temporary, path)) {
        let _ = fs::remove_file(&temporary);
        return Err(error.into());
    }
    platform::sync_directory(owner_dir)?;
    Ok(())
}
