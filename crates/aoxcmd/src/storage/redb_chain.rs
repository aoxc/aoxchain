// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use crate::{
    economy::ledger::LedgerState,
    error::{AppError, ErrorCode},
    node::state::NodeState,
    storage::{
        RuntimeStateStore,
        redb_ledger::{load_ledger_redb, persist_ledger_redb},
        redb_runtime::{RedbRuntimeStateStore, runtime_state_redb_path},
    },
};
use redb::{Database, ReadableDatabase, ReadableTable, TableDefinition};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

const CHAIN_LOG_TABLE: TableDefinition<u64, &str> = TableDefinition::new("chain_log");

/// Canonical append-only chain-log record persisted in the runtime redb store.
///
/// Audit rationale:
/// - The structure is intentionally compact and deterministic.
/// - Every record carries a monotonic logical sequence identifier.
/// - Field naming remains explicit so downstream evidence/reporting surfaces
///   can consume the payload without ambiguous translation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChainLogEntry {
    pub sequence: u64,
    pub domain: String,
    pub action: String,
    pub details: String,
}

/// Returns the canonical primary AOXC redb path used by runtime state surfaces.
///
/// Compatibility rationale:
/// - Historical callers expect `redb_chain` to expose the canonical runtime db path.
/// - The authoritative path definition remains delegated to `redb_runtime`.
pub fn main_redb_path() -> Result<PathBuf, AppError> {
    runtime_state_redb_path()
}

/// Opens or creates the canonical runtime redb database backing chain-log
/// compatibility surfaces.
fn open_or_create_runtime_db(path: &Path) -> Result<Database, AppError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|error| {
            AppError::with_source(
                ErrorCode::FilesystemIoFailed,
                format!("Failed to create runtime redb parent {}", parent.display()),
                error,
            )
        })?;
    }

    Database::create(path).map_err(|error| {
        AppError::with_source(
            ErrorCode::FilesystemIoFailed,
            format!(
                "Failed to open or create canonical runtime redb database {}",
                path.display()
            ),
            error,
        )
    })
}

/// Loads the canonical node state through the dedicated runtime redb store.
///
/// Compatibility rationale:
/// - Historical callers import node-state helpers from `redb_chain`.
/// - The authoritative runtime persistence logic remains isolated in
///   `storage::redb_runtime`.
pub fn load_node_state() -> Result<NodeState, AppError> {
    let store = RedbRuntimeStateStore::open_default()?;
    let _ = store.initialize_if_absent()?;
    store.load_state()
}

/// Persists canonical node state through the dedicated runtime redb store.
///
/// Validation policy:
/// - Semantic validation remains enforced by the underlying runtime store.
/// - This facade intentionally adds no alternate persistence semantics.
pub fn persist_node_state(state: &NodeState) -> Result<(), AppError> {
    RedbRuntimeStateStore::open_default()?.persist_state(state)
}

/// Loads canonical ledger state through the dedicated ledger redb store.
///
/// Compatibility rationale:
/// - Historical callers still import ledger helpers from `redb_chain`.
/// - The authoritative ledger persistence logic remains delegated to
///   `storage::redb_ledger`.
pub fn load_ledger_state() -> Result<LedgerState, AppError> {
    load_ledger_redb()
}

/// Persists canonical ledger state through the dedicated ledger redb store.
///
/// Validation policy:
/// - Semantic validation remains enforced by the underlying ledger store.
/// - This facade exists strictly to preserve the legacy public API.
pub fn persist_ledger_state(ledger: &LedgerState) -> Result<(), AppError> {
    persist_ledger_redb(ledger)
}

/// Appends a deterministic breadcrumb into the canonical runtime redb chain-log table.
///
/// Storage contract:
/// - Chain-log records are stored inside the canonical runtime database.
/// - The table is append-only from the caller perspective.
/// - Sequence allocation is derived from the current highest logical key.
pub fn append_chain_log(domain: &str, action: &str, details: &str) -> Result<(), AppError> {
    let domain = domain.trim();
    let action = action.trim();
    let details = details.trim();

    if domain.is_empty() {
        return Err(AppError::new(
            ErrorCode::NodeStateInvalid,
            "Chain-log domain must not be empty",
        ));
    }

    if action.is_empty() {
        return Err(AppError::new(
            ErrorCode::NodeStateInvalid,
            "Chain-log action must not be empty",
        ));
    }

    let path = main_redb_path()?;
    let db = open_or_create_runtime_db(&path)?;

    let write_txn = db.begin_write().map_err(|error| {
        AppError::with_source(
            ErrorCode::FilesystemIoFailed,
            "Failed to begin runtime redb write transaction for chain-log append",
            error,
        )
    })?;

    {
        let mut table = write_txn.open_table(CHAIN_LOG_TABLE).map_err(|error| {
            AppError::with_source(
                ErrorCode::FilesystemIoFailed,
                "Failed to open chain_log table in runtime redb",
                error,
            )
        })?;

        let iter = table.iter().map_err(|error| {
            AppError::with_source(
                ErrorCode::FilesystemIoFailed,
                "Failed to iterate chain_log table for sequence allocation",
                error,
            )
        })?;

        let mut last_sequence: Option<u64> = None;
        for item in iter {
            let (key, _) = item.map_err(|error| {
                AppError::with_source(
                    ErrorCode::FilesystemIoFailed,
                    "Failed to inspect existing chain_log entry during sequence allocation",
                    error,
                )
            })?;
            last_sequence = Some(key.value());
        }

        let next_sequence = last_sequence.map_or(1_u64, |value| value.saturating_add(1));

        let entry = ChainLogEntry {
            sequence: next_sequence,
            domain: domain.to_string(),
            action: action.to_string(),
            details: details.to_string(),
        };

        let payload = serde_json::to_string(&entry).map_err(|error| {
            AppError::with_source(
                ErrorCode::OutputEncodingFailed,
                "Failed to encode chain-log entry for redb persistence",
                error,
            )
        })?;

        table
            .insert(next_sequence, payload.as_str())
            .map_err(|error| {
                AppError::with_source(
                    ErrorCode::FilesystemIoFailed,
                    "Failed to append chain-log entry into runtime redb",
                    error,
                )
            })?;
    }

    write_txn.commit().map_err(|error| {
        AppError::with_source(
            ErrorCode::FilesystemIoFailed,
            "Failed to commit runtime redb chain-log transaction",
            error,
        )
    })?;

    Ok(())
}

/// Loads chain-log records from the canonical runtime redb store.
///
/// Compatibility contract:
/// - `limit` restricts the number of returned records from newest to oldest.
/// - `category` filters by `domain` when provided.
/// - Returned records are normalized into ascending sequence order so callers
///   receive deterministic output.
pub fn load_chain_logs(
    limit: usize,
    category: Option<&str>,
) -> Result<Vec<ChainLogEntry>, AppError> {
    let path = main_redb_path()?;
    let db = open_or_create_runtime_db(&path)?;

    let read_txn = db.begin_read().map_err(|error| {
        AppError::with_source(
            ErrorCode::FilesystemIoFailed,
            "Failed to begin runtime redb read transaction for chain-log load",
            error,
        )
    })?;

    let table = read_txn.open_table(CHAIN_LOG_TABLE).map_err(|error| {
        AppError::with_source(
            ErrorCode::FilesystemIoFailed,
            "Failed to open chain_log table in runtime redb",
            error,
        )
    })?;

    let mut entries = Vec::new();
    let iter = table.iter().map_err(|error| {
        AppError::with_source(
            ErrorCode::FilesystemIoFailed,
            "Failed to iterate chain_log table",
            error,
        )
    })?;

    for item in iter {
        let (_, value) = item.map_err(|error| {
            AppError::with_source(
                ErrorCode::FilesystemIoFailed,
                "Failed to read chain_log entry from runtime redb",
                error,
            )
        })?;

        let entry = serde_json::from_str::<ChainLogEntry>(value.value()).map_err(|error| {
            AppError::with_source(
                ErrorCode::NodeStateInvalid,
                "Failed to decode chain-log entry from runtime redb JSON payload",
                error,
            )
        })?;

        entries.push(entry);
    }

    if let Some(category) = category.map(str::trim).filter(|value| !value.is_empty()) {
        entries.retain(|entry| entry.domain == category);
    }

    entries.sort_by_key(|entry| entry.sequence);

    if limit > 0 && entries.len() > limit {
        let start = entries.len() - limit;
        entries = entries.split_off(start);
    }

    Ok(entries)
}

#[cfg(test)]
mod tests {
    use super::{
        append_chain_log, load_chain_logs, load_ledger_state, load_node_state, main_redb_path,
        persist_ledger_state, persist_node_state,
    };
    use crate::{
        economy::ledger::LedgerState,
        error::ErrorCode,
        node::state::NodeState,
        test_support::{AoxcHomeGuard, TestHome, aoxc_home_test_lock},
    };

    fn with_test_home<T>(label: &str, test: impl FnOnce(&TestHome) -> T) -> T {
        let _lock = aoxc_home_test_lock();
        let home = TestHome::new(label);
        let _guard = AoxcHomeGuard::install(home.path());
        test(&home)
    }

    #[test]
    fn main_redb_path_resolves_inside_active_test_home() {
        with_test_home("redb-chain-main-path", |home| {
            let path = main_redb_path().expect("main redb path must resolve");

            assert_eq!(
                path,
                home.path().join("runtime").join("db").join("main.redb")
            );
        });
    }

    #[test]
    fn persist_and_reload_node_state_round_trips() {
        with_test_home("redb-chain-node-state-roundtrip", |_home| {
            let mut state = NodeState::bootstrap();
            state.current_height = 7;
            state.produced_blocks = 7;
            state.consensus.last_round = 3;
            state.consensus.last_message_kind = "commit".to_string();

            persist_node_state(&state).expect("node state should persist");
            let reloaded = load_node_state().expect("node state should reload");

            assert_eq!(reloaded.current_height, 7);
            assert_eq!(reloaded.produced_blocks, 7);
            assert_eq!(reloaded.consensus.last_round, 3);
            assert_eq!(reloaded.consensus.last_message_kind, "commit");
        });
    }

    #[test]
    fn persist_node_state_rejects_invalid_semantic_payload() {
        with_test_home("redb-chain-node-state-invalid", |_home| {
            let mut state = NodeState::bootstrap();
            state.current_height = 1;
            state.produced_blocks = 5;

            let error =
                persist_node_state(&state).expect_err("invalid node state must be rejected");

            assert_eq!(error.code(), ErrorCode::NodeStateInvalid.as_str());
        });
    }

    #[test]
    fn append_and_load_chain_logs_preserves_order() {
        with_test_home("redb-chain-log-roundtrip", |_home| {
            append_chain_log("runtime", "bootstrap", "node initialized")
                .expect("first chain-log append should succeed");
            append_chain_log("runtime", "persist", "state updated")
                .expect("second chain-log append should succeed");

            let logs = load_chain_logs(32, None).expect("chain-log entries should load");

            assert_eq!(logs.len(), 2);
            assert_eq!(logs[0].sequence, 1);
            assert_eq!(logs[0].domain, "runtime");
            assert_eq!(logs[0].action, "bootstrap");
            assert_eq!(logs[1].sequence, 2);
            assert_eq!(logs[1].action, "persist");
        });
    }

    #[test]
    fn append_and_load_chain_logs_support_category_filter() {
        with_test_home("redb-chain-log-category-filter", |_home| {
            append_chain_log("runtime", "bootstrap", "node initialized")
                .expect("runtime chain-log append should succeed");
            append_chain_log("ledger", "commit", "ledger persisted")
                .expect("ledger chain-log append should succeed");

            let logs =
                load_chain_logs(32, Some("ledger")).expect("filtered chain-log load should work");

            assert_eq!(logs.len(), 1);
            assert_eq!(logs[0].domain, "ledger");
            assert_eq!(logs[0].action, "commit");
        });
    }

    #[test]
    fn append_and_load_chain_logs_support_limit() {
        with_test_home("redb-chain-log-limit", |_home| {
            append_chain_log("runtime", "one", "1").expect("append one should succeed");
            append_chain_log("runtime", "two", "2").expect("append two should succeed");
            append_chain_log("runtime", "three", "3").expect("append three should succeed");

            let logs = load_chain_logs(2, None).expect("limited chain-log load should work");

            assert_eq!(logs.len(), 2);
            assert_eq!(logs[0].action, "two");
            assert_eq!(logs[1].action, "three");
        });
    }

    #[test]
    fn append_chain_log_rejects_empty_domain() {
        with_test_home("redb-chain-log-empty-domain", |_home| {
            let error = append_chain_log("   ", "persist", "detail")
                .expect_err("empty domain must be rejected");

            assert_eq!(error.code(), ErrorCode::NodeStateInvalid.as_str());
        });
    }

    #[test]
    fn append_chain_log_rejects_empty_action() {
        with_test_home("redb-chain-log-empty-action", |_home| {
            let error = append_chain_log("runtime", "   ", "detail")
                .expect_err("empty action must be rejected");

            assert_eq!(error.code(), ErrorCode::NodeStateInvalid.as_str());
        });
    }

    #[test]
    fn load_and_persist_ledger_state_delegate_to_ledger_redb_store() {
        with_test_home("redb-chain-ledger-facade", |_home| {
            let ledger = LedgerState::new();

            persist_ledger_state(&ledger).expect("ledger facade persistence should succeed");
            let reloaded = load_ledger_state().expect("ledger facade load should succeed");

            assert_eq!(reloaded.treasury_balance, ledger.treasury_balance);
            assert_eq!(reloaded.transfers, ledger.transfers);
            assert_eq!(reloaded.delegations, ledger.delegations);
        });
    }
}
