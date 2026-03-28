// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use crate::{
    data_home::resolve_home,
    economy::ledger::LedgerState,
    error::{AppError, ErrorCode},
    node::state::NodeState,
};
use chrono::Utc;
use redb::{Database, ReadableDatabase, TableDefinition};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

const CHAIN_TABLE: TableDefinition<&str, &str> = TableDefinition::new("chain_state");
const CHAIN_LOG_TABLE: TableDefinition<&str, &str> = TableDefinition::new("chain_logs");
const NODE_STATE_KEY: &str = "node_state";
const LEDGER_STATE_KEY: &str = "ledger_state";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ChainLogEntry {
    pub id: String,
    pub ts: String,
    pub category: String,
    pub action: String,
    pub detail: String,
}

pub fn main_redb_path() -> Result<PathBuf, AppError> {
    Ok(resolve_home()?.join("runtime").join("db").join("main.redb"))
}

fn open_or_create_db(path: &Path) -> Result<Database, AppError> {
    if path.exists() {
        Database::open(path).map_err(|e| {
            AppError::new(
                ErrorCode::FilesystemIoFailed,
                format!("Failed to open chain redb database {}: {e}", path.display()),
            )
        })
    } else {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                AppError::with_source(
                    ErrorCode::FilesystemIoFailed,
                    format!("Failed to create chain redb parent {}", parent.display()),
                    e,
                )
            })?;
        }

        Database::create(path).map_err(|e| {
            AppError::new(
                ErrorCode::FilesystemIoFailed,
                format!(
                    "Failed to create chain redb database {}: {e}",
                    path.display()
                ),
            )
        })
    }
}

fn read_json_value<T: serde::de::DeserializeOwned>(
    db: &Database,
    key: &str,
    not_found_code: ErrorCode,
    decode_code: ErrorCode,
    missing_message: String,
) -> Result<T, AppError> {
    let read_txn = db.begin_read().map_err(|e| {
        AppError::new(
            not_found_code,
            format!("Failed to begin chain redb read transaction: {e}"),
        )
    })?;

    let table = read_txn.open_table(CHAIN_TABLE).map_err(|e| {
        AppError::new(
            not_found_code,
            format!("Failed to open chain state table in redb: {e}"),
        )
    })?;

    let value = table.get(key).map_err(|e| {
        AppError::new(
            not_found_code,
            format!("Failed to read key `{key}` from chain redb: {e}"),
        )
    })?;

    let Some(value) = value else {
        return Err(AppError::new(not_found_code, missing_message));
    };

    serde_json::from_str::<T>(value.value()).map_err(|e| {
        AppError::with_source(
            decode_code,
            format!("Failed to decode `{key}` payload from chain redb"),
            e,
        )
    })
}

fn write_json_value<T: Serialize>(
    db: &Database,
    key: &str,
    value: &T,
    code: ErrorCode,
) -> Result<(), AppError> {
    let json = serde_json::to_string_pretty(value).map_err(|e| {
        AppError::with_source(
            ErrorCode::OutputEncodingFailed,
            format!("Failed to encode `{key}` payload for chain redb"),
            e,
        )
    })?;

    let write_txn = db.begin_write().map_err(|e| {
        AppError::new(
            code,
            format!("Failed to begin chain redb write transaction: {e}"),
        )
    })?;

    {
        let mut table = write_txn.open_table(CHAIN_TABLE).map_err(|e| {
            AppError::new(
                code,
                format!("Failed to open chain state table for write: {e}"),
            )
        })?;
        table.insert(key, json.as_str()).map_err(|e| {
            AppError::new(
                code,
                format!("Failed to write key `{key}` into chain redb: {e}"),
            )
        })?;
    }

    write_txn.commit().map_err(|e| {
        AppError::new(
            code,
            format!("Failed to commit chain redb write transaction: {e}"),
        )
    })?;

    Ok(())
}

pub fn load_node_state() -> Result<NodeState, AppError> {
    let path = main_redb_path()?;
    let db = open_or_create_db(&path)?;

    let state: NodeState = read_json_value(
        &db,
        NODE_STATE_KEY,
        ErrorCode::NodeStateInvalid,
        ErrorCode::NodeStateInvalid,
        format!(
            "Main chain redb database {} does not contain node_state",
            path.display()
        ),
    )?;

    state
        .validate()
        .map_err(|e| AppError::new(ErrorCode::NodeStateInvalid, e))?;
    Ok(state)
}

pub fn persist_node_state(state: &NodeState) -> Result<(), AppError> {
    let db = open_or_create_db(&main_redb_path()?)?;
    write_json_value(&db, NODE_STATE_KEY, state, ErrorCode::FilesystemIoFailed)?;
    Ok(())
}

pub fn load_ledger_state() -> Result<LedgerState, AppError> {
    let path = main_redb_path()?;
    let db = open_or_create_db(&path)?;

    let ledger: LedgerState = read_json_value(
        &db,
        LEDGER_STATE_KEY,
        ErrorCode::LedgerInvalid,
        ErrorCode::LedgerInvalid,
        format!(
            "Main chain redb database {} does not contain ledger_state",
            path.display()
        ),
    )?;

    ledger
        .validate()
        .map_err(|e| AppError::new(ErrorCode::LedgerInvalid, e))?;
    Ok(ledger)
}

pub fn persist_ledger_state(ledger: &LedgerState) -> Result<(), AppError> {
    ledger
        .validate()
        .map_err(|e| AppError::new(ErrorCode::LedgerInvalid, e))?;
    let db = open_or_create_db(&main_redb_path()?)?;
    write_json_value(&db, LEDGER_STATE_KEY, ledger, ErrorCode::FilesystemIoFailed)?;
    Ok(())
}

pub fn append_chain_log(category: &str, action: &str, detail: &str) -> Result<(), AppError> {
    let db = open_or_create_db(&main_redb_path()?)?;
    let ts = Utc::now().to_rfc3339();
    let id = format!("{}-{}-{}", ts, category, action);
    let entry = ChainLogEntry {
        id: id.clone(),
        ts,
        category: category.to_string(),
        action: action.to_string(),
        detail: detail.to_string(),
    };

    let payload = serde_json::to_string(&entry).map_err(|e| {
        AppError::with_source(
            ErrorCode::OutputEncodingFailed,
            "Failed to encode chain log entry",
            e,
        )
    })?;

    let write_txn = db.begin_write().map_err(|e| {
        AppError::new(
            ErrorCode::FilesystemIoFailed,
            format!("Failed to begin chain log write transaction: {e}"),
        )
    })?;
    {
        let mut table = write_txn.open_table(CHAIN_LOG_TABLE).map_err(|e| {
            AppError::new(
                ErrorCode::FilesystemIoFailed,
                format!("Failed to open chain log table for write: {e}"),
            )
        })?;
        table.insert(id.as_str(), payload.as_str()).map_err(|e| {
            AppError::new(
                ErrorCode::FilesystemIoFailed,
                format!("Failed to persist chain log entry: {e}"),
            )
        })?;
    }
    write_txn.commit().map_err(|e| {
        AppError::new(
            ErrorCode::FilesystemIoFailed,
            format!("Failed to commit chain log write transaction: {e}"),
        )
    })?;
    Ok(())
}
