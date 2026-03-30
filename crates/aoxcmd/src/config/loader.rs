// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use crate::{
    config::settings::Settings,
    data_home::{read_file, resolve_home, write_file},
    error::{AppError, ErrorCode},
};
use std::path::PathBuf;

/// Returns the canonical AOXC CLI settings path.
///
/// Canonical path policy:
/// - The effective AOXC settings document is stored at:
///   `<AOXC_HOME>/config/settings.json`.
///
/// Operational rationale:
/// - The settings document is treated as the operator-facing runtime
///   configuration surface.
/// - Path resolution is always derived from the effective AOXC home so that
///   command-scoped home overrides and isolated test homes behave consistently.
pub fn settings_path() -> Result<PathBuf, AppError> {
    Ok(resolve_home()?.join("config").join("settings.json"))
}

/// Creates and persists the canonical default AOXC settings document.
///
/// Defaulting policy:
/// - The safe default profile is `validation`, as defined by
///   `Settings::default_for(...)`.
///
/// Persistence contract:
/// - The generated document is validated before persistence.
/// - The validated document is written to the canonical AOXC settings path.
pub fn init_default() -> Result<Settings, AppError> {
    let home = resolve_home()?;
    let settings = Settings::default_for(home.display().to_string());
    persist(&settings)?;
    Ok(settings)
}

/// Creates and persists profile-specific AOXC settings.
///
/// Intended usage:
/// - Bootstrap flows that already resolved the intended canonical profile name.
/// - Profile-aware initialization paths where the caller wants deterministic
///   configuration materialization for a specific environment.
///
/// Validation policy:
/// - Invalid profile requests are converted into stable AOXC configuration
///   errors rather than leaking lower-level string failures.
pub fn init_for_profile(profile: &str) -> Result<Settings, AppError> {
    let home = resolve_home()?;
    let settings = Settings::default_for_profile(home.display().to_string(), profile)
        .map_err(|error| AppError::new(ErrorCode::ConfigInvalid, error))?;

    persist(&settings)?;
    Ok(settings)
}

/// Loads and validates the AOXC settings document from disk.
///
/// Error behavior:
/// - Missing settings are normalized into `ConfigMissing`.
/// - Invalid JSON is normalized into `ConfigInvalid`.
/// - Semantically invalid settings are rejected after deserialization.
///
/// Safety rationale:
/// - The loader never returns partially parsed or unvalidated configuration.
pub fn load() -> Result<Settings, AppError> {
    let path = settings_path()?;
    let raw = read_file(&path).map_err(|error| {
        if error.has_io_error_kind(std::io::ErrorKind::NotFound) {
            AppError::new(
                ErrorCode::ConfigMissing,
                format!("Configuration file is missing at {}", path.display()),
            )
        } else {
            AppError::with_source(
                ErrorCode::FilesystemIoFailed,
                format!("Failed to read configuration file at {}", path.display()),
                error,
            )
        }
    })?;

    let settings: Settings = serde_json::from_str(&raw).map_err(|error| {
        AppError::with_source(
            ErrorCode::ConfigInvalid,
            "Failed to parse configuration file",
            error,
        )
    })?;

    settings
        .validate()
        .map_err(|error| AppError::new(ErrorCode::ConfigInvalid, error))?;

    Ok(settings)
}

/// Loads settings if present, otherwise initializes canonical defaults.
///
/// Fallback policy:
/// - Missing configuration triggers safe default initialization.
/// - Invalid configuration remains a hard failure.
///
/// Security rationale:
/// - Operator-authored invalid configuration must not be silently replaced,
///   because overwriting an edited file would obscure an operational problem.
pub fn load_or_init() -> Result<Settings, AppError> {
    match load() {
        Ok(settings) => Ok(settings),
        Err(error) if error.code() == ErrorCode::ConfigMissing.as_str() => init_default(),
        Err(error) => Err(error),
    }
}

/// Loads settings if present, otherwise initializes a requested canonical profile.
///
/// Profile-aware fallback policy:
/// - Missing configuration triggers initialization for the requested profile.
/// - Existing valid configuration is preserved as-is.
/// - Invalid configuration remains a hard failure.
pub fn load_or_init_for_profile(profile: &str) -> Result<Settings, AppError> {
    match load() {
        Ok(settings) => Ok(settings),
        Err(error) if error.code() == ErrorCode::ConfigMissing.as_str() => {
            init_for_profile(profile)
        }
        Err(error) => Err(error),
    }
}

/// Persists a validated AOXC settings document to disk.
///
/// Validation and encoding flow:
/// 1. Validate the semantic settings payload.
/// 2. Serialize as stable pretty-printed JSON.
/// 3. Write to the canonical AOXC settings path.
///
/// Failure policy:
/// - Validation failures are reported as `ConfigInvalid`.
/// - Serialization failures are reported as `OutputEncodingFailed`.
/// - Filesystem write failures are surfaced by the lower-level data-home layer.
pub fn persist(settings: &Settings) -> Result<(), AppError> {
    settings
        .validate()
        .map_err(|error| AppError::new(ErrorCode::ConfigInvalid, error))?;

    let path = settings_path()?;
    let content = serde_json::to_string_pretty(settings).map_err(|error| {
        AppError::with_source(
            ErrorCode::OutputEncodingFailed,
            "Failed to encode configuration",
            error,
        )
    })?;

    write_file(&path, &content)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        data_home::{ensure_layout, read_file},
        test_support::{AoxcHomeGuard, TestHome, aoxc_home_test_lock},
    };
    use serde_json::json;
    use std::path::PathBuf;

    /// Executes a loader test inside a process-safe isolated AOXC home.
    ///
    /// Isolation policy:
    /// - Reuses the shared crate-level AOXC home lock so every test that mutates
    ///   `AOXC_HOME` participates in the same serialization domain.
    /// - Reuses the shared RAII home guard so environment restoration occurs
    ///   even when a test fails or panics.
    /// - Reuses the shared `TestHome` helper so disposable state remains under
    ///   the canonical AOXC test namespace.
    fn with_test_home<T>(label: &str, test: impl FnOnce(&TestHome) -> T) -> T {
        let _lock = aoxc_home_test_lock();
        let home = TestHome::new(label);
        let _guard = AoxcHomeGuard::install(home.path());
        test(&home)
    }

    /// Returns the expected canonical settings path for the supplied test home.
    fn expected_settings_file(home: &TestHome) -> PathBuf {
        home.path().join("config").join("settings.json")
    }

    /// Returns the actual canonical settings path derived from the active AOXC home.
    fn canonical_settings_file() -> PathBuf {
        settings_path().expect("settings path must resolve")
    }

    /// Ensures the active AOXC home layout exists.
    fn ensure_active_layout() {
        let home = resolve_home().expect("active AOXC home must resolve");
        ensure_layout(&home).expect("active AOXC layout must be created");
    }

    /// Removes the canonical settings file when present so missing-file tests start
    /// from a deterministic filesystem state.
    fn remove_settings_file_if_present() {
        let path = canonical_settings_file();
        let _ = std::fs::remove_file(path);
    }

    /// Writes raw fixture content to the canonical settings path used by the loader.
    fn write_settings_fixture(content: &str) -> PathBuf {
        ensure_active_layout();
        let path = canonical_settings_file();
        write_file(&path, content).expect("settings fixture should be written");
        path
    }

    /// Builds a valid settings fixture bound to the active AOXC home.
    fn valid_settings_for_active_home() -> Settings {
        let home = resolve_home().expect("test home should resolve");
        Settings::default_for(home.display().to_string())
    }

    /// Builds syntactically valid but semantically invalid settings JSON matching
    /// the current `Settings` schema.
    fn semantically_invalid_settings_json() -> String {
        json!({
            "home_dir": "",
            "profile": "validation",
            "logging": {
                "level": "info",
                "json": false
            },
            "network": {
                "bind_host": "",
                "p2p_port": 28657,
                "rpc_port": 28657,
                "enforce_official_peers": true
            },
            "telemetry": {
                "enable_metrics": true,
                "prometheus_port": 28657
            },
            "policy": {
                "require_key_material": true,
                "require_genesis": true,
                "allow_remote_peers": false
            }
        })
        .to_string()
    }

    #[test]
    fn settings_path_points_to_config_settings_json() {
        with_test_home("loader-settings-path", |_home| {
            let path = settings_path().expect("settings path must resolve");
            let rendered = path.display().to_string();

            assert!(rendered.ends_with("config/settings.json"));
        });
    }

    #[test]
    fn settings_path_resolves_inside_active_test_home() {
        with_test_home("loader-settings-home-resolution", |home| {
            let path = settings_path().expect("settings path must resolve");

            assert_eq!(path, expected_settings_file(home));
        });
    }

    #[test]
    fn init_default_creates_validation_profile_settings() {
        with_test_home("loader-init-default", |_home| {
            let settings = init_default().expect("default settings initialization must succeed");

            assert_eq!(settings.profile, "validation");
        });
    }

    #[test]
    fn init_default_persists_configuration_to_disk() {
        with_test_home("loader-init-default-persist", |home| {
            let settings = init_default().expect("default settings initialization must succeed");

            assert!(expected_settings_file(home).is_file());
            assert_eq!(settings.profile, "validation");
        });
    }

    #[test]
    fn init_default_roundtrip_loads_the_same_profile() {
        with_test_home("loader-init-default-roundtrip", |_home| {
            let initialized = init_default().expect("default settings initialization must succeed");
            let loaded = load().expect("persisted settings must load successfully");

            assert_eq!(initialized.profile, "validation");
            assert_eq!(loaded.profile, "validation");
        });
    }

    #[test]
    fn init_for_profile_accepts_validation_alias() {
        with_test_home("loader-init-validator-alias", |_home| {
            let settings = init_for_profile("validator")
                .expect("legacy validator alias initialization must succeed");

            assert_eq!(settings.profile, "validation");
        });
    }

    #[test]
    fn init_for_profile_persists_requested_profile() {
        with_test_home("loader-init-profile-persist", |_home| {
            let settings = init_for_profile("devnet").expect("profile initialization must succeed");
            let loaded = load().expect("persisted settings must load successfully");

            assert_eq!(settings.profile, "devnet");
            assert_eq!(loaded.profile, "devnet");
        });
    }

    #[test]
    fn init_for_profile_rejects_invalid_profile_names() {
        with_test_home("loader-init-invalid-profile", |_home| {
            let error =
                init_for_profile("not-a-real-profile").expect_err("invalid profile must fail");

            assert_eq!(error.code(), "AOXC-CFG-002");
            assert!(format!("{error}").contains("profile must be one of"));
        });
    }

    #[test]
    fn persist_roundtrip_preserves_settings() {
        with_test_home("loader-persist-roundtrip", |_home| {
            let original = valid_settings_for_active_home();

            persist(&original).expect("settings persistence must succeed");
            let loaded = load().expect("settings load must succeed");

            assert_eq!(original, loaded);
        });
    }

    #[test]
    fn persist_writes_pretty_printed_json_to_disk() {
        with_test_home("loader-persist-pretty-json", |_home| {
            let settings = valid_settings_for_active_home();

            persist(&settings).expect("settings persistence must succeed");
            let raw =
                read_file(&canonical_settings_file()).expect("persisted settings file must exist");

            assert!(raw.contains('\n'));
            assert!(raw.contains("\"profile\""));
        });
    }

    #[test]
    fn load_returns_config_missing_when_file_is_absent() {
        with_test_home("loader-load-missing", |_home| {
            ensure_active_layout();
            remove_settings_file_if_present();

            let error = load().expect_err("missing settings file must be rejected");

            assert_eq!(error.code(), "AOXC-CFG-001");
            assert!(format!("{error}").contains("Configuration file is missing"));
        });
    }

    #[test]
    fn load_rejects_invalid_json() {
        with_test_home("loader-load-invalid-json", |home| {
            let path = write_settings_fixture("{ invalid json");

            assert_eq!(path, canonical_settings_file());
            assert_eq!(path, expected_settings_file(home));

            let error = load().expect_err("invalid JSON must be rejected");

            assert_eq!(error.code(), "AOXC-CFG-002");
            assert!(format!("{error}").contains("Failed to parse configuration file"));
        });
    }

    #[test]
    fn load_rejects_semantically_invalid_settings() {
        with_test_home("loader-load-invalid-settings", |home| {
            let path = write_settings_fixture(&semantically_invalid_settings_json());

            assert_eq!(path, canonical_settings_file());
            assert_eq!(path, expected_settings_file(home));

            let error =
                load().expect_err("semantically invalid settings must be rejected after parsing");

            assert_eq!(error.code(), "AOXC-CFG-002");
        });
    }

    #[test]
    fn load_or_init_creates_default_settings_when_missing() {
        with_test_home("loader-load-or-init-default", |_home| {
            ensure_active_layout();
            remove_settings_file_if_present();

            let settings = load_or_init().expect("load_or_init must succeed for missing config");

            assert_eq!(settings.profile, "validation");
            assert!(canonical_settings_file().is_file());
        });
    }

    #[test]
    fn load_or_init_returns_existing_settings_without_overwriting() {
        with_test_home("loader-load-or-init-existing", |_home| {
            let existing = Settings::default_for_profile(
                resolve_home()
                    .expect("test home should resolve")
                    .display()
                    .to_string(),
                "testnet",
            )
            .expect("test fixture settings must be created");

            persist(&existing).expect("existing settings fixture must persist");

            let loaded = load_or_init().expect("existing configuration should load successfully");

            assert_eq!(loaded.profile, "testnet");
        });
    }

    #[test]
    fn load_or_init_propagates_invalid_existing_configuration() {
        with_test_home("loader-load-or-init-invalid-existing", |_home| {
            write_settings_fixture("{ invalid json");

            let error = load_or_init()
                .expect_err("invalid existing configuration must remain a hard failure");

            assert_eq!(error.code(), "AOXC-CFG-002");
            assert!(format!("{error}").contains("Failed to parse configuration file"));
        });
    }

    #[test]
    fn load_or_init_for_profile_initializes_requested_profile_when_missing() {
        with_test_home("loader-load-or-init-devnet", |_home| {
            ensure_active_layout();
            remove_settings_file_if_present();

            let settings = load_or_init_for_profile("devnet")
                .expect("profile-aware load or init must succeed");

            assert_eq!(settings.profile, "devnet");
            assert!(canonical_settings_file().is_file());
        });
    }

    #[test]
    fn load_or_init_for_profile_returns_existing_settings_without_overwriting_profile() {
        with_test_home("loader-load-existing-profile", |_home| {
            let existing = Settings::default_for_profile(
                resolve_home()
                    .expect("test home should resolve")
                    .display()
                    .to_string(),
                "testnet",
            )
            .expect("test fixture settings must be created");

            persist(&existing).expect("existing settings fixture must persist");

            let loaded = load_or_init_for_profile("devnet")
                .expect("existing configuration should load successfully");

            assert_eq!(loaded.profile, "testnet");
        });
    }

    #[test]
    fn load_or_init_for_profile_propagates_invalid_existing_configuration() {
        with_test_home("loader-load-existing-invalid", |_home| {
            let path = write_settings_fixture("{ invalid json");

            assert_eq!(path, canonical_settings_file());

            let error = load_or_init_for_profile("devnet")
                .expect_err("invalid existing configuration must remain a hard failure");

            assert_eq!(error.code(), "AOXC-CFG-002");
            assert!(format!("{error}").contains("Failed to parse configuration file"));
        });
    }

    #[test]
    fn load_or_init_for_profile_rejects_invalid_requested_profile_when_missing() {
        with_test_home("loader-load-or-init-invalid-profile", |_home| {
            ensure_active_layout();
            remove_settings_file_if_present();

            let error = load_or_init_for_profile("not-a-real-profile")
                .expect_err("invalid requested profile must be rejected");

            assert_eq!(error.code(), "AOXC-CFG-002");
            assert!(format!("{error}").contains("profile must be one of"));
            assert!(!canonical_settings_file().is_file());
        });
    }
}
