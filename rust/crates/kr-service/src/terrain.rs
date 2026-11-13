//! Candidate bounded terrain policy; no native oracle or general KR physics.
//! Port target: pinned15x13 ExVillage01, CP171/172/174. Config is trusted design.
use serde::Deserialize;
use std::{
    collections::{BTreeMap, BTreeSet, VecDeque},
    io,
};
#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    width: u8,
    height: u8,
    kinds: Vec<u8>,
    map_sha256: String,
    seed_sha256: String,
    fixtures: BTreeMap<String, String>,
    maximum_power: u8,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Bomb {
    pub id: u16,
    pub owner: u8,
    pub row: u8,
    pub column: u8,
    pub power: u8,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Blast {
    pub processed: Vec<u16>,
    pub cleared: BTreeSet<(u8, u8)>,
}
#[derive(Clone, Debug)]
pub(crate) struct Terrain {
    seed: Vec<u8>,
    kinds: Vec<u8>,
    width: u8,
    height: u8,
    max_power: u8,
    owner_slots: u8,
    pending: BTreeMap<u16, Bomb>,
    seen: BTreeSet<u16>,
    /// Cell clearance becomes eligible only after the originating158 has
    /// completed actualwrites to ALL original recipients in this round.
    cleared: BTreeMap<(u8, u8), (u64, u16)>,
    generation: u64,
}
fn sha(data: &[u8]) -> String {
    crate::sha256_hex(data)
}
impl Config {
    pub(crate) fn validate(&self, map_frame: &[u8], start_frame: &[u8]) -> io::Result<Terrain> {
        self.validate_limits(map_frame, start_frame, 2, 2)
    }
    pub(crate) fn validate_measured(
        &self,
        map_frame: &[u8],
        start_frame: &[u8],
        maximum_power: u8,
    ) -> io::Result<Terrain> {
        self.validate_limits(map_frame, start_frame, maximum_power, 8)
    }
    fn validate_limits(
        &self,
        map_frame: &[u8],
        start_frame: &[u8],
        maximum_power: u8,
        owner_slots: u8,
    ) -> io::Result<Terrain> {
        let error = || io::Error::other("unsupported pinned terrain profile");
        if self.width != 15
            || self.height != 13
            || self.kinds.len() != 195
            || self.kinds.iter().any(|k| *k > 2)
            || !(1..=9).contains(&maximum_power)
            || !(1..=maximum_power).contains(&self.maximum_power)
            || sha(&self.kinds) != self.seed_sha256
            || self.map_sha256.len() != 64
            || !self
                .map_sha256
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
            || self.fixtures.len() != 2
            || self.fixtures.get("map") != Some(&sha(map_frame))
            || self.fixtures.get("start") != Some(&sha(start_frame))
        {
            return Err(error());
        }
        Ok(Terrain {
            seed: self.kinds.clone(),
            kinds: self.kinds.clone(),
            width: self.width,
            height: self.height,
            max_power: self.maximum_power,
            owner_slots,
            pending: BTreeMap::new(),
            seen: BTreeSet::new(),
            cleared: BTreeMap::new(),
            generation: 0,
        })
    }
}
impl Terrain {
    pub fn begin(&mut self, generation: u64) {
        self.generation = generation;
        self.kinds = self.seed.clone();
        self.pending.clear();
        self.seen.clear();
        self.cleared.clear();
    }
    pub fn finish(&mut self) {
        self.pending.clear();
        self.cleared.clear();
        self.generation = 0;
    }
    pub fn place(&mut self, bomb: Bomb) -> io::Result<()> {
        if self.generation == 0
            || bomb.id == 0
            || self.seen.contains(&bomb.id)
            || bomb.owner >= self.owner_slots
            || bomb.row >= self.height
            || bomb.column >= self.width
            || !(1..=self.max_power).contains(&bomb.power)
            || self.kinds
                [usize::from(bomb.row) * usize::from(self.width) + usize::from(bomb.column)]
                != 0
            || self
                .pending
                .values()
                .any(|b| (b.row, b.column) == (bomb.row, bomb.column))
        {
            return Err(io::Error::other("unsupported bounded terrain placement"));
        }
        self.seen.insert(bomb.id);
        self.pending.insert(bomb.id, bomb);
        Ok(())
    }
    /// Must run under the room owner ONLY at full-roster158write completion.
    /// Snapshot immutable for cascade. An earlier chain-processed secondary's
    /// own later158 returns an emptydelta; no duplicate reveal eligibility.
    pub fn blast_committed(&mut self, generation: u64, root: u16) -> Blast {
        if self.generation != generation || !self.pending.contains_key(&root) {
            return Blast {
                processed: vec![],
                cleared: BTreeSet::new(),
            };
        }
        let occupied: BTreeMap<_, _> = self
            .pending
            .values()
            .map(|b| ((b.row, b.column), b.id))
            .collect();
        let mut queue = VecDeque::from([root]);
        let mut queued = BTreeSet::from([root]);
        let mut processed = Vec::new();
        let mut cleared = BTreeSet::new();
        while let Some(id) = queue.pop_front() {
            let bomb = &self.pending[&id];
            processed.push(id);
            for (dr, dc) in [(0, -1), (-1, 0), (0, 1), (1, 0)] {
                for distance in 1..=i16::from(bomb.power) {
                    let row = i16::from(bomb.row) + dr * distance;
                    let column = i16::from(bomb.column) + dc * distance;
                    if row < 0
                        || column < 0
                        || row >= i16::from(self.height)
                        || column >= i16::from(self.width)
                    {
                        break;
                    }
                    let cell = (row as u8, column as u8);
                    let kind = self.kinds[row as usize * usize::from(self.width) + column as usize];
                    if kind != 0 {
                        if kind == 2 {
                            cleared.insert(cell);
                        }
                        break;
                    }
                    if let Some(other) = occupied.get(&cell) {
                        if queued.insert(*other) {
                            queue.push_back(*other);
                        }
                        break;
                    }
                }
            }
        }
        for id in &processed {
            self.pending.remove(id);
        }
        for &(row, column) in &cleared {
            self.kinds[usize::from(row) * usize::from(self.width) + usize::from(column)] = 0;
            self.cleared
                .entry((row, column))
                .or_insert((generation, root));
        }
        Blast { processed, cleared }
    }
    pub fn clearance(&self, generation: u64, row: u8, column: u8) -> Option<u16> {
        self.cleared
            .get(&(row, column))
            .and_then(|(g, id)| (*g == generation && self.generation == generation).then_some(*id))
    }
}
#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    fn config() -> Config {
        let mut kinds = vec![0; 195];
        kinds[3 * 15 + 5] = 2;
        kinds[4 * 15 + 4] = 2;
        kinds[2 * 15 + 3] = 1;
        Config {
            width: 15,
            height: 13,
            seed_sha256: sha(&kinds),
            kinds,
            map_sha256: "0".repeat(64),
            fixtures: BTreeMap::from([
                ("map".into(), sha(b"map")),
                ("start".into(), sha(b"start")),
            ]),
            maximum_power: 2,
        }
    }
    pub(crate) fn map() -> Terrain {
        config().validate(b"map", b"start").unwrap()
    }
    pub(crate) fn basic_items_map() -> Terrain {
        let mut cfg = config();
        cfg.kinds = vec![0; 195];
        for (row, col, kind) in [(2, 3, 2), (3, 2, 2), (3, 8, 1), (3, 9, 2)] {
            cfg.kinds[row * 15 + col] = kind;
        }
        cfg.seed_sha256 = sha(&cfg.kinds);
        cfg.maximum_power = 9;
        cfg.validate_measured(b"map", b"start", 9).unwrap()
    }
    #[test]
    fn measured_owner_and_power_bounds_stop_at_solid_and_preserve_legacy_profile() {
        let mut legacy = map();
        legacy.begin(1);
        assert!(
            legacy
                .place(Bomb {
                    id: 1,
                    owner: 2,
                    row: 3,
                    column: 3,
                    power: 1
                })
                .is_err()
        );
        let mut t = basic_items_map();
        t.begin(7);
        for (owner, power) in [(8, 1), (7, 10), (7, 0)] {
            assert!(
                t.place(Bomb {
                    id: 1,
                    owner,
                    row: 3,
                    column: 3,
                    power
                })
                .is_err()
            );
        }
        t.place(Bomb {
            id: 1,
            owner: 7,
            row: 3,
            column: 3,
            power: 9,
        })
        .unwrap();
        assert_eq!(t.blast_committed(6, 1).cleared, BTreeSet::new());
        assert_eq!(
            t.blast_committed(7, 1).cleared,
            BTreeSet::from([(2, 3), (3, 2)])
        );
        assert!(t.clearance(7, 3, 9).is_none());
    }
    #[test]
    fn no_clearance_before_commit_foreign_generation_reset_and_geometry_rejections() {
        let mut t = map();
        t.begin(1);
        for b in [
            Bomb {
                id: 1,
                owner: 0,
                row: 2,
                column: 3,
                power: 1,
            },
            Bomb {
                id: 1,
                owner: 0,
                row: 13,
                column: 3,
                power: 1,
            },
            Bomb {
                id: 1,
                owner: 0,
                row: 3,
                column: 3,
                power: 0,
            },
            Bomb {
                id: 1,
                owner: 2,
                row: 3,
                column: 3,
                power: 1,
            },
        ] {
            assert!(t.place(b).is_err());
        }
        t.place(Bomb {
            id: 1,
            owner: 0,
            row: 3,
            column: 4,
            power: 1,
        })
        .unwrap();
        assert!(
            t.place(Bomb {
                id: 2,
                owner: 1,
                row: 3,
                column: 4,
                power: 1
            })
            .is_err()
        );
        assert_eq!(t.clearance(1, 3, 5), None);
        assert!(t.blast_committed(2, 1).processed.is_empty());
        assert_eq!(t.clearance(1, 3, 5), None);
        let clone = t.clone();
        assert!(t.blast_committed(1, 1).cleared.contains(&(3, 5)));
        assert_eq!(clone.clearance(1, 3, 5), None);
        assert!(t.clearance(1, 3, 5).is_some());
        t.begin(2);
        assert_eq!(t.clearance(2, 3, 5), None);
        assert!(
            t.place(Bomb {
                id: 2,
                owner: 0,
                row: 3,
                column: 5,
                power: 1
            })
            .is_err()
        );
    }
    #[test]
    fn pinned_config_and_blocker_chain_snapshot_are_bounded() {
        let mut t = map();
        t.begin(1);
        t.place(Bomb {
            id: 1,
            owner: 0,
            row: 3,
            column: 3,
            power: 2,
        })
        .unwrap();
        t.place(Bomb {
            id: 2,
            owner: 1,
            row: 3,
            column: 4,
            power: 2,
        })
        .unwrap();
        let blast = t.blast_committed(1, 1);
        assert_eq!(blast.processed, vec![1, 2]);
        assert_eq!(blast.cleared, BTreeSet::from([(3, 5), (4, 4)]));
        assert_eq!(t.clearance(1, 3, 5), Some(1));
        assert!(t.blast_committed(1, 2).cleared.is_empty());
        assert!(
            t.place(Bomb {
                id: 1,
                owner: 0,
                row: 3,
                column: 3,
                power: 1
            })
            .is_err()
        );
        t.begin(2);
        assert_eq!(t.clearance(2, 3, 5), None);
        assert!(t.blast_committed(1, 1).cleared.is_empty());
        t.finish();
        assert_eq!(t.clearance(2, 4, 4), None);
    }
}
