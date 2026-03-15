use std::collections::HashSet;
use std::fs::{self, OpenOptions};
use std::io;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use crate::logging::init::{self, ChainContext, LogLevel};
use crate::node::state::AOXCNode;

use aoxcunity::block_builder::BlockBuilder;
use aoxcunity::fork_choice::BlockMeta;
use aoxcunity::messages::ConsensusMessage;

const ARCHIVE_ROOT: &str = "AOXC_DATA/db/blocks";

/// AOXC primary node execution loop.
///
/// Security and operational objectives:
/// - preserve chain continuity by linking newly proposed blocks to the latest known parent hash,
/// - prevent silent arithmetic overflow during height progression,
/// - avoid duplicate handling of externally observed finality broadcasts,
/// - persist archival records using a temporary-file and rename pattern,
/// - keep runtime flow observable and deterministic under normal operating conditions.
///
/// Operational note:
/// This function is intentionally long-running and blocking by design.
///
/// Current implementation note:
/// The consensus crate currently exposes vote-pool and round state, but does not
/// yet expose a public finalization facade such as `finalize(...)` or
/// `prune_votes(...)`. This runtime therefore performs proposal production,
/// gossip handling, archival persistence, and vote admission, while treating
/// finality broadcasts as externally observed signals.
pub fn run(node: &mut AOXCNode) {
    const BLOCK_TIME_SECS: u64 = 6;
    const MEMPOOL_BATCH_LIMIT: usize = 100;
    const LOOP_SLEEP_MILLIS: u64 = 50;

    let block_time = Duration::from_secs(BLOCK_TIME_SECS);
    let mut current_height: u64 = 0;
    let mut last_block_hash = [0u8; 32];
    let mut last_production = Instant::now();

    // Tracks block hashes that were already observed as finalized from the
    // network. This prevents duplicate processing and duplicate operator noise.
    let mut finalized_heads: HashSet<[u8; 32]> = HashSet::new();

    init::log(
        LogLevel::INFO,
        "ENGINE",
        None,
        "Consensus engine is active. Block target: 6s",
    );

    init::log(
        LogLevel::TRACE,
        "ENGINE",
        None,
        "Execution engine bootstrap completed. Entering main scheduling loop.",
    );

    loop {
        // -----------------------------------------------------------------
        // Layer 1: Proposer Layer
        // Produces a candidate block when the configured production interval
        // has elapsed and a proposer is available for the next height.
        // -----------------------------------------------------------------
        if last_production.elapsed() >= block_time {
            let next_height = match current_height.checked_add(1) {
                Some(value) => value,
                None => {
                    init::log(
                        LogLevel::ERROR,
                        "ENGINE",
                        None,
                        "Height counter overflow detected. Engine will halt to prevent undefined chain state.",
                    );
                    break;
                }
            };

            if let Some(proposer_id) = node.rotation.proposer(next_height) {
                let collected = node.mempool.collect(MEMPOOL_BATCH_LIMIT);

                if collected.is_empty() {
                    let idle_msg = format!(
                        "Height candidate {} skipped because the mempool is empty.",
                        next_height
                    );
                    init::log(LogLevel::DEBUG, "IDLE", None, &idle_msg);
                } else {
                    let parent_hash = last_block_hash;

                    // The block builder accepts raw transaction payloads, while the
                    // mempool returns structured collection records. This boundary
                    // intentionally extracts only payload bytes for block assembly.
                    let txs: Vec<Vec<u8>> = collected.into_iter().map(|tx| tx.payload).collect();

                    let block = match BlockBuilder::build_block(parent_hash, next_height, txs) {
                        Ok(block) => block,
                        Err(error) => {
                            let build_error_msg = format!(
                                "Block construction failed at scheduled height {}: {}",
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
                        proposer_id,
                        short_hash(&parent_hash),
                        short_hash(&block_hash)
                    );

                    init::log(LogLevel::INFO, "PROPOSER", Some(&context), &proposal_msg);

                    if let Err(error) = archive_block(current_height, parent_hash, block_hash) {
                        let storage_msg = format!(
                            "Block archival failed for height {} and hash {}: {}",
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

                    node.gossip
                        .broadcast(ConsensusMessage::BlockProposal { block_hash });
                }
            } else {
                let warning_msg = format!(
                    "No eligible proposer returned by rotation for scheduled height {}. Production was skipped.",
                    next_height
                );
                init::log(LogLevel::WARN, "PROPOSER", None, &warning_msg);
            }

            last_production = Instant::now();
        }

        // -----------------------------------------------------------------
        // Layer 2: Network and Validation Layer
        // Consumes gossip messages and updates local consensus-related state.
        // -----------------------------------------------------------------
        while let Some(message) = node.gossip.receive() {
            match message {
                ConsensusMessage::BlockProposal { block_hash } => {
                    let context = chain_context(0, current_height, block_hash);
                    let msg = format!(
                        "Candidate block received from network: {}",
                        short_hash(&block_hash)
                    );
                    init::log(LogLevel::INFO, "NETWORK", Some(&context), &msg);
                }
                ConsensusMessage::Vote(vote) => match node.consensus.vote_pool.add_vote(vote) {
                    Ok(()) => {
                        init::log(
                            LogLevel::TRACE,
                            "NETWORK",
                            None,
                            "Consensus vote received from gossip layer and admitted into the vote pool.",
                        );
                    }
                    Err(error) => {
                        let msg = format!("Consensus vote rejected: {}", error);
                        init::log(LogLevel::WARN, "NETWORK", None, &msg);
                    }
                },
                ConsensusMessage::Finalize { block_hash } => {
                    let context = chain_context(0, current_height, block_hash);

                    if finalized_heads.insert(block_hash) {
                        let msg = format!(
                            "Finality broadcast observed for block {}",
                            short_hash(&block_hash)
                        );
                        init::log(LogLevel::INFO, "NETWORK", Some(&context), &msg);
                    } else {
                        let msg = format!(
                            "Duplicate finality broadcast ignored for block {}",
                            short_hash(&block_hash)
                        );
                        init::log(LogLevel::WARN, "NETWORK", Some(&context), &msg);
                    }
                }
            }
        }

        // -----------------------------------------------------------------
        // Layer 3: Consensus State Visibility
        // Exposes a lightweight runtime trace for operators without invoking
        // a finalization API that is not yet provided by the consensus crate.
        // -----------------------------------------------------------------
        if let Some(head) = node.fork_choice.get_head() {
            let context = chain_context(0, current_height, head);
            let round = node.consensus.round.round;

            if finalized_heads.contains(&head) {
                let msg = format!(
                    "Head {} is already marked as finalized by an observed network signal.",
                    short_hash(&head)
                );
                init::log(LogLevel::TRACE, "FINALITY", Some(&context), &msg);
            } else {
                let msg = format!(
                    "Head {} remains non-finalized at local round {} because no public finalization facade is currently exposed by the consensus crate.",
                    short_hash(&head),
                    round
                );
                init::log(LogLevel::TRACE, "FINALITY", Some(&context), &msg);
            }
        }

        // -----------------------------------------------------------------
        // Maintenance Layer
        // Yields execution time to avoid a hot loop.
        // -----------------------------------------------------------------
        thread::sleep(Duration::from_millis(LOOP_SLEEP_MILLIS));
    }
}

/// Persists a block archive entry beneath `AOXC_DATA/db/blocks`.
///
/// Security properties:
/// - ensures the target directory exists before any write operation,
/// - writes to a temporary file and renames it into place,
/// - flushes and synchronizes file contents before rename,
/// - stores wall-clock UNIX time suitable for durable records.
///
/// Returns:
/// - `Ok(())` if the archive entry is successfully written,
/// - `Err(io::Error)` if directory creation, write, sync, or rename fails.
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

/// Builds a deterministic temporary archive file path for atomic replacement.
fn temporary_archive_path(base_dir: &Path, height: u64) -> PathBuf {
    base_dir.join(format!(".height_{}.data.tmp", height))
}

/// Returns a UNIX timestamp in seconds.
///
/// This function uses wall-clock time rather than monotonic process time,
/// making it suitable for durable archival records.
fn current_unix_timestamp_secs() -> io::Result<u64> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .map_err(|error| io::Error::other(format!("system time error: {}", error)))
}

/// Builds a logging context for the current chain state.
fn chain_context(era: u64, block: u64, hash: [u8; 32]) -> ChainContext {
    ChainContext {
        era,
        block,
        block_hash: hex_hash(&hash),
    }
}

/// Converts a block hash to a full lowercase hexadecimal string.
fn hex_hash(hash: &[u8; 32]) -> String {
    let mut out = String::with_capacity(64);

    for byte in hash {
        use std::fmt::Write as _;
        let _ = write!(&mut out, "{:02x}", byte);
    }

    out
}

/// Produces a short, operator-friendly display form of a block hash.
fn short_hash(hash: &[u8; 32]) -> String {
    let full = hex_hash(hash);
    full[..8].to_string()
}

