// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use std::{
    env, fs,
    path::{Path, PathBuf},
    process,
    time::{SystemTime, UNIX_EPOCH},
};

/// Returns the canonical AOXC production root for the current user.
///
/// Production path policy:
/// - `$HOME/.AOXCData`
fn production_home_root() -> PathBuf {
    let home = env::var("HOME").expect("HOME must be set");
    PathBuf::from(home).join(".AOXCData")
}

/// Returns a unique isolated test root under the canonical AOXC namespace.
///
/// Test path policy:
/// - Tests do not write into the production AOXC root directly.
/// - Tests are grouped under `$HOME/.AOXCData/.test/`.
/// - Each test instance receives its own disposable subdirectory.
fn unique_test_home(label: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after unix epoch")
        .as_nanos();

    production_home_root()
        .join(".test")
        .join(format!("aoxcmd-{label}-pid{}-{nanos}", process::id()))
}

/// Test helper that provides an isolated AOXC root under the user's AOXC test namespace.
///
/// Design objectives:
/// - Keep all AOXC-related test artifacts under a single hidden user-owned root.
/// - Prevent accidental mixing of test artifacts with production AOXC state.
/// - Ensure each test receives a unique disposable root directory.
pub(crate) struct TestHome {
    path: PathBuf,
}

impl TestHome {
    /// Creates a new isolated AOXC test root.
    pub(crate) fn new(label: &str) -> Self {
        let path = unique_test_home(label);
        Self { path }
    }

    /// Returns the isolated AOXC root associated with this test helper.
    pub(crate) fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TestHome {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}
