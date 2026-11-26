//! KR 00967 inventory payloads from the static readers and writer (F4 Waves 2–4).
//! See docs/protocol/f4-inventory-cosmetics.md. These codecs operate after frame
//! decryption, using the existing KR field primitives. They do not establish
//! item semantics, route admission, or native compatibility.

use super::{decode_bytes, encode_bytes, put, text949x, uint};
use crate::{FRAME_OVERHEAD, MAX_FRAME_LEN, Width};
use std::{io, num::NonZeroU8};

const MAX_BODY: usize = MAX_FRAME_LEN - FRAME_OVERHEAD;

/// Uninterpreted bytes after the known fields, in their original payload form
/// (after frame decryption, with any field encoding still intact).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct InventoryTail(pub Vec<u8>);

struct Reader<'a>(&'a [u8]);

impl<'a> Reader<'a> {
    fn new(body: &'a [u8]) -> io::Result<Self> {
        if body.len() > MAX_BODY {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "inventory body too large",
            ));
        }
        Ok(Self(body))
    }

    fn take(&mut self, len: usize) -> io::Result<&'a [u8]> {
        let (field, remaining) = self
            .0
            .split_at_checked(len)
            .ok_or(io::ErrorKind::UnexpectedEof)?;
        self.0 = remaining;
        Ok(field)
    }

    fn require_count(&self, count: usize, min_entry_len: usize) -> io::Result<()> {
        if count > self.0.len() / min_entry_len {
            return Err(io::ErrorKind::UnexpectedEof.into());
        }
        Ok(())
    }

    fn u8(&mut self) -> io::Result<u8> {
        Ok(uint(self.take(1)?, Width::U8)? as u8)
    }

    fn u16(&mut self) -> io::Result<u16> {
        Ok(uint(self.take(2)?, Width::U16)? as u16)
    }

    fn u32(&mut self) -> io::Result<u32> {
        Ok(uint(self.take(4)?, Width::U32)? as u32)
    }

    fn u64(&mut self) -> io::Result<u64> {
        uint(self.take(8)?, Width::U64)
    }

    fn text(&mut self) -> io::Result<String> {
        let len = usize::from(self.u16()?);
        text949x::decode(&decode_bytes(self.take(len)?)?)
    }

    fn finish(self) -> io::Result<()> {
        if !self.0.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "trailing inventory bytes",
            ));
        }
        Ok(())
    }

    fn tail(self) -> InventoryTail {
        InventoryTail(self.0.to_vec())
    }
}

#[derive(Default)]
struct Writer(Vec<u8>);

impl Writer {
    fn bytes(&mut self, bytes: &[u8]) -> io::Result<()> {
        if bytes.len() > MAX_BODY - self.0.len() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "inventory body too large",
            ));
        }
        self.0.extend_from_slice(bytes);
        Ok(())
    }

    fn fields(&mut self, fields: &[(u64, Width)]) -> io::Result<()> {
        for &(value, width) in fields {
            self.bytes(&put(value, width)?)?;
        }
        Ok(())
    }

    fn text(&mut self, text: &str) -> io::Result<()> {
        // Bound the temporary conversion as well. A representable character
        // uses at most three UTF-8 bytes and at least one native byte.
        if text.len() > MAX_BODY * 3 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "inventory text too large",
            ));
        }
        let bytes = text949x::encode(text)?;
        let len = u16::try_from(bytes.len()).map_err(io::Error::other)?;
        self.fields(&[(len.into(), Width::U16)])?;
        self.bytes(&encode_bytes(&bytes)?)
    }

    fn with_tail(mut self, tail: &InventoryTail) -> io::Result<Vec<u8>> {
        self.bytes(&tail.0)?;
        Ok(self.0)
    }
}

/// Record read by `sub_11CE930` for 219 and 224. Names follow wire order, since
/// the item fields have no verified gameplay meanings. The `+80` text uses
/// the shared KR u16 byte-length and string codec, without name/chat policy.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ItemRecord11CE930 {
    pub a: u32,
    pub b: u16,
    pub c: u32,
    pub text: String,
    /// Eight payload bytes consumed by reader slot `+88`, kept verbatim.
    pub raw8: [u8; 8],
    pub d: u32,
    pub e: u8,
}

impl ItemRecord11CE930 {
    const MIN_LEN: usize = 25; // Empty text still has a two-byte length.

    pub fn decode(body: &[u8]) -> io::Result<Self> {
        let mut input = Reader::new(body)?;
        let record = Self::read(&mut input)?;
        input.finish()?;
        Ok(record)
    }

    pub fn encode(&self) -> io::Result<Vec<u8>> {
        let mut output = Writer::default();
        self.write(&mut output)?;
        Ok(output.0)
    }

    fn read(input: &mut Reader<'_>) -> io::Result<Self> {
        Ok(Self {
            a: input.u32()?,
            b: input.u16()?,
            c: input.u32()?,
            text: input.text()?,
            raw8: input.take(8)?.try_into().expect("exactly eight bytes"),
            d: input.u32()?,
            e: input.u8()?,
        })
    }

    fn write(&self, output: &mut Writer) -> io::Result<()> {
        output.fields(&[
            (self.a.into(), Width::U32),
            (self.b.into(), Width::U16),
            (self.c.into(), Width::U32),
        ])?;
        output.text(&self.text)?;
        output.bytes(&self.raw8)?;
        output.fields(&[(self.d.into(), Width::U32), (self.e.into(), Width::U8)])
    }
}

/// A selector byte and a u32 count of item records. The selector is retained
/// even for 219, whose callee indexes the container by loop ordinal.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InventoryItemGroup {
    pub selector: u8,
    pub items: Vec<ItemRecord11CE930>,
}

fn decode_groups(body: &[u8]) -> io::Result<Vec<InventoryItemGroup>> {
    let mut input = Reader::new(body)?;
    let count = usize::from(input.u8()?);
    input.require_count(count, 5)?;
    let mut groups = Vec::with_capacity(count);
    for _ in 0..count {
        let selector = input.u8()?;
        let count = input.u32()? as usize;
        input.require_count(count, ItemRecord11CE930::MIN_LEN)?;
        let mut items = Vec::with_capacity(count);
        for _ in 0..count {
            items.push(ItemRecord11CE930::read(&mut input)?);
        }
        groups.push(InventoryItemGroup { selector, items });
    }
    input.finish()?;
    Ok(groups)
}

fn encode_groups(groups: &[InventoryItemGroup]) -> io::Result<Vec<u8>> {
    let mut output = Writer::default();
    let count = u8::try_from(groups.len()).map_err(io::Error::other)?;
    output.fields(&[(count.into(), Width::U8)])?;
    for group in groups {
        let count = u32::try_from(group.items.len()).map_err(io::Error::other)?;
        output.fields(&[
            (group.selector.into(), Width::U8),
            (count.into(), Width::U32),
        ])?;
        for item in &group.items {
            item.write(&mut output)?;
        }
    }
    Ok(output.0)
}

/// S→C 219 (`sub_11CEA10`): counted groups, without the 224 reset.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Inventory219 {
    pub groups: Vec<InventoryItemGroup>,
}

impl Inventory219 {
    pub const OPCODE: u32 = 219;

    pub fn decode(body: &[u8]) -> io::Result<Self> {
        Ok(Self {
            groups: decode_groups(body)?,
        })
    }

    pub fn encode(&self) -> io::Result<Vec<u8>> {
        encode_groups(&self.groups)
    }
}

/// S→C 224 (`sub_11CE4C0`): same envelope as 219; the client resets each
/// selected group before reading its records.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Inventory224 {
    pub groups: Vec<InventoryItemGroup>,
}

impl Inventory224 {
    pub const OPCODE: u32 = 224;

    pub fn decode(body: &[u8]) -> io::Result<Self> {
        Ok(Self {
            groups: decode_groups(body)?,
        })
    }

    pub fn encode(&self) -> io::Result<Vec<u8>> {
        encode_groups(&self.groups)
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Inventory225Slot {
    /// The client only attempts a lookup for a nonzero value.
    pub a: u32,
    /// Read for every slot, including when `a == 0`; meaning unknown.
    pub text: String,
}

/// S→C 225 (`sub_11CE6A0`): a u8 row count followed by eight pairs per row.
/// The client clears those slots before consuming the pairs.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Inventory225 {
    pub rows: Vec<[Inventory225Slot; 8]>,
}

impl Inventory225 {
    pub const OPCODE: u32 = 225;

    pub fn decode(body: &[u8]) -> io::Result<Self> {
        let mut input = Reader::new(body)?;
        let count = usize::from(input.u8()?);
        input.require_count(count, 8 * (4 + 2))?;
        let mut rows = Vec::with_capacity(count);
        for _ in 0..count {
            let mut row = std::array::from_fn(|_| Inventory225Slot::default());
            for slot in &mut row {
                slot.a = input.u32()?;
                slot.text = input.text()?;
            }
            rows.push(row);
        }
        input.finish()?;
        Ok(Self { rows })
    }

    pub fn encode(&self) -> io::Result<Vec<u8>> {
        let mut output = Writer::default();
        let count = u8::try_from(self.rows.len()).map_err(io::Error::other)?;
        output.fields(&[(count.into(), Width::U8)])?;
        for slot in self.rows.iter().flatten() {
            output.fields(&[(slot.a.into(), Width::U32)])?;
            output.text(&slot.text)?;
        }
        Ok(output.0)
    }
}

/// S→C 313 (`sub_D779C0`): unconditional prefix. The tail may start with
/// `Inventory313Lookup` if the client's catalog lookup of `b` succeeded.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Inventory313 {
    pub a: u8,
    pub b: u32,
    pub c: u32,
    pub tail: InventoryTail,
}

impl Inventory313 {
    pub const OPCODE: u32 = 313;

    pub fn decode(body: &[u8]) -> io::Result<Self> {
        let mut input = Reader::new(body)?;
        Ok(Self {
            a: input.u8()?,
            b: input.u32()?,
            c: input.u32()?,
            tail: input.tail(),
        })
    }

    pub fn encode(&self) -> io::Result<Vec<u8>> {
        let mut output = Writer::default();
        output.fields(&[
            (self.a.into(), Width::U8),
            (self.b.into(), Width::U32),
            (self.c.into(), Width::U32),
        ])?;
        output.with_tail(&self.tail)
    }
}

/// The confirmed extra reads on the 313 lookup-hit branch. Decode this only
/// when the caller knows that branch was taken; length alone cannot select it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Inventory313Lookup {
    pub d: u32,
    pub e: u16,
    pub f: u8,
    pub tail: InventoryTail,
}

impl Inventory313Lookup {
    pub fn decode(tail: &InventoryTail) -> io::Result<Self> {
        let mut input = Reader::new(&tail.0)?;
        Ok(Self {
            d: input.u32()?,
            e: input.u16()?,
            f: input.u8()?,
            tail: input.tail(),
        })
    }

    pub fn encode(&self) -> io::Result<InventoryTail> {
        let mut output = Writer::default();
        output.fields(&[
            (self.d.into(), Width::U32),
            (self.e.into(), Width::U16),
            (self.f.into(), Width::U8),
        ])?;
        output.with_tail(&self.tail).map(InventoryTail)
    }
}

/// S→C 314 (`sub_D77BD0`): known fields followed by any unproven tail.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Inventory314 {
    pub a: u8,
    pub b: u32,
    pub c: u32,
    pub d: u32,
    pub tail: InventoryTail,
}

impl Inventory314 {
    pub const OPCODE: u32 = 314;

    pub fn decode(body: &[u8]) -> io::Result<Self> {
        let mut input = Reader::new(body)?;
        Ok(Self {
            a: input.u8()?,
            b: input.u32()?,
            c: input.u32()?,
            d: input.u32()?,
            tail: input.tail(),
        })
    }

    pub fn encode(&self) -> io::Result<Vec<u8>> {
        let mut output = Writer::default();
        output.fields(&[
            (self.a.into(), Width::U8),
            (self.b.into(), Width::U32),
            (self.c.into(), Width::U32),
            (self.d.into(), Width::U32),
        ])?;
        output.with_tail(&self.tail)
    }
}

/// S→C 315 (`sub_D77D60`): pre-lookup fields. Preserve all 64 bits of `f`
/// without deciding its signedness or assigning it timestamp semantics.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Inventory315 {
    pub a: u8,
    pub b: u8,
    pub c: u32,
    pub d: u32,
    pub e: u32,
    pub f: u64,
    pub tail: InventoryTail,
}

impl Inventory315 {
    pub const OPCODE: u32 = 315;

    pub fn decode(body: &[u8]) -> io::Result<Self> {
        let mut input = Reader::new(body)?;
        Ok(Self {
            a: input.u8()?,
            b: input.u8()?,
            c: input.u32()?,
            d: input.u32()?,
            e: input.u32()?,
            f: input.u64()?,
            tail: input.tail(),
        })
    }

    pub fn encode(&self) -> io::Result<Vec<u8>> {
        let mut output = Writer::default();
        output.fields(&[
            (self.a.into(), Width::U8),
            (self.b.into(), Width::U8),
            (self.c.into(), Width::U32),
            (self.d.into(), Width::U32),
            (self.e.into(), Width::U32),
            (self.f, Width::U64),
        ])?;
        output.with_tail(&self.tail)
    }
}

/// S→C 316 (`sub_D77EF0`): unconditional prefix. The tail may start with
/// `Inventory316Guarded` when the client's feature, lookup and item guards pass.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Inventory316 {
    pub a: u8,
    pub b: u32,
    pub c: u32,
    pub tail: InventoryTail,
}

impl Inventory316 {
    pub const OPCODE: u32 = 316;

    pub fn decode(body: &[u8]) -> io::Result<Self> {
        let mut input = Reader::new(body)?;
        Ok(Self {
            a: input.u8()?,
            b: input.u32()?,
            c: input.u32()?,
            tail: input.tail(),
        })
    }

    pub fn encode(&self) -> io::Result<Vec<u8>> {
        let mut output = Writer::default();
        output.fields(&[
            (self.a.into(), Width::U8),
            (self.b.into(), Width::U32),
            (self.c.into(), Width::U32),
        ])?;
        output.with_tail(&self.tail)
    }
}

/// The confirmed extra read on the 316 guarded branch. Decode this only when
/// the caller knows that branch was taken; length alone cannot select it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Inventory316Guarded {
    pub d: u64,
    pub tail: InventoryTail,
}

impl Inventory316Guarded {
    pub fn decode(tail: &InventoryTail) -> io::Result<Self> {
        let mut input = Reader::new(&tail.0)?;
        Ok(Self {
            d: input.u64()?,
            tail: input.tail(),
        })
    }

    pub fn encode(&self) -> io::Result<InventoryTail> {
        let mut output = Writer::default();
        output.fields(&[(self.d, Width::U64)])?;
        output.with_tail(&self.tail).map(InventoryTail)
    }
}

/// The extra reads when the first byte of 317 is nonzero. The eight words
/// are typed u16 fields from reader `+128`; their calendar semantics are open.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Inventory317Extension {
    pub flag: NonZeroU8,
    pub words: [u16; 8],
    pub e: u32,
}

/// S→C 317 (`sub_D84410`): a zero first byte has no extra reads in the
/// documented prefix. Nonzero values and unproven trailing bytes are retained.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Inventory317 {
    pub b: u8,
    pub c: u32,
    pub d: u32,
    pub extension: Option<Inventory317Extension>,
    pub tail: InventoryTail,
}

impl Inventory317 {
    pub const OPCODE: u32 = 317;

    pub fn decode(body: &[u8]) -> io::Result<Self> {
        let mut input = Reader::new(body)?;
        let flag = NonZeroU8::new(input.u8()?);
        let b = input.u8()?;
        let c = input.u32()?;
        let d = input.u32()?;
        let extension = if let Some(flag) = flag {
            let mut words = [0; 8];
            for word in &mut words {
                *word = input.u16()?;
            }
            Some(Inventory317Extension {
                flag,
                words,
                e: input.u32()?,
            })
        } else {
            None
        };
        Ok(Self {
            b,
            c,
            d,
            extension,
            tail: input.tail(),
        })
    }

    pub fn encode(&self) -> io::Result<Vec<u8>> {
        let mut output = Writer::default();
        let flag = self.extension.as_ref().map_or(0, |extra| extra.flag.get());
        output.fields(&[
            (flag.into(), Width::U8),
            (self.b.into(), Width::U8),
            (self.c.into(), Width::U32),
            (self.d.into(), Width::U32),
        ])?;
        if let Some(extra) = &self.extension {
            for word in extra.words {
                output.fields(&[(word.into(), Width::U16)])?;
            }
            output.fields(&[(extra.e.into(), Width::U32)])?;
        }
        output.with_tail(&self.tail)
    }
}

/// C→S 566 (writer site `0x11AFA5D`): a single u8. Its meaning and the
/// live UI trigger are unverified.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Inventory566 {
    pub a: u8,
}

impl Inventory566 {
    pub const OPCODE: u32 = 566;

    pub fn decode(body: &[u8]) -> io::Result<Self> {
        let mut input = Reader::new(body)?;
        let a = input.u8()?;
        input.finish()?;
        Ok(Self { a })
    }

    pub fn encode(self) -> io::Result<Vec<u8>> {
        put(self.a.into(), Width::U8)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fmt::Debug;

    // Hand-assembled payloads from the static field order and KR primitives.
    // These are conformance vectors, not observed game packets.
    const ITEM: &[u8] = &[
        0x97, 0xc8, 0x50, 0x91, 0xb5, 0x49, 0x87, 0xe8, 0x60, 0xd1, // a, b, c
        0xa5, 0x6c, 0x12, 0x18, 0x3a, 0xc3, 0x2e, // five native bytes
        0x00, 0x01, 0x7f, 0x80, 0xfe, 0xff, 0x55, 0xaa, // opaque eight bytes
        0x68, 0x16, 0xe9, 0x0d, 0xbf, // d, e
    ];
    const LOOKUP_PREFIX: &[u8] = &[0x5b, 0x97, 0xc8, 0x50, 0x91, 0x87, 0xe8, 0x60, 0xd1];
    const U64_VALUE: u64 = 0xfedc_ba98_7654_3210;
    const U64_BYTES: &[u8] = &[0xfe, 0xdc, 0xba, 0x98, 0x07, 0xe5, 0xb7, 0x99];

    fn item(text: &str) -> ItemRecord11CE930 {
        ItemRecord11CE930 {
            a: 0x0102_0304,
            b: 0x1020,
            c: 0x1122_3344,
            text: text.into(),
            raw8: [0x00, 0x01, 0x7f, 0x80, 0xfe, 0xff, 0x55, 0xaa],
            d: 0xfedc_ba98,
            e: 0xe5,
        }
    }

    fn check<T: Debug + Eq>(
        packet: T,
        encode: impl FnOnce(&T) -> io::Result<Vec<u8>>,
        decode: fn(&[u8]) -> io::Result<T>,
        expected: &[u8],
        required: usize,
    ) {
        let encoded = encode(&packet).unwrap();
        assert_eq!(encoded, expected);
        assert_eq!(decode(&encoded).unwrap(), packet);
        for len in 0..required {
            assert!(decode(&expected[..len]).is_err(), "accepted {len} bytes");
        }
    }

    #[test]
    fn item_record_order_raw_bytes_and_text_boundaries() {
        check(
            item("Việt"),
            ItemRecord11CE930::encode,
            ItemRecord11CE930::decode,
            ITEM,
            ITEM.len(),
        );
        assert!(ItemRecord11CE930::decode(&[ITEM, &[0]].concat()).is_err());

        // Empty and multibyte text use the byte length, without player-name rules.
        for text in ["", " 한 글 ", "Việt"] {
            let record = item(text);
            let body = record.encode().unwrap();
            assert_eq!(
                body.len() - ItemRecord11CE930::MIN_LEN,
                text949x::encode(text).unwrap().len()
            );
            assert_eq!(ItemRecord11CE930::decode(&body).unwrap(), record);
        }
        assert!(item("🙂").encode().is_err());
        let mut overrun = ITEM.to_vec();
        overrun[10..12].copy_from_slice(&[0x5a, 0x96]); // length u16::MAX
        assert!(ItemRecord11CE930::decode(&overrun).is_err());
        let mut invalid_text = ITEM[..10].to_vec();
        invalid_text.extend([0xa5, 0x68, 0xa5]); // length 1, native byte 0xff
        invalid_text.extend(&ITEM[17..]);
        assert!(ItemRecord11CE930::decode(&invalid_text).is_err());

        let max = item(&"A".repeat(MAX_BODY - ItemRecord11CE930::MIN_LEN));
        let body = max.encode().unwrap();
        assert_eq!(body.len(), MAX_BODY);
        assert_eq!(ItemRecord11CE930::decode(&body).unwrap(), max);
        assert!(item(&(max.text + "A")).encode().is_err());
    }

    #[test]
    fn group_packets_share_the_envelope_and_keep_each_selector() {
        let groups = vec![
            InventoryItemGroup {
                selector: 0xfe,
                items: vec![item("Việt"), item("Việt")],
            },
            InventoryItemGroup {
                selector: 5,
                items: vec![],
            },
        ];
        let expected = [
            &[0x58, 0xa4, 0x96, 0xca, 0x53, 0x97][..], // count 2, selector 0xfe, two items
            ITEM,
            ITEM,
            &[0x5f, 0x96, 0xca, 0x53, 0x95], // selector 5, zero items
        ]
        .concat();
        check(
            Inventory219 {
                groups: groups.clone(),
            },
            Inventory219::encode,
            Inventory219::decode,
            &expected,
            expected.len(),
        );
        check(
            Inventory224 { groups },
            Inventory224::encode,
            Inventory224::decode,
            &expected,
            expected.len(),
        );
        assert!(Inventory219::decode(&[0x5a]).unwrap().groups.is_empty());
        assert_eq!(Inventory224 { groups: vec![] }.encode().unwrap(), [0x5a]);

        let mut huge_count = expected.clone();
        huge_count[2..6].copy_from_slice(&[0x69, 0x35, 0xac, 0x6a]); // u32::MAX
        let trailing = [&expected[..], &[0]].concat();
        for bad in [huge_count, trailing] {
            assert!(Inventory219::decode(&bad).is_err());
            assert!(Inventory224::decode(&bad).is_err());
        }

        let empty = InventoryItemGroup {
            selector: 0,
            items: vec![],
        };
        assert!(
            Inventory219 {
                groups: vec![empty.clone(); 256]
            }
            .encode()
            .is_err()
        );
        let max = Inventory224 {
            groups: vec![empty; 255],
        };
        assert_eq!(Inventory224::decode(&max.encode().unwrap()).unwrap(), max);
    }

    #[test]
    fn packet225_consumes_eight_pairs_per_row_even_for_zero_values() {
        let empty_pair = [0x96, 0xca, 0x53, 0x95, 0xa5, 0x69];
        let mut rows = vec![std::array::from_fn(|_| Inventory225Slot::default()); 2];
        rows[0][0].text = "ệ".into();
        rows[1][7].a = 0x0102_0304;
        let expected = [
            &[0x58][..],                                       // two rows
            &[0x96, 0xca, 0x53, 0x95, 0xa5, 0x6b, 0x21, 0xbb], // zero and "ệ"
            &empty_pair.repeat(14),
            &[0x97, 0xc8, 0x50, 0x91, 0xa5, 0x69], // nonzero and empty
        ]
        .concat();
        check(
            Inventory225 { rows },
            Inventory225::encode,
            Inventory225::decode,
            &expected,
            expected.len(),
        );
        assert!(Inventory225::decode(&[expected.as_slice(), &[0]].concat()).is_err());
        let mut bad = expected;
        bad[0] = 0x59; // two rows present, count claims three
        assert!(Inventory225::decode(&bad).is_err());
        assert!(Inventory225::decode(&[0x5a]).unwrap().rows.is_empty());
        assert!(
            Inventory225 {
                rows: vec![std::array::from_fn(|_| Inventory225Slot::default()); 256]
            }
            .encode()
            .is_err()
        );
    }

    #[test]
    fn lookup_guarded_tails_are_interpreted_only_on_request() {
        let tail = InventoryTail(vec![0xa8, 0xb7]);
        let lookup = Inventory313Lookup {
            d: 0,
            e: 0x1020,
            f: 0xe5,
            tail: tail.clone(),
        };
        let lookup_tail = lookup.encode().unwrap();
        assert_eq!(
            lookup_tail.0,
            [0x96, 0xca, 0x53, 0x95, 0xb5, 0x49, 0xbf, 0xa8, 0xb7]
        );
        assert_eq!(Inventory313Lookup::decode(&lookup_tail).unwrap(), lookup);
        assert!(Inventory313Lookup::decode(&InventoryTail(lookup_tail.0[..6].to_vec())).is_err());
        check(
            Inventory313 {
                a: 1,
                b: 0x0102_0304,
                c: 0x1122_3344,
                tail: lookup_tail,
            },
            Inventory313::encode,
            Inventory313::decode,
            &[
                LOOKUP_PREFIX,
                &[0x96, 0xca, 0x53, 0x95, 0xb5, 0x49, 0xbf, 0xa8, 0xb7],
            ]
            .concat(),
            9,
        );
        let guarded = Inventory316Guarded { d: U64_VALUE, tail };
        let guarded_tail = guarded.encode().unwrap();
        assert_eq!(guarded_tail.0, [U64_BYTES, &[0xa8, 0xb7]].concat());
        assert_eq!(Inventory316Guarded::decode(&guarded_tail).unwrap(), guarded);
        assert!(Inventory316Guarded::decode(&InventoryTail(U64_BYTES[..7].to_vec())).is_err());
        check(
            Inventory316 {
                a: 1,
                b: 0x0102_0304,
                c: 0x1122_3344,
                tail: guarded_tail,
            },
            Inventory316::encode,
            Inventory316::decode,
            &[LOOKUP_PREFIX, U64_BYTES, &[0xa8, 0xb7]].concat(),
            9,
        );
        assert!(
            Inventory313::decode(LOOKUP_PREFIX)
                .unwrap()
                .tail
                .0
                .is_empty()
        );
        assert!(
            Inventory316::decode(LOOKUP_PREFIX)
                .unwrap()
                .tail
                .0
                .is_empty()
        );
        let unrelated = [LOOKUP_PREFIX, &[0x1b]].concat();
        assert_eq!(Inventory313::decode(&unrelated).unwrap().tail.0, [0x1b]);
        assert_eq!(Inventory316::decode(&unrelated).unwrap().tail.0, [0x1b]);
    }

    #[test]
    fn packets314_and315_preserve_unknown_trailing_bytes() {
        let tail = InventoryTail(vec![0xab, 0xcd]);
        check(
            Inventory314 {
                a: 1,
                b: 0x0102_0304,
                c: 0x1122_3344,
                d: 0xfedc_ba98,
                tail: tail.clone(),
            },
            Inventory314::encode,
            Inventory314::decode,
            &[LOOKUP_PREFIX, &[0x68, 0x16, 0xe9, 0x0d, 0xab, 0xcd]].concat(),
            13,
        );
        check(
            Inventory315 {
                a: 2,
                b: 1,
                c: 0x0102_0304,
                d: 0x1122_3344,
                e: 0xfedc_ba98,
                f: U64_VALUE,
                tail,
            },
            Inventory315::encode,
            Inventory315::decode,
            &[
                &[0x58],
                LOOKUP_PREFIX,
                &[0x68, 0x16, 0xe9, 0x0d],
                U64_BYTES,
                &[0xab, 0xcd],
            ]
            .concat(),
            22,
        );
    }

    #[test]
    fn packet317_uses_the_raw_flag_to_select_eight_typed_words_and_a_dword() {
        let tail = InventoryTail(vec![0x6d]);
        let zero = Inventory317 {
            b: 1,
            c: 0x0102_0304,
            d: 0x1122_3344,
            extension: None,
            tail,
        };
        check(
            zero.clone(),
            Inventory317::encode,
            Inventory317::decode,
            &[&[0x5a], LOOKUP_PREFIX, &[0x6d]].concat(),
            10,
        );
        let nonzero = Inventory317 {
            extension: Some(Inventory317Extension {
                flag: NonZeroU8::new(0xff).unwrap(),
                words: [1, 2, 3, 4, 5, 6, 7, 0x1020],
                e: 0xfedc_ba98,
            }),
            ..zero
        };
        let fields = [
            0xa5, 0x68, 0xa5, 0x6b, 0xa5, 0x6a, 0xa5, 0x6d, 0xa5, 0x6c, 0xa5, 0x6f, 0xa5, 0x6e,
            0xb5, 0x49, 0x68, 0x16, 0xe9, 0x0d,
        ];
        check(
            nonzero,
            Inventory317::encode,
            Inventory317::decode,
            &[&[0xa5], LOOKUP_PREFIX, &fields, &[0x6d]].concat(),
            30,
        );
    }

    #[test]
    fn packet566_is_exactly_one_encoded_byte() {
        for (a, wire) in [(0, 0x5a), (1, 0x5b), (0xff, 0xa5)] {
            check(
                Inventory566 { a },
                |value| value.encode(),
                Inventory566::decode,
                &[wire],
                1,
            );
            assert!(Inventory566::decode(&[wire, 0]).is_err());
        }
    }

    #[test]
    fn opaque_tails_respect_the_existing_frame_budget() {
        let mut body = LOOKUP_PREFIX.to_vec();
        body.resize(MAX_BODY, 0x77);
        let mut packet = Inventory313::decode(&body).unwrap();
        assert_eq!(packet.encode().unwrap(), body);
        packet.tail.0.push(0x77);
        body.push(0x77);
        assert!(packet.encode().is_err());
        assert!(Inventory313::decode(&body).is_err());
    }
}
