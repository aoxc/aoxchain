use aoxcmd::build_info::BuildInfo;
use aoxcmd::economy::ledger::EconomyState;
use aoxcmd::keys::{KeyBootstrapRequest, KeyManager, KeyPaths};
use aoxcmd::node::engine::produce_single_block;
use aoxcmd::node::state;
use aoxcmd::telemetry::prometheus::MetricsSnapshot;
use aoxcmd::telemetry::tracing::TraceProfile;

use aoxcdata::{BlockEnvelope, HybridDataStore, IndexBackend};
use aoxcnet::gossip::consensus_gossip::GossipEngine;
use aoxcnet::gossip::peer::{NodeCertificate, Peer};
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
        "version" | "--version" | "-V" => cmd_version(),
        "help" | "--help" | "-h" => {
            print_usage();
            Ok(())
        }
        "vision" => cmd_vision(),
        "compat-matrix" => cmd_compat_matrix(),
        "key-bootstrap" => cmd_key_bootstrap(&args[2..]),
        "genesis-init" => cmd_genesis_init(&args[2..]),
        "node-bootstrap" => cmd_node_bootstrap(),
        "produce-once" => cmd_produce_once(&args[2..]),
        "network-smoke" => cmd_network_smoke(),
        "storage-smoke" => cmd_storage_smoke(&args[2..]),
        "economy-init" => cmd_economy_init(&args[2..]),
        "treasury-transfer" => cmd_treasury_transfer(&args[2..]),
        "stake-delegate" => cmd_stake_delegate(&args[2..]),
        "stake-undelegate" => cmd_stake_undelegate(&args[2..]),
        "economy-status" => cmd_economy_status(&args[2..]),
        "runtime-status" => cmd_runtime_status(&args[2..]),
        other => Err(format!("unknown command: {other}")),
    }
}

fn cmd_version() -> Result<(), String> {
    let build = BuildInfo::collect();
    let output = serde_json::json!({
        "name": "aoxc",
        "name": "aoxcmd",
        "version": build.semver,
        "git_commit": build.git_commit,
        "git_dirty": build.git_dirty,
        "source_date_epoch": build.source_date_epoch,
        "embedded_cert": {
            "path": build.cert_path,
            "sha256": build.cert_sha256,
            "error": build.cert_error,
        }
    });

    println!(
        "{}",
        serde_json::to_string_pretty(&output)
            .map_err(|error| format!("JSON_SERIALIZE_ERROR: {error}"))?
    );

    Ok(())
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

fn cmd_compat_matrix() -> Result<(), String> {
    let output = serde_json::json!({
        "execution_lanes": ["EVM", "WASM", "Sui Move", "Cardano UTXO"],
        "network_surface": ["Gossip", "Discovery", "Sync", "RPC"],
        "transport_profiles": ["TCP", "UDP", "QUIC"],
        "compatibility": {
            "evm_chains": "bridge-compatible via aoxcvm::lanes::evm",
            "wasm_chains": "bridge-compatible via aoxcvm::lanes::wasm",
            "move_ecosystem": "bridge-compatible via aoxcvm::lanes::sui_move",
            "utxo_ecosystem": "bridge-compatible via aoxcvm::lanes::cardano"
        },
        "note": "Deterministic coordination is implemented; production interoperability requires chain-specific bridge adapters and audits."
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

    let cert = NodeCertificate {
        subject: "validator-1".to_string(),
        issuer: "AOXC-ROOT-CA".to_string(),
        valid_from_unix: 1,
        valid_until_unix: u64::MAX,
        serial: "validator-1-serial".to_string(),
    };
    let peer = Peer::new("validator-1", "127.0.0.1:26656", cert);

    gossip
        .register_peer(peer)
        .map_err(|error| format!("NETWORK_PEER_REGISTER_ERROR: {error}"))?;
    gossip
        .establish_session("validator-1")
        .map_err(|error| format!("NETWORK_SESSION_ERROR: {error}"))?;

    let vote = Vote {
        voter: [7u8; 32],
        block_hash: [9u8; 32],
        height: 1,
        round: 0,
        kind: VoteKind::Prepare,
    };

    gossip
        .broadcast_from_peer("validator-1", ConsensusMessage::Vote(vote))
        .map_err(|error| format!("NETWORK_BROADCAST_ERROR: {error}"))?;

    let inbound = gossip.receive();
    let (peer_count, session_count) = gossip.stats();

    let output = serde_json::json!({
        "transport": "in-memory-secure-shell",
        "security": "mutual-auth-session-gated",
        "registered_peers": peer_count,
        "active_sessions": session_count,
        "broadcast": "ok",
        "inbound_message": inbound,
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

fn cmd_economy_init(args: &[String]) -> Result<(), String> {
    let state_path =
        arg_value(args, "--state").unwrap_or_else(|| "AOXC_DATA/economy/state.json".to_string());
    let treasury_supply: u128 = arg_value(args, "--treasury-supply")
        .unwrap_or_else(|| "1000000000000".to_string())
        .parse()
        .map_err(|_| "--treasury-supply must be a valid u128".to_string())?;

    let mut state = EconomyState::default();
    state.mint_to_treasury(treasury_supply);
    state.save(&state_path)?;

    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "state_path": state_path,
            "treasury_account": state.treasury_account,
            "treasury_balance": state.treasury_balance(),
            "total_staked": state.total_staked(),
        }))
        .map_err(|e| format!("JSON_SERIALIZE_ERROR: {e}"))?
    );

    Ok(())
}

fn cmd_treasury_transfer(args: &[String]) -> Result<(), String> {
    let state_path =
        arg_value(args, "--state").unwrap_or_else(|| "AOXC_DATA/economy/state.json".to_string());
    let to = arg_value(args, "--to").ok_or_else(|| "--to is required".to_string())?;
    let amount: u128 = arg_value(args, "--amount")
        .ok_or_else(|| "--amount is required".to_string())?
        .parse()
        .map_err(|_| "--amount must be a valid u128".to_string())?;

    let mut state = EconomyState::load_or_default(&state_path)?;
    let treasury = state.treasury_account.clone();
    state.transfer(&treasury, &to, amount)?;
    state.save(&state_path)?;

    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "state_path": state_path,
            "to": to,
            "amount": amount,
            "treasury_balance": state.treasury_balance(),
            "recipient_balance": state.balances.get(&to).copied().unwrap_or_default(),
        }))
        .map_err(|e| format!("JSON_SERIALIZE_ERROR: {e}"))?
    );

    Ok(())
}

fn cmd_stake_delegate(args: &[String]) -> Result<(), String> {
    let state_path =
        arg_value(args, "--state").unwrap_or_else(|| "AOXC_DATA/economy/state.json".to_string());
    let staker = arg_value(args, "--staker").ok_or_else(|| "--staker is required".to_string())?;
    let validator =
        arg_value(args, "--validator").ok_or_else(|| "--validator is required".to_string())?;
    let amount: u128 = arg_value(args, "--amount")
        .ok_or_else(|| "--amount is required".to_string())?
        .parse()
        .map_err(|_| "--amount must be a valid u128".to_string())?;

    let mut state = EconomyState::load_or_default(&state_path)?;
    state.delegate(&staker, &validator, amount)?;
    state.save(&state_path)?;

    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "state_path": state_path,
            "staker": staker,
            "validator": validator,
            "delegated_amount": amount,
            "total_staked": state.total_staked(),
        }))
        .map_err(|e| format!("JSON_SERIALIZE_ERROR: {e}"))?
    );

    Ok(())
}

fn cmd_stake_undelegate(args: &[String]) -> Result<(), String> {
    let state_path =
        arg_value(args, "--state").unwrap_or_else(|| "AOXC_DATA/economy/state.json".to_string());
    let staker = arg_value(args, "--staker").ok_or_else(|| "--staker is required".to_string())?;
    let validator =
        arg_value(args, "--validator").ok_or_else(|| "--validator is required".to_string())?;
    let amount: u128 = arg_value(args, "--amount")
        .ok_or_else(|| "--amount is required".to_string())?
        .parse()
        .map_err(|_| "--amount must be a valid u128".to_string())?;

    let mut state = EconomyState::load_or_default(&state_path)?;
    state.undelegate(&staker, &validator, amount)?;
    state.save(&state_path)?;

    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "state_path": state_path,
            "staker": staker,
            "validator": validator,
            "undelegated_amount": amount,
            "total_staked": state.total_staked(),
        }))
        .map_err(|e| format!("JSON_SERIALIZE_ERROR: {e}"))?
    );

    Ok(())
}

fn cmd_economy_status(args: &[String]) -> Result<(), String> {
    let state_path =
        arg_value(args, "--state").unwrap_or_else(|| "AOXC_DATA/economy/state.json".to_string());
    let state = EconomyState::load_or_default(&state_path)?;

    let output = serde_json::json!({
        "state_path": state_path,
        "treasury_account": state.treasury_account,
        "treasury_balance": state.treasury_balance(),
        "total_accounts": state.balances.len(),
        "total_staked": state.total_staked(),
        "positions": state.stakes,
        "balances": state.balances,
    });

    println!(
        "{}",
        serde_json::to_string_pretty(&output).map_err(|e| format!("JSON_SERIALIZE_ERROR: {e}"))?
    );
    Ok(())
}

fn cmd_runtime_status(args: &[String]) -> Result<(), String> {
    let profile_arg = arg_value(args, "--trace").unwrap_or_else(|| "standard".to_string());
    let trace_profile = match profile_arg.as_str() {
        "minimal" => TraceProfile::Minimal,
        "standard" => TraceProfile::Standard,
        "verbose" => TraceProfile::Verbose,
        other => {
            return Err(format!(
                "unsupported --trace profile: {other}, expected minimal|standard|verbose"
            ));
        }
    };

    let tps: f64 = arg_value(args, "--tps")
        .unwrap_or_else(|| "0.0".to_string())
        .parse()
        .map_err(|_| "--tps must be a valid f64".to_string())?;
    let peer_count: usize = arg_value(args, "--peers")
        .unwrap_or_else(|| "0".to_string())
        .parse()
        .map_err(|_| "--peers must be a valid usize".to_string())?;
    let error_rate: f64 = arg_value(args, "--error-rate")
        .unwrap_or_else(|| "0.0".to_string())
        .parse()
        .map_err(|_| "--error-rate must be a valid f64".to_string())?;

    let metrics = MetricsSnapshot {
        tps,
        peer_count,
        error_rate,
    };

    let output = serde_json::json!({
        "tracing": {
            "profile": profile_arg,
            "filter": trace_profile.as_filter(),
        },
        "telemetry": {
            "snapshot": {
                "tps": metrics.tps,
                "peer_count": metrics.peer_count,
                "error_rate": metrics.error_rate,
            },
            "prometheus": metrics.to_prometheus(),
        }
    });

    println!(
        "{}",
        serde_json::to_string_pretty(&output).map_err(|e| format!("JSON_SERIALIZE_ERROR: {e}"))?
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
        "AOXC Command Surface\n\nCommands:\n  vision\n  compat-matrix\n  version\n  key-bootstrap --password <secret> [--base-dir <dir>] [--name <name>] [--chain <id>] [--role <role>] [--zone <zone>] [--issuer <issuer>] [--validity-secs <u64>]\n  genesis-init [--path <file>] [--chain-num <u32>] [--block-time <u64>] [--treasury <u128>]\n  node-bootstrap\n  produce-once [--tx <payload>]\n  network-smoke\n  storage-smoke [--base-dir <dir>] [--index sqlite|redb]\n  economy-init [--state <file>] [--treasury-supply <u128>]\n  treasury-transfer --to <account> --amount <u128> [--state <file>]\n  stake-delegate --staker <account> --validator <id> --amount <u128> [--state <file>]\n  stake-undelegate --staker <account> --validator <id> --amount <u128> [--state <file>]\n  economy-status [--state <file>]\n  runtime-status [--trace minimal|standard|verbose] [--tps <f64>] [--peers <usize>] [--error-rate <f64>]\n  help\n"
    );
}
