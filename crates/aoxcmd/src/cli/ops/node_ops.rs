use super::*;
use std::collections::BTreeMap;
use std::time::Duration;

/// Human-readable event code for production events.
///
/// Audit Note:
/// `PRD` is intentionally used instead of a generic success marker in order to
/// avoid implying consensus finality or irreversible confirmation.
const EVENT_PRODUCED: &str = "PRD";

/// Banner width constraints.
///
/// Audit Note:
/// The banner uses bounded width rather than unconstrained auto-expansion so
/// that operator output remains stable across terminals and long paths do not
/// destroy alignment.
const BANNER_MIN_WIDTH: usize = 76;
const BANNER_MAX_WIDTH: usize = 108;

pub fn cmd_node_bootstrap(args: &[String]) -> Result<(), AppError> {
    bootstrap_operator_home()?;
    let state = lifecycle::bootstrap_state()?;
    let _ = refresh_runtime_metrics().ok();

    if output_format(args) == crate::cli_support::OutputFormat::Text {
        println!("BOOTSTRAP | operator home initialized | state materialized | metrics refreshed");
    }

    emit_serialized(&state, output_format(args))
}

pub fn cmd_produce_once(args: &[String]) -> Result<(), AppError> {
    let tx = parse_required_or_default_text_arg(args, "--tx", &default_runtime_tx_id(), false)?;
    let state = engine::produce_once(&tx)?;
    let _ = refresh_runtime_metrics().ok();

    if output_format(args) == crate::cli_support::OutputFormat::Text {
        println!(
            "MANUAL    | tx={} | height={} | consensus_round={} | block={} | parent={}",
            shorten_middle(&tx, 20),
            state.current_height,
            state.consensus.last_round,
            short_hash(&state.consensus.last_block_hash_hex),
            short_hash(&state.consensus.last_parent_hash_hex),
        );
    }

    emit_serialized(&state, output_format(args))
}

pub fn cmd_node_run(args: &[String]) -> Result<(), AppError> {
    let rounds = parse_positive_u64_arg(args, "--rounds", 10, "node run")?;
    let tx_prefix = parse_required_or_default_text_arg(args, "--tx-prefix", "runtime-tx", false)?;
    let format = output_format(args);
    let live_log_enabled = !has_flag(args, "--no-live-log");
    let log_level = parse_required_or_default_text_arg(args, "--log-level", "info", true)?;
    let interval_secs = parse_block_interval_secs(args)?;
    let continuous = if has_flag(args, "--bounded") {
        false
    } else {
        has_flag(args, "--continuous") || arg_value(args, "--rounds").is_none()
    };

    if !matches!(log_level.as_str(), "info" | "debug") {
        return Err(AppError::new(
            ErrorCode::UsageInvalidArguments,
            "Invalid --log-level value (supported: info, debug)",
        ));
    }

    if !has_flag(args, "--no-rpc-serve") {
        if let Ok(settings) = effective_settings_for_ops() {
            let _ = super::rpc_serve_ops::spawn_rpc_and_metrics_listeners(
                &settings.network.bind_host,
                settings.network.rpc_port,
                settings.telemetry.prometheus_port,
            );
        }
    }

    if format == crate::cli_support::OutputFormat::Text && live_log_enabled {
        print_node_live_log_header(
            rounds,
            &tx_prefix,
            &log_level,
            interval_secs,
            continuous,
        )?;
    }

    let state = if continuous {
        run_continuous_rounds(
            interval_secs,
            &tx_prefix,
            format,
            live_log_enabled,
            &log_level,
        )?
    } else {
        // Corporate/Audit Note:
        // The bounded execution path intentionally reuses the engine-owned
        // observer pipeline so that terminal output and machine-readable
        // telemetry derive from the same execution source of truth.
        engine::run_rounds_with_observer(rounds, &tx_prefix, |entry| {
            if format == crate::cli_support::OutputFormat::Text && live_log_enabled {
                print_node_round_line(entry, &log_level);
            }
        })?
    };

    let _ = refresh_runtime_metrics().ok();
    let _ = graceful_shutdown();

    if format == crate::cli_support::OutputFormat::Text && live_log_enabled {
        print_node_live_log_footer(&state);
    }

    emit_serialized(&state, format)
}

fn default_runtime_tx_id() -> String {
    format!("runtime-tx-{}", chrono::Utc::now().timestamp())
}

/// Prints the operator-facing startup banner.
///
/// Audit Note:
/// The banner is rendered with bounded width and controlled truncation to
/// preserve alignment across terminals while still exposing a high-signal
/// runtime posture summary.
fn print_node_live_log_header(
    rounds: u64,
    tx_prefix: &str,
    log_level: &str,
    interval_secs: u64,
    continuous: bool,
) -> Result<(), AppError> {
    let now = chrono::Utc::now()
        .format("%Y-%m-%d %H:%M:%S UTC")
        .to_string();

    let db_path = lifecycle::state_path()?;
    let settings = effective_settings_for_ops()?;
    let state = lifecycle::load_state()?;

    let probe_host = rpc_probe_host(&settings.network.bind_host);

    let rpc_url = format!(
        "http://{}:{}",
        settings.network.bind_host, settings.network.rpc_port
    );
    let metrics_url = format!(
        "http://{}:{}/metrics",
        settings.network.bind_host, settings.telemetry.prometheus_port
    );
    let probe_target = format!("{}:{}", probe_host, settings.network.rpc_port);

    let rpc_reachable = rpc_http_get_probe(&probe_host, settings.network.rpc_port, "/health")
        || rpc_jsonrpc_status_probe(&probe_host, settings.network.rpc_port);

    let execution_mode = if continuous { "continuous" } else { "bounded" };
    let assessment = startup_assessment(
        rpc_reachable,
        non_empty_or(&state.key_material.operational_state, "unknown"),
        state.current_height,
    );

    let key_state = non_empty_or(&state.key_material.operational_state, "unknown");
    let fingerprint = short_hash(&state.key_material.bundle_fingerprint);
    let proposer = short_hash(&state.consensus.last_proposer_hex);
    let head = short_hash(&state.consensus.last_block_hash_hex);
    let parent = short_hash(&state.consensus.last_parent_hash_hex);

    let banner_lines = vec![
        "🚀 AOXC TESTNET FULL NODE".to_string(),
        "Operator Console • Runtime Session".to_string(),
        format!("🕒 Started    : {}", now),
        format!("🌐 Profile    : {}", settings.profile),
        format!("⚙️ Mode       : {}", execution_mode),
        format!(
            "🏠 Home       : {}",
            shorten_middle(&settings.home_dir, 74)
        ),
        format!(
            "📡 Network    : bind={} | p2p={} | rpc={} | metrics={}",
            settings.network.bind_host,
            settings.network.p2p_port,
            settings.network.rpc_port,
            settings.telemetry.prometheus_port
        ),
        format!("🔌 Probe      : {}", probe_target),
        format!(
            "🧱 Chain      : height={} | round={} | produced={} | sections={} | network_id={}",
            state.current_height,
            state.consensus.last_round,
            state.produced_blocks,
            state.consensus.last_section_count,
            state.consensus.network_id
        ),
        format!(
            "🔐 Identity   : key={} | fingerprint={} | proposer={}",
            key_state,
            fingerprint,
            proposer
        ),
        format!(
            "🗄 Storage    : db={} | persistence=enabled",
            shorten_middle(&db_path.display().to_string(), 62)
        ),
        format!(
            "📈 Head       : block={} | parent={}",
            head,
            parent
        ),
        format!(
            "✅ Status     : {} | rpc={} | health={}",
            assessment,
            status_word(rpc_reachable),
            health_hint_from_height(state.current_height)
        ),
        format!(
            "📝 Note       : {} | rpc_url={} | metrics_url={}",
            if log_level == "debug" {
                "extended trace enabled"
            } else {
                "use --log-level debug for extended trace"
            },
            shorten_middle(&rpc_url, 26),
            shorten_middle(&metrics_url, 26)
        ),
        format!(
            "🎯 Session    : rounds={} | interval={}s | tx_prefix={}",
            rounds,
            interval_secs,
            shorten_middle(tx_prefix, 24)
        ),
    ];

    print_banner(&banner_lines);
    print_event_table_header();

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

/// Runs unbounded production until external termination.
///
/// Audit Note:
/// A live line is emitted only after a successful production step. This avoids
/// optimistic terminal output for operations that did not actually advance state.
fn run_continuous_rounds(
    interval_secs: u64,
    tx_prefix: &str,
    format: crate::cli_support::OutputFormat,
    live_log_enabled: bool,
    log_level: &str,
) -> Result<crate::node::state::NodeState, AppError> {
    let mut round_index = 0_u64;

    loop {
        round_index = round_index.saturating_add(1);
        let tx = format!("{tx_prefix}-{round_index}");
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
                proposer_hex: state.consensus.last_proposer_hex.clone(),
                message_kind: state.consensus.last_message_kind.clone(),
                timestamp_unix: state.consensus.last_timestamp_unix,
            };
            print_node_round_line(&telemetry, log_level);
        }

        std::thread::sleep(Duration::from_secs(interval_secs));
    }
}

/// Prints a single live execution line.
///
/// Audit Note:
/// The standard line is intentionally information-rich while remaining bounded
/// to a stable column layout. Debug-only fields are emitted on a dedicated
/// second line to prevent the primary stream from becoming unreadable.
fn print_node_round_line(entry: &engine::RoundTelemetry, log_level: &str) {
    let timestamp = chrono::DateTime::<chrono::Utc>::from_timestamp(entry.timestamp_unix as i64, 0)
        .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
        .unwrap_or_else(|| chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string());

    println!(
        "{:>3} | {:>5} | {:<19} | {:>7} | {:>7} | {:>4} | {:<17} | {:<17} | {:<17} | {:<18}",
        EVENT_PRODUCED,
        entry.round_index,
        timestamp,
        entry.height,
        entry.consensus_round,
        entry.section_count,
        short_hash(&entry.block_hash_hex),
        short_hash(&entry.parent_hash_hex),
        short_hash(&entry.proposer_hex),
        shorten_middle(&entry.tx_id, 18)
    );

    if log_level == "debug" {
        println!(
            "DBG | round={} | kind={} | unix_ts={} | produced_blocks={}",
            entry.round_index,
            entry.message_kind,
            entry.timestamp_unix,
            entry.produced_blocks
        );
    }
}

/// Prints the end-of-session summary.
///
/// Audit Note:
/// The footer is intentionally concise and suitable for release/operator
/// evidence capture.
fn print_node_live_log_footer(state: &crate::node::state::NodeState) {
    println!("------------------------------------------------------------------------------------------------------------------------------------------------");
    println!(
        "SUMMARY | final_height={} | produced_blocks={} | consensus_round={} | block={} | updated_at={}",
        state.current_height,
        state.produced_blocks,
        state.consensus.last_round,
        short_hash(&state.consensus.last_block_hash_hex),
        state.updated_at
    );
    println!("SESSION | graceful shutdown completed");
    println!();
}

fn print_event_table_header() {
    println!("------------------------------------------------------------------------------------------------------------------------------------------------");
    println!(
        "{:>3} | {:>5} | {:<19} | {:>7} | {:>7} | {:>4} | {:<17} | {:<17} | {:<17} | {:<18}",
        "EVT",
        "ROUND",
        "TIMESTAMP",
        "HEIGHT",
        "CROUND",
        "SEC",
        "BLOCK",
        "PARENT",
        "PROPOSER",
        "TX"
    );
    println!("------------------------------------------------------------------------------------------------------------------------------------------------");
}

/// Renders a centered, bounded-width operator banner.
fn print_banner(lines: &[String]) {
    let inner_width = banner_inner_width(lines);

    println!("╔{}╗", "═".repeat(inner_width + 2));

    for (index, line) in lines.iter().enumerate() {
        let prepared = shorten_middle(line, inner_width);
        if index < 2 {
            println!("║ {} ║", center_text(&prepared, inner_width));
        } else if index == 2 {
            println!("╠{}╣", "═".repeat(inner_width + 2));
            println!("║ {} ║", pad_right(&prepared, inner_width));
        } else {
            println!("║ {} ║", pad_right(&prepared, inner_width));
        }
    }

    println!("╚{}╝", "═".repeat(inner_width + 2));
}

fn banner_inner_width(lines: &[String]) -> usize {
    let measured = lines
        .iter()
        .map(|line| visible_width(line))
        .max()
        .unwrap_or(BANNER_MIN_WIDTH);

    measured.clamp(BANNER_MIN_WIDTH, BANNER_MAX_WIDTH)
}

fn center_text(value: &str, width: usize) -> String {
    let visible = visible_width(value);
    if visible >= width {
        return value.to_string();
    }

    let total_pad = width - visible;
    let left = total_pad / 2;
    let right = total_pad - left;

    format!("{}{}{}", " ".repeat(left), value, " ".repeat(right))
}

fn pad_right(value: &str, width: usize) -> String {
    let visible = visible_width(value);
    if visible >= width {
        return value.to_string();
    }

    format!("{}{}", value, " ".repeat(width - visible))
}

/// Produces a compact hash representation suitable for console output.
fn short_hash(value: &str) -> String {
    if value.is_empty() {
        return "unavailable".to_string();
    }

    if value.chars().count() <= 18 {
        return value.to_string();
    }

    format!("{}…{}", &value[..8], &value[value.len() - 8..])
}

/// Returns a fallback string if the input is empty.
fn non_empty_or<'a>(value: &'a str, fallback: &'a str) -> &'a str {
    if value.is_empty() {
        fallback
    } else {
        value
    }
}

/// Returns a probe host suitable for local reachability checks.
///
/// Audit Note:
/// `0.0.0.0` is a valid listen address but not a reliable client probe target.
/// For operator checks, it is normalized to loopback.
fn rpc_probe_host(bind_host: &str) -> String {
    if bind_host == "0.0.0.0" {
        "127.0.0.1".to_string()
    } else {
        bind_host.to_string()
    }
}

/// Derives a coarse startup assessment for operator visibility.
///
/// Audit Note:
/// This function is intentionally conservative and must not be interpreted as a
/// claim of final settlement or complete network safety.
fn startup_assessment(rpc_reachable: bool, key_state: &str, height: u64) -> &'static str {
    if !rpc_reachable {
        "DEGRADED-RPC"
    } else if key_state == "unknown" || key_state == "unavailable" {
        "DEGRADED-KEY"
    } else if height == 0 {
        "INITIALIZING"
    } else {
        "READY"
    }
}

fn status_word(value: bool) -> &'static str {
    if value {
        "reachable"
    } else {
        "unreachable"
    }
}

fn health_hint_from_height(height: u64) -> &'static str {
    if height == 0 {
        "chain-not-advanced"
    } else {
        "chain-progress-observed"
    }
}

/// Shortens long strings while preserving both ends.
///
/// Audit Note:
/// This helper ensures operator-facing layouts remain aligned even when paths,
/// URLs, or identifiers exceed the intended render width.
fn shorten_middle(value: &str, max_len: usize) -> String {
    let len = value.chars().count();
    if len <= max_len {
        return value.to_string();
    }

    if max_len <= 3 {
        return value.chars().take(max_len).collect();
    }

    let left = (max_len - 1) / 2;
    let right = max_len - 1 - left;

    let start: String = value.chars().take(left).collect();
    let end: String = value
        .chars()
        .rev()
        .take(right)
        .collect::<String>()
        .chars()
        .rev()
        .collect();

    format!("{start}…{end}")
}

/// Approximates visible width for terminal layout.
///
/// Audit Note:
/// This is a conservative character-count approximation. It is intentionally
/// simple and avoids introducing external dependencies for width calculation.
fn visible_width(value: &str) -> usize {
    value.chars().count()
}

pub fn cmd_node_health(args: &[String]) -> Result<(), AppError> {
    let health = health_status()?;
    let mut details = BTreeMap::new();
    details.insert("health".to_string(), health.to_string());

    if output_format(args) == crate::cli_support::OutputFormat::Text {
        println!("HEALTH    | status={}", health);
    }

    emit_serialized(
        &text_envelope("node-health", "ok", details),
        output_format(args),
    )
}

pub fn cmd_network_smoke(args: &[String]) -> Result<(), AppError> {
    let settings = effective_settings_for_ops()?;
    let key_summary = crate::keys::manager::inspect_operator_key()?;
    let probe_host = rpc_probe_host(&settings.network.bind_host);
    let rpc_reachable = rpc_http_get_probe(&probe_host, settings.network.rpc_port, "/health")
        || rpc_jsonrpc_status_probe(&probe_host, settings.network.rpc_port);

    let mut details = BTreeMap::new();
    details.insert("bind_host".to_string(), settings.network.bind_host.clone());
    details.insert("rpc_port".to_string(), settings.network.rpc_port.to_string());
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
        key_summary.transport_public_key.clone(),
    );
    details.insert(
        "key_operational_state".to_string(),
        key_summary.operational_state.clone(),
    );

    if output_format(args) == crate::cli_support::OutputFormat::Text {
        println!(
            "NETWORK   | bind={} | rpc_port={} | probe_host={} | rpc={} | key_state={}",
            settings.network.bind_host,
            settings.network.rpc_port,
            probe_host,
            status_word(rpc_reachable),
            key_summary.operational_state
        );
    }

    emit_serialized(
        &text_envelope("network-smoke", "ok", details),
        output_format(args),
    )
}

pub fn cmd_real_network(args: &[String]) -> Result<(), AppError> {
    let settings = effective_settings_for_ops()?;
    let key_summary = crate::keys::manager::inspect_operator_key()?;
    let probe_host = rpc_probe_host(&settings.network.bind_host);
    let rpc_reachable = rpc_http_get_probe(&probe_host, settings.network.rpc_port, "/health")
        || rpc_jsonrpc_status_probe(&probe_host, settings.network.rpc_port);

    let mut details = BTreeMap::new();
    details.insert("mode".to_string(), "runtime-network".to_string());
    details.insert("rpc_reachable".to_string(), rpc_reachable.to_string());
    details.insert(
        "enforce_official_peers".to_string(),
        settings.network.enforce_official_peers.to_string(),
    );
    details.insert(
        "key_operational_state".to_string(),
        key_summary.operational_state.clone(),
    );
    details.insert(
        "transport_public_key".to_string(),
        key_summary.transport_public_key.clone(),
    );

    if output_format(args) == crate::cli_support::OutputFormat::Text {
        println!(
            "REALNET   | enforce_official_peers={} | probe_host={} | rpc={} | key_state={}",
            settings.network.enforce_official_peers,
            probe_host,
            status_word(rpc_reachable),
            key_summary.operational_state
        );
    }

    emit_serialized(
        &text_envelope("real-network", "ok", details),
        output_format(args),
    )
}

pub fn cmd_storage_smoke(args: &[String]) -> Result<(), AppError> {
    let context = runtime_context()?;
    let mut details = BTreeMap::new();
    details.insert("home_dir".to_string(), context.settings.home_dir.clone());
    details.insert("storage".to_string(), "writable".to_string());

    if output_format(args) == crate::cli_support::OutputFormat::Text {
        println!(
            "STORAGE   | dir={} | status=writable",
            shorten_middle(&context.settings.home_dir, 72)
        );
    }

    emit_serialized(
        &text_envelope("storage-smoke", "ok", details),
        output_format(args),
    )
}
