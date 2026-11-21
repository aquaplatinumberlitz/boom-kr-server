//! Short-code rendezvous over configured Nostr mailboxes. The mailbox sees
//! only an index, ephemeral signing key and ciphertext; no LAN addresses.
use crate::{Endpoint, EndpointId, Invite, RelayMode, RelayUrl, room::Route};
use data_encoding::{BASE64URL_NOPAD, HEXLOWER};
use nostr_sdk::prelude::*;
use ring::{
    aead, hkdf,
    rand::{SecureRandom, SystemRandom},
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    io,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

const KIND: Kind = Kind::Custom(30078);
const TTL: u64 = 120;
const LIMIT: usize = 2048;
const CONNECT: Duration = Duration::from_secs(3);
const LOOKUP: Duration = Duration::from_secs(4);

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Record {
    node: EndpointId,
    relay: RelayUrl,
    expires: u64,
}

fn clock() -> io::Result<u64> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .map_err(io::Error::other)
}

fn key_error(_: ring::error::Unspecified) -> io::Error {
    io::Error::other("O1 mailbox record")
}

fn index(code: &Invite) -> String {
    HEXLOWER.encode(
        &Sha256::new()
            .chain_update(b"boom-kr/O1/mailbox-index/v1\0")
            .chain_update(code.to_string())
            .finalize(),
    )
}

fn encryption_key(code: &Invite) -> io::Result<aead::LessSafeKey> {
    let salt = hkdf::Salt::new(hkdf::HKDF_SHA256, b"boom-kr/O1/mailbox-key/v1");
    let secret = salt.extract(code.to_string().as_bytes());
    let mut key = [0; 32];
    secret
        .expand(&[b"chacha20-poly1305"], &aead::CHACHA20_POLY1305)
        .map_err(key_error)?
        .fill(&mut key)
        .map_err(key_error)?;
    let result = aead::UnboundKey::new(&aead::CHACHA20_POLY1305, &key).map_err(key_error);
    key.fill(0);
    result.map(aead::LessSafeKey::new)
}

fn seal(code: &Invite, record: &Record) -> io::Result<String> {
    let mut body = serde_json::to_vec(record)?;
    let mut nonce = [0; 12];
    SystemRandom::new().fill(&mut nonce).map_err(key_error)?;
    encryption_key(code)?
        .seal_in_place_append_tag(
            aead::Nonce::assume_unique_for_key(nonce),
            aead::Aad::from(index(code)),
            &mut body,
        )
        .map_err(key_error)?;
    let mut packet = nonce.to_vec();
    packet.extend(body);
    Ok(BASE64URL_NOPAD.encode(&packet))
}

fn open(code: &Invite, encrypted: &str, now: u64) -> io::Result<Route> {
    if !(40..=LIMIT).contains(&encrypted.len()) {
        return Err(io::Error::other("O1 mailbox length"));
    }
    let mut bytes = BASE64URL_NOPAD
        .decode(encrypted.as_bytes())
        .map_err(io::Error::other)?;
    let (nonce, body) = bytes.split_at_mut(12);
    let body = encryption_key(code)?
        .open_in_place(
            aead::Nonce::assume_unique_for_key(nonce.try_into().expect("12-byte nonce")),
            aead::Aad::from(index(code)),
            body,
        )
        .map_err(key_error)?;
    let record: Record = serde_json::from_slice(body)?;
    if !code.matches_node(record.node)
        || !(now.saturating_add(1)..=now.saturating_add(TTL + 15)).contains(&record.expires)
        || !RelayMode::Default.relay_map().contains(&record.relay)
    {
        return Err(io::Error::other("O1 mailbox record expired or mismatched"));
    }
    Ok(Route {
        node: record.node,
        addresses: Vec::new(),
        relay: Some(record.relay),
    })
}

async fn client(relays: &[String]) -> io::Result<Client> {
    if relays.is_empty()
        || relays.len() > 4
        || relays
            .iter()
            .any(|url| !url.starts_with("wss://") || url.len() > 256)
    {
        return Err(io::Error::other("O1 directory relays"));
    }
    let client = Client::new(Keys::generate());
    for url in relays {
        client.add_relay(url).await.map_err(io::Error::other)?;
    }
    client.connect().await;
    client.wait_for_connection(CONNECT).await;
    Ok(client)
}

pub async fn publish(endpoint: Endpoint, relays: Vec<String>) -> io::Result<()> {
    let client = client(&relays).await?;
    let code = Invite::room(endpoint.id());
    loop {
        endpoint.online().await;
        if let Some(relay) = endpoint.addr().relay_urls().next() {
            let expires = clock()?.saturating_add(TTL);
            let record = Record {
                node: endpoint.id(),
                relay: relay.clone(),
                expires,
            };
            let event = EventBuilder::new(KIND, seal(&code, &record)?).tags([
                Tag::identifier(index(&code)),
                Tag::expiration(Timestamp::from(expires)),
            ]);
            let _ = tokio::time::timeout(LOOKUP, client.send_event_builder(event)).await;
        }
        tokio::time::sleep(Duration::from_secs(35)).await;
    }
}

pub async fn resolve(code: &Invite, relays: &[String]) -> io::Result<Route> {
    if !matches!(code, Invite::Room(_)) {
        return Err(io::Error::other("not an O1 code"));
    }
    let client = client(relays).await?;
    let now = clock()?;
    let filter = Filter::new()
        .kind(KIND)
        .identifier(index(code))
        .since(Timestamp::from(now.saturating_sub(TTL + 15)))
        .limit(16);
    let events = tokio::time::timeout(
        LOOKUP + Duration::from_secs(1),
        client.fetch_events(filter, LOOKUP),
    )
    .await
    .map_err(io::Error::other)?;
    client.disconnect().await;
    events
        .map_err(io::Error::other)?
        .iter()
        .take(16)
        .filter(|event| event.content.len() <= LIMIT && event.verify().is_ok())
        .find_map(|event| open(code, &event.content, now).ok())
        .ok_or_else(|| io::Error::other("O1 directory has no current room"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mailbox_hides_node_and_requires_current_room_key_and_routes() {
        let host = crate::SecretKey::generate().public();
        let code = Invite::room(host);
        let other = Invite::room(crate::SecretKey::generate().public());
        let record = Record {
            node: host,
            relay: "https://use1-1.relay.n0.iroh.link.".parse().unwrap(),
            expires: 100 + TTL,
        };
        let payload = seal(&code, &record).unwrap();
        assert!(!payload.contains(&host.to_string()));
        assert!(open(&other, &payload, 100).is_err());
        assert!(open(&code, &payload, 100 + TTL).is_err());
        assert_eq!(open(&code, &payload, 100).unwrap().node, host);
        let mut raw = BASE64URL_NOPAD.decode(payload.as_bytes()).unwrap();
        raw[20] ^= 1;
        assert!(open(&code, &BASE64URL_NOPAD.encode(&raw), 100).is_err());
    }
}
