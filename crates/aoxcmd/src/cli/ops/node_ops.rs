use super::*;

pub fn cmd_node_bootstrap(args: &[String]) -> Result<(), AppError> {
    bootstrap_operator_home()?;
    let state = lifecycle::bootstrap_state()?;
    let _ = refresh_runtime_metrics().ok();
    emit_serialized(&state, output_format(args))
}

pub fn cmd_produce_once(args: &[String]) -> Result<(), AppError> {
    let tx = parse_optional_text_arg(args, "--tx", false).ok_or_else(|| {
        AppError::new(
            ErrorCode::UsageInvalidArguments,
            "produce-once requires --tx with a real transaction payload",
        )
    })?;
    let state = engine::produce_once(&tx)?;
    let _ = refresh_runtime_metrics().ok();
    emit_serialized(&state, output_format(args))
}

pub fn cmd_node_run(args: &[String]) -> Result<(), AppError> {
    let rounds = parse_positive_u64_arg(args, "--rounds", 10, "node run")?;
    let format = output_format(args);
    let live_log_enabled = !has_flag(args, "--no-live-log");
    let log_level = parse_required_or_default_text_arg(args, "--log-level", "info", true)?;
    let interval_secs = parse_block_interval_secs(args)?;
    let continuous = has_flag(args, "--continuous");
    let mut tx_source = parse_round_tx_source(args)?;

    if !matches!(log_level.as_str(), "info" | "debug") {
        return Err(AppError::new(
            ErrorCode::UsageInvalidArguments,
            "Invalid --log-level value (supported: info, debug)",
        ));
    }

    if format == crate::cli_support::OutputFormat::Text && live_log_enabled {
        print_node_live_log_header(
            rounds,
            tx_source.describe(),
            &log_level,
            interval_secs,
            continuous,
        )?;
    }

    let state = if continuous {
        run_continuous_rounds(
            interval_secs,
            format,
            live_log_enabled,
            &log_level,
            &mut tx_source,
        )?
    } else {
        run_bounded_rounds(rounds, format, live_log_enabled, &log_level, &mut tx_source)?
    };

    let _ = refresh_runtime_metrics().ok();
    let _ = graceful_shutdown();

    if format == crate::cli_support::OutputFormat::Text && live_log_enabled {
        print_node_live_log_footer(&state);
    }

    emit_serialized(&state, format)
}

enum RoundTxSource {
    Static(String),
    File {
        entries: Vec<String>,
        next_index: usize,
    },
    SyntheticPrefix(String),
}

impl RoundTxSource {
    fn describe(&self) -> &'static str {
        match self {
            Self::Static(_) => "static-tx",
            Self::File { .. } => "tx-file",
            Self::SyntheticPrefix(_) => "synthetic-prefix",
        }
    }

    fn next_tx(&mut self, round_index: u64) -> String {
        match self {
            Self::Static(tx) => tx.clone(),
            Self::File {
                entries,
                next_index,
            } => {
                let idx = *next_index % entries.len();
                *next_index = next_index.saturating_add(1);
                entries[idx].clone()
            }
            Self::SyntheticPrefix(prefix) => format!("{prefix}-{round_index}"),
        }
    }
}

fn parse_round_tx_source(args: &[String]) -> Result<RoundTxSource, AppError> {
    let tx = parse_optional_text_arg(args, "--tx", false);
    let tx_file = parse_optional_text_arg(args, "--tx-file", false);
    let tx_prefix = parse_optional_text_arg(args, "--tx-prefix", false);
    let allow_synthetic = has_flag(args, "--allow-synthetic-tx");

    if tx.is_some() && tx_file.is_some() {
        return Err(AppError::new(
            ErrorCode::UsageInvalidArguments,
            "Use only one of --tx or --tx-file",
        ));
    }

    if let Some(value) = tx {
        return Ok(RoundTxSource::Static(value));
    }

    if let Some(path) = tx_file {
        let entries = load_tx_file_entries(&path)?;
        return Ok(RoundTxSource::File {
            entries,
            next_index: 0,
        });
    }

    if let Some(prefix) = tx_prefix {
        if !allow_synthetic {
            return Err(AppError::new(
                ErrorCode::UsageInvalidArguments,
                "Synthetic transaction generation requires --allow-synthetic-tx",
            ));
        }
        return Ok(RoundTxSource::SyntheticPrefix(prefix));
    }

    Err(AppError::new(
        ErrorCode::UsageInvalidArguments,
        "node-run requires --tx or --tx-file (or --tx-prefix with --allow-synthetic-tx)",
    ))
}

fn load_tx_file_entries(path: &str) -> Result<Vec<String>, AppError> {
    let raw = fs::read_to_string(path).map_err(|error| {
        AppError::with_source(
            ErrorCode::FilesystemIoFailed,
            format!("Failed to read tx file: {path}"),
            error,
        )
    })?;

    let entries = raw
        .lines()
        .filter_map(|line| normalize_text(line, false))
        .collect::<Vec<_>>();

    if entries.is_empty() {
        return Err(AppError::new(
            ErrorCode::UsageInvalidArguments,
            format!("tx file is empty: {path}"),
        ));
    }

    Ok(entries)
}

fn print_node_live_log_header(
    rounds: u64,
    tx_source: &str,
    log_level: &str,
    interval_secs: u64,
    continuous: bool,
) -> Result<(), AppError> {
    let now = chrono::Utc::now().to_rfc3339();
    let db_path = lifecycle::state_path()?;

    println!("🚀 [{}] node-run startup", now);
    println!(
        "🧭 mode=live rounds={} continuous={} interval_secs={} tx_source={} log_level={}",
        rounds, continuous, interval_secs, tx_source, log_level
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

fn parse_block_interval_secs(args: &[String]) -> Result<u64, AppError> {
    let interval_secs = match arg_value(args, "--interval-secs") {
        Some(value) => parse_positive_u64_value(&value, "--interval-secs", "node run")?,
        None => 6,
    };

    if !(2..=600).contains(&interval_secs) {
        return Err(AppError::new(
            ErrorCode::UsageInvalidArguments,
            "Flag --interval-secs must be between 2 and 600",
        ));
    }

    Ok(interval_secs)
}

fn run_continuous_rounds(
    interval_secs: u64,
    format: crate::cli_support::OutputFormat,
    live_log_enabled: bool,
    log_level: &str,
    tx_source: &mut RoundTxSource,
) -> Result<crate::node::state::NodeState, AppError> {
    let mut round_index = 0_u64;
    loop {
        round_index = round_index.saturating_add(1);
        let tx = tx_source.next_tx(round_index);
        let state = engine::produce_once(&tx)?;

        if format == crate::cli_support::OutputFormat::Text && live_log_enabled {
            let telemetry = engine::RoundTelemetry {
                round_index,
                tx_id: tx,
                height: state.current_height,
                produced_blocks: state.produced_blocks,
                consensus_round: state.consensus.last_round,
                section_count: state.consensus.last_section_count,
                block_hash_hex: state.consensus.last_block_hash_hex.clone(),
                parent_hash_hex: state.consensus.last_parent_hash_hex.clone(),
                timestamp_unix: state.consensus.last_timestamp_unix,
            };
            print_node_round_line(&telemetry, log_level);
        }

        std::thread::sleep(Duration::from_secs(interval_secs));
    }
}

fn run_bounded_rounds(
    rounds: u64,
    format: crate::cli_support::OutputFormat,
    live_log_enabled: bool,
    log_level: &str,
    tx_source: &mut RoundTxSource,
) -> Result<crate::node::state::NodeState, AppError> {
    let mut last_state = None;
    for round_index in 1..=rounds {
        let tx = tx_source.next_tx(round_index);
        let state = engine::produce_once(&tx)?;

        if format == crate::cli_support::OutputFormat::Text && live_log_enabled {
            let telemetry = engine::RoundTelemetry {
                round_index,
                tx_id: tx,
                height: state.current_height,
                produced_blocks: state.produced_blocks,
                consensus_round: state.consensus.last_round,
                section_count: state.consensus.last_section_count,
                block_hash_hex: state.consensus.last_block_hash_hex.clone(),
                parent_hash_hex: state.consensus.last_parent_hash_hex.clone(),
                timestamp_unix: state.consensus.last_timestamp_unix,
            };
            print_node_round_line(&telemetry, log_level);
        }

        last_state = Some(state);
    }

    last_state.ok_or_else(|| {
        AppError::new(
            ErrorCode::UsageInvalidArguments,
            "Round count must be greater than zero",
        )
    })
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
    let rpc_reachable =
        rpc_http_get_probe(
            &settings.network.bind_host,
            settings.network.rpc_port,
            "/health",
        ) || rpc_jsonrpc_status_probe(&settings.network.bind_host, settings.network.rpc_port);

    let mut details = BTreeMap::new();
    details.insert("bind_host".to_string(), settings.network.bind_host);
    details.insert(
        "rpc_port".to_string(),
        settings.network.rpc_port.to_string(),
    );
    details.insert(
        "probe".to_string(),
        if rpc_reachable {
            "rpc-reachable".to_string()
        } else {
            "rpc-unreachable".to_string()
        },
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
    let rpc_reachable =
        rpc_http_get_probe(
            &settings.network.bind_host,
            settings.network.rpc_port,
            "/health",
        ) || rpc_jsonrpc_status_probe(&settings.network.bind_host, settings.network.rpc_port);

    let mut details = BTreeMap::new();
    details.insert("mode".to_string(), "runtime-network".to_string());
    details.insert("rpc_reachable".to_string(), rpc_reachable.to_string());
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
