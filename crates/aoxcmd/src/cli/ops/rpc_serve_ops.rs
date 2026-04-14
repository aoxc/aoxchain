use super::*;
use serde_json::json;
use sha2::{Digest, Sha256};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

/// Starts RPC and metrics listeners in the foreground.
///
/// Audit Note:
/// This command is intended for explicit operator invocation. It binds both
/// listeners and serves requests synchronously until the process is terminated.
pub fn cmd_rpc_serve(args: &[String]) -> Result<(), AppError> {
    let settings = effective_settings_for_ops()?;
    let host =
        parse_required_or_default_text_arg(args, "--host", &settings.network.bind_host, true)?;
    let rpc_port =
        parse_optional_port_arg(args, "--rpc-port")?.unwrap_or(settings.network.rpc_port);
    let metrics_port = parse_optional_port_arg(args, "--metrics-port")?
        .unwrap_or(settings.telemetry.prometheus_port);

    let rpc_listener = bind_listener(&host, rpc_port, "RPC")?;
    let metrics_listener = bind_listener(&host, metrics_port, "metrics")?;

    println!(
        "RPC_SERVE | rpc=http://{}:{} | metrics=http://{}:{}/metrics",
        host, rpc_port, host, metrics_port
    );

    serve_listeners(rpc_listener, metrics_listener);

    Ok(())
}

/// Spawns RPC and metrics listeners in background threads.
///
/// Audit Note:
/// This helper is intended for integration with node execution flows. It binds
/// listeners once and returns immediately after spawning serving threads.
pub fn spawn_rpc_and_metrics_listeners(
    host: &str,
    rpc_port: u16,
    metrics_port: u16,
) -> Result<(), AppError> {
    let rpc_listener = bind_listener(host, rpc_port, "RPC")?;
    let metrics_listener = bind_listener(host, metrics_port, "metrics")?;

    println!(
        "RPC_LISTEN | rpc=http://{}:{} | metrics=http://{}:{}/metrics",
        host, rpc_port, host, metrics_port
    );

    std::thread::spawn(move || {
        for stream in metrics_listener.incoming().flatten() {
            let _ = handle_connection(stream, true);
        }
    });

    std::thread::spawn(move || {
        for stream in rpc_listener.incoming().flatten() {
            let _ = handle_connection(stream, false);
        }
    });

    Ok(())
}

/// Binds a TCP listener and maps binding failures to the canonical AOXC error
/// taxonomy.
///
/// Audit Note:
/// Binding failures are classified as OS-level I/O failures because socket bind
/// errors originate from the operating environment and resource layer.
fn bind_listener(host: &str, port: u16, label: &str) -> Result<TcpListener, AppError> {
    TcpListener::bind((host, port)).map_err(|error| {
        AppError::with_source(
            ErrorCode::FilesystemIoFailed,
            format!("Failed to bind {label} listener on {host}:{port}"),
            error,
        )
    })
}

/// Serves both listeners in foreground mode.
///
/// Audit Note:
/// Metrics is served on a background thread while RPC remains attached to the
/// foreground flow, preserving the expected blocking behavior of `cmd_rpc_serve`.
fn serve_listeners(rpc_listener: TcpListener, metrics_listener: TcpListener) {
    std::thread::spawn(move || {
        for stream in metrics_listener.incoming().flatten() {
            let _ = handle_connection(stream, true);
        }
    });

    for stream in rpc_listener.incoming().flatten() {
        let _ = handle_connection(stream, false);
    }
}

fn parse_optional_port_arg(args: &[String], flag: &str) -> Result<Option<u16>, AppError> {
    let Some(raw) = parse_optional_text_arg(args, flag, false) else {
        return Ok(None);
    };

    let parsed = parse_positive_u64_value(&raw, flag, "rpc-serve")?;
    if parsed == 0 || parsed > u16::MAX as u64 {
        return Err(AppError::new(
            ErrorCode::UsageInvalidArguments,
            format!("Flag {flag} must be between 1 and {}", u16::MAX),
        ));
    }

    Ok(Some(parsed as u16))
}

/// Handles a single HTTP connection.
///
/// Audit Note:
/// This implementation is intentionally lightweight and dependency-minimal. It
/// is suitable for internal operator tooling and controlled testnet exposure.
/// It should not be treated as a hardened internet-facing gateway without
/// additional request validation, concurrency controls, and timeout policy.
fn handle_connection(mut stream: TcpStream, metrics_only: bool) -> std::io::Result<()> {
    let mut buf = [0_u8; 16_384];
    let size = stream.read(&mut buf)?;
    if size == 0 {
        return Ok(());
    }

    let request = String::from_utf8_lossy(&buf[..size]);
    let mut lines = request.lines();
    let request_line = lines.next().unwrap_or_default();
    let mut parts = request_line.split_whitespace();
    let method = parts.next().unwrap_or("GET");
    let target = parts.next().unwrap_or("/");

    let (_, body) = request.split_once("\r\n\r\n").unwrap_or(("", ""));

    let response = if metrics_only {
        route_metrics(method, target)
    } else {
        route_rpc(method, target, body)
    };

    stream.write_all(response.as_bytes())?;
    stream.flush()?;
    Ok(())
}

/// Routes requests for the dedicated metrics listener.
fn route_metrics(method: &str, target: &str) -> String {
    if method == "GET" && target.starts_with("/metrics") {
        return http_ok("text/plain; version=0.0.4", &metrics_payload());
    }

    if method == "GET" && (target == "/health" || target == "/live") {
        return http_ok(
            "application/json",
            &json!({"status":"ok","service":"metrics"}).to_string(),
        );
    }

    if method == "GET" && target == "/ready" {
        return http_ok(
            "application/json",
            &json!({"status":"ready","service":"metrics"}).to_string(),
        );
    }

    http_not_found(method, target)
}

/// Routes HTTP and JSON-RPC requests for the RPC listener.
///
/// Audit Note:
/// The response model is intentionally conservative. The exposed surfaces are
/// operational query helpers and must not be mistaken for a final public API
/// contract unless separately versioned and documented.
fn route_rpc(method: &str, target: &str, body: &str) -> String {
    if method == "GET" && (target == "/health" || target == "/live") {
        return http_ok("application/json", &json!({"status":"ok"}).to_string());
    }

    if method == "GET" && target == "/ready" {
        return http_ok(
            "application/json",
            &json!({"status":"ready","queryable":true}).to_string(),
        );
    }

    if method == "GET" && target == "/metrics" {
        return http_ok("text/plain; version=0.0.4", &metrics_payload());
    }

    if method == "POST"
        && let Ok(v) = serde_json::from_str::<serde_json::Value>(body)
        && v.get("method") == Some(&serde_json::Value::String("status".to_string()))
    {
        return http_ok(
            "application/json",
            &json!({
                "jsonrpc":"2.0",
                "id":v.get("id").cloned().unwrap_or(json!(1)),
                "result":chain_status_json()
            })
            .to_string(),
        );
    }

    match (method, target) {
        ("GET", "/status") | ("GET", "/chain/status") => {
            http_ok("application/json", &chain_status_json().to_string())
        }
        ("GET", "/network/status") => {
            http_ok("application/json", &network_status_json().to_string())
        }
        ("GET", "/peer/list") => http_ok("application/json", &peer_list_json().to_string()),
        ("GET", "/state/root") => http_ok(
            "application/json",
            &json!({"state_root":state_root_hex()}).to_string(),
        ),
        _ => {
            if method == "GET" && target.starts_with("/block/get") {
                return http_ok("application/json", &block_view_json(target).to_string());
            }

            if method == "GET" && target.starts_with("/tx/get") {
                return http_ok("application/json", &tx_view_json(target).to_string());
            }

            if method == "GET" && target.starts_with("/tx/receipt") {
                return http_ok("application/json", &tx_receipt_json(target).to_string());
            }

            if method == "GET" && target.starts_with("/account/get") {
                return http_ok("application/json", &account_json(target).to_string());
            }

            if method == "GET" && target.starts_with("/balance/get") {
                return http_ok("application/json", &balance_json(target).to_string());
            }

            http_not_found(method, target)
        }
    }
}

/// Produces a chain-status view from local runtime state.
fn chain_status_json() -> serde_json::Value {
    match lifecycle::load_state() {
        Ok(state) => json!({
            "network_id": state.consensus.network_id,
            "height": state.current_height,
            "last_block_hash": state.consensus.last_block_hash_hex,
            "last_parent_hash": state.consensus.last_parent_hash_hex,
            "last_round": state.consensus.last_round,
            "produced_blocks": state.produced_blocks,
            "section_count": state.consensus.last_section_count,
            "updated_at": state.updated_at,
        }),
        Err(_) => json!({
            "network_id":"AOXC-UNSET",
            "height":0_u64,
            "degraded":true
        }),
    }
}

/// Produces a network-status view from effective runtime settings.
fn network_status_json() -> serde_json::Value {
    match effective_settings_for_ops() {
        Ok(settings) => json!({
            "bind_host": settings.network.bind_host,
            "p2p_port": settings.network.p2p_port,
            "rpc_port": settings.network.rpc_port,
            "metrics_port": settings.telemetry.prometheus_port,
            "enforce_official_peers": settings.network.enforce_official_peers,
            "profile": settings.profile,
        }),
        Err(_) => json!({"degraded":true}),
    }
}

/// Returns an explicit empty peer list until real peer inventory exposure is available.
///
/// Audit Note:
/// Returning an empty set is preferable to fabricating peer topology.
fn peer_list_json() -> serde_json::Value {
    json!([])
}

fn state_root_hex() -> String {
    match lifecycle::load_state() {
        Ok(state) => {
            let mut h = Sha256::new();
            h.update(state.consensus.last_block_hash_hex.as_bytes());
            h.update(state.current_height.to_be_bytes());
            format!("0x{}", hex::encode(h.finalize()))
        }
        Err(_) => "0x0000000000000000000000000000000000000000000000000000000000000000".to_string(),
    }
}

fn query_value(target: &str, key: &str) -> Option<String> {
    let (_, query) = target.split_once('?')?;
    for item in query.split('&') {
        let (k, v) = item.split_once('=')?;
        if k == key {
            return Some(v.to_string());
        }
    }
    None
}

fn account_id_from_target(target: &str) -> Option<String> {
    query_value(target, "id")
        .or_else(|| query_value(target, "account"))
        .and_then(|value| normalize_text(&value, false))
}

struct AccountLookupResult {
    account_id: String,
    known: bool,
    balance: u64,
    source: &'static str,
    degraded: bool,
}

fn lookup_account_from_ledger(target: &str) -> AccountLookupResult {
    let account_id = account_id_from_target(target).unwrap_or_else(|| "unknown".to_string());

    match ledger::load() {
        Ok(ledger_state) => {
            let known =
                account_id == "treasury" || ledger_state.delegations.contains_key(&account_id);
            let balance = if account_id == "treasury" {
                ledger_state.treasury_balance
            } else {
                ledger_state
                    .delegations
                    .get(&account_id)
                    .copied()
                    .unwrap_or(0)
            };
            AccountLookupResult {
                account_id,
                known,
                balance,
                source: "local-ledger",
                degraded: false,
            }
        }
        Err(_) => AccountLookupResult {
            account_id,
            known: false,
            balance: 0,
            source: "ledger-unavailable",
            degraded: true,
        },
    }
}

fn block_view_json(target: &str) -> serde_json::Value {
    let requested_height = query_value(target, "height").unwrap_or_else(|| "latest".to_string());

    match lifecycle::load_state() {
        Ok(state) => json!({
            "height": requested_height,
            "hash": state.consensus.last_block_hash_hex,
            "parent_hash": state.consensus.last_parent_hash_hex,
            "round": state.consensus.last_round,
            "section_count": state.consensus.last_section_count,
            "proposer": state.consensus.last_proposer_hex,
        }),
        Err(_) => json!({
            "height": requested_height,
            "degraded": true
        }),
    }
}

fn tx_view_json(target: &str) -> serde_json::Value {
    json!({
        "tx_id": query_value(target, "tx_id").unwrap_or_else(|| "unknown".to_string()),
        "status": "observed",
    })
}

fn tx_receipt_json(target: &str) -> serde_json::Value {
    json!({
        "tx_id": query_value(target, "tx_id").unwrap_or_else(|| "unknown".to_string()),
        "success": true,
        "gas_used": 0_u64,
    })
}

fn account_json(target: &str) -> serde_json::Value {
    let lookup = lookup_account_from_ledger(target);

    json!({
        "account": lookup.account_id,
        "known": lookup.known,
        "balance": lookup.balance,
        "nonce": 0_u64,
        "source": lookup.source,
        "degraded": lookup.degraded,
    })
}

fn balance_json(target: &str) -> serde_json::Value {
    let lookup = lookup_account_from_ledger(target);

    json!({
        "account": lookup.account_id,
        "known": lookup.known,
        "balance": lookup.balance,
        "source": lookup.source,
        "degraded": lookup.degraded,
    })
}

/// Emits a minimal Prometheus payload.
///
/// Audit Note:
/// Metrics are intentionally conservative and truthful. Minimal exposure is
/// preferable to overstating observability coverage.
fn metrics_payload() -> String {
    let (height, produced_blocks) = match lifecycle::load_state() {
        Ok(state) => (state.current_height, state.produced_blocks),
        Err(_) => (0_u64, 0_u64),
    };

    format!(
        "# HELP aox_node_up AOXC node process health\n\
# TYPE aox_node_up gauge\n\
aox_node_up 1\n\
# HELP aox_rpc_queryable RPC query availability\n\
# TYPE aox_rpc_queryable gauge\n\
aox_rpc_queryable 1\n\
# HELP aox_chain_height AOXC current chain height\n\
# TYPE aox_chain_height gauge\n\
aox_chain_height {height}\n\
# HELP aox_produced_blocks AOXC produced block count\n\
# TYPE aox_produced_blocks counter\n\
aox_produced_blocks {produced_blocks}\n"
    )
}

fn http_ok(content_type: &str, body: &str) -> String {
    format!(
        "HTTP/1.1 200 OK\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    )
}

fn http_not_found(method: &str, target: &str) -> String {
    let body = json!({
        "code":"METHOD_NOT_FOUND",
        "message": format!("unsupported route {method} {target}")
    })
    .to_string();

    format!(
        "HTTP/1.1 404 Not Found\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn route_rpc_exposes_readiness_and_liveness() {
        let live = route_rpc("GET", "/live", "");
        assert!(live.contains("200 OK"));

        let ready = route_rpc("GET", "/ready", "");
        assert!(ready.contains("queryable"));
    }

    #[test]
    fn route_rpc_exposes_query_surfaces() {
        let chain = route_rpc("GET", "/chain/status", "");
        assert!(chain.contains("height"));

        let tx = route_rpc("GET", "/tx/get?tx_id=abc", "");
        assert!(tx.contains("abc"));
    }

    #[test]
    fn route_rpc_accepts_jsonrpc_status() {
        let body = r#"{"jsonrpc":"2.0","id":7,"method":"status","params":[]}"#;
        let response = route_rpc("POST", "/", body);
        assert!(response.contains("\"jsonrpc\":\"2.0\""));
        assert!(response.contains("\"id\":7"));
    }

    #[test]
    fn route_rpc_account_and_balance_accept_id_alias() {
        let account = route_rpc("GET", "/account/get?id=treasury", "");
        assert!(account.contains("\"account\":\"treasury\""));
        assert!(account.contains("\"known\":"));
        assert!(account.contains("\"source\":"));

        let balance = route_rpc("GET", "/balance/get?id=treasury", "");
        assert!(balance.contains("\"account\":\"treasury\""));
        assert!(balance.contains("\"known\":"));
        assert!(balance.contains("\"source\":"));
    }

    #[test]
    fn account_id_lookup_accepts_account_and_id_keys() {
        assert_eq!(
            account_id_from_target("/account/get?id=AOXC_TEST_ABC"),
            Some("AOXC_TEST_ABC".to_string())
        );
        assert_eq!(
            account_id_from_target("/account/get?account=AOXC_TEST_DEF"),
            Some("AOXC_TEST_DEF".to_string())
        );
        assert_eq!(account_id_from_target("/account/get"), None);
    }
}
