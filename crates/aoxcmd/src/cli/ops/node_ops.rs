use super::*;

pub fn cmd_node_bootstrap(args: &[String]) -> Result<(), AppError> {
    bootstrap_operator_home()?;
    let state = lifecycle::bootstrap_state()?;
    let _ = refresh_runtime_metrics().ok();
    emit_serialized(&state, output_format(args))
}

pub fn cmd_produce_once(args: &[String]) -> Result<(), AppError> {
    let tx = parse_required_or_default_text_arg(args, "--tx", "boot-sequence-1", false)?;
    let state = engine::produce_once(&tx)?;
    let _ = refresh_runtime_metrics().ok();
    emit_serialized(&state, output_format(args))
}

pub fn cmd_node_run(args: &[String]) -> Result<(), AppError> {
    let rounds = parse_positive_u64_arg(args, "--rounds", 10, "node run")?;
    let tx_prefix = parse_required_or_default_text_arg(args, "--tx-prefix", "AOXC-RUN", false)?;
    let format = output_format(args);
    let live_log_enabled = !has_flag(args, "--no-live-log");
    let log_level = parse_required_or_default_text_arg(args, "--log-level", "info", true)?;

    if !matches!(log_level.as_str(), "info" | "debug") {
        return Err(AppError::new(
            ErrorCode::UsageInvalidArguments,
            "Invalid --log-level value (supported: info, debug)",
        ));
    }

    if format == crate::cli_support::OutputFormat::Text && live_log_enabled {
        print_node_live_log_header(rounds, &tx_prefix, &log_level)?;
    }

    let state = if format == crate::cli_support::OutputFormat::Text && live_log_enabled {
        engine::run_rounds_with_observer(rounds, &tx_prefix, |entry| {
            print_node_round_line(entry, &log_level);
        })?
    } else {
        engine::run_rounds(rounds, &tx_prefix)?
    };

    let _ = refresh_runtime_metrics().ok();
    let _ = graceful_shutdown();

    if format == crate::cli_support::OutputFormat::Text && live_log_enabled {
        print_node_live_log_footer(&state);
    }

    emit_serialized(&state, format)
}

fn print_node_live_log_header(
    rounds: u64,
    tx_prefix: &str,
    log_level: &str,
) -> Result<(), AppError> {
    let now = chrono::Utc::now().to_rfc3339();
    let db_path = lifecycle::state_path()?;

    println!("🚀 [{}] node-run startup", now);
    println!(
        "🧭 mode=live rounds={} tx_prefix={} log_level={}",
        rounds, tx_prefix, log_level
    );
    println!("🗄️  state_db={}", db_path.display());
    println!(
        "📋 {:>5} | {:<25} | {:>8} | {:>8} | {:>8} | {:<12}",
        "round", "timestamp", "height", "blocks", "sections", "tx"
    );
    println!(
        "────────────────────────────────────────────────────────────────────────────────────────"
    );
    Ok(())
}

fn print_node_round_line(entry: &engine::RoundTelemetry, log_level: &str) {
    let timestamp = chrono::DateTime::<chrono::Utc>::from_timestamp(entry.timestamp_unix as i64, 0)
        .map(|dt| dt.to_rfc3339())
        .unwrap_or_else(|| chrono::Utc::now().to_rfc3339());

    println!(
        "✅ {:>5} | {:<25} | {:>8} | {:>8} | {:>8} | {:<12}",
        entry.round_index,
        timestamp,
        entry.height,
        entry.produced_blocks,
        entry.section_count,
        entry.tx_id
    );

    if log_level == "debug" {
        println!(
            "   🔍 round={} consensus_round={} block={} parent={}",
            entry.round_index,
            entry.consensus_round,
            short_hash(&entry.block_hash_hex),
            short_hash(&entry.parent_hash_hex)
        );
    }
}

fn print_node_live_log_footer(state: &crate::node::state::NodeState) {
    println!(
        "────────────────────────────────────────────────────────────────────────────────────────"
    );
    println!(
        "🏁 completed height={} produced_blocks={} updated_at={}",
        state.current_height, state.produced_blocks, state.updated_at
    );
}

fn short_hash(value: &str) -> String {
    if value.len() <= 16 {
        return value.to_string();
    }
    format!("{}…{}", &value[..8], &value[value.len() - 8..])
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
    let settings = effective_settings_for_ops()?;
    let key_summary = crate::keys::manager::inspect_operator_key()?;

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
    details.insert(
        "transport_public_key".to_string(),
        key_summary.transport_public_key,
    );
    details.insert(
        "key_operational_state".to_string(),
        key_summary.operational_state,
    );

    emit_serialized(
        &text_envelope("network-smoke", "ok", details),
        output_format(args),
    )
}

pub fn cmd_real_network(args: &[String]) -> Result<(), AppError> {
    let settings = effective_settings_for_ops()?;
    let key_summary = crate::keys::manager::inspect_operator_key()?;

    let mut details = BTreeMap::new();
    details.insert("mode".to_string(), "deterministic-local".to_string());
    details.insert(
        "enforce_official_peers".to_string(),
        settings.network.enforce_official_peers.to_string(),
    );
    details.insert(
        "key_operational_state".to_string(),
        key_summary.operational_state,
    );
    details.insert(
        "transport_public_key".to_string(),
        key_summary.transport_public_key,
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
