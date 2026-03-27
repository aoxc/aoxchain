// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use crate::{
    cli_support::{arg_value, emit_serialized, output_format},
    data_home::{ensure_layout, resolve_home},
    error::{AppError, ErrorCode},
};
use aoxcdata::{BlockEnvelope, HybridDataStore, IndexBackend};
use serde::Serialize;
use std::{fs, path::PathBuf};

fn parse_backend(args: &[String]) -> Result<IndexBackend, AppError> {
    match arg_value(args, "--backend")
        .unwrap_or_else(|| "sqlite".to_string())
        .to_ascii_lowercase()
        .as_str()
    {
        "sqlite" => Ok(IndexBackend::Sqlite),
        "redb" => Ok(IndexBackend::Redb),
        value => Err(AppError::new(
            ErrorCode::UsageInvalidArguments,
            format!("Invalid --backend value '{value}'. Use sqlite|redb."),
        )),
    }
}

fn db_root() -> Result<PathBuf, AppError> {
    let home = resolve_home()?;
    ensure_layout(&home)?;
    Ok(home.join("runtime").join("db"))
}

fn open_store(backend: IndexBackend) -> Result<(HybridDataStore, PathBuf), AppError> {
    let root = db_root()?;
    let store = HybridDataStore::new(&root, backend).map_err(|err| {
        AppError::with_source(
            ErrorCode::FilesystemIoFailed,
            format!("Failed to open data store at {}", root.display()),
            err,
        )
    })?;
    Ok((store, root))
}

fn parse_u64_arg(args: &[String], flag: &str) -> Result<u64, AppError> {
    arg_value(args, flag)
        .ok_or_else(|| {
            AppError::new(
                ErrorCode::UsageInvalidArguments,
                format!("Missing required {flag} value"),
            )
        })?
        .parse::<u64>()
        .map_err(|_| {
            AppError::new(
                ErrorCode::UsageInvalidArguments,
                format!("Invalid numeric value for {flag}"),
            )
        })
}

fn parse_required_arg(args: &[String], flag: &str) -> Result<String, AppError> {
    arg_value(args, flag).ok_or_else(|| {
        AppError::new(
            ErrorCode::UsageInvalidArguments,
            format!("Missing required {flag} value"),
        )
    })
}

#[derive(Debug, Serialize)]
struct DbStatus {
    backend: String,
    db_root: String,
    ipfs_object_count: usize,
    has_index_snapshot: bool,
    has_index_journal: bool,
}

pub fn cmd_db_init(args: &[String]) -> Result<(), AppError> {
    let backend = parse_backend(args)?;
    let (_, root) = open_store(backend)?;
    let status = DbStatus {
        backend: format!("{backend:?}").to_ascii_lowercase(),
        db_root: root.display().to_string(),
        ipfs_object_count: count_ipfs_objects(&root)?,
        has_index_snapshot: root.join("index").join("snapshot.json").exists(),
        has_index_journal: root.join("index").join("journal.log").exists(),
    };
    emit_serialized(&status, output_format(args))
}

pub fn cmd_db_status(args: &[String]) -> Result<(), AppError> {
    let backend = parse_backend(args)?;
    let (_, root) = open_store(backend)?;
    let status = DbStatus {
        backend: format!("{backend:?}").to_ascii_lowercase(),
        db_root: root.display().to_string(),
        ipfs_object_count: count_ipfs_objects(&root)?,
        has_index_snapshot: root.join("index").join("snapshot.json").exists(),
        has_index_journal: root.join("index").join("journal.log").exists(),
    };
    emit_serialized(&status, output_format(args))
}

pub fn cmd_db_put_block(args: &[String]) -> Result<(), AppError> {
    let backend = parse_backend(args)?;
    let block_file = parse_required_arg(args, "--block-file")?;
    let encoded = fs::read_to_string(&block_file).map_err(|err| {
        AppError::with_source(
            ErrorCode::FilesystemIoFailed,
            format!("Failed to read block file {block_file}"),
            err,
        )
    })?;
    let block: BlockEnvelope = serde_json::from_str(&encoded).map_err(|err| {
        AppError::with_source(
            ErrorCode::UsageInvalidArguments,
            format!("Invalid block JSON at {block_file}"),
            err,
        )
    })?;

    let (store, root) = open_store(backend)?;
    let meta = store.put_block(&block).map_err(|err| {
        AppError::with_source(
            ErrorCode::LedgerInvalid,
            format!("Failed to persist block into {}", root.display()),
            err,
        )
    })?;
    emit_serialized(&meta, output_format(args))
}

pub fn cmd_db_get_height(args: &[String]) -> Result<(), AppError> {
    let backend = parse_backend(args)?;
    let height = parse_u64_arg(args, "--height")?;
    let (store, root) = open_store(backend)?;
    let block = store.get_block_by_height(height).map_err(|err| {
        AppError::with_source(
            ErrorCode::LedgerInvalid,
            format!(
                "Failed to load block at height {height} from {}",
                root.display()
            ),
            err,
        )
    })?;
    emit_serialized(&block, output_format(args))
}

pub fn cmd_db_get_hash(args: &[String]) -> Result<(), AppError> {
    let backend = parse_backend(args)?;
    let hash = parse_required_arg(args, "--hash")?;
    let (store, root) = open_store(backend)?;
    let block = store.get_block_by_hash(&hash).map_err(|err| {
        AppError::with_source(
            ErrorCode::LedgerInvalid,
            format!("Failed to load block hash {hash} from {}", root.display()),
            err,
        )
    })?;
    emit_serialized(&block, output_format(args))
}

pub fn cmd_db_compact(args: &[String]) -> Result<(), AppError> {
    let backend = parse_backend(args)?;
    let (store, root) = open_store(backend)?;
    store.compact_index().map_err(|err| {
        AppError::with_source(
            ErrorCode::LedgerInvalid,
            format!("Failed to compact metadata index at {}", root.display()),
            err,
        )
    })?;
    cmd_db_status(args)
}

fn count_ipfs_objects(root: &std::path::Path) -> Result<usize, AppError> {
    let ipfs_dir = root.join("ipfs");
    if !ipfs_dir.exists() {
        return Ok(0);
    }

    let entries = fs::read_dir(&ipfs_dir).map_err(|err| {
        AppError::with_source(
            ErrorCode::FilesystemIoFailed,
            format!("Failed to scan {}", ipfs_dir.display()),
            err,
        )
    })?;
    Ok(entries.filter_map(Result::ok).count())
}

#[cfg(test)]
mod tests {
    use super::{cmd_db_get_hash, cmd_db_get_height, cmd_db_init, cmd_db_put_block, parse_backend};
    use crate::test_support::TestHome;
    use sha2::{Digest, Sha256};
    use std::fs;

    fn sample_hash(height: u64, parent_hash: &str, payload: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(b"AOXC_BLOCK_V1");
        hasher.update(height.to_le_bytes());
        hasher.update(hex::decode(parent_hash).expect("parent hash should decode"));
        hasher.update((payload.len() as u64).to_le_bytes());
        hasher.update(payload);
        hex::encode(hasher.finalize())
    }

    #[test]
    fn backend_parser_accepts_known_values() {
        let args = vec!["--backend".to_string(), "redb".to_string()];
        assert_eq!(
            format!("{:?}", parse_backend(&args).expect("backend should parse")),
            "Redb"
        );
    }

    #[test]
    fn db_roundtrip_flow_with_cli_commands() {
        let home = TestHome::new("db-roundtrip");
        let parent = "00".repeat(32);
        let payload = b"{\"tx\":\"db-smoke\"}";
        let block_hash = sample_hash(1, &parent, payload);
        let block_file = home.path().join("support").join("sample-block.json");
        let block_json = format!(
            "{{\"height\":1,\"block_hash_hex\":\"{block_hash}\",\"parent_hash_hex\":\"{parent}\",\"payload\":[123,34,116,120,34,58,34,100,98,45,115,109,111,107,101,34,125]}}"
        );
        fs::create_dir_all(
            block_file
                .parent()
                .expect("sample block parent directory must exist"),
        )
        .expect("sample block directory should be created");
        fs::write(&block_file, block_json).expect("sample block should be written");

        cmd_db_init(&[]).expect("db init should succeed");
        cmd_db_put_block(&["--block-file".to_string(), block_file.display().to_string()])
            .expect("db put should succeed");
        cmd_db_get_height(&["--height".to_string(), "1".to_string()])
            .expect("db get by height should succeed");
        cmd_db_get_hash(&["--hash".to_string(), block_hash])
            .expect("db get by hash should succeed");
    }
}
