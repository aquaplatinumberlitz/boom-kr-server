//! Reuse the managed room kernel with the original in-game account handles.
//! No identity lookup, socket alias or new DB grant is introduced here.
use super::*;
use crate::{accounts::CharacterPreference, room_io, room_service};
use std::collections::BTreeSet;

/// Choices name a local ordinal, never a DB or room identity. Translate only
/// the typed selector before passing the measured single-member envelope to
/// the existing policy/authority checks.
fn local_command(opcode: u32, body: &[u8], count: usize) -> io::Result<(usize, Option<Vec<u8>>)> {
    if count == 1 {
        return Ok((0, None));
    }
    use kr_wire::{Width, kr00967 as wire, write_uint};
    match opcode {
        92 => {
            let (index, character) = wire::local_character92(body, count)?;
            let mut body = write_uint(0, Width::U8).map_err(io::Error::other)?;
            body.extend(write_uint(character.into(), Width::U16).map_err(io::Error::other)?);
            Ok((index, Some(body)))
        }
        97 => {
            let (index, team) = wire::local_team97(body, count)?;
            Ok((
                index,
                Some(write_uint(team.into(), Width::U8).map_err(io::Error::other)?),
            ))
        }
        // Room-wide control and bounded observations without an actor index.
        59 | 86 | 96 | 99 | 104 | 122 | 123 | 142 | 167 | 190 | 202 | 266 | 528 | 585 => {
            Ok((0, None))
        }
        _ => Err(io::Error::other("unmeasured multi-local room command")),
    }
}

// Preserve the original event fields while identifying the channel and the
// exact managed room. The controller can then distinguish host-only setup
// from two streams that actually joined the same room.
fn emit_room_gate(
    resources: &Resources,
    session: &room_io::Session,
    stream_id: u64,
    mut event: serde_json::Value,
) {
    event["stream_id"] = serde_json::json!(stream_id);
    if let Some(room) = session.public_room_key() {
        event["room_id"] = serde_json::json!(room.id.0);
        event["room_session"] = serde_json::json!(room.session);
        event["generation"] = serde_json::json!(room.generation);
    }
    let _ = resources.events.try_send(event);
}

#[derive(Default)]
pub(super) struct State {
    sessions: Vec<room_io::Session>,
    pending: Vec<(usize, room_io::Batch)>,
    known: BTreeSet<(u16, u32)>,
    preferences: Vec<Option<CharacterPreference>>,
    last_chat: Option<Instant>,
    lobby_chat: Option<lobby_chat::Visit>,
}
impl State {
    fn owned_slot(&self, slot: u8) -> io::Result<usize> {
        for (i, session) in self.sessions.iter().enumerate() {
            if session.owns_slot(slot)? {
                return Ok(i);
            }
        }
        Err(io::Error::other(
            "room slot does not belong to this transport",
        ))
    }

    fn visit_lobby(&mut self, driver: &Driver, resources: &Resources) -> io::Result<()> {
        // Two-principal transports share one in-game chat box attributed to
        // the primary local player (`authorities[0]`).
        if resources.patched_chat
            && let Some(owner) = driver.authorities().first()
        {
            self.lobby_chat = Some(resources.lobby_chat.enter(owner.clone())?);
        }
        Ok(())
    }

    pub(super) fn admitted(
        &mut self,
        catalog: &room_service::Catalog,
        driver: &Driver,
        resources: &Resources,
    ) -> io::Result<()> {
        self.known = catalog.tuples.clone();
        self.visit_lobby(driver, resources)
    }
    pub(super) async fn packet(
        &mut self,
        input: &native::Input,
        driver: &mut Driver,
        lane: &mut Lane,
        send: &mut quinn::SendStream,
        resources: &Resources,
        stream_id: u64,
    ) -> io::Result<bool> {
        if !lane.in_lobby() {
            return Ok(false);
        }
        let opcode = input.opcode;
        if matches!(opcode, 81 | 82) {
            if !self.sessions.is_empty() {
                return Err(io::Error::other("native selection before completed return"));
            }
            driver.inspect().await?;
            self.preferences.clear();
            for owner in driver.authorities() {
                owner
                    .authorize_activity()
                    .await
                    .map_err(Driver::authority_error)?;
                self.preferences.push(if resources.persist_character {
                    Some(
                        owner
                            .character_preference(None)
                            .await
                            .map_err(|_| io::Error::other("native preference denied"))?,
                    )
                } else {
                    None
                });
            }
            let selection = resources
                .lobby
                .lock()
                .await
                .as_ref()
                .ok_or_else(|| io::Error::other("native lobby absent"))?
                .select_roster(opcode, input.body(), Ipv4Addr::LOCALHOST)?;
            match selection {
                room_service::LocalSelection::Full => {
                    driver
                        .lobby_reply(
                            lane,
                            send,
                            87,
                            &kr_wire::write_uint(2, kr_wire::Width::U8)
                                .map_err(io::Error::other)?,
                        )
                        .await?;
                }
                room_service::LocalSelection::Joined(sessions) => {
                    self.lobby_chat = None;
                    for (session, preference) in sessions.iter().zip(&self.preferences) {
                        if let Some(preference) = preference {
                            session.restore_character(preference.character_key)?;
                        }
                    }
                    self.sessions = sessions;
                    if let Some(session) = self.sessions.first() {
                        emit_room_gate(
                            resources,
                            session,
                            stream_id,
                            serde_json::json!({
                                "event": "native_tls_room_selected", "opcode": opcode, "players": self.sessions.len()
                            }),
                        );
                    }
                }
            }
            return Ok(true);
        }
        let count = self.sessions.len();
        if opcode == 18 && resources.patched_chat {
            if self
                .last_chat
                .is_some_and(|at| at.elapsed() < Duration::from_millis(500))
            {
                return Err(io::Error::other("native chat rate bound"));
            }
            driver.inspect().await?;
            match self.sessions.first() {
                None => {
                    self.lobby_chat
                        .as_ref()
                        .ok_or_else(|| io::Error::other("native chat has no lobby visit"))?
                        .send(input.body())
                        .await?;
                }
                Some(session) => {
                    session.wait_journal().await?;
                    driver.inspect().await?;
                    driver.authorities()[0]
                        .authorize_activity()
                        .await
                        .map_err(Driver::authority_error)?;
                    session.room_chat(input.body())?;
                }
            }
            self.last_chat = Some(Instant::now());
            return Ok(true);
        }
        if count == 0 || !room_io::Session::handles(opcode) {
            return Ok(false);
        }
        driver.inspect().await?;
        let (index, normalized) = match (count, opcode) {
            (2, 118) => {
                let slot = crate::bombs::Request::parse(input.body())?.owner_slot();
                (self.owned_slot(slot)?, None)
            }
            (2, 124 | 127) => {
                let slot = kr_wire::read_uint(
                    input
                        .body()
                        .get(1..2)
                        .ok_or_else(|| io::Error::other("missing room slot"))?,
                    kr_wire::Width::U8,
                )
                .map_err(io::Error::other)? as u8;
                (self.owned_slot(slot)?, None)
            }
            _ => local_command(opcode, input.body(), count)?,
        };
        let body = normalized.as_deref().unwrap_or(input.body());
        let session = &self.sessions[index];
        session.wait_journal().await?;
        driver.inspect().await?;
        let owner = driver.authorities()[index].clone();
        if !matches!(opcode, 59 | 79 | 528) {
            owner
                .authorize_activity()
                .await
                .map_err(Driver::authority_error)?;
        }
        if count == 2 && matches!(opcode, 585 | 86 | 99 | 104) {
            driver.authorities()[1]
                .authorize_activity()
                .await
                .map_err(Driver::authority_error)?;
            if matches!(opcode, 585 | 86) {
                session.local_waiting_room(opcode, body)?;
                let _ = resources.events.try_send(serde_json::json!({
                    "event": "native_tls_room_request", "opcode": opcode, "players": count
                }));
                return Ok(true);
            }
        }
        if opcode == 99 {
            session.wait_peer_ready().await?;
            driver.inspect().await?;
        }
        let observation = if opcode == 92 && resources.persist_character {
            if let Some((membership, character)) = session.prepare_character(body)? {
                let expected = self.preferences[index]
                    .as_ref()
                    .ok_or_else(|| io::Error::other("native character preference absent"))?
                    .revision;
                let deadline = Instant::now() + Duration::from_secs(2);
                let stored = loop {
                    match owner
                        .character_preference(Some((character, expected)))
                        .await
                    {
                        Ok(stored) => break stored,
                        Err(crate::account_bridge::Error::Backend(
                            crate::account_worker::Error::Busy
                            | crate::account_worker::Error::QueueExpired
                            | crate::account_worker::Error::Backend(crate::accounts::Error::Busy),
                        )) if Instant::now() < deadline => {
                            tokio::time::sleep(Duration::from_millis(20)).await
                        }
                        Err(_) => {
                            return Err(io::Error::other("native character persistence denied"));
                        }
                    }
                };
                self.preferences[index] = Some(stored);
                session.persisted_character(membership, body)?
            } else {
                session.packet(opcode, body)?
            }
        } else {
            session.packet(opcode, body)?
        };
        if !observation.character_stats.is_empty() {
            emit_room_gate(
                resources,
                session,
                stream_id,
                serde_json::json!({
                    "event": "native_tls_character_stats", "opcode": opcode,
                    "players": observation.character_stats, "pickup": observation.pickup,
                }),
            );
        }
        let _ = resources
            .events
            .try_send(serde_json::json!({"event":"native_tls_room_request","opcode":opcode}));
        Ok(true)
    }

    pub(super) async fn poll(
        &mut self,
        driver: &mut Driver,
        lane: &mut Lane,
        send: &mut quinn::SendStream,
        resources: &Resources,
        stream_id: u64,
    ) -> io::Result<()> {
        let queue_bound = 16 * self.sessions.len();
        if let Some(primary) = self.sessions.first() {
            driver
                .inspect()
                .await
                .map_err(|error| room_poll_failure(error, "authority", None))?;
            if self
                .sessions
                .iter()
                .any(|session| !session.channel_current())
            {
                return Err(room_poll_failure(
                    io::Error::other("native room channel retired"),
                    "membership",
                    None,
                ));
            }
            primary
                .tick()
                .map_err(|error| room_poll_failure(error, "tick", None))?;
            primary
                .pump_relay()
                .await
                .map_err(|error| room_poll_failure(error, "relay", None))?;
            for (index, session) in self.sessions.iter_mut().enumerate() {
                while let Ok(batch) = session.inbox.try_recv() {
                    if self.pending.len() >= queue_bound {
                        return Err(room_poll_failure(
                            io::Error::other("native room timer queue bound"),
                            "queue",
                            None,
                        ));
                    }
                    self.pending.push((index, batch));
                }
            }
        }
        self.pending.sort_by_key(|(_, batch)| batch.due);
        while self
            .pending
            .first()
            .is_some_and(|(_, batch)| batch.due <= Instant::now())
        {
            let (index, mut batch) = self.pending.remove(0);
            let session = &self.sessions[index];
            driver
                .inspect()
                .await
                .map_err(|error| room_poll_failure(error, "authority", None))?;
            if !session.valid(&batch) {
                continue;
            }
            if let Some(ticket) = &batch.journal {
                ticket
                    .wait()
                    .await
                    .map_err(|error| room_poll_failure(error, "journal", None))?;
                driver
                    .inspect()
                    .await
                    .map_err(|error| room_poll_failure(error, "authority", None))?;
                if !session.valid(&batch) {
                    continue;
                }
            }
            if !session
                .prepare_delivery(&mut batch)
                .map_err(|error| room_poll_failure(error, "prepare_batch", None))?
            {
                continue;
            }
            for (label, reply) in &batch.packets {
                if !session
                    .authorize_batch_source(&batch)
                    .await
                    .map_err(|error| room_poll_failure(error, "batch_source", Some(reply.opcode)))?
                {
                    break;
                }
                driver
                    .inspect()
                    .await
                    .map_err(|error| room_poll_failure(error, "authority", None))?;
                if !session.valid(&batch) {
                    return Err(room_poll_failure(
                        io::Error::other("native room batch retired"),
                        "membership",
                        Some(reply.opcode),
                    ));
                }
                let Some(reply) = session
                    .project(reply)
                    .map_err(|error| room_poll_failure(error, "project", Some(reply.opcode)))?
                else {
                    continue;
                };
                if reply.opcode == 91 {
                    continue;
                }
                driver
                    .lobby_reply(lane, send, reply.opcode, &reply.payload)
                    .await
                    .map_err(|error| room_poll_failure(error, "write", Some(reply.opcode)))?;
                let event = match (reply.opcode, label.as_str()) {
                    (110, "ready110") => Some("native_tls_room_ready_written"),
                    (121, "start121") => Some("native_tls_room_start_written"),
                    (153, "peer153") => Some("native_tls_room_peer_written"),
                    _ => None,
                };
                if let Some(event) = event {
                    let _ = resources.events.try_send(serde_json::json!({
                        "event": event,
                        "opcode": reply.opcode,
                        "stream_id": stream_id,
                    }));
                }
            }
            session
                .finish(&batch)
                .map_err(|error| room_poll_failure(error, "finish_batch", None))?;
            if batch.outcome_complete.is_some() {
                // Input drivers can wait for each recipient's room refresh
                // instead of treating the result or the return packet as Idle.
                let _ = resources.events.try_send(serde_json::json!({
                    "event": "native_tls_room_return_refreshed",
                    "stream_id": stream_id,
                }));
            }
            if batch.setup.is_some() {
                emit_room_gate(
                    resources,
                    session,
                    stream_id,
                    serde_json::json!({
                        "event": "native_tls_room_setup_written", "players": self.sessions.len()
                    }),
                );
            }
            if batch.packets.iter().any(|(alias, _)| alias == "return241") {
                self.sessions.clear();
                self.pending.clear();
                resources
                    .lobby
                    .lock()
                    .await
                    .as_ref()
                    .ok_or_else(|| io::Error::other("native lobby absent"))?
                    .returned()?;
                self.visit_lobby(driver, resources)?;
            }
        }
        if self.sessions.is_empty() {
            if let Some(visit) = self.lobby_chat.as_mut() {
                // One tick drains at most the queue bound even if new
                // messages arrive while checking the source and recipient.
                for _ in 0..lobby_chat::QUEUE {
                    let Some(message) = visit.receive() else {
                        break;
                    };
                    if message.source_is_current().await {
                        driver
                            .inspect()
                            .await
                            .map_err(|error| room_poll_failure(error, "authority", None))?;
                        if message.source_still_current() {
                            driver.lobby_reply(lane, send, 28, &message.body).await?;
                        }
                    }
                }
            }
            let mut lobby = resources.lobby.lock().await;
            let lobby = lobby
                .as_mut()
                .ok_or_else(|| io::Error::other("native lobby absent"))?;
            if lobby.changed()? {
                let catalog = lobby.catalog_since(&self.known)?;
                for (opcode, body) in catalog.packets {
                    driver.lobby_reply(lane, send, opcode, &body).await?;
                }
                self.known = catalog.tuples;
            }
        }
        Ok(())
    }
}
