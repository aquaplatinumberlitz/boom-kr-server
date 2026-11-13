//! Floor-only fixture policy, CP030/031/049/055/063. Not native world authority.
//! List2 stays unavailable until terrain/reveal is ported; no inventory/reward.
#[cfg(test)]
use kr_wire::write_uint;
use kr_wire::{Width, read_uint};
use serde::Serialize;
use std::{
    collections::{BTreeMap, BTreeSet},
    io,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Item {
    pub index: u32,
    pub row: u8,
    pub column: u8,
    pub kind: u16,
}
#[derive(Clone, Debug, Default)]
pub(crate) struct Catalog {
    floor: BTreeMap<u32, Item>,
    hidden: BTreeMap<u32, Item>,
}
impl Catalog {
    pub(crate) fn validate_needle(&self, enabled: bool) -> io::Result<()> {
        let count = self
            .floor
            .values()
            .chain(self.hidden.values())
            .filter(|i| i.kind == 145)
            .count();
        if count > 1 || (count > 0 && !enabled) {
            return Err(io::Error::other(
                "needle floor requires enabled one-per-round policy",
            ));
        }
        Ok(())
    }
    /// Decode only the already measured raw (tag0) serialized582 branch. Catalog
    /// and bytes sent to the clients have one source; no parallel config list.
    pub fn from_floor(body: &[u8]) -> io::Result<Self> {
        let err = || io::Error::other("unsupported measured floor582 catalog");
        if !(16..=4096).contains(&body.len()) {
            return Err(err());
        }
        let size = read_uint(&body[..4], Width::U32).map_err(io::Error::other)? as usize;
        if size != body.len() - 4 {
            return Err(err());
        }
        let stream = crate::lobby::decode_bytes(&body[4..])?;
        if stream.len() < 12
            || stream[0] != 0x18
            || stream[5] != 0
            || u32::from_le_bytes(stream[1..5].try_into().expect("bounded")) as usize
                != stream.len() - 5
        {
            return Err(err());
        }
        let mut at = 6;
        let mut result = Self::default();
        let mut indices = BTreeSet::new();
        for group in 0..3 {
            let count = stream.get(at..at + 2).ok_or_else(err)?;
            let count = u16::from_le_bytes(count.try_into().expect("bounded")) as usize;
            at += 2;
            if count > 64 || (group == 2 && count != 0) {
                return Err(err());
            }
            for _ in 0..count {
                let row = stream.get(at..at + 10).ok_or_else(err)?;
                at += 10;
                let index = u32::from_le_bytes(row[..4].try_into().expect("bounded"));
                let kind = u32::from_le_bytes(row[6..10].try_into().expect("bounded"));
                // This fixed ExVillage01 slice only advertises Bomb/Fire/Roller.
                if row[4] >= 13
                    || row[5] >= 15
                    || !matches!(kind, 0..=2 | 145)
                    || !indices.insert(index)
                    || indices.len() > 64
                {
                    return Err(err());
                }
                let item = Item {
                    index,
                    row: row[4],
                    column: row[5],
                    kind: kind as u16,
                };
                if group == 0 {
                    result.floor.insert(index, item);
                } else {
                    result.hidden.insert(index, item);
                }
            }
        }
        if at != stream.len() {
            return Err(err());
        }
        Ok(result)
    }
}
#[derive(Clone, Debug)]
pub(crate) struct Request {
    context: u8,
    slot: u8,
    item: Item,
    // A latched client logical counter, not a server timestamp or serial.
    client_tick: u32,
    key: u32,
}
impl Request {
    pub fn parse(body: &[u8]) -> io::Result<Self> {
        if body.len() != 19 {
            return Err(io::Error::other("pickup124 size"));
        }
        let widths = [
            Width::U8,
            Width::U8,
            Width::U8,
            Width::U8,
            Width::U32,
            Width::U16,
            Width::U32,
            Width::U32,
            Width::U8,
        ];
        let lengths = [1, 1, 1, 1, 4, 2, 4, 4, 1];
        let mut at = 0;
        let mut v = Vec::new();
        for (w, n) in widths.into_iter().zip(lengths) {
            v.push(read_uint(&body[at..at + n], w).map_err(io::Error::other)?);
            at += n;
        }
        if v[8] != 0 {
            return Err(io::Error::other("unsupported pickup124 flag"));
        }
        Ok(Self {
            context: v[0] as u8,
            slot: v[1] as u8,
            item: Item {
                index: v[4] as u32,
                row: v[2] as u8,
                column: v[3] as u8,
                kind: v[5] as u16,
            },
            client_tick: v[6] as u32,
            key: v[7] as u32,
        })
    }
    pub fn kind(&self) -> u16 {
        self.item.kind
    }
    pub fn confirmation(&self) -> io::Result<Vec<u8>> {
        kr_wire::kr00967::pickup165(
            self.context,
            self.slot,
            self.item.row,
            self.item.column,
            self.item.index,
            self.item.kind,
        )
    }
    pub fn decision(&self, reason: &'static str) -> Decision {
        Decision {
            accepted: reason == "accepted_floor",
            reason,
            index: self.item.index,
            slot: self.slot,
            client_tick: self.client_tick,
        }
    }
}
#[derive(Debug, Serialize)]
pub(crate) struct Decision {
    pub accepted: bool,
    reason: &'static str,
    index: u32,
    slot: u8,
    client_tick: u32,
}
#[derive(Clone, Debug, Default)]
pub(crate) struct Round {
    generation: Option<u64>,
    key: u32,
    catalog: Catalog,
    consumed: BTreeSet<u32>,
    revealed_cells: BTreeSet<(u8, u8)>,
}
impl Round {
    pub fn begin(generation: u64, catalog: &Catalog) -> io::Result<Self> {
        Ok(Self {
            generation: Some(generation),
            key: crate::room::wire_key(generation)?,
            catalog: catalog.clone(),
            consumed: BTreeSet::new(),
            revealed_cells: BTreeSet::new(),
        })
    }
    pub fn finish(&mut self) {
        self.generation = None;
        self.catalog = Catalog::default();
        self.consumed.clear();
        self.revealed_cells.clear();
    }
    /// Mutate only the candidate model. Caller reserves ALL recipient queues
    /// before publishing this consumed set, under the same room owner lock.
    pub fn reveal(&mut self, generation: u64, row: u8, column: u8) -> bool {
        if self.generation != Some(generation) {
            return false;
        }
        let ids: Vec<_> = self
            .catalog
            .hidden
            .values()
            .filter(|item| (item.row, item.column) == (row, column))
            .map(|item| item.index)
            .collect();
        if ids.is_empty() {
            return false;
        }
        self.revealed_cells.insert((row, column));
        for id in ids {
            let item = self.catalog.hidden.remove(&id).expect("matchedhidden");
            self.catalog.floor.insert(id, item);
        }
        true
    }
    pub fn hidden_cell(&self, row: u8, column: u8) -> bool {
        self.revealed_cells.contains(&(row, column))
            || self
                .catalog
                .hidden
                .values()
                .any(|item| (item.row, item.column) == (row, column))
    }
    pub fn claim(&mut self, generation: u64, bound_slot: u8, request: &Request) -> Decision {
        let reason = if self.generation != Some(generation) {
            "inactive_round"
        } else if request.key != self.key {
            "round_key_mismatch"
        } else if request.slot != bound_slot {
            "slot_mismatch"
        } else if request.context != 0 {
            "context_mismatch"
        } else if self.catalog.hidden.contains_key(&request.item.index) {
            "hidden_unrevealed"
        } else if self.catalog.floor.get(&request.item.index) != Some(&request.item) {
            "catalog_mismatch"
        } else if self.consumed.contains(&request.item.index) {
            "already_consumed"
        } else {
            self.consumed.insert(request.item.index);
            "accepted_floor"
        };
        request.decision(reason)
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    pub(crate) fn catalog() -> Catalog {
        Catalog {
            floor: BTreeMap::from([(
                17,
                Item {
                    index: 17,
                    row: 4,
                    column: 3,
                    kind: 2,
                },
            )]),
            hidden: BTreeMap::from([(
                27,
                Item {
                    index: 27,
                    row: 2,
                    column: 3,
                    kind: 0,
                },
            )]),
        }
    }
    fn request() -> Request {
        Request {
            context: 0,
            slot: 0,
            item: Item {
                index: 17,
                row: 4,
                column: 3,
                kind: 2,
            },
            client_tick: 23,
            key: 12346,
        }
    }
    #[test]
    fn reveal_moves_only_current_matching_hidden_cell_and_reset_clears_it() {
        let mut round = Round::begin(1, &catalog()).unwrap();
        let mut hidden = request();
        hidden.item = Item {
            index: 27,
            row: 2,
            column: 3,
            kind: 0,
        };
        assert_eq!(round.claim(1, 0, &hidden).reason, "hidden_unrevealed");
        assert!(!round.reveal(2, 2, 3));
        assert!(!round.reveal(1, 3, 3));
        assert!(round.reveal(1, 2, 3));
        assert!(round.hidden_cell(2, 3));
        assert!(!round.reveal(1, 2, 3));
        assert!(round.claim(1, 0, &hidden).accepted);
        assert!(!round.claim(1, 0, &hidden).accepted);
        let mut reset = Round::begin(2, &catalog()).unwrap();
        hidden.key += 1;
        assert_eq!(reset.claim(2, 0, &hidden).reason, "hidden_unrevealed");
        reset.finish();
        assert!(!reset.reveal(2, 2, 3));
    }
    #[test]
    fn wrong_claims_never_reserve_index_and_only_one_bound_winner() {
        let mut round = Round::begin(1, &catalog()).unwrap();
        let good = request();
        let mut bad = good.clone();
        bad.key -= 1;
        assert_eq!(round.claim(1, 0, &bad).reason, "round_key_mismatch");
        assert_eq!(round.claim(1, 1, &good).reason, "slot_mismatch");
        let mut bad = good.clone();
        bad.context = 1;
        assert_eq!(round.claim(1, 0, &bad).reason, "context_mismatch");
        for bad_item in [
            Item {
                index: 17,
                row: 3,
                column: 3,
                kind: 2,
            },
            Item {
                index: 17,
                row: 4,
                column: 4,
                kind: 2,
            },
            Item {
                index: 17,
                row: 4,
                column: 3,
                kind: 1,
            },
            Item {
                index: 999,
                row: 4,
                column: 3,
                kind: 2,
            },
            Item {
                index: 27,
                row: 2,
                column: 3,
                kind: 0,
            },
        ] {
            let mut bad = good.clone();
            bad.item = bad_item;
            assert!(!round.claim(1, 0, &bad).accepted);
        }
        assert!(round.claim(1, 0, &good).accepted);
        let mut other = good.clone();
        other.slot = 1;
        assert_eq!(round.claim(1, 1, &other).reason, "already_consumed");
        assert_eq!(round.claim(1, 0, &good).reason, "already_consumed");
    }
    #[test]
    fn finish_and_new_generation_do_not_reuse_old_claim_or_key() {
        let mut round = Round::begin(1, &catalog()).unwrap();
        let good = request();
        assert!(round.claim(1, 0, &good).accepted);
        round.finish();
        assert_eq!(round.claim(1, 0, &good).reason, "inactive_round");
        let mut round = Round::begin(2, &catalog()).unwrap();
        assert_eq!(round.claim(1, 0, &good).reason, "inactive_round");
        assert_eq!(round.claim(2, 0, &good).reason, "round_key_mismatch");
        let mut current = good.clone();
        current.key += 1;
        current.client_tick = 0;
        assert!(round.claim(2, 0, &current).accepted);
    }
    #[test]
    fn raw_floor_catalog_rejects_bad_envelope_duplicates_hidden_and_trailing_bytes() {
        fn packet(groups: [Vec<(u32, u8, u8, u32)>; 3]) -> Vec<u8> {
            let mut inner = Vec::new();
            for g in groups {
                inner.extend((g.len() as u16).to_le_bytes());
                for (i, r, c, k) in g {
                    inner.extend(i.to_le_bytes());
                    inner.extend([r, c]);
                    inner.extend(k.to_le_bytes());
                }
            }
            let mut stream = vec![0x18];
            stream.extend(((inner.len() + 1) as u32).to_le_bytes());
            stream.push(0);
            stream.extend(inner);
            let mut b = write_uint(stream.len() as u64, Width::U32).unwrap();
            let mut at = 0;
            while at < stream.len() {
                let (n, w) = if stream.len() - at >= 4 {
                    (4, Width::U32)
                } else if stream.len() - at >= 2 {
                    (2, Width::U16)
                } else {
                    (1, Width::U8)
                };
                let mut raw = [0u8; 8];
                raw[..n].copy_from_slice(&stream[at..at + n]);
                let v = u64::from_le_bytes(raw);
                b.extend(write_uint(v, w).unwrap());
                at += n;
            }
            b
        }
        let p = packet([vec![(17, 4, 3, 2)], vec![(27, 2, 3, 0)], vec![]]);
        let cat = Catalog::from_floor(&p).unwrap();
        assert_eq!(cat.floor.len(), 1);
        assert_eq!(cat.hidden.len(), 1);
        assert!(!cat.floor.contains_key(&27));
        for p in [
            packet([vec![(17, 4, 3, 2), (17, 4, 4, 2)], vec![], vec![]]),
            packet([vec![(17, 13, 3, 2)], vec![], vec![]]),
            packet([vec![(17, 4, 3, 999)], vec![], vec![]]),
            packet([vec![], vec![], vec![(17, 4, 3, 2)]]),
        ] {
            assert!(Catalog::from_floor(&p).is_err());
        }
        let mut truncated = p.clone();
        truncated.pop();
        assert!(Catalog::from_floor(&truncated).is_err());
        let mut trailing = p;
        trailing.push(0);
        assert!(Catalog::from_floor(&trailing).is_err());
        assert!(Request::parse(&[0; 18]).is_err());
        assert!(Request::parse(&[0; 19]).is_err());
    }
}
