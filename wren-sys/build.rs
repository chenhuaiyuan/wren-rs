use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

// #[allow(dead_code)]
// fn make_debug(dir: &Path) {
//     let status = Command::new("make").current_dir(dir).arg("debug").status();
//     assert!(status.unwrap().success());
//     println!("cargo:rustc-link-lib=static=wrend");
// }

fn make(dir: &Path) {
    let status = Command::new("make").current_dir(dir).arg("wren").status();
    assert!(status.unwrap().success());
    println!("cargo:rustc-link-lib=static=wren");
}

fn main() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let manifest_path = Path::new(&manifest_dir);
    let wren_make_dir: PathBuf;
    if cfg!(target_os = "macos") {
        wren_make_dir = manifest_path.join("wren/projects/make.mac");
    } else if cfg!(target_os = "linux") {
        wren_make_dir = manifest_path.join("wren/projects/make");
    } else {
        wren_make_dir = manifest_path.join("wren/projects/make");
    }
    let wren_lib_dir = manifest_path.join("wren/lib");

    make(&wren_make_dir);

    println!("cargo:rustc-link-search=native={}", wren_lib_dir.display());
}
