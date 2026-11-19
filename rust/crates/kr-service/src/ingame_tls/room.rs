//! Reuse the managed room kernel with the original in-game account handles.
//! No identity lookup, socket alias or new DB grant is introduced here.
use super::*;
use crate::{accounts::CharacterPreference, room_io, room_service};
use std::collections::BTreeSet;

#[derive(Default)]
pub(super) struct State {
    session: Option<room_io::Session>,
    pending: Vec<room_io::Batch>,
    known: BTreeSet<(u16, u32)>,
    preference: Option<CharacterPreference>,
    last_chat: Option<Instant>,
}
impl State {
    pub(super) fn admitted(&mut self, catalog: &room_service::Catalog) {
        self.known = catalog.tuples.clone();
    }
    pub(super) async fn packet(
        &mut self,
        input: &native::Input,
        driver: &mut Driver,
        lane: &mut Lane,
        send: &mut quinn::SendStream,
        resources: &Resources,
    ) -> io::Result<bool> {
        if !lane.in_lobby() {
            return Ok(false);
        }
        let opcode = input.opcode;
        if matches!(opcode, 81 | 82) {
            if self.session.is_some() {
                return Err(io::Error::other("native selection before completed return"));
            }
            driver.inspect().await?;
            // Multi-local room bodies need their own measured command/consumer
            // mapping; never silently admit just the first account of a pair.
            if driver.authorities().len() != 1 {
                return Err(io::Error::other("unmeasured multi-local room layout"));
            }
            let owner = &driver.authorities()[0];
            owner
                .authorize_activity()
                .await
                .map_err(|_| io::Error::other("native activity denied"))?;
            self.preference = if resources.persist_character {
                Some(
                    owner
                        .character_preference(None)
                        .await
                        .map_err(|_| io::Error::other("native preference denied"))?,
                )
            } else {
                None
            };
            let selection = resources
                .lobby
                .lock()
                .await
                .as_ref()
                .ok_or_else(|| io::Error::other("native lobby absent"))?
                .select(opcode, input.body(), Ipv4Addr::LOCALHOST)?;
            match selection {
                room_service::Selection::Full => {
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
                room_service::Selection::Joined(session) => {
                    if let Some(preference) = &self.preference {
                        session.restore_character(preference.character_key)?;
                    }
                    self.session = Some(session);
                }
            }
            return Ok(true);
        }
        let Some(session) = &self.session else {
            return Ok(false);
        };
        if opcode != 18 && !room_io::Session::handles(opcode) {
            return Ok(false);
        }
        session.wait_journal().await?;
        driver.inspect().await?;
        let owner = driver
            .authorities()
            .first()
            .cloned()
            .ok_or_else(|| io::Error::other("native room owner absent"))?;
        if !matches!(opcode, 59 | 79) {
            owner
                .authorize_activity()
                .await
                .map_err(|_| io::Error::other("native activity denied"))?;
        }
        if opcode == 18 {
            if self
                .last_chat
                .is_some_and(|at| at.elapsed() < Duration::from_millis(500))
            {
                return Err(io::Error::other("native room chat rate bound"));
            }
            session.room_chat(input.body())?;
            self.last_chat = Some(Instant::now());
        } else {
            if opcode == 99 {
                session.wait_peer_ready().await?;
                driver.inspect().await?;
            }
            if opcode == 92 && resources.persist_character {
                if let Some((membership, character)) = session.prepare_character(input.body())? {
                    let expected = self
                        .preference
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
                                return Err(io::Error::other(
                                    "native character persistence denied",
                                ));
                            }
                        }
                    };
                    self.preference = Some(stored);
                    session.persisted_character(membership, input.body())?;
                } else {
                    session.packet(opcode, input.body())?;
                }
            } else {
                session.packet(opcode, input.body())?;
            }
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
    ) -> io::Result<()> {
        if let Some(session) = &mut self.session {
            driver.inspect().await?;
            if !session.channel_current() {
                return Err(io::Error::other("native room channel retired"));
            }
            session.tick()?;
            session.pump_relay().await?;
            while let Ok(batch) = session.inbox.try_recv() {
                if self.pending.len() >= 16 {
                    return Err(io::Error::other("native room timer queue bound"));
                }
                self.pending.push(batch);
            }
        }
        self.pending.sort_by_key(|batch| batch.due);
        while self
            .pending
            .first()
            .is_some_and(|batch| batch.due <= Instant::now())
        {
            let mut batch = self.pending.remove(0);
            let session = self
                .session
                .as_ref()
                .ok_or_else(|| io::Error::other("native room session absent"))?;
            driver.inspect().await?;
            if !session.valid(&batch) {
                continue;
            }
            if let Some(ticket) = &batch.journal {
                ticket.wait().await?;
                driver.inspect().await?;
                if !session.valid(&batch) {
                    continue;
                }
            }
            if !session.prepare_delivery(&mut batch)? {
                continue;
            }
            for (_, reply) in &batch.packets {
                if !session.authorize_batch_source(&batch).await? {
                    break;
                }
                driver.inspect().await?;
                if !session.valid(&batch) {
                    return Err(io::Error::other("native room batch retired"));
                }
                let Some(reply) = session.project(reply)? else {
                    continue;
                };
                if reply.opcode == 91 {
                    continue;
                }
                driver
                    .lobby_reply(lane, send, reply.opcode, &reply.payload)
                    .await?;
            }
            session.finish(&batch)?;
            if batch.packets.iter().any(|(alias, _)| alias == "return241") {
                self.session.take();
                self.pending.clear();
                resources
                    .lobby
                    .lock()
                    .await
                    .as_ref()
                    .ok_or_else(|| io::Error::other("native lobby absent"))?
                    .returned()?;
            }
        }
        if self.session.is_none() {
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
