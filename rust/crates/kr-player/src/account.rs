//! Account management over the existing launcher TLS endpoint.
use crate::{Form, Screen, game::Config};
use kr_tunnel::launch_tls;
use serde::{Deserialize, Serialize};
use std::{fs, net::TcpStream, time::Duration};
use zeroize::Zeroizing;

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
    let timeout = Duration::from_secs(3);
    let socket =
        TcpStream::connect_timeout(&config.server.ok_or(())?.into(), timeout).map_err(|_| ())?;
    let mut tls = launch_tls::connect(
        socket,
        fs::read(config.certificate.as_ref().ok_or(())?).map_err(|_| ())?,
        timeout,
    )
    .map_err(|_| ())?;
    launch_tls::write_frame(&mut tls, &data, launch_tls::REQUEST_LIMIT).map_err(|_| ())?;
    drop(data);
    let frame = launch_tls::read_frame(&mut tls, launch_tls::RESPONSE_LIMIT).map_err(|_| ())?;
    let response: Response = serde_json::from_slice(&frame).map_err(|_| ())?;
    let code = response.recovery_code.map(Zeroizing::new);
    if response.ok
        && code.as_ref().is_some_and(|s| {
            s.len() != 64
                || !s
                    .bytes()
                    .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        })
    {
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
