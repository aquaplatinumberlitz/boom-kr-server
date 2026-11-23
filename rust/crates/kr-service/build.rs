//! Embed the brand icon and the application manifest into `boom-server.exe`.
//! Non-Windows targets (the Linux service, CI clippy/tests) do nothing here.
use std::{env, path::PathBuf, process::Command};

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    if env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }
    let crate_dir = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").expect("cargo sets it"));
    let resources = crate_dir.join("windows");
    let brand = crate_dir.join("../../../assets/brand");
    let script = resources.join("boom-server.rc");
    for path in [
        script.clone(),
        resources.join("boom-server.manifest"),
        brand.join("icon.ico"),
    ] {
        println!("cargo:rerun-if-changed={}", path.display());
    }
    println!("cargo:rerun-if-env-changed=WINDRES");
    if env::var("CARGO_CFG_TARGET_ENV").as_deref() != Ok("gnu") {
        println!("cargo:warning=BoomServer resources are only built for the windows-gnu target");
        return;
    }
    let windres = env::var("WINDRES").unwrap_or_else(|_| "x86_64-w64-mingw32-windres".into());
    let out =
        PathBuf::from(env::var_os("OUT_DIR").expect("cargo sets it")).join("boom-server-res.o");
    let status = Command::new(&windres)
        .arg("--input-format=rc")
        .arg("--output-format=coff")
        .arg("--codepage=65001")
        .arg("--include-dir")
        .arg(&brand)
        .arg("--include-dir")
        .arg(&resources)
        .arg("--input")
        .arg(&script)
        .arg("--output")
        .arg(&out)
        .status()
        .unwrap_or_else(|error| panic!("cannot run {windres} for BoomServer resources: {error}"));
    assert!(
        status.success(),
        "{windres} failed for {}",
        script.display()
    );
    // Only the desktop host gets the icon, visual styles and DPI manifest.
    println!("cargo:rustc-link-arg-bin=boom-server={}", out.display());
}
