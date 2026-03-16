use aoxcmd::build_info::BuildInfo;
use aoxcmd::data_home;
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
use std::collections::BTreeMap;

use std::env;
use std::process;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CliLanguage {
    En,
    Tr,
    Es,
    De,
}

impl CliLanguage {
    fn from_code(input: &str) -> Self {
        match input.trim().to_ascii_lowercase().as_str() {
            "tr" | "tr-tr" | "turkish" | "türkçe" => Self::Tr,
            "es" | "es-es" | "spanish" | "español" => Self::Es,
            "de" | "de-de" | "german" | "deutsch" => Self::De,
            _ => Self::En,
        }
    }
}

fn main() {
    if let Err(error) = run_cli() {
        eprintln!("AOXCMD_ERROR: {error}");
        process::exit(1);
    }
}

fn run_cli() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();
    let lang = detect_language(&args[1..]);
    apply_home_override(&args[1..]);

    if args.len() < 2 {
        print_usage(lang);
        return Ok(());
    }

    match args[1].as_str() {
        "version" | "--version" | "-V" => cmd_version(),
        "help" | "--help" | "-h" => {
            print_usage(lang);
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
        "interop-readiness" => cmd_interop_readiness(),
        "interop-gate" => cmd_interop_gate(&args[2..]),
        "production-audit" => cmd_production_audit(&args[2..]),
        other => Err(localized_unknown_command(lang, other)),
    }
}

fn apply_home_override(args: &[String]) {
    if let Some(home) = arg_value(args, "--home") {
        // SAFETY: this process performs environment mutation during single-threaded
        // CLI bootstrap before any background threads are started.
        unsafe {
            env::set_var("AOXC_HOME", home);
        }
    }
}

fn detect_language(args: &[String]) -> CliLanguage {
    if let Some(explicit) = arg_value(args, "--lang") {
        return CliLanguage::from_code(&explicit);
    }

    let from_env = env::var("AOXC_LANG").unwrap_or_else(|_| "en".to_string());
    CliLanguage::from_code(&from_env)
}

fn localized_unknown_command(lang: CliLanguage, command: &str) -> String {
    match lang {
        CliLanguage::Tr => format!("bilinmeyen komut: {command}"),
        CliLanguage::Es => format!("comando desconocido: {command}"),
        CliLanguage::De => format!("unbekannter befehl: {command}"),
        CliLanguage::En => format!("unknown command: {command}"),
    }
}

fn cmd_version() -> Result<(), String> {
    let build = BuildInfo::collect();
    let output = serde_json::json!({
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
    let defaults = bootstrap_defaults(args)?;
    assert_mainnet_key_policy(args, defaults.profile)?;

    let home = data_home::resolve_data_home(args);
    let base_dir = arg_value(args, "--base-dir").unwrap_or_else(|| data_home::join(&home, "keys"));
    let name = arg_value(args, "--name").unwrap_or(defaults.name);
    let chain = arg_value(args, "--chain").unwrap_or(defaults.chain);
    let role = arg_value(args, "--role").unwrap_or_else(|| "validator".to_string());
    let zone = arg_value(args, "--zone").unwrap_or_else(|| "core".to_string());
    let issuer = arg_value(args, "--issuer").unwrap_or(defaults.issuer);
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

    let output = serde_json::json!({
        "profile": defaults.profile,
        "summary": material.summary(),
    });

    println!(
        "{}",
        serde_json::to_string_pretty(&output)
            .map_err(|error| format!("JSON_SERIALIZE_ERROR: {error}"))?
    );

    Ok(())
}

fn cmd_genesis_init(args: &[String]) -> Result<(), String> {
    let home = data_home::resolve_data_home(args);
    let path = arg_value(args, "--path")
        .unwrap_or_else(|| data_home::join(&home, "identity/genesis.json"));
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
    let home = data_home::resolve_data_home(args);
    let base_dir =
        arg_value(args, "--base-dir").unwrap_or_else(|| data_home::join(&home, "storage"));
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
    let home = data_home::resolve_data_home(args);
    let state_path =
        arg_value(args, "--state").unwrap_or_else(|| data_home::join(&home, "economy/state.json"));
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
    let home = data_home::resolve_data_home(args);
    let state_path =
        arg_value(args, "--state").unwrap_or_else(|| data_home::join(&home, "economy/state.json"));
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
    let home = data_home::resolve_data_home(args);
    let state_path =
        arg_value(args, "--state").unwrap_or_else(|| data_home::join(&home, "economy/state.json"));
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
    let home = data_home::resolve_data_home(args);
    let state_path =
        arg_value(args, "--state").unwrap_or_else(|| data_home::join(&home, "economy/state.json"));
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
    let home = data_home::resolve_data_home(args);
    let state_path =
        arg_value(args, "--state").unwrap_or_else(|| data_home::join(&home, "economy/state.json"));
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

fn cmd_interop_readiness() -> Result<(), String> {
    let output = serde_json::json!({
        "identity": {
            "key_algorithms": [
                {
                    "name": "Dilithium3",
                    "role": "post-quantum signing for actor identity",
                    "status": "implemented in aoxcore::identity::pq_keys"
                },
                {
                    "name": "Argon2id + AES-256-GCM keyfile",
                    "role": "password-protected local key material at rest",
                    "status": "implemented in aoxcore::identity::keyfile"
                }
            ]
        },
        "execution_lanes": [
            {"lane": "EVM", "priority": "high", "next_step": "RPC and receipt parity test vectors"},
            {"lane": "WASM", "priority": "high", "next_step": "host-call compatibility matrix"},
            {"lane": "Sui Move", "priority": "medium", "next_step": "object/state adapter validation"},
            {"lane": "Cardano UTXO", "priority": "medium", "next_step": "UTXO translator and witness mapping"}
        ],
        "production_checklist": [
            "cross-chain finality assumptions documented per target chain",
            "bridge adapter fuzz + property testing",
            "deterministic serialization and replay tests",
            "observability SLOs and alerting thresholds",
            "external security audit for bridge and key lifecycle"
        ]
    });

    println!(
        "{}",
        serde_json::to_string_pretty(&output).map_err(|e| format!("JSON_SERIALIZE_ERROR: {e}"))?
    );

    Ok(())
}

fn cmd_interop_gate(args: &[String]) -> Result<(), String> {
    let audit_complete = arg_bool_value(args, "--audit-complete").unwrap_or(false);
    let fuzz_complete = arg_bool_value(args, "--fuzz-complete").unwrap_or(false);
    let replay_complete = arg_bool_value(args, "--replay-complete").unwrap_or(false);
    let finality_matrix_complete =
        arg_bool_value(args, "--finality-matrix-complete").unwrap_or(false);
    let slo_complete = arg_bool_value(args, "--slo-complete").unwrap_or(false);

    let checks = [
        ("external_security_audit", audit_complete),
        ("bridge_fuzz_property_testing", fuzz_complete),
        ("deterministic_replay_suite", replay_complete),
        ("finality_assumption_matrix", finality_matrix_complete),
        ("observability_slo_alerting", slo_complete),
    ];

    let passed = checks.iter().filter(|(_, ok)| *ok).count();
    let total = checks.len();
    let readiness_percent = ((passed as f64 / total as f64) * 100.0 * 100.0).round() / 100.0;
    let missing: Vec<&str> = checks
        .iter()
        .filter_map(|(name, ok)| if *ok { None } else { Some(*name) })
        .collect();

    let enforce = arg_flag(args, "--enforce");

    let output = serde_json::json!({
        "pass": missing.is_empty(),
        "readiness_percent": readiness_percent,
        "passed_checks": passed,
        "total_checks": total,
        "missing_controls": missing,
    });

    println!(
        "{}",
        serde_json::to_string_pretty(&output).map_err(|e| format!("JSON_SERIALIZE_ERROR: {e}"))?
    );

    if enforce && !output["pass"].as_bool().unwrap_or(false) {
        return Err("interop gate failed: missing required controls".to_string());
    }

    Ok(())
}

fn cmd_production_audit(args: &[String]) -> Result<(), String> {
    let home = data_home::resolve_data_home(args);
    let genesis_path = arg_value(args, "--genesis")
        .unwrap_or_else(|| data_home::join(&home, "identity/genesis.json"));
    let economy_state_path =
        arg_value(args, "--state").unwrap_or_else(|| data_home::join(&home, "economy/state.json"));

    let ai_model_signed = arg_bool_value(args, "--ai-model-signed").unwrap_or(false);
    let ai_prompt_guard = arg_bool_value(args, "--ai-prompt-guard").unwrap_or(false);
    let ai_anomaly_detection = arg_bool_value(args, "--ai-anomaly-detection").unwrap_or(false);
    let ai_human_override = arg_bool_value(args, "--ai-human-override").unwrap_or(false);

    let build = BuildInfo::collect();

    let genesis = GenesisLoader::load(&genesis_path).ok();
    let genesis_hash = genesis.as_ref().map(|g| g.config.state_hash());
    let genesis_chain_id = genesis.as_ref().map(|g| g.config.chain_id.clone());
    let genesis_valid = genesis.is_some();

    let economy = EconomyState::load_or_default(&economy_state_path)?;
    let mut stake_by_validator: BTreeMap<String, u128> = BTreeMap::new();
    for position in &economy.stakes {
        let entry = stake_by_validator
            .entry(position.validator.clone())
            .or_insert(0);
        *entry = entry.saturating_add(position.amount);
    }

    let node = state::setup().map_err(|error| error.to_string())?;

    let ai_checks = [
        ("model_signature_verification", ai_model_signed),
        ("prompt_injection_guard", ai_prompt_guard),
        ("anomaly_detection_for_ai_paths", ai_anomaly_detection),
        ("human_override_for_high_risk_actions", ai_human_override),
    ];

    let mut recommendations = Vec::new();
    if !genesis_valid {
        recommendations.push(
            "Provide a valid genesis file and verify canonical state_hash before mainnet rollout",
        );
    }
    if build.cert_sha256 == "not-configured" {
        recommendations.push("Embed node certificate fingerprint into build pipeline and enforce startup verification");
    }
    for (name, ok) in ai_checks {
        if !ok {
            recommendations.push(match name {
                "model_signature_verification" => {
                    "Enable cryptographic AI model artifact signature verification"
                }
                "prompt_injection_guard" => "Enable AI prompt injection and jail-break guardrails",
                "anomaly_detection_for_ai_paths" => {
                    "Enable anomaly detection for AI-assisted decision paths"
                }
                "human_override_for_high_risk_actions" => {
                    "Require human override on high-risk AI decisions"
                }
                _ => "Enable missing AI security controls",
            });
        }
    }

    let ai_security_score = ai_control_score(&ai_checks);

    let output = serde_json::json!({
        "genesis": {
            "path": genesis_path,
            "valid": genesis_valid,
            "chain_id": genesis_chain_id,
            "state_hash": genesis_hash,
        },
        "certificates": {
            "embedded_cert_path": build.cert_path,
            "embedded_cert_sha256": build.cert_sha256,
            "embedded_cert_error": build.cert_error,
        },
        "key_security": {
            "mainnet_key_generation_requires_explicit_opt_in": true,
            "env_override": "AOXC_ALLOW_MAINNET_KEYS",
        },
        "ai_security": {
            "controls": ai_checks,
            "score": ai_security_score,
        },
        "validator_network": {
            "configured_validators": node.rotation.validators().len(),
            "quorum": {
                "numerator": node.consensus.quorum.numerator,
                "denominator": node.consensus.quorum.denominator,
            }
        },
        "treasury_and_staking": {
            "state_path": economy_state_path,
            "treasury_account": economy.treasury_account,
            "treasury_balance": economy.treasury_balance(),
            "total_staked": economy.total_staked(),
            "stake_by_validator": stake_by_validator,
            "positions": economy.stakes,
        },
        "recommendations": recommendations,
    });

    println!(
        "{}",
        serde_json::to_string_pretty(&output).map_err(|e| format!("JSON_SERIALIZE_ERROR: {e}"))?
    );

    Ok(())
}

fn ai_control_score(controls: &[(&str, bool)]) -> u8 {
    if controls.is_empty() {
        return 0;
    }

    ((controls.iter().filter(|(_, ok)| *ok).count() as f64 / controls.len() as f64) * 100.0).round()
        as u8
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

#[derive(Debug, Clone)]
struct BootstrapDefaults {
    profile: &'static str,
    name: String,
    chain: String,
    issuer: String,
}

fn bootstrap_defaults(args: &[String]) -> Result<BootstrapDefaults, String> {
    let profile = arg_value(args, "--profile").unwrap_or_else(|| "mainnet".to_string());

    match profile.as_str() {
        "mainnet" => Ok(BootstrapDefaults {
            profile: "mainnet",
            name: "node".to_string(),
            chain: "AOXC-MAIN".to_string(),
            issuer: "AOXC-ROOT-CA".to_string(),
        }),
        "testnet" | "test" => Ok(BootstrapDefaults {
            profile: "testnet",
            name: "TEST-VALIDATOR-01".to_string(),
            chain: "TEST-XXX-XX-LOCAL".to_string(),
            issuer: "TEST-XXX-ROOT-CA".to_string(),
        }),
        other => Err(format!(
            "unsupported --profile value: {other}, expected mainnet|testnet"
        )),
    }
}

fn arg_bool_value(args: &[String], key: &str) -> Option<bool> {
    arg_value(args, key).map(|value| matches!(value.as_str(), "1" | "true" | "yes" | "on"))
}

fn arg_flag(args: &[String], key: &str) -> bool {
    args.iter().any(|arg| arg == key)
}

fn assert_mainnet_key_policy(args: &[String], profile: &str) -> Result<(), String> {
    if profile != "mainnet" {
        return Ok(());
    }

    let allow_flag = arg_flag(args, "--allow-mainnet");
    let allow_env = env::var("AOXC_ALLOW_MAINNET_KEYS")
        .map(|v| matches!(v.as_str(), "1" | "true" | "yes" | "on"))
        .unwrap_or(false);

    if allow_flag || allow_env {
        return Ok(());
    }

    Err(
        "mainnet key bootstrap blocked: pass --allow-mainnet or set AOXC_ALLOW_MAINNET_KEYS=true"
            .to_string(),
    )
}

fn print_usage(lang: CliLanguage) {
    println!("{}", usage_text(lang));
}

fn usage_text(lang: CliLanguage) -> &'static str {
    match lang {
        CliLanguage::Tr => {
            "AOXC Komut Yüzeyi

Komutlar:
  vision
  compat-matrix
  version
  key-bootstrap --password <secret> [--home <dir>] [--profile mainnet|testnet] [--allow-mainnet] [--base-dir <dir>] [--name <name>] [--chain <id>] [--role <role>] [--zone <zone>] [--issuer <issuer>] [--validity-secs <u64>]
  genesis-init [--home <dir>] [--path <file>] [--chain-num <u32>] [--block-time <u64>] [--treasury <u128>]
  node-bootstrap
  produce-once [--tx <payload>]
  network-smoke
  storage-smoke [--home <dir>] [--base-dir <dir>] [--index sqlite|redb]
  economy-init [--home <dir>] [--state <file>] [--treasury-supply <u128>]
  treasury-transfer --to <account> --amount <u128> [--home <dir>] [--state <file>]
  stake-delegate --staker <account> --validator <id> --amount <u128> [--home <dir>] [--state <file>]
  stake-undelegate --staker <account> --validator <id> --amount <u128> [--home <dir>] [--state <file>]
  economy-status [--home <dir>] [--state <file>]
  runtime-status [--trace minimal|standard|verbose] [--tps <f64>] [--peers <usize>] [--error-rate <f64>]
  interop-readiness
  interop-gate [--audit-complete <bool>] [--fuzz-complete <bool>] [--replay-complete <bool>] [--finality-matrix-complete <bool>] [--slo-complete <bool>] [--enforce]
  production-audit [--home <dir>] [--genesis <file>] [--state <file>] [--ai-model-signed <bool>] [--ai-prompt-guard <bool>] [--ai-anomaly-detection <bool>] [--ai-human-override <bool>]
  help

Global:
  --lang <en|tr|es|de> (veya AOXC_LANG ortam değişkeni)
  --home <dir> (varsayılan: $HOME/.AOXC-Data, veya AOXC_HOME)
"
        }
        CliLanguage::Es => {
            "Superficie de Comandos AOXC

Comandos:
  vision
  compat-matrix
  version
  key-bootstrap --password <secret> [--home <dir>] [--profile mainnet|testnet] [--allow-mainnet] [--base-dir <dir>] [--name <name>] [--chain <id>] [--role <role>] [--zone <zone>] [--issuer <issuer>] [--validity-secs <u64>]
  genesis-init [--home <dir>] [--path <file>] [--chain-num <u32>] [--block-time <u64>] [--treasury <u128>]
  node-bootstrap
  produce-once [--tx <payload>]
  network-smoke
  storage-smoke [--home <dir>] [--base-dir <dir>] [--index sqlite|redb]
  economy-init [--home <dir>] [--state <file>] [--treasury-supply <u128>]
  treasury-transfer --to <account> --amount <u128> [--home <dir>] [--state <file>]
  stake-delegate --staker <account> --validator <id> --amount <u128> [--home <dir>] [--state <file>]
  stake-undelegate --staker <account> --validator <id> --amount <u128> [--home <dir>] [--state <file>]
  economy-status [--home <dir>] [--state <file>]
  runtime-status [--trace minimal|standard|verbose] [--tps <f64>] [--peers <usize>] [--error-rate <f64>]
  interop-readiness
  interop-gate [--audit-complete <bool>] [--fuzz-complete <bool>] [--replay-complete <bool>] [--finality-matrix-complete <bool>] [--slo-complete <bool>] [--enforce]
  production-audit [--home <dir>] [--genesis <file>] [--state <file>] [--ai-model-signed <bool>] [--ai-prompt-guard <bool>] [--ai-anomaly-detection <bool>] [--ai-human-override <bool>]
  help

Global:
  --lang <en|tr|es|de> (o variable AOXC_LANG)
  --home <dir> (por defecto: $HOME/.AOXC-Data, o AOXC_HOME)
"
        }
        CliLanguage::De => {
            "AOXC Kommandooberfläche

Befehle:
  vision
  compat-matrix
  version
  key-bootstrap --password <secret> [--home <dir>] [--profile mainnet|testnet] [--allow-mainnet] [--base-dir <dir>] [--name <name>] [--chain <id>] [--role <role>] [--zone <zone>] [--issuer <issuer>] [--validity-secs <u64>]
  genesis-init [--home <dir>] [--path <file>] [--chain-num <u32>] [--block-time <u64>] [--treasury <u128>]
  node-bootstrap
  produce-once [--tx <payload>]
  network-smoke
  storage-smoke [--home <dir>] [--base-dir <dir>] [--index sqlite|redb]
  economy-init [--home <dir>] [--state <file>] [--treasury-supply <u128>]
  treasury-transfer --to <account> --amount <u128> [--home <dir>] [--state <file>]
  stake-delegate --staker <account> --validator <id> --amount <u128> [--home <dir>] [--state <file>]
  stake-undelegate --staker <account> --validator <id> --amount <u128> [--home <dir>] [--state <file>]
  economy-status [--home <dir>] [--state <file>]
  runtime-status [--trace minimal|standard|verbose] [--tps <f64>] [--peers <usize>] [--error-rate <f64>]
  interop-readiness
  interop-gate [--audit-complete <bool>] [--fuzz-complete <bool>] [--replay-complete <bool>] [--finality-matrix-complete <bool>] [--slo-complete <bool>] [--enforce]
  production-audit [--home <dir>] [--genesis <file>] [--state <file>] [--ai-model-signed <bool>] [--ai-prompt-guard <bool>] [--ai-anomaly-detection <bool>] [--ai-human-override <bool>]
  help

Global:
  --lang <en|tr|es|de> (oder AOXC_LANG Umgebungsvariable)
  --home <dir> (Standard: $HOME/.AOXC-Data oder AOXC_HOME)
"
        }
        CliLanguage::En => {
            "AOXC Command Surface

Commands:
  vision
  compat-matrix
  version
  key-bootstrap --password <secret> [--home <dir>] [--profile mainnet|testnet] [--allow-mainnet] [--base-dir <dir>] [--name <name>] [--chain <id>] [--role <role>] [--zone <zone>] [--issuer <issuer>] [--validity-secs <u64>]
  genesis-init [--home <dir>] [--path <file>] [--chain-num <u32>] [--block-time <u64>] [--treasury <u128>]
  node-bootstrap
  produce-once [--tx <payload>]
  network-smoke
  storage-smoke [--home <dir>] [--base-dir <dir>] [--index sqlite|redb]
  economy-init [--home <dir>] [--state <file>] [--treasury-supply <u128>]
  treasury-transfer --to <account> --amount <u128> [--home <dir>] [--state <file>]
  stake-delegate --staker <account> --validator <id> --amount <u128> [--home <dir>] [--state <file>]
  stake-undelegate --staker <account> --validator <id> --amount <u128> [--home <dir>] [--state <file>]
  economy-status [--home <dir>] [--state <file>]
  runtime-status [--trace minimal|standard|verbose] [--tps <f64>] [--peers <usize>] [--error-rate <f64>]
  interop-readiness
  interop-gate [--audit-complete <bool>] [--fuzz-complete <bool>] [--replay-complete <bool>] [--finality-matrix-complete <bool>] [--slo-complete <bool>] [--enforce]
  production-audit [--home <dir>] [--genesis <file>] [--state <file>] [--ai-model-signed <bool>] [--ai-prompt-guard <bool>] [--ai-anomaly-detection <bool>] [--ai-human-override <bool>]
  help

Global:
  --lang <en|tr|es|de> (or AOXC_LANG environment variable)
  --home <dir> (default: $HOME/.AOXC-Data, or AOXC_HOME)
"
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        CliLanguage, ai_control_score, arg_bool_value, assert_mainnet_key_policy,
        bootstrap_defaults, detect_language, localized_unknown_command, usage_text,
    };

    #[test]
    fn bootstrap_defaults_mainnet() {
        let args = vec![];
        let defaults = bootstrap_defaults(&args).expect("mainnet defaults");
        assert_eq!(defaults.profile, "mainnet");
        assert_eq!(defaults.chain, "AOXC-MAIN");
    }

    #[test]
    fn bootstrap_defaults_testnet() {
        let args = vec!["--profile".to_string(), "testnet".to_string()];
        let defaults = bootstrap_defaults(&args).expect("testnet defaults");
        assert_eq!(defaults.profile, "testnet");
        assert!(defaults.chain.starts_with("TEST-"));
        assert!(defaults.issuer.starts_with("TEST-"));
    }

    #[test]
    fn bool_argument_parser_works() {
        let args = vec!["--audit-complete".to_string(), "true".to_string()];
        assert_eq!(arg_bool_value(&args, "--audit-complete"), Some(true));
    }

    #[test]
    fn mainnet_profile_requires_explicit_override() {
        let allow_env = std::env::var("AOXC_ALLOW_MAINNET_KEYS")
            .map(|v| matches!(v.as_str(), "1" | "true" | "yes" | "on"))
            .unwrap_or(false);

        if !allow_env {
            let args = vec![];
            assert!(assert_mainnet_key_policy(&args, "mainnet").is_err());
        }

        let args = vec!["--allow-mainnet".to_string()];
        assert!(assert_mainnet_key_policy(&args, "mainnet").is_ok());

        let args = vec![];
        assert!(assert_mainnet_key_policy(&args, "testnet").is_ok());
    }

    #[test]
    fn detect_language_prefers_explicit_flag() {
        let args = vec!["help".to_string(), "--lang".to_string(), "tr".to_string()];
        assert_eq!(detect_language(&args), CliLanguage::Tr);
    }

    #[test]
    fn usage_text_contains_localized_headers() {
        assert!(usage_text(CliLanguage::En).contains("AOXC Command Surface"));
        assert!(usage_text(CliLanguage::Tr).contains("AOXC Komut Yüzeyi"));
        assert!(usage_text(CliLanguage::Es).contains("Superficie de Comandos AOXC"));
        assert!(usage_text(CliLanguage::De).contains("AOXC Kommandooberfläche"));
    }

    #[test]
    fn unknown_command_is_localized() {
        assert_eq!(
            localized_unknown_command(CliLanguage::Tr, "foo"),
            "bilinmeyen komut: foo"
        );
        assert_eq!(
            localized_unknown_command(CliLanguage::Es, "foo"),
            "comando desconocido: foo"
        );
        assert_eq!(
            localized_unknown_command(CliLanguage::De, "foo"),
            "unbekannter befehl: foo"
        );
    }

    #[test]
    fn ai_control_score_is_stable() {
        let controls = [
            ("model_signature_verification", true),
            ("prompt_injection_guard", true),
            ("anomaly_detection_for_ai_paths", false),
            ("human_override_for_high_risk_actions", false),
        ];

        assert_eq!(ai_control_score(&controls), 50);
        assert_eq!(ai_control_score(&[]), 0);
    }
}
