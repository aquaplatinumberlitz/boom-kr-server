//! Snapshots use an independent SQLite reader, never a raw copy of a WAL DB.
use super::*;
use std::{ffi::OsString, path::PathBuf};

fn storage<T>(value: std::io::Result<T>) -> Result<T> {
    value.map_err(|_| Error::Storage)
}

pub(super) fn private_parent(path: &Path) -> Result<&Path> {
    let parent = path.parent().ok_or(Error::InvalidInput)?;
    let meta = storage(fs::symlink_metadata(parent))?;
    if !meta.is_dir() || meta.permissions().mode() & 0o077 != 0 {
        return Err(Error::InvalidInput);
    }
    Ok(parent)
}
fn read_database(path: &Path) -> Result<Connection> {
    private_parent(path)?;
    let meta = storage(fs::symlink_metadata(path))?;
    if !meta.is_file() || meta.permissions().mode() & 0o077 != 0 {
        return Err(Error::InvalidInput);
    }
    let read = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
    read.busy_timeout(Duration::from_secs(2))?;
    read.pragma_update(None, "trusted_schema", false)?;
    Ok(read)
}
fn absent(path: &Path) -> Result<()> {
    match fs::symlink_metadata(path) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Ok(_) => Err(Error::Conflict),
        Err(_) => Err(Error::Storage),
    }
}
fn no_sidecars(path: &Path) -> Result<()> {
    for suffix in ["-wal", "-shm", "-journal"] {
        let mut sidecar = path.as_os_str().to_owned();
        sidecar.push(suffix);
        absent(Path::new(&sidecar))?;
    }
    Ok(())
}
pub fn verify_snapshot(path: &Path) -> Result<i32> {
    no_sidecars(path)?;
    let read = read_database(path)?;
    let integrity: String = read.query_row("PRAGMA integrity_check", [], |r| r.get(0))?;
    if integrity != "ok" || read.prepare("PRAGMA foreign_key_check")?.exists([])? {
        return Err(Error::Schema);
    }
    Ok(Store::validate_schema(&read)?.0)
}

struct Temp(PathBuf);
impl Drop for Temp {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}
pub fn snapshot(source: &Path, destination: &Path) -> Result<i32> {
    let parent = private_parent(destination)?;
    absent(destination)?;
    no_sidecars(destination)?;
    let read = read_database(source)?;
    let mut random = [0; 16];
    entropy(&mut random)?;
    let path = parent.join(format!(".snapshot-{}", hex(&random)));
    let temp_name = path.to_str().ok_or(Error::InvalidInput)?;
    let file = storage(
        OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(&path),
    )?;
    let temp = Temp(path.clone());
    read.execute("VACUUM INTO ?1", [temp_name])?;
    let version = verify_snapshot(&temp.0)?;
    storage(file.sync_all())?;
    // A hard link publishes the already verified file atomically without replacing a DB.
    storage(fs::hard_link(&temp.0, destination))?;
    drop(temp);
    storage(storage(fs::File::open(parent))?.sync_all())?;
    Ok(version)
}

pub fn restore(source: &Path, destination: &Path) -> Result<i32> {
    verify_snapshot(source)?;
    snapshot(source, destination)
}

pub(super) fn before_migration(path: &Path, old: i32) -> Result<()> {
    let name = path.file_name().ok_or(Error::InvalidInput)?;
    let mut random = [0; 16];
    entropy(&mut random)?;
    let mut backup = OsString::from(name);
    backup.push(format!(
        ".before-v{old}-to-v{}-{}.sqlite",
        SCHEMA_VERSION,
        hex(&random)
    ));
    if snapshot(path, &path.with_file_name(backup))? != old {
        return Err(Error::Schema);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::accounts::tests::Temp;

    #[test]
    fn corrupt_snapshot_and_stale_sidecars_are_never_published_as_restore() {
        let _serial = TEST_SERIAL.lock().unwrap();
        let source = Temp::new();
        let db = source.0.join("live.sqlite");
        drop(Store::open(&db).unwrap());
        let target = Temp::new();
        let saved = target.0.join("saved.sqlite");
        assert_eq!(snapshot(&db, &saved), Ok(SCHEMA_VERSION));
        let restored = target.0.join("restore.sqlite");
        let wal = target.0.join("restore.sqlite-wal");
        fs::write(&wal, b"leftover").unwrap();
        assert_eq!(restore(&saved, &restored), Err(Error::Conflict));
        assert!(!restored.exists());
        assert_eq!(fs::read(wal).unwrap(), b"leftover");
        let bad = target.0.join("bad.sqlite");
        snapshot(&db, &bad).unwrap();
        let edit = Connection::open(&bad).unwrap();
        edit.pragma_update(None, "foreign_keys", false).unwrap();
        edit.execute("INSERT INTO account_recovery VALUES(1,zeroblob(32))", [])
            .unwrap();
        drop(edit);
        assert_eq!(verify_snapshot(&bad), Err(Error::Schema));
        let rejected = target.0.join("rejected.sqlite");
        assert_eq!(restore(&bad, &rejected), Err(Error::Schema));
        assert!(!rejected.exists());
    }
}
