//! Local diagnostics. Only static labels and numeric metadata enter this log.
use std::{
    fmt,
    fs::{File, OpenOptions},
    io::{self, Write},
    path::Path,
    sync::{Arc, Mutex},
    time::Instant,
};

pub const LOG_NAME: &str = "prelogin.log";
const LIMIT: usize = 32 * 1024;
const TRUNCATED: &[u8] = b"event=log_limit\n";

#[derive(Clone, Default)]
pub struct Trace(Option<Arc<Mutex<Log>>>);

struct Log {
    start: Instant,
    file: File,
    remaining: usize,
}

impl Trace {
    /// The parent is the owner's unique run directory. Never overwrite a log.
    pub fn create(run: &Path) -> io::Result<Self> {
        let file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(run.join(LOG_NAME))?;
        Ok(Self(Some(Arc::new(Mutex::new(Log {
            start: Instant::now(),
            file,
            remaining: LIMIT,
        })))))
    }

    pub(super) fn event(&self, event: &'static str, fields: fmt::Arguments<'_>) {
        let Some(mut log) = self.0.as_ref().and_then(|log| log.lock().ok()) else {
            return;
        };
        if log.remaining == 0 {
            return;
        }
        let line = format!(
            "ms={} event={event}{fields}\n",
            log.start.elapsed().as_millis()
        );
        if line.len() + TRUNCATED.len() > log.remaining {
            let _ = log.file.write_all(TRUNCATED);
            log.remaining = 0;
        } else if log.file.write_all(line.as_bytes()).is_ok() {
            log.remaining -= line.len();
        } else {
            log.remaining = 0;
        }
    }

    pub fn startup(&self, stage: Startup, success: bool) {
        self.event(
            "startup",
            format_args!(" stage={} ok={success}", stage.label()),
        );
    }

    pub fn stop(&self, cause: StopCause) {
        self.event("stop_requested", format_args!(" reason={}", cause.label()));
    }
}

/// The binary passes fixed labels, never open-ended error messages or paths.
pub enum Startup {
    Endpoint,
    RoomAdmission,
    Ready,
}
impl Startup {
    const fn label(self) -> &'static str {
        match self {
            Self::Endpoint => "endpoint",
            Self::RoomAdmission => "room_admission",
            Self::Ready => "ready",
        }
    }
}
pub enum StopCause {
    Cancel,
    Heartbeat,
    Interrupt,
}
impl StopCause {
    const fn label(self) -> &'static str {
        match self {
            Self::Cancel => "cancel",
            Self::Heartbeat => "heartbeat",
            Self::Interrupt => "interrupt",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trace_never_exceeds_limit_and_does_not_replace_existing_log() {
        let dir = std::env::temp_dir().join(format!(
            "kr-prelogin-trace-{}-{:x}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir(&dir).unwrap();
        let trace = Trace::create(&dir).unwrap();
        assert!(Trace::create(&dir).is_err());
        for i in 0..5000 {
            trace.event("tcp_accept", format_args!(" port=3838 id={i}"));
        }
        let data = std::fs::read(dir.join(LOG_NAME)).unwrap();
        assert!(data.len() <= LIMIT && data.ends_with(TRUNCATED));
        drop(trace);
        std::fs::remove_dir_all(dir).unwrap();
    }
}
