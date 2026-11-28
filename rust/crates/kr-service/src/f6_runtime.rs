//! Server-side F6 claims. Eligibility and reward selection belong to the
//! trusted caller; this module does not accept claims from the KR wire.

use crate::{
    accounts::{Admin, AuditFilter, ECONOMY_AMOUNT_MAX, Error, ItemGrant, Store, Validate, sku},
    f6_policy::{F6EventPolicy, F6QuotaTracker, QuotaRejection},
    sha256_hex,
};
use serde::Serialize;
use std::collections::HashMap;

// Reserve this audit actor and ledger key prefix for the runtime.
const ACTOR: &str = "f6-event-runtime";
const KEY_PREFIX: &str = "f6:v1:";
const MAX_CLAIM_KEY_BYTES: usize = 128;
const WIRE_STATUS: &str = "policy_runtime_only_pending_client_capture";

/// Aggregate quota counters for the current runtime and UTC windows.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct F6QuotaUsage {
    pub claims: usize,
    pub hourly: usize,
    pub minute: usize,
    pub daily: usize,
    pub seasonal: usize,
}

/// A path-free operational view of the F6 policy and claim runtime.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct F6EventStatusSnapshot {
    pub enabled: bool,
    pub configured_systems: Vec<&'static str>,
    pub active_systems: Vec<&'static str>,
    pub quota_usage: F6QuotaUsage,
    pub wire_status: &'static str,
}

/// One reward per claim keeps the ledger, reward and audit in one transaction.
#[derive(Clone, Copy, Debug, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum F6Reward<'a> {
    Lucci {
        amount: i64,
    },
    Item {
        sku_id: &'a str,
        grant: ItemGrant,
        quantity: u32,
    },
}

impl F6Reward<'_> {
    fn validate(self) -> Result<(), Error> {
        match self {
            Self::Lucci { amount } => {
                if amount <= 0 || amount.unsigned_abs() > ECONOMY_AMOUNT_MAX {
                    return Err(Error::InvalidInput);
                }
            }
            Self::Item {
                sku_id,
                grant,
                quantity,
            } => {
                sku(sku_id)?;
                grant.validate()?;
                let valid_quantity = match grant {
                    ItemGrant::Permanent => quantity == 1,
                    ItemGrant::Stackable { max_stack } => (1..=max_stack).contains(&quantity),
                    ItemGrant::Timed { duration_seconds } => {
                        quantity > 0 && duration_seconds <= i64::MAX as u64 / u64::from(quantity)
                    }
                };
                if !valid_quantity {
                    return Err(Error::InvalidInput);
                }
            }
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Serialize)]
pub struct F6Claim<'a> {
    /// A stable, server-issued key unique to the logical claim across systems
    /// and players in this store. It must not be regenerated on retry.
    #[serde(skip)]
    pub key: &'a str,
    pub system: &'a str,
    pub player_id: i64,
    pub reward: F6Reward<'a>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ClaimOutcome {
    Granted,
    Replayed,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ClaimError {
    Policy(QuotaRejection),
    InvalidKey,
    Conflict,
    Accounts(Error),
}

#[derive(Debug)]
struct QuotaCharge {
    system: String,
    player_id: i64,
    utc: i64,
}

/// Keep one runtime per account store and serialize claims through it. Claim
/// identities survive restart through the grant's audit row; quota counters
/// are in memory for the lifetime of this runtime, as in `F6QuotaTracker`.
pub struct F6EventRuntime<'a> {
    store: &'a Store,
    policy: F6EventPolicy,
    quota: F6QuotaTracker,
    charges: Vec<QuotaCharge>,
    last_quota_rejection: Option<QuotaCharge>,
    claims: HashMap<String, String>,
}

impl<'a> F6EventRuntime<'a> {
    pub fn new(store: &'a Store, policy: F6EventPolicy) -> Result<Self, Error> {
        let mut claims = HashMap::new();
        let mut before_id = None;
        // The store writes the audit row in the same transaction as the
        // grant. Keep the full claim fingerprint there: the ledger's request
        // hash covers the reward and player but has no event system field.
        loop {
            let page = store.operator_audit(AuditFilter {
                actor: Some(ACTOR.into()),
                before_id,
                limit: Some(100),
                ..AuditFilter::default()
            })?;
            if page.is_empty() {
                break;
            }
            for row in page {
                let (key, fingerprint) = row
                    .reason
                    .strip_prefix(KEY_PREFIX)
                    .and_then(|reason| reason.split_once(':'))
                    .ok_or(Error::Schema)?;
                if row.result != "ok"
                    || !matches!(
                        row.operation.as_str(),
                        "admin_grant_lucci" | "admin_grant_item"
                    )
                    || !is_digest(key)
                    || !is_digest(fingerprint)
                    || claims
                        .insert(key.to_owned(), fingerprint.to_owned())
                        .is_some()
                {
                    return Err(Error::Schema);
                }
                before_id = Some(row.id);
            }
        }
        Ok(Self {
            store,
            policy,
            quota: F6QuotaTracker::default(),
            charges: Vec::new(),
            last_quota_rejection: None,
            claims,
        })
    }

    /// Return a stable, path-free status view of policy and in-memory quota
    /// usage. The active window is evaluated at the supplied UTC second.
    pub fn status_snapshot(&self, now_unix_secs: i64) -> F6EventStatusSnapshot {
        let configured_systems: Vec<&'static str> = self.policy.configured_system_names().collect();
        let active_systems: Vec<&'static str> = configured_systems
            .iter()
            .copied()
            .filter(|name| self.policy.is_system_enabled_at(name, now_unix_secs))
            .collect();
        let is_extended = |system: &str| {
            self.policy
                .enabled_extended_systems()
                .any(|(name, _)| name == system)
        };
        let claims = self.charges.len();
        let hourly = self
            .charges
            .iter()
            .filter(|charge| charge.utc / 3_600 == now_unix_secs / 3_600)
            .count();
        let minute = self
            .charges
            .iter()
            .filter(|charge| charge.utc / 60 == now_unix_secs / 60)
            .count();
        let daily = self
            .charges
            .iter()
            .filter(|charge| {
                is_extended(&charge.system) && charge.utc / 86_400 == now_unix_secs / 86_400
            })
            .count();
        let seasonal = self
            .charges
            .iter()
            .filter(|charge| is_extended(&charge.system))
            .count();

        F6EventStatusSnapshot {
            enabled: !configured_systems.is_empty(),
            configured_systems,
            active_systems,
            quota_usage: F6QuotaUsage {
                claims,
                hourly,
                minute,
                daily,
                seasonal,
            },
            wire_status: WIRE_STATUS,
        }
    }

    /// Apply a trusted claim at the supplied server UTC second. An identical
    /// replay still requires an active schedule but never charges quota.
    pub fn claim(&mut self, claim: F6Claim<'_>, now_utc: i64) -> Result<ClaimOutcome, ClaimError> {
        if !(1..=MAX_CLAIM_KEY_BYTES).contains(&claim.key.len())
            || claim.key.chars().any(char::is_control)
        {
            return Err(ClaimError::InvalidKey);
        }
        let digest = sha256_hex(claim.key.as_bytes());
        let fingerprint = sha256_hex(
            &serde_json::to_vec(&claim).map_err(|_| ClaimError::Accounts(Error::InvalidInput))?,
        );
        if let Some(previous) = self.claims.get(&digest) {
            if previous != &fingerprint {
                return Err(ClaimError::Conflict);
            }
            self.policy
                .check_operation(
                    &mut F6QuotaTracker::default(),
                    claim.system,
                    claim.player_id,
                    now_utc,
                )
                .map_err(ClaimError::Policy)?;
            return Ok(ClaimOutcome::Replayed);
        }

        claim.reward.validate().map_err(ClaimError::Accounts)?;
        // The schema allows exactly one player per account. Using the player
        // ID for per-account caps prevents mismatched player/account inputs.
        if let Err(error) =
            self.policy
                .check_operation(&mut self.quota, claim.system, claim.player_id, now_utc)
        {
            if matches!(
                error,
                QuotaRejection::AccountQuotaExceeded
                    | QuotaRejection::GlobalQuotaExceeded
                    | QuotaRejection::ExtendedDailyQuotaExceeded
                    | QuotaRejection::ExtendedSeasonQuotaExceeded
            ) {
                self.last_quota_rejection = Some(QuotaCharge {
                    system: claim.system.into(),
                    player_id: claim.player_id,
                    utc: now_utc,
                });
            }
            return Err(ClaimError::Policy(error));
        }

        // Both accounts APIs use the same per-player admin_grant namespace
        // and accept ASCII hex and colons, but not slashes.
        let key = format!("{KEY_PREFIX}{digest}");
        let reason = format!("{key}:{fingerprint}");
        let admin = Admin {
            actor: ACTOR,
            reason: &reason,
        };
        let grant = match claim.reward {
            F6Reward::Lucci { amount } => {
                self.store
                    .admin_grant_lucci(&admin, claim.player_id, amount, &key, now_utc)
            }
            F6Reward::Item {
                sku_id,
                grant,
                quantity,
            } => self.store.admin_grant_item(
                &admin,
                claim.player_id,
                sku_id,
                grant,
                quantity,
                &key,
                now_utc,
            ),
        };
        match grant {
            Ok(receipt) if !receipt.replayed => {
                self.claims.insert(digest, fingerprint);
                self.charges.push(QuotaCharge {
                    system: claim.system.into(),
                    player_id: claim.player_id,
                    utc: now_utc,
                });
                self.last_quota_rejection = None;
                Ok(ClaimOutcome::Granted)
            }
            other => {
                self.restore_quota();
                match other {
                    // A ledger replay without a known claim fingerprint cannot
                    // establish that the event claim matches.
                    Ok(_) | Err(Error::Conflict) => Err(ClaimError::Conflict),
                    Err(error) => Err(ClaimError::Accounts(error)),
                }
            }
        }
    }

    fn restore_quota(&mut self) {
        // The policy tracker has no undo API. Rebuild only on grant failure
        // from successful charges and the most recent quota rejection (which
        // may have advanced the clock and rolled the UTC buckets).
        let mut quota = F6QuotaTracker::default();
        for charge in &self.charges {
            self.policy
                .check_operation(&mut quota, &charge.system, charge.player_id, charge.utc)
                .expect("committed quota charge must replay under the same policy");
        }
        if let Some(rejection) = &self.last_quota_rejection {
            let _ = self.policy.check_operation(
                &mut quota,
                &rejection.system,
                rejection.player_id,
                rejection.utc,
            );
        }
        self.quota = quota;
    }
}

fn is_digest(input: &str) -> bool {
    input.len() == 64
        && input
            .bytes()
            .all(|b| b.is_ascii_digit() || matches!(b, b'a'..=b'f'))
}
