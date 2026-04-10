// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use crate::{
    cli_support::{arg_value, detect_language, localized_unknown_command, print_usage},
    data_home::ScopedHomeOverride,
    error::{AppError, ErrorCode},
};

pub(crate) mod audit;
pub(crate) mod bootstrap;
pub(crate) mod db;
pub(crate) mod describe;
pub(crate) mod evidence;
pub(crate) mod ops;

pub(crate) const AOXC_RELEASE_NAME: &str = "AOXC Mainnet-Candidate Operator Plane";
pub(crate) const TESTNET_FIXTURE_MEMBERS: [(&str, &str, u16, u16, u16, &str); 5] = [
    ("atlas", "Atlas Validator", 39001, 19101, 1, "atlas-seed"),
    ("boreal", "Boreal Validator", 39002, 19102, 2, "boreal-seed"),
    ("cypher", "Cypher Validator", 39003, 19103, 3, "cypher-seed"),
    ("delta", "Delta Validator", 39004, 19104, 4, "delta-seed"),
    ("ember", "Ember Validator", 39005, 19105, 5, "ember-seed"),
];

/// Executes the AOXC operator CLI command surface.
///
/// Operational objectives:
/// - Preserve deterministic command routing from the raw process argument vector.
/// - Detect language preference before usage or unknown-command handling.
/// - Scope `--home` overrides strictly to the current CLI execution.
pub fn run_cli() -> Result<(), AppError> {
    let args: Vec<String> = std::env::args().collect();
    let lang = detect_language(&args[1..]);

    let home_override = arg_value(&args[1..], "--home")
        .filter(|value| !value.trim().is_empty())
        .map(std::path::PathBuf::from);

    let _home_override_guard = home_override.as_deref().map(ScopedHomeOverride::install);

    if args.len() < 2 {
        print_usage(lang);
        return Ok(());
    }

    match args[1].as_str() {
        "chain" => route_chain_group(&args[2..]),
        "genesis" => route_genesis_group(&args[2..]),
        "validator" => route_validator_group(&args[2..]),
        "wallet" => route_wallet_group(&args[2..]),
        "account" => route_account_group(&args[2..]),
        "node" => route_node_group(&args[2..]),
        "network" => route_network_group(&args[2..]),
        "role" => route_role_group(&args[2..]),
        "api" => route_api_group(&args[2..]),
        "query" => route_query_group(&args[2..]),
        "tx" => route_tx_group(&args[2..]),
        "stake" => route_stake_group(&args[2..]),
        "doctor" => route_doctor_group(&args[2..]),
        "audit" => route_audit_group(&args[2..]),
        "version" | "--version" | "-V" => describe::cmd_version(),
        "help" | "--help" | "-h" => {
            print_usage(lang);
            Ok(())
        }
        "vision" => describe::cmd_vision(),
        "build-manifest" => describe::cmd_build_manifest(),
        "node-connection-policy" => describe::cmd_node_connection_policy(&args[2..]),
        "sovereign-core" => describe::cmd_sovereign_core(),
        "module-architecture" => describe::cmd_module_architecture(),
        "compat-matrix" => describe::cmd_compat_matrix(),
        "quantum-blueprint" => describe::cmd_quantum_blueprint(),
        "quantum-posture" => describe::cmd_quantum_posture(&args[2..]),
        "port-map" => describe::cmd_port_map(),
        "api-contract" => ops::cmd_api_contract(&args[2..]),
        "profile-baseline" => ops::cmd_profile_baseline(&args[2..]),
        "testnet-fixture-init" => bootstrap::cmd_testnet_fixture_init(&args[2..]),
        "load-benchmark" => ops::cmd_load_benchmark(&args[2..]),
        "mainnet-readiness" => ops::cmd_mainnet_readiness(&args[2..]),
        "testnet-readiness" => ops::cmd_testnet_readiness(&args[2..]),
        "full-surface-readiness" => ops::cmd_full_surface_readiness(&args[2..]),
        "full-surface-gate" => ops::cmd_full_surface_gate(&args[2..]),
        "network-identity-gate" => ops::cmd_network_identity_gate(&args[2..]),
        "level-score" => ops::cmd_level_score(&args[2..]),
        "operator-evidence-record" => evidence::cmd_operator_evidence_record(&args[2..]),
        "operator-evidence-list" => evidence::cmd_operator_evidence_list(&args[2..]),
        "key-bootstrap" => bootstrap::cmd_key_bootstrap(&args[2..]),
        "key-rotate" => bootstrap::cmd_key_rotate(&args[2..]),
        "keys-inspect" => bootstrap::cmd_keys_inspect(&args[2..]),
        "keys-show-fingerprint" => bootstrap::cmd_keys_show_fingerprint(&args[2..]),
        "keys-verify" => bootstrap::cmd_keys_verify(&args[2..]),
        "address-create" => bootstrap::cmd_address_create(&args[2..]),
        "genesis-init" => bootstrap::cmd_genesis_init(&args[2..]),
        "genesis-add-account" => bootstrap::cmd_genesis_add_account(&args[2..]),
        "genesis-add-validator" => bootstrap::cmd_genesis_add_validator(&args[2..]),
        "genesis-validate" => bootstrap::cmd_genesis_validate(&args[2..]),
        "genesis-inspect" => bootstrap::cmd_genesis_inspect(&args[2..]),
        "genesis-template-advanced" => bootstrap::cmd_genesis_template_advanced(&args[2..]),
        "genesis-advanced-system" => bootstrap::cmd_genesis_advanced_system(&args[2..]),
        "genesis-security-audit" => bootstrap::cmd_genesis_security_audit(&args[2..]),
        "genesis-hash" => bootstrap::cmd_genesis_hash(&args[2..]),
        "genesis-start" => bootstrap::cmd_genesis_start(&args[2..]),
        "genesis-production-gate" => bootstrap::cmd_genesis_production_gate(&args[2..]),
        "config-init" => bootstrap::cmd_config_init(&args[2..]),
        "config-validate" => bootstrap::cmd_config_validate(&args[2..]),
        "config-print" => bootstrap::cmd_config_print(&args[2..]),
        "production-bootstrap" => bootstrap::cmd_production_bootstrap(&args[2..]),
        "topology-bootstrap" => bootstrap::cmd_topology_bootstrap(&args[2..]),
        "dual-profile-bootstrap" => bootstrap::cmd_dual_profile_bootstrap(&args[2..]),
        "node-bootstrap" => ops::cmd_node_bootstrap(&args[2..]),
        "produce-once" => ops::cmd_produce_once(&args[2..]),
        "node-run" => ops::cmd_node_run(&args[2..]),
        "node-health" => ops::cmd_node_health(&args[2..]),
        "network-smoke" => ops::cmd_network_smoke(&args[2..]),
        "network-join-check" => ops::cmd_network_join_check(&args[2..]),
        "real-network" => ops::cmd_real_network(&args[2..]),
        "storage-smoke" => ops::cmd_storage_smoke(&args[2..]),
        "db-init" => db::cmd_db_init(&args[2..]),
        "db-status" => db::cmd_db_status(&args[2..]),
        "db-put-block" => db::cmd_db_put_block(&args[2..]),
        "db-get-height" => db::cmd_db_get_height(&args[2..]),
        "db-get-hash" => db::cmd_db_get_hash(&args[2..]),
        "db-compact" => db::cmd_db_compact(&args[2..]),
        "economy-init" => ops::cmd_economy_init(&args[2..]),
        "treasury-transfer" => ops::cmd_treasury_transfer(&args[2..]),
        "stake-delegate" => ops::cmd_stake_delegate(&args[2..]),
        "stake-undelegate" => ops::cmd_stake_undelegate(&args[2..]),
        "validator-join" => ops::cmd_validator_join(&args[2..]),
        "validator-activate" => ops::cmd_validator_activate(&args[2..]),
        "validator-bond" => ops::cmd_validator_bond(&args[2..]),
        "validator-unbond" => ops::cmd_validator_unbond(&args[2..]),
        "validator-set-status" => ops::cmd_validator_set_status(&args[2..]),
        "validator-commission-set" => ops::cmd_validator_commission_set(&args[2..]),
        "economy-status" => ops::cmd_economy_status(&args[2..]),
        "faucet-status" => ops::cmd_faucet_status(&args[2..]),
        "faucet-history" => ops::cmd_faucet_history(&args[2..]),
        "faucet-balance" => ops::cmd_faucet_balance(&args[2..]),
        "faucet-enable" => ops::cmd_faucet_enable(&args[2..]),
        "faucet-disable" => ops::cmd_faucet_disable(&args[2..]),
        "faucet-config-show" => ops::cmd_faucet_config_show(&args[2..]),
        "faucet-audit" => ops::cmd_faucet_audit(&args[2..]),
        "faucet-config" => ops::cmd_faucet_config(&args[2..]),
        "faucet-claim" => ops::cmd_faucet_claim(&args[2..]),
        "faucet-reset" => ops::cmd_faucet_reset(&args[2..]),
        "runtime-status" => ops::cmd_runtime_status(&args[2..]),
        "runtime-snapshot" => ops::cmd_runtime_snapshot(&args[2..]),
        "runtime-snapshot-list" => ops::cmd_runtime_snapshot_list(&args[2..]),
        "runtime-snapshot-prune" => ops::cmd_runtime_snapshot_prune(&args[2..]),
        "runtime-restore-latest" => ops::cmd_runtime_restore_latest(&args[2..]),
        "chain-status" => ops::cmd_chain_status(&args[2..]),
        "consensus-status" => ops::cmd_consensus_status(&args[2..]),
        "consensus-validators" => ops::cmd_consensus_validators(&args[2..]),
        "consensus-proposer" => ops::cmd_consensus_proposer(&args[2..]),
        "consensus-round" => ops::cmd_consensus_round(&args[2..]),
        "consensus-finality" => ops::cmd_consensus_finality(&args[2..]),
        "consensus-commits" => ops::cmd_consensus_commits(&args[2..]),
        "consensus-evidence" => ops::cmd_consensus_evidence(&args[2..]),
        "vm-status" => ops::cmd_vm_status(&args[2..]),
        "vm-call" => ops::cmd_vm_call(&args[2..]),
        "vm-simulate" => ops::cmd_vm_simulate(&args[2..]),
        "vm-storage-get" => ops::cmd_vm_storage_get(&args[2..]),
        "vm-contract-get" => ops::cmd_vm_contract_get(&args[2..]),
        "vm-code-get" => ops::cmd_vm_code_get(&args[2..]),
        "vm-estimate-gas" => ops::cmd_vm_estimate_gas(&args[2..]),
        "vm-trace" => ops::cmd_vm_trace(&args[2..]),
        "block-get" => ops::cmd_block_get(&args[2..]),
        "tx-get" => ops::cmd_tx_get(&args[2..]),
        "tx-receipt" => ops::cmd_tx_receipt(&args[2..]),
        "account-get" => ops::cmd_account_get(&args[2..]),
        "balance-get" => ops::cmd_balance_get(&args[2..]),
        "peer-list" => ops::cmd_peer_list(&args[2..]),
        "network-status" => ops::cmd_network_status(&args[2..]),
        "state-root" => ops::cmd_state_root(&args[2..]),
        "metrics" => ops::cmd_metrics(&args[2..]),
        "rpc-status" => ops::cmd_rpc_status(&args[2..]),
        "rpc-curl-smoke" => ops::cmd_rpc_curl_smoke(&args[2..]),
        "rpc-serve" => ops::cmd_rpc_serve(&args[2..]),
        "diagnostics-doctor" => audit::cmd_diagnostics_doctor(&args[2..]),
        "diagnostics-bundle" => audit::cmd_diagnostics_bundle(&args[2..]),
        "interop-readiness" => audit::cmd_interop_readiness(&args[2..]),
        "interop-gate" => audit::cmd_interop_gate(&args[2..]),
        "production-audit" => audit::cmd_production_audit(&args[2..]),
        other => Err(localized_unknown_command(lang, other)),
    }
}

fn route_chain_group(args: &[String]) -> Result<(), AppError> {
    let Some((subcommand, tail)) = args.split_first() else {
        return invalid_group_usage("chain", "missing subcommand");
    };

    match subcommand.as_str() {
        "init" => bootstrap::cmd_config_init(tail),
        "create" => bootstrap::cmd_production_bootstrap(tail),
        "start" => ops::cmd_node_run(tail),
        "status" => ops::cmd_runtime_status(tail),
        "doctor" => audit::cmd_diagnostics_doctor(tail),
        "consensus-audit" => bootstrap::cmd_consensus_profile_audit(tail),
        "demo" => ops::cmd_real_network(tail),
        _ => invalid_group_usage("chain", "unsupported subcommand"),
    }
}

fn route_genesis_group(args: &[String]) -> Result<(), AppError> {
    let Some((subcommand, tail)) = args.split_first() else {
        return invalid_group_usage("genesis", "missing subcommand");
    };

    match subcommand.as_str() {
        "init" => bootstrap::cmd_genesis_init(tail),
        "add-validator" => bootstrap::cmd_genesis_add_validator(tail),
        "add-account" => bootstrap::cmd_genesis_add_account(tail),
        "build" | "verify" => bootstrap::cmd_genesis_validate(tail),
        "finalize" | "seal" | "sign" | "freeze" => bootstrap::cmd_genesis_production_gate(tail),
        "inspect" => bootstrap::cmd_genesis_inspect(tail),
        "template-advanced" => bootstrap::cmd_genesis_template_advanced(tail),
        "advanced-system" => bootstrap::cmd_genesis_advanced_system(tail),
        "security-audit" => bootstrap::cmd_genesis_security_audit(tail),
        "production-gate" => bootstrap::cmd_genesis_production_gate(tail),
        "start" => bootstrap::cmd_genesis_start(tail),
        "fingerprint" => bootstrap::cmd_genesis_hash(tail),
        _ => invalid_group_usage("genesis", "unsupported subcommand"),
    }
}

fn route_validator_group(args: &[String]) -> Result<(), AppError> {
    let Some((subcommand, tail)) = args.split_first() else {
        return invalid_group_usage("validator", "missing subcommand");
    };

    match subcommand.as_str() {
        "create" => bootstrap::cmd_key_bootstrap(tail),
        "join" | "register" => ops::cmd_validator_join(tail),
        "activate" | "bond" => ops::cmd_validator_activate(tail),
        "unbond" => ops::cmd_validator_unbond(tail),
        "set-status" => ops::cmd_validator_set_status(tail),
        "commission-set" => ops::cmd_validator_commission_set(tail),
        "inspect" | "status" => bootstrap::cmd_keys_inspect(tail),
        "rotate-key" => bootstrap::cmd_key_rotate(tail),
        _ => invalid_group_usage("validator", "unsupported subcommand"),
    }
}

fn route_wallet_group(args: &[String]) -> Result<(), AppError> {
    let Some((subcommand, tail)) = args.split_first() else {
        return invalid_group_usage("wallet", "missing subcommand");
    };

    match subcommand.as_str() {
        "create" => bootstrap::cmd_address_create(tail),
        "balance" => ops::cmd_economy_status(tail),
        _ => invalid_group_usage("wallet", "unsupported subcommand"),
    }
}

fn route_account_group(args: &[String]) -> Result<(), AppError> {
    let Some((subcommand, tail)) = args.split_first() else {
        return invalid_group_usage("account", "missing subcommand");
    };

    match subcommand.as_str() {
        "fund" => {
            let mapped = remap_flags(
                tail,
                &[
                    ("--to", "--to"),
                    ("--amount", "--amount"),
                    ("--from", "--from"),
                ],
            );
            ops::cmd_treasury_transfer(&mapped)
        }
        _ => invalid_group_usage("account", "unsupported subcommand"),
    }
}

fn route_node_group(args: &[String]) -> Result<(), AppError> {
    let Some((subcommand, tail)) = args.split_first() else {
        return invalid_group_usage("node", "missing subcommand");
    };

    match subcommand.as_str() {
        "init" => ops::cmd_node_bootstrap(tail),
        "join" => ops::cmd_node_join(tail),
        "start" => ops::cmd_node_run(tail),
        "status" => ops::cmd_node_health(tail),
        "doctor" => audit::cmd_diagnostics_doctor(tail),
        _ => invalid_group_usage("node", "unsupported subcommand"),
    }
}

fn route_network_group(args: &[String]) -> Result<(), AppError> {
    let Some((subcommand, tail)) = args.split_first() else {
        return invalid_group_usage("network", "missing subcommand");
    };

    match subcommand.as_str() {
        "create" => bootstrap::cmd_dual_profile_bootstrap(tail),
        "join" | "peer-add" | "seed-add" | "bootstrap-peer-add" => route_node_join_alias(tail),
        "start" => ops::cmd_real_network(tail),
        "status" | "verify" => ops::cmd_network_smoke(tail),
        "join-check" => ops::cmd_network_join_check(tail),
        "identity-gate" => ops::cmd_network_identity_gate(tail),
        "doctor" => audit::cmd_diagnostics_doctor(tail),
        _ => invalid_group_usage("network", "unsupported subcommand"),
    }
}

fn route_role_group(args: &[String]) -> Result<(), AppError> {
    let Some((subcommand, tail)) = args.split_first() else {
        return ops::cmd_role_model_status(args);
    };

    match subcommand.as_str() {
        "list" => ops::cmd_role_list(tail),
        "status" | "model-status" => ops::cmd_role_model_status(tail),
        "activate-core7" => ops::cmd_role_activate_core7(tail),
        _ => invalid_group_usage("role", "unsupported subcommand"),
    }
}

fn route_query_group(args: &[String]) -> Result<(), AppError> {
    let Some((subcommand, tail)) = args.split_first() else {
        return invalid_group_usage("query", "missing subcommand");
    };

    match subcommand.as_str() {
        "chain" => route_query_chain_group(tail),
        "consensus" => route_query_consensus_group(tail),
        "vm" => route_query_vm_group(tail),
        "full" => ops::cmd_query_full(tail),
        "block" => ops::cmd_block_get(tail),
        "tx" => ops::cmd_tx_get(tail),
        "receipt" => ops::cmd_tx_receipt(tail),
        "account" => ops::cmd_account_get(tail),
        "balance" => ops::cmd_balance_get(tail),
        "network" => route_query_network_group(tail),
        "runtime" => ops::cmd_query_runtime(tail),
        "state-root" => ops::cmd_state_root(tail),
        "rpc" => ops::cmd_rpc_status(tail),
        _ => invalid_group_usage("query", "unsupported subcommand"),
    }
}

fn route_query_chain_group(args: &[String]) -> Result<(), AppError> {
    let Some((subcommand, tail)) = args.split_first() else {
        return ops::cmd_chain_status(args);
    };

    match subcommand.as_str() {
        "status" => ops::cmd_chain_status(tail),
        "block" => ops::cmd_block_get(tail),
        "tx" => ops::cmd_tx_get(tail),
        "receipt" => ops::cmd_tx_receipt(tail),
        _ => invalid_group_usage("query chain", "unsupported subcommand"),
    }
}

fn route_query_consensus_group(args: &[String]) -> Result<(), AppError> {
    let Some((subcommand, tail)) = args.split_first() else {
        return ops::cmd_consensus_status(args);
    };

    match subcommand.as_str() {
        "status" => ops::cmd_consensus_status(tail),
        "validators" | "validator-set" => ops::cmd_consensus_validators(tail),
        "proposer" => ops::cmd_consensus_proposer(tail),
        "round" => ops::cmd_consensus_round(tail),
        "finality" => ops::cmd_consensus_finality(tail),
        "commits" => ops::cmd_consensus_commits(tail),
        "evidence" => ops::cmd_consensus_evidence(tail),
        _ => invalid_group_usage("query consensus", "unsupported subcommand"),
    }
}

fn route_query_vm_group(args: &[String]) -> Result<(), AppError> {
    let Some((subcommand, tail)) = args.split_first() else {
        return ops::cmd_vm_status(args);
    };

    match subcommand.as_str() {
        "status" => ops::cmd_vm_status(tail),
        "call" => ops::cmd_vm_call(tail),
        "simulate" => ops::cmd_vm_simulate(tail),
        "storage" | "storage-get" => ops::cmd_vm_storage_get(tail),
        "contract" | "contract-get" => ops::cmd_vm_contract_get(tail),
        "code" | "code-get" => ops::cmd_vm_code_get(tail),
        "estimate-gas" => ops::cmd_vm_estimate_gas(tail),
        "trace" => ops::cmd_vm_trace(tail),
        _ => invalid_group_usage("query vm", "unsupported subcommand"),
    }
}

fn route_query_network_group(args: &[String]) -> Result<(), AppError> {
    let Some((subcommand, tail)) = args.split_first() else {
        return ops::cmd_network_full(args);
    };

    match subcommand.as_str() {
        "status" => ops::cmd_network_status(tail),
        "peers" => ops::cmd_peer_list(tail),
        "full" => ops::cmd_network_full(tail),
        _ => invalid_group_usage("query network", "unsupported subcommand"),
    }
}

fn route_tx_group(args: &[String]) -> Result<(), AppError> {
    let Some((subcommand, tail)) = args.split_first() else {
        return invalid_group_usage("tx", "missing subcommand");
    };

    match subcommand.as_str() {
        "transfer" => {
            let mapped = remap_flags(tail, &[("--to", "--to"), ("--amount", "--amount")]);
            ops::cmd_treasury_transfer(&mapped)
        }
        "stake" => route_stake_group(tail),
        _ => invalid_group_usage("tx", "unsupported subcommand"),
    }
}

fn route_api_group(args: &[String]) -> Result<(), AppError> {
    let Some((subcommand, tail)) = args.split_first() else {
        return ops::cmd_rpc_status(args);
    };

    match subcommand.as_str() {
        "status" | "rpc" => ops::cmd_rpc_status(tail),
        "contract" | "api-contract" => ops::cmd_api_contract(tail),
        "smoke" | "curl-smoke" => ops::cmd_rpc_curl_smoke(tail),
        "metrics" => ops::cmd_metrics(tail),
        "health" => ops::cmd_runtime_status(tail),
        "full" => ops::cmd_query_full(tail),
        "chain" => route_query_chain_group(tail),
        "consensus" => route_query_consensus_group(tail),
        "vm" => route_query_vm_group(tail),
        "block" => ops::cmd_block_get(tail),
        "tx" => ops::cmd_tx_get(tail),
        "receipt" => ops::cmd_tx_receipt(tail),
        "account" => ops::cmd_account_get(tail),
        "balance" => ops::cmd_balance_get(tail),
        "state-root" => ops::cmd_state_root(tail),
        "network" => route_query_network_group(tail),
        "runtime" => ops::cmd_query_runtime(tail),
        _ => invalid_group_usage("api", "unsupported subcommand"),
    }
}

fn route_stake_group(args: &[String]) -> Result<(), AppError> {
    let Some((subcommand, tail)) = args.split_first() else {
        return invalid_group_usage("stake", "missing subcommand");
    };

    match subcommand.as_str() {
        "delegate" => {
            let mapped = remap_flags(tail, &[("--to", "--validator"), ("--amount", "--amount")]);
            ops::cmd_stake_delegate(&mapped)
        }
        "undelegate" => {
            let mapped = remap_flags(tail, &[("--to", "--validator"), ("--amount", "--amount")]);
            ops::cmd_stake_undelegate(&mapped)
        }
        "rewards" | "validators" => ops::cmd_economy_status(tail),
        _ => invalid_group_usage("stake", "unsupported subcommand"),
    }
}

fn route_doctor_group(args: &[String]) -> Result<(), AppError> {
    if args.is_empty() {
        return audit::cmd_diagnostics_doctor(args);
    }

    match args[0].as_str() {
        "network" | "node" | "runtime" => audit::cmd_diagnostics_doctor(&args[1..]),
        _ => invalid_group_usage("doctor", "unsupported subcommand"),
    }
}

fn route_audit_group(args: &[String]) -> Result<(), AppError> {
    let Some((subcommand, tail)) = args.split_first() else {
        return audit::cmd_production_audit(args);
    };

    match subcommand.as_str() {
        "chain" | "genesis" | "validator-set" => audit::cmd_production_audit(tail),
        _ => invalid_group_usage("audit", "unsupported subcommand"),
    }
}

fn remap_flags(args: &[String], mappings: &[(&str, &str)]) -> Vec<String> {
    args.iter()
        .map(|arg| {
            mappings
                .iter()
                .find_map(|(from, to)| (*from == arg).then(|| (*to).to_string()))
                .unwrap_or_else(|| arg.clone())
        })
        .collect()
}

fn route_node_join_alias(args: &[String]) -> Result<(), AppError> {
    let forwarded = std::iter::once("join".to_string())
        .chain(args.iter().cloned())
        .collect::<Vec<_>>();
    route_node_group(&forwarded)
}

fn invalid_group_usage(group: &str, detail: &str) -> Result<(), AppError> {
    Err(AppError::new(
        ErrorCode::UsageInvalidArguments,
        format!("Invalid {group} command: {detail}"),
    ))
}

#[cfg(test)]
mod tests {
    use super::{
        remap_flags, route_api_group, route_query_consensus_group, route_query_group,
        route_query_vm_group, run_cli,
    };

    #[test]
    fn cli_module_is_linkable() {
        let _ = run_cli as fn() -> Result<(), crate::error::AppError>;
    }

    #[test]
    fn remap_flags_rewrites_expected_aliases() {
        let input = vec![
            "--from".to_string(),
            "alice".to_string(),
            "--to".to_string(),
            "validator-01".to_string(),
            "--amount".to_string(),
            "100".to_string(),
        ];

        let output = remap_flags(&input, &[("--to", "--validator")]);
        assert_eq!(output[2], "--validator");
        assert_eq!(output[3], "validator-01");
    }

    #[test]
    fn query_consensus_group_supports_extended_subcommands() {
        let command = vec!["validators".to_string()];
        assert!(route_query_consensus_group(&command).is_ok());
    }

    #[test]
    fn query_vm_group_supports_extended_subcommands() {
        let command = vec!["trace".to_string()];
        assert!(route_query_vm_group(&command).is_ok());
    }

    #[test]
    fn api_group_rejects_unknown_subcommands() {
        let command = vec!["unknown".to_string()];
        assert!(route_api_group(&command).is_err());
    }

    #[test]
    fn query_group_supports_full_subcommand() {
        let command = vec!["full".to_string()];
        assert!(route_query_group(&command).is_ok());
    }

    #[test]
    fn api_group_supports_full_subcommand() {
        let command = vec!["full".to_string()];
        assert!(route_api_group(&command).is_ok());
    }
}
