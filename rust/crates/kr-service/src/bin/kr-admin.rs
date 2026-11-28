//! Local trusted-administrator interface. JSON credentials are stdin-only.
#![forbid(unsafe_code)]
use kr_service::{
    accounts::{
        self, Admin, AuditFilter, AuditRow, ChatMuteReason, Error, HistoryFilter, ItemGrant,
        SCHEMA_VERSION, Store, Token,
    },
    choco_chat::{F5PlazaZone, F5SocialPolicy},
    custom_map_store::CustomMapPolicy,
    f3_pve_policy::{
        DispatchRejection, F3PvePolicy, MonsterAttackGuard, PveAuthority, PveModeFamily,
    },
    f4_extended_policy::{
        BoosterTicketPolicy, GiftCouponPolicy, LevelRewardPolicy, LucciVaultPolicy,
        PowerupKitPolicy, RecordEraserPolicy, SecondPasswordPolicy, TitlePolicy, UpgradePolicy,
    },
    f6_policy::{F6EventPolicy, F6QuotaTracker, QuotaRejection},
    f6_runtime::{ClaimError, ClaimOutcome, F6Claim, F6EventRuntime, F6Reward},
    operations::{self, LiveCommand, RoomTarget},
};
use serde::{
    Deserialize, Serialize,
    de::{DeserializeSeed, Deserializer, MapAccess, SeqAccess, Visitor},
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, HashMap},
    ffi::{OsStr, OsString},
    io::{self, Read},
    path::Path,
};
use zeroize::Zeroizing;
struct Secret(Zeroizing<String>);
impl<'de> Deserialize<'de> for Secret {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        String::deserialize(d).map(|s| Self(Zeroizing::new(s)))
    }
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Request {
    actor: String,
    reason: String,
    command: Command,
}
#[derive(Deserialize)]
#[serde(tag = "op", deny_unknown_fields)]
enum Command {
    #[serde(rename = "audit-list")]
    AuditList {
        #[serde(default)]
        filter: AuditFilter,
    },
    #[serde(rename = "list-rooms")]
    ListRooms {},
    #[serde(rename = "close-room")]
    CloseRoom { target: RoomTarget },
    #[serde(rename = "f2-runtime")]
    F2Runtime {
        target: RoomTarget,
        command: operations::F2Command,
    },
    #[serde(rename = "delete")]
    Delete { account_id: i64 },
    #[serde(rename = "recovery-reset")]
    RecoveryReset { account_id: i64 },
    #[serde(rename = "match-history-account")]
    MatchHistoryAccount { account_id: i64, limit: u8 },
    #[serde(rename = "match-history-room")]
    MatchHistoryRoom { room_id: u32, limit: u8 },
    #[serde(rename = "init")]
    Init {},
    #[serde(rename = "begin-instance")]
    BeginInstance {},
    #[serde(rename = "create")]
    Create {
        login: String,
        name: String,
        password: Secret,
    },
    #[serde(rename = "lookup")]
    Lookup { login: String },
    #[serde(rename = "login")]
    Login {
        login: String,
        password: Secret,
        ttl_seconds: i64,
    },
    #[serde(rename = "authorize")]
    Authorize { token: Secret, player_id: i64 },
    #[serde(rename = "revoke", alias = "kick")]
    Revoke { account_id: i64 },
    #[serde(rename = "disable")]
    Disable { account_id: i64 },
    #[serde(rename = "enable")]
    Enable { account_id: i64 },
    #[serde(rename = "rename")]
    Rename {
        account_id: i64,
        name: String,
        expected_version: i64,
    },
    #[serde(rename = "password-reset")]
    Reset { account_id: i64, password: Secret },
    #[serde(rename = "profile-get")]
    ProfileGet { player_id: i64 },
    #[serde(rename = "ledger-list")]
    LedgerList { player_id: i64, limit: u8 },
    #[serde(rename = "inventory-list")]
    InventoryList { player_id: i64 },
    #[serde(rename = "admin-grant-lucci")]
    AdminGrantLucci {
        player_id: i64,
        /// Positive grant or negative correction; never below a zero balance.
        lucci_delta: i64,
        idempotency_key: String,
    },
    #[serde(rename = "admin-grant-item")]
    AdminGrantItem {
        player_id: i64,
        sku_id: String,
        quantity: u32,
        grant: ItemGrant,
        idempotency_key: String,
    },
    #[serde(rename = "social-friend-list")]
    SocialFriendList { player_id: i64 },
    #[serde(rename = "social-block-list")]
    SocialBlockList { player_id: i64 },
    #[serde(rename = "social-mute-set")]
    SocialMuteSet {
        player_id: i64,
        duration_seconds: u32,
        mute_reason: ChatMuteReason,
    },
    #[serde(rename = "social-mute-clear")]
    SocialMuteClear { player_id: i64 },
    #[serde(rename = "social-abuse-list")]
    SocialAbuseList {
        sender_id: Option<i64>,
        before_id: Option<i64>,
        limit: u8,
    },
    #[serde(rename = "social-guild-create")]
    SocialGuildCreate { leader_player_id: i64, name: String },
    #[serde(rename = "social-guild-get")]
    SocialGuildGet { guild_id: i64 },
    #[serde(rename = "social-guild-member-add")]
    SocialGuildMemberAdd { guild_id: i64, player_id: i64 },
    #[serde(rename = "social-guild-member-remove")]
    SocialGuildMemberRemove { guild_id: i64, player_id: i64 },
    #[serde(rename = "social-guild-disband")]
    SocialGuildDisband { guild_id: i64 },
    #[serde(rename = "social-ranking-list")]
    SocialRankingList { offset: u32, limit: u32 },
    #[serde(rename = "f6-policy-check")]
    F6PolicyCheck {
        policy_path: Option<std::path::PathBuf>,
        at_unix_secs: Option<i64>,
    },
    #[serde(rename = "f4-policy-check")]
    F4PolicyCheck {
        policy_path: Option<std::path::PathBuf>,
    },
    #[serde(rename = "f3-policy-check")]
    F3PolicyCheck {
        policy_path: Option<std::path::PathBuf>,
    },
    #[serde(rename = "f5-policy-check")]
    F5PolicyCheck {
        policy_path: Option<std::path::PathBuf>,
    },
}
impl Command {
    fn audit_context(&self) -> (&'static str, &'static str, String, bool) {
        use Command::*;
        let account = |op, id: &i64, recorded| (op, "account", id.to_string(), recorded);
        let player = |op, id: &i64, recorded| (op, "player", id.to_string(), recorded);
        let guild = |op, id: &i64, recorded| (op, "guild", id.to_string(), recorded);
        let login = |op, value: &str, recorded| {
            let id = if (3..=32).contains(&value.len())
                && value
                    .bytes()
                    .all(|c| c.is_ascii_alphanumeric() || c == b'_')
            {
                value.to_ascii_lowercase()
            } else {
                "invalid-login".into()
            };
            (op, "login", id, recorded)
        };
        match self {
            MatchHistoryAccount { account_id, .. } => {
                account("match_history_account", account_id, false)
            }
            MatchHistoryRoom { room_id, .. } => {
                ("match_history_room", "room", room_id.to_string(), false)
            }
            Init {} => ("init", "service", "database".into(), false),
            BeginInstance {} => ("begin_instance", "service", "instance".into(), true),
            Create { login: value, .. } => login("create_account", value, true),
            Lookup { login: value } => login("lookup", value, false),
            Login { login: value, .. } => login("login", value, false),
            Authorize { player_id, .. } => ("authorize", "player", player_id.to_string(), false),
            Revoke { account_id } => account("revoke", account_id, true),
            Disable { account_id } => account("disable", account_id, true),
            Enable { account_id } => account("enable", account_id, true),
            Rename { account_id, .. } => account("rename_player", account_id, true),
            Reset { account_id, .. } => account("reset_password", account_id, true),
            RecoveryReset { account_id } => account("reset_recovery", account_id, true),
            Delete { account_id } => account("delete_account", account_id, true),
            AuditList { .. } => ("audit_list", "service", "audit".into(), false),
            ProfileGet { player_id } => player("profile_get", player_id, false),
            LedgerList { player_id, .. } => player("ledger_list", player_id, false),
            InventoryList { player_id } => player("inventory_list", player_id, false),
            AdminGrantLucci { player_id, .. } => player("admin_grant_lucci", player_id, true),
            AdminGrantItem { player_id, .. } => player("admin_grant_item", player_id, true),
            SocialFriendList { player_id } => player("social_friend_list", player_id, false),
            SocialBlockList { player_id } => player("social_block_list", player_id, false),
            SocialMuteSet { player_id, .. } => player("social_mute_set", player_id, true),
            SocialMuteClear { player_id } => player("social_mute_clear", player_id, true),
            SocialAbuseList {
                sender_id: Some(id),
                ..
            } => player("social_abuse_list", id, false),
            SocialAbuseList {
                sender_id: None, ..
            } => ("social_abuse_list", "service", "social_abuse".into(), false),
            SocialGuildCreate {
                leader_player_id, ..
            } => player("social_guild_create", leader_player_id, true),
            SocialGuildGet { guild_id } => guild("social_guild_get", guild_id, false),
            SocialGuildMemberAdd { guild_id, .. } => {
                guild("social_guild_member_add", guild_id, true)
            }
            SocialGuildMemberRemove { guild_id, .. } => {
                guild("social_guild_member_remove", guild_id, true)
            }
            SocialGuildDisband { guild_id } => guild("social_guild_disband", guild_id, true),
            SocialRankingList { .. } => ("social_ranking_list", "service", "ranking".into(), false),
            F6PolicyCheck { .. } => ("f6_policy_check", "service", "f6_policy".into(), false),
            F4PolicyCheck { .. } => ("f4_policy_check", "service", "f4_policy".into(), false),
            F3PolicyCheck { .. } => ("f3_policy_check", "service", "f3_policy".into(), false),
            F5PolicyCheck { .. } => ("f5_policy_check", "service", "f5_policy".into(), false),
            ListRooms {} | CloseRoom { .. } | F2Runtime { .. } => {
                unreachable!("handled by live admin")
            }
        }
    }
}

fn validate_policy_path(path: &Path, policy_name: &str) -> io::Result<()> {
    if path.as_os_str().is_empty()
        || path
            .components()
            .any(|part| matches!(part, std::path::Component::ParentDir))
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("invalid {policy_name} path"),
        ));
    }
    for parent in path
        .ancestors()
        .skip(1)
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        if !std::fs::symlink_metadata(parent)?.is_dir() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("{policy_name} parents must be directories without symlinks"),
            ));
        }
    }
    if !std::fs::symlink_metadata(path)?.is_file() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("{policy_name} must be a regular file without symlinks"),
        ));
    }
    Ok(())
}

#[derive(Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct F4ExtendedPolicy {
    powerup_kits: PowerupKitPolicy,
    booster_tickets: BoosterTicketPolicy,
    titles: TitlePolicy,
    level_rewards: LevelRewardPolicy,
    lucci_vault: LucciVaultPolicy,
    record_erasers: RecordEraserPolicy,
    gift_coupons: GiftCouponPolicy,
    second_password: SecondPasswordPolicy,
    upgrades: UpgradePolicy,
}

struct DuplicateSeed;
struct DuplicateVisitor;

impl<'de> DeserializeSeed<'de> for DuplicateSeed {
    type Value = ();

    fn deserialize<D: serde::Deserializer<'de>>(self, deserializer: D) -> Result<(), D::Error> {
        deserializer.deserialize_any(DuplicateVisitor)
    }
}

impl<'de> Visitor<'de> for DuplicateVisitor {
    type Value = ();

    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("a JSON value")
    }

    fn visit_map<M: MapAccess<'de>>(self, mut map: M) -> Result<(), M::Error> {
        let mut keys = std::collections::BTreeSet::new();
        while let Some(key) = map.next_key::<String>()? {
            if !keys.insert(key) {
                return Err(serde::de::Error::custom("duplicate JSON field"));
            }
            map.next_value_seed(DuplicateSeed)?;
        }
        Ok(())
    }

    fn visit_seq<S: SeqAccess<'de>>(self, mut sequence: S) -> Result<(), S::Error> {
        while sequence.next_element_seed(DuplicateSeed)?.is_some() {}
        Ok(())
    }

    fn visit_bool<E>(self, _: bool) -> Result<(), E> {
        Ok(())
    }
    fn visit_i64<E>(self, _: i64) -> Result<(), E> {
        Ok(())
    }
    fn visit_u64<E>(self, _: u64) -> Result<(), E> {
        Ok(())
    }
    fn visit_f64<E>(self, _: f64) -> Result<(), E> {
        Ok(())
    }
    fn visit_str<E>(self, _: &str) -> Result<(), E> {
        Ok(())
    }
    fn visit_string<E>(self, _: String) -> Result<(), E> {
        Ok(())
    }
    fn visit_none<E>(self) -> Result<(), E> {
        Ok(())
    }
    fn visit_unit<E>(self) -> Result<(), E> {
        Ok(())
    }
}

fn reject_duplicate_json(input: &str) -> Result<(), serde_json::Error> {
    let mut deserializer = serde_json::Deserializer::from_str(input);
    deserializer.deserialize_any(DuplicateVisitor)?;
    deserializer.end()
}

fn load_f4_policy(path: Option<&Path>) -> io::Result<(F4ExtendedPolicy, Value)> {
    let Some(path) = path else {
        return Ok((F4ExtendedPolicy::default(), json!({})));
    };
    validate_policy_path(path, "f4_extended_policy")?;
    let contents = std::fs::read_to_string(path)?;
    let invalid = || {
        io::Error::new(
            io::ErrorKind::InvalidData,
            "invalid f4_extended_policy configuration",
        )
    };
    if !contents.trim_start().starts_with('{') {
        return Err(invalid());
    }
    reject_duplicate_json(&contents).map_err(|_| invalid())?;
    // Deserialize the typed profile first so duplicate and unknown fields are
    // rejected before the summary view is decoded.
    let policy: F4ExtendedPolicy = serde_json::from_str(&contents).map_err(|_| invalid())?;
    let value: Value = serde_json::from_str(&contents).map_err(|_| invalid())?;
    Ok((policy, value))
}

fn f4_policy_check(policy_path: Option<&Path>) -> io::Result<Value> {
    let (_policy, value) = load_f4_policy(policy_path)?;
    let policies = [
        "powerup_kits",
        "booster_tickets",
        "titles",
        "level_rewards",
        "lucci_vault",
        "record_erasers",
        "gift_coupons",
        "second_password",
        "upgrades",
    ]
    .into_iter()
    // These policies are not time-gated; active here describes the config.
    .map(|name| {
        let enabled = value
            .get(name)
            .and_then(|policy| policy.get("enabled"))
            .and_then(Value::as_bool)
            .unwrap_or(false);
        (name.into(), json!({"enabled": enabled, "active": enabled}))
    })
    .collect::<serde_json::Map<_, _>>();
    Ok(json!({"default_enabled": false, "policies": policies}))
}

fn load_f3_policy(policy_path: Option<&Path>) -> io::Result<F3PvePolicy> {
    match policy_path {
        None => Ok(F3PvePolicy::default()),
        Some(path) => {
            validate_policy_path(path, "f3_pve_policy")?;
            let contents = std::fs::read_to_string(path)?;
            serde_json::from_str(&contents).map_err(|_| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    "invalid f3_pve_policy configuration",
                )
            })
        }
    }
}

fn f3_policy_check(policy_path: Option<&Path>) -> io::Result<Value> {
    let policy = load_f3_policy(policy_path)?;
    let monster_s1_dispatch = match policy.ensure_dispatch_allowed(PveModeFamily::MonsterS1) {
        Ok(()) => "allowed",
        Err(DispatchRejection::Disabled) => "disabled",
        Err(DispatchRejection::WineEvidenceRequired(_)) => "wine_evidence_required",
    };
    let monster_attack_authority = match MonsterAttackGuard::AUTHORITY {
        PveAuthority::HostCandidate => "host_candidate",
        PveAuthority::ServerAuthoritative => "server_authoritative",
        PveAuthority::Unmeasured => "unmeasured",
    };
    Ok(json!({
        "default_enabled": F3PvePolicy::default().enabled,
        "enabled": policy.enabled,
        "authority_default": "unmeasured",
        "dispatch": {"monster_s1": monster_s1_dispatch},
        "monster_attack_guard": {
            "authority": monster_attack_authority,
            "requires_current_host": true,
            "requires_current_round_epoch": true,
            "requires_increasing_attack_sequence": true,
        },
    }))
}

fn load_f6_policy(path: Option<&Path>) -> io::Result<F6EventPolicy> {
    let Some(path) = path else {
        return Ok(F6EventPolicy::disabled_default());
    };
    validate_policy_path(path, "f6_events_policy")?;
    F6EventPolicy::from_path(path)
}

fn f6_policy_check(policy_path: Option<&Path>, at_unix_secs: Option<i64>) -> io::Result<Value> {
    let policy = load_f6_policy(policy_path)?;
    let evaluated_at_unix_secs = match at_unix_secs {
        Some(at) => at,
        None => accounts::utc().map_err(|error| io::Error::other(error.to_string()))?,
    };
    let systems = F6EventPolicy::system_names()
        .iter()
        .map(|name| {
            json!({
                "name": name,
                "enabled": policy.is_system_enabled_at(name, evaluated_at_unix_secs),
            })
        })
        .collect::<Vec<_>>();
    let extended_systems = F6EventPolicy::extended_system_names()
        .iter()
        .map(|name| {
            json!({
                "name": name,
                "enabled": policy.is_system_enabled_at(name, evaluated_at_unix_secs),
            })
        })
        .collect::<Vec<_>>();
    let enabled_systems = policy
        .enabled_systems()
        .map(|(name, system)| {
            json!({
                "name": name,
                "start_unix_secs": system.start_utc,
                "end_unix_secs": system.end_utc,
                "active": evaluated_at_unix_secs >= 0
                    && policy.is_system_enabled_at(name, evaluated_at_unix_secs),
                "rate_caps": system.rate_caps,
            })
        })
        .collect::<Vec<_>>();
    Ok(json!({
        "schema": "kre.f6-events-policy.v1",
        "default_enabled": false,
        "excluded_systems": ["PCRoomBenefit"],
        "evaluated_at_unix_secs": evaluated_at_unix_secs,
        "systems": systems,
        "enabled_systems": enabled_systems,
        "extended_systems": extended_systems,
    }))
}

fn load_f7_policy(
    policy_path: Option<&Path>,
) -> Result<CustomMapPolicy, Box<dyn std::error::Error>> {
    Ok(match policy_path {
        None => CustomMapPolicy::default(),
        Some(path) => {
            validate_policy_path(path, "f7_custom_map_policy")?;
            CustomMapPolicy::from_path(path)?
        }
    })
}

fn f7_policy_check(policy_path: Option<&Path>) -> Result<Value, Box<dyn std::error::Error>> {
    let policy = load_f7_policy(policy_path)?;
    Ok(json!({
        "default_enabled": false,
        "enabled": policy.enabled,
        "max_maps_per_owner": policy.max_maps_per_owner,
        "max_payload_bytes": policy.max_payload_bytes,
        "wire_contract": "unmeasured",
    }))
}

fn load_f5_policy(policy_path: Option<&Path>) -> io::Result<F5SocialPolicy> {
    match policy_path {
        None => Ok(F5SocialPolicy::default()),
        Some(path) => {
            validate_policy_path(path, "f5_social_policy")?;
            let contents = std::fs::read_to_string(path)?;
            if !contents.trim_start().starts_with('{') || reject_duplicate_json(&contents).is_err()
            {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "invalid f5_social_policy configuration",
                ));
            }
            F5SocialPolicy::load_from_path(path).map_err(|error| {
                io::Error::new(
                    error.kind(),
                    format!("invalid f5_social_policy configuration: {error}"),
                )
            })
        }
    }
}

fn f5_all_disabled(policy: &F5SocialPolicy) -> bool {
    !policy.whisper
        && !policy.guild_chat
        && policy.megaphone.is_none()
        && policy.note_inbox_cap.is_none()
        && !policy.couple
        && policy.plaza_zones.is_empty()
}

fn f5_policy_check(policy_path: Option<&Path>) -> io::Result<Value> {
    let policy = load_f5_policy(policy_path)?;
    let plaza_names = policy
        .plaza_zones
        .keys()
        .map(|zone| match zone {
            F5PlazaZone::LobbySquare => "lobby_square",
            F5PlazaZone::LobbyCaffe => "lobby_caffe",
            F5PlazaZone::LobbyClothing => "lobby_clothing",
            F5PlazaZone::LobbyJewelry => "lobby_jewelry",
            F5PlazaZone::LobbyPet => "lobby_pet",
            F5PlazaZone::Square => "square",
        })
        .map(str::to_owned)
        .collect::<Vec<_>>();
    Ok(json!({
        "whisper": policy.whisper,
        "guild_chat": policy.guild_chat,
        "megaphone": policy.megaphone.is_some(),
        "note_inbox_cap": policy.note_inbox_cap.map(|cap| cap.get()),
        "couple": policy.couple,
        "plaza_zones": {
            "count": plaza_names.len(),
            "names": plaza_names,
        },
        "all_disabled": f5_all_disabled(&policy),
    }))
}

const POLICY_AUDIT_USAGE: &str = "usage: kr-admin policy-audit [--f3-policy PATH] [--f4-policy PATH] [--f5-policy PATH] [--f6-policy PATH] [--f7-policy PATH]";
const F6_CLAIM_USAGE: &str = "usage: kr-admin f6-claim-event --db PATH --f6-policy PATH [--account LOGIN] [--player-id ID] --event-id ID --now-unix SECS --operator NAME --reason TEXT [--claim-id ID] (--lucci AMOUNT | --sku-id SKU [--quantity N] [--max-stack N | --duration-seconds SECS])";

type CliResult<T> = Result<T, Box<dyn std::error::Error>>;

struct Options<'a>(BTreeMap<&'a str, &'a OsStr>);

impl<'a> Options<'a> {
    fn parse(args: &'a [OsString], allowed: &[&str], usage: &'static str) -> CliResult<Self> {
        let mut options = BTreeMap::new();
        for pair in args.chunks(2) {
            let [flag, value] = pair else {
                return Err(usage.into());
            };
            let flag = flag.to_str().ok_or(usage)?;
            if !allowed.contains(&flag) || options.insert(flag, value.as_os_str()).is_some() {
                return Err(usage.into());
            }
        }
        Ok(Self(options))
    }

    fn path(&'a self, flag: &str) -> Option<&'a Path> {
        self.0.get(flag).map(Path::new)
    }

    fn value<T: std::str::FromStr>(&self, flag: &str) -> CliResult<Option<T>> {
        self.0
            .get(flag)
            .map(|value| {
                value
                    .to_str()
                    .and_then(|value| value.parse().ok())
                    .ok_or_else(|| format!("invalid {flag}").into())
            })
            .transpose()
    }

    fn text(&self, flag: &str) -> CliResult<Option<&'a str>> {
        self.0
            .get(flag)
            .map(|value| {
                value
                    .to_str()
                    .ok_or_else(|| format!("invalid {flag}").into())
            })
            .transpose()
    }
}

fn policy_audit(args: &[OsString]) -> CliResult<Value> {
    let flags = [
        "--f3-policy",
        "--f4-policy",
        "--f5-policy",
        "--f6-policy",
        "--f7-policy",
    ];
    let options = Options::parse(args, &flags, POLICY_AUDIT_USAGE)?;
    let [f3, f4, f5, f6, f7] = flags.map(|flag| options.path(flag));
    let f3 = load_f3_policy(f3)?;
    let (f4, _) = load_f4_policy(f4)?;
    let f5 = load_f5_policy(f5)?;
    let f6 = load_f6_policy(f6)?;
    let f7 = load_f7_policy(f7)?;
    Ok(json!({
        "F3": {"enabled": f3.enabled},
        "F4": {"enabled": f4.powerup_kits.enabled || f4.booster_tickets.enabled
            || f4.titles.enabled || f4.level_rewards.enabled || f4.lucci_vault.enabled
            || f4.record_erasers.enabled || f4.gift_coupons.enabled
            || f4.second_password.enabled || f4.upgrades.enabled},
        "F5": {"enabled": !f5_all_disabled(&f5)},
        // Report opt-ins, even when their scheduled windows have not started.
        "F6": {"enabled": f6.enabled_systems().next().is_some()
            || f6.enabled_extended_systems().next().is_some()},
        "F7": {"enabled": f7.enabled},
    }))
}

const F6_RUNTIME_ACTOR: &str = "f6-event-runtime";
const F6_CLAIM_OP: &str = "f6_claim_event";
const F6_CLAIM_TARGET: &str = "f6_event_claim";

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct F6ClaimAudit {
    digest: String,
    event_id: String,
    season_key: Option<String>,
}

fn audit_rows(store: &Store, filter: impl Fn() -> AuditFilter) -> Result<Vec<AuditRow>, Error> {
    let mut before_id = None;
    let mut rows = Vec::new();
    loop {
        let page = store.operator_audit(AuditFilter {
            before_id,
            limit: Some(100),
            ..filter()
        })?;
        let Some(last) = page.last() else {
            return Ok(rows);
        };
        before_id = Some(last.id);
        rows.extend(page);
    }
}

fn f6_policy_rejection(rejection: QuotaRejection) -> Box<dyn std::error::Error> {
    format!("F6 claim rejected: {rejection:?}").into()
}

/// The runtime persists claim fingerprints but holds quotas in memory. For
/// this one-shot CLI, pair each committed grant with its operator audit so
/// subsequent invocations can count grants by event, player and season.
fn check_f6_claim_history(
    store: &Store,
    policy: &F6EventPolicy,
    claim: &F6ClaimAudit,
    player_id: i64,
    now: i64,
) -> CliResult<()> {
    let incomplete = || "incomplete F6 claim audit history";
    let mut metadata = HashMap::new();
    for row in audit_rows(store, || AuditFilter {
        operation: Some(F6_CLAIM_OP.into()),
        target_kind: Some(F6_CLAIM_TARGET.into()),
        result: Some("ok".into()),
        ..AuditFilter::default()
    })? {
        let record: F6ClaimAudit =
            serde_json::from_str(row.target_id.as_deref().ok_or_else(incomplete)?)
                .map_err(|_| incomplete())?;
        if metadata.insert(record.digest.clone(), record).is_some() {
            return Err(incomplete().into());
        }
    }
    let (mut hourly, mut minute, mut daily, mut seasonal) = (0u64, 0u64, 0u64, 0u64);
    for row in audit_rows(store, || AuditFilter {
        actor: Some(F6_RUNTIME_ACTOR.into()),
        ..AuditFilter::default()
    })? {
        let (digest, _) = row
            .reason
            .strip_prefix("f6:v1:")
            .and_then(|reason| reason.split_once(':'))
            .ok_or_else(incomplete)?;
        let previous = metadata.remove(digest).ok_or_else(incomplete)?;
        if digest == claim.digest {
            return Err("duplicate F6 claim".into());
        }
        if previous.event_id != claim.event_id {
            continue;
        }
        let target = row
            .target_id
            .as_deref()
            .and_then(|id| id.parse::<i64>().ok())
            .filter(|_| row.target_kind.as_deref() == Some("player"))
            .ok_or_else(incomplete)?;
        if row.at_utc > now {
            return Err(f6_policy_rejection(QuotaRejection::ClockRegression));
        }
        minute += u64::from(row.at_utc / 60 == now / 60);
        if target == player_id {
            hourly += u64::from(row.at_utc / 3_600 == now / 3_600);
            daily += u64::from(row.at_utc / 86_400 == now / 86_400);
            seasonal += u64::from(previous.season_key == claim.season_key);
        }
    }
    if !metadata.is_empty() {
        return Err(incomplete().into());
    }

    let extended = policy
        .enabled_extended_systems()
        .find(|(name, _)| *name == claim.event_id);
    let rates = match extended {
        Some((_, system)) => system.rate_caps,
        None => {
            policy
                .enabled_systems()
                .find(|(name, _)| *name == claim.event_id)
                .ok_or(Error::InvalidInput)?
                .1
                .rate_caps
        }
    };
    if hourly >= u64::from(rates.per_account_per_hour) {
        return Err(f6_policy_rejection(QuotaRejection::AccountQuotaExceeded));
    }
    if minute >= u64::from(rates.global_per_minute) {
        return Err(f6_policy_rejection(QuotaRejection::GlobalQuotaExceeded));
    }
    if let Some((_, system)) = extended {
        if daily >= u64::from(system.max_grants_per_day) {
            return Err(f6_policy_rejection(
                QuotaRejection::ExtendedDailyQuotaExceeded,
            ));
        }
        if seasonal >= u64::from(system.max_grants_per_season) {
            return Err(f6_policy_rejection(
                QuotaRejection::ExtendedSeasonQuotaExceeded,
            ));
        }
    }
    Ok(())
}

fn f6_reward<'a>(options: &Options<'a>) -> CliResult<F6Reward<'a>> {
    let lucci = options.value("--lucci")?;
    let sku_id = options.text("--sku-id")?;
    let quantity = options.value("--quantity")?;
    let max_stack = options.value("--max-stack")?;
    let duration_seconds = options.value("--duration-seconds")?;
    match (lucci, sku_id, quantity, max_stack, duration_seconds) {
        (Some(amount), None, None, None, None) => Ok(F6Reward::Lucci { amount }),
        (None, Some(sku_id), quantity, max_stack, duration_seconds) => Ok(F6Reward::Item {
            sku_id,
            quantity: quantity.unwrap_or(1),
            grant: match (max_stack, duration_seconds) {
                (None, None) => ItemGrant::Permanent,
                (Some(max_stack), None) => ItemGrant::Stackable { max_stack },
                (None, Some(duration_seconds)) => ItemGrant::Timed { duration_seconds },
                _ => return Err(F6_CLAIM_USAGE.into()),
            },
        }),
        _ => Err(F6_CLAIM_USAGE.into()),
    }
}

fn f6_claim_event(args: &[OsString]) -> CliResult<Value> {
    let options = Options::parse(
        args,
        &[
            "--db",
            "--f6-policy",
            "--account",
            "--player-id",
            "--event-id",
            "--now-unix",
            "--operator",
            "--reason",
            "--claim-id",
            "--lucci",
            "--sku-id",
            "--quantity",
            "--max-stack",
            "--duration-seconds",
        ],
        F6_CLAIM_USAGE,
    )?;
    let db = options.path("--db").ok_or(F6_CLAIM_USAGE)?;
    let policy_path = options.path("--f6-policy").ok_or(F6_CLAIM_USAGE)?;
    let event_id = options.text("--event-id")?.ok_or(F6_CLAIM_USAGE)?;
    let now = options.value("--now-unix")?.ok_or(F6_CLAIM_USAGE)?;
    let admin = Admin {
        actor: options.text("--operator")?.ok_or(F6_CLAIM_USAGE)?,
        reason: options.text("--reason")?.ok_or(F6_CLAIM_USAGE)?,
    };
    admin.validate()?;
    if admin.actor == F6_RUNTIME_ACTOR {
        return Err("reserved F6 runtime operator".into());
    }
    let account = options.text("--account")?;
    let player_id = options.value("--player-id")?;
    if account.is_none() && player_id.is_none() {
        return Err(F6_CLAIM_USAGE.into());
    }
    let claim_id = options.text("--claim-id")?;
    if claim_id.is_some_and(|id| !(1..=128).contains(&id.len()) || id.chars().any(char::is_control))
    {
        return Err("invalid --claim-id".into());
    }
    let reward = f6_reward(&options)?;

    let policy = load_f6_policy(Some(policy_path))?;
    // Check the policy before touching the database; the player is resolved
    // under the instance lock and the runtime checks the real ID on grant.
    policy
        .check_operation(&mut F6QuotaTracker::default(), event_id, 1, now)
        .map_err(f6_policy_rejection)?;
    let _instance = operations::lock_database(db)?;
    let mut store = Store::open_existing(db)?;
    let account_player_id = account
        .map(|login| {
            store
                .lookup(login)
                .map(|player| player.player_id)
                .map_err(|error| format!("F6 account lookup failed: {error}"))
        })
        .transpose()?;
    let player_id = match (player_id, account_player_id) {
        (Some(pid), Some(apid)) if pid != apid => {
            return Err("--account and --player-id identify different players".into());
        }
        (Some(pid), _) | (None, Some(pid)) => pid,
        (None, None) => return Err(F6_CLAIM_USAGE.into()),
    };
    store
        .get_profile(player_id)
        .map_err(|error| format!("F6 player lookup failed: {error}"))?;

    // Scope an operator-supplied occurrence ID to the event and player.
    // Without one, each player can claim that event only once.
    let identity =
        serde_json::to_vec(&("kr-admin/f6-claim-event/v1", event_id, player_id, claim_id))?;
    let key = Sha256::digest(&identity)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    let audit = F6ClaimAudit {
        digest: Sha256::digest(key.as_bytes())
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect(),
        event_id: event_id.into(),
        season_key: policy
            .enabled_extended_systems()
            .find(|(name, _)| *name == event_id)
            .map(|(_, system)| system.season_key.clone()),
    };
    check_f6_claim_history(&store, &policy, &audit, player_id, now)?;
    let target_id = serde_json::to_string(&audit)?;
    let mut runtime = F6EventRuntime::new(&store, policy)?;
    match runtime.claim(
        F6Claim {
            key: &key,
            system: event_id,
            player_id,
            reward,
        },
        now,
    ) {
        Ok(ClaimOutcome::Granted) => {}
        Ok(ClaimOutcome::Replayed) => return Err("duplicate F6 claim".into()),
        Err(ClaimError::Policy(rejection)) => return Err(f6_policy_rejection(rejection)),
        Err(ClaimError::Accounts(error)) => return Err(error.into()),
        Err(error) => return Err(format!("F6 claim rejected: {error:?}").into()),
    }

    // The grant and runtime audit have already committed atomically. If
    // attribution fails or the process dies here, future claims fail closed
    // on the missing metadata instead of resetting quotas or repeating grants.
    let operator_audit_id = store
        .operator_event(&admin, F6_CLAIM_OP, (F6_CLAIM_TARGET, &target_id), "ok")
        .map_err(|error| format!("F6 grant committed; operator audit failed: {error}"))?;
    Ok(json!({
        "granted": true,
        "event_id": event_id,
        "player_id": player_id,
        "claimed_at_unix": now,
        "reward": reward,
        "operator_audit_request_id": operator_audit_id,
    }))
}

fn manage(database: &Path) -> Result<Value, Box<dyn std::error::Error>> {
    let mut input = Zeroizing::new(Vec::new());
    io::stdin()
        .lock()
        .take(4097)
        .read_to_end(&mut input)
        .map_err(|_| Error::InvalidInput)?;
    if input.len() > 4096 {
        return Err(Error::InvalidInput.into());
    }
    let r: Request = serde_json::from_slice(&input).map_err(|_| Error::InvalidInput)?;
    let admin = Admin {
        actor: &r.actor,
        reason: &r.reason,
    };
    admin.validate()?;
    let cmd = match r.command {
        Command::ListRooms {} => {
            return Ok(operations::live_command(
                database,
                &admin,
                LiveCommand::ListRooms,
            )?);
        }
        Command::CloseRoom { target } => {
            return Ok(operations::live_command(
                database,
                &admin,
                LiveCommand::CloseRoom { target },
            )?);
        }
        Command::F2Runtime { target, command } => {
            return Ok(operations::live_command(
                database,
                &admin,
                LiveCommand::F2Runtime { target, command },
            )?);
        }
        command => command,
    };
    let (op, kind, id, recorded) = cmd.audit_context();
    let _instance = matches!(cmd, Command::Init {} | Command::BeginInstance {})
        .then(|| operations::lock_database(database))
        .transpose()?;
    let mut s = if matches!(cmd, Command::Init {}) {
        match std::fs::symlink_metadata(database) {
            Err(error) if error.kind() == io::ErrorKind::NotFound => Store::open(database)?,
            Ok(_) => {
                if let Ok(mut store) = Store::open_existing(database) {
                    store.operator_event(&admin, op, (kind, &id), "Conflict")?;
                }
                return Err(Error::Conflict.into());
            }
            Err(error) => return Err(error.into()),
        }
    } else {
        Store::open_existing(database)?
    };
    let result = execute(&mut s, &admin, cmd);
    match &result {
        // An idempotent replay applied nothing; record the attempt itself.
        Ok(value) if recorded && value["replayed"] == true => {
            s.operator_event(&admin, op, (kind, &id), "replayed")?;
        }
        Ok(_) if !recorded => {
            s.operator_event(&admin, op, (kind, &id), "ok")?;
        }
        Err(e) => {
            s.operator_event(&admin, op, (kind, &id), &e.to_string())?;
        }
        _ => {}
    }
    Ok(result?)
}

fn execute(s: &mut Store, admin: &Admin<'_>, command: Command) -> Result<Value, Error> {
    Ok(match command {
        Command::AuditList { filter } => json!({"audit":s.operator_audit(filter)?}),
        Command::Delete { account_id } => {
            s.delete_account(admin, account_id, None)?;
            json!({"ok":true})
        }
        Command::RecoveryReset { account_id } => {
            let code = s.reset_recovery(admin, account_id, None)?;
            json!({"recovery_code":code.as_str()})
        }
        Command::ListRooms {} | Command::CloseRoom { .. } | Command::F2Runtime { .. } => {
            unreachable!()
        }
        Command::MatchHistoryAccount { account_id, limit } => {
            json!({"matches":s.match_history(admin,HistoryFilter::Account(account_id),limit)?})
        }
        Command::MatchHistoryRoom { room_id, limit } => {
            json!({"matches":s.match_history(admin,HistoryFilter::Room(room_id),limit)?})
        }
        Command::Init {} => json!({"schema":SCHEMA_VERSION}),
        Command::BeginInstance {} => json!({"epoch":s.begin_instance(admin,accounts::utc()?)?}),
        Command::Create {
            login,
            name,
            password,
        } => json!({"player":s.create_account(admin,&login,&name,password.0.as_bytes(),None)?}),
        Command::Lookup { login } => json!({"player":s.lookup(&login)?}),
        Command::Login {
            login,
            password,
            ttl_seconds,
        } => {
            let g = s.login_current(&login, password.0.as_bytes(), ttl_seconds)?;
            json!({"player":g.player,"expires_at":g.expires_at,"token":g.token.expose_secret()})
        }
        Command::Authorize { token, player_id } => {
            json!({"player":s.authorize(&Token::parse(&token.0)?,player_id,accounts::utc()?)?})
        }
        Command::Revoke { account_id } => {
            s.revoke(admin, account_id, None)?;
            json!({"ok":true})
        }
        Command::Disable { account_id } => {
            s.set_disabled(admin, account_id, true, None)?;
            json!({"ok":true})
        }
        Command::Enable { account_id } => {
            s.set_disabled(admin, account_id, false, None)?;
            json!({"ok":true})
        }
        Command::Rename {
            account_id,
            name,
            expected_version,
        } => json!({"player":s.rename_player(admin,account_id,&name,expected_version,None)?}),
        Command::Reset {
            account_id,
            password,
        } => {
            s.reset_password(admin, account_id, password.0.as_bytes(), None)?;
            json!({"ok":true})
        }
        Command::ProfileGet { player_id } => json!({
            "profile": s.get_profile(player_id)?,
            "equipped": s.list_equipped(player_id)?,
        }),
        Command::LedgerList { player_id, limit } => {
            json!({"ledger": s.list_ledger(player_id, limit)?})
        }
        Command::InventoryList { player_id } => {
            json!({"inventory": s.list_inventory(player_id, accounts::utc()?)?})
        }
        Command::AdminGrantLucci {
            player_id,
            lucci_delta,
            idempotency_key,
        } => serde_json::to_value(s.admin_grant_lucci(
            admin,
            player_id,
            lucci_delta,
            &idempotency_key,
            None,
        )?)
        .map_err(|_| Error::Storage)?,
        Command::AdminGrantItem {
            player_id,
            sku_id,
            quantity,
            grant,
            idempotency_key,
        } => serde_json::to_value(s.admin_grant_item(
            admin,
            player_id,
            &sku_id,
            grant,
            quantity,
            &idempotency_key,
            None,
        )?)
        .map_err(|_| Error::Storage)?,
        Command::SocialFriendList { player_id } => {
            json!({"friends": s.list_friends(player_id, accounts::utc()?)?})
        }
        Command::SocialBlockList { player_id } => {
            json!({"blocks": s.list_blocks(player_id, accounts::utc()?)?})
        }
        Command::SocialMuteSet {
            player_id,
            duration_seconds,
            mute_reason,
        } => {
            json!({"expires_at": s.admin_set_social_mute(admin, player_id, duration_seconds, mute_reason)?})
        }
        Command::SocialMuteClear { player_id } => {
            json!({"cleared": s.admin_clear_social_mute(admin, player_id)?})
        }
        Command::SocialAbuseList {
            sender_id,
            before_id,
            limit,
        } => {
            json!({"events": s.list_chat_abuse(sender_id, before_id, limit)?})
        }
        Command::SocialGuildCreate {
            leader_player_id,
            name,
        } => {
            json!({"guild": s.admin_create_guild(admin, leader_player_id, &name)?})
        }
        Command::SocialGuildGet { guild_id } => {
            let (guild, members) = s.get_guild(guild_id)?;
            json!({"guild": guild, "members": members})
        }
        Command::SocialGuildMemberAdd {
            guild_id,
            player_id,
        } => {
            json!({"changed": s.admin_add_guild_member(admin, guild_id, player_id)?})
        }
        Command::SocialGuildMemberRemove {
            guild_id,
            player_id,
        } => {
            json!({"changed": s.admin_remove_guild_member(admin, guild_id, player_id)?})
        }
        Command::SocialGuildDisband { guild_id } => {
            s.admin_disband_guild(admin, guild_id)?;
            json!({"ok": true})
        }
        Command::SocialRankingList { offset, limit } => {
            json!({"ranking": s.ranking(offset, limit)?})
        }
        Command::F6PolicyCheck {
            policy_path,
            at_unix_secs,
        } => f6_policy_check(policy_path.as_deref(), at_unix_secs)
            .map_err(|_| Error::InvalidInput)?,
        Command::F4PolicyCheck { policy_path } => {
            f4_policy_check(policy_path.as_deref()).map_err(|_| Error::InvalidInput)?
        }
        Command::F3PolicyCheck { policy_path } => {
            f3_policy_check(policy_path.as_deref()).map_err(|_| Error::InvalidInput)?
        }
        Command::F5PolicyCheck { policy_path } => {
            f5_policy_check(policy_path.as_deref()).map_err(|_| Error::InvalidInput)?
        }
    })
}
fn run(args: &[OsString]) -> Result<Value, Box<dyn std::error::Error>> {
    Ok(match args {
        [flag, options @ ..] if flag == "f3-policy-check" => match options {
            [] => f3_policy_check(None)?,
            [option, path] if option == "--policy" => f3_policy_check(Some(Path::new(path)))?,
            _ => return Err("usage: kr-admin f3-policy-check [--policy PATH]".into()),
        },
        [flag, options @ ..] if flag == "f4-policy-check" => match options {
            [] => f4_policy_check(None)?,
            [option, path] if option == "--policy" => f4_policy_check(Some(Path::new(path)))?,
            _ => return Err("usage: kr-admin f4-policy-check [--policy PATH]".into()),
        },
        [flag, options @ ..] if flag == "f7-policy-check" => match options {
            [] => f7_policy_check(None)?,
            [option, path] if option == "--policy" => f7_policy_check(Some(Path::new(path)))?,
            _ => return Err("usage: kr-admin f7-policy-check [--policy PATH]".into()),
        },
        [flag, options @ ..] if flag == "f5-policy-check" => match options {
            [] => f5_policy_check(None)?,
            [option, path] if option == "--policy" => f5_policy_check(Some(Path::new(path)))?,
            _ => return Err("usage: kr-admin f5-policy-check [--policy PATH]".into()),
        },
        [flag, options @ ..] if flag == "f6-claim-event" => f6_claim_event(options)?,
        [flag, options @ ..] if flag == "policy-audit" => policy_audit(options)?,
        [flag, dir] if flag == "private-dir" => {
            kr_service::platform::provision_private_dir(Path::new(dir))?;
            json!({"ok":true})
        }
        [flag, run] if flag == "health" => kr_service::monitoring::check(Path::new(run))?,
        [flag, source, destination] if flag == "backup" =>
            json!({"schema": accounts::snapshot(Path::new(source), Path::new(destination))?}),
        [flag, snapshot] if flag == "verify-backup" =>
            json!({"schema": accounts::verify_snapshot(Path::new(snapshot))?}),
        [flag, source, destination] if flag == "restore" => {
            let destination = Path::new(destination);
            let _instance = operations::lock_database(destination)?;
            json!({"schema": accounts::restore(Path::new(source), destination)?})
        }
        [flag, db] if flag == "migrate" => {
            let db = Path::new(db);
            let _instance = operations::lock_database(db)?;
            if !std::fs::symlink_metadata(db)?.is_file() {
                return Err(Error::InvalidInput.into());
            }
            Store::open(db)?;
            json!({"schema": SCHEMA_VERSION})
        }
        [database] => manage(Path::new(database))?,
        _ => return Err(format!("usage: kr-admin DATABASE | backup SOURCE DESTINATION | verify-backup SNAPSHOT | restore SNAPSHOT DATABASE | migrate DATABASE | health RUN_DIRECTORY | private-dir DIRECTORY | f3-policy-check [--policy PATH] | f4-policy-check [--policy PATH] | f5-policy-check [--policy PATH] | f7-policy-check [--policy PATH]\n{F6_CLAIM_USAGE}\n{POLICY_AUDIT_USAGE}").into()),
    })
}
fn main() {
    match run(&std::env::args_os().skip(1).collect::<Vec<_>>()) {
        Ok(value) => println!("{value}"),
        Err(e) => {
            eprintln!("kr-admin: {e}");
            std::process::exit(1);
        }
    }
}
