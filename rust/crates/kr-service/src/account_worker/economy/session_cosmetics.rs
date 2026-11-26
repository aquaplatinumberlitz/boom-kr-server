use super::EconomySettings;
use crate::accounts::{Error, Result, equip_slot};
use std::{collections::BTreeMap, sync::Arc};

/// Selections owned by one O1 session. The base character uses the catalog's
/// `character` slot; other slots are independent. Drop this value at logout.
/// This type holds no worker or database handle.
pub struct O1SessionCosmetics {
    settings: Arc<EconomySettings>,
    slots: BTreeMap<String, String>,
}

impl O1SessionCosmetics {
    pub fn new(settings: Arc<EconomySettings>) -> Self {
        Self {
            settings,
            slots: BTreeMap::new(),
        }
    }

    /// Select an enabled SKU for its catalog slot, replacing that slot only.
    pub fn equip(&mut self, sku_id: &str, slot: &str) -> Result<()> {
        let item = self
            .settings
            .catalog()
            .item(sku_id)
            .filter(|item| item.enabled && item.equip_slot.as_deref() == Some(slot))
            .ok_or(Error::InvalidInput)?;
        self.slots.insert(slot.into(), item.sku_id.clone());
        Ok(())
    }

    /// Clear a catalog slot. Returns false when that slot was already empty.
    pub fn unequip(&mut self, slot: &str) -> Result<bool> {
        equip_slot(slot)?;
        if !self
            .settings
            .catalog()
            .items
            .iter()
            .any(|item| item.equip_slot.as_deref() == Some(slot))
        {
            return Err(Error::InvalidInput);
        }
        Ok(self.slots.remove(slot).is_some())
    }

    pub fn equipped(&self, slot: &str) -> Option<&str> {
        self.slots.get(slot).map(String::as_str)
    }

    pub fn snapshot(&self) -> BTreeMap<String, String> {
        self.slots.clone()
    }

    pub fn clear(&mut self) {
        self.slots.clear();
    }
}
