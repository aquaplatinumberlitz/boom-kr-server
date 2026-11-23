//! Shop purchase, operator item grants, inventory and equip slots.
use super::*;
use serde_json::json;

fn read_item(
    db: &Connection,
    player: i64,
    sku_id: &str,
    now: i64,
) -> Result<Option<InventoryItem>> {
    let item = db
        .query_row(
            "SELECT sku_id,grant_kind,quantity,expires_at,acquired_at,updated_at
             FROM player_inventory WHERE player_id=?1 AND sku_id=?2",
            params![player, sku_id],
            |r| inventory_row(r, now),
        )
        .optional()?;
    item.map(|mut item| {
        item.equipped_slots = equipped_slots(db, player, &item.sku_id)?;
        Ok(item)
    })
    .transpose()
}
fn inventory_row(r: &rusqlite::Row<'_>, now: i64) -> rusqlite::Result<InventoryItem> {
    let expires_at: Option<i64> = r.get(3)?;
    Ok(InventoryItem {
        sku_id: r.get(0)?,
        grant_kind: r.get(1)?,
        quantity: r.get(2)?,
        expires_at,
        acquired_at: r.get(4)?,
        updated_at: r.get(5)?,
        active: expires_at.is_none_or(|at| at > now),
        equipped_slots: Vec::new(),
    })
}
fn equipped_slots(db: &Connection, player: i64, sku_id: &str) -> Result<Vec<String>> {
    Ok(db
        .prepare("SELECT slot FROM player_equipped WHERE player_id=?1 AND sku_id=?2 ORDER BY slot")?
        .query_map(params![player, sku_id], |r| r.get(0))?
        .collect::<std::result::Result<_, _>>()?)
}

/// Apply one grant to the inventory row. Permanent cannot be owned twice,
/// stackable cannot exceed its cap, and timed extends from the later of `now`
/// and the current expiry while keeping the original `acquired_at`.
fn grant_item(
    tx: &Transaction<'_>,
    player: i64,
    sku_id: &str,
    grant: ItemGrant,
    quantity: u32,
    now: i64,
) -> Result<InventoryItem> {
    let current: Option<(String, i64, Option<i64>)> = tx
        .query_row(
            "SELECT grant_kind,quantity,expires_at FROM player_inventory WHERE player_id=?1 AND sku_id=?2",
            params![player, sku_id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .optional()?;
    if current.as_ref().is_some_and(|c| c.0 != grant.kind()) {
        return Err(Error::Conflict);
    }
    let (quantity, expires_at) = match grant {
        ItemGrant::Permanent => {
            if current.is_some() {
                return Err(Error::Conflict);
            }
            (1, None)
        }
        ItemGrant::Stackable { max_stack } => {
            let total = current
                .as_ref()
                .map_or(0, |c| c.1)
                .checked_add(i64::from(quantity))
                .ok_or(Error::Conflict)?;
            if total > i64::from(max_stack) {
                return Err(Error::Conflict);
            }
            (total, None)
        }
        ItemGrant::Timed { duration_seconds } => {
            let extend = duration_seconds
                .checked_mul(u64::from(quantity))
                .and_then(|v| i64::try_from(v).ok())
                .ok_or(Error::Exhausted)?;
            let base = now.max(current.as_ref().and_then(|c| c.2).unwrap_or(0));
            (1, Some(base.checked_add(extend).ok_or(Error::Exhausted)?))
        }
    };
    tx.execute(
        "INSERT INTO player_inventory(player_id,sku_id,grant_kind,quantity,expires_at,acquired_at,updated_at)
         VALUES(?1,?2,?3,?4,?5,?6,?6)
         ON CONFLICT(player_id,sku_id) DO UPDATE SET
           quantity=excluded.quantity,expires_at=excluded.expires_at,updated_at=excluded.updated_at",
        params![player, sku_id, grant.kind(), quantity, expires_at, now],
    )?;
    read_item(tx, player, sku_id, now)?.ok_or(Error::Storage)
}

impl Store {
    /// Buy `quantity` units of an enabled catalog SKU with Lucci, once per
    /// `(player, idempotency_key)`. A replay with the same request returns the
    /// original ledger row; a different request with that key is a conflict.
    pub fn purchase_shop_item(
        &self,
        player_id: i64,
        catalog: &ShopCatalog,
        sku_id: &str,
        quantity: u32,
        key: &str,
        now: i64,
    ) -> Result<PurchaseReceipt> {
        catalog.validate()?;
        sku(sku_id)?;
        idempotency_key(key)?;
        let item = catalog.item(sku_id).ok_or(Error::InvalidInput)?;
        if quantity == 0 || (item.grant == ItemGrant::Permanent && quantity != 1) {
            return Err(Error::InvalidInput);
        }
        let cost = item
            .price_lucci
            .checked_mul(u64::from(quantity))
            .and_then(|v| i64::try_from(v).ok())
            .ok_or(Error::InvalidInput)?;
        let hash = request_hash(&json!({
            "v": 1, "op": "shop_purchase", "player_id": player_id, "sku_id": sku_id,
            "quantity": quantity, "price_lucci": item.price_lucci, "category": item.category,
            "grant": item.grant, "equip_slot": item.equip_slot,
        }));
        self.economy_tx(now, |tx| {
            Self::active_player(tx, player_id, now)?;
            if let Some(old) = Self::find_ledger(tx, player_id, "shop_purchase", key)? {
                if old.request_hash != hash {
                    return Err(Error::Conflict);
                }
                return Ok(PurchaseReceipt {
                    ledger_id: old.id,
                    player_id,
                    sku_id: sku_id.into(),
                    quantity,
                    lucci_spent: -old.lucci_delta,
                    replayed: true,
                    profile: Self::read_profile(tx, player_id)?,
                    item: read_item(tx, player_id, sku_id, now)?,
                });
            }
            if !item.enabled {
                return Err(Error::InvalidInput);
            }
            let profile = Self::read_profile(tx, player_id)?;
            let lucci = profile
                .lucci
                .checked_sub(cost)
                .ok_or(Error::InsufficientFunds)?;
            if lucci < 0 {
                return Err(Error::InsufficientFunds);
            }
            let granted = grant_item(tx, player_id, sku_id, item.grant, quantity, now)?;
            let ledger_id = Self::insert_ledger(
                tx,
                player_id,
                "shop_purchase",
                key,
                &hash,
                (0, -cost),
                None,
                Some(sku_id),
                now,
            )?;
            let profile = Self::write_profile(tx, &PlayerProfile { lucci, ..profile })?;
            Ok(PurchaseReceipt {
                ledger_id,
                player_id,
                sku_id: sku_id.into(),
                quantity,
                lucci_spent: cost,
                replayed: false,
                profile,
                item: Some(granted),
            })
        })
    }

    /// Audited operator item grant with an explicit grant rule (no Lucci).
    #[allow(clippy::too_many_arguments)]
    pub fn admin_grant_item(
        &self,
        admin: &Admin<'_>,
        player_id: i64,
        sku_id: &str,
        grant: ItemGrant,
        quantity: u32,
        key: &str,
        now: impl Into<Option<i64>>,
    ) -> Result<AdminGrantReceipt> {
        admin.validate()?;
        sku(sku_id)?;
        idempotency_key(key)?;
        grant.validate()?;
        if quantity == 0 || (grant == ItemGrant::Permanent && quantity != 1) {
            return Err(Error::InvalidInput);
        }
        let hash = request_hash(&json!({
            "v": 1, "op": "admin_grant_item", "player_id": player_id, "sku_id": sku_id,
            "quantity": quantity, "grant": grant,
        }));
        let request = hex(&self.random::<32>()?);
        let now = match now.into() {
            Some(now) => now,
            None => utc()?,
        };
        self.economy_tx(now, |tx| {
            let account = Self::granted_player(tx, player_id)?;
            if let Some(old) = Self::find_ledger(tx, player_id, "admin_grant", key)? {
                if old.request_hash != hash {
                    return Err(Error::Conflict);
                }
                return Ok(AdminGrantReceipt {
                    ledger_id: old.id,
                    player_id,
                    lucci_delta: 0,
                    sku_id: Some(sku_id.into()),
                    replayed: true,
                    profile: Self::read_profile(tx, player_id)?,
                    item: read_item(tx, player_id, sku_id, now)?,
                });
            }
            let granted = grant_item(tx, player_id, sku_id, grant, quantity, now)?;
            let ledger_id = Self::insert_ledger(
                tx,
                player_id,
                "admin_grant",
                key,
                &hash,
                (0, 0),
                None,
                Some(sku_id),
                now,
            )?;
            let profile = Self::write_profile(tx, &Self::read_profile(tx, player_id)?)?;
            Self::economy_audit(
                tx,
                &request,
                admin,
                "admin_grant_item",
                account,
                player_id,
                now,
            )?;
            Ok(AdminGrantReceipt {
                ledger_id,
                player_id,
                lucci_delta: 0,
                sku_id: Some(sku_id.into()),
                replayed: false,
                profile,
                item: Some(granted),
            })
        })
    }

    /// Equip an owned, non-expired SKU whose catalog slot is exactly `slot`,
    /// replacing whatever that slot held. Ownership comes from inventory, so a
    /// SKU later disabled for sale stays equippable by its owner.
    pub fn equip_item(
        &self,
        player_id: i64,
        catalog: &ShopCatalog,
        sku_id: &str,
        slot: &str,
        now: i64,
    ) -> Result<EquippedItem> {
        catalog.validate()?;
        sku(sku_id)?;
        equip_slot(slot)?;
        let item = catalog.item(sku_id).ok_or(Error::InvalidInput)?;
        if item.equip_slot.as_deref() != Some(slot) {
            return Err(Error::InvalidInput);
        }
        self.economy_tx(now, |tx| {
            Self::active_player(tx, player_id, now)?;
            let owned = read_item(tx, player_id, sku_id, now)?.ok_or(Error::Conflict)?;
            if !owned.active {
                return Err(Error::Conflict);
            }
            let current: Option<(String, i64)> = tx
                .query_row(
                    "SELECT sku_id,equipped_at FROM player_equipped WHERE player_id=?1 AND slot=?2",
                    params![player_id, slot],
                    |r| Ok((r.get(0)?, r.get(1)?)),
                )
                .optional()?;
            if let Some((current, equipped_at)) = current
                && current == sku_id
            {
                return Ok(EquippedItem {
                    slot: slot.into(),
                    sku_id: current,
                    equipped_at,
                });
            }
            tx.execute(
                "INSERT INTO player_equipped(player_id,slot,sku_id,equipped_at) VALUES(?1,?2,?3,?4)
                 ON CONFLICT(player_id,slot) DO UPDATE SET sku_id=excluded.sku_id,equipped_at=excluded.equipped_at",
                params![player_id, slot, sku_id, now],
            )?;
            Self::write_profile(tx, &Self::read_profile(tx, player_id)?)?;
            Ok(EquippedItem {
                slot: slot.into(),
                sku_id: sku_id.into(),
                equipped_at: now,
            })
        })
    }

    /// Clear one slot. Returns false when the slot was already empty.
    pub fn unequip_slot(&self, player_id: i64, slot: &str, now: i64) -> Result<bool> {
        equip_slot(slot)?;
        self.economy_tx(now, |tx| {
            Self::active_player(tx, player_id, now)?;
            let removed = tx.execute(
                "DELETE FROM player_equipped WHERE player_id=?1 AND slot=?2",
                params![player_id, slot],
            )? == 1;
            if removed {
                Self::write_profile(tx, &Self::read_profile(tx, player_id)?)?;
            }
            Ok(removed)
        })
    }

    /// Read-only inventory with each SKU's equipped slots; `active` is false for
    /// a timed item with `expires_at <= now`.
    pub fn list_inventory(&self, player_id: i64, now: i64) -> Result<Vec<InventoryItem>> {
        Self::player_exists(&self.db, player_id)?;
        let items: Vec<InventoryItem> = self
            .db
            .prepare(
                "SELECT sku_id,grant_kind,quantity,expires_at,acquired_at,updated_at
                 FROM player_inventory WHERE player_id=?1 ORDER BY sku_id",
            )?
            .query_map([player_id], |r| inventory_row(r, now))?
            .collect::<std::result::Result<_, _>>()?;
        items
            .into_iter()
            .map(|mut item| {
                item.equipped_slots = equipped_slots(&self.db, player_id, &item.sku_id)?;
                Ok(item)
            })
            .collect()
    }

    pub fn list_equipped(&self, player_id: i64) -> Result<Vec<EquippedItem>> {
        Self::player_exists(&self.db, player_id)?;
        Ok(self
            .db
            .prepare(
                "SELECT slot,sku_id,equipped_at FROM player_equipped WHERE player_id=?1 ORDER BY slot",
            )?
            .query_map([player_id], |r| {
                Ok(EquippedItem {
                    slot: r.get(0)?,
                    sku_id: r.get(1)?,
                    equipped_at: r.get(2)?,
                })
            })?
            .collect::<std::result::Result<_, _>>()?)
    }
}
