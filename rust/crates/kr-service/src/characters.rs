//! Scoped KR00967 character data. Source: BomberAttr keys0/1, cross-checked
//! against native six-stat wrappers in CP282 and m7e-start-stat-probe-r2.
//! These are client levels, never physical movement units or item authority.
use std::{collections::BTreeMap, io};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Stats {
    pub capacity: usize,
    pub maximum_capacity: usize,
    pub power: u8,
    pub maximum_power: u8,
    pub speed: u8,
    pub maximum_speed: u8,
}
impl Stats {
    /// Measured Bomb/Fire/Roller levels; never physical movement authority.
    pub(crate) fn basic_pickup(&mut self, kind: u16) -> io::Result<()> {
        match kind {
            0 => self.capacity = self.capacity.saturating_add(1).min(self.maximum_capacity),
            1 => self.power = self.power.saturating_add(1).min(self.maximum_power),
            2 => self.speed = self.speed.saturating_add(1).min(self.maximum_speed),
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
            if stats.capacity == 0
                || stats.capacity > stats.maximum_capacity
                || stats.maximum_capacity > 10
                || stats.power == 0
                || stats.power > stats.maximum_power
                || stats.maximum_power > 9
                || stats.speed == 0
                || stats.speed > stats.maximum_speed
                || stats.maximum_speed > 10
            {
                return Err(io::Error::other("invalid measured character bounds"));
            }
            if rows.insert(key, stats).is_some() {
                return Err(io::Error::other("duplicate character profile"));
            }
        }
        Ok(Self(rows))
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
}
