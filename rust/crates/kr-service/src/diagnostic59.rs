//! Measured record envelope, not recovered diagnostic purpose or server policy.
//! Native Forest/Patrit type1/count5:50 bytes. All record contents stay opaque.
use flate2::{Decompress, FlushDecompress, Status};
use kr_wire::{Width, read_uint};
use std::io;

pub(crate) fn measured(body: &[u8]) -> io::Result<()> {
    let error = || io::Error::other("unmeasured diagnostic59 envelope");
    if !(23..=96).contains(&body.len())
        || read_uint(&body[..1], Width::U8).map_err(io::Error::other)? != 1
        || read_uint(&body[1..3], Width::U16).map_err(io::Error::other)? != 5
    {
        return Err(error());
    }
    let size = read_uint(&body[3..7], Width::U32).map_err(io::Error::other)? as usize;
    if size != body.len() - 7 {
        return Err(error());
    }
    let stream = crate::lobby::decode_bytes(&body[7..])?;
    if stream.len() < 16
        || stream[0] != 24
        || u32::from_le_bytes(stream[1..5].try_into().expect("bounded")) as usize
            != stream.len() - 5
        || stream[5] != 1
        || u32::from_le_bytes(stream[6..10].try_into().expect("bounded")) != 50
    {
        return Err(error());
    }
    // Fixed allocation and one extra output byte detect expansion beyond50.
    // Require one complete stream, checksum and exact consumption, no trailer.
    let compressed = &stream[10..];
    let mut decoded = [0_u8; 51];
    let mut decoder = Decompress::new(true);
    let status = decoder
        .decompress(compressed, &mut decoded, FlushDecompress::Finish)
        .map_err(io::Error::other)?;
    if status != Status::StreamEnd
        || decoder.total_in() != compressed.len() as u64
        || decoder.total_out() != 50
    {
        return Err(error());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use flate2::{Compression, write::ZlibEncoder};
    use kr_wire::write_uint;
    use std::io::Write;

    fn bytes(value: &str) -> Vec<u8> {
        value
            .as_bytes()
            .as_chunks::<2>()
            .0
            .iter()
            .map(|x| u8::from_str_radix(std::str::from_utf8(x).unwrap(), 16).unwrap())
            .collect()
    }
    fn body(inner: &[u8], advertised: u32, trailer: bool) -> Vec<u8> {
        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(inner).unwrap();
        let mut compressed = encoder.finish().unwrap();
        if trailer {
            compressed.push(0);
        }
        let mut stream = vec![24];
        stream.extend((5 + compressed.len() as u32).to_le_bytes());
        stream.push(1);
        stream.extend(advertised.to_le_bytes());
        stream.extend(compressed);
        let mut encoded = Vec::new();
        let mut tail = stream.as_slice();
        while !tail.is_empty() {
            let (size, width) = if tail.len() >= 4 {
                (4, Width::U32)
            } else if tail.len() >= 2 {
                (2, Width::U16)
            } else {
                (1, Width::U8)
            };
            let mut value = [0; 8];
            value[..size].copy_from_slice(&tail[..size]);
            encoded.extend(write_uint(u64::from_le_bytes(value), width).unwrap());
            tail = &tail[size..];
        }
        let mut result = write_uint(1, Width::U8).unwrap();
        result.extend(write_uint(5, Width::U16).unwrap());
        result.extend(write_uint(encoded.len() as u64, Width::U32).unwrap());
        result.extend(encoded);
        result
    }
    #[test]
    fn captured_native59_profiles_accept_but_other_headers_partial_trailing_and_expansion_fail() {
        // Independent actual TCP payloads from Patrit8-r2, not this encoder.
        for hex in [
            "5ba56c96ca53bf96ca768d96f852954cb25395560a9cee94ca9905ce0865fe37d2abfaa6c81a39dae977039e0953835b6c",
            "5ba56c96ca53bc96ca778d96f852954cb25395560a9dee94ca99053aab184f3aab48e8c6462f8eac720e4390ae9695be",
            "5ba56c96ca53bf96ca768d96f852954cb25395560a952e94ca9905ce08649e37d2ab328ecb6d39b0dbcede3872539e676c",
            // Same batch schema with record55/nonzero value, Patrit8-r3/H.
            "5ba56c96ca53ba96ca798d96f852954cb25395560a9f4e94ca99059ac6cfd4cfcd5f73f0984b8dbbfc975756a4e21421ca5240a38990",
        ] {
            let captured = bytes(hex);
            measured(&captured).unwrap();
            for length in 0..captured.len() {
                assert!(measured(&captured[..length]).is_err());
            }
            for index in 0..7 {
                let mut malformed = captured.clone();
                malformed[index] ^= 1;
                assert!(measured(&malformed).is_err());
            }
            let mut bad_checksum = captured.clone();
            *bad_checksum.last_mut().unwrap() ^= 1;
            assert!(measured(&bad_checksum).is_err());
            let mut oversized = captured.clone();
            oversized.resize(97, 0);
            assert!(measured(&oversized).is_err());
        }
        // Generated bodies are adversarial parser controls, never an oracle.
        let mut records = [0_u8; 50];
        for record in records.as_chunks_mut::<10>().0 {
            record[4] = 101;
        }
        measured(&body(&records, 50, false)).unwrap();
        assert!(measured(&body(&records, 50, true)).is_err());
        for size in [49, 51, 4096] {
            assert!(measured(&body(&vec![0; size], 50, false)).is_err());
        }
        assert!(measured(&body(&records, 51, false)).is_err());
        records[4] = 102;
        measured(&body(&records, 50, false)).unwrap();
        records[4] = 101;
        records[9] = 1;
        measured(&body(&records, 50, false)).unwrap();
    }
}
