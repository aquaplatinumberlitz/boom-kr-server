//! Offline retained-oracle decoder; no listener, authentication or secret output.
use kr_service::account_bridge::wire;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::io::{self, Read};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Input {
    kind: String,
    body_hex: String,
    peer: std::net::Ipv4Addr,
}
#[derive(Serialize)]
struct Output {
    ok: bool,
    token_sha256: Option<String>,
    name: Option<String>,
    wire_id: Option<u32>,
    echo: Option<u32>,
}
fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}
fn unhex(text: &str) -> Option<Vec<u8>> {
    if text.len() > 256 || !text.len().is_multiple_of(2) || !text.is_ascii() {
        return None;
    }
    (0..text.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&text[i..i + 2], 16).ok())
        .collect()
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut input = String::new();
    io::stdin().take(65_537).read_to_string(&mut input)?;
    if input.len() > 65_536 {
        return Err("input bound".into());
    }
    let cases: Vec<Input> = serde_json::from_str(&input)?;
    if cases.len() > 128 {
        return Err("case bound".into());
    }
    let outputs: Vec<_> = cases
        .into_iter()
        .map(|case| {
            let mut result = Output {
                ok: false,
                token_sha256: None,
                name: None,
                wire_id: None,
                echo: None,
            };
            if let Some(body) = unhex(&case.body_hex) {
                if case.kind == "login" {
                    if let Ok(token) = wire::login_token(&body, case.peer) {
                        result.ok = true;
                        result.token_sha256 =
                            Some(hex(&Sha256::digest(token.expose_secret().as_bytes())));
                    }
                } else if case.kind == "channel"
                    && let Ok(identity) = wire::channel_identity(&body)
                {
                    result.ok = true;
                    result.name = Some(String::from_utf8(identity.name).expect("ASCII profile"));
                    result.wire_id = Some(identity.wire_id);
                    result.echo = Some(identity.echo);
                }
            }
            result
        })
        .collect();
    println!("{}", serde_json::to_string(&outputs)?);
    Ok(())
}
