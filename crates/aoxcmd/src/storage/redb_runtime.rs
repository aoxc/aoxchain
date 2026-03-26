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

fn runtime_state_redb_path() -> Result<PathBuf, AppError> {
    Ok(resolve_home()?.join("runtime").join("state.redb"))
}

fn open_or_create_db(path: &Path) -> Result<Database, AppError> {
    if path.exists() {
        Database::open(path).map_err(|e| {
            AppError::new(
                ErrorCode::FilesystemIoFailed,
                format!(
                    "Failed to open runtime redb database {}: {e}",
                    path.display()
                ),
            )
        })
    } else {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                AppError::with_source(
                    ErrorCode::FilesystemIoFailed,
                    format!("Failed to create runtime redb parent {}", parent.display()),
                    e,
                )
            })?;
        }

        Database::create(path).map_err(|e| {
            AppError::new(
                ErrorCode::FilesystemIoFailed,
                format!(
                    "Failed to create runtime redb database {}: {e}",
                    path.display()
                ),
            )
        })
    }
}

#[derive(Debug)]
pub struct RedbRuntimeStateStore {
    db: Database,
}

impl RedbRuntimeStateStore {
    pub fn open_default() -> Result<Self, AppError> {
        let path = runtime_state_redb_path()?;
        let db = open_or_create_db(&path)?;
        Ok(Self { db })
    }
}

impl RuntimeStateStore for RedbRuntimeStateStore {
    fn load_state(&self) -> Result<NodeState, AppError> {
        let read_txn = self.db.begin_read().map_err(|e| {
            AppError::new(
                ErrorCode::NodeStateInvalid,
                format!("Failed to begin runtime redb read transaction: {e}"),
            )
        })?;

        let table = read_txn.open_table(RUNTIME_STATE_TABLE).map_err(|e| {
            AppError::new(
                ErrorCode::NodeStateInvalid,
                format!("Failed to open runtime redb table: {e}"),
            )
        })?;

        let value = table.get(NODE_STATE_KEY).map_err(|e| {
            AppError::new(
                ErrorCode::NodeStateInvalid,
                format!("Failed to read runtime node state from redb: {e}"),
            )
        })?;

        let Some(value) = value else {
            return Err(AppError::new(
                ErrorCode::NodeStateInvalid,
                "Runtime redb database does not contain node_state",
            ));
        };

        let payload: &str = value.value();
        let state: NodeState = serde_json::from_str(payload).map_err(|e| {
            AppError::with_source(
                ErrorCode::NodeStateInvalid,
                "Failed to decode runtime node state from redb",
                e,
            )
        })?;

        state
            .validate()
            .map_err(|e| AppError::new(ErrorCode::NodeStateInvalid, e))?;

        Ok(state)
    }

    fn persist_state(&self, state: &NodeState) -> Result<(), AppError> {
        let json = serde_json::to_string_pretty(state).map_err(|e| {
            AppError::with_source(
                ErrorCode::OutputEncodingFailed,
                "Failed to encode node state for redb persistence",
                e,
            )
        })?;

        let write_txn = self.db.begin_write().map_err(|e| {
            AppError::new(
                ErrorCode::FilesystemIoFailed,
                format!("Failed to begin runtime redb write transaction: {e}"),
            )
        })?;

        {
            let mut table = write_txn.open_table(RUNTIME_STATE_TABLE).map_err(|e| {
                AppError::new(
                    ErrorCode::FilesystemIoFailed,
                    format!("Failed to open runtime redb table: {e}"),
                )
            })?;

            table.insert(NODE_STATE_KEY, json.as_str()).map_err(|e| {
                AppError::new(
                    ErrorCode::FilesystemIoFailed,
                    format!("Failed to write runtime node state into redb: {e}"),
                )
            })?;
        }

        write_txn.commit().map_err(|e| {
            AppError::new(
                ErrorCode::FilesystemIoFailed,
                format!("Failed to commit runtime redb transaction: {e}"),
            )
        })?;

        Ok(())
    }
}
