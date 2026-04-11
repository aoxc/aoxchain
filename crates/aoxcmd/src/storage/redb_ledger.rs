// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use crate::{
    data_home::resolve_home,
    economy::ledger::LedgerState,
    error::{AppError, ErrorCode},
};
use redb::{Database, ReadableDatabase, TableDefinition};
use std::path::{Path, PathBuf};

const LEDGER_TABLE: TableDefinition<&str, &str> = TableDefinition::new("ledger_state");
const LEDGER_KEY: &str = "ledger";

/// Returns the canonical AOXC redb ledger path.
///
/// Canonical storage policy:
/// - Ledger state is stored at:
///   `<AOXC_HOME>/ledger/db/main.redb`.
///
/// Design rationale:
/// - This keeps the ledger storage layout aligned with the broader AOXC
///   redb-backed storage model.
/// - Path resolution is derived strictly from the effective AOXC home so that
///   production homes, local-dev homes, and isolated test homes remain
///   consistent.
pub fn ledger_redb_path() -> Result<PathBuf, AppError> {
    Ok(resolve_home()?.join("ledger").join("db").join("main.redb"))
}

/// Opens or creates the canonical ledger redb database.
///
/// Failure policy:
/// - Parent directory creation failures map to `FilesystemIoFailed`.
/// - redb open/create failures map to `FilesystemIoFailed`.
fn open_or_create_db(path: &Path) -> Result<Database, AppError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|error| {
            AppError::with_source(
                ErrorCode::FilesystemIoFailed,
                format!("Failed to create ledger redb parent {}", parent.display()),
                error,
            )
        })?;
    }

    Database::create(path).map_err(|error| {
        AppError::with_source(
            ErrorCode::FilesystemIoFailed,
            format!(
                "Failed to open or create ledger redb database {}",
                path.display()
            ),
            error,
        )
    })
}

/// Loads and validates ledger state from the canonical redb store.
///
/// Error policy:
/// - Transaction and table access failures map to `FilesystemIoFailed`.
/// - Missing logical ledger payload maps to `LedgerInvalid`.
/// - Decode and semantic validation failures map to `LedgerInvalid`.
pub fn load_ledger_redb() -> Result<LedgerState, AppError> {
    let path = ledger_redb_path()?;
    let db = open_or_create_db(&path)?;

    let read_txn = db.begin_read().map_err(|error| {
        AppError::with_source(
            ErrorCode::LedgerInvalid,
            "Failed to begin ledger redb read transaction",
            error,
        )
    })?;

    let table = read_txn.open_table(LEDGER_TABLE).map_err(|error| {
        AppError::with_source(
            ErrorCode::LedgerInvalid,
            "Ledger redb table is missing or unreadable",
            error,
        )
    })?;

    let value = table.get(LEDGER_KEY).map_err(|error| {
        AppError::with_source(
            ErrorCode::LedgerInvalid,
            "Failed to read ledger from redb",
            error,
        )
    })?;

    let Some(value) = value else {
        return Err(AppError::new(
            ErrorCode::LedgerInvalid,
            format!(
                "Ledger redb database {} does not contain a canonical ledger payload",
                path.display()
            ),
        ));
    };

    let ledger = serde_json::from_str::<LedgerState>(value.value()).map_err(|error| {
        AppError::with_source(
            ErrorCode::LedgerInvalid,
            "Failed to decode ledger from redb JSON payload",
            error,
        )
    })?;

    ledger
        .validate()
        .map_err(|error| AppError::new(ErrorCode::LedgerInvalid, error))?;

    Ok(ledger)
}

/// Persists validated ledger state into the canonical redb store.
///
/// Validation policy:
/// - Semantic validation is enforced before serialization and write.
/// - Only validated ledger payloads are committed.
pub fn persist_ledger_redb(ledger: &LedgerState) -> Result<(), AppError> {
    ledger
        .validate()
        .map_err(|error| AppError::new(ErrorCode::LedgerInvalid, error))?;

    let path = ledger_redb_path()?;
    let db = open_or_create_db(&path)?;

    let json = serde_json::to_string_pretty(ledger).map_err(|error| {
        AppError::with_source(
            ErrorCode::OutputEncodingFailed,
            "Failed to encode ledger for redb persistence",
            error,
        )
    })?;

    let write_txn = db.begin_write().map_err(|error| {
        AppError::with_source(
            ErrorCode::FilesystemIoFailed,
            "Failed to begin ledger redb write transaction",
            error,
        )
    })?;

    {
        let mut table = write_txn.open_table(LEDGER_TABLE).map_err(|error| {
            AppError::with_source(
                ErrorCode::FilesystemIoFailed,
                "Failed to open ledger redb table",
                error,
            )
        })?;

        table.insert(LEDGER_KEY, json.as_str()).map_err(|error| {
            AppError::with_source(
                ErrorCode::FilesystemIoFailed,
                "Failed to write ledger into redb",
                error,
            )
        })?;
    }

    write_txn.commit().map_err(|error| {
        AppError::with_source(
            ErrorCode::FilesystemIoFailed,
            "Failed to commit ledger redb transaction",
            error,
        )
    })?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{ledger_redb_path, load_ledger_redb, persist_ledger_redb};
    use crate::{
        economy::ledger::LedgerState,
        error::ErrorCode,
        test_support::{AoxcHomeGuard, TestHome, aoxc_home_test_lock},
    };

    fn with_test_home<T>(label: &str, test: impl FnOnce(&TestHome) -> T) -> T {
        let _lock = aoxc_home_test_lock();
        let home = TestHome::new(label).expect("test home should be created");
        let _guard = AoxcHomeGuard::install(&_lock, home.path());
        test(&home)
    }

    #[test]
    fn ledger_redb_path_resolves_inside_active_test_home() {
        with_test_home("ledger-redb-path", |home| {
            let path = ledger_redb_path().expect("ledger redb path must resolve");

            assert_eq!(
                path,
                home.path().join("ledger").join("db").join("main.redb")
            );
        });
    }

    #[test]
    fn persist_and_reload_ledger_redb_round_trips_state() {
        with_test_home("ledger-redb-roundtrip", |_home| {
            let ledger = LedgerState::new();

            persist_ledger_redb(&ledger).expect("ledger should persist into redb");
            let reloaded = load_ledger_redb().expect("persisted ledger should reload");

            assert_eq!(reloaded.treasury_balance, ledger.treasury_balance);
            assert_eq!(reloaded.transfers, ledger.transfers);
            assert_eq!(reloaded.delegations, ledger.delegations);
        });
    }

    #[test]
    fn persist_ledger_redb_rejects_invalid_semantic_payload() {
        with_test_home("ledger-redb-invalid-semantic", |_home| {
            let mut ledger = LedgerState::new();
            ledger.delegations.insert("   ".to_string(), 10);

            let error = persist_ledger_redb(&ledger).expect_err("invalid ledger must be rejected");

            assert_eq!(error.code(), ErrorCode::LedgerInvalid.as_str());
        });
    }

    #[test]
    fn load_ledger_redb_rejects_missing_logical_record() {
        with_test_home("ledger-redb-missing-record", |_home| {
            let error =
                load_ledger_redb().expect_err("missing logical ledger record must be rejected");

            assert_eq!(error.code(), ErrorCode::LedgerInvalid.as_str());
        });
    }
}
