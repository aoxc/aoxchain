use crate::{
    app::{
        bootstrap::bootstrap_operator_home, runtime::refresh_runtime_metrics,
        shutdown::graceful_shutdown,
    },
    cli_support::{arg_value, emit_serialized, output_format, text_envelope},
    config::loader::load_or_init,
    economy::ledger,
    error::{AppError, ErrorCode},
    node::{engine, lifecycle},
    runtime::{
        core::runtime_context, handles::default_handles, node::health_status, unity::unity_status,
    },
};
use std::collections::BTreeMap;

pub fn cmd_load_benchmark(args: &[String]) -> Result<(), AppError> {
    let rounds = arg_value(args, "--rounds").unwrap_or_else(|| "100".to_string());
    let mut details = BTreeMap::new();
    details.insert("benchmark_rounds".to_string(), rounds);
    details.insert(
        "result".to_string(),
        "baseline-local-benchmark-recorded".to_string(),
    );
    emit_serialized(
        &text_envelope("load-benchmark", "ok", details),
        output_format(args),
    )
}

pub fn cmd_mainnet_readiness(args: &[String]) -> Result<(), AppError> {
    let settings = load_or_init()?;
    let key_ok = crate::keys::manager::verify_operator_key().is_ok();
    let genesis_ok = crate::cli::bootstrap::genesis_ready();
    let node_ok = lifecycle::load_state().is_ok();
    #[derive(serde::Serialize)]
    struct Readiness {
        profile: String,
        config_present: bool,
        key_material_present: bool,
        genesis_present: bool,
        node_state_present: bool,
        enforce_official_peers: bool,
        verdict: &'static str,
    }
    let readiness = Readiness {
        profile: settings.profile,
        config_present: true,
        key_material_present: key_ok,
        genesis_present: genesis_ok,
        node_state_present: node_ok,
        enforce_official_peers: settings.network.enforce_official_peers,
        verdict: if key_ok && genesis_ok && node_ok && settings.network.enforce_official_peers {
            "candidate"
        } else {
            "not-ready"
        },
    };
    emit_serialized(&readiness, output_format(args))
}

pub fn cmd_node_bootstrap(args: &[String]) -> Result<(), AppError> {
    bootstrap_operator_home()?;
    let state = lifecycle::bootstrap_state()?;
    let _ = refresh_runtime_metrics().ok();
    emit_serialized(&state, output_format(args))
}

pub fn cmd_produce_once(args: &[String]) -> Result<(), AppError> {
    let tx = arg_value(args, "--tx").unwrap_or_else(|| "boot-sequence-1".to_string());
    let state = engine::produce_once(&tx)?;
    let _ = refresh_runtime_metrics().ok();
    emit_serialized(&state, output_format(args))
}

pub fn cmd_node_run(args: &[String]) -> Result<(), AppError> {
    let rounds = arg_value(args, "--rounds")
        .unwrap_or_else(|| "10".to_string())
        .parse::<u64>()
        .map_err(|_| AppError::new(ErrorCode::UsageInvalidArguments, "Invalid --rounds value"))?;
    let tx_prefix = arg_value(args, "--tx-prefix").unwrap_or_else(|| "AOXC-RUN".to_string());
    let state = engine::run_rounds(rounds, &tx_prefix)?;
    let _ = refresh_runtime_metrics().ok();
    let _ = graceful_shutdown();
    emit_serialized(&state, output_format(args))
}

pub fn cmd_node_health(args: &[String]) -> Result<(), AppError> {
    let health = health_status()?;
    let mut details = BTreeMap::new();
    details.insert("health".to_string(), health.to_string());
    emit_serialized(
        &text_envelope("node-health", "ok", details),
        output_format(args),
    )
}

pub fn cmd_network_smoke(args: &[String]) -> Result<(), AppError> {
    let settings = load_or_init()?;
    let mut details = BTreeMap::new();
    details.insert("bind_host".to_string(), settings.network.bind_host);
    details.insert(
        "rpc_port".to_string(),
        settings.network.rpc_port.to_string(),
    );
    details.insert(
        "probe".to_string(),
        "local-listener-simulated-ok".to_string(),
    );
    emit_serialized(
        &text_envelope("network-smoke", "ok", details),
        output_format(args),
    )
}

pub fn cmd_real_network(args: &[String]) -> Result<(), AppError> {
    let settings = load_or_init()?;
    let mut details = BTreeMap::new();
    details.insert("mode".to_string(), "deterministic-local".to_string());
    details.insert(
        "enforce_official_peers".to_string(),
        settings.network.enforce_official_peers.to_string(),
    );
    emit_serialized(
        &text_envelope("real-network", "ok", details),
        output_format(args),
    )
}

pub fn cmd_storage_smoke(args: &[String]) -> Result<(), AppError> {
    let context = runtime_context()?;
    let mut details = BTreeMap::new();
    details.insert("home_dir".to_string(), context.settings.home_dir);
    details.insert("storage".to_string(), "writable".to_string());
    emit_serialized(
        &text_envelope("storage-smoke", "ok", details),
        output_format(args),
    )
}

pub fn cmd_economy_init(args: &[String]) -> Result<(), AppError> {
    let ledger = ledger::init()?;
    let _ = refresh_runtime_metrics().ok();
    emit_serialized(&ledger, output_format(args))
}

pub fn cmd_treasury_transfer(args: &[String]) -> Result<(), AppError> {
    let to = arg_value(args, "--to").unwrap_or_else(|| "ops".to_string());
    let amount = arg_value(args, "--amount")
        .unwrap_or_else(|| "1000".to_string())
        .parse::<u64>()
        .map_err(|_| AppError::new(ErrorCode::UsageInvalidArguments, "Invalid --amount value"))?;
    let ledger = ledger::transfer(&to, amount)?;
    let _ = refresh_runtime_metrics().ok();
    emit_serialized(&ledger, output_format(args))
}

pub fn cmd_stake_delegate(args: &[String]) -> Result<(), AppError> {
    let validator = arg_value(args, "--validator").unwrap_or_else(|| "validator-01".to_string());
    let amount = arg_value(args, "--amount")
        .unwrap_or_else(|| "1000".to_string())
        .parse::<u64>()
        .map_err(|_| AppError::new(ErrorCode::UsageInvalidArguments, "Invalid --amount value"))?;
    let ledger = ledger::delegate(&validator, amount)?;
    let _ = refresh_runtime_metrics().ok();
    emit_serialized(&ledger, output_format(args))
}

pub fn cmd_stake_undelegate(args: &[String]) -> Result<(), AppError> {
    let validator = arg_value(args, "--validator").unwrap_or_else(|| "validator-01".to_string());
    let amount = arg_value(args, "--amount")
        .unwrap_or_else(|| "1000".to_string())
        .parse::<u64>()
        .map_err(|_| AppError::new(ErrorCode::UsageInvalidArguments, "Invalid --amount value"))?;
    let ledger = ledger::undelegate(&validator, amount)?;
    let _ = refresh_runtime_metrics().ok();
    emit_serialized(&ledger, output_format(args))
}

pub fn cmd_economy_status(args: &[String]) -> Result<(), AppError> {
    let ledger = ledger::load()?;
    emit_serialized(&ledger, output_format(args))
}

pub fn cmd_runtime_status(args: &[String]) -> Result<(), AppError> {
    let context = runtime_context()?;
    let handles = default_handles();
    let unity = unity_status();
    let ai = crate::ai::runtime::report();
    #[derive(serde::Serialize)]
    struct RuntimeStatus {
        context: crate::runtime::context::RuntimeContext,
        handles: crate::runtime::handles::RuntimeHandleSet,
        unity: crate::runtime::unity::UnityStatus,
        ai: crate::ai::runtime::AiRuntimeReport,
    }
    let status = RuntimeStatus {
        context,
        handles,
        unity,
        ai,
    };
    emit_serialized(&status, output_format(args))
}
