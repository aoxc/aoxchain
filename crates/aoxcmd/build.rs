// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use sha2::{Digest, Sha256};
use std::{
    env, fs,
    io::ErrorKind,
    path::{Path, PathBuf},
};

/// Resolves the effective AOXC data root for build-time metadata derivation.
///
/// Resolution order:
/// 1. `AOXC_DATA_ROOT`, when present and non-empty
/// 2. `$HOME/.AOXCData`
///
/// This mirrors the canonical AOXC path contract used by the workspace
/// Makefile and operator tooling.
fn resolve_data_root() -> Result<PathBuf, String> {
    if let Ok(value) = env::var("AOXC_DATA_ROOT") {
        let trimmed = value.trim();
        if !trimmed.is_empty() {
            return Ok(PathBuf::from(trimmed));
        }
    }

    let home = env::var("HOME").map_err(|_| "HOME environment variable is not set".to_string())?;
    Ok(Path::new(&home).join(".AOXCData"))
}

/// Resolves the effective AOXC home used for build-time genesis fingerprinting.
///
/// Resolution order:
/// 1. `AOXC_HOME`, when present and non-empty
/// 2. `<AOXC_DATA_ROOT>/home/default`
///
/// Rationale:
/// - This aligns build metadata derivation with the canonical AOXC runtime
///   storage contract rather than a crate-relative source tree path.
/// - The result is deterministic for local builds while still allowing
///   explicit overrides in CI or operator environments.
fn resolve_home() -> Result<PathBuf, String> {
    if let Ok(value) = env::var("AOXC_HOME") {
        let trimmed = value.trim();
        if !trimmed.is_empty() {
            return Ok(PathBuf::from(trimmed));
        }
    }

    Ok(resolve_data_root()?.join("home").join("default"))
}

/// Computes the SHA-256 digest for the supplied file.
fn sha256_file(path: &Path) -> Result<String, String> {
    let bytes = fs::read(path).map_err(|error| {
        format!("Failed to read {}: {}", path.display(), error)
    })?;

    let mut hasher = Sha256::new();
    hasher.update(bytes);
    Ok(hex::encode(hasher.finalize()))
}

/// Attempts to derive the build-time genesis fingerprint.
///
/// Behavior:
/// - Returns `Ok(Some(digest))` when the genesis file is present and readable
/// - Returns `Ok(None)` when the file is absent, which is treated as a normal
///   non-fatal build condition
/// - Returns `Err(...)` only when an unexpected runtime error occurs
fn try_resolve_genesis_digest(path: &Path) -> Result<Option<String>, String> {
    match fs::metadata(path) {
        Ok(metadata) => {
            if !metadata.is_file() {
                return Err(format!(
                    "Resolved genesis path is not a regular file: {}",
                    path.display()
                ));
            }

            sha256_file(path).map(Some)
        }
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(None),
        Err(error) => Err(format!(
            "Failed to access {}: {}",
            path.display(),
            error
        )),
    }
}

fn main() {
    println!("cargo:rerun-if-env-changed=AOXC_HOME");
    println!("cargo:rerun-if-env-changed=AOXC_DATA_ROOT");
    println!("cargo:rerun-if-env-changed=HOME");

    let genesis_path = match resolve_home() {
        Ok(home) => home.join("identity").join("genesis.json"),
        Err(error) => {
            println!(
                "cargo:warning=AOXC build genesis path resolution failed: {}",
                error
            );
            println!("cargo:rustc-env=AOXC_BUILD_GENESIS_SHA256=unavailable");
            return;
        }
    };

    println!("cargo:rerun-if-changed={}", genesis_path.display());

    let digest = match try_resolve_genesis_digest(&genesis_path) {
        Ok(Some(digest)) => digest,
        Ok(None) => String::from("unavailable"),
        Err(error) => {
            println!(
                "cargo:warning=AOXC build genesis fingerprint resolution failed: {}",
                error
            );
            String::from("unavailable")
        }
    };

    println!("cargo:rustc-env=AOXC_BUILD_GENESIS_SHA256={}", digest);
}
