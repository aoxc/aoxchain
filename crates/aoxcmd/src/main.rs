use aoxcmd::keys::{KeyBootstrapRequest, KeyManager, KeyPaths};
use aoxcmd::node::engine::produce_single_block;
use aoxcmd::node::state;

use aoxcdata::{BlockEnvelope, HybridDataStore, IndexBackend};
use aoxcnet::gossip::consensus_gossip::GossipEngine;
use aoxcore::genesis::config::{GenesisConfig, TREASURY_ACCOUNT};
use aoxcore::genesis::loader::GenesisLoader;
use aoxcore::identity::ca::CertificateAuthority;
use aoxcunity::messages::ConsensusMessage;
use aoxcunity::vote::{Vote, VoteKind};

use std::env;
use std::process;

fn main() {
    if let Err(error) = run_cli() {
        eprintln!("AOXCMD_ERROR: {error}");
        process::exit(1);
    }
}

fn run_cli() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_usage();
        return Ok(());
    }

    match args[1].as_str() {
        "help" | "--help" | "-h" => {
            print_usage();
            Ok(())
        }
        "vision" => cmd_vision(),
        "key-bootstrap" => cmd_key_bootstrap(&args[2..]),
        "genesis-init" => cmd_genesis_init(&args[2..]),
        "node-bootstrap" => cmd_node_bootstrap(),
        "produce-once" => cmd_produce_once(&args[2..]),
        "network-smoke" => cmd_network_smoke(),
        "storage-smoke" => cmd_storage_smoke(&args[2..]),
        other => Err(format!("unknown command: {other}")),
    }
}

fn cmd_vision() -> Result<(), String> {
    let output = serde_json::json!({
        "chain_positioning": "interop relay-oriented coordination chain",
        "primary_goal": "cross-chain compatibility and deterministic coordination over raw throughput",
        "execution_strategy": "multi-lane model compatible with heterogeneous external networks",
        "identity_model": "post-quantum capable key/certificate/passport pipeline",
        "consensus_model": "quorum-based proposer/vote/finalization with explicit rotation",
        "status": "pre-mainnet; deterministic local smoke path available"
    });

    println!(
        "{}",
        serde_json::to_string_pretty(&output)
            .map_err(|error| format!("JSON_SERIALIZE_ERROR: {error}"))?
    );

    Ok(())
}

fn cmd_key_bootstrap(args: &[String]) -> Result<(), String> {
    let base_dir = arg_value(args, "--base-dir").unwrap_or_else(|| "AOXC_DATA/keys".to_string());
    let name = arg_value(args, "--name").unwrap_or_else(|| "node".to_string());
    let chain = arg_value(args, "--chain").unwrap_or_else(|| "AOXC-MAIN".to_string());
    let role = arg_value(args, "--role").unwrap_or_else(|| "validator".to_string());
    let zone = arg_value(args, "--zone").unwrap_or_else(|| "core".to_string());
    let issuer = arg_value(args, "--issuer").unwrap_or_else(|| "AOXC-ROOT-CA".to_string());
    let password = arg_value(args, "--password")
        .ok_or_else(|| "--password is required for key-bootstrap".to_string())?;

    let validity_secs: u64 = arg_value(args, "--validity-secs")
        .unwrap_or_else(|| "31536000".to_string())
        .parse()
        .map_err(|_| "--validity-secs must be a valid u64".to_string())?;

    let paths = KeyPaths::new(base_dir, &name);
    let request = KeyBootstrapRequest::new(chain, role, zone, password, validity_secs);
    let manager = KeyManager::new(paths, request);
    let ca = CertificateAuthority::new(issuer);

    let material = manager
        .load_or_create(&ca)
        .map_err(|error| format!("key bootstrap failed [{}]: {}", error.code(), error))?;

    println!(
        "{}",
        serde_json::to_string_pretty(&material.summary())
            .map_err(|error| format!("JSON_SERIALIZE_ERROR: {error}"))?
    );

    Ok(())
}

fn cmd_genesis_init(args: &[String]) -> Result<(), String> {
    let path =
        arg_value(args, "--path").unwrap_or_else(|| "AOXC_DATA/identity/genesis.json".to_string());
    let chain_num: u32 = arg_value(args, "--chain-num")
        .unwrap_or_else(|| "1".to_string())
        .parse()
        .map_err(|_| "--chain-num must be a valid u32".to_string())?;
    let block_time: u64 = arg_value(args, "--block-time")
        .unwrap_or_else(|| "6".to_string())
        .parse()
        .map_err(|_| "--block-time must be a valid u64".to_string())?;
    let treasury: u128 = arg_value(args, "--treasury")
        .unwrap_or_else(|| "1000000000".to_string())
        .parse()
        .map_err(|_| "--treasury must be a valid u128".to_string())?;

    let mut config = GenesisConfig::new();
    config.chain_num = chain_num;
    config.chain_id = GenesisConfig::generate_chain_id(chain_num);
    config.block_time = block_time;
    config.treasury = treasury;
    config.add_account(TREASURY_ACCOUNT.to_string(), treasury);

    config.validate()?;
    GenesisLoader::save(&config, &path).map_err(|error| error.to_string())?;

    let loaded = GenesisLoader::load(&path).map_err(|error| error.to_string())?;

    let output = serde_json::json!({
        "saved_path": path,
        "chain_num": loaded.config.chain_num,
        "chain_id": loaded.config.chain_id,
        "block_time": loaded.config.block_time,
        "treasury": loaded.config.treasury,
        "total_supply": loaded.config.total_supply(),
        "state_hash": loaded.config.state_hash()
    });

    println!(
        "{}",
        serde_json::to_string_pretty(&output)
            .map_err(|error| format!("JSON_SERIALIZE_ERROR: {error}"))?
    );

    Ok(())
}

fn cmd_node_bootstrap() -> Result<(), String> {
    let node = state::setup().map_err(|error| error.to_string())?;

    let output = serde_json::json!({
        "mempool_max_txs": node.mempool.config().max_txs,
        "mempool_max_tx_size": node.mempool.config().max_tx_size,
        "validator_count": node.rotation.validators().len(),
        "quorum": {
            "numerator": node.consensus.quorum.numerator,
            "denominator": node.consensus.quorum.denominator
        }
    });

    println!(
        "{}",
        serde_json::to_string_pretty(&output)
            .map_err(|error| format!("JSON_SERIALIZE_ERROR: {error}"))?
    );

    Ok(())
}

fn cmd_produce_once(args: &[String]) -> Result<(), String> {
    let tx = arg_value(args, "--tx").unwrap_or_else(|| "AOXC_RELAY_DEMO_TX".to_string());

    let mut node = state::setup().map_err(|error| error.to_string())?;
    let outcome = produce_single_block(&mut node, vec![tx.into_bytes()])?;

    let output = serde_json::json!({
        "height": outcome.block.header.height,
        "hash": hex::encode(outcome.block.hash),
        "parent": hex::encode(outcome.block.header.parent_hash),
        "finalized": outcome.seal.is_some(),
        "seal": outcome.seal,
    });

    println!(
        "{}",
        serde_json::to_string_pretty(&output)
            .map_err(|error| format!("JSON_SERIALIZE_ERROR: {error}"))?
    );

    Ok(())
}

fn cmd_network_smoke() -> Result<(), String> {
    let mut gossip = GossipEngine::new();

    let vote = Vote {
        voter: [7u8; 32],
        block_hash: [9u8; 32],
        height: 1,
        round: 0,
        kind: VoteKind::Prepare,
    };

    gossip.broadcast(ConsensusMessage::Vote(vote));
    let inbound = gossip.receive();

    let output = serde_json::json!({
        "transport": "stub",
        "broadcast": "ok",
        "inbound_message": inbound,
        "note": "None is expected until p2p transport integration is implemented"
    });

    println!(
        "{}",
        serde_json::to_string_pretty(&output)
            .map_err(|error| format!("JSON_SERIALIZE_ERROR: {error}"))?
    );

    Ok(())
}

fn cmd_storage_smoke(args: &[String]) -> Result<(), String> {
    let base_dir = arg_value(args, "--base-dir").unwrap_or_else(|| "AOXC_DATA/storage".to_string());
    let backend = arg_value(args, "--index").unwrap_or_else(|| "sqlite".to_string());

    let index_backend = match backend.as_str() {
        "sqlite" => IndexBackend::Sqlite,
        "redb" => IndexBackend::Redb,
        other => {
            return Err(format!(
                "unsupported --index backend: {other}, expected sqlite|redb"
            ));
        }
    };

    let store = HybridDataStore::new(&base_dir, index_backend).map_err(|e| e.to_string())?;

    let block = BlockEnvelope {
        height: 1,
        block_hash_hex: "aa".repeat(32),
        parent_hash_hex: "00".repeat(32),
        payload: b"aoxc-relay-ipfs-block".to_vec(),
    };

    let meta = store.put_block(&block).map_err(|e| e.to_string())?;
    let loaded = store.get_block_by_height(1).map_err(|e| e.to_string())?;

    let output = serde_json::json!({
        "base_dir": base_dir,
        "index_backend": backend,
        "cid": meta.cid,
        "height": loaded.height,
        "payload_len": loaded.payload.len(),
        "storage_policy": {
            "block_body": "ipfs(ipld-compatible content addressing)",
            "state_index": "sqlite_or_redb"
        }
    });

    println!(
        "{}",
        serde_json::to_string_pretty(&output)
            .map_err(|error| format!("JSON_SERIALIZE_ERROR: {error}"))?
    );

    Ok(())
}

fn arg_value(args: &[String], key: &str) -> Option<String> {
    args.windows(2).find_map(|window| {
        if window[0] == key {
            Some(window[1].clone())
        } else {
            None
        }
    })
}

fn print_usage() {
    println!(
        "AOXC Command Surface\n\nCommands:\n  vision\n  key-bootstrap --password <secret> [--base-dir <dir>] [--name <name>] [--chain <id>] [--role <role>] [--zone <zone>] [--issuer <issuer>] [--validity-secs <u64>]\n  genesis-init [--path <file>] [--chain-num <u32>] [--block-time <u64>] [--treasury <u128>]\n  node-bootstrap\n  produce-once [--tx <payload>]\n  network-smoke\n  storage-smoke [--base-dir <dir>] [--index sqlite|redb]\n  help\n"
    );
}
