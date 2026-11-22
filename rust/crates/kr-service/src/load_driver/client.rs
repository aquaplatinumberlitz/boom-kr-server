// The login and room sequence follows ingame_tls/tests.rs and the managed
// player contract. This client never opens legacy plaintext listeners.
use super::{Account, Config, failure};
use kr_tunnel::prelogin;
use kr_wire::{Packet, Width, Width::*, read_uint, write_uint};
use std::{
    io,
    net::{Ipv4Addr, SocketAddrV4},
    time::Duration,
};
use tokio::io::AsyncReadExt;
use tokio::{net::UdpSocket, sync::watch, task::JoinSet, time::timeout};
use zeroize::Zeroizing;

const DEADLINE: Duration = Duration::from_secs(20);

fn fields(items: &[(u64, Width)]) -> Vec<u8> {
    items
        .iter()
        .flat_map(|&(value, width)| write_uint(value, width).expect("bounded wire field"))
        .collect()
}

fn get(body: &[u8], at: usize, width: Width) -> io::Result<u64> {
    let size = match width {
        U8 => 1,
        U16 => 2,
        U32 => 4,
        U64 => 8,
    };
    read_uint(
        body.get(at..at + size)
            .ok_or_else(|| failure("truncated reply"))?,
        width,
    )
    .map_err(io::Error::other)
}

fn claims(auth: &[u8], expected_name: &str) -> io::Result<Vec<u8>> {
    let len = get(auth, 29, U16)? as usize;
    let name = auth
        .get(31..31 + len)
        .ok_or_else(|| failure("truncated account profile"))?;
    let name = kr_wire::kr00967::decode_bytes(name).map_err(io::Error::other)?;
    let id = get(auth, 25, U32)?;
    let echo = get(auth, 86 + len, U32)?;
    if name != expected_name.as_bytes() || id == 0 || echo == 0 {
        return Err(failure("account profile mismatch"));
    }
    let mut body = fields(&[(1, U8), (255, U8), (len as u64, U16)]);
    body.extend(kr_wire::kr00967::encode_bytes(&name).map_err(io::Error::other)?);
    body.extend(fields(&[(id, U32), (echo, U32), (1, U64), (0, U8)]));
    body.extend(fields(&[
        (0, U8),
        (1, U8),
        (0, U32),
        (0, U32),
        (0, U8),
        (0, U32),
    ]));
    Ok(body)
}

struct Wire {
    tx: quinn::SendStream,
    rx: quinn::RecvStream,
    send_seq: u32,
    recv_seq: u32,
}

impl Wire {
    async fn open(conn: &quinn::Connection, port: u16) -> io::Result<Self> {
        let (tx, mut rx) = prelogin::open(conn, port).await?;
        rx.read_exact(&mut [0; 15])
            .await
            .map_err(io::Error::other)?;
        Ok(Self {
            tx,
            rx,
            send_seq: 40,
            recv_seq: 43,
        })
    }

    async fn send(&mut self, op: u32, body: &[u8]) -> io::Result<()> {
        let bytes = Zeroizing::new(
            kr_wire::encode(&[0; 37], op, body, self.send_seq).map_err(io::Error::other)?,
        );
        timeout(DEADLINE, self.tx.write_all(&bytes))
            .await?
            .map_err(io::Error::other)?;
        self.send_seq = self.send_seq.wrapping_add(3);
        Ok(())
    }

    async fn expect(&mut self, expected: u32, skip: &[u32]) -> io::Result<Packet> {
        timeout(DEADLINE, async {
            for _ in 0..256 {
                let mut header = [0; 37];
                self.rx
                    .read_exact(&mut header)
                    .await
                    .map_err(io::Error::other)?;
                let n = u32::from_le_bytes(header[13..17].try_into().unwrap()) as usize;
                if n > 65536 {
                    return Err(failure("oversize reply"));
                }
                let mut raw = header.to_vec();
                raw.resize(38 + n, 0);
                self.rx
                    .read_exact(&mut raw[37..])
                    .await
                    .map_err(io::Error::other)?;
                let packet = kr_wire::decode(&raw, self.recv_seq).map_err(io::Error::other)?;
                self.recv_seq = self.recv_seq.wrapping_add(3);
                if packet.opcode == expected {
                    return Ok(packet);
                }
                if !skip.contains(&packet.opcode) {
                    return Err(io::Error::other(format!(
                        "expected opcode {expected}, got {}",
                        packet.opcode
                    )));
                }
            }
            Err(failure("too many frames before expected reply"))
        })
        .await?
    }
}

struct Owner(quinn::Connection);
impl Drop for Owner {
    fn drop(&mut self) {
        self.0.close(0u32.into(), b"load client finished");
    }
}

pub struct Client {
    _owner: Owner,
    wire: Wire,
    pub udp: UdpSocket,
}

impl Client {
    pub async fn login(
        endpoint: &quinn::Endpoint,
        config: &Config,
        account: &Account,
        companions: &mut JoinSet<io::Result<()>>,
        stop: watch::Receiver<bool>,
    ) -> io::Result<Self> {
        timeout(DEADLINE, async {
            let conn = Owner(prelogin::connect(endpoint, config.server.into()).await?);
            let mut login = Wire::open(&conn.0, config.login_port).await?;
            let mut credentials = Zeroizing::new(fields(&[(0, U8), (1, U8)]));
            for (data, key) in [
                (account.login.as_bytes(), 0x11223344),
                (account.password.as_bytes(), 0x44332211),
            ] {
                credentials.extend(fields(&[(data.len() as u64, U16)]));
                credentials.extend(kr_wire::crypt(data, key, false));
            }
            credentials.extend([0; 16]);
            credentials.extend(fields(&[(0, U32), (0x0200007f, U32), (0, U8)]));
            login.send(1, &credentials).await?;
            drop(credentials);
            let claim =
                Zeroizing::new(claims(&login.expect(4, &[]).await?.payload, &account.name)?);
            login.send(9, &[0; 8]).await?;
            login.expect(15, &[]).await?;
            login.expect(62, &[]).await?;
            login.send(5, &[0; 9]).await?;
            login.expect(7, &[]).await?;
            login.tx.finish().map_err(io::Error::other)?;
            let mut wire = Wire::open(&conn.0, config.channel_port).await?;
            wire.send(6, &claim).await?;
            wire.expect(17, &[]).await?;
            wire.send(67, &fields(&[(0, U16)])).await?;
            for op in [241, 91, 14, 291] {
                wire.expect(op, &[90, 91]).await?;
            }
            let (relay_endpoint, link) = companion(&conn.0, config).await?;
            companions.spawn(async move {
                let result =
                    kr_tunnel::companion::run(link, Ipv4Addr::LOCALHOST.into(), 64, stop).await;
                relay_endpoint.close(0u32.into(), b"load client finished");
                result
            });
            Ok(Self {
                _owner: conn,
                wire,
                udp: UdpSocket::bind((Ipv4Addr::LOCALHOST, 0)).await?,
            })
        })
        .await?
    }

    pub async fn create(&mut self, label: &str) -> io::Result<u64> {
        let mut body = fields(&[(label.len() as u64, U16)]);
        body.extend(kr_wire::kr00967::encode_bytes(label.as_bytes()).map_err(io::Error::other)?);
        body.extend(fields(&[(0, U16)]));
        let values = [0, 0, 5, 8191, 0, 0, 0, u32::MAX, 0, 0, 0, 0];
        body.extend(fields(&values.map(|v| (u64::from(v), U32))));
        body.extend(fields(&[(0, U8); 4]));
        self.wire.send(81, &body).await?;
        get(&self.wire.expect(86, &[90, 91]).await?.payload, 1, U16)
    }

    pub async fn join(&mut self, room: u64, slot: usize) -> io::Result<()> {
        self.wire
            .send(
                82,
                &fields(&[(room, U16), (room, U32), (0, U16), (1, U8), (5, U16)]),
            )
            .await?;
        if get(&self.wire.expect(87, &[90, 91]).await?.payload, 1, U8)? != slot as u64 {
            return Err(failure("unexpected room slot"));
        }
        Ok(())
    }

    pub async fn setup_room(&mut self) -> io::Result<()> {
        self.wire.send(585, &fields(&[(0, U32)])).await?;
        self.wire.expect(117, &[90, 91, 106, 110, 238]).await?;
        Ok(())
    }

    pub async fn select_map(&mut self, code: u16, index: u32) -> io::Result<()> {
        self.wire
            .send(96, &fields(&[(code.into(), U16), (index.into(), U32)]))
            .await
    }

    pub async fn map_selected(&mut self, code: u16, index: u32) -> io::Result<()> {
        let body = self
            .wire
            .expect(117, &[90, 91, 106, 110, 238])
            .await?
            .payload;
        if get(&body, 2, U16)? != u64::from(code) || get(&body, 4, U32)? != u64::from(index) {
            return Err(failure("unexpected map selection acknowledgement"));
        }
        Ok(())
    }

    pub async fn team(&mut self) -> io::Result<()> {
        self.wire.send(97, &fields(&[(3, U8)])).await
    }

    pub async fn team_updates(&mut self, roster: usize, changed: usize) -> io::Result<()> {
        let mut seen = 0u16;
        for _ in 0..roster {
            let update = self.wire.expect(110, &[90, 91, 106, 238]).await?;
            let slot = get(&update.payload, 0, U8)? as usize;
            if slot >= roster || seen & (1 << slot) != 0 {
                return Err(failure("invalid room choice snapshot"));
            }
            seen |= 1 << slot;
            if slot == changed && get(&update.payload, 1, U32)? & 0xe0 != 0x60 {
                return Err(failure("team selection not acknowledged"));
            }
        }
        Ok(())
    }

    pub async fn character(&mut self, character: u16) -> io::Result<()> {
        self.wire
            .send(92, &fields(&[(0, U8), (character.into(), U16)]))
            .await
    }

    pub async fn character_updates(&mut self, changed: usize, character: u16) -> io::Result<()> {
        let mut seen = 0;
        for _ in 0..2 {
            let body = self.wire.expect(110, &[90, 91, 106, 238]).await?.payload;
            let slot = get(&body, 0, U8)? as usize;
            if slot >= 2 || seen & (1 << slot) != 0 {
                return Err(failure("invalid character choice snapshot"));
            }
            seen |= 1 << slot;
            if slot == changed && (get(&body, 1, U32)? >> 16) & 0x1fff != u64::from(character) {
                return Err(failure("character selection not acknowledged"));
            }
        }
        Ok(())
    }

    pub async fn claim_basic_item(
        &mut self,
        slot: usize,
        item: (u32, u8, u8, u16),
        key: u32,
    ) -> io::Result<()> {
        let (index, row, col, kind) = item;
        self.wire
            .send(
                124,
                &fields(&[
                    (0, U8),
                    (slot as u64, U8),
                    (row.into(), U8),
                    (col.into(), U8),
                    (index.into(), U32),
                    (kind.into(), U16),
                    (index.into(), U32),
                    (key.into(), U32),
                    (0, U8),
                ]),
            )
            .await
    }

    pub async fn pickup_confirmed(
        &mut self,
        slot: usize,
        item: (u32, u8, u8, u16),
    ) -> io::Result<()> {
        let (index, row, col, kind) = item;
        let expected = kr_wire::kr00967::pickup165(0, slot as u8, row, col, index, kind)?;
        if self.wire.expect(165, &[90, 91]).await?.payload != expected {
            return Err(failure("basic item confirmation mismatch"));
        }
        Ok(())
    }

    pub async fn ready(&mut self) -> io::Result<()> {
        let mut ready = fields(&[(6, U32)]);
        ready.extend([0; 6]);
        ready.extend(fields(&[(1, U8), (0, U32)]));
        self.wire.send(95, &ready).await
    }

    pub async fn ready_updates(&mut self, roster: usize) -> io::Result<()> {
        let mut seen = 0u8;
        for _ in 0..roster * roster {
            let body = self.wire.expect(110, &[90, 91, 106, 238]).await?.payload;
            let slot = get(&body, 0, U8)? as usize;
            if slot > 0 && slot < roster && get(&body, 1, U32)? & 4 != 0 {
                seen |= 1 << slot;
                if u16::from(seen) == (1u16 << roster) - 2 {
                    return Ok(());
                }
            }
        }
        Err(failure("ready roster incomplete"))
    }

    pub async fn start(&mut self) -> io::Result<()> {
        self.wire.send(104, &[0; 20]).await
    }

    pub async fn load(
        &mut self,
        expected: Option<[u16; 2]>,
        expected_map: Option<u16>,
    ) -> io::Result<Option<crate::pickups::Catalog>> {
        let floor = self.wire.expect(582, &[90, 91, 110]).await?;
        let start = self.wire.expect(121, &[]).await?;
        if let Some(code) = expected_map
            && kr_wire::kr00967::start121_map(&start.payload)?.0 != code
        {
            return Err(failure("unexpected map in Start121"));
        }
        let floor = if let Some([a, b]) = expected {
            if kr_wire::kr00967::start121_characters(&start.payload)? != [(0, a), (1, b)] {
                return Err(failure("character Start roster mismatch"));
            }
            Some(crate::pickups::Catalog::from_floor(&floor.payload)?)
        } else {
            None
        };
        self.wire
            .send(
                99,
                &fields(&[
                    (0, U32),
                    (0, U32),
                    (self.udp.local_addr()?.port().into(), U16),
                    (0, U32),
                    (0, U16),
                    (0, U8),
                    (0, U8),
                    (0, U8),
                    (0, U32),
                ]),
            )
            .await?;
        Ok(floor)
    }

    pub async fn routes(
        &mut self,
        slot: usize,
        roster: usize,
    ) -> io::Result<(u32, Vec<Option<SocketAddrV4>>)> {
        self.wire.expect(197, &[]).await?;
        self.wire.expect(196, &[]).await?;
        let body = self.wire.expect(153, &[]).await?.payload;
        Ok((
            get(&body, 15, U32)? as u32,
            parse_routes(&body, slot, roster)?,
        ))
    }

    pub async fn die(&mut self, slot: usize, round: u64) -> io::Result<()> {
        for (kind, size) in [(17, 14u32), (19, 11)] {
            let mut body = fields(&[
                (kind, U8),
                (slot as u64, U8),
                (120, U16),
                (159, U16),
                (round * 100 + kind, U32),
                (0, U32),
                (size.into(), U32),
            ]);
            let mut transient = vec![0u8; size as usize];
            transient[0] = 24;
            transient[1..5].copy_from_slice(&(size - 5).to_le_bytes());
            if kind == 17 {
                transient[10..14].copy_from_slice(&7u32.to_le_bytes());
            }
            body.extend(kr_wire::kr00967::encode_bytes(&transient).map_err(io::Error::other)?);
            self.wire.send(127, &body).await?;
        }
        Ok(())
    }

    pub async fn finish(&mut self) -> io::Result<()> {
        self.wire.expect(175, &[170, 90, 91]).await?;
        let mut tail = fields(&[(0, U8), (1, U8)]);
        tail.extend([0; 20]);
        self.wire.send(202, &tail).await
    }

    pub async fn returned(&mut self, roster: usize, expected: Option<[u16; 2]>) -> io::Result<()> {
        self.wire.expect(180, &[90, 91]).await?;
        self.wire.send(585, &fields(&[(0, U32)])).await?;
        for slot in 0..roster {
            let update = self.wire.expect(110, &[90, 91]).await?;
            if get(&update.payload, 0, U8)? != slot as u64 || get(&update.payload, 1, U32)? & 4 != 0
            {
                return Err(failure("roster did not reset"));
            }
            if let Some(keys) = expected
                && (get(&update.payload, 1, U32)? >> 16) & 0x1fff != u64::from(keys[slot])
            {
                return Err(failure("character changed on return"));
            }
        }
        Ok(())
    }

    pub async fn leave(&mut self) -> io::Result<()> {
        self.wire.send(86, &[]).await?;
        self.wire.expect(241, &[90, 91, 107, 110, 235]).await?;
        Ok(())
    }
}

fn parse_routes(body: &[u8], slot: usize, roster: usize) -> io::Result<Vec<Option<SocketAddrV4>>> {
    let mut routes = Vec::with_capacity(roster);
    let mut at = 35;
    for peer in 0..26 {
        let present = get(body, at, U8)?;
        at += 1;
        if present != u64::from(peer < roster) {
            return Err(failure("route roster mismatch"));
        }
        if present == 1 {
            let ip = Ipv4Addr::from(get(body, at, U32)? as u32);
            let port = get(body, at + 4, U16)? as u16;
            at += 18;
            let route = if peer == slot && ip.is_unspecified() && port == 0 {
                None
            } else if peer != slot && ip == Ipv4Addr::LOCALHOST && port != 0 {
                Some(SocketAddrV4::new(ip, port))
            } else {
                return Err(failure("non-loopback or missing route"));
            };
            if route.is_some() && routes.contains(&route) {
                return Err(failure("duplicate proxy port"));
            }
            routes.push(route);
        }
    }
    if at != body.len() {
        return Err(failure("unexpected route tail"));
    }
    Ok(routes)
}

async fn companion(
    conn: &quinn::Connection,
    cfg: &Config,
) -> io::Result<(quinn::Endpoint, kr_tunnel::Link)> {
    let mut offer = conn.accept_uni().await.map_err(io::Error::other)?;
    if offer.read_u8().await? != 1 {
        return Err(failure("expected one companion offer"));
    }
    let mut ip = [0; 4];
    offer.read_exact(&mut ip).await.map_err(io::Error::other)?;
    let advertised = SocketAddrV4::new(ip.into(), offer.read_u16().await?);
    if advertised != cfg.relay {
        return Err(failure("relay offer differs from test target"));
    }
    let size = offer.read_u16().await? as usize;
    if !(1..=4096).contains(&size) {
        return Err(failure("companion certificate length"));
    }
    let mut cert = vec![0; size];
    offer
        .read_exact(&mut cert)
        .await
        .map_err(io::Error::other)?;
    let mut secret = Zeroizing::new([0; 32]);
    offer
        .read_exact(secret.as_mut())
        .await
        .map_err(io::Error::other)?;
    if offer
        .read(&mut [0; 1])
        .await
        .map_err(io::Error::other)?
        .is_some()
    {
        return Err(failure("unexpected companion offer tail"));
    }
    let dial = cfg.relay_dial.unwrap_or(advertised);
    let ep = kr_tunnel::client((Ipv4Addr::UNSPECIFIED, 0).into(), cert.into())?;
    let link =
        kr_tunnel::Link::connect(&ep, dial.into(), &kr_tunnel::Secret::from_bytes(*secret)).await?;
    Ok((ep, link))
}
