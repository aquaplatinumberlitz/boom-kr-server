use super::*;

/// A use accepted by the server's match rules for the authenticated player.
/// The caller must retain the original key and digest when retrying.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MatchItemConsumptionRequest {
    pub player_id: i64,
    pub match_id: i64,
    pub sku_id: String,
    pub quantity: u32,
    pub idempotency_key: String,
    /// Lowercase hex SHA-256 of the server's canonical action, including the
    /// match and the item use. This digest must not come from client input.
    pub request_hash: String,
}

impl MatchItemConsumptionRequest {
    pub(crate) fn validate(&self) -> Result<()> {
        sku(&self.sku_id)?;
        idempotency_key(&self.idempotency_key)?;
        if self.player_id < 1
            || self.match_id < 1
            || self.quantity == 0
            || self.request_hash.len() != 64
            || !self
                .request_hash
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        {
            return Err(Error::InvalidInput);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct MatchItemConsumptionReceipt {
    pub id: i64,
    pub player_id: i64,
    pub match_id: i64,
    pub sku_id: String,
    pub consumed_qty: u32,
    /// Inventory quantity immediately after this action, even on replay.
    pub remaining_qty: i64,
    pub created_at: i64,
    pub replayed: bool,
}

impl Store {
    /// Apply an accepted action once per player and key. New uses require a
    /// running match and a journaled participant. The caller owns live session
    /// and gameplay checks, including whether the player is still in the room.
    pub fn consume_item_in_match(
        &self,
        request: &MatchItemConsumptionRequest,
        now: i64,
    ) -> Result<MatchItemConsumptionReceipt> {
        request.validate()?;
        let player = request.player_id;
        let sku = &request.sku_id;
        let tx = Transaction::new_unchecked(&self.db, TransactionBehavior::Immediate)?;
        if let Some((hash, receipt)) = tx
            .query_row(
                "SELECT request_hash,id,match_id,sku_id,consumed_qty,remaining_qty,created_at
                 FROM match_item_consumptions WHERE player_id=?1 AND idempotency_key=?2",
                params![player, request.idempotency_key],
                |r| {
                    Ok((
                        r.get::<_, String>(0)?,
                        MatchItemConsumptionReceipt {
                            id: r.get(1)?,
                            player_id: player,
                            match_id: r.get(2)?,
                            sku_id: r.get(3)?,
                            consumed_qty: r.get(4)?,
                            remaining_qty: r.get(5)?,
                            created_at: r.get(6)?,
                            replayed: true,
                        },
                    ))
                },
            )
            .optional()?
        {
            if hash != request.request_hash
                || receipt.match_id != request.match_id
                || receipt.sku_id != *sku
                || receipt.consumed_qty != request.quantity
            {
                return Err(Error::Conflict);
            }
            tx.commit()?;
            return Ok(receipt);
        }

        Self::clock(&tx, now)?;
        Self::active_player(&tx, player, now)?;
        let in_match: bool = tx.query_row(
            "SELECT EXISTS(SELECT 1 FROM matches m JOIN match_players p ON p.match_id=m.id
                           WHERE m.id=?1 AND m.state='running' AND p.player_id=?2)",
            params![request.match_id, player],
            |r| r.get(0),
        )?;
        if !in_match {
            return Err(Error::InvalidSession);
        }

        let (kind, quantity): (String, i64) = tx
            .query_row(
                "SELECT grant_kind,quantity FROM player_inventory WHERE player_id=?1 AND sku_id=?2",
                params![player, sku],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .optional()?
            .ok_or(Error::InsufficientQuantity)?;
        if kind != "stackable" {
            return Err(Error::InvalidInput);
        }
        let remaining_qty = quantity - i64::from(request.quantity);
        match remaining_qty.cmp(&0) {
            std::cmp::Ordering::Less => return Err(Error::InsufficientQuantity),
            std::cmp::Ordering::Equal => {
                tx.execute(
                    "DELETE FROM player_equipped WHERE player_id=?1 AND sku_id=?2",
                    params![player, sku],
                )?;
                tx.execute(
                    "DELETE FROM player_inventory WHERE player_id=?1 AND sku_id=?2",
                    params![player, sku],
                )?;
            }
            std::cmp::Ordering::Greater => {
                tx.execute(
                    "UPDATE player_inventory SET quantity=?3,updated_at=?4 WHERE player_id=?1 AND sku_id=?2",
                    params![player, sku, remaining_qty, now],
                )?;
            }
        }

        Self::write_profile(&tx, &Self::read_profile(&tx, player)?)?;
        tx.execute(
            "INSERT INTO match_item_consumptions
               (player_id,match_id,sku_id,idempotency_key,request_hash,consumed_qty,remaining_qty,created_at)
             VALUES(?1,?2,?3,?4,?5,?6,?7,?8)",
            params![
                player, request.match_id, sku, request.idempotency_key, request.request_hash,
                request.quantity, remaining_qty, now,
            ],
        )?;
        let id = tx.last_insert_rowid();
        tx.commit()?;
        Ok(MatchItemConsumptionReceipt {
            id,
            player_id: player,
            match_id: request.match_id,
            sku_id: sku.clone(),
            consumed_qty: request.quantity,
            remaining_qty,
            created_at: now,
            replayed: false,
        })
    }
}
