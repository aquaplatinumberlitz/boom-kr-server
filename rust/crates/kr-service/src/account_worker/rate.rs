//! In-memory per-principal fixed-window admission; backend design, not a durable
//! abuse defense. Every clone of one account worker shares this bounded table.
use super::{Error, Result};
use sha2::{Digest, Sha256};
use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

#[derive(Clone, Copy)]
pub struct RatePolicy {
    pub attempts: u32,
    pub interval: Duration,
    pub principals: usize,
}
impl Default for RatePolicy {
    fn default() -> Self {
        Self {
            attempts: 5,
            interval: Duration::from_secs(60),
            principals: 1024,
        }
    }
}
struct Window {
    since: Instant,
    attempts: u32,
}
pub(super) struct Limiter {
    policy: RatePolicy,
    entries: HashMap<[u8; 32], Window>,
}
impl Limiter {
    pub(super) fn new(policy: RatePolicy) -> Result<Self> {
        if !(1..=1000).contains(&policy.attempts)
            || !(1..=4096).contains(&policy.principals)
            || !(Duration::from_secs(1)..=Duration::from_secs(3600)).contains(&policy.interval)
        {
            return Err(Error::InvalidInput);
        }
        Ok(Self {
            policy,
            entries: HashMap::new(),
        })
    }
    pub(super) fn key(name: &str) -> [u8; 32] {
        Sha256::digest(name.to_ascii_lowercase().as_bytes()).into()
    }
    // Called under the shared admission mutex. Expired entries may be dropped;
    // active principals are never evicted just to admit new attacker keys.
    pub(super) fn check(&mut self, key: [u8; 32], now: Instant) -> Result<()> {
        if let Some(w) = self.entries.get(&key) {
            let elapsed = now
                .checked_duration_since(w.since)
                .ok_or(Error::RateLimited)?;
            if elapsed < self.policy.interval {
                return if w.attempts < self.policy.attempts {
                    Ok(())
                } else {
                    Err(Error::RateLimited)
                };
            }
            return Ok(());
        }
        if self.entries.len() == self.policy.principals {
            self.entries.retain(|_, w| {
                now.checked_duration_since(w.since)
                    .is_none_or(|elapsed| elapsed < self.policy.interval)
            });
        }
        if self.entries.len() == self.policy.principals {
            return Err(Error::RateLimited);
        }
        Ok(())
    }
    // Commit only once a command is actually queued. Full/closed queue consumes
    // no attempt; later caller cancellation still consumes admitted work.
    pub(super) fn commit(&mut self, key: [u8; 32], now: Instant) {
        let w = self.entries.entry(key).or_insert(Window {
            since: now,
            attempts: 0,
        });
        if now.duration_since(w.since) >= self.policy.interval {
            w.since = now;
            w.attempts = 0;
        }
        w.attempts += 1;
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn casefold_bounds_exact_window_and_no_active_eviction() {
        let now = Instant::now();
        let alice = Limiter::key("ALICE");
        assert_eq!(alice, Limiter::key("alice"));
        let mut l = Limiter::new(RatePolicy {
            attempts: 2,
            interval: Duration::from_secs(60),
            principals: 1,
        })
        .unwrap();
        for _ in 0..2 {
            l.check(alice, now).unwrap();
            l.commit(alice, now);
        }
        assert_eq!(
            l.check(alice, now + Duration::from_secs(59)),
            Err(Error::RateLimited)
        );
        assert_eq!(
            l.check(Limiter::key("unknown"), now),
            Err(Error::RateLimited)
        );
        assert_eq!(
            l.check(alice, now - Duration::from_secs(1)),
            Err(Error::RateLimited)
        );
        l.check(alice, now + Duration::from_secs(60)).unwrap();
        l.commit(alice, now + Duration::from_secs(60));
        assert_eq!(l.entries.len(), 1);
        l.check(Limiter::key("unknown"), now + Duration::from_secs(120))
            .unwrap();
        l.commit(Limiter::key("unknown"), now + Duration::from_secs(120));
        assert_eq!(l.entries.len(), 1);
        assert!(
            Limiter::new(RatePolicy {
                attempts: 0,
                ..RatePolicy::default()
            })
            .is_err()
        );
        assert!(
            Limiter::new(RatePolicy {
                principals: 4097,
                ..RatePolicy::default()
            })
            .is_err()
        );
    }
}
