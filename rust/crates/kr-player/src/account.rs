//! Account management and web shop token RPCs over the launcher TLS endpoint.
use crate::{Form, Screen, game::Config};
use kr_tunnel::launch_tls;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::{fs, net::TcpStream, time::Duration};
use zeroize::Zeroizing;

const HEX_TOKEN_LEN: usize = 64;

pub(crate) fn validate_hex_token(token: &str) -> bool {
    token.len() == HEX_TOKEN_LEN
        && token
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

fn tls_rpc<T: DeserializeOwned>(config: &Config, data: &[u8]) -> Result<T, ()> {
    let timeout = Duration::from_secs(3);
    let socket =
        TcpStream::connect_timeout(&config.server.ok_or(())?.into(), timeout).map_err(|_| ())?;
    let mut tls = launch_tls::connect(
        socket,
        fs::read(config.certificate.as_ref().ok_or(())?).map_err(|_| ())?,
        timeout,
    )
    .map_err(|_| ())?;
    launch_tls::write_frame(&mut tls, data, launch_tls::REQUEST_LIMIT).map_err(|_| ())?;
    let frame = launch_tls::read_frame(&mut tls, launch_tls::RESPONSE_LIMIT).map_err(|_| ())?;
    serde_json::from_slice(&frame).map_err(|_| ())
}

pub(crate) fn account(
    config: &Config,
    screen: Screen,
    form: Form,
) -> Result<Option<Zeroizing<String>>, ()> {
    #[derive(Serialize)]
    #[serde(tag = "kind", rename_all = "snake_case")]
    enum Action<'a> {
        Register {
            display_name: &'a str,
            password: &'a str,
        },
        Change {
            current: &'a str,
            password: &'a str,
        },
        Delete {
            current: &'a str,
        },
        Recover {
            code: &'a str,
            password: &'a str,
        },
    }
    #[derive(Serialize)]
    struct Request<'a> {
        op: &'static str,
        login: &'a str,
        action: Action<'a>,
    }
    #[derive(Deserialize)]
    struct Response {
        ok: bool,
        recovery_code: Option<String>,
    }
    let action = match screen {
        Screen::Register => Action::Register {
            display_name: &form.display_name,
            password: &form.password,
        },
        Screen::Change => Action::Change {
            current: &form.current,
            password: &form.password,
        },
        Screen::Delete => Action::Delete {
            current: &form.current,
        },
        Screen::Recover => Action::Recover {
            code: form.code.trim(),
            password: &form.password,
        },
        _ => return Err(()),
    };
    let mut data = Zeroizing::new(
        serde_json::to_vec(&Request {
            op: "account",
            login: form.login.trim(),
            action,
        })
        .map_err(|_| ())?,
    );
    data.push(b'\n');
    let response: Response = tls_rpc(config, &data)?;
    drop(data);
    let code = response.recovery_code.map(Zeroizing::new);
    if response.ok && code.as_ref().is_some_and(|s| !validate_hex_token(s)) {
        return Err(());
    }
    let code_matches = match screen {
        Screen::Register | Screen::Recover => code.is_some(),
        Screen::Change => true,
        _ => code.is_none(),
    };
    if response.ok && code_matches {
        Ok(code)
    } else {
        Err(())
    }
}

/// Request a short-lived F4-d HTTP bearer token bound to an active launcher session.
// Consumed by unit tests now and launcher WebShop UI in Wave 4.
#[allow(dead_code)]
pub(crate) fn web_shop_token(
    config: &Config,
    session: &str,
    player_id: i64,
) -> Result<Zeroizing<String>, ()> {
    if player_id < 1 || session.trim().is_empty() {
        return Err(());
    }
    #[derive(Serialize)]
    struct Request<'a> {
        op: &'static str,
        session: &'a str,
        player_id: i64,
    }
    #[derive(Deserialize)]
    struct Response {
        ok: bool,
        token: Option<String>,
    }
    let mut data = Zeroizing::new(
        serde_json::to_vec(&Request {
            op: "web_shop_token",
            session,
            player_id,
        })
        .map_err(|_| ())?,
    );
    data.push(b'\n');
    let response: Response = tls_rpc(config, &data)?;
    drop(data);
    if !response.ok {
        return Err(());
    }
    let token = Zeroizing::new(response.token.ok_or(())?);
    if !validate_hex_token(&token) {
        return Err(());
    }
    Ok(token)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{net::TcpListener, thread};

    #[test]
    fn hex_token_validation_and_web_shop_rpc_fail_closed() {
        assert!(validate_hex_token(&"a".repeat(64)));
        assert!(!validate_hex_token(&"A".repeat(64)));
        assert!(!validate_hex_token(&"a".repeat(63)));
        assert!(!validate_hex_token(&"g".repeat(64)));

        let root = std::env::temp_dir().join(format!("kr-player-ws-rpc-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir(&root).unwrap();
        let (cert, key) = launch_tls::generate_identity().unwrap();
        let cert_path = root.join("cert.der");
        fs::write(&cert_path, &cert).unwrap();
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let std::net::SocketAddr::V4(addr) = listener.local_addr().unwrap() else {
            unreachable!();
        };
        let tls_server = launch_tls::server_config(cert, key.to_vec()).unwrap();

        let good_token = "0123456789abcdef".repeat(4);
        let responses = vec![
            format!("{{\"ok\":true,\"token\":\"{good_token}\"}}\n"),
            "{\"ok\":false,\"error\":\"denied_or_busy\"}\n".into(),
            "{\"ok\":true,\"token\":\"UPPERCASE_BAD\"}\n".into(),
        ];
        let handle = thread::spawn(move || {
            for reply in responses {
                let (sock, _) = listener.accept().unwrap();
                let mut stream =
                    launch_tls::accept(sock, tls_server.clone(), Duration::from_secs(3)).unwrap();
                let req_bytes =
                    launch_tls::read_frame(&mut stream, launch_tls::REQUEST_LIMIT).unwrap();
                let req: serde_json::Value = serde_json::from_slice(&req_bytes).unwrap();
                assert_eq!(req["op"], "web_shop_token");
                assert_eq!(req["player_id"], 7);
                launch_tls::write_frame(&mut stream, reply.as_bytes(), launch_tls::RESPONSE_LIMIT)
                    .unwrap();
            }
        });

        let cfg = Config::test_tls_config(Some(addr), Some(cert_path));
        // Client-side preflight rejects empty session or non-positive player_id without hitting socket.
        assert!(web_shop_token(&cfg, "", 7).is_err());
        assert!(web_shop_token(&cfg, "sess", 0).is_err());

        let token = web_shop_token(&cfg, "session-secret", 7).unwrap();
        assert_eq!(token.as_str(), good_token);
        assert!(web_shop_token(&cfg, "session-secret", 7).is_err());
        assert!(web_shop_token(&cfg, "session-secret", 7).is_err());

        handle.join().unwrap();
        fs::remove_dir_all(&root).unwrap();
    }
}
