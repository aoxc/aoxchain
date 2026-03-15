use std::fs::{self, OpenOptions};
use std::io;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use crate::logging::init::{self, ChainContext, LogLevel};
use crate::node::state::AOXCNode;

use aoxcunity::block::{BlockBody, BlockBuilder};
use aoxcunity::fork_choice::BlockMeta;

const ARCHIVE_ROOT: &str = "AOXC_DATA/db/blocks";

pub fn run(node: &mut AOXCNode) {
    const BLOCK_TIME_SECS: u64 = 6;
    const MEMPOOL_BATCH_LIMIT: usize = 100;
    const LOOP_SLEEP_MILLIS: u64 = 50;

    let block_time = Duration::from_secs(BLOCK_TIME_SECS);
    let mut current_height: u64 = 0;
    let mut last_block_hash = [0u8; 32];
    let mut last_production = Instant::now();

    init::log(
        LogLevel::INFO,
        "ENGINE",
        None,
        "Consensus engine is active. Block target: 6s",
    );

    loop {
        if last_production.elapsed() >= block_time {
            let next_height = match current_height.checked_add(1) {
                Some(value) => value,
                None => {
                    init::log(
                        LogLevel::ERROR,
                        "ENGINE",
                        None,
                        "Height overflow detected, stopping engine.",
                    );
                    break;
                }
            };

            if let Some(proposer_id) = node.rotation.proposer(next_height) {
                let collected = node.mempool.collect(MEMPOOL_BATCH_LIMIT);

                if collected.is_empty() {
                    let idle_msg = format!("Height {} skipped: mempool empty", next_height);
                    init::log(LogLevel::DEBUG, "IDLE", None, &idle_msg);
                } else {
                    let parent_hash = last_block_hash;
                    let timestamp = match current_unix_timestamp_secs() {
                        Ok(ts) => ts,
                        Err(error) => {
                            let msg = format!("Cannot obtain timestamp: {}", error);
                            init::log(LogLevel::ERROR, "PROPOSER", None, &msg);
                            last_production = Instant::now();
                            continue;
                        }
                    };

                    let block = match BlockBuilder::build(
                        0,
                        parent_hash,
                        next_height,
                        0,
                        node.consensus.round.round,
                        timestamp,
                        proposer_id,
                        BlockBody::default(),
                    ) {
                        Ok(block) => block,
                        Err(error) => {
                            let build_error_msg = format!(
                                "Block construction failed at height {}: {}",
                                next_height, error
                            );
                            init::log(LogLevel::ERROR, "PROPOSER", None, &build_error_msg);
                            last_production = Instant::now();
                            continue;
                        }
                    };

                    let block_hash = block.hash;
                    current_height = next_height;
                    last_block_hash = block_hash;

                    let context = chain_context(0, current_height, block_hash);
                    let proposal_msg = format!(
                        "New block proposed by {} | Parent: {} | Hash: {}",
                        short_hash(&proposer_id),
                        short_hash(&parent_hash),
                        short_hash(&block_hash)
                    );
                    init::log(LogLevel::INFO, "PROPOSER", Some(&context), &proposal_msg);

                    if let Err(error) = archive_block(current_height, parent_hash, block_hash) {
                        let storage_msg = format!(
                            "Block archival failed for height {} hash {}: {}",
                            current_height,
                            short_hash(&block_hash),
                            error
                        );
                        init::log(LogLevel::ERROR, "STORAGE", Some(&context), &storage_msg);
                    }

                    node.fork_choice.insert_block(BlockMeta {
                        hash: block_hash,
                        parent: parent_hash,
                        height: current_height,
                        seal: None,
                    });
                }
            }

            last_production = Instant::now();
        }

        if let Some(head) = node.fork_choice.get_head() {
            let context = chain_context(0, current_height, head);
            let msg = format!(
                "Head {} at round {} (finalization wired in consensus state)",
                short_hash(&head),
                node.consensus.round.round
            );
            init::log(LogLevel::TRACE, "FINALITY", Some(&context), &msg);
        }

        thread::sleep(Duration::from_millis(LOOP_SLEEP_MILLIS));
    }
}

fn archive_block(height: u64, parent: [u8; 32], hash: [u8; 32]) -> io::Result<()> {
    let base_dir = Path::new(ARCHIVE_ROOT);
    fs::create_dir_all(base_dir)?;

    let final_path = base_dir.join(format!("height_{}.data", height));
    let temp_path = temporary_archive_path(base_dir, height);
    let unix_ts = current_unix_timestamp_secs()?;

    let data = format!(
        "BlockHeight: {}\nParentHash: {}\nHash: {}\nTimestampUnix: {}\n",
        height,
        hex_hash(&parent),
        hex_hash(&hash),
        unix_ts
    );

    fs::write(&temp_path, data)?;

    let file = OpenOptions::new().read(true).write(true).open(&temp_path)?;
    file.sync_all()?;
    drop(file);

    fs::rename(&temp_path, &final_path)?;
    Ok(())
}

fn temporary_archive_path(base_dir: &Path, height: u64) -> PathBuf {
    base_dir.join(format!(".height_{}.data.tmp", height))
}

fn current_unix_timestamp_secs() -> io::Result<u64> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .map_err(|error| io::Error::other(format!("system time error: {}", error)))
}

fn chain_context(era: u64, block: u64, hash: [u8; 32]) -> ChainContext {
    ChainContext {
        era,
        block,
        block_hash: hex_hash(&hash),
    }
}

fn hex_hash(hash: &[u8; 32]) -> String {
    let mut out = String::with_capacity(64);

    for byte in hash {
        use std::fmt::Write as _;
        let _ = write!(&mut out, "{:02x}", byte);
    }

    out
}

fn short_hash(hash: &[u8; 32]) -> String {
    let full = hex_hash(hash);
    full[..8].to_string()
}
