//! Process-local epochs for managed rooms. These are fences, not credentials.
//! The native peer reader uses a low-byte key. Until wrap/reuse has independent
//! evidence, never lease the same low byte twice in one service instance.
use std::{
    io,
    sync::atomic::{AtomicU64, Ordering},
};

#[derive(Debug)]
pub(crate) struct RoundEpochs {
    last: AtomicU64,
    step: u64,
    maximum: u64,
}
impl Default for RoundEpochs {
    fn default() -> Self {
        Self {
            last: AtomicU64::new(0),
            step: 1,
            maximum: 255,
        }
    }
}

impl RoundEpochs {
    /// Protected companion routes carry independent random IDs and replay
    /// windows, while local endpoints never cross their own session/round.
    /// Repeating the low byte stresses that native UDP key is not authority;
    /// the full TCP wire key remains unique and the internal epoch monotonic.
    pub(crate) fn authenticated() -> Self {
        Self {
            last: AtomicU64::new(0),
            step: 256,
            maximum: u64::from(u32::MAX - 12345),
        }
    }
    pub(crate) fn next(&self) -> io::Result<u64> {
        self.last
            .try_update(Ordering::Relaxed, Ordering::Relaxed, |last| {
                last.checked_add(self.step)
                    .filter(|next| *next <= self.maximum)
            })
            .map(|last| last + self.step)
            .map_err(|_| io::Error::other("managed round key budget exhausted; no key reuse"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{collections::BTreeSet, sync::Arc};

    #[test]
    fn concurrent_rooms_never_share_full_or_low_byte_keys_and_exhaust_closed() {
        let epochs = Arc::new(RoundEpochs::default());
        let workers: Vec<_> = (0..3)
            .map(|_| {
                let epochs = epochs.clone();
                std::thread::spawn(move || {
                    (0..85).map(|_| epochs.next().unwrap()).collect::<Vec<_>>()
                })
            })
            .collect();
        let values: Vec<_> = workers
            .into_iter()
            .flat_map(|worker| worker.join().unwrap())
            .collect();
        assert_eq!(values.iter().copied().collect::<BTreeSet<_>>().len(), 255);
        let keys: BTreeSet<_> = values
            .into_iter()
            .map(|epoch| crate::room::wire_key(epoch).unwrap() as u8)
            .collect();
        assert_eq!(keys.len(), 255);
        for _ in 0..3 {
            assert!(epochs.next().is_err());
        }
    }
}
