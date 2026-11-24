//! Rotating diagnostic log written into the same directory opened by Settings -> Open logs.
use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

const LOG_DIR: &str = "prelogin-logs";
const LOG_FILE: &str = "launcher.log";
const OLD_FILE: &str = "launcher.old.log";
const LOG_LIMIT: u64 = 1024 * 1024;
const MAX_DETAIL: usize = 400;

/// Append one privacy-safe diagnostic line to `<state>/prelogin-logs/launcher.log`.
pub(crate) fn record(state: &Path, event: &str, detail: &str) {
    record_with_limit(state, event, detail, LOG_LIMIT);
}

fn record_with_limit(state: &Path, event: &str, detail: &str, limit: u64) {
    let dir = state.join(LOG_DIR);
    if fs::create_dir_all(&dir).is_err() {
        return;
    }
    let path = dir.join(LOG_FILE);
    if let Ok(meta) = fs::symlink_metadata(&path) {
        if meta.file_type().is_symlink() {
            return;
        }
        if meta.len() >= limit {
            let old = dir.join(OLD_FILE);
            if fs::symlink_metadata(&old).is_ok_and(|m| m.file_type().is_symlink()) {
                return;
            }
            // Remove any prior backup first because Windows rename fails if destination exists.
            let _ = fs::remove_file(&old);
            let _ = fs::rename(&path, &old);
        }
    }
    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(&path) {
        let _ = writeln!(file, "[{}] {event}: {}", utc_stamp(), sanitize(detail));
    }
}

fn sanitize(detail: &str) -> String {
    let mut cleaned = String::with_capacity(detail.len().min(MAX_DETAIL));
    for word in detail.split_whitespace() {
        if !cleaned.is_empty() {
            cleaned.push(' ');
        }
        let token = if word.starts_with("S1-") || word.starts_with("O1-") {
            "<redacted-invite>"
        } else {
            word
        };
        if cleaned.len() + token.len() > MAX_DETAIL {
            let mut remain = MAX_DETAIL.saturating_sub(cleaned.len());
            while remain > 0 && !token.is_char_boundary(remain) {
                remain -= 1;
            }
            cleaned.push_str(&token[..remain]);
            cleaned.push('…');
            break;
        }
        cleaned.push_str(token);
    }
    cleaned
}

fn utc_stamp() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let days = (secs / 86_400) as i64;
    let rem = secs % 86_400;
    let hour = rem / 3_600;
    let minute = (rem % 3_600) / 60;
    let second = rem % 60;
    let (year, month, day) = civil_from_days(days);
    format!("{year:04}-{month:02}-{day:02} {hour:02}:{minute:02}:{second:02}Z")
}

// Howard Hinnant's civil-from-days algorithm.
fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u64;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let y = (yoe as i64) + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = y + i64::from(m <= 2);
    (year, m as u32, d as u32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redacts_invites_and_flattens_newlines() {
        let out = sanitize("connect failed\nfor S1-secretcode and O1-roomcode\tok");
        assert!(!out.contains("secretcode"));
        assert!(!out.contains("roomcode"));
        assert!(!out.contains('\n'));
        assert!(!out.contains('\t'));
        assert!(out.contains("<redacted-invite>"));
        assert!(out.ends_with("ok"));
    }

    #[test]
    fn rotates_launcher_log_at_limit() {
        let root = std::env::temp_dir().join(format!(
            "kr-player-diag-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        fs::create_dir_all(&root).unwrap();
        record_with_limit(&root, "first", "alpha", 128);
        record_with_limit(&root, "second", &"x".repeat(120), 128);
        record_with_limit(&root, "third", "beta", 128);
        let dir = root.join(LOG_DIR);
        let current = fs::read_to_string(dir.join(LOG_FILE)).unwrap();
        let old = fs::read_to_string(dir.join(OLD_FILE)).unwrap();
        assert!(current.contains("third: beta"));
        assert!(old.contains("second:"));
        fs::remove_dir_all(root).unwrap();
    }
}
