//! Candidate bounded terrain policy; no native oracle or general KR physics.
//! Supports the pinned 15×13 profiles and the 19×17 BigBattle grid.
//! The separate F3 opt-in checks the grid sizes pinned by the KR 00967 pack census.
use serde::Deserialize;
use std::{
    collections::{BTreeMap, BTreeSet, VecDeque},
    io,
};
#[derive(Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    width: u8,
    height: u8,
    kinds: Vec<u8>,
    /// Optional per-cell hit counts; zero on non-box cells, 1..=254 on boxes.
    #[serde(default)]
    box_hits: Vec<u8>,
    /// Optional per-cell exploding-box flag (+33 == 1 in KR Tile*_Box): 0 or 1 on boxes.
    #[serde(default)]
    exploding_boxes: Vec<u8>,
    /// Optional map-owned stop-before-cell layer, independent of box/solid flags.
    #[serde(default)]
    ray_barriers: Vec<u8>,
    /// Optional per-cell open ray sides in KR BA0E30/17929B0 order:
    /// W (bit 0), N (bit 1), E (bit 2), S (bit 3). `0b1010` opens N/S and
    /// blocks W/E (Tent cells).
    #[serde(default)]
    ray_ports: Vec<u8>,
    /// Map-owned placement policy; omitted on existing profiles.
    #[serde(default)]
    disable_bombs: bool,
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
    seed_hits: Vec<u8>,
    remaining_hits: Vec<u8>,
    exploding_boxes: Vec<u8>,
    ray_barriers: Vec<u8>,
    ray_ports: Vec<u8>,
    width: u8,
    height: u8,
    disable_bombs: bool,
    max_power: u8,
    owner_slots: u8,
    pending: BTreeMap<u16, Bomb>,
    seen: BTreeSet<u16>,
    /// Cell clearance becomes eligible only after the originating158 has
    /// completed actualwrites to ALL original recipients in this round.
    cleared: BTreeMap<(u8, u8), (u64, u16)>,
    generation: u64,
}
const ALL_RAY_PORTS: u8 = 0b1111;
pub(crate) const STANDARD_SIZE: (u8, u8) = (15, 13);
pub(crate) const BIG_BATTLE_SIZE: (u8, u8) = (19, 17);

/// Exact (width, height) pairs from the KR 00967 F3 pack census.
/// See `docs/evidence/content/f3-pve-large-grid-re.json` (kr_static/confirmed).
pub(crate) const F3_LARGE_GRID_SIZES: [(u8, u8); 35] = [
    (15, 51),
    (20, 12),
    (20, 13),
    (20, 15),
    (20, 16),
    (20, 18),
    (20, 19),
    (20, 20),
    (20, 22),
    (20, 25),
    (21, 12),
    (21, 16),
    (21, 17),
    (21, 19),
    (21, 20),
    (21, 21),
    (22, 22),
    (23, 17),
    (23, 23),
    (24, 24),
    (25, 12),
    (25, 13),
    (25, 15),
    (25, 17),
    (25, 19),
    (25, 20),
    (25, 22),
    (25, 25),
    (29, 29),
    (33, 29),
    (37, 29),
    (40, 10),
    (40, 12),
    (67, 10),
    (100, 10),
];

pub(crate) fn is_pinned_f3_large_grid(width: u8, height: u8) -> bool {
    F3_LARGE_GRID_SIZES.contains(&(width, height))
}

fn sha(data: &[u8]) -> String {
    crate::sha256_hex(data)
}
impl Config {
    pub(crate) fn validate_for_map(
        &self,
        map_frame: &[u8],
        start_frame: &[u8],
        payload_sha256: &str,
    ) -> io::Result<Terrain> {
        if self.map_sha256 != payload_sha256 {
            return Err(io::Error::other("terrain belongs to another pack map"));
        }
        self.validate_measured(map_frame, start_frame, 9)
    }
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
    /// Opt in to F3 large-grid dimensions for a pinned map and its fixtures.
    /// The existing server terrain limits still apply; the census pins dimensions only.
    #[allow(dead_code, reason = "F3 large grids are not wired into room loading")]
    pub(crate) fn validate_large_f3_grid(
        &self,
        map_frame: &[u8],
        start_frame: &[u8],
        payload_sha256: &str,
    ) -> io::Result<Terrain> {
        if self.map_sha256 != payload_sha256 {
            return Err(io::Error::other("terrain belongs to another pack map"));
        }
        if !is_pinned_f3_large_grid(self.width, self.height) {
            return Err(io::Error::other("unsupported pinned F3 large grid"));
        }
        self.validate_profile(map_frame, start_frame, 9, 8)
    }
    fn validate_limits(
        &self,
        map_frame: &[u8],
        start_frame: &[u8],
        maximum_power: u8,
        owner_slots: u8,
    ) -> io::Result<Terrain> {
        if !matches!((self.width, self.height), STANDARD_SIZE | BIG_BATTLE_SIZE) {
            return Err(io::Error::other("unsupported pinned terrain profile"));
        }
        self.validate_profile(map_frame, start_frame, maximum_power, owner_slots)
    }
    fn validate_profile(
        &self,
        map_frame: &[u8],
        start_frame: &[u8],
        maximum_power: u8,
        owner_slots: u8,
    ) -> io::Result<Terrain> {
        let error = || io::Error::other("unsupported pinned terrain profile");
        if self.kinds.len() != usize::from(self.width) * usize::from(self.height)
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
        let ray_barriers = if self.ray_barriers.is_empty() {
            vec![0; self.kinds.len()]
        } else if self.ray_barriers.len() == self.kinds.len()
            && self.ray_barriers.iter().all(|flag| *flag <= 1)
        {
            self.ray_barriers.clone()
        } else {
            return Err(error());
        };
        let ray_ports = if self.ray_ports.is_empty() {
            vec![ALL_RAY_PORTS; self.kinds.len()]
        } else if self.ray_ports.len() == self.kinds.len()
            && self.ray_ports.iter().all(|mask| *mask <= ALL_RAY_PORTS)
        {
            self.ray_ports.clone()
        } else {
            return Err(error());
        };
        let hits = if self.box_hits.is_empty() {
            self.kinds.iter().map(|kind| u8::from(*kind == 2)).collect()
        } else {
            if self.box_hits.len() != self.kinds.len()
                || self.kinds.iter().zip(&self.box_hits).any(|(kind, hits)| {
                    if *kind == 2 {
                        !(1..=254).contains(hits)
                    } else {
                        *hits != 0
                    }
                })
            {
                return Err(error());
            }
            self.box_hits.clone()
        };
        let exploding_boxes = if self.exploding_boxes.is_empty() {
            vec![0; self.kinds.len()]
        } else if self.exploding_boxes.len() == self.kinds.len()
            && self
                .kinds
                .iter()
                .zip(&self.exploding_boxes)
                .all(|(kind, flag)| if *kind == 2 { *flag <= 1 } else { *flag == 0 })
        {
            self.exploding_boxes.clone()
        } else {
            return Err(error());
        };
        Ok(Terrain {
            seed: self.kinds.clone(),
            kinds: self.kinds.clone(),
            seed_hits: hits.clone(),
            remaining_hits: hits,
            exploding_boxes,
            ray_barriers,
            ray_ports,
            width: self.width,
            height: self.height,
            disable_bombs: self.disable_bombs,
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
    pub(crate) fn grid_size(&self) -> (u8, u8) {
        (self.width, self.height)
    }
    pub(crate) fn bombs_disabled(&self) -> bool {
        self.disable_bombs
    }
    pub fn begin(&mut self, generation: u64) {
        self.generation = generation;
        self.kinds = self.seed.clone();
        self.remaining_hits = self.seed_hits.clone();
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
    /// Reserve the measured landing cell on the room's candidate clone.
    /// This does not simulate an airborne trajectory or grant actor authority.
    pub(crate) fn move_bomb(
        &mut self,
        generation: u64,
        id: u16,
        source: (u8, u8),
        target: (u8, u8),
    ) -> io::Result<()> {
        if self.generation != generation
            || target.0 >= self.height
            || target.1 >= self.width
            || self.kinds[usize::from(target.0) * usize::from(self.width) + usize::from(target.1)]
                != 0
            || self
                .pending
                .values()
                .any(|bomb| bomb.id != id && (bomb.row, bomb.column) == target)
            || self
                .pending
                .get(&id)
                .is_none_or(|bomb| (bomb.row, bomb.column) != source)
        {
            return Err(io::Error::other("unsupported terrain bomb motion"));
        }
        let bomb = self.pending.get_mut(&id).expect("validated current bomb");
        (bomb.row, bomb.column) = target;
        Ok(())
    }
    /// Report-derived bounded Shoe corridor, not original collision authority.
    /// EAB1C0 has an actor-supplied range: a free cell beyond the reported target
    /// does not prove an invalid early stop. Never infer unlimited kick distance.
    pub(crate) fn move_shoe(
        &mut self,
        generation: u64,
        id: u16,
        source: (u8, u8),
        target: (u8, u8),
    ) -> io::Result<()> {
        if source == target
            || source.0 >= self.height
            || target.0 >= self.height
            || source.1 >= self.width
            || target.1 >= self.width
            || (source.0 != target.0 && source.1 != target.1)
        {
            return Err(io::Error::other("unmeasured Shoe corridor"));
        }
        let cells: Vec<_> = if source.0 == target.0 {
            (source.1.min(target.1)..=source.1.max(target.1))
                .map(|column| (source.0, column))
                .collect()
        } else {
            (source.0.min(target.0)..=source.0.max(target.0))
                .map(|row| (row, source.1))
                .collect()
        };
        if cells.into_iter().any(|(row, column)| {
            let index = usize::from(row) * usize::from(self.width) + usize::from(column);
            self.kinds[index] != 0
                || self.ray_barriers[index] != 0
                || self
                    .pending
                    .values()
                    .any(|bomb| bomb.id != id && (bomb.row, bomb.column) == (row, column))
        }) {
            return Err(io::Error::other("blocked Shoe corridor"));
        }
        self.move_bomb(generation, id, source, target)
    }
    fn hit_box(
        &mut self,
        (row, column): (u8, u8),
        generation: u64,
        root: u16,
        cleared: &mut BTreeSet<(u8, u8)>,
        secondary: &mut VecDeque<(u8, u8)>,
    ) {
        let index = usize::from(row) * usize::from(self.width) + usize::from(column);
        if self.kinds[index] != 2 || self.remaining_hits[index] == 0 {
            return;
        }
        self.remaining_hits[index] -= 1;
        if self.remaining_hits[index] == 0 {
            self.kinds[index] = 0;
            cleared.insert((row, column));
            self.cleared
                .entry((row, column))
                .or_insert((generation, root));
            if self.exploding_boxes[index] != 0 {
                secondary.push_back((row, column));
            }
        }
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
        let mut occupied: BTreeMap<_, _> = self
            .pending
            .values()
            .map(|b| ((b.row, b.column), b.id))
            .collect();
        let mut queue = VecDeque::from([root]);
        let mut queued = BTreeSet::from([root]);
        let mut secondary = VecDeque::new();
        let mut processed = Vec::new();
        let mut cleared = BTreeSet::new();
        while !queue.is_empty() || !secondary.is_empty() {
            let mut hit_boxes = BTreeSet::new();
            while let Some(id) = queue.pop_front() {
                let bomb = &self.pending[&id];
                occupied.remove(&(bomb.row, bomb.column));
                processed.push(id);
                let origin =
                    usize::from(bomb.row) * usize::from(self.width) + usize::from(bomb.column);
                if self.ray_barriers[origin] != 0 {
                    continue;
                }
                // Keep the [W, N, E, S] cascade order matching KR BA0E30/17929B0
                // bit indices (0=W, 1=N, 2=E, 3=S) so `direction ^ 2` flips W<->E and N<->S.
                for (direction, dr, dc) in [(0, 0, -1), (1, -1, 0), (2, 0, 1), (3, 1, 0)] {
                    let outgoing = 1 << direction;
                    let incoming = 1 << (direction ^ 2);
                    let mut previous = origin;
                    for distance in 1..=i16::from(bomb.power) {
                        if self.ray_ports[previous] & outgoing == 0 {
                            break;
                        }
                        let row = i16::from(bomb.row) + dr * distance;
                        let column = i16::from(bomb.column) + dc * distance;
                        if row < 0
                            || column < 0
                            || row >= i16::from(self.height)
                            || column >= i16::from(self.width)
                        {
                            break;
                        }
                        let index = row as usize * usize::from(self.width) + column as usize;
                        // E9F220 checks map+1D24 and the incoming side before
                        // entering the cell, so a blocked cell cannot be hit.
                        if self.ray_barriers[index] != 0 || self.ray_ports[index] & incoming == 0 {
                            break;
                        }
                        let cell = (row as u8, column as u8);
                        let kind = self.kinds[index];
                        if kind != 0 {
                            if kind == 2 {
                                hit_boxes.insert(cell);
                            }
                            break;
                        }
                        if let Some(other) = occupied.get(&cell) {
                            if queued.insert(*other) {
                                queue.push_back(*other);
                            }
                            break;
                        }
                        previous = index;
                    }
                }
            }
            // The immutable ray snapshot stops at a box even on its final hit.
            // Multiple chain rays mark one cell once in a callback, as the native
            // shared cell state does; a later secondary timer cannot hit it again.
            for cell in hit_boxes {
                self.hit_box(cell, generation, root, &mut cleared, &mut secondary);
            }
            if let Some((row, column)) = secondary.pop_front() {
                let origin = usize::from(row) * usize::from(self.width) + usize::from(column);
                for (direction, dr, dc) in [(0, 0, -1), (1, -1, 0), (2, 0, 1), (3, 1, 0)] {
                    let outgoing = 1 << direction;
                    let incoming = 1 << (direction ^ 2);
                    if self.ray_ports[origin] & outgoing == 0 {
                        continue;
                    }
                    let nr = i16::from(row) + dr;
                    let nc = i16::from(column) + dc;
                    if nr < 0
                        || nc < 0
                        || nr >= i16::from(self.height)
                        || nc >= i16::from(self.width)
                    {
                        continue;
                    }
                    let index = nr as usize * usize::from(self.width) + nc as usize;
                    if self.ray_barriers[index] != 0 || self.ray_ports[index] & incoming == 0 {
                        continue;
                    }
                    let cell = (nr as u8, nc as u8);
                    if self.kinds[index] == 2 {
                        self.hit_box(cell, generation, root, &mut cleared, &mut secondary);
                    } else if self.kinds[index] == 0
                        && let Some(other) = occupied.get(&cell)
                        && queued.insert(*other)
                    {
                        queue.push_back(*other);
                    }
                }
            }
        }
        for id in &processed {
            self.pending.remove(id);
        }
        Blast { processed, cleared }
    }
    /// Immediately clears a live box cell (used by Drill 127/sub3) and records
    /// clearance for hidden-item reveal without triggering exploding-box chains.
    pub(crate) fn clear_box(&mut self, generation: u64, row: u8, column: u8) -> bool {
        if self.generation != generation || row >= self.height || column >= self.width {
            return false;
        }
        let index = usize::from(row) * usize::from(self.width) + usize::from(column);
        if self.kinds[index] != 2 || self.remaining_hits[index] == 0 {
            return false;
        }
        self.kinds[index] = 0;
        self.remaining_hits[index] = 0;
        self.cleared.entry((row, column)).or_insert((generation, 0));
        true
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
            box_hits: Vec::new(),
            exploding_boxes: Vec::new(),
            ray_barriers: Vec::new(),
            ray_ports: Vec::new(),
            disable_bombs: false,
        }
    }
    #[test]
    fn big_battle_rays_clear_edge_boxes_without_wrapping() {
        let mut cfg = config();
        (cfg.width, cfg.height) = BIG_BATTLE_SIZE;
        cfg.kinds = vec![0; 323];
        for cell in [18, 15 * 19, 15 * 19 + 18, 16 * 19 + 17] {
            cfg.kinds[cell] = 2;
        }
        cfg.seed_sha256 = sha(&cfg.kinds);
        let mut terrain = cfg.validate_measured(b"map", b"start", 9).unwrap();
        terrain.begin(1);
        for (row, column) in [(17, 18), (16, 19)] {
            assert!(
                terrain
                    .place(Bomb {
                        id: 1,
                        owner: 7,
                        row,
                        column,
                        power: 2,
                    })
                    .is_err()
            );
        }
        terrain
            .place(Bomb {
                id: 1,
                owner: 7,
                row: 16,
                column: 18,
                power: 2,
            })
            .unwrap();
        assert_eq!(
            terrain.blast_committed(1, 1).cleared,
            BTreeSet::from([(15, 18), (16, 17)])
        );
        for (width, height) in [(17, 19), (19, 13), (15, 17), STANDARD_SIZE] {
            (cfg.width, cfg.height) = (width, height);
            assert!(cfg.validate_measured(b"map", b"start", 9).is_err());
        }
        (cfg.width, cfg.height) = BIG_BATTLE_SIZE;
        cfg.kinds.pop();
        cfg.seed_sha256 = sha(&cfg.kinds);
        assert!(cfg.validate_measured(b"map", b"start", 9).is_err());
    }

    pub(crate) fn lovely_room_map() -> Terrain {
        let mut cfg = config();
        cfg.disable_bombs = true;
        cfg.validate(b"map", b"start").unwrap()
    }
    pub(crate) fn map() -> Terrain {
        config().validate(b"map", b"start").unwrap()
    }
    pub(crate) fn durable_items_map() -> Terrain {
        let mut terrain = basic_items_map();
        terrain.seed_hits[2 * 15 + 3] = 5;
        terrain.remaining_hits = terrain.seed_hits.clone();
        terrain
    }
    #[test]
    fn ray_barrier_keeps_floor_clear_but_shields_box_and_chain_until_independent_callback() {
        let mut cfg = config();
        cfg.kinds = vec![0; 195];
        cfg.kinds[3 * 15 + 5] = 2;
        cfg.kinds[3 * 15 + 8] = 2;
        cfg.seed_sha256 = sha(&cfg.kinds);
        cfg.maximum_power = 9;
        cfg.ray_barriers = vec![0; 195];
        cfg.ray_barriers[3 * 15 + 4] = 1;
        cfg.ray_barriers[3 * 15 + 7] = 1;
        let mut t = cfg.validate_measured(b"map", b"start", 9).unwrap();
        t.begin(10);
        for (id, column, power) in [(1, 3, 5), (2, 6, 2)] {
            t.place(Bomb {
                id,
                owner: 0,
                row: 3,
                column,
                power,
            })
            .unwrap();
        }
        let blocked = t.blast_committed(10, 1);
        assert_eq!(blocked.processed, vec![1]);
        assert!(blocked.cleared.is_empty());
        assert!(t.pending.contains_key(&2));
        assert_eq!(t.kinds[3 * 15 + 4], 0);
        assert_eq!(t.remaining_hits[3 * 15 + 5], 1);
        assert!(t.clearance(10, 3, 5).is_none());
        let independent = t.blast_committed(10, 2);
        assert_eq!(independent.processed, vec![2]);
        assert_eq!(independent.cleared, BTreeSet::from([(3, 5)]));
        assert_eq!(t.clearance(10, 3, 5), Some(2));
        assert_eq!(t.remaining_hits[3 * 15 + 8], 1);
        assert!(t.blast_committed(10, 2).processed.is_empty());
        t.finish();
        t.begin(11);
        assert_eq!(t.kinds, t.seed);
        assert_eq!(t.remaining_hits[3 * 15 + 5], 1);
        assert_eq!(t.ray_barriers[3 * 15 + 4], 1);
        assert!(t.clearance(10, 3, 5).is_none());
        t.place(Bomb {
            id: 3,
            owner: 0,
            row: 3,
            column: 4,
            power: 9,
        })
        .unwrap();
        assert!(t.blast_committed(10, 3).processed.is_empty());
        let origin_blocked = t.blast_committed(11, 3);
        assert_eq!(origin_blocked.processed, vec![3]);
        assert!(origin_blocked.cleared.is_empty());
        assert_eq!(t.remaining_hits[3 * 15 + 5], 1);
    }

    #[test]
    fn ray_mask_is_optional_bounded_boolean_data_with_existing_map_binding() {
        let mut cfg = config();
        let absent = cfg.validate(b"map", b"start").unwrap();
        cfg.ray_barriers = vec![0; 195];
        let zero = cfg.validate(b"map", b"start").unwrap();
        assert_eq!(absent.ray_barriers, zero.ray_barriers);
        cfg.ray_barriers[0] = 1;
        assert!(
            cfg.validate_for_map(b"map", b"start", &"0".repeat(64))
                .is_ok()
        );
        assert!(
            cfg.validate_for_map(b"map", b"start", &"1".repeat(64))
                .is_err()
        );
        assert!(cfg.validate(b"changed map", b"start").is_err());
        for mask in [vec![0; 194], vec![0; 196], vec![2; 195]] {
            cfg.ray_barriers = mask;
            assert!(cfg.validate(b"map", b"start").is_err());
        }
    }

    #[test]
    fn tent_ports_block_both_horizontal_entries_and_allow_vertical_chains() {
        let mut cfg = config();
        cfg.kinds.fill(0);
        cfg.seed_sha256 = sha(&cfg.kinds);
        cfg.ray_ports = vec![ALL_RAY_PORTS; 195];
        // KR TileInfoList TentBlue (1988), dword +32 = 0x01000100 (bits 1=N, 3=S).
        cfg.ray_ports[6 * 15 + 7] = 0b1010;
        for (source, chains) in [
            ((5, 7), true),
            ((7, 7), true),
            ((6, 6), false),
            ((6, 8), false),
        ] {
            let mut t = cfg.validate(b"map", b"start").unwrap();
            t.begin(1);
            for (id, (row, column)) in [(1, source), (2, (6, 7))] {
                t.place(Bomb {
                    id,
                    owner: 0,
                    row,
                    column,
                    power: 1,
                })
                .unwrap();
            }
            let blast = t.blast_committed(1, 1);
            assert_eq!(blast.processed, if chains { vec![1, 2] } else { vec![1] });
        }
    }

    #[test]
    fn ray_ports_validate_four_bits_and_use_opposite_side_for_entry() {
        let mut cfg = config();
        cfg.kinds.fill(0);
        cfg.kinds[6 * 15 + 8] = 2;
        cfg.seed_sha256 = sha(&cfg.kinds);
        for invalid in [vec![0; 194], vec![16; 195]] {
            cfg.ray_ports = invalid;
            assert!(cfg.validate(b"map", b"start").is_err());
        }
        cfg.ray_ports = vec![ALL_RAY_PORTS; 195];
        // West-only side (bit 0 = W): entry from the west works, but crossing the
        // cell (or leaving that cell as the bomb origin) east must stop.
        cfg.ray_ports[6 * 15 + 7] = 0b0001;
        let mut t = cfg.validate(b"map", b"start").unwrap();
        t.begin(1);
        for (id, column) in [(1, 6), (2, 7)] {
            t.place(Bomb {
                id,
                owner: 0,
                row: 6,
                column,
                power: 2,
            })
            .unwrap();
        }
        let blast = t.blast_committed(1, 1);
        assert_eq!(blast.processed, vec![1, 2]);
        assert!(blast.cleared.is_empty());
        t.begin(2);
        t.place(Bomb {
            id: 1,
            owner: 0,
            row: 6,
            column: 6,
            power: 2,
        })
        .unwrap();
        assert!(t.blast_committed(2, 1).cleared.is_empty());
    }

    #[test]
    fn durable_boxes_validate_counts_keep_last_hit_ray_stop_and_reset_without_old_clearance() {
        let mut cfg = config();
        let normal = cfg.validate(b"map", b"start").unwrap();
        cfg.box_hits = vec![0; 195];
        assert!(cfg.validate(b"map", b"start").is_err());
        cfg.box_hits[3 * 15 + 5] = 5;
        cfg.box_hits[4 * 15 + 4] = 2;
        let mut terrain = cfg.validate(b"map", b"start").unwrap();
        for (index, hits) in [(0, 1), (3 * 15 + 5, 255)] {
            let mut bad = cfg.clone();
            bad.box_hits[index] = hits;
            assert!(bad.validate(b"map", b"start").is_err());
        }
        let mut short = cfg.clone();
        short.box_hits.pop();
        assert!(short.validate(b"map", b"start").is_err());
        assert_eq!(normal.seed_hits[3 * 15 + 5], 1);
        terrain.begin(7);
        for id in 1..=5 {
            terrain
                .place(Bomb {
                    id,
                    owner: 0,
                    row: 3,
                    column: 4,
                    power: 2,
                })
                .unwrap();
            assert!(terrain.clearance(7, 3, 5).is_none());
            assert!(terrain.blast_committed(6, id).processed.is_empty());
            let blast = terrain.blast_committed(7, id);
            assert_eq!(blast.cleared.contains(&(3, 5)), id == 5);
            assert_eq!(terrain.remaining_hits[3 * 15 + 5], 5 - id as u8);
            assert!(terrain.blast_committed(7, id).processed.is_empty());
        }
        assert_eq!(terrain.clearance(7, 3, 5), Some(5));
        terrain.finish();
        terrain.begin(8);
        assert_eq!(terrain.remaining_hits[3 * 15 + 5], 5);
        assert!(terrain.clearance(7, 3, 5).is_none());
        assert!(terrain.clearance(8, 3, 5).is_none());
        assert!(
            terrain
                .place(Bomb {
                    id: 6,
                    owner: 0,
                    row: 3,
                    column: 5,
                    power: 1
                })
                .is_err()
        );
    }
    #[test]
    fn one_chain_callback_decrements_shared_box_once_and_final_hit_does_not_pass_through() {
        let mut cfg = config();
        cfg.kinds = vec![0; 195];
        for (r, c) in [(3, 5), (3, 6)] {
            cfg.kinds[r * 15 + c] = 2;
        }
        cfg.seed_sha256 = sha(&cfg.kinds);
        cfg.maximum_power = 9;
        cfg.box_hits = cfg.kinds.iter().map(|kind| u8::from(*kind == 2)).collect();
        cfg.box_hits[3 * 15 + 5] = 2;
        let mut t = cfg.validate_measured(b"map", b"start", 9).unwrap();
        t.begin(1);
        // Root at3,3 chains both arms; both secondary rays meet the same box.
        for (id, r, c, power) in [(1, 3, 3, 3), (2, 3, 4, 3), (3, 2, 3, 3), (4, 2, 5, 3)] {
            t.place(Bomb {
                id,
                owner: 0,
                row: r,
                column: c,
                power,
            })
            .unwrap();
        }
        assert_eq!(t.blast_committed(1, 1).processed.len(), 4);
        assert_eq!(t.remaining_hits[3 * 15 + 5], 1);
        assert!(t.blast_committed(1, 2).processed.is_empty());
        t.place(Bomb {
            id: 5,
            owner: 0,
            row: 3,
            column: 4,
            power: 9,
        })
        .unwrap();
        assert_eq!(t.blast_committed(1, 5).cleared, BTreeSet::from([(3, 5)]));
        assert_eq!(t.remaining_hits[3 * 15 + 6], 1);
        t.place(Bomb {
            id: 6,
            owner: 0,
            row: 3,
            column: 4,
            power: 9,
        })
        .unwrap();
        assert_eq!(t.blast_committed(1, 6).cleared, BTreeSet::from([(3, 6)]));
    }

    #[test]
    fn selected_map_requires_its_pack_payload_and_exact_map_start_frames() {
        let cfg = config();
        assert!(
            cfg.validate_for_map(b"map", b"start", &"0".repeat(64))
                .is_ok()
        );
        for (map, start, payload) in [
            (&b"other"[..], &b"start"[..], "0".repeat(64)),
            (&b"map"[..], &b"other"[..], "0".repeat(64)),
            (&b"map"[..], &b"start"[..], "1".repeat(64)),
        ] {
            assert!(cfg.validate_for_map(map, start, &payload).is_err());
        }
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
    pub(crate) fn f1_item_map(boxes: &[(usize, usize)]) -> Terrain {
        let mut cfg = config();
        cfg.kinds = vec![0; 195];
        for &(row, col) in boxes {
            cfg.kinds[row * 15 + col] = 2;
        }
        cfg.seed_sha256 = sha(&cfg.kinds);
        cfg.maximum_power = 9;
        cfg.validate_measured(b"map", b"start", 9).unwrap()
    }
    pub(crate) fn motion_map() -> Terrain {
        let mut terrain = basic_items_map();
        terrain.kinds[5 * 15 + 4] = 2;
        terrain.seed[5 * 15 + 4] = 2;
        terrain.remaining_hits[5 * 15 + 4] = 1;
        terrain.seed_hits[5 * 15 + 4] = 1;
        terrain
    }
    pub(crate) fn blocked_shoe_map() -> Terrain {
        let mut terrain = motion_map();
        let index = 4 * 15 + 3;
        terrain.kinds[index] = 2;
        terrain.seed[index] = 2;
        terrain.remaining_hits[index] = 1;
        terrain.seed_hits[index] = 1;
        terrain
    }
    #[test]
    fn shoe_cannot_skip_an_obstacle_or_move_diagonally_and_does_not_mutate_on_rejection() {
        for blocked in [false, true] {
            let mut terrain = if blocked {
                blocked_shoe_map()
            } else {
                motion_map()
            };
            terrain.begin(1);
            terrain
                .place(Bomb {
                    id: 1,
                    owner: 0,
                    row: if blocked { 6 } else { 7 },
                    column: 3,
                    power: 1,
                })
                .unwrap();
            let before = terrain.pending.clone();
            let source = if blocked { (6, 3) } else { (7, 3) };
            let target = if blocked { (3, 3) } else { (4, 4) };
            assert!(terrain.move_shoe(1, 1, source, target).is_err());
            assert_eq!(terrain.pending, before);
        }
    }
    #[test]
    fn shoe_short_stops_and_four_axes_keep_corridor_collision_and_current_allocation_checks() {
        // Synthetic policy controls only. EAB1C0 accepts an actor range, so a
        // clear cell beyond the endpoint cannot by itself reject a short stop.
        for target in [
            (6, 8),
            (6, 14),
            (6, 6),
            (6, 0),
            (5, 7),
            (0, 7),
            (7, 7),
            (12, 7),
        ] {
            let mut terrain = motion_map();
            terrain.seed.fill(0);
            terrain.seed_hits.fill(0);
            terrain.begin(1);
            terrain
                .place(Bomb {
                    id: 1,
                    owner: 0,
                    row: 6,
                    column: 7,
                    power: 1,
                })
                .unwrap();
            let before = terrain.pending.clone();
            assert!(terrain.move_shoe(2, 1, (6, 7), target).is_err());
            assert_eq!(terrain.pending, before);
            let at = usize::from(target.0) * 15 + usize::from(target.1);
            terrain.ray_barriers[at] = 1;
            assert!(terrain.move_shoe(1, 1, (6, 7), target).is_err());
            assert_eq!(terrain.pending, before);
            terrain.ray_barriers[at] = 0;
            terrain
                .place(Bomb {
                    id: 2,
                    owner: 1,
                    row: target.0,
                    column: target.1,
                    power: 1,
                })
                .unwrap();
            let occupied = terrain.pending.clone();
            assert!(terrain.move_shoe(1, 1, (6, 7), target).is_err());
            assert_eq!(terrain.pending, occupied);
            terrain.pending.remove(&2);
            terrain.move_shoe(1, 1, (6, 7), target).unwrap();
            assert_eq!(
                (terrain.pending[&1].row, terrain.pending[&1].column),
                target
            );
            assert!(terrain.move_shoe(1, 1, (6, 7), (6, 9)).is_err());
        }
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
    #[test]
    fn exploding_boxes_validate_flags_cross_clear_chain_and_bomb_cascade() {
        let idx = |row: usize, col: usize| row * 15 + col;
        let mut cfg = config();
        cfg.kinds = vec![0; 195];
        cfg.kinds[idx(2, 3)] = 1; // Solid neighbor should stay intact.
        cfg.kinds[idx(0, 9)] = 0; // Bomb A placed here
        cfg.kinds[idx(0, 10)] = 2; // Block32 exploding
        cfg.kinds[idx(0, 11)] = 2; // Normal Block33 east of Block32
        cfg.kinds[idx(0, 12)] = 2; // Normal box 2 cells east (not touched by 1-step cross)
        cfg.kinds[idx(1, 10)] = 0; // Bomb B placed here (chained by Block32)
        cfg.kinds[idx(2, 10)] = 2; // Normal box reached by Bomb B's ray (power 1)
        cfg.kinds[idx(5, 5)] = 2; // Exploding chain starter
        cfg.kinds[idx(5, 6)] = 2; // Exploding chain second hop
        cfg.kinds[idx(5, 7)] = 2; // Normal box cleared by second hop
        cfg.kinds[idx(4, 5)] = 2; // Protected by ray_barrier
        cfg.ray_barriers = vec![0; 195];
        cfg.ray_barriers[idx(4, 5)] = 1;
        cfg.seed_sha256 = sha(&cfg.kinds);
        // Validation rejects mismatched length, values > 1, or non-box cells marked exploding.
        for bad_flags in [
            vec![0; 194],
            {
                let mut f = vec![0; 195];
                f[idx(0, 10)] = 2;
                f
            },
            {
                let mut f = vec![0; 195];
                f[idx(2, 3)] = 1;
                f
            },
        ] {
            let mut bad = cfg.clone();
            bad.exploding_boxes = bad_flags;
            assert!(bad.validate_measured(b"map", b"start", 9).is_err());
        }
        cfg.exploding_boxes = vec![0; 195];
        cfg.exploding_boxes[idx(0, 10)] = 1;
        cfg.exploding_boxes[idx(5, 5)] = 1;
        cfg.exploding_boxes[idx(5, 6)] = 1;
        let mut t = cfg.validate_measured(b"map", b"start", 9).unwrap();
        t.begin(1);
        t.place(Bomb {
            id: 3,
            owner: 0,
            row: 0,
            column: 9,
            power: 1,
        })
        .unwrap();
        t.place(Bomb {
            id: 4,
            owner: 1,
            row: 1,
            column: 10,
            power: 1,
        })
        .unwrap();
        let blast = t.blast_committed(1, 3);
        assert_eq!(blast.processed, vec![3, 4]);
        assert_eq!(blast.cleared, BTreeSet::from([(0, 10), (0, 11), (2, 10)]));
        assert!(t.clearance(1, 0, 12).is_none());
        t.place(Bomb {
            id: 5,
            owner: 0,
            row: 5,
            column: 4,
            power: 1,
        })
        .unwrap();
        let chain = t.blast_committed(1, 5);
        assert_eq!(chain.cleared, BTreeSet::from([(5, 5), (5, 6), (5, 7)]));
        assert!(t.clearance(1, 4, 5).is_none());
    }
    #[test]
    fn clear_box_removes_live_box_and_records_clearance_without_secondary_chain() {
        let mut cfg = config();
        let idx = |r: usize, c: usize| r * 15 + c;
        cfg.kinds[idx(3, 6)] = 2;
        cfg.seed_sha256 = sha(&cfg.kinds);
        cfg.exploding_boxes = vec![0; 195];
        cfg.exploding_boxes[idx(3, 5)] = 1;
        let mut t = cfg.validate(b"map", b"start").unwrap();
        assert!(!t.clear_box(1, 3, 5));
        t.begin(1);
        assert!(!t.clear_box(1, 0, 0));
        assert!(!t.clear_box(1, 2, 3));
        assert!(!t.clear_box(1, 13, 0));
        assert!(t.clear_box(1, 3, 5));
        assert!(!t.clear_box(1, 3, 5));
        assert_eq!(t.clearance(1, 3, 5), Some(0));
        assert!(t.clearance(1, 3, 6).is_none());
        t.place(Bomb {
            id: 1,
            owner: 0,
            row: 3,
            column: 5,
            power: 1,
        })
        .unwrap();
        t.begin(2);
        assert!(t.clearance(1, 3, 5).is_none());
    }
}
