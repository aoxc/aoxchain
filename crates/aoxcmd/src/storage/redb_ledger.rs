// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use crate::{
    data_home::resolve_home,
    error::{AppError, ErrorCode},
};
use redb::{Database, TableDefinition};
use std::path::{Path, PathBuf};

/// Replace this import with the exact concrete ledger state type used in AOXC.
use crate::economy::ledger::LedgerState;

const LEDGER_TABLE: TableDefinition<&str, &str> = TableDefinition::new("ledger_state");

fn ledger_redb_path() -> Result<PathBuf, AppError> {
    Ok(resolve_home()?.join("ledger").join("ledger.redb"))
}

fn open_or_create_db(path: &Path) -> Result<Database, AppError> {
    if path.exists() {
        Database::open(path).map_err(|e| {
            AppError::new(
                ErrorCode::FilesystemIoFailed,
                format!("Failed to open ledger redb database {}: {e}", path.display()),
            )
        })
    } else {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                AppError::with_source(
                    ErrorCode::FilesystemIoFailed,
                    format!("Failed to create ledger redb parent {}", parent.display()),
                    e,
                )
            })?;
        }

        Database::create(path).map_err(|e| {
            AppError::new(
                ErrorCode::FilesystemIoFailed,
                format!("Failed to create ledger redb database {}: {e}", path.display()),
            )
        })
    }
}

pub fn load_ledger_redb() -> Result<LedgerState, AppError> {
    let path = ledger_redb_path()?;
    let db = open_or_create_db(&path)?;

    let read_txn = db.begin_read().map_err(|e| {
        AppError::new(
            ErrorCode::FilesystemIoFailed,
            format!("Failed to begin ledger redb read transaction: {e}"),
        )
    })?;

    let table = read_txn.open_table(LEDGER_TABLE).map_err(|e| {
        AppError::new(
            ErrorCode::FilesystemIoFailed,
            format!("Failed to open ledger redb table: {e}"),
        )
    })?;

    let value = table.get("ledger").map_err(|e| {
        AppError::new(
            ErrorCode::FilesystemIoFailed,
            format!("Failed to read ledger from redb: {e}"),
        )
    })?;

    let Some(value) = value else {
        return Err(AppError::new(
            ErrorCode::FilesystemIoFailed,
            format!("Ledger redb database {} does not contain ledger", path.display()),
        ));
    };

    serde_json::from_str::<LedgerState>(value.value()).map_err(|e| {
        AppError::with_source(
            ErrorCode::OutputEncodingFailed,
            "Failed to decode ledger from redb JSON payload",
            e,
        )
    })
}

pub fn persist_ledger_redb(ledger: &LedgerState) -> Result<(), AppError> {
    let path = ledger_redb_path()?;
    let db = open_or_create_db(&path)?;

    let json = serde_json::to_string_pretty(ledger).map_err(|e| {
        AppError::with_source(
            ErrorCode::OutputEncodingFailed,
            "Failed to encode ledger for redb persistence",
            e,
        )
    })?;

    let write_txn = db.begin_write().map_err(|e| {
        AppError::new(
            ErrorCode::FilesystemIoFailed,
            format!("Failed to begin ledger redb write transaction: {e}"),
        )
    })?;

    {
        let mut table = write_txn.open_table(LEDGER_TABLE).map_err(|e| {
            AppError::new(
                ErrorCode::FilesystemIoFailed,
                format!("Failed to open ledger redb table: {e}"),
            )
        })?;

        table.insert("ledger", json.as_str()).map_err(|e| {
            AppError::new(
                ErrorCode::FilesystemIoFailed,
                format!("Failed to write ledger into redb: {e}"),
            )
        })?;
    }

    write_txn.commit().map_err(|e| {
        AppError::new(
            ErrorCode::FilesystemIoFailed,
            format!("Failed to commit ledger redb transaction: {e}"),
        )
    })?;

    Ok(())
}
