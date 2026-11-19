//! Bounded native login/channel exchange inside an already owned transport.
//! No listener is opened here. TLS admission is a separate required layer.
use super::{Cancel, LocalAuthority, LocalSession};
use kr_wire::{HEADER_LEN, Width, kr00967::AccountTemplate, write_uint};
use std::{io, sync::Arc, time::Duration};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use zeroize::Zeroizing;

/// No Debug or serialization: even obfuscated credential bytes are secrets.
pub struct Input {
    pub opcode: u32,
    payload: Zeroizing<Vec<u8>>,
}
impl Input {
    pub fn new(opcode: u32, payload: Zeroizing<Vec<u8>>) -> io::Result<Self> {
        if payload.len() > 512 {
            return Err(io::Error::other("native input bound"));
        }
        Ok(Self { opcode, payload })
    }
}

/// Secret-owning reader. CRC failure also erases the decrypted allocation;
/// the general diagnostic Packet/Receiver types are not used for passwords.
pub async fn read<S: AsyncRead + Unpin>(stream: &mut S, sequence: &mut u32) -> io::Result<Input> {
    tokio::time::timeout(Duration::from_secs(2), async {
        let mut frame = Zeroizing::new(vec![0; HEADER_LEN]);
        stream.read_exact(&mut frame).await?;
        let size = u32::from_le_bytes(frame[13..17].try_into().expect("fixed header")) as usize;
        if size > 512 {
            return Err(io::Error::other("native frame bound"));
        }
        frame.resize(HEADER_LEN + size + 1, 0);
        stream.read_exact(&mut frame[HEADER_LEN..]).await?;
        let payload = Zeroizing::new(kr_wire::crypt(
            &frame[HEADER_LEN..frame.len() - 1],
            *sequence,
            true,
        ));
        if kr_wire::crc8(*sequence, &payload) != frame[frame.len() - 1] {
            return Err(io::Error::other("native checksum"));
        }
        let opcode = u32::from_le_bytes(frame[5..9].try_into().expect("fixed header"));
        *sequence = sequence.wrapping_add(3);
        Input::new(opcode, payload)
    })
    .await
    .map_err(|_| io::Error::other("native read deadline"))?
}

/// Bodies come from the same validated measured fixtures used by M5. Unknown
/// fields retain their provenance; this adapter changes only owned auth rows
/// and the measured per-slot readiness notification.
#[derive(Clone)]
pub struct Templates {
    pub account: Arc<AccountTemplate>,
    pub header: [u8; HEADER_LEN],
    pub netinfo_header: [u8; HEADER_LEN],
    pub flags_header: [u8; HEADER_LEN],
    pub redirect_header: [u8; HEADER_LEN],
    pub channel_header: [u8; HEADER_LEN],
    pub netinfo: Vec<u8>,
    pub redirect: Vec<u8>,
    pub channel: Vec<u8>,
}
impl Templates {
    pub fn startup_frames(profile: &crate::Validated) -> (Vec<u8>, Vec<u8>) {
        (profile.bootstrap.clone(), profile.wcp_second.clone())
    }
    /// Explicit pinned diagnostic handoff for a loopback acceptance run. It
    /// never rewrites an operator's fixture or accepts an unpinned endpoint.
    pub fn local_handoff(
        &mut self,
        netinfo: &crate::LobbyPacketPin,
        redirect: &crate::LobbyPacketPin,
    ) -> io::Result<()> {
        for (pin, opcode) in [(netinfo, 15), (redirect, 7)] {
            let raw = pin.fixture.load()?;
            let packet = kr_wire::decode(
                &raw,
                pin.sequence
                    .ok_or_else(|| io::Error::other("native fixture sequence"))?,
            )
            .map_err(io::Error::other)?;
            if packet.opcode != opcode {
                return Err(io::Error::other("native fixture opcode"));
            }
            let header = raw[..HEADER_LEN].try_into().map_err(io::Error::other)?;
            if opcode == 15 {
                self.netinfo_header = header;
                self.netinfo = packet.payload;
            } else {
                self.redirect_header = header;
                self.redirect = packet.payload;
            }
        }
        Ok(())
    }
    pub fn from_profile(profile: &crate::Validated) -> io::Result<Self> {
        let bound = profile
            .bound
            .as_ref()
            .ok_or_else(|| io::Error::other("native account profile required"))?;
        let account = bound
            .account_template
            .clone()
            .ok_or_else(|| io::Error::other("native database projection required"))?;
        // Reuse the already validated zero-profile source, exactly as M5b does;
        // runtime identities are always projected from DB, never actor aliases.
        let actor = bound
            .actors
            .values()
            .min_by_key(|actor| actor.identity)
            .ok_or_else(|| io::Error::other("native template source missing"))?;
        let get = |name: &str| {
            actor
                .packets
                .get(name)
                .ok_or_else(|| io::Error::other("native template missing"))
        };
        Ok(Self {
            account,
            header: get("auth_A")?.header,
            netinfo_header: get("netinfo")?.header,
            flags_header: get("flags")?.header,
            redirect_header: get("redirect")?.header,
            channel_header: get("channel")?.header,
            netinfo: get("netinfo")?.payload.clone(),
            redirect: get("redirect")?.payload.clone(),
            channel: get("channel")?.payload.clone(),
        })
    }
}
#[derive(Clone, Copy, PartialEq, Eq)]
enum Phase {
    Login,
    Info,
    Redirect,
    LoginDone,
    Channel,
    Bound,
    Denied,
}
pub struct Lane {
    connection: u64,
    phase: Phase,
    send_sequence: u32,
    ignored: usize,
}
impl Lane {
    pub fn login(connection: u64) -> io::Result<Self> {
        Self::new(connection, Phase::Login)
    }
    pub fn channel(connection: u64) -> io::Result<Self> {
        Self::new(connection, Phase::Channel)
    }
    fn new(connection: u64, phase: Phase) -> io::Result<Self> {
        if connection == 0 {
            return Err(io::Error::other("native connection identity"));
        }
        Ok(Self {
            connection,
            phase,
            send_sequence: 43,
            ignored: 0,
        })
    }
}
pub struct Driver {
    session: LocalSession,
    templates: Templates,
    login_connection: u64,
    local_count: usize,
    authorities: Vec<LocalAuthority>,
}
impl Driver {
    pub fn new(session: LocalSession, templates: Templates) -> io::Result<Self> {
        if [&templates.netinfo, &templates.redirect, &templates.channel]
            .iter()
            .any(|body| body.is_empty() || body.len() > 4096)
        {
            return Err(io::Error::other("native reply template bound"));
        }
        let login_connection = session
            .shared
            .lock()
            .map_err(|_| io::Error::other("native owner"))?
            .login;
        Ok(Self {
            session,
            templates,
            login_connection,
            local_count: 0,
            authorities: Vec::new(),
        })
    }
    pub fn authorities(&self) -> &[LocalAuthority] {
        &self.authorities
    }
    pub async fn inspect(&mut self) -> io::Result<()> {
        self.session
            .inspect()
            .await
            .map_err(|_| io::Error::other("native owner denied"))
    }
    async fn write<S: AsyncWrite + Unpin>(
        &self,
        lane: &mut Lane,
        writer: &mut S,
        opcode: u32,
        body: &[u8],
    ) -> io::Result<()> {
        let header = match opcode {
            4 => &self.templates.header,
            15 => &self.templates.netinfo_header,
            62 => &self.templates.flags_header,
            7 => &self.templates.redirect_header,
            17 => &self.templates.channel_header,
            _ => return Err(io::Error::other("native reply opcode denied")),
        };
        let bytes =
            kr_wire::encode(header, opcode, body, lane.send_sequence).map_err(io::Error::other)?;
        tokio::time::timeout(Duration::from_secs(2), writer.write_all(&bytes))
            .await
            .map_err(|_| io::Error::other("native write deadline"))??;
        lane.send_sequence = lane.send_sequence.wrapping_add(3);
        Ok(())
    }
    pub async fn packet<S: AsyncWrite + Unpin>(
        &mut self,
        lane: &mut Lane,
        input: Input,
        writer: &mut S,
        ttl: i64,
        idle: Option<i64>,
    ) -> io::Result<()> {
        // Cancellation while a reply is pending must revoke authority too;
        // checking only returned write errors leaves the owner alive on abort.
        let mut cancel = Cancel(self.session.shared.clone(), true);
        let result = self.exchange(lane, input, writer, ttl, idle).await;
        if result.is_ok() {
            cancel.1 = false;
        }
        result
    }
    async fn exchange<S: AsyncWrite + Unpin>(
        &mut self,
        lane: &mut Lane,
        input: Input,
        writer: &mut S,
        ttl: i64,
        idle: Option<i64>,
    ) -> io::Result<()> {
        let body = &input.payload;
        match (lane.phase, input.opcode) {
            (Phase::Login, opcode @ (1 | 62)) if lane.connection == self.login_connection => {
                let admitted = if opcode == 1 {
                    self.session.login(body, ttl, idle).await
                } else {
                    self.session.trial(body, ttl, idle).await
                };
                let rows = match admitted {
                    Ok(rows) => rows,
                    Err(_) => {
                        lane.phase = Phase::Denied;
                        self.write(
                            lane,
                            writer,
                            4,
                            &write_uint(1, Width::U8).map_err(io::Error::other)?,
                        )
                        .await?;
                        return Err(io::Error::other("native login denied"));
                    }
                };
                let names = rows
                    .iter()
                    .map(|(player, _)| {
                        kr_wire::kr00967::text949x::encode_name(&player.display_name)
                    })
                    .collect::<io::Result<Vec<_>>>()?;
                let roster = rows
                    .iter()
                    .zip(&names)
                    .map(|((player, echo), name)| (player.wire_id, name.as_slice(), *echo))
                    .collect::<Vec<_>>();
                let reply = self.templates.account.project_local_auth(&roster)?;
                self.write(lane, writer, 4, &reply).await?;
                self.local_count = rows.len();
                lane.phase = Phase::Info;
            }
            (Phase::Info, 9) if lane.connection == self.login_connection && body.len() == 8 => {
                self.session
                    .inspect()
                    .await
                    .map_err(|_| io::Error::other("native owner denied"))?;
                self.write(lane, writer, 15, &self.templates.netinfo)
                    .await?;
                for slot in 0..self.local_count {
                    let mut flags = write_uint(1, Width::U8).map_err(io::Error::other)?;
                    flags.extend(write_uint(slot as u64, Width::U8).map_err(io::Error::other)?);
                    flags.extend(write_uint(2, Width::U8).map_err(io::Error::other)?);
                    self.write(lane, writer, 62, &flags).await?;
                }
                lane.phase = Phase::Redirect;
            }
            (Phase::Redirect, 5) if lane.connection == self.login_connection && body.len() == 9 => {
                self.session
                    .inspect()
                    .await
                    .map_err(|_| io::Error::other("native owner denied"))?;
                self.write(lane, writer, 7, &self.templates.redirect)
                    .await?;
                self.session
                    .redirected(lane.connection)
                    .map_err(|_| io::Error::other("native handoff denied"))?;
                lane.phase = Phase::LoginDone;
            }
            (Phase::Channel, 6) => {
                self.authorities = self
                    .session
                    .channel(body, lane.connection)
                    .await
                    .map_err(|_| io::Error::other("native channel denied"))?;
                self.write(lane, writer, 17, &self.templates.channel)
                    .await?;
                lane.phase = Phase::Bound;
            }
            _ if lane.phase != Phase::Denied
                && matches!(
                    (input.opcode, body.len()),
                    (60, 20) | (15, 31) | (744, 4) | (745, 0) | (40, 1)
                ) =>
            {
                if lane.ignored >= 64 {
                    return Err(io::Error::other("native ancillary bound"));
                }
                // Pre-auth or foreign lanes cannot use ancillary traffic to
                // borrow a logged-in owner's lifetime or renew idle TTL.
                if lane.connection != self.login_connection
                    && !self
                        .authorities
                        .iter()
                        .any(|a| a.connection() == lane.connection)
                {
                    return Err(io::Error::other("native foreign lane"));
                }
                self.session
                    .inspect()
                    .await
                    .map_err(|_| io::Error::other("native owner denied"))?;
                lane.ignored += 1;
            }
            _ => return Err(io::Error::other("native stage or shape denied")),
        }
        Ok(())
    }
    pub async fn disconnected(&mut self, lane: &Lane) -> io::Result<bool> {
        self.session
            .disconnected(lane.connection)
            .await
            .map_err(|_| io::Error::other("native cleanup failed"))
    }
    pub async fn finish(self) -> io::Result<()> {
        self.session
            .finish()
            .await
            .map_err(|_| io::Error::other("native cleanup failed"))
    }
}

#[cfg(test)]
mod tests;
