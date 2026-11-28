use super::token_digest;
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::io;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use zeroize::Zeroizing;

const MAX_HEADER_BYTES: usize = 8 * 1024;
const MAX_HEADERS: usize = 32;

pub struct Request {
    method: String,
    target: String,
    headers: Vec<(String, String)>,
    body: Zeroizing<Vec<u8>>,
}
impl Request {
    pub fn method(&self) -> &str {
        &self.method
    }
    pub fn path(&self) -> &str {
        self.target.split('?').next().unwrap_or("")
    }
    pub fn query(&self) -> Option<&str> {
        self.target.split_once('?').map(|(_, q)| q)
    }
    pub fn require_method(&self, method: &str) -> Result<(), Status> {
        if self.method == method {
            Ok(())
        } else {
            Err(Status::MethodNotAllowed)
        }
    }
    pub fn require_queryless_get(&self) -> Result<(), Status> {
        self.require_method("GET")?;
        if self.query().is_some() {
            return Err(Status::BadRequest);
        }
        Ok(())
    }
    pub fn digest(&self) -> Option<zeroize::Zeroizing<[u8; 32]>> {
        let value = self
            .headers
            .iter()
            .find(|(name, _)| name == "authorization")?
            .1
            .trim();
        let token = value.strip_prefix("Bearer ")?;
        if token.len() != 64
            || !token
                .bytes()
                .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
        {
            return None;
        }
        let digest = token_digest(token);
        Some(digest)
    }
    pub fn json<T: DeserializeOwned>(&self) -> Result<T, Status> {
        serde_json::from_slice(&self.body).map_err(|_| Status::BadRequest)
    }
    pub fn json_object<T: DeserializeOwned>(&self) -> Result<T, Status> {
        if self.body.iter().copied().find(|b| !b.is_ascii_whitespace()) != Some(b'{') {
            return Err(Status::BadRequest);
        }
        self.json()
    }
}

pub async fn read(stream: &mut TcpStream, max_body: usize) -> Result<Request, Option<Status>> {
    // A read can contain both the headers and part of a credential body.
    let mut data = Zeroizing::new(Vec::with_capacity(1024));
    let mut chunk = Zeroizing::new([0u8; 1024]);
    let header_end = loop {
        if data.len() > MAX_HEADER_BYTES {
            return Err(Some(Status::RequestTooLarge));
        }
        let n = stream.read(chunk.as_mut()).await.map_err(|_| None)?;
        if n == 0 {
            return Err(None);
        }
        data.extend_from_slice(&chunk[..n]);
        if let Some(pos) = data.windows(4).position(|w| w == b"\r\n\r\n") {
            break pos + 4;
        }
    };
    let head = std::str::from_utf8(&data[..header_end]).map_err(|_| Some(Status::BadRequest))?;
    let mut lines = head.split("\r\n");
    let first = lines.next().ok_or(Some(Status::BadRequest))?;
    let mut parts = first.split_whitespace();
    let method = parts.next().ok_or(Some(Status::BadRequest))?;
    let target = parts.next().ok_or(Some(Status::BadRequest))?;
    let version = parts.next().ok_or(Some(Status::BadRequest))?;
    if parts.next().is_some()
        || version != "HTTP/1.1"
        || target.len() > 2048
        || !target.starts_with('/')
    {
        return Err(Some(Status::BadRequest));
    }
    let mut headers = Vec::new();
    let mut content_length = 0usize;
    for line in lines {
        if line.is_empty() {
            continue;
        }
        if headers.len() >= MAX_HEADERS {
            return Err(Some(Status::BadRequest));
        }
        let (name, value) = line.split_once(':').ok_or(Some(Status::BadRequest))?;
        let name = name.trim().to_ascii_lowercase();
        let value = value.trim().to_string();
        if name.is_empty() || !name.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-') {
            return Err(Some(Status::BadRequest));
        }
        if name == "transfer-encoding" && !value.eq_ignore_ascii_case("identity") {
            return Err(Some(Status::NotImplemented));
        }
        if name == "content-length" {
            content_length = value
                .parse::<usize>()
                .map_err(|_| Some(Status::BadRequest))?;
            if content_length > max_body {
                return Err(Some(Status::RequestTooLarge));
            }
        }
        headers.push((name, value));
    }
    let buffered = content_length.min(data.len() - header_end);
    let mut body = Zeroizing::new(Vec::with_capacity(content_length));
    body.extend_from_slice(&data[header_end..header_end + buffered]);
    while body.len() < content_length {
        let need = content_length - body.len();
        let n = stream
            .read(&mut chunk[..need.min(1024)])
            .await
            .map_err(|_| None)?;
        if n == 0 {
            return Err(Some(Status::BadRequest));
        }
        body.extend_from_slice(&chunk[..n]);
    }
    Ok(Request {
        method: method.to_string(),
        target: target.to_string(),
        headers,
        body,
    })
}

#[derive(Clone, Copy)]
pub enum Status {
    Ok,
    BadRequest,
    Unauthorized,
    PaymentRequired,
    Forbidden,
    NotFound,
    Conflict,
    MethodNotAllowed,
    RequestTooLarge,
    NotImplemented,
    TooManyRequests,
    Unavailable,
    Internal,
    UnlockAllNoPurchase,
}
impl Status {
    fn code(self) -> u16 {
        match self {
            Self::Ok => 200,
            Self::BadRequest => 400,
            Self::Unauthorized => 401,
            Self::PaymentRequired => 402,
            Self::Forbidden => 403,
            Self::NotFound => 404,
            Self::MethodNotAllowed => 405,
            Self::RequestTooLarge => 413,
            Self::NotImplemented => 501,
            Self::TooManyRequests => 429,
            Self::Unavailable => 503,
            Self::Conflict | Self::UnlockAllNoPurchase => 409,
            Self::Internal => 500,
        }
    }
    fn phrase(self) -> &'static str {
        match self {
            Self::Ok => "OK",
            Self::BadRequest => "Bad Request",
            Self::Unauthorized => "Unauthorized",
            Self::PaymentRequired => "Payment Required",
            Self::Forbidden => "Forbidden",
            Self::NotFound => "Not Found",
            Self::MethodNotAllowed => "Method Not Allowed",
            Self::RequestTooLarge => "Payload Too Large",
            Self::NotImplemented => "Not Implemented",
            Self::TooManyRequests => "Too Many Requests",
            Self::Unavailable => "Service Unavailable",
            Self::Conflict | Self::UnlockAllNoPurchase => "Conflict",
            Self::Internal => "Internal Server Error",
        }
    }
}

pub struct Response {
    status: Status,
    content_type: &'static str,
    body: Vec<u8>,
    csp: bool,
}
impl Response {
    pub fn json<T: Serialize>(value: &T) -> Self {
        Self::json_with_status(Status::Ok, value)
    }
    pub fn json_with_status<T: Serialize>(status: Status, value: &T) -> Self {
        Self {
            status,
            content_type: "application/json; charset=utf-8",
            body: serde_json::to_vec(value).unwrap_or_else(|_| b"{}".to_vec()),
            csp: false,
        }
    }
    pub fn html(value: &str) -> Self {
        Self {
            status: Status::Ok,
            content_type: "text/html; charset=utf-8",
            body: value.as_bytes().to_vec(),
            csp: true,
        }
    }
    pub fn error(status: Status) -> Self {
        let error = match status {
            Status::UnlockAllNoPurchase => "unlock_all_no_purchase",
            Status::Unauthorized => "unauthorized",
            _ => "request_failed",
        };
        Self {
            status,
            content_type: "application/json; charset=utf-8",
            body: serde_json::json!({"error": error}).to_string().into_bytes(),
            csp: false,
        }
    }
    pub async fn write(self, stream: &mut TcpStream) -> io::Result<()> {
        let mut headers = format!(
            "HTTP/1.1 {} {}\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\nCache-Control: no-store\r\nX-Content-Type-Options: nosniff\r\n",
            self.status.code(),
            self.status.phrase(),
            self.content_type,
            self.body.len()
        );
        if self.csp {
            headers.push_str("Content-Security-Policy: default-src 'none'; script-src 'unsafe-inline'; style-src 'unsafe-inline'; connect-src 'self'\r\n");
        }
        headers.push_str("\r\n");
        stream.write_all(headers.as_bytes()).await?;
        stream.write_all(&self.body).await
    }
}
