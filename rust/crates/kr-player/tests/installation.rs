use kr_player::{Entry, Language, Manifest, Pack, Pin};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};
static NEXT: AtomicU64 = AtomicU64::new(0);
struct Directory(PathBuf);
impl Directory {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!(
            "kr-player-test-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&root).unwrap();
        Self(root)
    }
}
impl Drop for Directory {
    fn drop(&mut self) {
        // Windows read-only shared packs need their attribute reset before
        // removing this test's own temporary fixture only.
        fn writable(path: &Path) {
            for entry in fs::read_dir(path).unwrap().flatten() {
                let path = entry.path();
                if path.is_dir() {
                    writable(&path);
                } else if let Ok(metadata) = fs::metadata(&path) {
                    let mut permissions = metadata.permissions();
                    #[cfg(windows)]
                    permissions.set_readonly(false);
                    #[cfg(unix)]
                    {
                        use std::os::unix::fs::PermissionsExt;
                        permissions.set_mode(0o600);
                    }
                    let _ = fs::set_permissions(&path, permissions);
                }
            }
        }
        writable(&self.0);
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn payload(root: &Path, name: &str, bytes: &[u8]) -> Pin {
    fs::write(root.join(name), bytes).unwrap();
    Pin {
        path: name.into(),
        bytes: bytes.len() as u64,
        sha256: kr_player::sha256(&root.join(name)).unwrap(),
    }
}
fn fixture(root: &Path) -> String {
    let base = payload(root, "base", b"KR");
    let vi = payload(root, "vi-tail", b"VI");
    let en = payload(root, "en-tail", b"EN");
    let full = payload(root, "expected-pack", b"KRVIEN");
    let common = payload(root, "common", b"unchanged library");
    let mut languages = BTreeMap::new();
    for id in ["vi", "en", "ko"] {
        let exe = payload(
            root,
            &format!("{id}-exe"),
            format!("patched949x-{id}").as_bytes(),
        );
        let idx = payload(root, &format!("{id}-idx"), format!("index-{id}").as_bytes());
        languages.insert(
            id.into(),
            Language {
                encoding: "949x".into(),
                images: id.into(),
                files: vec![
                    Entry {
                        path: "CA.exe".into(),
                        payload: exe,
                    },
                    Entry {
                        path: "fx.idx".into(),
                        payload: idx,
                    },
                ],
            },
        );
    }
    let manifest = Manifest {
        schema: "kre-player-package-v1".into(),
        packs: vec![Pack {
            path: "fx.idd".into(),
            parts: vec![base, vi, en],
            bytes: full.bytes,
            sha256: full.sha256,
        }],
        common: vec![Entry {
            path: "support.dll".into(),
            payload: common,
        }],
        languages,
        limits: "Synthetic integrity regression, not native evidence".into(),
    };
    fs::write(
        root.join("manifest.json"),
        serde_json::to_vec(&manifest).unwrap(),
    )
    .unwrap();
    kr_player::sha256(&root.join("manifest.json")).unwrap()
}
#[test]
fn assembled_pack_is_shared_but_language_indices_and_executables_are_distinct() {
    let dir = Directory::new();
    let hash = fixture(&dir.0);
    let target = dir.0.join("install");
    kr_player::install(&dir.0, &hash, &target).unwrap();
    kr_player::verify_install(&dir.0, &hash, &target).unwrap();
    for language in ["vi", "en", "ko"] {
        assert_eq!(
            fs::read(target.join(language).join("fx.idd")).unwrap(),
            b"KRVIEN"
        );
        assert_eq!(
            fs::read(target.join(language).join("CA.exe")).unwrap(),
            format!("patched949x-{language}").as_bytes()
        );
        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;
            assert_eq!(
                fs::metadata(target.join(language).join("fx.idd"))
                    .unwrap()
                    .ino(),
                fs::metadata(target.join("shared/fx.idd")).unwrap().ino()
            );
        }
    }
    assert!(kr_player::install(&dir.0, &hash, &target).is_err());
}
#[test]
fn corrupt_tail_or_wrong_manifest_cannot_complete_installation() {
    let dir = Directory::new();
    let hash = fixture(&dir.0);
    let target = dir.0.join("install");
    fs::write(dir.0.join("vi-tail"), b"NO").unwrap();
    assert!(kr_player::install(&dir.0, &hash, &target).is_err());
    assert!(!target.join("installation.sha256").exists());
    assert!(kr_player::verify_install(&dir.0, &hash, &target).is_err());
    assert!(kr_player::install(&dir.0, &"0".repeat(64), &dir.0.join("wrong")).is_err());
    assert!(!dir.0.join("wrong").exists());
}
#[test]
fn post_install_replacement_and_platform_unsafe_paths_are_rejected() {
    let dir = Directory::new();
    let hash = fixture(&dir.0);
    let target = dir.0.join("install");
    kr_player::install(&dir.0, &hash, &target).unwrap();
    fs::write(target.join("ko/CA.exe"), b"original-unpatched").unwrap();
    assert!(kr_player::verify_install(&dir.0, &hash, &target).is_err());
    assert!(kr_player::verify_language(&dir.0, &hash, &target, "ko").is_err());
    kr_player::verify_language(&dir.0, &hash, &target, "vi").unwrap();
    for name in [
        "../outside",
        "/absolute",
        "C:/drive",
        "a\\b",
        "fx.idd.",
        "CON",
        "NUL.exe",
        "a//b",
    ] {
        assert!(kr_player::relative(name).is_err(), "{name}");
    }
}
