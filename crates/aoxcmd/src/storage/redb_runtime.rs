// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use crate::{
    data_home::resolve_home,
    error::{AppError, ErrorCode},
    node::state::NodeState,
    storage::RuntimeStateStore,
};
use redb::{Database, ReadableDatabase, TableDefinition};
use std::path::{Path, PathBuf};

const RUNTIME_STATE_TABLE: TableDefinition<&str, &str> = TableDefinition::new("runtime_state");
const NODE_STATE_KEY: &str = "node_state";

/// Returns the canonical AOXC redb runtime-state path.
///
/// Canonical storage policy:
/// - Runtime state is stored at:
///   `<AOXC_HOME>/runtime/db/main.redb`.
///
/// Design rationale:
/// - This aligns runtime-state persistence with the canonical AOXC redb layout
///   already used by the wider storage and lifecycle surfaces.
/// - Path resolution is derived strictly from the effective AOXC home so that
///   test homes, local-dev homes, and production homes behave consistently.
pub fn runtime_state_redb_path() -> Result<PathBuf, AppError> {
    Ok(resolve_home()?.join("runtime").join("db").join("main.redb"))
}

/// Opens the canonical runtime-state database, creating parent directories and
/// the database file when necessary.
///
/// Failure policy:
/// - Filesystem preparation failures map to `FilesystemIoFailed`.
/// - Database open/create failures map to `FilesystemIoFailed`.
fn open_or_create_db(path: &Path) -> Result<Database, AppError> {
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
                "Failed to open or create runtime redb database {}",
                path.display()
            ),
            error,
        )
    })
}

/// redb-backed AOXC runtime-state store.
///
/// Design intent:
/// - Persist the canonical `NodeState` payload in a stable single-record table.
/// - Keep semantic validation at the persistence boundary.
/// - Return only validated runtime state to callers.
#[derive(Debug)]
pub struct RedbRuntimeStateStore {
    db: Database,
}

impl RedbRuntimeStateStore {
    /// Opens the canonical default AOXC runtime-state store.
    pub fn open_default() -> Result<Self, AppError> {
        let path = runtime_state_redb_path()?;
        let db = open_or_create_db(&path)?;
        Ok(Self { db })
    }
}

impl RuntimeStateStore for RedbRuntimeStateStore {
    /// Loads and validates the canonical runtime state from redb.
    ///
    /// Error policy:
    /// - Missing table or missing `node_state` entry maps to `NodeStateInvalid`.
    /// - Decode and semantic validation failures map to `NodeStateInvalid`.
    fn load_state(&self) -> Result<NodeState, AppError> {
        let read_txn = self.db.begin_read().map_err(|error| {
            AppError::with_source(
                ErrorCode::NodeStateInvalid,
                "Failed to begin runtime redb read transaction",
                error,
            )
        })?;

        let table = read_txn.open_table(RUNTIME_STATE_TABLE).map_err(|error| {
            AppError::with_source(
                ErrorCode::NodeStateInvalid,
                "Failed to open runtime redb table",
                error,
            )
        })?;

        let value = table.get(NODE_STATE_KEY).map_err(|error| {
            AppError::with_source(
                ErrorCode::NodeStateInvalid,
                "Failed to read runtime node state from redb",
                error,
            )
        })?;

        let Some(value) = value else {
            return Err(AppError::new(
                ErrorCode::NodeStateInvalid,
                "Runtime redb database does not contain node_state",
            ));
        };

        let payload: &str = value.value();
        let state: NodeState = serde_json::from_str(payload).map_err(|error| {
            AppError::with_source(
                ErrorCode::NodeStateInvalid,
                "Failed to decode runtime node state from redb",
                error,
            )
        })?;

        state
            .validate()
            .map_err(|error| AppError::new(ErrorCode::NodeStateInvalid, error))?;

        Ok(state)
    }

    /// Persists validated runtime state into the canonical redb store.
    ///
    /// Validation policy:
    /// - Semantic validation is enforced before serialization and write.
    /// - Only validated runtime state is committed.
    fn persist_state(&self, state: &NodeState) -> Result<(), AppError> {
        state
            .validate()
            .map_err(|error| AppError::new(ErrorCode::NodeStateInvalid, error))?;

        let json = serde_json::to_string_pretty(state).map_err(|error| {
            AppError::with_source(
                ErrorCode::OutputEncodingFailed,
                "Failed to encode node state for redb persistence",
                error,
            )
        })?;

        let write_txn = self.db.begin_write().map_err(|error| {
            AppError::with_source(
                ErrorCode::FilesystemIoFailed,
                "Failed to begin runtime redb write transaction",
                error,
            )
        })?;

        {
            let mut table = write_txn.open_table(RUNTIME_STATE_TABLE).map_err(|error| {
                AppError::with_source(
                    ErrorCode::FilesystemIoFailed,
                    "Failed to open runtime redb table",
                    error,
                )
            })?;

            table
                .insert(NODE_STATE_KEY, json.as_str())
                .map_err(|error| {
                    AppError::with_source(
                        ErrorCode::FilesystemIoFailed,
                        "Failed to write runtime node state into redb",
                        error,
                    )
                })?;
        }

        write_txn.commit().map_err(|error| {
            AppError::with_source(
                ErrorCode::FilesystemIoFailed,
                "Failed to commit runtime redb transaction",
                error,
            )
        })?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{runtime_state_redb_path, RedbRuntimeStateStore};
    use crate::{
        error::ErrorCode,
        node::state::NodeState,
        storage::RuntimeStateStore,
        test_support::{aoxc_home_test_lock, AoxcHomeGuard, TestHome},
    };

    fn with_test_home<T>(label: &str, test: impl FnOnce(&TestHome) -> T) -> T {
        let _lock = aoxc_home_test_lock();
        let home = TestHome::new(label);
        let _guard = AoxcHomeGuard::install(home.path());
        test(&home)
    }

    #[test]
    fn runtime_state_redb_path_resolves_inside_active_test_home() {
        with_test_home("runtime-redb-path", |home| {
            let path = runtime_state_redb_path().expect("runtime redb path must resolve");

            assert_eq!(
                path,
                home.path().join("runtime").join("db").join("main.redb")
            );
        });
    }

    #[test]
    fn persist_and_reload_runtime_state_round_trips() {
        with_test_home("runtime-redb-roundtrip", |_home| {
            let store =
                RedbRuntimeStateStore::open_default().expect("runtime redb store should open");

            let mut state = NodeState::bootstrap();
            state.current_height = 7;
            state.produced_blocks = 7;
            state.last_tx = "smoke".to_string();
            state.consensus.last_round = 7;
            state.consensus.last_message_kind = "block_proposal".to_string();

            store
                .persist_state(&state)
                .expect("runtime state should persist");

            let reloaded = store.load_state().expect("runtime state should reload");

            assert_eq!(reloaded.current_height, 7);
            assert_eq!(reloaded.produced_blocks, 7);
            assert_eq!(reloaded.last_tx, "smoke");
            assert_eq!(reloaded.consensus.last_round, 7);
            assert_eq!(reloaded.consensus.last_message_kind, "block_proposal");
        });
    }

    #[test]
    fn persist_state_rejects_invalid_semantic_payload() {
        with_test_home("runtime-redb-invalid-semantic", |_home| {
            let store =
                RedbRuntimeStateStore::open_default().expect("runtime redb store should open");

            let mut state = NodeState::bootstrap();
            state.current_height = 1;
            state.produced_blocks = 5;

            let error = store
                .persist_state(&state)
                .expect_err("invalid semantic state must fail");

            assert_eq!(error.code(), ErrorCode::NodeStateInvalid.as_str());
        });
    }

    #[test]
    fn load_state_rejects_missing_node_state_record() {
        with_test_home("runtime-redb-missing-record", |_home| {
            let store =
                RedbRuntimeStateStore::open_default().expect("runtime redb store should open");

            let error = store
                .load_state()
                .expect_err("missing node_state record must fail");

            assert_eq!(error.code(), ErrorCode::NodeStateInvalid.as_str());
        });
    }
}
