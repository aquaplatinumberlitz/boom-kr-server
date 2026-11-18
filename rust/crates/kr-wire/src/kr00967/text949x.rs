//! Strict text boundary for the owner's patched 949x clients, version 1.
//! CP949 plus the append-only extension in tools/l10n_949x.py. This is not UTF-8
//! on the game wire and does not assert support in an unpatched KR executable.
use encoding_rs::EUC_KR;
use std::io;

#[path = "text949x_table.rs"]
mod table;

fn invalid() -> io::Error {
    io::Error::new(io::ErrorKind::InvalidInput, "unsupported 949x text")
}
fn trail(index: usize) -> u8 {
    match index {
        0..26 => b'A' + index as u8,
        26..52 => b'a' + (index - 26) as u8,
        _ => 0x81 + (index - 52) as u8,
    }
}
fn extension_index(lead: u8, byte: u8) -> Option<usize> {
    if !(0xc7..=0xfe).contains(&lead) {
        return None;
    }
    let position = match byte {
        b'A'..=b'Z' => usize::from(byte - b'A'),
        b'a'..=b'z' => 26 + usize::from(byte - b'a'),
        0x81..=0xa0 => 52 + usize::from(byte - 0x81),
        _ => return None,
    };
    Some(usize::from(lead - 0xc7) * 84 + position).filter(|index| *index < table::EXT.len())
}

/// No replacement characters, lossy truncation, or Unicode normalization.
pub fn encode(text: &str) -> io::Result<Vec<u8>> {
    let mut output = Vec::with_capacity(text.len());
    for character in text.chars() {
        if character.is_ascii() {
            output.push(character as u8);
        } else if let Ok(index) = table::EXT.binary_search(&character) {
            output.extend([0xc7 + (index / 84) as u8, trail(index % 84)]);
        } else {
            let mut buffer = [0; 4];
            let text = character.encode_utf8(&mut buffer);
            let (bytes, _, errors) = EUC_KR.encode(text);
            if errors || bytes.len() != 2 {
                return Err(invalid());
            }
            output.extend_from_slice(&bytes);
        }
    }
    Ok(output)
}

pub fn decode(bytes: &[u8]) -> io::Result<String> {
    let mut output = String::with_capacity(bytes.len());
    let mut position = 0;
    while position < bytes.len() {
        let lead = bytes[position];
        if lead < 0x80 {
            output.push(char::from(lead));
            position += 1;
        } else {
            let pair = bytes.get(position..position + 2).ok_or_else(invalid)?;
            if let Some(index) = extension_index(lead, pair[1]) {
                output.push(table::EXT[index]);
            } else {
                let text = EUC_KR
                    .decode_without_bom_handling_and_without_replacement(pair)
                    .ok_or_else(invalid)?;
                // WHATWG compatibility aliases must not introduce a second
                // byte spelling of an account name or accept UTF-8 by accident.
                if encode(&text)? != pair {
                    return Err(invalid());
                }
                output.push_str(&text);
            }
            position += 2;
        }
    }
    Ok(output)
}

/// Company account policy, bounded in native bytes rather than UTF-8 bytes.
/// Combining accents can follow a letter/digit; whitespace/control/bidi and
/// unrepresentable characters cannot become a player name.
pub fn encode_name(name: &str) -> io::Result<Vec<u8>> {
    // Representable characters are BMP and need at most three UTF-8 bytes;
    // this cheap bound precedes conversion/allocation for untrusted input.
    if name.is_empty() || name.len() > 45 {
        return Err(invalid());
    }
    let mut base = false;
    for character in name.chars() {
        if character.is_alphanumeric() || character == '_' {
            base = true;
        } else if !base || !('\u{0300}'..='\u{036f}').contains(&character) {
            return Err(invalid());
        }
    }
    let bytes = encode(name)?;
    if !(3..=15).contains(&bytes.len()) {
        return Err(invalid());
    }
    Ok(bytes)
}

pub fn validate_name(bytes: &[u8]) -> io::Result<()> {
    if !(3..=15).contains(&bytes.len()) || encode_name(&decode(bytes)?)? != bytes {
        return Err(invalid());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn python_reference_vectors_and_byte_bounds() {
        // Independent vectors from Python's cp949 plus owner VERSION=1 table.
        let cases: &[(&str, &[u8])] = &[
            ("KRLocal", b"KRLocal"),
            ("한국", &[0xc7, 0xd1, 0xb1, 0xb9]),
            ("Việt", &[0x56, 0x69, 0xd2, 0x84, 0x74]),
        ];
        for (text, expected) in cases {
            assert_eq!(encode(text).unwrap(), *expected);
            assert_eq!(decode(expected).unwrap(), *text);
            assert_eq!(encode_name(text).unwrap(), *expected);
        }
        assert!(encode_name("ệ".repeat(7).as_str()).is_ok());
        assert!(encode_name("ệ".repeat(8).as_str()).is_err());
        for bad in [
            "",
            "ab",
            "a b",
            "a\0b",
            "a\nb",
            "a\u{202e}b",
            "a😀b",
            "\u{0301}abc",
        ] {
            assert!(encode_name(bad).is_err(), "{bad:?}");
        }
        assert!(encode_name("Vie\u{0323}t").is_ok());
        for bytes in [
            &[0xff][..],
            &[0xc7],
            &[0x80, 0x41],
            &[0xc7, 0x20],
            &[0xfe, 0xa0],
        ] {
            assert!(decode(bytes).is_err(), "{bytes:?}");
        }
    }

    #[test]
    fn all_extension_slots_roundtrip_without_cp949_collisions() {
        assert!(table::EXT.windows(2).all(|pair| pair[0] < pair[1]));
        for (index, character) in table::EXT.iter().enumerate() {
            let bytes = [0xc7 + (index / 84) as u8, trail(index % 84)];
            assert_eq!(encode(&character.to_string()).unwrap(), bytes);
            assert_eq!(decode(&bytes).unwrap(), character.to_string());
            assert!(
                EUC_KR
                    .decode_without_bom_handling_and_without_replacement(&bytes)
                    .is_none()
            );
        }
    }

    #[test]
    fn independent_full_python_oracle_when_requested() {
        let Some(path) = std::env::var_os("KR_TEXT949X_ORACLE") else {
            return;
        };
        let raw = std::fs::read(path).unwrap();
        assert_eq!(raw.len() % 6, 0);
        assert!(raw.len() / 6 > 19000);
        for row in raw.as_chunks::<6>().0 {
            let character =
                char::from_u32(u32::from_le_bytes(row[..4].try_into().unwrap())).unwrap();
            let expected = if row[5] == 0 { &row[4..5] } else { &row[4..6] };
            assert_eq!(
                encode(&character.to_string()).unwrap(),
                expected,
                "U+{:04x}",
                character as u32
            );
            assert_eq!(decode(expected).unwrap(), character.to_string());
        }
    }
}
