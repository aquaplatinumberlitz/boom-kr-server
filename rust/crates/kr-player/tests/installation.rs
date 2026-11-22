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
    let settings = payload(root, "settings", b"seed settings");
    let cache = payload(root, "cache", b"seed dynamic map");
    let log = payload(root, "native-log", b"seed log");
    for name in ["settings", "cache", "native-log", "common"] {
        let path = root.join(name);
        let mut mode = fs::metadata(&path).unwrap().permissions();
        mode.set_readonly(true);
        fs::set_permissions(path, mode).unwrap();
    }
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
                    Entry {
                        path: "FxDynMapInfo.ssd".into(),
                        payload: cache.clone(),
                    },
                    Entry {
                        path: "settings/game.INI".into(),
                        payload: settings.clone(),
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
        common: vec![
            Entry {
                path: "support.dll".into(),
                payload: common,
            },
            Entry {
                path: "CALog.log".into(),
                payload: log,
            },
        ],
        languages,
        limits: "Synthetic integrity regression, not native evidence".into(),
        launcher_art: None,
        guide_images: BTreeMap::new(),
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
fn package_with_pinned_guide_images_can_install() {
    let dir = Directory::new();
    fixture(&dir.0);
    let file = dir.0.join("manifest.json");
    let mut manifest: serde_json::Value =
        serde_json::from_slice(&fs::read(&file).unwrap()).unwrap();
    fs::create_dir(dir.0.join("player")).unwrap();
    let guide = payload(&dir.0, "player/o1-join-lan.png", b"test image");
    manifest["guide_images"] = serde_json::json!({"o1-join-lan.png": guide});
    fs::write(&file, serde_json::to_vec(&manifest).unwrap()).unwrap();
    let hash = kr_player::sha256(&file).unwrap();
    let target = dir.0.join("install");
    kr_player::install(&dir.0, &hash, &target).unwrap();
    kr_player::verify_install(&dir.0, &hash, &target).unwrap();
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

#[test]
fn installed_game_files_are_writable_and_runtime_changes_reseed_from_pinned_inputs() {
    let dir = Directory::new();
    let hash = fixture(&dir.0);
    let target = dir.0.join("install");
    kr_player::install(&dir.0, &hash, &target).unwrap();
    let mutable = ["FxDynMapInfo.ssd", "settings/game.INI", "CALog.log"];
    for lang in ["vi", "en", "ko"] {
        assert!(
            fs::read(target.join(lang).join("CALog.log"))
                .unwrap()
                .is_empty()
        );
        for name in mutable.into_iter().chain(["fx.idd", "support.dll"]) {
            let installed = target.join(lang).join(name);
            assert!(!fs::metadata(installed).unwrap().permissions().readonly());
        }
    }
    for name in mutable {
        let installed = target.join("ko").join(name);
        fs::write(&installed, b"native updated").unwrap();
        let mut mode = fs::metadata(&installed).unwrap().permissions();
        mode.set_readonly(true);
        fs::set_permissions(&installed, mode).unwrap();
    }
    // The game may delete a cache; the next launch must still be possible.
    fs::remove_file(target.join("ko/FxDynMapInfo.ssd"))
        .or_else(|_| {
            let f = target.join("ko/FxDynMapInfo.ssd");
            #[cfg(windows)]
            {
                let mut mode = fs::metadata(&f)?.permissions();
                mode.set_readonly(false);
                fs::set_permissions(&f, mode)?;
            }
            fs::remove_file(f)
        })
        .unwrap();
    kr_player::verify_install(&dir.0, &hash, &target).unwrap();
    kr_player::prepare_language(&dir.0, &hash, &target, "ko").unwrap();
    assert_eq!(
        fs::read(target.join("ko/FxDynMapInfo.ssd")).unwrap(),
        b"seed dynamic map"
    );
    assert_eq!(
        fs::read(target.join("ko/settings/game.INI")).unwrap(),
        b"seed settings"
    );
    assert_eq!(
        fs::read(target.join("ko/CALog.log")).unwrap(),
        b"native updated"
    );
    for name in mutable {
        fs::write(target.join("ko").join(name), b"second native update").unwrap();
    }
    fs::remove_file(target.join("en/CALog.log")).unwrap();
    kr_player::prepare_language(&dir.0, &hash, &target, "en").unwrap();
    assert!(fs::read(target.join("en/CALog.log")).unwrap().is_empty());
    assert_eq!(fs::read(dir.0.join("native-log")).unwrap(), b"seed log");
    let error = kr_player::prepare_language(&dir.0, &hash, &target, "wrong").unwrap_err();
    assert!(matches!(error, kr_player::PreparationError::Integrity));
}

#[test]
fn decorative_art_requires_its_pin_but_can_be_missing_without_blocking_install() {
    let dir = Directory::new();
    let root = &dir.0;
    let old_hash = fixture(root);
    assert!(kr_player::launcher_art(root, &old_hash).unwrap().is_none());

    let mut manifest = kr_player::read_manifest(root, &old_hash).unwrap();
    manifest.launcher_art = Some(payload(root, "backdrop.jpg", b"pinned artwork"));
    let path = root.join("manifest.json");
    fs::write(&path, serde_json::to_vec(&manifest).unwrap()).unwrap();
    let hash = kr_player::sha256(&path).unwrap();
    assert_eq!(
        kr_player::launcher_art(root, &hash).unwrap().unwrap(),
        b"pinned artwork"
    );
    fs::write(root.join("backdrop.jpg"), b"wrong! artwork").unwrap();
    assert!(kr_player::launcher_art(root, &hash).is_err());
    fs::remove_file(root.join("backdrop.jpg")).unwrap();
    assert!(kr_player::launcher_art(root, &hash).is_err());

    let target = root.join("install");
    kr_player::install(root, &hash, &target).unwrap();
    kr_player::verify_install(root, &hash, &target).unwrap();
}

#[test]
fn repair_preserves_old_tree_and_refuses_unknown_or_running_installations() {
    let dir = Directory::new();
    let root = &dir.0;
    let hash = fixture(root);
    let game = root.join("game");
    assert!(!kr_player::maintain_install(root, &hash, &game, false).unwrap());
    let exe = game.join("vi/CA.exe");
    fs::write(&exe, b"damaged").unwrap();
    let log = game.join("player-note.txt");
    fs::write(&log, b"keep me").unwrap();
    let lock = kr_player::lock_installation(&game).unwrap();
    assert!(kr_player::maintain_install(root, &hash, &game, true).is_err());
    drop(lock);
    assert!(kr_player::maintain_install(root, &hash, &game, false).is_err());
    assert!(kr_player::maintain_install(root, &hash, &game, true).unwrap());
    kr_player::verify_install(root, &hash, &game).unwrap();
    let previous = fs::read_dir(root)
        .unwrap()
        .flatten()
        .map(|entry| entry.path().join("previous"))
        .find(|p| p.is_dir())
        .unwrap();
    assert_eq!(
        fs::read(previous.join("player-note.txt")).unwrap(),
        b"keep me"
    );
    let unrelated = root.join("unrelated");
    fs::create_dir(&unrelated).unwrap();
    assert!(kr_player::maintain_install(root, &hash, &unrelated, true).is_err());
    assert!(unrelated.is_dir());
}
