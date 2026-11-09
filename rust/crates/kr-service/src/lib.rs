//! Bounded diagnostic KR00967 transport; no account authentication or gameplay.
#![forbid(unsafe_code)]

use kr_wire::{HEADER_LEN, Receiver, decode, encode};
use serde::Deserialize;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::fmt::Write as _;
use std::{
    collections::HashSet,
    io,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    path::{Path, PathBuf},
    sync::Arc,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    sync::{Semaphore, mpsc, watch},
    task::JoinSet,
    time::timeout,
};

fn sha256_hex(data: &[u8]) -> String {
    let mut out = String::with_capacity(64);
    for byte in Sha256::digest(data) {
        write!(&mut out, "{byte:02x}").expect("String formatting");
    }
    out
}

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FixturePin {
    pub path: PathBuf,
    pub bytes: usize,
    pub sha256: String,
}
impl FixturePin {
    fn load(&self) -> io::Result<Vec<u8>> {
        if self.bytes > 65_536
            || self.sha256.len() != 64
            || !self
                .sha256
                .bytes()
                .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
        {
            return Err(io::Error::other("invalid fixture pin"));
        }
        if std::fs::metadata(&self.path)?.len() != self.bytes as u64 {
            return Err(io::Error::other("fixture size mismatch"));
        }
        let data = std::fs::read(&self.path)?;
        if sha256_hex(&data) != self.sha256 {
            return Err(io::Error::other("fixture SHA256 mismatch"));
        }
        Ok(data)
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub schema: String,
    pub diagnostic_login_error_only: bool,
    pub bind: Ipv4Addr,
    pub allowed_peers: Vec<Ipv4Addr>,
    pub ports: Vec<u16>,
    pub max_connections: usize,
    pub idle_timeout_seconds: u64,
    pub write_timeout_ms: u64,
    pub bootstrap: FixturePin,
    pub login_error: FixturePin,
    pub wcp_second: FixturePin,
}

pub struct Validated {
    config: Config,
    bootstrap: Vec<u8>,
    error_header: [u8; HEADER_LEN],
    wcp_second: Vec<u8>,
}
pub fn load_config(path: &Path) -> io::Result<Validated> {
    if std::fs::metadata(path)?.len() > 65_536 {
        return Err(io::Error::other("config too large"));
    }
    let config: Config = serde_json::from_slice(&std::fs::read(path)?).map_err(io::Error::other)?;
    if config.schema != "kr-rust-diagnostic-v1"
        || !config.diagnostic_login_error_only
        || !config.bind.is_loopback()
        || config.allowed_peers.is_empty()
        || config.allowed_peers.iter().any(|p| !p.is_loopback())
        || config.ports.is_empty()
        || config.ports.len() > 32
        || config.ports.contains(&0)
        || config.ports.iter().copied().collect::<HashSet<_>>().len() != config.ports.len()
        || !(1..=64).contains(&config.max_connections)
        || !(1..=120).contains(&config.idle_timeout_seconds)
        || !(1..=10_000).contains(&config.write_timeout_ms)
    {
        return Err(io::Error::other("unsupported diagnostic configuration"));
    }
    let bootstrap = config.bootstrap.load()?;
    let error = config.login_error.load()?;
    let wcp_second = config.wcp_second.load()?;
    if wcp_second.is_empty() {
        return Err(io::Error::other("empty WCP fixture"));
    }
    if bootstrap.len() != 15 || error.len() != 39 {
        return Err(io::Error::other("unsupported fixture profile"));
    }
    let packet = decode(&error, 43).map_err(io::Error::other)?;
    if packet.opcode != 4 || packet.payload != [0x5b] {
        return Err(io::Error::other("expected pinned KR result1"));
    }
    let error_header = error[..HEADER_LEN]
        .try_into()
        .expect("validated 39-byte fixture");
    if encode(&error_header, 4, &[0x5b], 43).map_err(io::Error::other)? != error {
        return Err(io::Error::other(
            "result1 encoding disagrees with native-tested fixture",
        ));
    }
    Ok(Validated {
        config,
        bootstrap,
        error_header,
        wcp_second,
    })
}

type Log = mpsc::Sender<Value>;
async fn emit(log: &Log, value: Value) -> io::Result<()> {
    timeout(Duration::from_millis(200), log.send(value))
        .await
        .map_err(|_| io::Error::other("log backpressure"))?
        .map_err(|_| io::Error::other("log writer unavailable"))
}
fn stamp(start: Instant, mut value: Value) -> Value {
    value["elapsed_ns"] = json!(start.elapsed().as_nanos().min(u64::MAX as u128) as u64);
    value
}
async fn state(out: &Path, ready: bool, stopped: bool) -> io::Result<()> {
    let value = json!({"schema":"kr-rust-diagnostic-v1", "ready":ready, "stopped":stopped, "tcp_events":[], "errors":[]});
    tokio::fs::write(
        out.join("service-state.tmp"),
        serde_json::to_vec(&value).map_err(io::Error::other)?,
    )
    .await?;
    tokio::fs::rename(
        out.join("service-state.tmp"),
        out.join("service-state.json"),
    )
    .await
}

async fn connection(
    mut socket: TcpStream,
    id: u64,
    profile: Arc<Validated>,
    log: Log,
    mut stop: watch::Receiver<bool>,
    start: Instant,
) -> io::Result<()> {
    socket.set_nodelay(true)?;
    let write_limit = Duration::from_millis(profile.config.write_timeout_ms);
    timeout(write_limit, socket.write_all(&profile.bootstrap))
        .await
        .map_err(|_| io::Error::other("hello write timeout"))??;
    emit(&log, stamp(start,json!({"event":"hello_sent","connection":id,"bytes":15,"sha256":profile.config.bootstrap.sha256}))).await?;
    // CP181 diagnostic mapping: the first two transports are the known WCP
    // bootstrap path. Its opaque reply is pinned; do not parse it as mode2.
    // This arrival mapping is not multi-account authentication or production routing.
    if id <= 2 {
        let mut bytes_read = 0usize;
        let mut replied = false;
        let mut data = [0; 4096];
        loop {
            let size = tokio::select! {
                _=stop.changed()=>return Ok(()),
                value=timeout(Duration::from_secs(profile.config.idle_timeout_seconds),socket.read(&mut data))=>value.map_err(|_|io::Error::other("WCP read timeout"))??,
            };
            if size == 0 {
                return Ok(());
            }
            bytes_read += size;
            if bytes_read > 65536 {
                return Err(io::Error::other("WCP diagnostic receive bound"));
            }
            if id == 2 && !replied {
                timeout(write_limit, socket.write_all(&profile.wcp_second))
                    .await
                    .map_err(|_| io::Error::other("WCP reply timeout"))??;
                replied = true;
                emit(&log,stamp(start,json!({"event":"wcp_reply_sent","connection":id,"bytes":profile.wcp_second.len(),"sha256":profile.config.wcp_second.sha256,"policy":"CP181_diagnostic_first_two_transports"}))).await?;
            }
        }
    }
    let mut receive = Receiver::new(40);
    let mut rejected = false;
    let mut ancillary_count = 0usize;
    let mut chunk = [0; 4096];
    loop {
        while let Some(packet) = receive.next_packet().map_err(io::Error::other)? {
            // Deliberately omit raw auth body, token and body digest from logs.
            emit(&log,stamp(start,json!({"event":"packet_received","connection":id,"opcode":packet.opcode,"sequence":packet.sequence,"payload_bytes":packet.payload.len()}))).await?;
            // CP181 login capture carries ancillary60/body20 and15/body31.
            // Python leaves these unanswered. Preserve that narrow policy,
            // without claiming their meaning or treating them as credentials.
            if rejected && matches!((packet.opcode, packet.payload.len()), (60, 20) | (15, 31)) {
                ancillary_count += 1;
                if ancillary_count > 64 {
                    return Err(io::Error::other("ancillary packet budget"));
                }
                emit(&log,stamp(start,json!({"event":"ancillary_ignored","connection":id,"opcode":packet.opcode,"payload_bytes":packet.payload.len(),"policy":"CP181_observed_shapes_no_reply"}))).await?;
                continue;
            }
            if rejected || packet.opcode != 1 {
                return Err(io::Error::other("unsupported packet/stage"));
            }
            let frame = encode(&profile.error_header, 4, &[0x5b], 43).map_err(io::Error::other)?;
            timeout(write_limit, socket.write_all(&frame))
                .await
                .map_err(|_| io::Error::other("result write timeout"))??;
            rejected = true;
            emit(&log,stamp(start,json!({"event":"login_error_sent","connection":id,"sequence":43,"opcode":4,"bytes":frame.len(),"sha256":sha256_hex(&frame),"policy":"diagnostic_result1_no_authentication"}))).await?;
        }
        let capacity = chunk.len().min(receive.remaining_capacity());
        if capacity == 0 {
            return Err(io::Error::other("receive buffer exhausted"));
        }
        let size = tokio::select! {
            _ = stop.changed() => return Ok(()),
            value = timeout(Duration::from_secs(profile.config.idle_timeout_seconds),socket.read(&mut chunk[..capacity])) => value.map_err(|_|io::Error::other("read idle timeout"))??,
        };
        if size == 0 {
            return receive.finish().map_err(io::Error::other);
        }
        receive.push(&chunk[..size]).map_err(io::Error::other)?;
    }
}

pub async fn serve(profile: Validated, out: &Path) -> io::Result<()> {
    tokio::fs::create_dir_all(out).await?;
    // Bind the whole configured set before readiness; startup failure drops all.
    let mut bound = Vec::new();
    for &port in &profile.config.ports {
        bound.push(TcpListener::bind((profile.config.bind, port)).await?);
    }
    let file = tokio::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(out.join("rust-events.jsonl"))
        .await?;
    let (log, mut events) = mpsc::channel::<Value>(256);
    let writer = tokio::spawn(async move {
        let mut file = file;
        while let Some(value) = events.recv().await {
            let mut data = serde_json::to_vec(&value).map_err(io::Error::other)?;
            data.push(b'\n');
            timeout(Duration::from_secs(2), file.write_all(&data))
                .await
                .map_err(|_| io::Error::other("log write timeout"))??;
            timeout(Duration::from_secs(2), file.flush())
                .await
                .map_err(|_| io::Error::other("log flush timeout"))??;
        }
        Ok::<_, io::Error>(())
    });
    let start = Instant::now();
    let boot = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(io::Error::other)?
        .as_nanos();
    emit(&log,json!({"event":"boot","pid":std::process::id(),"boot":boot.to_string(),"package":env!("CARGO_PKG_VERSION"),"source":option_env!("KR_SOURCE_REV").unwrap_or("unpublished"),"profile":"diagnostic_login_error_only","bootstrap_sha256":profile.config.bootstrap.sha256,"error_sha256":profile.config.login_error.sha256})).await?;
    let profile = Arc::new(profile);
    let (shutdown, stop) = watch::channel(false);
    let (accept_tx, mut accepted) = mpsc::channel::<(TcpStream, SocketAddr)>(32);
    let mut listeners = JoinSet::new();
    for listener in bound {
        let tx = accept_tx.clone();
        let mut stop = stop.clone();
        listeners.spawn(async move {
            loop {
                let pair=tokio::select! { _=stop.changed()=>return Ok::<_,io::Error>(()), p=listener.accept()=>p? };
                tokio::select! { _=stop.changed()=>return Ok(()), sent=tx.send(pair)=>if sent.is_err(){return Ok(());} }
            }
        });
    }
    drop(accept_tx);
    let quota = Arc::new(Semaphore::new(profile.config.max_connections));
    let mut workers = JoinSet::new();
    let mut next_id = 0u64;
    let mut term = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())?;
    state(out, true, false).await?;
    emit(
        &log,
        stamp(start, json!({"event":"ready","ports":profile.config.ports})),
    )
    .await?;
    let mut failure = None;
    let outcome: io::Result<()> = async { loop {
        tokio::select! {
            _=term.recv()=>break,
            value=tokio::signal::ctrl_c()=>{if let Err(e)=value {failure=Some(e);}break;},
            result=listeners.join_next(),if !listeners.is_empty()=>{failure=Some(io::Error::other(format!("listener stopped: {result:?}")));break;},
            result=workers.join_next(),if !workers.is_empty()=>if let Some(result)=result {match result {Ok(Ok(()))=>{},other=>{failure=Some(io::Error::other(format!("worker stopped: {other:?}")));break;}}},
            pair=accepted.recv()=>{
                let Some((socket,peer))=pair else {failure=Some(io::Error::other("all listeners stopped"));break;};
                let permitted=matches!(peer.ip(),IpAddr::V4(ip) if profile.config.allowed_peers.contains(&ip));
                let Ok(permit)=quota.clone().try_acquire_owned() else {drop(socket);emit(&log,stamp(start,json!({"event":"admission_rejected","reason":"connection_limit"}))).await?;continue;};
                if !permitted {drop(socket);drop(permit);emit(&log,stamp(start,json!({"event":"admission_rejected","reason":"peer_not_allowed"}))).await?;continue;}
                next_id+=1;let id=next_id;let local=socket.local_addr()?;
                emit(&log,stamp(start,json!({"event":"accepted","connection":id,"peer":peer.to_string(),"local":local.to_string(),"receive_sequence":40}))).await?;
                let profile=profile.clone();let log=log.clone();let stop=stop.clone();
                workers.spawn(async move {let _permit=permit;let result=connection(socket,id,profile,log.clone(),stop,start).await;emit(&log,stamp(start,json!({"event":"closed","connection":id,"reason":result.as_ref().err().map(ToString::to_string).unwrap_or_else(||"eof_or_shutdown".into())}))).await});
            }
        }
    }
    Ok(()) }.await;
    if let Err(error) = outcome {
        failure = Some(error);
    }
    let _ = shutdown.send(true);
    accepted.close();
    while let Ok(pair) = accepted.try_recv() {
        drop(pair);
    }
    if timeout(Duration::from_secs(3), async {
        while listeners.join_next().await.is_some() {}
        while workers.join_next().await.is_some() {}
    })
    .await
    .is_err()
    {
        listeners.abort_all();
        workers.abort_all();
        while listeners.join_next().await.is_some() {}
        while workers.join_next().await.is_some() {}
    }
    state(out, false, true).await?;
    emit(&log,stamp(start,json!({"event":"stopped","connections":next_id,"error":failure.as_ref().map(ToString::to_string)}))).await?;
    drop(log);
    timeout(Duration::from_secs(3), writer)
        .await
        .map_err(|_| io::Error::other("logger shutdown timeout"))?
        .map_err(io::Error::other)??;
    if let Some(error) = failure {
        return Err(error);
    }
    Ok(())
}
