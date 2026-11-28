//! F4 trusted service facade over the bounded account worker. Every DB read or
//! write runs on the dedicated `kr-account-db` thread, never on a Tokio task.
//! No KR wire field, price or reward formula is inferred; amounts come from the
//! operator's validated `EconomyPolicy` and `ShopCatalog`.
use super::{Client, Command, Error, Pending, Result, Worker};
use crate::accounts::BoosterSettlementRequest as StoreBoosterSettlementRequest;
use crate::accounts::{
    self, CatalogAccess, EconomyPolicy, EquippedItem, InventoryItem, ItemGrant,
    ItemUpgradeOperation, LedgerEntry, MatchItemConsumptionReceipt, MatchItemConsumptionRequest,
    PlayerProfile, PurchaseReceipt, RecordEraserOperation, ShopCatalog, Store, UsableItem,
    Validate,
};
use crate::f4_extended_policy::{
    BoosterRoomContext, BoosterTicket, BoosterTicketPolicy, CouponRequest, Decision,
    GiftCouponPolicy, LevelGrant, LevelRewardPolicy, LucciVaultPolicy, OperationKey,
    RecordEraserPolicy, RecordScope, SettlementKey, TitlePolicy, TitleUnlockDecision,
    UpgradeFamily, UpgradePolicy,
};
use std::{sync::Arc, time::Instant};
use tokio::sync::oneshot;

mod session_cosmetics;
pub use session_cosmetics::O1SessionCosmetics;

const MAX_EXTENDED_LABEL_BYTES: usize = 128;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MatchItemConsumptionOutcome {
    Persisted(MatchItemConsumptionReceipt),
    /// The session entitlement permits the use. Match rules still manage use
    /// limits and action replay; there is no inventory decrement or receipt.
    SessionUnlocked,
}

/// Validated operator economy configuration. The default keeps the M6/G1/F1
/// behavior: rewards disabled and an empty catalog, so nothing can be bought.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct EconomySettings {
    policy: EconomyPolicy,
    catalog: ShopCatalog,
    booster: BoosterTicketPolicy,
    level_rewards: LevelRewardPolicy,
    lucci_vault: LucciVaultPolicy,
    gift_coupon: GiftCouponPolicy,
    record_eraser: RecordEraserPolicy,
    upgrade: UpgradePolicy,
    title: TitlePolicy,
}
impl EconomySettings {
    pub fn new(policy: EconomyPolicy, catalog: ShopCatalog) -> Result<Self> {
        policy.validate().map_err(Error::Backend)?;
        catalog.validate().map_err(Error::Backend)?;
        Ok(Self {
            policy,
            catalog,
            ..Self::default()
        })
    }
    /// Opt in to the extended F4 settlement policies. Existing callers using
    /// [`Self::new`] retain the disabled, fail-closed defaults.
    pub fn with_extended_policies(
        mut self,
        booster: BoosterTicketPolicy,
        level_rewards: LevelRewardPolicy,
        lucci_vault: LucciVaultPolicy,
    ) -> Result<Self> {
        booster.validate().map_err(|_| Error::InvalidInput)?;
        level_rewards.validate().map_err(|_| Error::InvalidInput)?;
        lucci_vault.validate().map_err(|_| Error::InvalidInput)?;
        self.booster = booster;
        self.level_rewards = level_rewards;
        self.lucci_vault = lucci_vault;
        Ok(self)
    }
    pub fn set_extended_policies(
        &mut self,
        booster: BoosterTicketPolicy,
        level_rewards: LevelRewardPolicy,
        lucci_vault: LucciVaultPolicy,
    ) -> Result<()> {
        *self = self
            .clone()
            .with_extended_policies(booster, level_rewards, lucci_vault)?;
        Ok(())
    }
    /// Opt in to additional F4 operations independently of settlement policies.
    /// Coupon offer and title labels are limited to 128 bytes in the worker API.
    pub fn with_extended_ops_policies(
        mut self,
        gift_coupon: GiftCouponPolicy,
        record_eraser: RecordEraserPolicy,
        upgrade: UpgradePolicy,
        title: TitlePolicy,
    ) -> Result<Self> {
        self.set_extended_ops_policies(gift_coupon, record_eraser, upgrade, title)?;
        Ok(self)
    }
    pub fn set_extended_ops_policies(
        &mut self,
        gift_coupon: GiftCouponPolicy,
        record_eraser: RecordEraserPolicy,
        upgrade: UpgradePolicy,
        title: TitlePolicy,
    ) -> Result<()> {
        gift_coupon.validate().map_err(|_| Error::InvalidInput)?;
        record_eraser.validate().map_err(|_| Error::InvalidInput)?;
        upgrade.validate().map_err(|_| Error::InvalidInput)?;
        title.validate().map_err(|_| Error::InvalidInput)?;
        if gift_coupon
            .coupon_offers
            .iter()
            .any(|row| row.offer.len() > MAX_EXTENDED_LABEL_BYTES)
            || title
                .unlock_table
                .iter()
                .flatten()
                .any(|row| row.title.len() > MAX_EXTENDED_LABEL_BYTES)
        {
            return Err(Error::InvalidInput);
        }
        self.gift_coupon = gift_coupon;
        self.record_eraser = record_eraser;
        self.upgrade = upgrade;
        self.title = title;
        Ok(())
    }
    pub fn policy(&self) -> &EconomyPolicy {
        &self.policy
    }
    pub fn catalog(&self) -> &ShopCatalog {
        &self.catalog
    }
    pub fn booster_ticket_policy(&self) -> &BoosterTicketPolicy {
        &self.booster
    }
    pub fn level_reward_policy(&self) -> &LevelRewardPolicy {
        &self.level_rewards
    }
    pub fn lucci_vault_policy(&self) -> &LucciVaultPolicy {
        &self.lucci_vault
    }
    pub fn gift_coupon_policy(&self) -> &GiftCouponPolicy {
        &self.gift_coupon
    }
    pub fn record_eraser_policy(&self) -> &RecordEraserPolicy {
        &self.record_eraser
    }
    pub fn upgrade_policy(&self) -> &UpgradePolicy {
        &self.upgrade
    }
    pub fn title_policy(&self) -> &TitlePolicy {
        &self.title
    }
    /// True when finished matches settle ledger rewards.
    pub fn rewards_enabled(&self) -> bool {
        self.policy.enabled
    }
}

pub(super) enum EconomyRequest {
    Profile(i64),
    Ledger(i64, u8),
    Inventory(i64),
    Equipped(i64),
    Usable(i64),
    Purchase {
        player: i64,
        sku: String,
        quantity: u32,
        key: String,
    },
    Equip {
        player: i64,
        sku: String,
        slot: String,
    },
    Unequip {
        player: i64,
        slot: String,
    },
    Consume(MatchItemConsumptionRequest),
    ConsumeEquipped(MatchItemConsumptionRequest, accounts::Token, String),
    BoosterSettlement {
        player: i64,
        key: SettlementKey,
        room_group: String,
        mode: String,
        map: String,
        room_rules_allow: bool,
        normal_unranked: bool,
        ticket: BoosterTicket,
    },
    LevelMilestone {
        player: i64,
        grant: LevelGrant,
    },
    RedeemGiftCoupon {
        player: i64,
        key: OperationKey,
        coupon_id: u64,
        offer: String,
        issued_to: u64,
        issued_at: u64,
        locally_issued: bool,
    },
    RecordEraser {
        player: i64,
        key: u64,
        sku: String,
        scope: RecordScope,
    },
    ItemUpgrade {
        player: i64,
        key: u64,
        sku: String,
        family: UpgradeFamily,
        level: u16,
        roll_bps: u16,
    },
    TitleUnlock {
        player: i64,
        title: String,
    },
}
pub(super) enum EconomyReply {
    Profile(PlayerProfile),
    Ledger(Vec<LedgerEntry>),
    Inventory(Vec<InventoryItem>),
    Equipped(Vec<EquippedItem>),
    Usable(Vec<UsableItem>),
    Purchase(Box<PurchaseReceipt>),
    Equip(EquippedItem),
    Unequip(bool),
    Consume(MatchItemConsumptionReceipt),
    BoosterSettlement(Decision<LedgerEntry>),
    LevelMilestone(Decision<LedgerEntry>),
    GiftCoupon(Decision<LedgerEntry>),
    RecordEraser(Decision<LedgerEntry>),
    ItemUpgrade(Decision<LedgerEntry>),
    TitleUnlock(Option<TitleUnlockDecision>),
}

/// Execute one economy request on the worker thread with execution-time UTC.
pub(super) fn execute(
    store: &Store,
    settings: &EconomySettings,
    request: EconomyRequest,
    now: i64,
) -> accounts::Result<EconomyReply> {
    let catalog = &settings.catalog;
    Ok(match request {
        EconomyRequest::Profile(player) => EconomyReply::Profile(store.get_profile(player)?),
        EconomyRequest::Ledger(player, limit) => {
            EconomyReply::Ledger(store.list_ledger(player, limit)?)
        }
        EconomyRequest::Inventory(player) => {
            EconomyReply::Inventory(store.list_inventory(player, now)?)
        }
        EconomyRequest::Equipped(player) => EconomyReply::Equipped(store.list_equipped(player)?),
        EconomyRequest::Usable(player) => EconomyReply::Usable(store.usable_items(
            player,
            catalog,
            CatalogAccess { unlock_all: false },
            now,
        )?),
        EconomyRequest::Purchase {
            player,
            sku,
            quantity,
            key,
        } => EconomyReply::Purchase(Box::new(
            store.purchase_shop_item(player, catalog, &sku, quantity, &key, now)?,
        )),
        EconomyRequest::Equip { player, sku, slot } => {
            EconomyReply::Equip(store.equip_item(player, catalog, &sku, &slot, now)?)
        }
        EconomyRequest::Unequip { player, slot } => {
            EconomyReply::Unequip(store.unequip_slot(player, &slot, now)?)
        }
        EconomyRequest::Consume(request) => {
            EconomyReply::Consume(store.consume_item_in_match(&request, now)?)
        }
        EconomyRequest::ConsumeEquipped(request, token, slot) => EconomyReply::Consume(
            store.consume_equipped_item_in_match(&request, &token, &slot, now)?,
        ),
        EconomyRequest::BoosterSettlement {
            player,
            key,
            room_group,
            mode,
            map,
            room_rules_allow,
            normal_unranked,
            ticket,
        } => {
            if !settings.booster.enabled {
                return Err(accounts::Error::InvalidInput);
            }
            let room = BoosterRoomContext {
                room_group: &room_group,
                mode: &mode,
                map: &map,
                room_rules_allow,
                normal_unranked,
            };
            let request = StoreBoosterSettlementRequest { key, room, ticket };
            EconomyReply::BoosterSettlement(store.apply_booster_settlement(
                player,
                request,
                &settings.booster,
                &settings.lucci_vault,
                &settings.policy,
                now,
            )?)
        }
        EconomyRequest::LevelMilestone { player, grant } => {
            if !settings.level_rewards.enabled {
                return Err(accounts::Error::InvalidInput);
            }
            EconomyReply::LevelMilestone(store.claim_level_milestone_reward(
                player,
                &grant,
                &settings.level_rewards,
                &settings.lucci_vault,
                &settings.catalog,
                now,
            )?)
        }
        EconomyRequest::RedeemGiftCoupon {
            player,
            key,
            coupon_id,
            offer,
            issued_to,
            issued_at,
            locally_issued,
        } => EconomyReply::GiftCoupon(store.redeem_gift_coupon(
            player,
            CouponRequest {
                key,
                coupon_id,
                offer: &offer,
                issued_to,
                issued_at,
                now: u64::try_from(now).map_err(|_| accounts::Error::InvalidInput)?,
                locally_issued,
            },
            &settings.gift_coupon,
            &settings.lucci_vault,
            catalog,
        )?),
        EconomyRequest::RecordEraser {
            player,
            key,
            sku,
            scope,
        } => EconomyReply::RecordEraser(store.apply_record_eraser(
            player,
            RecordEraserOperation {
                key,
                sku: &sku,
                scope,
            },
            &settings.record_eraser,
            now,
        )?),
        EconomyRequest::ItemUpgrade {
            player,
            key,
            sku,
            family,
            level,
            roll_bps,
        } => EconomyReply::ItemUpgrade(store.apply_item_upgrade(
            player,
            ItemUpgradeOperation {
                key,
                sku: &sku,
                family,
                level,
                roll_bps,
            },
            &settings.upgrade,
            now,
        )?),
        EconomyRequest::TitleUnlock { player, title } => {
            if !settings.title.enabled {
                return Err(accounts::Error::InvalidInput);
            }
            EconomyReply::TitleUnlock(store.evaluate_title_unlock(
                player,
                &title,
                &settings.title,
                now,
            )?)
        }
    })
}

/// Trusted async F4 account API. Like `PersistenceJob`, it takes a player id
/// that the caller has already bound to an authorized session; it is not a
/// client-facing endpoint. Admission is bounded and nonblocking (`Busy` when
/// the queue is full); queue expiry is checked before execution.
#[derive(Clone)]
pub struct AccountService {
    pub(crate) client: Client,
    settings: Arc<EconomySettings>,
}
impl Worker {
    /// Service facade sharing this worker's queue and economy settings.
    pub fn service(&self) -> AccountService {
        AccountService {
            client: self.client.clone(),
            settings: self.economy.clone(),
        }
    }
}
impl AccountService {
    pub fn settings(&self) -> &EconomySettings {
        &self.settings
    }
    pub async fn profile_get(&self, player_id: i64) -> Result<PlayerProfile> {
        match self
            .call(player_id, EconomyRequest::Profile(player_id))
            .await?
        {
            EconomyReply::Profile(profile) => Ok(profile),
            _ => Err(Error::WorkerFault),
        }
    }
    /// Newest first; `limit` is 1..=100.
    pub async fn ledger_list(&self, player_id: i64, limit: u8) -> Result<Vec<LedgerEntry>> {
        if !(1..=100).contains(&limit) {
            return Err(Error::InvalidInput);
        }
        match self
            .call(player_id, EconomyRequest::Ledger(player_id, limit))
            .await?
        {
            EconomyReply::Ledger(rows) => Ok(rows),
            _ => Err(Error::WorkerFault),
        }
    }
    pub async fn inventory_list(&self, player_id: i64) -> Result<Vec<InventoryItem>> {
        match self
            .call(player_id, EconomyRequest::Inventory(player_id))
            .await?
        {
            EconomyReply::Inventory(items) => Ok(items),
            _ => Err(Error::WorkerFault),
        }
    }
    pub async fn equipped_list(&self, player_id: i64) -> Result<Vec<EquippedItem>> {
        match self
            .call(player_id, EconomyRequest::Equipped(player_id))
            .await?
        {
            EconomyReply::Equipped(items) => Ok(items),
            _ => Err(Error::WorkerFault),
        }
    }
    /// Buy from the configured catalog, once per `(player, key)`. An unknown
    /// SKU (including any SKU while the catalog is empty) is rejected before
    /// queue admission.
    pub async fn purchase_shop_item(
        &self,
        player_id: i64,
        sku_id: &str,
        quantity: u32,
        idempotency_key: &str,
    ) -> Result<PurchaseReceipt> {
        if self.settings.catalog.item(sku_id).is_none()
            || quantity == 0
            || !(1..=128).contains(&idempotency_key.len())
        {
            return Err(Error::InvalidInput);
        }
        let request = EconomyRequest::Purchase {
            player: player_id,
            sku: sku_id.into(),
            quantity,
            key: idempotency_key.into(),
        };
        match self.call(player_id, request).await? {
            EconomyReply::Purchase(receipt) => Ok(*receipt),
            _ => Err(Error::WorkerFault),
        }
    }
    pub async fn equip_item(
        &self,
        player_id: i64,
        sku_id: &str,
        slot: &str,
    ) -> Result<EquippedItem> {
        if self.settings.catalog.item(sku_id).is_none() || !(1..=32).contains(&slot.len()) {
            return Err(Error::InvalidInput);
        }
        let request = EconomyRequest::Equip {
            player: player_id,
            sku: sku_id.into(),
            slot: slot.into(),
        };
        match self.call(player_id, request).await? {
            EconomyReply::Equip(item) => Ok(item),
            _ => Err(Error::WorkerFault),
        }
    }
    /// Returns false when the slot was already empty.
    pub async fn unequip_slot(&self, player_id: i64, slot: &str) -> Result<bool> {
        if !(1..=32).contains(&slot.len()) {
            return Err(Error::InvalidInput);
        }
        let request = EconomyRequest::Unequip {
            player: player_id,
            slot: slot.into(),
        };
        match self.call(player_id, request).await? {
            EconomyReply::Unequip(removed) => Ok(removed),
            _ => Err(Error::WorkerFault),
        }
    }
    /// Persist a server-approved item use, or use the O1 catalog entitlement.
    /// In O1, the caller validates the live round and deduplicates actions in
    /// match state; its round identifier need not refer to a database match.
    pub async fn consume_item_in_match(
        &self,
        request: &MatchItemConsumptionRequest,
        access: CatalogAccess,
    ) -> Result<MatchItemConsumptionOutcome> {
        request.validate().map_err(|_| Error::InvalidInput)?;
        if access.unlock_all {
            let usable = self
                .settings
                .catalog
                .item(&request.sku_id)
                .is_some_and(|item| {
                    item.enabled && matches!(item.grant, ItemGrant::Stackable { .. })
                });
            return if usable {
                Ok(MatchItemConsumptionOutcome::SessionUnlocked)
            } else {
                Err(Error::InvalidInput)
            };
        }
        match self
            .call(request.player_id, EconomyRequest::Consume(request.clone()))
            .await?
        {
            EconomyReply::Consume(receipt) => Ok(MatchItemConsumptionOutcome::Persisted(receipt)),
            _ => Err(Error::WorkerFault),
        }
    }
    pub(crate) async fn consume_equipped_item_in_match(
        &self,
        request: MatchItemConsumptionRequest,
        token: accounts::Token,
        slot: &str,
    ) -> Result<MatchItemConsumptionOutcome> {
        request.validate().map_err(|_| Error::InvalidInput)?;
        accounts::equip_slot(slot).map_err(|_| Error::InvalidInput)?;
        match self
            .call(
                request.player_id,
                EconomyRequest::ConsumeEquipped(request, token, slot.into()),
            )
            .await?
        {
            EconomyReply::Consume(receipt) => Ok(MatchItemConsumptionOutcome::Persisted(receipt)),
            _ => Err(Error::WorkerFault),
        }
    }
    /// Start with empty selections for one authorized O1 session.
    pub fn o1_session_cosmetics(&self, access: CatalogAccess) -> Option<O1SessionCosmetics> {
        access
            .unlock_all
            .then(|| O1SessionCosmetics::new(self.settings.clone()))
    }
    /// O1 read-only helper. With `unlock_all` the answer comes from the catalog
    /// in memory: no queue admission, no DB read or write and no Lucci charge.
    /// Otherwise only the player's own active inventory counts.
    pub async fn usable_items(
        &self,
        player_id: i64,
        access: CatalogAccess,
    ) -> Result<Vec<UsableItem>> {
        if access.unlock_all {
            return Ok(self.settings.catalog.session_unlocked());
        }
        match self
            .call(player_id, EconomyRequest::Usable(player_id))
            .await?
        {
            EconomyReply::Usable(items) => Ok(items),
            _ => Err(Error::WorkerFault),
        }
    }
    /// Apply the configured booster ticket bonus to a completed match. The
    /// request is admitted through the same bounded queue as all economy I/O.
    pub async fn apply_booster_settlement(
        &self,
        player_id: i64,
        key: SettlementKey,
        room: BoosterRoomContext<'_>,
        ticket: BoosterTicket,
    ) -> Result<Decision<LedgerEntry>> {
        if !self.settings.booster.enabled
            || key.account == 0
            || key.match_id == 0
            || ticket.account != key.account
            || room.room_group.is_empty()
            || room.mode.is_empty()
            || room.map.is_empty()
            || room.room_group.len() > 64
            || room.mode.len() > 64
            || room.map.len() > 64
        {
            return Err(Error::InvalidInput);
        }
        match self
            .call(
                player_id,
                EconomyRequest::BoosterSettlement {
                    player: player_id,
                    key,
                    room_group: room.room_group.into(),
                    mode: room.mode.into(),
                    map: room.map.into(),
                    room_rules_allow: room.room_rules_allow,
                    normal_unranked: room.normal_unranked,
                    ticket,
                },
            )
            .await?
        {
            EconomyReply::BoosterSettlement(receipt) => Ok(receipt),
            _ => Err(Error::WorkerFault),
        }
    }
    /// Claim one configured level milestone grant, idempotently.
    pub async fn claim_level_milestone_reward(
        &self,
        player_id: i64,
        grant: &LevelGrant,
    ) -> Result<Decision<LedgerEntry>> {
        if !self.settings.level_rewards.enabled || grant.claim.account == 0 || grant.claim.level < 2
        {
            return Err(Error::InvalidInput);
        }
        match self
            .call(
                player_id,
                EconomyRequest::LevelMilestone {
                    player: player_id,
                    grant: grant.clone(),
                },
            )
            .await?
        {
            EconomyReply::LevelMilestone(receipt) => Ok(receipt),
            _ => Err(Error::WorkerFault),
        }
    }
    /// Redeem a server-resolved local coupon. The worker uses execution-time
    /// UTC in place of `request.now`.
    pub async fn redeem_gift_coupon(
        &self,
        player_id: i64,
        request: CouponRequest<'_>,
    ) -> Result<Decision<LedgerEntry>> {
        if !self.settings.gift_coupon.enabled
            || request.key.account == 0
            || request.key.id == 0
            || request.coupon_id == 0
            || !request.locally_issued
            || request.issued_to != request.key.account
            || !(1..=MAX_EXTENDED_LABEL_BYTES).contains(&request.offer.len())
        {
            return Err(Error::InvalidInput);
        }
        let request = EconomyRequest::RedeemGiftCoupon {
            player: player_id,
            key: request.key,
            coupon_id: request.coupon_id,
            offer: request.offer.into(),
            issued_to: request.issued_to,
            issued_at: request.issued_at,
            locally_issued: request.locally_issued,
        };
        match self.call(player_id, request).await? {
            EconomyReply::GiftCoupon(receipt) => Ok(receipt),
            _ => Err(Error::WorkerFault),
        }
    }
    /// Consume an owned eraser and apply the configured record reset.
    pub async fn apply_record_eraser(
        &self,
        player_id: i64,
        request: RecordEraserOperation<'_>,
    ) -> Result<Decision<LedgerEntry>> {
        if !self.settings.record_eraser.enabled || request.key == 0 {
            return Err(Error::InvalidInput);
        }
        accounts::sku(request.sku).map_err(|_| Error::InvalidInput)?;
        let request = EconomyRequest::RecordEraser {
            player: player_id,
            key: request.key,
            sku: request.sku.into(),
            scope: request.scope,
        };
        match self.call(player_id, request).await? {
            EconomyReply::RecordEraser(receipt) => Ok(receipt),
            _ => Err(Error::WorkerFault),
        }
    }
    /// Apply one configured upgrade using a trusted roll. A replay returns
    /// the stored receipt regardless of the new roll.
    pub async fn apply_item_upgrade(
        &self,
        player_id: i64,
        request: ItemUpgradeOperation<'_>,
    ) -> Result<Decision<LedgerEntry>> {
        if !self.settings.upgrade.enabled || request.key == 0 {
            return Err(Error::InvalidInput);
        }
        accounts::sku(request.sku).map_err(|_| Error::InvalidInput)?;
        let request = EconomyRequest::ItemUpgrade {
            player: player_id,
            key: request.key,
            sku: request.sku.into(),
            family: request.family,
            level: request.level,
            roll_bps: request.roll_bps,
        };
        match self.call(player_id, request).await? {
            EconomyReply::ItemUpgrade(receipt) => Ok(receipt),
            _ => Err(Error::WorkerFault),
        }
    }
    /// Read title eligibility. Returns `None` for an unknown or unmet title
    /// when enabled. This does not persist a title or emit a notification.
    pub async fn evaluate_title_unlock(
        &self,
        player_id: i64,
        title: &str,
    ) -> Result<Option<TitleUnlockDecision>> {
        if !self.settings.title.enabled || !(1..=MAX_EXTENDED_LABEL_BYTES).contains(&title.len()) {
            return Err(Error::InvalidInput);
        }
        let request = EconomyRequest::TitleUnlock {
            player: player_id,
            title: title.into(),
        };
        match self.call(player_id, request).await? {
            EconomyReply::TitleUnlock(decision) => Ok(decision),
            _ => Err(Error::WorkerFault),
        }
    }
    async fn call(&self, player_id: i64, request: EconomyRequest) -> Result<EconomyReply> {
        if player_id < 1 {
            return Err(Error::InvalidInput);
        }
        let (reply, rx) = oneshot::channel();
        self.client.submit(Command::Economy {
            request,
            deadline: Instant::now() + self.client.queue_ttl,
            reply,
        })?;
        Pending {
            reply: Some(rx),
            handoff: None,
        }
        .wait()
        .await
    }
}

#[cfg(test)]
mod tests;
