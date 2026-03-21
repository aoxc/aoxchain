use aoxcmd::build_info::BuildInfo;
use aoxcmd::data_home;
use aoxcmd::economy::ledger::EconomyState;
use aoxcmd::keys::{KeyBootstrapRequest, KeyManager, KeyPaths};
use aoxcmd::node::engine::produce_single_block;
use aoxcmd::node::state;
use aoxcmd::telemetry::prometheus::MetricsSnapshot;
use aoxcmd::telemetry::tracing::TraceProfile;

use aoxcdata::{BlockEnvelope, HybridDataStore, IndexBackend};
use aoxcnet::ports::{LIVE_SMOKE_TEST_PORT, PORT_BINDINGS, RPC_HTTP_PORT};
use aoxcnet::transport::live_tcp::run_live_tcp_smoke_on;
use aoxcore::genesis::config::{GenesisConfig, SettlementLink, TREASURY_ACCOUNT};
use aoxcore::genesis::loader::GenesisLoader;
use aoxcore::identity::ca::CertificateAuthority;
use aoxcore::identity::hd_path::HdPath;
use aoxcore::identity::key_engine::{KeyEngine, MASTER_SEED_LEN};
use aoxcore::protocol::{
    canonical_chain_families, canonical_message_envelope_fields, canonical_modules,
    canonical_sovereign_roots,
};
use serde::Serialize;
use sha3::{Digest, Sha3_256};
use std::collections::BTreeMap;

mod cli_support;

#[cfg(test)]
use cli_support::{CliLanguage, usage_text};
use cli_support::{
    arg_bool_value, arg_flag, arg_value, detect_language, localized_unknown_command, print_usage,
};

use std::env;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::process;
use std::thread;
use std::time::{Duration, Instant};

const AOXC_RELEASE_NAME: &str = "AOXC Alpha: Genesis V1";
const TESTNET_FIXTURE_MEMBERS: [(&str, &str, u16, u16, u16, &str); 5] = [
    (
        "atlas",
        "Atlas Validator",
        39001,
        19101,
        1,
        "11111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111",
    ),
    (
        "boreal",
        "Boreal Validator",
        39002,
        19102,
        2,
        "22222222222222222222222222222222222222222222222222222222222222222222222222222222222222222222222222222222222222222222222222222222",
    ),
    (
        "cypher",
        "Cypher Validator",
        39003,
        19103,
        3,
        "33333333333333333333333333333333333333333333333333333333333333333333333333333333333333333333333333333333333333333333333333333333",
    ),
    (
        "delta",
        "Delta Validator",
        39004,
        19104,
        4,
        "44444444444444444444444444444444444444444444444444444444444444444444444444444444444444444444444444444444444444444444444444444444",
    ),
    (
        "ember",
        "Ember Validator",
        39005,
        19105,
        5,
        "55555555555555555555555555555555555555555555555555555555555555555555555555555555555555555555555555555555555555555555555555555555",
    ),
];

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
        "build-manifest" => cmd_build_manifest(),
        "node-connection-policy" => cmd_node_connection_policy(&args[2..]),
        "sovereign-core" => cmd_sovereign_core(),
        "module-architecture" => cmd_module_architecture(),
        "compat-matrix" => cmd_compat_matrix(),
        "port-map" => cmd_port_map(),
        "testnet-fixture-init" => cmd_testnet_fixture_init(&args[2..]),
        "load-benchmark" => cmd_load_benchmark(&args[2..]),
        "mainnet-readiness" => cmd_mainnet_readiness(),
        "key-bootstrap" => cmd_key_bootstrap(&args[2..]),
        "genesis-init" => cmd_genesis_init(&args[2..]),
        "node-bootstrap" => cmd_node_bootstrap(&args[2..]),
        "produce-once" => cmd_produce_once(&args[2..]),
        "node-run" => cmd_node_run(&args[2..]),
        "network-smoke" => cmd_network_smoke(&args[2..]),
        "real-network" => cmd_real_network(&args[2..]),
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

fn cmd_version() -> Result<(), String> {
    let build = BuildInfo::collect();
    let output = version_payload(&build);

    println!(
        "{}",
        serde_json::to_string_pretty(&output)
            .map_err(|error| format!("JSON_SERIALIZE_ERROR: {error}"))?
    );

    Ok(())
}

fn version_payload(build: &BuildInfo) -> serde_json::Value {
    serde_json::json!({
        "name": "aoxcmd",
        "release_name": AOXC_RELEASE_NAME,
        "version": build.semver,
        "git_commit": build.git_commit,
        "git_dirty": build.git_dirty,
        "source_date_epoch": build.source_date_epoch,
        "build_profile": build.build_profile,
        "release_channel": build.release_channel,
        "attestation_hash": build.attestation_hash,
        "embedded_cert": {
            "path": build.cert_path,
            "sha256": build.cert_sha256,
            "error": build.cert_error,
        }
    })
}

fn cmd_build_manifest() -> Result<(), String> {
    let build = BuildInfo::collect();
    let output = build_manifest_payload(&build);

    println!(
        "{}",
        serde_json::to_string_pretty(&output)
            .map_err(|error| format!("JSON_SERIALIZE_ERROR: {error}"))?
    );

    Ok(())
}

fn cmd_node_connection_policy(args: &[String]) -> Result<(), String> {
    let build = BuildInfo::collect();
    let enforce = arg_flag(args, "--enforce-official");
    let official_release = is_official_release(&build);
    let output = node_connection_policy_payload(&build);

    println!(
        "{}",
        serde_json::to_string_pretty(&output)
            .map_err(|error| format!("JSON_SERIALIZE_ERROR: {error}"))?
    );

    if enforce && !official_release {
        return Err(
            "official node policy failed: build is not an official release artifact".to_string(),
        );
    }

    Ok(())
}

fn cmd_vision() -> Result<(), String> {
    let output = serde_json::json!({
        "release_name": AOXC_RELEASE_NAME,
        "chain_positioning": "interop relay-oriented coordination chain",
        "primary_goal": "cross-chain compatibility and deterministic coordination over raw throughput",
        "execution_strategy": "sovereign constitutional local core + remote execution domains",
        "recommended_topology": "local sovereign root modules + remote chain contracts/execution adapters",
        "constitutional_roots": [
            "identity",
            "supply",
            "governance",
            "relay",
            "security",
            "settlement",
            "treasury"
        ],
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

fn cmd_sovereign_core() -> Result<(), String> {
    let sovereign_roots: Vec<&str> = canonical_sovereign_roots()
        .iter()
        .map(|root| root.as_str())
        .collect();

    let output = serde_json::json!({
        "local_chain_role": "sovereign constitutional core",
        "remote_chain_role": "execution domains connected through contracts and settlement rules",
        "constitutional_roots": sovereign_roots,
        "local_must_keep": {
            "identity": [
                "root_account_registry",
                "chain_mappings",
                "signer_bindings",
                "recovery_authority",
                "key_rotation_rules",
                "delegate_registry"
            ],
            "supply": [
                "total_canonical_supply",
                "mint_authority_root",
                "burn_settlement_root",
                "global_supply_accounting",
                "emission_policy"
            ],
            "governance": [
                "protocol_upgrades",
                "module_approvals",
                "remote_domain_authorization",
                "risk_parameters",
                "bridge_mint_ceilings",
                "validator_policy"
            ],
            "relay": [
                "outbound_message_commitments",
                "inbound_settlement_acceptance_rules",
                "nonce_root",
                "replay_protection_root",
                "approved_remote_domains",
                "message_policy_classes"
            ],
            "security": [
                "validator_set",
                "attester_set",
                "quorum_thresholds",
                "slashing_logic",
                "signature_policy",
                "emergency_security_overrides"
            ],
            "settlement": [
                "final_settlement_records",
                "remote_execution_receipts_hash",
                "dispute_intake",
                "final_confirmation_state",
                "cross_domain_settlement_journal"
            ],
            "treasury": [
                "protocol_treasury",
                "reserve_balances",
                "insurance_reserve",
                "strategic_liquidity_authority",
                "module_funding_authority"
            ]
        },
        "local_must_not_keep": [
            "heavy_application_logic",
            "chain_specific_dapp_logic",
            "remote_integration_implementation_details",
            "large_data_payloads",
            "ai_decision_engine",
            "experimental_app_execution"
        ]
    });

    println!(
        "{}",
        serde_json::to_string_pretty(&output)
            .map_err(|error| format!("JSON_SERIALIZE_ERROR: {error}"))?
    );

    Ok(())
}

fn is_official_release(build: &BuildInfo) -> bool {
    let channel_ok = matches!(build.release_channel, "stable" | "official" | "mainnet");
    let cert_ok = !matches!(build.cert_sha256, "not-configured" | "unavailable");
    channel_ok && build.git_dirty == "false" && cert_ok && build.attestation_hash.len() == 64
}

fn build_manifest_payload(build: &BuildInfo) -> serde_json::Value {
    let official_release = is_official_release(build);

    serde_json::json!({
        "artifact": {
            "name": "aoxcmd",
            "release_name": AOXC_RELEASE_NAME,
            "version": build.semver,
            "git_commit": build.git_commit,
            "git_dirty": build.git_dirty,
            "source_date_epoch": build.source_date_epoch,
            "build_profile": build.build_profile,
            "release_channel": build.release_channel,
            "attestation_hash": build.attestation_hash,
        },
        "certificate": {
            "path": build.cert_path,
            "sha256": build.cert_sha256,
            "error": build.cert_error,
        },
        "supply_chain_policy": {
            "official_release": official_release,
            "requires_embedded_certificate": true,
            "requires_attestation_hash": true,
            "accept_unofficial_node_builds": false,
        }
    })
}

fn node_connection_policy_payload(build: &BuildInfo) -> serde_json::Value {
    let official_release = is_official_release(build);

    serde_json::json!({
        "local_build": {
            "release_name": AOXC_RELEASE_NAME,
            "version": build.semver,
            "release_channel": build.release_channel,
            "git_dirty": build.git_dirty,
            "attestation_hash": build.attestation_hash,
            "embedded_cert_sha256": build.cert_sha256,
            "official_release": official_release,
        },
        "accepted_remote_policy": {
            "require_mtls": true,
            "require_certificate_fingerprint_match": true,
            "require_attestation_hash_exchange": true,
            "allow_unofficial_remote_builds": false,
            "accepted_release_channels": ["stable", "official", "mainnet"],
        },
        "operator_guidance": [
            "Embed a node certificate at build time with AOXC_EMBED_CERT_PATH",
            "Distribute attestation_hash and certificate fingerprint via a signed release manifest",
            "Reject ad-hoc local builds for production peering unless explicitly approved",
        ]
    })
}

fn cmd_module_architecture() -> Result<(), String> {
    let relay_module_names: Vec<&str> = canonical_modules()
        .iter()
        .map(|module| module.as_str())
        .collect();
    let sovereign_roots: Vec<&str> = canonical_sovereign_roots()
        .iter()
        .map(|root| root.as_str())
        .collect();
    let supported_chain_families: Vec<&str> = canonical_chain_families()
        .iter()
        .map(|family| family.as_str())
        .collect();
    let envelope_fields = canonical_message_envelope_fields();

    let output = serde_json::json!({
        "relay_core": {
            "principle": "keep the relay chain thin, neutral, and durable",
            "canonical_modules": relay_module_names,
            "sovereign_roots": sovereign_roots,
            "responsibilities": [
                "finality_ordering",
                "shared_security",
                "validator_set_management",
                "cross_module_message_routing",
                "universal_identity_root",
                "state_commitment_and_proof_root_anchoring",
                "governance_and_upgrades",
                "fee_and_staking_settlement_root"
            ]
        },
        "attached_modules": [
            {
                "name": "AOXC-MODULE-IDENTITY",
                "purpose": "universal identity, address binding, recovery, delegates, chain account mapping",
                "must_depend_on_relay": ["identity_root", "governance", "state_commitment"]
            },
            {
                "name": "AOXC-MODULE-ASSET",
                "purpose": "native asset, wrapped assets, treasury accounting, bridge escrow and settlement balances",
                "must_depend_on_relay": ["settlement_root", "governance", "security_policy"]
            },
            {
                "name": "AOXC-MODULE-EXECUTION",
                "purpose": "contracts, programmable actions, intents, and app-specific logic outside the relay core",
                "must_depend_on_relay": ["checkpoint_acceptance", "message_bus", "governance"]
            },
            {
                "name": "AOXC-MODULE-INTEROP",
                "purpose": "single bridge domain with adapter families for external chain connectivity",
                "adapters": ["evm", "solana", "utxo", "ibc", "object"],
                "must_depend_on_relay": ["message_bus", "identity_root", "proof_anchoring", "security_policy"]
            },
            {
                "name": "AOXC-MODULE-PROOF",
                "purpose": "data commitments, proof publication, light-client support data, batch/blob references",
                "must_depend_on_relay": ["state_commitment", "finality", "governance"]
            }
        ],
        "message_envelope": {
            "fields": envelope_fields
        },
        "security_boundaries": {
            "relay_core": [
                "minimum_attack_surface",
                "critical_state_only",
                "no_heavy_app_logic",
                "governance_controlled_upgrades"
            ],
            "modules": [
                "separate_risk_domains",
                "separate_rate_limits",
                "separate_circuit_breakers",
                "separate_fee_policies",
                "separate_storage_proof_domains"
            ]
        },
        "compatibility_strategy": {
            "model": "functional modules + adapter families",
            "supported_chain_families": supported_chain_families,
            "do_not_do": "do not turn the relay chain into a heavy application chain",
            "why": "chain families evolve, but identity, asset, execution, interop, and proof responsibilities remain stable"
        }
    });

    println!(
        "{}",
        serde_json::to_string_pretty(&output)
            .map_err(|error| format!("JSON_SERIALIZE_ERROR: {error}"))?
    );

    Ok(())
}

fn cmd_port_map() -> Result<(), String> {
    let ports: Vec<_> = PORT_BINDINGS
        .iter()
        .map(|binding| {
            serde_json::json!({
                "name": binding.name,
                "protocol": binding.protocol,
                "bind": binding.bind,
                "port": binding.port,
                "purpose": binding.purpose,
            })
        })
        .collect();

    let output = serde_json::json!({
        "primary_rpc_port": RPC_HTTP_PORT,
        "ports": ports,
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
        "support_model": {
            "evm_family": "partial",
            "wasm_family": "partial",
            "move_family": "partial",
            "utxo_family": "partial",
            "all_chains_full_compatibility": false
        },
        "compatibility": {
            "evm_chains": "bridge-compatible via aoxcvm::lanes::evm",
            "wasm_chains": "bridge-compatible via aoxcvm::lanes::wasm",
            "move_ecosystem": "bridge-compatible via aoxcvm::lanes::sui_move",
            "utxo_ecosystem": "bridge-compatible via aoxcvm::lanes::cardano"
        },
        "hard_limits": [
            "No relay chain can honestly guarantee 100% security",
            "Full compatibility with every chain requires chain-specific adapters, test vectors, and finality proofs"
        ],
        "note": "Deterministic coordination is implemented; production interoperability requires chain-specific bridge adapters, replay/finality validation, and audits."
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
    let native_symbol = arg_value(args, "--native-symbol").unwrap_or_else(|| "AOXC".to_string());
    let native_decimals: u8 = arg_value(args, "--native-decimals")
        .unwrap_or_else(|| "18".to_string())
        .parse()
        .map_err(|_| "--native-decimals must be a valid u8".to_string())?;
    let settlement_network =
        arg_value(args, "--settlement-network").unwrap_or_else(|| "xlayer".to_string());
    let settlement_token_address = arg_value(args, "--xlayer-token")
        .unwrap_or_else(|| "0xeb9580c3946bb47d73aae1d4f7a94148b554b2f4".to_string());
    let settlement_main_contract = arg_value(args, "--xlayer-main-contract")
        .unwrap_or_else(|| "0x97bdd1fd1caf756e00efd42eba9406821465b365".to_string());
    let settlement_multisig_contract = arg_value(args, "--xlayer-multisig")
        .unwrap_or_else(|| "0x20c0dd8b6559912acfac2ce061b8d5b19db8ca84".to_string());
    let equivalence_mode =
        arg_value(args, "--equivalence-mode").unwrap_or_else(|| "1:1".to_string());

    let mut config = GenesisConfig::new();
    config.chain_num = chain_num;
    config.chain_id = GenesisConfig::generate_chain_id(chain_num);
    config.block_time = block_time;
    config.treasury = treasury;
    config.settlement_link = SettlementLink {
        native_symbol,
        native_decimals,
        settlement_network,
        settlement_token_address,
        settlement_main_contract,
        settlement_multisig_contract,
        equivalence_mode,
    };
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
        "state_hash": loaded.config.state_hash(),
        "settlement_link": {
            "native_symbol": loaded.config.settlement_link.native_symbol,
            "native_decimals": loaded.config.settlement_link.native_decimals,
            "settlement_network": loaded.config.settlement_link.settlement_network,
            "settlement_token_address": loaded.config.settlement_link.settlement_token_address,
            "settlement_main_contract": loaded.config.settlement_link.settlement_main_contract,
            "settlement_multisig_contract": loaded.config.settlement_link.settlement_multisig_contract,
            "equivalence_mode": loaded.config.settlement_link.equivalence_mode
        }
    });

    println!(
        "{}",
        serde_json::to_string_pretty(&output)
            .map_err(|error| format!("JSON_SERIALIZE_ERROR: {error}"))?
    );

    Ok(())
}

#[derive(Debug, Clone, Serialize)]
struct TestnetFixtureAccount {
    slug: String,
    display_name: String,
    chain_num: u32,
    hd_path: String,
    master_seed_hex: String,
    node_seed_path: String,
    account_address: String,
    validator_id_hex: String,
    account_funding: String,
    p2p_listen_addr: String,
    rpc_addr: String,
    peers: Vec<String>,
    key_engine_fingerprint: String,
}

#[derive(Debug, Clone, Serialize)]
struct TestnetFixtureManifest {
    profile: String,
    chain_num: u32,
    chain_id: String,
    block_time_secs: u64,
    security_mode: String,
    fund_amount_per_account: String,
    warning: String,
    accounts: Vec<TestnetFixtureAccount>,
}

fn cmd_testnet_fixture_init(args: &[String]) -> Result<(), String> {
    let output_dir = arg_value(args, "--output-dir")
        .unwrap_or_else(|| "configs/deterministic-testnet".to_string());
    let chain_num: u32 = arg_value(args, "--chain-num")
        .unwrap_or_else(|| "77".to_string())
        .parse()
        .map_err(|_| "--chain-num must be a valid u32".to_string())?;
    let fund_amount: u128 = arg_value(args, "--fund-amount")
        .unwrap_or_else(|| "2500000000000000000000".to_string())
        .parse()
        .map_err(|_| "--fund-amount must be a valid u128".to_string())?;

    let manifest = build_testnet_fixture_manifest(chain_num, fund_amount)?;
    write_testnet_fixture(&output_dir, &manifest)?;

    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "output_dir": output_dir,
            "chain_id": manifest.chain_id,
            "account_count": manifest.accounts.len(),
            "accounts_file": format!("{}/accounts.json", output_dir),
            "genesis_file": format!("{}/genesis.json", output_dir),
            "launch_script": format!("{}/launch-testnet.sh", output_dir),
            "warning": manifest.warning,
        }))
        .map_err(|error| format!("JSON_SERIALIZE_ERROR: {error}"))?
    );

    Ok(())
}

#[derive(Debug, Clone, Serialize)]
struct MainnetReadinessControl {
    name: &'static str,
    area: &'static str,
    status: &'static str,
    weight: u8,
    rationale: &'static str,
}

fn cmd_load_benchmark(args: &[String]) -> Result<(), String> {
    let rounds: u64 = arg_value(args, "--rounds")
        .unwrap_or_else(|| "25".to_string())
        .parse()
        .map_err(|_| "--rounds must be a valid u64".to_string())?;
    let tx_per_block: usize = arg_value(args, "--tx-per-block")
        .unwrap_or_else(|| "50".to_string())
        .parse()
        .map_err(|_| "--tx-per-block must be a valid usize".to_string())?;
    let payload_bytes: usize = arg_value(args, "--payload-bytes")
        .unwrap_or_else(|| "256".to_string())
        .parse()
        .map_err(|_| "--payload-bytes must be a valid usize".to_string())?;
    let network_rounds: u64 = arg_value(args, "--network-rounds")
        .unwrap_or_else(|| "10".to_string())
        .parse()
        .map_err(|_| "--network-rounds must be a valid u64".to_string())?;
    let timeout_ms: u64 = arg_value(args, "--timeout-ms")
        .unwrap_or_else(|| "2000".to_string())
        .parse()
        .map_err(|_| "--timeout-ms must be a valid u64".to_string())?;

    if rounds == 0 {
        return Err("--rounds must be greater than zero".to_string());
    }
    if tx_per_block == 0 {
        return Err("--tx-per-block must be greater than zero".to_string());
    }
    if payload_bytes == 0 {
        return Err("--payload-bytes must be greater than zero".to_string());
    }

    let home = data_home::resolve_data_home(args);
    let mut node = state::setup_with_home(&home).map_err(|error| error.to_string())?;

    let started = Instant::now();
    let mut produced_blocks = 0u64;
    let mut failed_rounds = Vec::new();
    let mut last_height = 0u64;

    for round in 0..rounds {
        let payloads = (0..tx_per_block)
            .map(|tx_index| synthetic_benchmark_payload(round, tx_index, payload_bytes))
            .collect::<Vec<_>>();

        match produce_single_block(&mut node, payloads) {
            Ok(outcome) => {
                produced_blocks += 1;
                last_height = outcome.block.header.height;
            }
            Err(error) => failed_rounds.push(format!("round {}: {}", round + 1, error)),
        }
    }

    let elapsed = started.elapsed();
    let total_txs_attempted = rounds as usize * tx_per_block;
    let total_txs_committed = produced_blocks as usize * tx_per_block;
    let tx_per_sec = if elapsed.as_secs_f64() == 0.0 {
        0.0
    } else {
        total_txs_committed as f64 / elapsed.as_secs_f64()
    };
    let blocks_per_sec = if elapsed.as_secs_f64() == 0.0 {
        0.0
    } else {
        produced_blocks as f64 / elapsed.as_secs_f64()
    };

    let mut network_rtts = Vec::new();
    let network_payload = synthetic_benchmark_payload(0, 0, payload_bytes.min(1024));
    for _ in 0..network_rounds {
        let report =
            run_live_tcp_smoke_on("127.0.0.1:0", &network_payload, Duration::from_millis(timeout_ms))
                .map_err(|error| format!("NETWORK_BENCHMARK_ERROR: {error}"))?;
        network_rtts.push(report.round_trip_ms);
    }

    let avg_network_rtt_ms = if network_rtts.is_empty() {
        None
    } else {
        Some((network_rtts.iter().sum::<u128>() / network_rtts.len() as u128) as u64)
    };

    let output = serde_json::json!({
        "command": "load-benchmark",
        "scope": "single-process local synthetic benchmark",
        "home": home,
        "configuration": {
            "rounds": rounds,
            "tx_per_block": tx_per_block,
            "payload_bytes": payload_bytes,
            "network_rounds": network_rounds,
            "network_timeout_ms": timeout_ms,
        },
        "results": {
            "elapsed_ms": elapsed.as_millis() as u64,
            "blocks_requested": rounds,
            "blocks_produced": produced_blocks,
            "rounds_failed": failed_rounds.len(),
            "error_free": failed_rounds.is_empty(),
            "last_height": last_height,
            "tx_attempted": total_txs_attempted,
            "tx_committed": total_txs_committed,
            "blocks_per_sec": blocks_per_sec,
            "tx_per_sec": tx_per_sec,
        },
        "network": {
            "loopback_round_trip_ms": {
                "min": network_rtts.iter().min().copied(),
                "max": network_rtts.iter().max().copied(),
                "avg": avg_network_rtt_ms,
            }
        },
        "failures": failed_rounds,
        "note": "These numbers represent a local synthetic benchmark, not internet-scale mainnet throughput or adversarial-load certification.",
    });

    println!(
        "{}",
        serde_json::to_string_pretty(&output)
            .map_err(|error| format!("JSON_SERIALIZE_ERROR: {error}"))?
    );

    Ok(())
}

fn cmd_mainnet_readiness() -> Result<(), String> {
    let controls = mainnet_readiness_controls();
    let total_weight: u32 = controls.iter().map(|control| u32::from(control.weight)).sum();
    let achieved_weight: u32 = controls
        .iter()
        .filter(|control| control.status == "ready")
        .map(|control| u32::from(control.weight))
        .sum();
    let readiness_percent = if total_weight == 0 {
        0.0
    } else {
        (achieved_weight as f64 / total_weight as f64) * 100.0
    };

    let blockers = controls
        .iter()
        .filter(|control| control.status == "missing")
        .map(|control| format!("{} ({})", control.name, control.area))
        .collect::<Vec<_>>();

    let partials = controls
        .iter()
        .filter(|control| control.status == "partial")
        .map(|control| format!("{} ({})", control.name, control.area))
        .collect::<Vec<_>>();

    let output = serde_json::json!({
        "command": "mainnet-readiness",
        "readiness_percent": readiness_percent,
        "grade": readiness_grade(readiness_percent),
        "summary": readiness_summary(readiness_percent),
        "controls": controls,
        "hard_blockers": blockers,
        "partial_gaps": partials,
        "recommendations": [
            "Complete multi-host p2p tests and sustained peer churn recovery.",
            "Add adversarial partition/byzantine/fault-injection suites.",
            "Implement state sync, replay recovery, and snapshot restore validation.",
            "Add long-duration soak tests and public testnet telemetry/SLO dashboards.",
            "Validate real-world latency and throughput on multiple machines before any mainnet claim."
        ],
        "note": "This is an engineering readiness estimate, not a security audit or a guarantee of production safety."
    });

    println!(
        "{}",
        serde_json::to_string_pretty(&output)
            .map_err(|error| format!("JSON_SERIALIZE_ERROR: {error}"))?
    );

    Ok(())
}

fn mainnet_readiness_controls() -> Vec<MainnetReadinessControl> {
    vec![
        MainnetReadinessControl {
            name: "Deterministic genesis and test fixture",
            area: "bootstrap",
            status: "ready",
            weight: 10,
            rationale: "Deterministic local fixture, funded genesis, and reproducible node homes exist.",
        },
        MainnetReadinessControl {
            name: "Single-node block production path",
            area: "consensus",
            status: "ready",
            weight: 10,
            rationale: "Local block production/finalization path is implemented and covered by tests.",
        },
        MainnetReadinessControl {
            name: "Loopback transport smoke tests",
            area: "network",
            status: "ready",
            weight: 8,
            rationale: "TCP loopback path and repeated local network probes are available.",
        },
        MainnetReadinessControl {
            name: "Storage smoke path",
            area: "data",
            status: "ready",
            weight: 8,
            rationale: "Hybrid block storage smoke flow exists for local verification.",
        },
        MainnetReadinessControl {
            name: "Multi-host peer network validation",
            area: "network",
            status: "missing",
            weight: 15,
            rationale: "No evidence yet of sustained cross-host production-grade p2p validation.",
        },
        MainnetReadinessControl {
            name: "Partition, byzantine, and fault-injection tests",
            area: "resilience",
            status: "missing",
            weight: 15,
            rationale: "Adversarial recovery evidence is not present in the current repo.",
        },
        MainnetReadinessControl {
            name: "State sync and snapshot recovery",
            area: "operations",
            status: "missing",
            weight: 12,
            rationale: "State sync/replay/snapshot recovery needs explicit validation before mainnet.",
        },
        MainnetReadinessControl {
            name: "Long-duration soak and SLO telemetry",
            area: "operations",
            status: "partial",
            weight: 12,
            rationale: "There are runtime/health probes, but no evidence of long-duration audited soak benchmarks.",
        },
        MainnetReadinessControl {
            name: "Official release / attestation controls",
            area: "supply-chain",
            status: "partial",
            weight: 10,
            rationale: "Build attestation surfaces exist, but deployment discipline still depends on release process.",
        },
    ]
}

fn readiness_grade(percent: f64) -> &'static str {
    if percent >= 85.0 {
        "A"
    } else if percent >= 70.0 {
        "B"
    } else if percent >= 55.0 {
        "C"
    } else if percent >= 40.0 {
        "D"
    } else {
        "E"
    }
}

fn readiness_summary(percent: f64) -> &'static str {
    if percent >= 85.0 {
        "Close to production candidate, but still requires external validation."
    } else if percent >= 70.0 {
        "Strong pre-mainnet engineering base with several critical gaps still open."
    } else if percent >= 55.0 {
        "Mid-stage readiness: useful local/system validation exists, but mainnet blockers remain."
    } else {
        "Early-stage readiness: architecture exists, but operational and adversarial evidence is insufficient."
    }
}

fn synthetic_benchmark_payload(round: u64, tx_index: usize, payload_bytes: usize) -> Vec<u8> {
    let prefix = format!("AOXC_BENCH_{round}_{tx_index}_");
    let mut payload = prefix.into_bytes();

    while payload.len() < payload_bytes {
        payload.extend_from_slice(b"X");
    }

    payload.truncate(payload_bytes);
    payload
}

fn build_testnet_fixture_manifest(
    chain_num: u32,
    fund_amount: u128,
) -> Result<TestnetFixtureManifest, String> {
    let mut accounts = Vec::with_capacity(TESTNET_FIXTURE_MEMBERS.len());

    for (slug, display_name, p2p_port, rpc_port, zone, master_seed_hex) in TESTNET_FIXTURE_MEMBERS {
        let seed = decode_master_seed_hex(master_seed_hex)?;
        let key_engine = KeyEngine::from_seed(seed);
        let hd_path = HdPath::new(chain_num, 1, u32::from(zone), 0)
            .map_err(|error| format!("invalid deterministic hd path: {error}"))?;
        let entropy = key_engine.derive_entropy(&hd_path);
        let account_address = deterministic_address(slug, &entropy);
        let validator_id_hex = hex::encode_upper(&entropy[..32]);

        let peers = TESTNET_FIXTURE_MEMBERS
            .iter()
            .filter(|(peer_slug, ..)| peer_slug != &slug)
            .map(|(_, _, peer_p2p_port, ..)| format!("127.0.0.1:{peer_p2p_port}"))
            .collect();

        accounts.push(TestnetFixtureAccount {
            slug: slug.to_string(),
            display_name: display_name.to_string(),
            chain_num,
            hd_path: hd_path.to_string(),
            master_seed_hex: master_seed_hex.to_string(),
            node_seed_path: "identity/test-node-seed.hex".to_string(),
            account_address,
            validator_id_hex,
            account_funding: fund_amount.to_string(),
            p2p_listen_addr: format!("127.0.0.1:{p2p_port}"),
            rpc_addr: format!("127.0.0.1:{rpc_port}"),
            peers,
            key_engine_fingerprint: key_engine.fingerprint(),
        });
    }

    Ok(TestnetFixtureManifest {
        profile: "deterministic-testnet".to_string(),
        chain_num,
        chain_id: GenesisConfig::generate_chain_id(chain_num),
        block_time_secs: 4,
        security_mode: "mutual_auth_test_fixture".to_string(),
        fund_amount_per_account: fund_amount.to_string(),
        warning: "TEST ONLY: all seeds in this fixture are public and must never be used in production."
            .to_string(),
        accounts,
    })
}

fn write_testnet_fixture(output_dir: &str, manifest: &TestnetFixtureManifest) -> Result<(), String> {
    fs::create_dir_all(output_dir).map_err(|error| error.to_string())?;
    fs::create_dir_all(format!("{output_dir}/nodes")).map_err(|error| error.to_string())?;
    fs::create_dir_all(format!("{output_dir}/homes")).map_err(|error| error.to_string())?;

    let accounts_json = serde_json::to_vec_pretty(manifest)
        .map_err(|error| format!("JSON_SERIALIZE_ERROR: {error}"))?;
    fs::write(format!("{output_dir}/accounts.json"), accounts_json).map_err(|error| error.to_string())?;

    let mut genesis = GenesisConfig::new();
    genesis.chain_num = manifest.chain_num;
    genesis.chain_id = manifest.chain_id.clone();
    genesis.block_time = manifest.block_time_secs;
    genesis.treasury = 5_000_000_000_000;
    genesis.add_account(TREASURY_ACCOUNT.to_string(), genesis.treasury);
    for account in &manifest.accounts {
        genesis.add_account(
            account.account_address.clone(),
            account
                .account_funding
                .parse()
                .map_err(|_| "invalid account_funding in manifest".to_string())?,
        );
    }
    GenesisLoader::save(&genesis, format!("{output_dir}/genesis.json"))
        .map_err(|error| error.to_string())?;

    for account in &manifest.accounts {
        fs::write(
            format!("{output_dir}/nodes/{}.toml", account.slug),
            render_node_toml(account, manifest),
        )
        .map_err(|error| error.to_string())?;

        let home_identity_dir = format!("{output_dir}/homes/{}/identity", account.slug);
        fs::create_dir_all(&home_identity_dir).map_err(|error| error.to_string())?;
        fs::write(
            format!("{home_identity_dir}/test-node-seed.hex"),
            format!("{}\n", account.master_seed_hex),
        )
        .map_err(|error| error.to_string())?;
        fs::copy(
            format!("{output_dir}/genesis.json"),
            format!("{home_identity_dir}/genesis.json"),
        )
        .map_err(|error| error.to_string())?;
    }

    let launch_script_path = format!("{output_dir}/launch-testnet.sh");
    fs::write(&launch_script_path, render_launch_script(manifest)).map_err(|error| error.to_string())?;
    let permissions = fs::Permissions::from_mode(0o755);
    fs::set_permissions(&launch_script_path, permissions).map_err(|error| error.to_string())?;

    Ok(())
}

fn render_node_toml(account: &TestnetFixtureAccount, manifest: &TestnetFixtureManifest) -> String {
    let peers = account
        .peers
        .iter()
        .map(|peer| format!("  \"{peer}\""))
        .collect::<Vec<_>>()
        .join(",\n");

    format!(
        "chain_id = \"{}\"\nnode_name = \"{}\"\nlisten_addr = \"{}\"\nrpc_addr = \"{}\"\npeers = [\n{}\n]\nsecurity_mode = \"{}\"\nhd_path = \"{}\"\nvalidator_id_hex = \"{}\"\naccount_address = \"{}\"\nwarning = \"TEST ONLY - public fixture seed\"\n",
        manifest.chain_id,
        account.slug,
        account.p2p_listen_addr,
        account.rpc_addr,
        peers,
        manifest.security_mode,
        account.hd_path,
        account.validator_id_hex,
        account.account_address,
    )
}

fn render_launch_script(manifest: &TestnetFixtureManifest) -> String {
    let mut script = String::from(
        "#!/usr/bin/env bash\nset -euo pipefail\n\nROOT_DIR=\"$(cd \"$(dirname \"${BASH_SOURCE[0]}\")\" && pwd)\"\nAOXC_BIN=\"${AOXC_BIN:-cargo run -q -p aoxcmd --}\"\nROUNDS=\"${ROUNDS:-2}\"\nSLEEP_MS=\"${SLEEP_MS:-250}\"\n\necho \"[fixture] chain_id=",
    );
    script.push_str(&manifest.chain_id);
    script.push_str("\"\n");
    script.push_str("echo \"[fixture] TEST ONLY seeds are public; do not reuse outside local/dev environments.\"\n\n");

    for account in &manifest.accounts {
        let tx_prefix = format!("{}-TX", account.slug.to_uppercase());
        script.push_str(&format!(
            "echo \"[fixture] bootstrapping {slug}\" \n$AOXC_BIN node-bootstrap --home \"$ROOT_DIR/homes/{slug}\" >/tmp/aoxc-{slug}-bootstrap.json\n$AOXC_BIN node-run --home \"$ROOT_DIR/homes/{slug}\" --rounds \"$ROUNDS\" --sleep-ms \"$SLEEP_MS\" --tx-prefix \"{tx_prefix}\" >/tmp/aoxc-{slug}-run.json\n",
            slug = account.slug,
            tx_prefix = tx_prefix,
        ));
    }

    script
}

fn decode_master_seed_hex(value: &str) -> Result<[u8; MASTER_SEED_LEN], String> {
    let raw = hex::decode(value).map_err(|_| "fixture seed must be valid hex".to_string())?;
    if raw.len() != MASTER_SEED_LEN {
        return Err(format!(
            "fixture seed must be {MASTER_SEED_LEN} bytes, got {}",
            raw.len()
        ));
    }

    let mut seed = [0u8; MASTER_SEED_LEN];
    seed.copy_from_slice(&raw);
    Ok(seed)
}

fn deterministic_address(slug: &str, entropy: &[u8]) -> String {
    let mut hasher = Sha3_256::new();
    hasher.update(b"AOXC-TESTNET-FIXTURE-ADDRESS-V1");
    hasher.update([0x00]);
    hasher.update(slug.as_bytes());
    hasher.update([0x00]);
    hasher.update(entropy);
    let digest = hasher.finalize();
    format!("AOXC_TEST_{}", hex::encode_upper(&digest[..20]))
}

fn cmd_node_bootstrap(args: &[String]) -> Result<(), String> {
    let home = data_home::resolve_data_home(args);
    let node = state::setup_with_home(&home).map_err(|error| error.to_string())?;

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

    let home = data_home::resolve_data_home(args);
    let mut node = state::setup_with_home(&home).map_err(|error| error.to_string())?;
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

fn cmd_node_run(args: &[String]) -> Result<(), String> {
    let rounds: u64 = arg_value(args, "--rounds")
        .unwrap_or_else(|| "10".to_string())
        .parse()
        .map_err(|_| "--rounds must be a valid u64".to_string())?;
    let sleep_ms: u64 = arg_value(args, "--sleep-ms")
        .unwrap_or_else(|| "2000".to_string())
        .parse()
        .map_err(|_| "--sleep-ms must be a valid u64".to_string())?;
    let tx_prefix =
        arg_value(args, "--tx-prefix").unwrap_or_else(|| "AOXC_NODE_RUN_TX".to_string());

    let home = data_home::resolve_data_home(args);
    let mut node = state::setup_with_home(&home).map_err(|error| error.to_string())?;

    let mut produced = 0u64;
    let mut last_height = 0u64;
    let mut failures = Vec::new();

    for round in 0..rounds {
        let tx = format!("{}-{}", tx_prefix, round + 1);
        match produce_single_block(&mut node, vec![tx.into_bytes()]) {
            Ok(outcome) => {
                produced += 1;
                last_height = outcome.block.header.height;
            }
            Err(error) => failures.push(format!("round {}: {}", round + 1, error)),
        }

        if round + 1 < rounds {
            thread::sleep(Duration::from_millis(sleep_ms));
        }
    }

    let output = serde_json::json!({
        "mode": "continuous-local-node-run",
        "rounds_requested": rounds,
        "rounds_produced": produced,
        "rounds_failed": failures.len(),
        "sleep_ms": sleep_ms,
        "final_height": last_height,
        "errors": failures,
    });

    println!(
        "{}",
        serde_json::to_string_pretty(&output)
            .map_err(|error| format!("JSON_SERIALIZE_ERROR: {error}"))?
    );

    Ok(())
}

fn cmd_network_smoke(args: &[String]) -> Result<(), String> {
    let timeout_ms: u64 = arg_value(args, "--timeout-ms")
        .unwrap_or_else(|| "3000".to_string())
        .parse()
        .map_err(|_| "--timeout-ms must be a valid u64".to_string())?;

    let payload = arg_value(args, "--payload")
        .unwrap_or_else(|| "AOXC_LIVE_TCP_PING".to_string())
        .into_bytes();

    let bind_host = arg_value(args, "--bind-host").unwrap_or_else(|| "127.0.0.1".to_string());
    let bind_port: u16 = arg_value(args, "--port")
        .unwrap_or_else(|| LIVE_SMOKE_TEST_PORT.to_string())
        .parse()
        .map_err(|_| "--port must be a valid u16".to_string())?;

    let bind_addr = format!("{bind_host}:{bind_port}");

    let report = run_live_tcp_smoke_on(&bind_addr, &payload, Duration::from_millis(timeout_ms))
        .map_err(|error| format!("NETWORK_LIVE_SMOKE_ERROR: {error}"))?;

    let output = serde_json::json!({
        "transport": "tcp",
        "mode": "live-loopback-socket",
        "listener": report.listener_addr.to_string(),
        "bytes_sent": report.bytes_sent,
        "bytes_received": report.bytes_received,
        "payload_echoed": report.payload_echoed,
        "round_trip_ms": report.round_trip_ms,
    });

    println!(
        "{}",
        serde_json::to_string_pretty(&output)
            .map_err(|error| format!("JSON_SERIALIZE_ERROR: {error}"))?
    );

    Ok(())
}

fn cmd_real_network(args: &[String]) -> Result<(), String> {
    let rounds: u64 = arg_value(args, "--rounds")
        .unwrap_or_else(|| "5".to_string())
        .parse()
        .map_err(|_| "--rounds must be a valid u64".to_string())?;
    let timeout_ms: u64 = arg_value(args, "--timeout-ms")
        .unwrap_or_else(|| "3000".to_string())
        .parse()
        .map_err(|_| "--timeout-ms must be a valid u64".to_string())?;
    let pause_ms: u64 = arg_value(args, "--pause-ms")
        .unwrap_or_else(|| "250".to_string())
        .parse()
        .map_err(|_| "--pause-ms must be a valid u64".to_string())?;

    let payload = arg_value(args, "--payload")
        .unwrap_or_else(|| "AOXC_REAL_NETWORK_PROBE".to_string())
        .into_bytes();
    let bind_host = arg_value(args, "--bind-host").unwrap_or_else(|| "127.0.0.1".to_string());
    let bind_port: u16 = arg_value(args, "--port")
        .unwrap_or_else(|| "0".to_string())
        .parse()
        .map_err(|_| "--port must be a valid u16".to_string())?;

    let bind_addr = format!("{bind_host}:{bind_port}");
    let mut passes = 0u64;
    let mut failures = Vec::new();
    let mut rtts: Vec<u128> = Vec::new();

    for round in 0..rounds {
        match run_live_tcp_smoke_on(&bind_addr, &payload, Duration::from_millis(timeout_ms)) {
            Ok(report) => {
                if report.payload_echoed {
                    passes += 1;
                    rtts.push(report.round_trip_ms);
                } else {
                    failures.push(format!("round {}: payload mismatch", round + 1));
                }
            }
            Err(error) => failures.push(format!("round {}: {}", round + 1, error)),
        }

        if round + 1 < rounds {
            thread::sleep(Duration::from_millis(pause_ms));
        }
    }

    let avg_rtt = if rtts.is_empty() {
        None
    } else {
        Some((rtts.iter().sum::<u128>() / rtts.len() as u128) as u64)
    };

    let output = serde_json::json!({
        "command": "real-network",
        "mode": "multi-round-live-tcp-probe",
        "rounds_requested": rounds,
        "rounds_passed": passes,
        "rounds_failed": failures.len(),
        "success_ratio": if rounds == 0 { 0.0 } else { passes as f64 / rounds as f64 },
        "bind_addr": bind_addr,
        "timeout_ms": timeout_ms,
        "pause_ms": pause_ms,
        "rtt_ms": {
            "min": rtts.iter().min().copied(),
            "max": rtts.iter().max().copied(),
            "avg": avg_rtt,
        },
        "failures": failures,
        "note": "This command validates repeated live TCP behavior. For internet-grade production readiness, run multi-host peer tests with partition/recovery scenarios.",
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
    let assessment = interop_assessment();

    let output = serde_json::json!({
        "assessment": {
            "estimated_readiness_percent": assessment.estimated_readiness_percent,
            "status": assessment.status,
            "ready_for_all_chains": assessment.ready_for_all_chains,
            "can_claim_100_percent_security": assessment.can_claim_100_percent_security,
        },
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
        ],
        "implemented_controls": assessment.implemented_controls,
        "missing_critical_controls": assessment.missing_critical_controls,
        "hard_blockers": assessment.hard_blockers,
        "next_priority_actions": assessment.next_priority_actions
    });

    println!(
        "{}",
        serde_json::to_string_pretty(&output).map_err(|e| format!("JSON_SERIALIZE_ERROR: {e}"))?
    );

    Ok(())
}

#[derive(Debug, Clone)]
struct InteropAssessment {
    estimated_readiness_percent: f64,
    status: &'static str,
    ready_for_all_chains: bool,
    can_claim_100_percent_security: bool,
    implemented_controls: Vec<&'static str>,
    missing_critical_controls: Vec<&'static str>,
    hard_blockers: Vec<&'static str>,
    next_priority_actions: Vec<&'static str>,
}

fn interop_assessment() -> InteropAssessment {
    InteropAssessment {
        estimated_readiness_percent: 38.0,
        status: "pre-mainnet-hardening",
        ready_for_all_chains: false,
        can_claim_100_percent_security: false,
        implemented_controls: vec![
            "relay-oriented multi-crate architecture",
            "multi-lane execution model (EVM/WASM/Sui Move/Cardano UTXO)",
            "runtime health/readiness and telemetry surfaces",
            "mainnet key generation explicit opt-in guard",
            "production audit CLI surface",
        ],
        missing_critical_controls: vec![
            "independent external security audit with remediation closure",
            "continuous fuzz/property testing for bridge and serialization paths",
            "deterministic replay suite across historical state transitions",
            "multi-node adversarial consensus and partition recovery tests",
            "chain-specific bridge adapter conformance vectors",
            "signed release artifacts, SBOM, and provenance attestation",
        ],
        hard_blockers: vec![
            "No proof that relay logic is safe against all target-chain finality differences",
            "No evidence of completed external audit closure for core/bridge/network paths",
            "No evidence of exhaustive cross-chain compatibility vectors per target family",
        ],
        next_priority_actions: vec![
            "Add 3+ node deterministic adversarial simulation suite",
            "Add replay fixtures and bridge proof failure-injection tests",
            "Add release signing, SBOM generation, and provenance verification",
            "Publish chain-family-specific compatibility matrices and acceptance criteria",
        ],
    }
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

    let node = state::setup_with_home(&home).map_err(|error| error.to_string())?;

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

#[cfg(test)]
mod tests {
    use super::{
        BuildInfo, CliLanguage, ai_control_score, arg_bool_value, assert_mainnet_key_policy,
        bootstrap_defaults, build_manifest_payload, build_testnet_fixture_manifest,
        detect_language, interop_assessment, is_official_release, localized_unknown_command,
        mainnet_readiness_controls, node_connection_policy_payload, readiness_grade,
        render_launch_script, synthetic_benchmark_payload, usage_text, version_payload,
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
    fn usage_text_mentions_port_map_and_network_port_override() {
        let usage = usage_text(CliLanguage::En);
        assert!(usage.contains("port-map"));
        assert!(usage.contains("build-manifest"));
        assert!(usage.contains("node-connection-policy"));
        assert!(usage.contains("sovereign-core"));
        assert!(usage.contains("module-architecture"));
        assert!(usage.contains("load-benchmark"));
        assert!(usage.contains("mainnet-readiness"));
        assert!(
            usage.contains("network-smoke [--timeout-ms <u64>] [--bind-host <addr>] [--port <u16>] [--payload <text>]")
        );
        assert!(usage.contains("real-network [--rounds <u64>] [--timeout-ms <u64>] [--pause-ms <u64>] [--bind-host <addr>] [--port <u16>] [--payload <text>]"));
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

    #[test]
    fn interop_assessment_is_explicitly_not_full_or_universal() {
        let assessment = interop_assessment();

        assert!(assessment.estimated_readiness_percent < 100.0);
        assert!(!assessment.ready_for_all_chains);
        assert!(!assessment.can_claim_100_percent_security);
        assert!(!assessment.hard_blockers.is_empty());
        assert!(!assessment.missing_critical_controls.is_empty());
    }

    #[test]
    fn official_release_policy_requires_clean_certified_stable_build() {
        let official = BuildInfo {
            semver: "0.1.0",
            git_commit: "abc123",
            git_dirty: "false",
            source_date_epoch: "123456",
            build_profile: "release",
            release_channel: "stable",
            attestation_hash: "a".repeat(64).leak(),
            cert_path: "/tmp/server.crt",
            cert_sha256: "b".repeat(64).leak(),
            cert_error: "none",
        };
        assert!(is_official_release(&official));

        let unofficial = BuildInfo {
            git_dirty: "true",
            ..official
        };
        assert!(!is_official_release(&unofficial));
    }

    #[test]
    fn version_payload_contains_release_name_and_attestation_hash() {
        let build = BuildInfo::collect();
        let payload = version_payload(&build);

        assert_eq!(payload["release_name"], "AOXC Alpha: Genesis V1");
        assert!(payload["attestation_hash"].as_str().is_some());
    }

    #[test]
    fn build_manifest_payload_contains_supply_chain_policy() {
        let build = BuildInfo::collect();
        let payload = build_manifest_payload(&build);

        assert_eq!(
            payload["artifact"]["release_name"],
            "AOXC Alpha: Genesis V1"
        );
        assert_eq!(
            payload["supply_chain_policy"]["accept_unofficial_node_builds"],
            false
        );
        assert_eq!(
            payload["supply_chain_policy"]["requires_attestation_hash"],
            true
        );
    }

    #[test]
    fn node_connection_policy_payload_requires_mtls() {
        let build = BuildInfo::collect();
        let payload = node_connection_policy_payload(&build);

        assert_eq!(
            payload["local_build"]["release_name"],
            "AOXC Alpha: Genesis V1"
        );
        assert_eq!(payload["accepted_remote_policy"]["require_mtls"], true);
        assert_eq!(
            payload["accepted_remote_policy"]["allow_unofficial_remote_builds"],
            false
        );
    }

    #[test]
    fn testnet_fixture_manifest_contains_five_named_accounts() {
        let manifest =
            build_testnet_fixture_manifest(77, 2_500_000_000_000_000_000_000).expect("fixture");

        assert_eq!(manifest.accounts.len(), 5);
        assert_eq!(manifest.chain_id, "AOXC-0077-MAIN");
        assert_eq!(manifest.accounts[0].slug, "atlas");
        assert!(manifest.accounts.iter().all(|account| {
            account.master_seed_hex.len() == 128
                && account.account_address.starts_with("AOXC_TEST_")
                && account.validator_id_hex.len() == 64
        }));
    }

    #[test]
    fn testnet_launch_script_mentions_each_fixture_home() {
        let manifest =
            build_testnet_fixture_manifest(77, 2_500_000_000_000_000_000_000).expect("fixture");
        let script = render_launch_script(&manifest);

        assert!(script.contains("homes/atlas"));
        assert!(script.contains("homes/boreal"));
        assert!(script.contains("homes/cypher"));
        assert!(script.contains("homes/delta"));
        assert!(script.contains("homes/ember"));
        assert!(script.contains("TEST ONLY seeds are public"));
    }

    #[test]
    fn readiness_controls_include_hard_blockers() {
        let controls = mainnet_readiness_controls();
        assert!(controls.iter().any(|control| control.status == "missing"));
        assert_eq!(readiness_grade(42.0), "D");
        assert_eq!(readiness_grade(88.0), "A");
    }

    #[test]
    fn synthetic_benchmark_payload_has_requested_size() {
        let payload = synthetic_benchmark_payload(3, 9, 128);
        assert_eq!(payload.len(), 128);
        assert!(String::from_utf8_lossy(&payload).starts_with("AOXC_BENCH_3_9_"));
    }
}
