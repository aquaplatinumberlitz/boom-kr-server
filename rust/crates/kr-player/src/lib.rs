//! First-party player installation. No assets or native wire changes live here.
#![forbid(unsafe_code)]
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::HashSet,
    fs::{self, File},
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
fn ordinary(root: &Path, name: &str) -> io::Result<PathBuf> {
    let metadata = fs::symlink_metadata(root)?;
    if !metadata.is_dir() || metadata.file_type().is_symlink() {
        return Err(invalid());
    }
    let mut path = root.to_path_buf();
    for component in relative(name)?.components() {
        path.push(component);
        if fs::symlink_metadata(&path)?.file_type().is_symlink() {
            return Err(invalid());
        }
    }
    if !fs::symlink_metadata(&path)?.is_file() {
        return Err(invalid());
    }
    Ok(path)
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
        let mut permissions = fs::metadata(&path)?.permissions();
        permissions.set_readonly(true);
        fs::set_permissions(&path, permissions)?;
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
        let mut permissions = fs::metadata(&path)?.permissions();
        permissions.set_readonly(true);
        fs::set_permissions(path, permissions)?;
    }
    for (id, language) in &manifest.languages {
        let game = target.join(id);
        fs::create_dir(&game)?;
        for name in manifest
            .packs
            .iter()
            .map(|p| &p.path)
            .chain(manifest.common.iter().map(|e| &e.path))
        {
            // Packs and unchanged files share actual storage, not just hashes.
            fs::hard_link(shared.join(relative(name)?), destination(&game, name)?)?;
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
    }
    // Only a fully checked installation gets a completion marker.
    let mut receipt = File::create_new(target.join("installation.sha256"))?;
    writeln!(receipt, "{expected}")?;
    receipt.sync_all()
}
pub fn verify_install(bundle: &Path, expected: &str, target: &Path) -> io::Result<()> {
    let manifest = read_manifest(bundle, expected)?;
    if fs::read_to_string(ordinary(target, "installation.sha256")?)?.trim() != expected {
        return Err(invalid());
    }
    for (id, language) in manifest.languages {
        let game = target.join(id);
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
        for entry in manifest.common.iter().chain(&language.files) {
            checked(
                &game,
                &Pin {
                    path: entry.path.clone(),
                    bytes: entry.payload.bytes,
                    sha256: entry.payload.sha256.clone(),
                },
            )?;
        }
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
    if fs::read_to_string(ordinary(target, "installation.sha256")?)?.trim() != expected {
        return Err(invalid());
    }
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
        checked(
            &game,
            &Pin {
                path: entry.path.clone(),
                bytes: entry.payload.bytes,
                sha256: entry.payload.sha256.clone(),
            },
        )?;
    }
    Ok(())
}
