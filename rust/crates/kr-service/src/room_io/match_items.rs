//! Trusted item use boundary shared by both managed room transports.
use super::{Phase, Session, Shared, err};
use crate::{
    account_serving::MatchItems,
    account_worker::{Error, MatchItemConsumptionOutcome, Result},
    accounts::{self, MatchItemConsumptionRequest},
};

pub(crate) fn install_match_items(shared: &Shared, items: MatchItems) -> std::io::Result<()> {
    let mut room = shared.lock().map_err(|_| err("room unavailable"))?;
    if room.match_items.is_some() || !room.channels.is_empty() || room.public_key.is_none() {
        return Err(err("match item install requires pristine managed room"));
    }
    room.match_items = Some(items);
    Ok(())
}

const STALE: Error = Error::Backend(accounts::Error::InvalidSession);

impl Session {
    /// Call only for a server-approved use of a carried item, with an action
    /// ID assigned by the match and kept on retry. Match rules own effects,
    /// usage limits and O1 deduplication. No KR packet calls this hook yet.
    #[allow(
        dead_code,
        reason = "the KR inventory action route still needs capture"
    )]
    pub(crate) async fn consume_item_in_match(
        &self,
        round: u64,
        action_id: u64,
        slot: &str,
        sku: &str,
        quantity: u32,
    ) -> Result<MatchItemConsumptionOutcome> {
        if action_id == 0 || !(1..=64).contains(&sku.len()) {
            return Err(Error::InvalidInput);
        }
        accounts::equip_slot(slot).map_err(|_| Error::InvalidInput)?;
        let (items, authority, member, match_id, token) = {
            let room = self.shared.lock().map_err(|_| Error::WorkerFault)?;
            let items = room.match_items.as_ref().ok_or(Error::InvalidInput)?;
            let member = room
                .member(self.identity, self.connection)
                .map_err(|_| STALE)?
                .key;
            if room.model.phase != (Phase::Armed { round })
                || (room.model.player_round() && !room.model.participants.contains_key(&member))
            {
                return Err(STALE);
            }
            let authority = room
                .channels
                .get(&self.identity)
                .and_then(|channel| channel.relay_authority.as_ref())
                .filter(|owner| owner.owns(self.identity, self.connection))
                .ok_or(STALE)?;
            let (match_id, token) = if items.access.unlock_all {
                let now = accounts::utc().map_err(Error::Backend)?;
                if !authority.o1_current(now) {
                    return Err(STALE);
                }
                (i64::try_from(round).map_err(|_| Error::InvalidInput)?, None)
            } else {
                let id = room
                    .journal
                    .as_ref()
                    .and_then(|journal| journal.running_id(round))
                    .ok_or(STALE)?;
                (id, Some(authority.item_use_token().map_err(|_| STALE)?))
            };
            (items.clone(), authority.clone(), member, match_id, token)
        };
        let key = format!("match-use:{match_id}:{action_id}");
        let player = authority.player().player_id;
        let canonical = serde_json::to_vec(&(&key, player, slot, sku, quantity))
            .map_err(|_| Error::InvalidInput)?;
        let request = MatchItemConsumptionRequest {
            player_id: player,
            match_id,
            sku_id: sku.into(),
            quantity,
            idempotency_key: key,
            request_hash: crate::sha256_hex(&canonical),
        };
        let outcome = if let Some(token) = token {
            items
                .service
                .consume_equipped_item_in_match(request, token, slot)
                .await?
        } else {
            items
                .service
                .consume_item_in_match(&request, items.access)
                .await?
        };
        // A debit already committed may have raced teardown. Never apply its
        // gameplay effect to a later visit or round.
        let room = self.shared.lock().map_err(|_| Error::WorkerFault)?;
        let current = if items.access.unlock_all {
            authority.o1_current(accounts::utc().map_err(Error::Backend)?)
        } else {
            authority.current()
        };
        if room.model.phase != (Phase::Armed { round }) || !room.model.current(member) || !current {
            return Err(STALE);
        }
        Ok(outcome)
    }
}

#[cfg(test)]
#[path = "../f4bc_runtime_tests.rs"]
mod tests;
