//! Private launcher child. Capabilities arrive only on stdin, never argv/logs.
#![forbid(unsafe_code)]
use kr_tunnel::{Link, Secret, client, companion};
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
}
async fn run() -> io::Result<()> {
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
    let endpoint = client(
        SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 0).into(),
        offer.certificate_der.into(),
    )?;
    let link = Link::connect(&endpoint, offer.endpoint.into(), &offer.capability.0).await?;
    drop(offer.capability);
    let (stop, stopped) = watch::channel(false);
    let mut terminate = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())?;
    let task = companion::run(link, request.native_ip, request.port_budget, stopped);
    tokio::pin!(task);
    // This means authenticated transport only. Round readiness is acknowledged
    // separately on the control stream before the service builds native 153.
    writeln!(io::stdout(), "{{\"event\":\"transport_ready\"}}")?;
    io::stdout().flush()?;
    let result = tokio::select! {
        result=&mut task=>result,
        result=tokio::signal::ctrl_c()=>{result?;let _=stop.send(true);task.await},
        _=terminate.recv()=>{let _=stop.send(true);task.await},
    };
    endpoint.close(0_u8.into(), b"launcher child ended");
    result
}
#[tokio::main]
async fn main() {
    if let Err(error) = run().await {
        eprintln!("companion stopped: {error}");
        std::process::exit(1);
    }
}
