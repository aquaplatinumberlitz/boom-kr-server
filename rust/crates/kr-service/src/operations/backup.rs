use crate::{accounts, platform};
use serde::Deserialize;
use std::{
    fs, io,
    path::{Path, PathBuf},
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tokio::{
    sync::{mpsc, watch},
    time::{self, MissedTickBehavior},
};

#[derive(Clone, Copy, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BackupSchedule {
    pub interval_seconds: u64,
    pub keep: usize,
}
impl BackupSchedule {
    pub(crate) fn validate(&self) -> io::Result<()> {
        if !(1..=604_800).contains(&self.interval_seconds) || !(1..=1000).contains(&self.keep) {
            return Err(io::Error::other("invalid backup interval or retention"));
        }
        Ok(())
    }
}

pub(crate) struct Scheduled {
    db: PathBuf,
    directory: PathBuf,
    schedule: BackupSchedule,
}
fn name_time(name: &str) -> Option<u128> {
    let time = name.strip_prefix("scheduled-")?.strip_suffix(".sqlite")?;
    (time.len() == 39 && time.bytes().all(|b| b.is_ascii_digit()))
        .then(|| time.parse().ok())
        .flatten()
}
impl Scheduled {
    pub(crate) fn open(db: &Path, schedule: BackupSchedule) -> io::Result<Self> {
        schedule.validate()?;
        let directory = db.with_extension("backups");
        match platform::create_private_dir(&directory) {
            Ok(()) => {}
            Err(e) if e.kind() == io::ErrorKind::AlreadyExists => {}
            Err(e) => return Err(e),
        }
        platform::private_dir(&directory)?;
        Ok(Self {
            db: db.to_owned(),
            directory,
            schedule,
        })
    }
    fn completed(&self) -> io::Result<Vec<(u128, PathBuf)>> {
        let mut backups = Vec::new();
        for dir_entry in fs::read_dir(&self.directory)? {
            let entry = dir_entry?;
            if entry.file_type()?.is_file()
                && let Some(created) = entry.file_name().to_str().and_then(name_time)
            {
                backups.push((created, entry.path()));
            }
        }
        backups.sort_unstable_by_key(|(created, _)| std::cmp::Reverse(*created));
        Ok(backups)
    }
    pub(super) fn run_once(&self, at: u128) -> io::Result<i32> {
        let created = self.completed()?.first().map_or(Ok(at), |(old, _)| {
            Ok::<_, io::Error>(
                at.max(
                    old.checked_add(1)
                        .ok_or_else(|| io::Error::other("backup sequence exhausted"))?,
                ),
            )
        })?;
        let name = format!("scheduled-{created:039}.sqlite");
        let version =
            accounts::snapshot(&self.db, &self.directory.join(name)).map_err(io::Error::other)?;
        // Keep the last verified generations. Never prune on a failed snapshot
        // or touch a migration snapshot/manual file.
        for (_, path) in self.completed()?.into_iter().skip(self.schedule.keep) {
            fs::remove_file(path)?;
        }
        platform::sync_directory(&self.directory)?;
        Ok(version)
    }
    pub(crate) async fn run(
        self,
        mut stop: watch::Receiver<bool>,
        log: mpsc::Sender<serde_json::Value>,
    ) -> io::Result<()> {
        let period = Duration::from_secs(self.schedule.interval_seconds);
        let mut clock = time::interval_at(time::Instant::now() + period, period);
        clock.set_missed_tick_behavior(MissedTickBehavior::Delay);
        let job = std::sync::Arc::new(self);
        loop {
            tokio::select! {
                biased;
                _ = stop.changed() => return Ok(()),
                _ = clock.tick() => {}
            }
            let job = job.clone();
            let backup = tokio::task::spawn_blocking(move || {
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map_err(io::Error::other)?;
                job.run_once(now.as_nanos())
            })
            .await
            .map_err(io::Error::other)?;
            let event = match backup {
                Ok(version) => serde_json::json!({"event":"backup_complete","schema":version}),
                Err(_) => serde_json::json!({"event":"backup_failed"}),
            };
            crate::emit(&log, event).await?;
        }
    }
}
