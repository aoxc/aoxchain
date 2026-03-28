// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use crate::{
    data_home::{read_file, resolve_home, write_file},
    economy::ledger::LedgerState,
    error::{AppError, ErrorCode},
};
use std::path::PathBuf;

/// Returns the canonical legacy JSON ledger path.
///
/// Canonical path policy:
/// - Legacy JSON ledger state is stored at:
///   `<AOXC_HOME>/ledger/ledger.json`.
///
/// Operational note:
/// - The canonical AOXC runtime ledger source of truth is the redb-backed
///   storage surface. This path helper exists only for legacy JSON
///   compatibility, migration, and explicit JSON-oriented tooling.
pub fn ledger_json_path() -> Result<PathBuf, AppError> {
    Ok(resolve_home()?.join("ledger").join("ledger.json"))
}

/// Loads and validates ledger state from the legacy JSON ledger path.
///
/// Error mapping policy:
/// - Missing file maps to `LedgerInvalid` only when the caller explicitly
///   requested legacy JSON and the file is absent.
/// - Other filesystem read failures map to `FilesystemIoFailed`.
/// - Decode and semantic validation failures map to `LedgerInvalid`.
pub fn load_ledger_json() -> Result<LedgerState, AppError> {
    let path = ledger_json_path()?;
    let raw = read_file(&path).map_err(|error| {
        if error.has_io_error_kind(std::io::ErrorKind::NotFound) {
            AppError::new(
                ErrorCode::LedgerInvalid,
                format!("Legacy ledger JSON is missing at {}", path.display()),
            )
        } else {
            AppError::with_source(
                ErrorCode::FilesystemIoFailed,
                format!("Failed to read legacy ledger JSON from {}", path.display()),
                error,
            )
        }
    })?;

    let ledger = serde_json::from_str::<LedgerState>(&raw).map_err(|error| {
        AppError::with_source(
            ErrorCode::LedgerInvalid,
            format!("Failed to parse legacy ledger JSON at {}", path.display()),
            error,
        )
    })?;

    ledger
        .validate()
        .map_err(|error| AppError::new(ErrorCode::LedgerInvalid, error.to_string()))?;

    Ok(ledger)
}

/// Persists validated ledger state to the legacy JSON ledger path.
///
/// Validation policy:
/// - Semantic validation is enforced before serialization.
/// - Only validated ledger payloads are encoded and written.
pub fn persist_ledger_json(ledger: &LedgerState) -> Result<(), AppError> {
    ledger
        .validate()
        .map_err(|error| AppError::new(ErrorCode::LedgerInvalid, error.to_string()))?;

    let path = ledger_json_path()?;
    let content = serde_json::to_string_pretty(ledger).map_err(|error| {
        AppError::with_source(
            ErrorCode::OutputEncodingFailed,
            format!("Failed to encode legacy ledger JSON for {}", path.display()),
            error,
        )
    })?;

    write_file(&path, &content)
}

#[cfg(test)]
mod tests {
    use super::{ledger_json_path, load_ledger_json, persist_ledger_json};
    use crate::{
        economy::ledger::LedgerState,
        error::ErrorCode,
        test_support::{aoxc_home_test_lock, AoxcHomeGuard, TestHome},
    };

    fn with_test_home<T>(label: &str, test: impl FnOnce(&TestHome) -> T) -> T {
        let _lock = aoxc_home_test_lock();
        let home = TestHome::new(label);
        let _guard = AoxcHomeGuard::install(home.path());
        test(&home)
    }

    #[test]
    fn ledger_json_path_resolves_inside_active_test_home() {
        with_test_home("ledger-json-path", |home| {
            let path = ledger_json_path().expect("ledger json path must resolve");

            assert_eq!(path, home.path().join("ledger").join("ledger.json"));
        });
    }

    #[test]
    fn persist_and_reload_legacy_ledger_json_round_trips_state() {
        with_test_home("ledger-json-roundtrip", |_home| {
            let ledger = LedgerState::new();

            persist_ledger_json(&ledger).expect("legacy ledger json should persist");
            let reloaded = load_ledger_json().expect("persisted legacy ledger json should load");

            assert_eq!(reloaded.treasury_balance, ledger.treasury_balance);
            assert_eq!(reloaded.transfers, ledger.transfers);
            assert_eq!(reloaded.delegations, ledger.delegations);
        });
    }

    #[test]
    fn load_ledger_json_rejects_invalid_payload() {
        with_test_home("ledger-json-invalid-payload", |_home| {
            let path = ledger_json_path().expect("ledger json path must resolve");
            crate::data_home::write_file(&path, "{ invalid json")
                .expect("invalid fixture should be written");

            let error = load_ledger_json().expect_err("invalid legacy ledger json must fail");

            assert_eq!(error.code(), ErrorCode::LedgerInvalid.as_str());
        });
    }

    #[test]
    fn persist_ledger_json_rejects_invalid_semantic_state() {
        with_test_home("ledger-json-invalid-semantic", |_home| {
            let mut ledger = LedgerState::new();
            ledger.delegations.insert("   ".to_string(), 10);

            let error =
                persist_ledger_json(&ledger).expect_err("invalid ledger state must be rejected");

            assert_eq!(error.code(), ErrorCode::LedgerInvalid.as_str());
        });
    }
}
