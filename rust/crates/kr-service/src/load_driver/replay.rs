use super::{client::Client, failure};
use serde::{Deserialize, Serialize};
use std::{io, net::SocketAddrV4, time::Duration};
use tokio::time::{Instant, sleep_until, timeout_at};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Replay {
    schema: String,
    checkpoint: String,
    windows: Vec<Window>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Window {
    duration_ns: u64,
    packets: Vec<(u64, usize)>,
}

impl Replay {
    pub fn validate(&self) -> io::Result<()> {
        if self.schema != "kr-m8-replay-v1" || self.checkpoint != "CP287" || self.windows.len() != 8
        {
            return Err(failure("expected eight CP287 replay windows"));
        }
        for win in &self.windows {
            if !(1_000_000_000..=120_000_000_000).contains(&win.duration_ns)
                || win.packets.is_empty()
                || win.packets.len() > 3500
                || win.packets.windows(2).any(|pair| pair[0].0 > pair[1].0)
                || win.packets.iter().any(|&(t, len)| {
                    t >= win.duration_ns || !(5..=kr_tunnel::MAX_NATIVE).contains(&len)
                })
            {
                return Err(failure("invalid replay window"));
            }
            let pps = win.packets.len() as f64 * 1e9 / win.duration_ns as f64;
            if !(11.55..=14.50).contains(&pps) {
                return Err(failure("replay rate differs from CP377"));
            }
            // Include the seam when repeatedly playing this same window.
            let times: Vec<_> = win
                .packets
                .iter()
                .map(|p| p.0)
                .chain(win.packets.iter().map(|p| p.0 + win.duration_ns))
                .collect();
            let mut left = 0;
            for (right, &t) in times.iter().enumerate() {
                while t - times[left] >= 1_000_000_000 {
                    left += 1;
                }
                if right + 1 - left > 27 {
                    return Err(failure("replay peak exceeds CP377"));
                }
            }
        }
        Ok(())
    }
}

#[derive(Clone, Copy)]
struct Event {
    at_ns: u64,
    from: usize,
    to: usize,
    len: usize,
}

fn timeline(replay: &Replay, roster: usize, active: Duration, seed: usize) -> Vec<Event> {
    let end = active.as_nanos() as u64;
    let mut events = Vec::new();
    for from in 0..roster {
        let win = &replay.windows[(seed + from) % replay.windows.len()];
        // Give each source a different phase without changing its intervals.
        let phase = from as u64 * 17_000_000;
        let mut next = from;
        for cycle in 0.. {
            let base = cycle * win.duration_ns + phase;
            if base >= end {
                break;
            }
            for &(t, len) in &win.packets {
                if base + t >= end {
                    break;
                }
                next = (next + 1) % roster;
                if next == from {
                    next = (next + 1) % roster;
                }
                events.push(Event {
                    at_ns: base + t,
                    from,
                    to: next,
                    len,
                });
            }
        }
    }
    events.sort_by_key(|ev| (ev.at_ns, ev.from));
    events
}

#[derive(Default, Serialize)]
pub struct Delivery {
    pub sent: usize,
    pub received: usize,
    pub loss_percent: f64,
    pub p50_ms: f64,
    pub p95_ms: f64,
    pub p99_ms: f64,
    pub max_ms: f64,
    pub late_over_250ms: usize,
    pub pacing_late_over_20ms: usize,
}

struct Receive {
    sequence: usize,
    at: Instant,
}

pub async fn exchange(
    clients: &[Client],
    routes: &[Vec<Option<SocketAddrV4>>],
    replay: &Replay,
    active: Duration,
    seed: usize,
    tag: u8,
) -> io::Result<Delivery> {
    let events = timeline(replay, clients.len(), active, seed);
    let epoch = Instant::now() + Duration::from_millis(100);
    let cutoff = epoch + active + Duration::from_secs(1);
    let mut sent = Vec::with_capacity(events.len());
    // Each loopback socket has one owner for reads and writes. Pump transmit
    // and all receivers concurrently; never serialize network RTT with sends.
    let sockets: Vec<_> = clients.iter().map(|c| &c.udp).collect();
    let send = async {
        let mut pacing_late_over_20ms = 0;
        let mut payload = [0u8; kr_tunnel::MAX_NATIVE];
        payload[4] = tag;
        for (seq, ev) in events.iter().enumerate() {
            let deadline = epoch + Duration::from_nanos(ev.at_ns);
            sleep_until(deadline).await;
            let now = Instant::now();
            if now >= epoch + active {
                return Err(failure("pacing exceeded the active window"));
            }
            pacing_late_over_20ms += usize::from(now - deadline > Duration::from_millis(20));
            payload[..4].copy_from_slice(&(seq as u32).to_le_bytes());
            let target = routes[ev.from][ev.to].ok_or_else(|| failure("missing send route"))?;
            sockets[ev.from].send_to(&payload[..ev.len], target).await?;
            sent.push(now);
        }
        Ok::<_, io::Error>(pacing_late_over_20ms)
    };
    let receive = async {
        let mut entries = Vec::with_capacity(events.len());
        // Duplicating each UDP handle would alter the measured local resource
        // footprint; use an indexed poll over Tokio's readiness futures.
        let mut payloads = vec![[0u8; kr_tunnel::MAX_NATIVE + 1]; sockets.len()];
        let mut first = 0;
        while let Ok(result) = timeout_at(
            cutoff,
            std::future::poll_fn(|cx| {
                for i in 0..sockets.len() {
                    let slot = (first + i) % sockets.len();
                    let mut b = tokio::io::ReadBuf::new(&mut payloads[slot]);
                    if let std::task::Poll::Ready(result) = sockets[slot].poll_recv_from(cx, &mut b)
                    {
                        first = (slot + 1) % sockets.len();
                        return std::task::Poll::Ready(
                            result.map(|peer| (slot, b.filled().len(), peer)),
                        );
                    }
                }
                std::task::Poll::Pending
            }),
        )
        .await
        {
            let (target, len, peer) = result?;
            let std::net::SocketAddr::V4(source_port) = peer else {
                return Err(failure("unexpected UDP source family"));
            };
            let b = &payloads[target];
            if len < 5 || b[4] != tag || entries.len() >= events.len() {
                return Err(failure("unexpected datagram or replay generation"));
            }
            let sequence = u32::from_le_bytes(b[..4].try_into().unwrap()) as usize;
            let event = events
                .get(sequence)
                .ok_or_else(|| failure("unknown datagram id"))?;
            if event.to != target
                || event.len != len
                || routes[target][event.from] != Some(source_port)
            {
                return Err(failure("misdelivered datagram"));
            }
            entries.push(Receive {
                sequence,
                at: Instant::now(),
            });
            // Keep timers for other rooms and the send side running.
            tokio::task::consume_budget().await;
        }
        Ok::<_, io::Error>(entries)
    };
    let (pacing_late_over_20ms, entries) = tokio::try_join!(send, receive)?;
    let mut received = vec![false; events.len()];
    let mut samples = Vec::with_capacity(entries.len());
    for Receive { sequence, at } in entries {
        if std::mem::replace(&mut received[sequence], true) || at < sent[sequence] {
            return Err(failure("duplicate or time-reversed datagram"));
        }
        samples.push((at - sent[sequence]).as_secs_f64() * 1000.);
    }
    samples.sort_by(f64::total_cmp);
    let percentile = |p: usize| {
        samples
            .get((samples.len() * p).div_ceil(100).saturating_sub(1))
            .copied()
            .unwrap_or_default()
    };
    Ok(Delivery {
        sent: sent.len(),
        received: samples.len(),
        loss_percent: 100. * (sent.len() - samples.len()) as f64 / sent.len() as f64,
        p50_ms: percentile(50),
        p95_ms: percentile(95),
        p99_ms: percentile(99),
        max_ms: percentile(100),
        late_over_250ms: samples.iter().filter(|&&ms| ms > 250.).count(),
        pacing_late_over_20ms,
    })
}
