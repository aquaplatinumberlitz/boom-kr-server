//! First-party player installation. No assets or native wire changes live here.
#![forbid(unsafe_code)]
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::HashSet,
    fs::{self, File, OpenOptions},
    io::{self, Read, Write},
    path::{Path, PathBuf},
};

#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Pin {
    pub path: String,
    pub bytes: u64,
    pub sha256: String,
}
#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Entry {
    pub path: String,
    pub payload: Pin,
}
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Pack {
    pub path: String,
    pub parts: Vec<Pin>,
    pub bytes: u64,
    pub sha256: String,
}
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Language {
    pub encoding: String,
    pub images: String,
    pub files: Vec<Entry>,
}
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    pub schema: String,
    pub packs: Vec<Pack>,
    pub common: Vec<Entry>,
    pub languages: std::collections::BTreeMap<String, Language>,
    pub limits: String,
}
fn invalid() -> io::Error {
    io::Error::other("player installation integrity or path check failed")
}
pub fn relative(value: &str) -> io::Result<PathBuf> {
    if value.is_empty() || value.contains(['\\', ':', '\0']) {
        return Err(invalid());
    }
    for part in value.split('/') {
        let stem = part.split('.').next().unwrap_or("").to_ascii_uppercase();
        if part.is_empty()
            || matches!(part, "." | "..")
            || part.ends_with(['.', ' '])
            || matches!(stem.as_str(), "CON" | "PRN" | "AUX" | "NUL")
            || (stem.len() == 4
                && (stem.starts_with("COM") || stem.starts_with("LPT"))
                && matches!(stem.as_bytes()[3], b'1'..=b'9'))
        {
            return Err(invalid());
        }
    }
    Ok(PathBuf::from(value))
}
fn reparse(metadata: &fs::Metadata) -> bool {
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        // FILE_ATTRIBUTE_REPARSE_POINT also covers directory junctions.
        metadata.file_attributes() & 0x400 != 0
    }
    #[cfg(not(windows))]
    metadata.file_type().is_symlink()
}

fn managed_file(root: &Path, name: &str, missing_allowed: bool) -> io::Result<PathBuf> {
    let metadata = fs::symlink_metadata(root)?;
    if !metadata.is_dir() || reparse(&metadata) {
        return Err(invalid());
    }
    let name = relative(name)?;
    let mut path = root.to_path_buf();
    let mut components = name.components().peekable();
    while let Some(component) = components.next() {
        path.push(component);
        let leaf = components.peek().is_none();
        match fs::symlink_metadata(&path) {
            Ok(meta) if !reparse(&meta) && (if leaf { meta.is_file() } else { meta.is_dir() }) => {}
            Err(e) if leaf && missing_allowed && e.kind() == io::ErrorKind::NotFound => (),
            _ => return Err(invalid()),
        }
    }
    Ok(path)
}

fn ordinary(root: &Path, name: &str) -> io::Result<PathBuf> {
    managed_file(root, name, false)
}

fn runtime_log(name: &str) -> bool {
    name.rsplit('/')
        .next()
        .is_some_and(|name| name.eq_ignore_ascii_case("calog.log"))
}

fn runtime_file(name: &str) -> bool {
    runtime_log(name)
        || Path::new(name)
            .extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| ext.eq_ignore_ascii_case("ssd") || ext.eq_ignore_ascii_case("ini"))
}

fn make_writable(path: &Path, meta: &fs::Metadata) -> io::Result<()> {
    let mut permissions = meta.permissions();
    #[cfg(windows)]
    {
        if !permissions.readonly() {
            return Ok(());
        }
        permissions.set_readonly(false);
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let wanted = if meta.is_dir() { 0o700 } else { 0o200 };
        if permissions.mode() & wanted == wanted {
            return Ok(());
        }
        permissions.set_mode(permissions.mode() | wanted);
    }
    fs::set_permissions(path, permissions)
}

fn normalize_game_tree(path: &Path) -> io::Result<()> {
    let meta = fs::symlink_metadata(path)?;
    if reparse(&meta) || !(meta.is_dir() || meta.is_file()) {
        return Err(invalid());
    }
    make_writable(path, &meta)?;
    if meta.is_dir() {
        for entry in fs::read_dir(path)? {
            normalize_game_tree(&entry?.path())?;
        }
    }
    Ok(())
}

pub fn sha256(path: &Path) -> io::Result<String> {
    let mut hash = Sha256::new();
    let mut file = File::open(path)?;
    let mut buffer = [0; 65536];
    loop {
        let count = file.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        hash.update(&buffer[..count]);
    }
    Ok(hash
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect())
}
fn checked(root: &Path, pin: &Pin) -> io::Result<PathBuf> {
    let path = ordinary(root, &pin.path)?;
    if fs::metadata(&path)?.len() != pin.bytes || sha256(&path)? != pin.sha256 {
        return Err(invalid());
    }
    Ok(path)
}
pub fn read_manifest(bundle: &Path, expected: &str) -> io::Result<Manifest> {
    let file = ordinary(bundle, "manifest.json")?;
    if expected.len() != 64
        || fs::metadata(&file)?.len() > 4 * 1024 * 1024
        || sha256(&file)? != expected
    {
        return Err(invalid());
    }
    let manifest: Manifest = serde_json::from_slice(&fs::read(file)?).map_err(|_| invalid())?;
    if manifest.schema != "kre-player-package-v1"
        || manifest
            .languages
            .keys()
            .map(String::as_str)
            .collect::<Vec<_>>()
            != ["en", "ko", "vi"]
        || manifest.packs.is_empty()
        || manifest.packs.len() > 128
    {
        return Err(invalid());
    }
    for language in manifest.languages.values() {
        if language.encoding != "949x" || !language.files.iter().any(|entry| entry.path == "CA.exe")
        {
            return Err(invalid());
        }
        let mut paths = HashSet::new();
        for name in manifest
            .common
            .iter()
            .map(|e| &e.path)
            .chain(manifest.packs.iter().map(|p| &p.path))
            .chain(language.files.iter().map(|e| &e.path))
        {
            relative(name)?;
            if !paths.insert(name.to_lowercase()) {
                return Err(invalid());
            }
        }
    }
    Ok(manifest)
}
fn destination(root: &Path, name: &str) -> io::Result<PathBuf> {
    let target = root.join(relative(name)?);
    fs::create_dir_all(target.parent().ok_or_else(invalid)?)?;
    Ok(target)
}
pub fn install(bundle: &Path, expected: &str, target: &Path) -> io::Result<()> {
    let manifest = read_manifest(bundle, expected)?;
    // A failed or previous installation is never silently overwritten.
    fs::create_dir(target)?;
    let shared = target.join("shared");
    fs::create_dir(&shared)?;
    for pack in &manifest.packs {
        let path = destination(&shared, &pack.path)?;
        let mut output = File::create_new(&path)?;
        for part in &pack.parts {
            io::copy(&mut File::open(checked(bundle, part)?)?, &mut output)?;
        }
        output.sync_all()?;
        drop(output);
        if fs::metadata(&path)?.len() != pack.bytes || sha256(&path)? != pack.sha256 {
            return Err(invalid());
        }
    }
    for entry in &manifest.common {
        let path = destination(&shared, &entry.path)?;
        fs::copy(checked(bundle, &entry.payload)?, &path)?;
        checked(
            &shared,
            &Pin {
                path: entry.path.clone(),
                ..entry.payload.clone()
            },
        )?;
    }
    for (id, language) in &manifest.languages {
        let game = target.join(id);
        fs::create_dir(&game)?;
        for pack in &manifest.packs {
            fs::hard_link(
                shared.join(relative(&pack.path)?),
                destination(&game, &pack.path)?,
            )?;
        }
        for entry in &manifest.common {
            let source = shared.join(relative(&entry.path)?);
            let output = destination(&game, &entry.path)?;
            if runtime_file(&entry.path) {
                fs::copy(source, output)?;
            } else {
                fs::hard_link(source, output)?;
            }
        }
        for entry in &language.files {
            fs::copy(
                checked(bundle, &entry.payload)?,
                destination(&game, &entry.path)?,
            )?;
            checked(
                &game,
                &Pin {
                    path: entry.path.clone(),
                    ..entry.payload.clone()
                },
            )?;
        }
        normalize_game_tree(&game)?;
    }
    // Only a fully checked installation gets a completion marker.
    let mut receipt = File::create_new(target.join("installation.sha256"))?;
    writeln!(receipt, "{expected}")?;
    receipt.sync_all()
}
fn check_receipt(target: &Path, expected: &str) -> io::Result<()> {
    if fs::read_to_string(ordinary(target, "installation.sha256")?)?.trim() != expected {
        return Err(invalid());
    }
    Ok(())
}

fn verify_entry(bundle: &Path, game: &Path, entry: &Entry) -> io::Result<()> {
    if runtime_file(&entry.path) {
        // A client may update or remove these files. Keep the trusted inputs
        // pinned; the launch preparation restores them without trusting edits.
        checked(bundle, &entry.payload)?;
        managed_file(game, &entry.path, true)?;
    } else {
        checked(
            game,
            &Pin {
                path: entry.path.clone(),
                bytes: entry.payload.bytes,
                sha256: entry.payload.sha256.clone(),
            },
        )?;
    }
    Ok(())
}

fn verify_tree(
    bundle: &Path,
    target: &Path,
    language: &str,
    manifest: &Manifest,
) -> io::Result<()> {
    let selected = manifest.languages.get(language).ok_or_else(invalid)?;
    let game = target.join(language);
    for pack in &manifest.packs {
        checked(
            &game,
            &Pin {
                path: pack.path.clone(),
                bytes: pack.bytes,
                sha256: pack.sha256.clone(),
            },
        )?;
    }
    for entry in manifest.common.iter().chain(&selected.files) {
        verify_entry(bundle, &game, entry)?;
    }
    Ok(())
}

pub fn verify_install(bundle: &Path, expected: &str, target: &Path) -> io::Result<()> {
    let manifest = read_manifest(bundle, expected)?;
    check_receipt(target, expected)?;
    for language in manifest.languages.keys() {
        verify_tree(bundle, target, language, &manifest)?;
    }
    Ok(())
}

pub fn verify_language(
    bundle: &Path,
    expected: &str,
    target: &Path,
    language: &str,
) -> io::Result<()> {
    let manifest = read_manifest(bundle, expected)?;
    check_receipt(target, expected)?;
    verify_tree(bundle, target, language, &manifest)
}

#[derive(Clone, Copy, Debug)]
pub enum PreparationError {
    Integrity,
    WritableFiles,
}

/// Restore mutable settings and cache to verified inputs before starting the
/// client, while retaining the client's log. Change attributes without
/// replacing existing files or altering ACLs.
pub fn prepare_language(
    bundle: &Path,
    expected: &str,
    target: &Path,
    language: &str,
) -> Result<(), PreparationError> {
    let manifest = read_manifest(bundle, expected).map_err(|_| PreparationError::Integrity)?;
    check_receipt(target, expected).map_err(|_| PreparationError::Integrity)?;
    verify_tree(bundle, target, language, &manifest).map_err(|_| PreparationError::Integrity)?;
    let selected = &manifest.languages[language];
    let game = target.join(language);
    for entry in manifest.common.iter().chain(&selected.files) {
        if runtime_file(&entry.path) {
            prepare_runtime_file(bundle, &game, entry)
                .map_err(|_| PreparationError::WritableFiles)?;
        }
    }
    Ok(())
}

fn prepare_runtime_file(bundle: &Path, game: &Path, entry: &Entry) -> io::Result<()> {
    let output = managed_file(game, &entry.path, true)?;
    let unchanged = match fs::symlink_metadata(&output) {
        Ok(metadata) => {
            make_writable(&output, &metadata)?;
            runtime_log(&entry.path)
                || (metadata.len() == entry.payload.bytes
                    && sha256(&output)? == entry.payload.sha256)
        }
        Err(e) if e.kind() == io::ErrorKind::NotFound => false,
        Err(e) => return Err(e),
    };
    let mut open = OpenOptions::new();
    open.write(true);
    if unchanged {
        open.open(output)?;
    } else {
        let source = checked(bundle, &entry.payload)?;
        let mut writer = open.create(true).truncate(true).open(output)?;
        io::copy(&mut File::open(source)?, &mut writer)?;
    }
    Ok(())
}
