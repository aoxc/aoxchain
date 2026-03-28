// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use std::{
    env, fs,
    path::{Path, PathBuf},
    process,
    time::{SystemTime, UNIX_EPOCH},
};

/// Returns the canonical AOXC data root for the current user.
///
/// Canonical data root policy:
/// - `$HOME/.AOXCData`
///
/// Important distinction:
/// - This is the top-level AOXC-owned namespace.
/// - It is not necessarily the effective runtime home used by commands.
/// - Test homes are created beneath this root under `.test/`.
fn canonical_data_root() -> PathBuf {
    let home = env::var("HOME").expect("HOME must be set");
    PathBuf::from(home).join(".AOXCData")
}

/// Returns a unique isolated AOXC test home beneath the canonical AOXC data root.
///
/// Test path policy:
/// - Tests must not write into the production default AOXC home.
/// - Disposable test homes are grouped under `$HOME/.AOXCData/.test/`.
/// - Each test instance receives its own unique effective home root.
fn unique_test_home(label: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after unix epoch")
        .as_nanos();

    canonical_data_root()
        .join(".test")
        .join(format!("aoxcmd-{label}-pid{}-{nanos}", process::id()))
}

/// Test helper that provides an isolated AOXC home for a single test execution.
///
/// Design objectives:
/// - Keep all AOXC-related test artifacts under the canonical AOXC namespace.
/// - Prevent accidental mixing of disposable test state with production state.
/// - Ensure each test receives a unique and removable effective home directory.
pub(crate) struct TestHome {
    path: PathBuf,
}

impl TestHome {
    /// Creates a new isolated AOXC test home.
    pub(crate) fn new(label: &str) -> Self {
        let path = unique_test_home(label);
        Self { path }
    }

    /// Returns the effective AOXC home associated with this test helper.
    pub(crate) fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TestHome {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}
