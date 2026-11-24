//! KR 00967 character levels. The baseline keys 0/1 were checked against
//! native six-stat wrappers in CP282 and m7e-start-stat-probe-r2.
//! These are client levels, never physical movement units or item authority.
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, io};

// KR 00967 BomberAttr: key, speed/max, capacity/max, power/max.
// Only the 33 members of cluster_character in f1-full-ida-clusters.json.
// The same-group rows 4019, 4025 and 4026 are excluded controls.
const CLASS_C_ROWS: [(u16, u8, u8, usize, usize, u8, u8); 33] = [
    (4000, 5, 9, 2, 9, 1, 9),
    (4001, 5, 8, 1, 9, 2, 9),
    (4002, 5, 9, 1, 9, 1, 8),
    (4003, 5, 9, 2, 8, 2, 8),
    (4004, 6, 9, 3, 9, 3, 7),
    (4005, 6, 9, 3, 9, 3, 7),
    (4006, 7, 10, 3, 10, 3, 10),
    (4007, 6, 9, 3, 9, 3, 7),
    (4008, 6, 9, 3, 9, 3, 7),
    (4009, 6, 9, 3, 9, 3, 7),
    (4010, 6, 9, 3, 9, 3, 7),
    (4011, 6, 9, 3, 9, 3, 7),
    (4012, 6, 9, 3, 9, 3, 7),
    (4013, 6, 9, 3, 9, 3, 7),
    (4014, 6, 9, 3, 9, 3, 7),
    (4015, 6, 9, 3, 9, 3, 7),
    (4016, 6, 9, 3, 9, 3, 7),
    (4017, 5, 9, 2, 9, 1, 7),
    (4018, 7, 10, 3, 10, 3, 10),
    (4020, 6, 9, 3, 9, 3, 7),
    (4021, 6, 9, 3, 9, 3, 7),
    (4022, 6, 9, 3, 9, 3, 7),
    (4023, 6, 9, 3, 9, 3, 7),
    (4024, 6, 9, 3, 7, 5, 8),
    (4027, 6, 9, 3, 9, 3, 7),
    (4028, 8, 10, 2, 9, 3, 6),
    (4029, 7, 10, 3, 10, 3, 10),
    (4030, 7, 10, 3, 10, 3, 10),
    (5000, 4, 9, 1, 9, 3, 7),
    (5001, 4, 9, 1, 9, 3, 7),
    (5002, 4, 9, 1, 9, 3, 7),
    (5003, 2, 2, 1, 1, 3, 7),
    (5004, 4, 9, 1, 9, 3, 7),
];

fn class_c_index(key: u16) -> Option<usize> {
    CLASS_C_ROWS.binary_search_by_key(&key, |row| row.0).ok()
}

pub(crate) fn is_class_c(key: u16) -> bool {
    class_c_index(key).is_some()
}

fn class_c_stats(key: u16) -> Option<Stats> {
    let (_, speed, maximum_speed, capacity, maximum_capacity, power, maximum_power) =
        CLASS_C_ROWS[class_c_index(key)?];
    Some(Stats {
        capacity,
        maximum_capacity,
        power,
        maximum_power,
        speed,
        maximum_speed,
    })
}

/// C2S79 carries one byte per local player. For a Class C choice the KR
/// writer can send the low byte or its fallback 5 (ownership check failed).
/// Neither byte is a new selection or evidence of the client's active actor.
pub(crate) fn class_c_loading79_matches(key: u16, selected: u8) -> bool {
    is_class_c(key) && (selected == key.to_le_bytes()[0] || selected == 5)
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct Stats {
    pub capacity: usize,
    pub maximum_capacity: usize,
    pub power: u8,
    pub maximum_power: u8,
    pub speed: u8,
    pub maximum_speed: u8,
}
impl Stats {
    fn valid(&self) -> bool {
        self.capacity > 0
            && self.capacity <= self.maximum_capacity
            && self.maximum_capacity <= 10
            && self.power > 0
            && self.power <= self.maximum_power
            && self.maximum_power <= 10
            && self.speed > 0
            && self.speed <= self.maximum_speed
            && self.maximum_speed <= 10
    }

    /// Measured Bomb/Fire/Roller/MaxPower levels; never movement authority.
    pub(crate) fn basic_pickup(&mut self, kind: u16) -> io::Result<()> {
        match kind {
            0 => self.capacity = self.capacity.saturating_add(1).min(self.maximum_capacity),
            1 => self.power = self.power.saturating_add(1).min(self.maximum_power),
            2 => self.speed = self.speed.saturating_add(1).min(self.maximum_speed),
            // KR CP045: effect6 changes current power to that character's max.
            // Capacity and speed retain their current values.
            5 => self.power = self.maximum_power,
            _ => return Err(io::Error::other("unmeasured basic item effect")),
        }
        Ok(())
    }
}
#[derive(Clone)]
pub(crate) struct Catalog(BTreeMap<u16, Stats>);
impl Catalog {
    pub(crate) fn measured(keys: &[u16]) -> io::Result<Self> {
        let mut rows = BTreeMap::new();
        if keys.is_empty() || !keys.contains(&0) {
            return Err(io::Error::other(
                "character catalog needs measured default0",
            ));
        }
        for &key in keys {
            let stats = match key {
                0 => Stats {
                    capacity: 1,
                    maximum_capacity: 10,
                    power: 1,
                    maximum_power: 7,
                    speed: 5,
                    maximum_speed: 7,
                },
                1 => Stats {
                    capacity: 2,
                    maximum_capacity: 7,
                    power: 1,
                    maximum_power: 9,
                    speed: 4,
                    maximum_speed: 8,
                },
                _ => return Err(io::Error::other("unmeasured character profile")),
            };
            if rows.insert(key, stats).is_some() {
                return Err(io::Error::other("duplicate character profile"));
            }
        }
        Ok(Self(rows))
    }
    pub(crate) fn from_pack(keys: &[u16], rows: BTreeMap<u16, Stats>) -> io::Result<Self> {
        let baseline = Self::measured(&[0, 1])?;
        if !keys.contains(&0)
            || rows.len() != keys.len()
            || !keys.iter().all(|key| rows.contains_key(key))
            || keys
                .iter()
                .copied()
                .collect::<std::collections::BTreeSet<_>>()
                .len()
                != keys.len()
            || rows.iter().any(|(&key, row)| match key {
                0 | 1 => baseline.0.get(&key) != Some(row),
                // Matching the pack export scope is intentional. A key that
                // fits the wire does not automatically qualify for F1.
                2..=11 | 120..=131 | 150..=151 | 3000..=3004 => !row.valid(),
                _ => true,
            })
        {
            return Err(io::Error::other("invalid F1 pack character catalog"));
        }
        Ok(Self(rows))
    }
    pub(crate) fn player_round(
        keys: &[u16],
        pack_stats: Option<&BTreeMap<u16, Stats>>,
        allow_class_c_characters: bool,
    ) -> io::Result<Self> {
        let mut ordinary = Vec::with_capacity(keys.len());
        let mut class_c = BTreeMap::new();
        for &key in keys {
            if allow_class_c_characters && let Some(stats) = class_c_stats(key) {
                if !stats.valid() || class_c.insert(key, stats).is_some() {
                    return Err(io::Error::other("invalid F1 Class C character catalog"));
                }
            } else {
                ordinary.push(key);
            }
        }
        let mut catalog = match pack_stats {
            Some(rows) => Self::from_pack(&ordinary, rows.clone())?,
            None => Self::measured(&ordinary)?,
        };
        catalog.0.extend(class_c);
        Ok(catalog)
    }
    pub(crate) fn maximum_power(&self) -> u8 {
        self.0
            .values()
            .map(|stats| stats.maximum_power)
            .max()
            .expect("nonempty catalog")
    }
    pub(crate) fn get(&self, key: u16) -> io::Result<Stats> {
        self.0
            .get(&key)
            .copied()
            .ok_or_else(|| io::Error::other("character not in active catalog"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn basic_levels_follow_measured_character_vectors_clamp_and_refuse_unknown_kind() {
        let catalog = Catalog::measured(&[0, 1]).unwrap();
        for (key, after, limits) in [(0, (2, 2, 6), (10, 7, 7)), (1, (3, 2, 5), (7, 9, 8))] {
            let initial = catalog.get(key).unwrap();
            let mut stats = initial;
            for kind in [2, 1, 0] {
                stats.basic_pickup(kind).unwrap();
            }
            assert_eq!((stats.capacity, stats.power, stats.speed), after);
            for _ in 0..12 {
                for kind in [0, 1, 2] {
                    stats.basic_pickup(kind).unwrap();
                }
            }
            assert_eq!((stats.capacity, stats.power, stats.speed), limits);
            let prior = stats;
            assert!(stats.basic_pickup(145).is_err());
            assert_eq!(stats, prior);
            assert_eq!(catalog.get(key).unwrap(), initial);
        }
    }
    #[test]
    fn f1_pack_catalog_serves_all_scoped_b_keys_and_rejects_skill_branch() {
        let row = |speed, max_speed, capacity, max_capacity, power, max_power| Stats {
            capacity,
            maximum_capacity: max_capacity,
            power,
            maximum_power: max_power,
            speed,
            maximum_speed: max_speed,
        };
        let keys = [0, 1, 2, 120, 150, 3000];
        let rows = BTreeMap::from([
            (0, row(5, 7, 1, 10, 1, 7)),
            (1, row(4, 8, 2, 7, 1, 9)),
            (2, row(5, 8, 1, 8, 1, 8)),
            (120, row(5, 9, 2, 9, 1, 7)),
            (150, row(5, 10, 2, 9, 2, 8)),
            (3000, row(6, 9, 3, 9, 2, 9)),
        ]);
        let catalog = Catalog::from_pack(&keys, rows).unwrap();
        assert_eq!(catalog.get(3000).unwrap().maximum_capacity, 9);
        assert_eq!(catalog.get(150).unwrap().power, 2);
        assert!(
            Catalog::from_pack(
                &[0, 1, 4019],
                BTreeMap::from([
                    (0, row(5, 7, 1, 10, 1, 7)),
                    (1, row(4, 8, 2, 7, 1, 9)),
                    (4019, row(6, 9, 3, 9, 3, 7)),
                ])
            )
            .is_err()
        );
    }

    #[test]
    fn f1_pack_catalog_matches_every_kr_00967_character_vector_and_each_pickup_cap() {
        // BomberAttr KR 00967, fields are key, group, speed/max, capacity/max,
        // power/max. Keeping the complete vector here makes a pack regression
        // fail at the Rust boundary instead of silently serving a partial
        // roster.
        let rows = [
            (0, 5, 7, 1, 10, 1, 7),
            (1, 4, 8, 2, 7, 1, 9),
            (2, 5, 8, 1, 8, 1, 5),
            (3, 4, 8, 1, 10, 1, 8),
            (4, 4, 8, 2, 9, 1, 6),
            (5, 5, 9, 1, 6, 1, 7),
            (6, 5, 8, 1, 6, 2, 7),
            (7, 4, 8, 1, 9, 2, 8),
            (8, 6, 10, 2, 9, 1, 7),
            (9, 5, 10, 3, 9, 1, 7),
            (10, 6, 10, 2, 9, 1, 7),
            (11, 6, 10, 2, 9, 1, 7),
            (120, 5, 9, 2, 9, 1, 7),
            (121, 5, 9, 2, 8, 1, 9),
            (122, 5, 8, 2, 8, 2, 7),
            (123, 6, 10, 2, 8, 1, 8),
            (124, 5, 10, 3, 9, 1, 7),
            (125, 6, 10, 2, 9, 2, 8),
            (126, 7, 9, 1, 9, 1, 8),
            (127, 6, 8, 2, 9, 1, 9),
            (128, 5, 9, 2, 9, 2, 8),
            (129, 6, 10, 3, 9, 2, 8),
            (130, 6, 10, 2, 9, 2, 8),
            (131, 6, 10, 2, 9, 2, 8),
            (150, 5, 10, 2, 9, 2, 8),
            (151, 7, 10, 2, 10, 1, 7),
            (3000, 6, 9, 3, 9, 3, 7),
            (3001, 5, 9, 2, 9, 2, 8),
            (3002, 5, 9, 2, 9, 2, 8),
            (3003, 4, 6, 3, 10, 2, 10),
            (3004, 7, 10, 1, 7, 1, 9),
        ];
        let keys: Vec<_> = rows.iter().map(|row| row.0).collect();
        let map = rows
            .iter()
            .map(
                |&(key, speed, maximum_speed, capacity, maximum_capacity, power, maximum_power)| {
                    (
                        key,
                        Stats {
                            capacity,
                            maximum_capacity,
                            power,
                            maximum_power,
                            speed,
                            maximum_speed,
                        },
                    )
                },
            )
            .collect();
        let catalog = Catalog::from_pack(&keys, map).unwrap();
        assert_eq!(keys.len(), 31);
        for &(key, speed, maximum_speed, capacity, maximum_capacity, power, maximum_power) in &rows
        {
            let seed = catalog.get(key).unwrap();
            assert_eq!(
                (
                    seed.speed,
                    seed.maximum_speed,
                    seed.capacity,
                    seed.maximum_capacity,
                    seed.power,
                    seed.maximum_power
                ),
                (
                    speed,
                    maximum_speed,
                    capacity,
                    maximum_capacity,
                    power,
                    maximum_power
                ),
            );
            let mut capped = seed;
            for _ in 0..12 {
                for kind in [0, 1, 2] {
                    capped.basic_pickup(kind).unwrap();
                }
            }
            capped.basic_pickup(5).unwrap();
            assert_eq!(
                (capped.capacity, capped.power, capped.speed),
                (maximum_capacity, maximum_power, maximum_speed),
            );
            assert_eq!(catalog.get(key).unwrap(), seed);
        }
        for key in [4000, 4019, 4025, 4026] {
            let mut invalid = rows
                .iter()
                .map(|&(key, ..)| (key, catalog.get(key).unwrap()))
                .collect::<BTreeMap<_, _>>();
            invalid.insert(key, catalog.get(0).unwrap());
            let mut keys_with_invalid = keys.clone();
            keys_with_invalid.push(key);
            assert!(
                Catalog::from_pack(&keys_with_invalid, invalid).is_err(),
                "key {key}"
            );
        }
    }

    #[test]
    fn maxpower_uses_each_character_cap_without_changing_other_levels_or_seed() {
        let catalog = Catalog::measured(&[0, 1]).unwrap();
        for (key, maximum) in [(0, 7), (1, 9)] {
            let seed = catalog.get(key).unwrap();
            let mut stats = seed;
            stats.basic_pickup(0).unwrap();
            stats.basic_pickup(2).unwrap();
            let before = stats;
            stats.basic_pickup(5).unwrap();
            assert_eq!(stats.power, maximum);
            assert_eq!(
                (stats.capacity, stats.speed, stats.maximum_power),
                (before.capacity, before.speed, before.maximum_power)
            );
            let capped = stats;
            for kind in [5, 1, 5] {
                stats.basic_pickup(kind).unwrap();
                assert_eq!(stats, capped);
            }
            assert_eq!(catalog.get(key).unwrap(), seed);
            assert_eq!(seed.power, 1);
        }
    }
}
