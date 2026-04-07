use super::*;
use std::collections::BTreeMap;
use std::time::Duration;

/// Terminal presentation constants.
///
/// Audit Note:
/// ANSI sequences are intentionally isolated in a compact constant block so that
/// terminal styling remains explicit, reviewable, and easy to disable or replace
/// in future operator-console revisions.
const ANSI_RESET: &str = "\x1b[0m";
const ANSI_BOLD: &str = "\x1b[1m";
const ANSI_DIM: &str = "\x1b[2m";
const ANSI_GREEN: &str = "\x1b[32m";
const ANSI_YELLOW: &str = "\x1b[33m";
const ANSI_BLUE: &str = "\x1b[34m";
const ANSI_MAGENTA: &str = "\x1b[35m";
const ANSI_CYAN: &str = "\x1b[36m";
const ANSI_RED: &str = "\x1b[31m";
const ANSI_BORDER: &str = "\x1b[38;5;239m";

/// Human-readable event codes used in the live execution stream.
///
/// Audit Note:
/// These codes are chosen to prevent semantic ambiguity. In particular, the code
/// avoids using a generic success checkmark that could be misread as block
/// finalization or irreversible confirmation.
const EVENT_PRODUCED: &str = "PRD";
const EVENT_ERROR: &str = "ERR";

pub fn cmd_node_bootstrap(args: &[String]) -> Result<(), AppError> {
    bootstrap_operator_home()?;
    let state = lifecycle::bootstrap_state()?;
    let _ = refresh_runtime_metrics().ok();

    if output_format(args) == crate::cli_support::OutputFormat::Text {
        println!(
            "{}BOOTSTRAP{} operator home initialized | state materialized | metrics refreshed",
            paint(ANSI_GREEN, "✔ "),
            ANSI_RESET
        );
    }

    emit_serialized(&state, output_format(args))
}

pub fn cmd_produce_once(args: &[String]) -> Result<(), AppError> {
    let tx = parse_required_or_default_text_arg(args, "--tx", &default_runtime_tx_id(), false)?;
    let state = engine::produce_once(&tx)?;
    let _ = refresh_runtime_metrics().ok();

    if output_format(args) == crate::cli_support::OutputFormat::Text {
        println!(
            "{}MANUAL{} block produced | tx={} | height={} | round={}",
            paint(ANSI_CYAN, "▶ "),
            ANSI_RESET,
            tx,
            state.current_height,
            state.consensus.last_round
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
        // The bounded execution path intentionally reuses the engine-owned observer
        // pipeline so that live terminal output and machine telemetry derive from
        // the same execution source of truth.
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
/// This banner is intentionally compact, deterministic, and text-oriented. Its
/// purpose is to present a high-signal operator snapshot without requiring the
/// reader to inspect multiple independent commands before understanding runtime
/// posture.
fn print_node_live_log_header(
    rounds: u64,
    tx_prefix: &str,
    log_level: &str,
    interval_secs: u64,
    continuous: bool,
) -> Result<(), AppError> {
    let now = chrono::Utc::now().to_rfc3339();
    let db_path = lifecycle::state_path()?;
    let settings = effective_settings_for_ops()?;
    let state = lifecycle::load_state()?;

    let rpc_url = format!(
        "http://{}:{}",
        settings.network.bind_host, settings.network.rpc_port
    );
    let metrics_url = format!(
        "http://{}:{}/metrics",
        settings.network.bind_host, settings.telemetry.prometheus_port
    );

    let rpc_reachable = rpc_http_get_probe(
        &settings.network.bind_host,
        settings.network.rpc_port,
        "/health",
    ) || rpc_jsonrpc_status_probe(&settings.network.bind_host, settings.network.rpc_port);

    let key_state = non_empty_or(&state.key_material.operational_state, "unknown");
    let key_fingerprint = non_empty_or(&state.key_material.bundle_fingerprint, "unavailable");
    let proposer = short_hash(&state.consensus.last_proposer_hex);
    let head = short_hash(&state.consensus.last_block_hash_hex);
    let parent = short_hash(&state.consensus.last_parent_hash_hex);

    let execution_mode = if continuous { "continuous" } else { "bounded" };
    let log_mode = if log_level == "debug" { "debug" } else { "info" };

    let assessment = startup_assessment(rpc_reachable, key_state, state.current_height);

    println!();
    print_border_top();
    print_box_line(
        "🚀 AOXC OPERATOR SESSION",
        &format!("[{}]", now),
    );
    print_separator();

    print_box_kv(
        "🧭 RUNTIME",
        &format!(
            "mode={} | rounds={} | interval={}s | tx_prefix={} | log={}",
            execution_mode, rounds, interval_secs, tx_prefix, log_mode
        ),
    );
    print_box_kv(
        "🛰 NETWORK",
        &format!(
            "profile={} | bind={} | p2p={} | rpc={} | metrics={}",
            settings.profile,
            settings.network.bind_host,
            settings.network.p2p_port,
            settings.network.rpc_port,
            settings.telemetry.prometheus_port
        ),
    );
    print_box_kv(
        "📡 ENDPOINTS",
        &format!("rpc={} | metrics={}", rpc_url, metrics_url),
    );
    print_box_kv(
        "🧱 CHAIN",
        &format!(
            "height={} | produced={} | network_id={} | consensus_round={} | sections={}",
            state.current_height,
            state.produced_blocks,
            state.consensus.network_id,
            state.consensus.last_round,
            state.consensus.last_section_count
        ),
    );
    print_box_kv(
        "🔐 IDENTITY",
        &format!(
            "key_state={} | fingerprint={} | proposer={}",
            key_state, key_fingerprint, proposer
        ),
    );
    print_box_kv(
        "🗄 STORAGE",
        &format!("state_db={} | persistence=enabled", db_path.display()),
    );
    print_box_kv(
        "📈 HEAD",
        &format!(
            "block={} | parent={} | updated_at={}",
            head, parent, state.updated_at
        ),
    );
    print_box_kv(
        "🛡 ASSESSMENT",
        &format!(
            "status={} | rpc={} | key={} | health_hint={}",
            assessment,
            status_word(rpc_reachable),
            key_state,
            health_hint_from_height(state.current_height)
        ),
    );
    print_box_kv(
        "📝 NOTE",
        if log_level == "debug" {
            "extended trace enabled | parent/proposer/unix-ts visible"
        } else {
            "use --log-level debug for extended trace fields"
        },
    );

    print_separator();
    print_table_header();
    print_border_bottom();

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
/// The loop emits a live line only after a successful production step. This
/// avoids displaying optimistic terminal events for operations that did not
/// actually advance state.
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
/// The event code is intentionally explicit. `PRD` means the engine produced a
/// new block-related state transition for this round. It does not claim economic
/// finality unless the underlying engine explicitly exposes such a state.
fn print_node_round_line(entry: &engine::RoundTelemetry, log_level: &str) {
    let timestamp = chrono::DateTime::<chrono::Utc>::from_timestamp(entry.timestamp_unix as i64, 0)
        .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
        .unwrap_or_else(|| chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string());

    println!(
        "{}{:>3}{} │ {:>5} │ {:<19} │ {:>7} │ {:>7} │ {:>4} │ {:>7} │ {:<17} │ {:<16}",
        paint(ANSI_GREEN, EVENT_PRODUCED),
        "",
        ANSI_RESET,
        entry.round_index,
        timestamp,
        entry.height,
        entry.produced_blocks,
        entry.section_count,
        entry.consensus_round,
        short_hash(&entry.block_hash_hex),
        entry.tx_id
    );

    if log_level == "debug" {
        println!(
            "{}DBG{} │ kind={} | parent={} | proposer={} | unix_ts={}",
            paint(ANSI_DIM, ""),
            ANSI_RESET,
            entry.message_kind,
            short_hash(&entry.parent_hash_hex),
            short_hash(&entry.proposer_hex),
            entry.timestamp_unix
        );
    }
}

/// Prints the end-of-session summary.
///
/// Audit Note:
/// The footer is concise by design. It provides terminal evidence of the final
/// observed runtime posture without duplicating the startup snapshot.
fn print_node_live_log_footer(state: &crate::node::state::NodeState) {
    println!(
        "{}{}────────────────────────────────────────────────────────────────────────────────────────────────────────────{}",
        ANSI_BOLD, ANSI_BORDER, ANSI_RESET
    );
    println!(
        "🏁 final_height={} | produced_blocks={} | consensus_round={} | updated_at={}",
        state.current_height,
        state.produced_blocks,
        state.consensus.last_round,
        state.updated_at
    );
    println!(
        "{}SESSION{} graceful shutdown completed",
        paint(ANSI_GREEN, "✔ "),
        ANSI_RESET
    );
    println!();
}

/// Produces a compact hash representation suitable for console use.
fn short_hash(value: &str) -> String {
    if value.is_empty() {
        return "unavailable".to_string();
    }

    if value.len() <= 18 {
        return value.to_string();
    }

    format!("{}…{}", &value[..8], &value[value.len() - 8..])
}

/// Returns a fallback string if the source string is empty after inspection.
fn non_empty_or<'a>(value: &'a str, fallback: &'a str) -> &'a str {
    if value.is_empty() {
        fallback
    } else {
        value
    }
}

/// Derives a coarse startup assessment for operator visibility.
///
/// Audit Note:
/// This function intentionally avoids overstating health. It provides a coarse
/// readiness indicator rather than an assertion of finality or economic safety.
fn startup_assessment(rpc_reachable: bool, key_state: &str, height: u64) -> &'static str {
    if !rpc_reachable {
        "degraded-rpc"
    } else if key_state == "unknown" || key_state == "unavailable" {
        "degraded-key"
    } else if height == 0 {
        "initializing"
    } else {
        "ready"
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

fn paint(code: &str, text: &str) -> String {
    format!("{code}{text}{ANSI_RESET}")
}

fn print_border_top() {
    println!(
        "{}{}╭────────────────────────────────────────────────────────────────────────────────────────────╮{}",
        ANSI_BOLD, ANSI_BORDER, ANSI_RESET
    );
}

fn print_separator() {
    println!(
        "{}{}├────────────────────────────────────────────────────────────────────────────────────────────┤{}",
        ANSI_BOLD, ANSI_BORDER, ANSI_RESET
    );
}

fn print_border_bottom() {
    println!(
        "{}{}╰────────────────────────────────────────────────────────────────────────────────────────────╯{}",
        ANSI_BOLD, ANSI_BORDER, ANSI_RESET
    );
}

fn print_box_line(title: &str, suffix: &str) {
    println!(
        "{}{}│{} {} {}{}",
        ANSI_BOLD, ANSI_BORDER, ANSI_RESET, title, suffix, pad_box_end(title, suffix)
    );
}

fn print_box_kv(label: &str, value: &str) {
    println!(
        "{}{}│{} {} {}{}",
        ANSI_BOLD, ANSI_BORDER, ANSI_RESET, paint_label(label), value, pad_box_kv_end(label, value)
    );
}

fn print_table_header() {
    println!(
        "{}{}│{} {:>3} │ {:>5} │ {:<19} │ {:>7} │ {:>7} │ {:>4} │ {:>7} │ {:<17} │ {:<16} {}{}│{}",
        ANSI_BOLD,
        ANSI_BORDER,
        ANSI_RESET,
        "EVT",
        "ROUND",
        "TIMESTAMP",
        "HEIGHT",
        "BLOCKS",
        "SEC",
        "CROUND",
        "BLOCK",
        "TX",
        ANSI_RESET,
        ANSI_BOLD,
        ANSI_BORDER,
        ANSI_RESET
    );
}

fn paint_label(label: &str) -> String {
    let color = match label {
        "🧭 RUNTIME" => ANSI_YELLOW,
        "🛰 NETWORK" => ANSI_BLUE,
        "📡 ENDPOINTS" => ANSI_CYAN,
        "🧱 CHAIN" => ANSI_MAGENTA,
        "🔐 IDENTITY" => ANSI_GREEN,
        "🗄 STORAGE" => ANSI_DIM,
        "📈 HEAD" => ANSI_RESET,
        "🛡 ASSESSMENT" => ANSI_GREEN,
        "📝 NOTE" => ANSI_YELLOW,
        _ => ANSI_RESET,
    };

    if color == ANSI_RESET {
        label.to_string()
    } else {
        format!("{color}{label}{ANSI_RESET}")
    }
}

fn pad_box_end(title: &str, suffix: &str) -> String {
    let used = visible_width(title) + 1 + visible_width(suffix);
    let total = 92usize;
    let padding = total.saturating_sub(used);
    format!("{}│", " ".repeat(padding))
}

fn pad_box_kv_end(label: &str, value: &str) -> String {
    let used = visible_width(label) + 1 + visible_width(value);
    let total = 92usize;
    let padding = total.saturating_sub(used);
    format!("{}│", " ".repeat(padding))
}

/// Approximates visible width for simple terminal layout.
///
/// Audit Note:
/// This helper intentionally uses a conservative character-count approximation.
/// It is sufficient for stable internal operator tooling, but it should not be
/// treated as full Unicode display-width accounting.
fn visible_width(value: &str) -> usize {
    value.chars().count()
}

pub fn cmd_node_health(args: &[String]) -> Result<(), AppError> {
    let health = health_status()?;
    let mut details = BTreeMap::new();
    details.insert("health".to_string(), health.to_string());

    if output_format(args) == crate::cli_support::OutputFormat::Text {
        println!("🏥 health={}", health);
    }

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
            "📡 network_smoke | bind={} | rpc_port={} | rpc={} | key_state={}",
            settings.network.bind_host,
            settings.network.rpc_port,
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
        key_summary.operational_state.clone(),
    );
    details.insert(
        "transport_public_key".to_string(),
        key_summary.transport_public_key.clone(),
    );

    if output_format(args) == crate::cli_support::OutputFormat::Text {
        println!(
            "🌐 real_network | enforce_official_peers={} | rpc={} | key_state={}",
            settings.network.enforce_official_peers,
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
            "🗄 storage_smoke | dir={} | status=writable",
            context.settings.home_dir
        );
    }

    emit_serialized(
        &text_envelope("storage-smoke", "ok", details),
        output_format(args),
    )
}
