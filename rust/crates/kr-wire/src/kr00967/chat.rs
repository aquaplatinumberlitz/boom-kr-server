//! Minimum patched949x room-chat adapter, not full social/chat protocol.
//! Request18: CRC-verified owner y9/y10 captures, u16 string only (unlike VN).
//! Reply28: KR CChatStage 0xD41DF0, string then u8 style0; native reply pending.
use super::{decode_bytes, encode_bytes, put, text949x, uint};
use crate::Width;
use std::io;

// Company policy budgets, not recovered limits of the original KR server/client.
pub const MAX_ROOM_CHAT_BYTES: usize = 192;
pub const MAX_ROOM_CHAT_REPLY_BYTES: usize = 224;

fn invalid() -> io::Error {
    io::Error::new(io::ErrorKind::InvalidInput, "unsupported normal room chat")
}
fn safe_literal(text: &str) -> bool {
    !text.is_empty()
        && !text.chars().any(|character| {
            character.is_control()
                || ('\u{202a}'..='\u{202e}').contains(&character)
                || ('\u{2066}'..='\u{2069}').contains(&character)
        })
}

/// Decode a single strict949x string. No flags, count, claimed sender, routing
/// target, or trailing fields are admitted by this measured KR request branch.
pub fn room_chat18(body: &[u8]) -> io::Result<String> {
    if !(3..=MAX_ROOM_CHAT_BYTES + 2).contains(&body.len()) {
        return Err(invalid());
    }
    let length = uint(&body[..2], Width::U16)? as usize;
    if length + 2 != body.len() {
        return Err(invalid());
    }
    let bytes = decode_bytes(&body[2..])?;
    let text = text949x::decode(&bytes)?;
    if !safe_literal(&text) || text949x::encode(&text)? != bytes {
        return Err(invalid());
    }
    Ok(text)
}

/// Builds only literal style0. Service supplies authenticated owner/routing;
/// this encoder never reads sender identity from client text.
pub fn room_chat28_literal(text: &str) -> io::Result<Vec<u8>> {
    if text.len() > MAX_ROOM_CHAT_REPLY_BYTES * 3 || !safe_literal(text) {
        return Err(invalid());
    }
    let bytes = text949x::encode(text)?;
    if bytes.len() > MAX_ROOM_CHAT_REPLY_BYTES {
        return Err(invalid());
    }
    let mut body = put(bytes.len() as u64, Width::U16)?;
    body.extend(encode_bytes(&bytes)?);
    body.extend(put(0, Width::U8)?);
    Ok(body)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hex(text: &str) -> Vec<u8> {
        text.as_bytes()
            .as_chunks::<2>()
            .0
            .iter()
            .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
            .collect()
    }

    #[test]
    fn independently_captured_kr_requests_are_single_strings() {
        // Owner L1 en05/y9/y10 packet-local frame/sequence/CRC verified.
        // Body hashes and original captures are pinned in M8 chat metadata.
        for (raw, expected) in [
            (
                "a548b6a43acdfa0d3bf651a973faf4ea30f8b6a4014771799b2b2a769c5244eab72a08",
                "Xin chào các bạn 안녕하세요 ạ",
            ),
            (
                "a57571799b2b2a769c522feab72a2b63e3482205e755590d734c7275ef29",
                "안녕하세요 반갑습니다 하세요",
            ),
        ] {
            let body = hex(raw);
            assert_eq!(room_chat18(&body).unwrap(), expected);
            for length in 0..body.len() {
                assert!(room_chat18(&body[..length]).is_err());
            }
            let mut extra = body.clone();
            extra.push(0);
            assert!(room_chat18(&extra).is_err());
            let mut wrong_length = body.clone();
            wrong_length[1] ^= 1;
            assert!(room_chat18(&wrong_length).is_err());
        }
    }

    #[test]
    fn literal_reply_uses_native_byte_count_and_style_zero_without_replacements() {
        let text = "Việt01 : Xin chào 한국";
        let body = room_chat28_literal(text).unwrap();
        let size = uint(&body[..2], Width::U16).unwrap() as usize;
        assert_ne!(size, text.len());
        assert_eq!(body.len(), size + 3);
        assert_eq!(
            text949x::decode(&decode_bytes(&body[2..2 + size]).unwrap()).unwrap(),
            text
        );
        assert_eq!(uint(&body[2 + size..], Width::U8).unwrap(), 0);
        for text in ["", "a\0b", "a\nb", "a\u{202e}b", "a😀b"] {
            assert!(room_chat28_literal(text).is_err());
        }
        assert!(room_chat28_literal(&"ệ".repeat(MAX_ROOM_CHAT_REPLY_BYTES / 2 + 1)).is_err());
    }
}
