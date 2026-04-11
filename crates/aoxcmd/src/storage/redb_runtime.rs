// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use crate::{
    data_home::resolve_home,
    error::{AppError, ErrorCode},
    node::state::NodeState,
    storage::RuntimeStateStore,
};
use redb::{Database, ReadableDatabase, ReadableTable, TableDefinition};
use std::path::{Path, PathBuf};

const RUNTIME_STATE_TABLE: TableDefinition<&str, &str> = TableDefinition::new("runtime_state");
const NODE_STATE_KEY: &str = "node_state";

/// Ensures the canonical runtime-state table exists in the provided database.
///
/// Initialization contract:
/// - The canonical `runtime_state` table must exist before any read-path
///   attempts to open it from a read transaction.
/// - This operation is idempotent and safe to call on every startup.
pub fn ensure_runtime_state_table(db: &Database) -> Result<(), AppError> {
    let write_txn = db.begin_write().map_err(|error| {
        AppError::with_source(
            ErrorCode::FilesystemIoFailed,
            "Failed to begin runtime redb write transaction for table initialization",
            error,
        )
    })?;

    {
        write_txn.open_table(RUNTIME_STATE_TABLE).map_err(|error| {
            AppError::with_source(
                ErrorCode::FilesystemIoFailed,
                "Failed to create or open runtime redb table",
                error,
            )
        })?;
    }

    write_txn.commit().map_err(|error| {
        AppError::with_source(
            ErrorCode::FilesystemIoFailed,
            "Failed to commit runtime redb table initialization transaction",
            error,
        )
    })
}

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
        ensure_runtime_state_table(&db)?;
        Ok(Self { db })
    }

    /// Initializes runtime state with a canonical bootstrap payload when absent.
    ///
    /// Initialization policy:
    /// - Missing `node_state` is initialized with `NodeState::bootstrap()`.
    /// - Existing `node_state` is preserved without overwrite.
    /// - The operation is idempotent and safe across repeated startups.
    ///
    /// Returns:
    /// - `Ok(true)` when bootstrap state was written.
    /// - `Ok(false)` when state already existed.
    pub fn initialize_if_absent(&self) -> Result<bool, AppError> {
        let write_txn = self.db.begin_write().map_err(|error| {
            AppError::with_source(
                ErrorCode::FilesystemIoFailed,
                "Failed to begin runtime redb write transaction for state initialization",
                error,
            )
        })?;

        let initialized = {
            let mut table = write_txn.open_table(RUNTIME_STATE_TABLE).map_err(|error| {
                AppError::with_source(
                    ErrorCode::FilesystemIoFailed,
                    "Failed to open runtime redb table for state initialization",
                    error,
                )
            })?;

            let state_exists = table
                .get(NODE_STATE_KEY)
                .map_err(|error| {
                    AppError::with_source(
                        ErrorCode::FilesystemIoFailed,
                        "Failed to inspect runtime node state during initialization",
                        error,
                    )
                })?
                .is_some();

            if state_exists {
                false
            } else {
                let state = NodeState::bootstrap();
                state
                    .validate()
                    .map_err(|error| AppError::new(ErrorCode::NodeStateInvalid, error))?;

                let payload = serde_json::to_string_pretty(&state).map_err(|error| {
                    AppError::with_source(
                        ErrorCode::OutputEncodingFailed,
                        "Failed to encode bootstrap node state for redb initialization",
                        error,
                    )
                })?;

                table
                    .insert(NODE_STATE_KEY, payload.as_str())
                    .map_err(|error| {
                        AppError::with_source(
                            ErrorCode::FilesystemIoFailed,
                            "Failed to write bootstrap node state during redb initialization",
                            error,
                        )
                    })?;

                true
            }
        };

        write_txn.commit().map_err(|error| {
            AppError::with_source(
                ErrorCode::FilesystemIoFailed,
                "Failed to commit runtime redb state initialization transaction",
                error,
            )
        })?;

        Ok(initialized)
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
    use super::{RUNTIME_STATE_TABLE, RedbRuntimeStateStore, runtime_state_redb_path};
    use crate::{
        error::ErrorCode,
        node::state::NodeState,
        storage::RuntimeStateStore,
        test_support::{AoxcHomeGuard, TestHome, aoxc_home_test_lock},
    };
    use redb::ReadableDatabase;

    fn with_test_home<T>(label: &str, test: impl FnOnce(&TestHome) -> T) -> T {
        let _lock = aoxc_home_test_lock();
        let home = TestHome::new(label).expect("test home should be created");
        let _guard = AoxcHomeGuard::install(&_lock, home.path());
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

    #[test]
    fn open_default_ensures_runtime_state_table_exists() {
        with_test_home("runtime-redb-ensure-table", |_home| {
            let store =
                RedbRuntimeStateStore::open_default().expect("runtime redb store should open");

            let read_txn = store
                .db
                .begin_read()
                .expect("read transaction should open after default initialization");

            read_txn
                .open_table(RUNTIME_STATE_TABLE)
                .expect("runtime_state table should exist after open_default");
        });
    }

    #[test]
    fn initialize_if_absent_bootstraps_once_and_preserves_state() {
        with_test_home("runtime-redb-idempotent-init", |_home| {
            let store =
                RedbRuntimeStateStore::open_default().expect("runtime redb store should open");

            let first = store
                .initialize_if_absent()
                .expect("first initialization should succeed");
            assert!(first, "first initialization must bootstrap node_state");

            let mut state = store
                .load_state()
                .expect("bootstrapped state should be loadable");
            state.current_height = 11;
            state.produced_blocks = 11;
            state.last_tx = "init-idempotent-smoke".to_string();
            state.consensus.last_round = 11;
            state.consensus.last_message_kind = "block_proposal".to_string();
            store
                .persist_state(&state)
                .expect("state mutation should persist");

            let second = store
                .initialize_if_absent()
                .expect("second initialization should remain idempotent");
            assert!(!second, "existing node_state must not be overwritten");

            let reloaded = store
                .load_state()
                .expect("persisted state should remain after idempotent init");
            assert_eq!(reloaded.current_height, 11);
            assert_eq!(reloaded.produced_blocks, 11);
            assert_eq!(reloaded.last_tx, "init-idempotent-smoke");
            assert_eq!(reloaded.consensus.last_round, 11);
            assert_eq!(reloaded.consensus.last_message_kind, "block_proposal");
        });
    }

    #[test]
    fn load_state_rejects_corrupt_record_without_overwrite() {
        with_test_home("runtime-redb-corrupt-record", |_home| {
            let store =
                RedbRuntimeStateStore::open_default().expect("runtime redb store should open");

            let write_txn = store
                .db
                .begin_write()
                .expect("write transaction should open");
            {
                let mut table = write_txn
                    .open_table(RUNTIME_STATE_TABLE)
                    .expect("runtime_state table should open");
                table
                    .insert("node_state", "{not-valid-json")
                    .expect("corrupt payload fixture should persist");
            }
            write_txn
                .commit()
                .expect("corrupt payload fixture commit should succeed");

            let initialized = store
                .initialize_if_absent()
                .expect("initialization should detect existing record");
            assert!(
                !initialized,
                "existing corrupt payload must not be silently overwritten"
            );

            let error = store
                .load_state()
                .expect_err("corrupt payload must be rejected on load");
            assert_eq!(error.code(), ErrorCode::NodeStateInvalid.as_str());
        });
    }
}
