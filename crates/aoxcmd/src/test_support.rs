// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use crate::data_home::ScopedHomeOverride;
use std::{
    env, fs, io,
    path::{Path, PathBuf},
    process,
    sync::{Mutex, MutexGuard, OnceLock},
    time::{SystemTime, UNIX_EPOCH},
};

/// Returns the validated user home directory.
///
/// Security rationale:
/// - Test infrastructure must derive all filesystem paths from a trusted,
///   absolute home directory boundary.
/// - An unset or relative HOME value is rejected because it weakens path
///   determinism and may cause writes outside the intended namespace.
///
/// Failure mode:
/// - This helper fails fast because tests must not proceed with ambiguous or
///   unsafe filesystem roots.
fn validated_home_dir() -> PathBuf {
    let home = env::var_os("HOME").unwrap_or_else(|| {
        panic!("HOME must be set for AOXC test isolation");
    });

    let path = PathBuf::from(home);

    assert!(
        !path.as_os_str().is_empty(),
        "HOME must not be empty for AOXC test isolation"
    );
    assert!(
        path.is_absolute(),
        "HOME must be an absolute path for AOXC test isolation"
    );

    path
}

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
    validated_home_dir().join(".AOXCData")
}

/// Returns the canonical AOXC test root beneath the AOXC-owned namespace.
///
/// Test path policy:
/// - Tests must never write into the production default AOXC home.
/// - Disposable test homes are grouped under `$HOME/.AOXCData/.test/`.
fn canonical_test_root() -> PathBuf {
    canonical_data_root().join(".test")
}

/// Sanitizes a test label for safe filesystem embedding.
///
/// Security rationale:
/// - Labels may appear in directory names and therefore must not contain
///   path separators or uncontrolled characters.
/// - The output is intentionally restricted to a conservative ASCII subset
///   suitable for deterministic test artifacts.
///
/// Policy:
/// - Allowed characters: `[A-Za-z0-9_-]`
/// - Any other character is replaced with `_`
/// - Output length is capped to avoid pathological path growth
/// - Empty results are normalized to `test`
fn sanitize_label(label: &str) -> String {
    let mut out = String::with_capacity(label.len().min(64));

    for ch in label.chars() {
        if out.len() >= 64 {
            break;
        }

        if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
            out.push(ch);
        } else {
            out.push('_');
        }
    }

    if out.is_empty() {
        "test".to_owned()
    } else {
        out
    }
}

/// Returns a unique isolated AOXC test home beneath the canonical AOXC test root.
///
/// Uniqueness strategy:
/// - Sanitized label
/// - Current process identifier
/// - Current UNIX timestamp in nanoseconds
///
/// Security and determinism rationale:
/// - Each test instance receives its own dedicated root.
/// - The generated path is always anchored beneath the AOXC-owned `.test`
///   namespace and never derived from untrusted absolute input.
fn unique_test_home(label: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time must be after the UNIX epoch")
        .as_nanos();

    let label = sanitize_label(label);

    canonical_test_root().join(format!(
        "aoxcmd-{label}-pid{}-{nanos}",
        process::id()
    ))
}

/// Returns the shared process-wide lock used by tests that install an AOXC home override.
///
/// Security and determinism rationale:
/// - The AOXC home override is process-local shared state.
/// - Any test mutating this state must serialize access across the crate.
/// - Poisoned state is explicitly tolerated so a single panic does not create
///   cascading and unrelated failures in subsequent tests.
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
/// Security rationale:
/// - The constructor requires a lock reference so callers cannot accidentally
///   mutate process-wide override state without first serializing access.
pub(crate) struct AoxcHomeGuard {
    _override_guard: ScopedHomeOverride,
}

impl AoxcHomeGuard {
    /// Installs a temporary AOXC home override for the current process.
    ///
    /// Contract enforcement:
    /// - A live lock guard reference is required to make the serialization
    ///   requirement explicit at the call site.
    pub(crate) fn install(_lock: &MutexGuard<'static, ()>, root: &Path) -> Self {
        assert!(
            root.is_absolute(),
            "AOXC test home override root must be absolute"
        );

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
    /// Creates a new isolated AOXC test home and materializes it on disk.
    ///
    /// Security rationale:
    /// - The directory is created eagerly so downstream code operates on a real,
    ///   known-good root rather than an unmaterialized path assumption.
    pub(crate) fn new(label: &str) -> io::Result<Self> {
        let test_root = canonical_test_root();
        fs::create_dir_all(&test_root)?;

        let path = unique_test_home(label);
        fs::create_dir_all(&path)?;

        Ok(Self { path })
    }

    /// Returns the effective AOXC home associated with this test helper.
    pub(crate) fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TestHome {
    fn drop(&mut self) {
        let test_root = canonical_test_root();

        // Refuse cleanup if the path is not inside the expected AOXC test namespace.
        if !self.path.starts_with(&test_root) {
            return;
        }

        // Refuse cleanup if the final directory name does not follow the expected
        // internal naming convention. This provides an additional defensive check
        // against accidental path substitution.
        let Some(name) = self.path.file_name().and_then(|n| n.to_str()) else {
            return;
        };

        if !name.starts_with("aoxcmd-") {
            return;
        }

        // If the path no longer exists, cleanup is already complete.
        let Ok(metadata) = fs::symlink_metadata(&self.path) else {
            return;
        };

        // Refuse to recursively remove a top-level symlink. This avoids deleting
        // through an unexpected redirection target if the path has been tampered with.
        if metadata.file_type().is_symlink() {
            return;
        }

        let _ = fs::remove_dir_all(&self.path);
    }
}
