//! F4 trusted service facade over the bounded account worker. Every DB read or
//! write runs on the dedicated `kr-account-db` thread, never on a Tokio task.
//! No KR wire field, price or reward formula is inferred; amounts come from the
//! operator's validated `EconomyPolicy` and `ShopCatalog`.
use super::{Client, Command, Error, Pending, Result, Worker};
use crate::accounts::{
    self, CatalogAccess, EconomyPolicy, EquippedItem, InventoryItem, LedgerEntry, PlayerProfile,
    PurchaseReceipt, ShopCatalog, Store, UsableItem, Validate,
};
use std::{sync::Arc, time::Instant};
use tokio::sync::oneshot;

/// Validated operator economy configuration. The default keeps the M6/G1/F1
/// behavior: rewards disabled and an empty catalog, so nothing can be bought.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct EconomySettings {
    policy: EconomyPolicy,
    catalog: ShopCatalog,
}
impl EconomySettings {
    pub fn new(policy: EconomyPolicy, catalog: ShopCatalog) -> Result<Self> {
        policy.validate().map_err(Error::Backend)?;
        catalog.validate().map_err(Error::Backend)?;
        Ok(Self { policy, catalog })
    }
    pub fn policy(&self) -> &EconomyPolicy {
        &self.policy
    }
    pub fn catalog(&self) -> &ShopCatalog {
        &self.catalog
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
    })
}

/// Trusted async F4 account API. Like `PersistenceJob`, it takes a player id
/// that the caller has already bound to an authorized session; it is not a
/// client-facing endpoint. Admission is bounded and nonblocking (`Busy` when
/// the queue is full); queue expiry is checked before execution.
#[derive(Clone)]
pub struct AccountService {
    client: Client,
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
