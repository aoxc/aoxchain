// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use crate::{
    data_home::{read_file, resolve_home, write_file},
    error::{AppError, ErrorCode},
};
use std::path::PathBuf;

/// Replace this import with the exact concrete ledger state type used in AOXC.
use crate::economy::ledger::LedgerState;

pub fn ledger_json_path() -> Result<PathBuf, AppError> {
    Ok(resolve_home()?.join("ledger").join("ledger.json"))
}

pub fn load_ledger_json() -> Result<LedgerState, AppError> {
    let path = ledger_json_path()?;
    let raw = read_file(&path).map_err(|_| {
        AppError::new(
            ErrorCode::FilesystemIoFailed,
            format!("Ledger file is missing at {}", path.display()),
        )
    })?;

    serde_json::from_str::<LedgerState>(&raw).map_err(|e| {
        AppError::with_source(ErrorCode::OutputEncodingFailed, "Failed to parse ledger JSON", e)
    })
}

pub fn persist_ledger_json(ledger: &LedgerState) -> Result<(), AppError> {
    let path = ledger_json_path()?;
    let content = serde_json::to_string_pretty(ledger).map_err(|e| {
        AppError::with_source(
            ErrorCode::OutputEncodingFailed,
            "Failed to encode ledger JSON",
            e,
        )
    })?;

    write_file(&path, &content)
}
