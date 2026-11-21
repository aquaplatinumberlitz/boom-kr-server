use ring::hmac;
use std::io;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

const NONCE_BYTES: usize = 32;
const TAG_BYTES: usize = 32;
const SERVER: &[u8] = b"kr-local-ipc-v1/server\0";
const CLIENT: &[u8] = b"kr-local-ipc-v1/client\0";

fn nonce() -> io::Result<[u8; NONCE_BYTES]> {
    let mut value = [0; NONCE_BYTES];
    getrandom::fill(&mut value).map_err(io::Error::other)?;
    Ok(value)
}

fn message(role: &[u8], client: &[u8], server: &[u8]) -> Vec<u8> {
    [role, client, server].concat()
}

fn verify(key: &hmac::Key, message: &[u8], tag: &[u8]) -> io::Result<()> {
    hmac::verify(key, message, tag)
        .map_err(|_| io::Error::new(io::ErrorKind::PermissionDenied, "local IPC authentication"))
}

pub(super) async fn server(
    stream: &mut (impl AsyncRead + AsyncWrite + Unpin),
    key: &hmac::Key,
) -> io::Result<()> {
    let mut client = [0; NONCE_BYTES];
    stream.read_exact(&mut client).await?;
    let server = nonce()?;
    stream.write_all(&server).await?;
    stream
        .write_all(hmac::sign(key, &message(SERVER, &client, &server)).as_ref())
        .await?;
    let mut tag = [0; TAG_BYTES];
    stream.read_exact(&mut tag).await?;
    verify(key, &message(CLIENT, &client, &server), &tag)
}

pub(super) async fn client(
    stream: &mut (impl AsyncRead + AsyncWrite + Unpin),
    key: &hmac::Key,
) -> io::Result<()> {
    let client = nonce()?;
    stream.write_all(&client).await?;
    let mut server = [0; NONCE_BYTES];
    let mut tag = [0; TAG_BYTES];
    stream.read_exact(&mut server).await?;
    stream.read_exact(&mut tag).await?;
    verify(key, &message(SERVER, &client, &server), &tag)?;
    stream
        .write_all(hmac::sign(key, &message(CLIENT, &client, &server)).as_ref())
        .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::duplex;

    #[tokio::test]
    async fn mutual_auth_preserves_following_request_bytes() {
        let key = hmac::Key::new(hmac::HMAC_SHA256, &[4; 32]);
        let (mut a, mut b) = duplex(256);
        let (served, ()) = tokio::join!(
            async {
                server(&mut a, &key).await.unwrap();
                let mut request = Vec::new();
                a.read_to_end(&mut request).await.unwrap();
                request
            },
            async {
                client(&mut b, &key).await.unwrap();
                b.write_all(b"operator request").await.unwrap();
                b.shutdown().await.unwrap();
            }
        );
        assert_eq!(served, b"operator request");
    }

    #[tokio::test]
    async fn unrelated_endpoint_cannot_elicit_a_client_proof() {
        let key = hmac::Key::new(hmac::HMAC_SHA256, &[4; 32]);
        let other = hmac::Key::new(hmac::HMAC_SHA256, &[5; 32]);
        let (mut a, mut b) = duplex(256);
        let (rejected, failed) = tokio::join!(async move { client(&mut a, &key).await }, async {
            server(&mut b, &other).await
        });
        assert_eq!(
            rejected.unwrap_err().kind(),
            io::ErrorKind::PermissionDenied
        );
        assert_eq!(failed.unwrap_err().kind(), io::ErrorKind::UnexpectedEof);
    }

    #[tokio::test]
    async fn server_proof_cannot_be_reflected_as_client_proof() {
        let key = hmac::Key::new(hmac::HMAC_SHA256, &[4; 32]);
        let (mut a, mut b) = duplex(256);
        let (result, ()) = tokio::join!(server(&mut a, &key), async {
            b.write_all(&[0; NONCE_BYTES]).await.unwrap();
            let mut challenge = [0; NONCE_BYTES + TAG_BYTES];
            b.read_exact(&mut challenge).await.unwrap();
            b.write_all(&challenge[NONCE_BYTES..]).await.unwrap();
        });
        assert_eq!(result.unwrap_err().kind(), io::ErrorKind::PermissionDenied);
    }
}
