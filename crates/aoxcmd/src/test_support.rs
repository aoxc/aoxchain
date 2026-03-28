// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use crate::data_home::ScopedHomeOverride;
use std::{
    env, fs,
    path::{Path, PathBuf},
    process,
    sync::{Mutex, MutexGuard, OnceLock},
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

/// Returns the shared process-wide lock used by tests that install an AOXC home override.
///
/// Security and determinism rationale:
/// - The AOXC home override is process-local shared state.
/// - Any test that changes it must serialize access across the crate.
/// - Poisoned state is explicitly tolerated so one panic does not cascade into
///   unrelated failures in later tests.
pub(crate) fn aoxc_home_test_lock() -> MutexGuard<'static, ()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// RAII guard that installs and restores the in-process AOXC home override.
///
/// Usage contract:
/// - Acquire `aoxc_home_test_lock()` first.
/// - Install this guard for the duration of the test body.
/// - Allow automatic restoration on drop.
///
/// Rust 2024 compatibility:
/// - This guard does not mutate process environment variables.
/// - It relies on the in-process scoped home override provided by `data_home`.
pub(crate) struct AoxcHomeGuard {
    _override_guard: ScopedHomeOverride,
}

impl AoxcHomeGuard {
    /// Installs a temporary AOXC home override for the current process.
    pub(crate) fn install(root: &Path) -> Self {
        Self {
            _override_guard: ScopedHomeOverride::install(root),
        }
    }
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
