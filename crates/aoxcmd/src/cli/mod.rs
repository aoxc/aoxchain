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
        "port-map" => describe::cmd_port_map(),
        "profile-baseline" => ops::cmd_profile_baseline(&args[2..]),
        "testnet-fixture-init" => bootstrap::cmd_testnet_fixture_init(&args[2..]),
        "load-benchmark" => ops::cmd_load_benchmark(&args[2..]),
        "mainnet-readiness" => ops::cmd_mainnet_readiness(&args[2..]),
        "testnet-readiness" => ops::cmd_testnet_readiness(&args[2..]),
        "full-surface-readiness" => ops::cmd_full_surface_readiness(&args[2..]),
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
        "genesis-security-audit" => bootstrap::cmd_genesis_security_audit(&args[2..]),
        "consensus-profile-audit" => bootstrap::cmd_consensus_profile_audit(&args[2..]),
        "genesis-hash" => bootstrap::cmd_genesis_hash(&args[2..]),
        "config-init" => bootstrap::cmd_config_init(&args[2..]),
        "config-validate" => bootstrap::cmd_config_validate(&args[2..]),
        "config-print" => bootstrap::cmd_config_print(&args[2..]),
        "production-bootstrap" => bootstrap::cmd_production_bootstrap(&args[2..]),
        "dual-profile-bootstrap" => bootstrap::cmd_dual_profile_bootstrap(&args[2..]),
        "node-bootstrap" => ops::cmd_node_bootstrap(&args[2..]),
        "produce-once" => ops::cmd_produce_once(&args[2..]),
        "node-run" => ops::cmd_node_run(&args[2..]),
        "node-health" => ops::cmd_node_health(&args[2..]),
        "network-smoke" => ops::cmd_network_smoke(&args[2..]),
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
        "economy-status" => ops::cmd_economy_status(&args[2..]),
        "runtime-status" => ops::cmd_runtime_status(&args[2..]),
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
        "inspect" => bootstrap::cmd_genesis_inspect(tail),
        "template-advanced" => bootstrap::cmd_genesis_template_advanced(tail),
        "security-audit" => bootstrap::cmd_genesis_security_audit(tail),
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
        "start" => ops::cmd_real_network(tail),
        "status" | "verify" => ops::cmd_network_smoke(tail),
        "doctor" => audit::cmd_diagnostics_doctor(tail),
        _ => invalid_group_usage("network", "unsupported subcommand"),
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

fn invalid_group_usage(group: &str, detail: &str) -> Result<(), AppError> {
    Err(AppError::new(
        ErrorCode::UsageInvalidArguments,
        format!("Invalid {group} command: {detail}"),
    ))
}

#[cfg(test)]
mod tests {
    use super::{remap_flags, run_cli};

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
}
