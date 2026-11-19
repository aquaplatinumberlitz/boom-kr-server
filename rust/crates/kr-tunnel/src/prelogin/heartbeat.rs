//! Use launcher progress, not filesystem modification time. Windows can
//! leave the mtime unchanged when a zero-length file is written again.
use std::{
    fs::File,
    io::{self, Read, Write},
    path::Path,
    time::{Duration, Instant},
};

const MAX_AGE: Duration = Duration::from_secs(10);

pub fn publish(path: &Path, generation: u64) -> io::Result<()> {
    writeln!(File::create(path)?, "{generation}")
}

pub struct Heartbeat {
    last: Instant,
    generation: Option<u64>,
}
impl Heartbeat {
    pub const fn new(now: Instant) -> Self {
        Self {
            last: now,
            generation: None,
        }
    }

    pub fn expired(&mut self, path: &Path, now: Instant) -> bool {
        let mut input = String::new();
        // Ignore a partial write between truncate and WriteFile. Never let
        // one stale file extend the deadline by being opened repeatedly.
        if File::open(path)
            .and_then(|f| f.take(22).read_to_string(&mut input))
            .is_ok()
            && let Some(line) = input.strip_suffix('\n')
            && let Ok(next) = line.parse::<u64>()
            && self.generation.is_none_or(|previous| next > previous)
        {
            self.generation = Some(next);
            self.last = now;
        }
        now.saturating_duration_since(self.last) >= MAX_AGE
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expired_uses_progress_and_ignores_empty_and_stale_updates() {
        let path = std::env::temp_dir().join(format!(
            "kr-heartbeat-{}-{:x}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let start = Instant::now();
        let at = |seconds| start + Duration::from_secs(seconds);
        let mut beat = Heartbeat::new(start);
        publish(&path, 0).unwrap();
        assert!(!beat.expired(&path, at(0)));
        std::fs::write(&path, []).unwrap();
        assert!(!beat.expired(&path, at(9)));
        assert!(beat.expired(&path, at(10)));
        let mut beat = Heartbeat::new(start);
        publish(&path, 0).unwrap();
        assert!(!beat.expired(&path, at(0)));
        publish(&path, 1).unwrap();
        assert!(!beat.expired(&path, at(9)));
        publish(&path, 2).unwrap();
        assert!(!beat.expired(&path, at(18)));
        publish(&path, 1).unwrap();
        assert!(beat.expired(&path, at(28)));
        std::fs::remove_file(path).unwrap();
    }
}
