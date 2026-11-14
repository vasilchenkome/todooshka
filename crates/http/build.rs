use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR is set by Cargo");
    let dist = PathBuf::from(manifest_dir).join("../../frontend/dist");

    if !dist.exists() {
        fs::create_dir_all(&dist).expect("create frontend/dist for rust-embed");
    }

    println!("cargo:rerun-if-changed={}", dist.display());
}
