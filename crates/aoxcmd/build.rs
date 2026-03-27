// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use sha2::{Digest, Sha256};
use std::{env, fs, path::Path};

fn main() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR must be present");
    let genesis_path = Path::new(&manifest_dir)
        .join("AOXC_DATA")
        .join("identity")
        .join("genesis.json");
    println!("cargo:rerun-if-changed={}", genesis_path.display());

    let digest = match fs::read(&genesis_path) {
        Ok(bytes) => {
            let mut hasher = Sha256::new();
            hasher.update(bytes);
            hex::encode(hasher.finalize())
        }
        Err(_) => String::from("unavailable"),
    };

    println!("cargo:rustc-env=AOXC_BUILD_GENESIS_SHA256={digest}");
}
