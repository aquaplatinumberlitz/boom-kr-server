use data_encoding::{BASE64URL_NOPAD, Encoding, Specification};
use iroh::EndpointId;
use sha2::{Digest, Sha256};
use std::{fmt, str::FromStr, sync::LazyLock};

pub const VERSION: u16 = 1;

static ROOM32: LazyLock<Encoding> = LazyLock::new(|| {
    let mut spec = Specification::new();
    spec.symbols.push_str("0123456789ABCDEFGHJKMNPQRSTVWXYZ");
    spec.translate.from.push_str("abcdefghjkmnpqrstvwxyzOoIiLl");
    spec.translate.to.push_str("ABCDEFGHJKMNPQRSTVWXYZ001111");
    spec.encoding().expect("Crockford base32 alphabet")
});

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Invite {
    Room([u8; 10]),
    Server {
        node: EndpointId,
        tls_sha256: [u8; 32],
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CodeError {
    Format,
    Version(u16),
}

impl fmt::Display for CodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Format => f.write_str("invalid room or server invite code"),
            Self::Version(version) => write!(
                f,
                "unsupported code version {version}; download the current package"
            ),
        }
    }
}

impl std::error::Error for CodeError {}

impl Invite {
    pub fn room(node: EndpointId) -> Self {
        let digest = Sha256::new()
            .chain_update(b"boom-kr/p2p/O1/v1\0")
            .chain_update(node.as_bytes())
            .finalize();
        Self::Room(digest[..10].try_into().expect("80-bit commitment"))
    }

    /// Use the exact DER leaf certificate pinned by the existing TLS client.
    pub fn server(node: EndpointId, tls_der: &[u8]) -> Self {
        Self::Server {
            node,
            tls_sha256: Sha256::digest(tls_der).into(),
        }
    }

    pub fn matches_node(&self, node: EndpointId) -> bool {
        match self {
            Self::Room(_) => *self == Self::room(node),
            Self::Server { node: expected, .. } => *expected == node,
        }
    }
}

impl fmt::Display for Invite {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Room(hash) => {
                let body = ROOM32.encode(hash);
                write!(
                    f,
                    "O1-{VERSION}-{}-{}-{}-{}",
                    &body[..4],
                    &body[4..8],
                    &body[8..12],
                    &body[12..]
                )
            }
            Self::Server { node, tls_sha256 } => {
                let mut body = [0; 64];
                body[..32].copy_from_slice(node.as_bytes());
                body[32..].copy_from_slice(tls_sha256);
                write!(f, "S1-{VERSION}-{}", BASE64URL_NOPAD.encode(&body))
            }
        }
    }
}

impl FromStr for Invite {
    type Err = CodeError;

    fn from_str(code: &str) -> Result<Self, Self::Err> {
        let (kind, rest) = code.trim().split_once('-').ok_or(CodeError::Format)?;
        let (version, body) = rest.split_once('-').ok_or(CodeError::Format)?;
        if !matches!(kind, "O1" | "S1") || !version.bytes().all(|c| c.is_ascii_digit()) {
            return Err(CodeError::Format);
        }
        let version = version.parse::<u16>().map_err(|_| CodeError::Format)?;
        if version != VERSION {
            return Err(CodeError::Version(version));
        }
        match kind {
            "O1" => {
                let body = body.as_bytes();
                if body.len() != 19 || [4, 9, 14].iter().any(|&i| body[i] != b'-') {
                    return Err(CodeError::Format);
                }
                let mut symbols = [0; 16];
                for i in 0..4 {
                    symbols[i * 4..i * 4 + 4].copy_from_slice(&body[i * 5..i * 5 + 4]);
                }
                let hash = ROOM32.decode(&symbols).map_err(|_| CodeError::Format)?;
                Ok(Self::Room(hash.try_into().map_err(|_| CodeError::Format)?))
            }
            "S1" => {
                if body.len() != 86 {
                    return Err(CodeError::Format);
                }
                let body = BASE64URL_NOPAD
                    .decode(body.as_bytes())
                    .map_err(|_| CodeError::Format)?;
                let node =
                    EndpointId::from_bytes(body[..32].try_into().map_err(|_| CodeError::Format)?)
                        .map_err(|_| CodeError::Format)?;
                let tls_sha256 = body[32..].try_into().map_err(|_| CodeError::Format)?;
                Ok(Self::Server { node, tls_sha256 })
            }
            _ => unreachable!(),
        }
    }
}
