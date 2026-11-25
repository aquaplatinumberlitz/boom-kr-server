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

/// Authenticated launcher session used for follow-up RPCs such as `web_shop_token`.
pub(crate) struct LoginSession {
    pub(crate) player_id: i64,
    pub(crate) session: Zeroizing<String>,
}

/// Authenticate against the launcher TLS `login` endpoint to obtain `player_id` and `session`.
pub(crate) fn login(config: &Config, login_name: &str, password: &str) -> Result<LoginSession, ()> {
    let login_name = login_name.trim();
    if login_name.is_empty() || password.is_empty() {
        return Err(());
    }
    #[derive(Serialize)]
    struct Request<'a> {
        op: &'static str,
        login: &'a str,
        password: &'a str,
    }
    #[derive(Deserialize)]
    struct PlayerInfo {
        player_id: i64,
    }
    #[derive(Deserialize)]
    struct Response {
        ok: bool,
        player: Option<PlayerInfo>,
        session: Option<String>,
    }
    let mut data = Zeroizing::new(
        serde_json::to_vec(&Request {
            op: "login",
            login: login_name,
            password,
        })
        .map_err(|_| ())?,
    );
    data.push(b'\n');
    let response: Response = tls_rpc(config, &data)?;
    drop(data);
    if !response.ok {
        return Err(());
    }
    let player_id = response.player.ok_or(())?.player_id;
    let session = Zeroizing::new(response.session.ok_or(())?);
    if player_id < 1 || !validate_hex_token(&session) {
        return Err(());
    }
    Ok(LoginSession { player_id, session })
}

/// Request a short-lived F4-d HTTP bearer token bound to an active launcher session.
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

/// Log in via launcher TLS, exchange a `web_shop_token`, and return the validated `#token=...` URL.
#[cfg(test)]
pub(crate) fn fetch_web_shop_url(
    config: &Config,
    login_name: &str,
    password: &str,
) -> Result<Zeroizing<String>, ()> {
    if config.web_shop_url().is_none() {
        return Err(());
    }
    let sess = login(config, login_name, password)?;
    let token = web_shop_token(config, &sess.session, sess.player_id)?;
    crate::desktop::build_web_shop_url(config, &token).map_err(|_| ())
}

/// Authenticate the player and open the local Web Shop in their default browser.
pub(crate) fn open_web_shop_with_login(
    config: &Config,
    login_name: &str,
    password: &str,
) -> Result<(), ()> {
    let sess = login(config, login_name, password)?;
    let token = web_shop_token(config, &sess.session, sess.player_id)?;
    crate::desktop::open_web_shop(config, &token).map_err(|_| ())
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

    #[test]
    fn login_and_fetch_web_shop_url_round_trip_and_fail_closed() {
        let root = std::env::temp_dir().join(format!("kr-player-ws-login-{}", std::process::id()));
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

        let good_sess = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
        let good_tok = "0123456789abcdef".repeat(4);
        let sess_reply = format!(
            "{{\"ok\":true,\"player\":{{\"account_id\":1,\"player_id\":7,\"wire_id\":1,\"display_name\":\"A\",\"profile_version\":0}},\"session\":\"{good_sess}\",\"expires_at\":9999999999}}\n"
        );
        let tok_reply = format!("{{\"ok\":true,\"token\":\"{good_tok}\"}}\n");
        let bad_reply = "{\"ok\":false,\"error\":\"denied_or_busy\"}\n".to_owned();
        let expected_sess = good_sess.to_owned();

        let handle = thread::spawn(move || {
            for idx in 0..3 {
                let (sock, _) = listener.accept().unwrap();
                let mut stream =
                    launch_tls::accept(sock, tls_server.clone(), Duration::from_secs(3)).unwrap();
                let req_bytes =
                    launch_tls::read_frame(&mut stream, launch_tls::REQUEST_LIMIT).unwrap();
                let req: serde_json::Value = serde_json::from_slice(&req_bytes).unwrap();
                let reply = match idx {
                    0 => {
                        assert_eq!(req["op"], "login");
                        assert_eq!(req["login"], "alice");
                        assert_eq!(req["password"], "pw1234567890");
                        sess_reply.as_bytes()
                    }
                    1 => {
                        assert_eq!(req["op"], "web_shop_token");
                        assert_eq!(req["player_id"], 7);
                        assert_eq!(req["session"], expected_sess);
                        tok_reply.as_bytes()
                    }
                    _ => {
                        assert_eq!(req["op"], "login");
                        bad_reply.as_bytes()
                    }
                };
                launch_tls::write_frame(&mut stream, reply, launch_tls::RESPONSE_LIMIT).unwrap();
            }
        });

        let no_url_cfg = Config::test_tls_config(Some(addr), Some(cert_path.clone()));
        assert!(fetch_web_shop_url(&no_url_cfg, "alice", "pw1234567890").is_err());

        let mut cfg = Config::test_tls_config(Some(addr), Some(cert_path));
        cfg.web_shop_url = Some("http://127.0.0.1:8080/".to_owned());
        let url = fetch_web_shop_url(&cfg, " alice ", "pw1234567890").unwrap();
        assert_eq!(
            url.as_str(),
            format!("http://127.0.0.1:8080/#token={good_tok}")
        );

        assert!(fetch_web_shop_url(&cfg, "alice", "wrong-pass-1234").is_err());

        handle.join().unwrap();
        fs::remove_dir_all(&root).unwrap();
    }
}
