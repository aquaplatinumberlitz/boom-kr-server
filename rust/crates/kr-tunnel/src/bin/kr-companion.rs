//! Private launcher child. Capabilities arrive only on stdin, never argv/logs.
#![forbid(unsafe_code)]
use kr_tunnel::{Link, Secret, companion};
use serde::Deserialize;
use std::{
    io::{self, Read, Write},
    net::{IpAddr, Ipv4Addr, SocketAddrV4},
};
use tokio::sync::watch;
use zeroize::Zeroizing;

struct Capability(Secret);
impl<'de> Deserialize<'de> for Capability {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let value = Zeroizing::new(String::deserialize(d)?);
        if value.len() != 64 {
            return Err(serde::de::Error::custom("invalid capability length"));
        }
        let mut bytes = Zeroizing::new([0; 32]);
        for (i, pair) in value.as_bytes().as_chunks::<2>().0.iter().enumerate() {
            let nibble = |b| match b {
                b'0'..=b'9' => Some(b - b'0'),
                b'a'..=b'f' => Some(b - b'a' + 10),
                _ => None,
            };
            let hi = nibble(pair[0])
                .ok_or_else(|| serde::de::Error::custom("invalid capability encoding"))?;
            let lo = nibble(pair[1])
                .ok_or_else(|| serde::de::Error::custom("invalid capability encoding"))?;
            bytes[i] = (hi << 4) | lo;
        }
        Ok(Self(Secret::from_bytes(*bytes)))
    }
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Offer {
    endpoint: SocketAddrV4,
    certificate_der: Vec<u8>,
    capability: Capability,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Request {
    peer_transport: Offer,
    native_ip: IpAddr,
    port_budget: usize,
    #[serde(default)]
    diagnostics: bool,
    #[serde(default)]
    udp_backend: kr_tunnel::UdpBackend,
}
#[cfg(unix)]
fn termination_signal() -> io::Result<impl std::future::Future<Output = ()>> {
    let mut signal = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())?;
    Ok(async move {
        signal.recv().await;
    })
}
#[cfg(windows)]
fn termination_signal() -> io::Result<impl std::future::Future<Output = ()>> {
    let mut close = tokio::signal::windows::ctrl_close()?;
    let mut shutdown = tokio::signal::windows::ctrl_shutdown()?;
    let mut interrupt = tokio::signal::windows::ctrl_break()?;
    Ok(async move {
        tokio::select! {
            _ = close.recv() => {},
            _ = shutdown.recv() => {},
            _ = interrupt.recv() => {},
        }
    })
}
async fn run(
    stage: &mut &'static str,
    diagnostics: &mut Option<std::sync::Arc<kr_tunnel::udp_diagnostics::Counters>>,
) -> io::Result<()> {
    if std::env::args_os().len() != 1 {
        return Err(io::Error::other("private launcher stdin only"));
    }
    let mut input = Zeroizing::new(Vec::new());
    io::stdin().lock().take(8193).read_to_end(&mut input)?;
    if input.len() > 8192 {
        return Err(io::Error::other("private launcher input bound"));
    }
    let request: Request = serde_json::from_slice(&input)
        .map_err(|_| io::Error::other("invalid private launcher request"))?;
    drop(input);
    let offer = request.peer_transport;
    if offer.endpoint.port() == 0
        || offer.endpoint.ip().is_unspecified()
        || offer.endpoint.ip().is_multicast()
        || offer.endpoint.ip().is_broadcast()
        || offer.certificate_der.is_empty()
        || offer.certificate_der.len() > 4096
        || !(26..=32768).contains(&request.port_budget)
        || request.native_ip.is_unspecified()
        || request.native_ip.is_multicast()
    {
        return Err(io::Error::other("private launcher transport bounds"));
    }
    *stage = "endpoint_setup";
    let address = SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 0).into();
    let (endpoint, counters) = kr_tunnel::client_configured(
        address,
        offer.certificate_der.into(),
        request.udp_backend,
        request.diagnostics,
    )?;
    if let Some(counters) = &counters {
        kr_tunnel::udp_diagnostics::install_redacted_log(counters.clone())?;
    }
    *diagnostics = counters;
    *stage = "transport_authentication";
    let link = Link::connect_observed(
        &endpoint,
        offer.endpoint.into(),
        &offer.capability.0,
        |label| *stage = label,
    )
    .await?;
    if let Some(counters) = diagnostics.as_ref() {
        report_counters(counters, "authenticated");
    }
    drop(offer.capability);
    let (stop, stopped) = watch::channel(false);
    *stage = "termination_setup";
    let terminate = termination_signal()?;
    tokio::pin!(terminate);
    let task = companion::run(link, request.native_ip, request.port_budget, stopped);
    tokio::pin!(task);
    // This means authenticated transport only. Round readiness is acknowledged
    // separately on the control stream before the service builds native 153.
    writeln!(io::stdout(), "{{\"event\":\"transport_ready\"}}")?;
    io::stdout().flush()?;
    *stage = "transport_running";
    let result = tokio::select! {
        result=&mut task=>result,
        result=tokio::signal::ctrl_c()=>{result?;let _=stop.send(true);task.await},
        _=&mut terminate=>{let _=stop.send(true);task.await},
    };
    endpoint.close(0_u8.into(), b"launcher child ended");
    result
}
#[tokio::main]
async fn main() {
    let mut stage = "installation";
    let mut diagnostics = None;
    let result = run(&mut stage, &mut diagnostics).await;
    if let Some(counters) = diagnostics {
        report_counters(&counters, "stopped");
    }
    if let Err(error) = result {
        // Do not forward arbitrary QUIC/I/O errors, endpoints or capabilities.
        let os_error = error
            .raw_os_error()
            .map_or("null".to_owned(), |code| code.to_string());
        let transport_error = error
            .get_ref()
            .and_then(|inner| inner.downcast_ref::<quinn::ConnectionError>())
            .map_or(
                if error.kind() == io::ErrorKind::TimedOut {
                    "local_deadline"
                } else {
                    "unclassified"
                },
                |error| match error {
                    quinn::ConnectionError::VersionMismatch => "version_mismatch",
                    quinn::ConnectionError::TransportError(_) => "transport",
                    quinn::ConnectionError::ConnectionClosed(_) => "connection_closed",
                    quinn::ConnectionError::ApplicationClosed(_) => "application_closed",
                    quinn::ConnectionError::Reset => "reset",
                    quinn::ConnectionError::TimedOut => "timed_out",
                    quinn::ConnectionError::LocallyClosed => "locally_closed",
                    quinn::ConnectionError::CidsExhausted => "cids_exhausted",
                },
            );
        eprintln!(
            "{{\"event\":\"companion_failed\",\"stage\":\"{stage}\",\"os_error\":{os_error},\"transport_error\":\"{transport_error}\"}}"
        );
        std::process::exit(1);
    }
}
fn report_counters(counters: &kr_tunnel::udp_diagnostics::Counters, phase: &'static str) {
    if let Ok(encoded) = serde_json::to_string(&counters.snapshot()) {
        eprintln!(
            "{{\"event\":\"companion_udp_counters\",\"phase\":\"{phase}\",\"counters\":{encoded}}}"
        );
    }
}
