use std::{env, fs, path::PathBuf, process::Command};

fn main() {
    println!("cargo:rerun-if-changed=../../../assets/brand/icon.ico");
    if env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }
    let out = PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR"));
    let icon = PathBuf::from("../../../assets/brand/icon.ico")
        .canonicalize()
        .expect("LANBoom icon");
    let rc = out.join("lanboom.rc");
    let object = out.join("lanboom-icon.o");
    fs::write(
        &rc,
        format!(
            "1 ICON \"{}\"\n",
            icon.display().to_string().replace('\\', "/")
        ),
    )
    .expect("LANBoom resource script");
    assert!(
        Command::new("x86_64-w64-mingw32-windres")
            .args(["-O", "coff", "-i"])
            .arg(&rc)
            .arg("-o")
            .arg(&object)
            .status()
            .expect("run windres")
            .success(),
        "compile LANBoom icon"
    );
    println!("cargo:rustc-link-arg-bin=kr-player={}", object.display());
}
