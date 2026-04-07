use super::*;
use serde_json::json;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

pub fn cmd_rpc_serve(args: &[String]) -> Result<(), AppError> {
    let settings = effective_settings_for_ops()?;
    let host =
        parse_required_or_default_text_arg(args, "--host", &settings.network.bind_host, true)?;
    let rpc_port =
        parse_optional_port_arg(args, "--rpc-port")?.unwrap_or(settings.network.rpc_port);
    let metrics_port = parse_optional_port_arg(args, "--metrics-port")?
        .unwrap_or(settings.telemetry.prometheus_port);

    let rpc_listener = TcpListener::bind((host.as_str(), rpc_port)).map_err(|error| {
        AppError::new(
            ErrorCode::RuntimeUnavailable,
            format!("Failed to bind RPC listener on {host}:{rpc_port}: {error}"),
        )
    })?;

    let metrics_listener = TcpListener::bind((host.as_str(), metrics_port)).map_err(|error| {
        AppError::new(
            ErrorCode::RuntimeUnavailable,
            format!("Failed to bind metrics listener on {host}:{metrics_port}: {error}"),
        )
    })?;

    println!(
        "RPC_SERVE | rpc=http://{}:{} | metrics=http://{}:{}/metrics",
        host, rpc_port, host, metrics_port
    );

    std::thread::spawn(move || {
        for stream in metrics_listener.incoming().flatten() {
            let _ = handle_connection(stream, true);
        }
    });

    for stream in rpc_listener.incoming().flatten() {
        let _ = handle_connection(stream, false);
    }

    Ok(())
}

fn parse_optional_port_arg(args: &[String], flag: &str) -> Result<Option<u16>, AppError> {
    let Some(raw) = parse_optional_text_arg(args, flag, false) else {
        return Ok(None);
    };

    let parsed = parse_positive_u64_value(&raw, flag, "rpc-serve")?;
    if parsed > u16::MAX as u64 {
        return Err(AppError::new(
            ErrorCode::UsageInvalidArguments,
            format!("Flag {flag} must be between 1 and {}", u16::MAX),
        ));
    }

    Ok(Some(parsed as u16))
}

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
        if method == "GET" && target.starts_with("/metrics") {
            http_ok("text/plain; version=0.0.4", &metrics_payload())
        } else if method == "GET" && target.starts_with("/health") {
            http_ok(
                "application/json",
                &json!({"status":"ok","service":"metrics"}).to_string(),
            )
        } else {
            http_not_found(method, target)
        }
    } else {
        route_rpc(method, target, body)
    };

    stream.write_all(response.as_bytes())?;
    stream.flush()?;
    Ok(())
}

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

    if method == "POST" {
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(body) {
            if v.get("method") == Some(&serde_json::Value::String("status".to_string())) {
                return http_ok(
                    "application/json",
                    &json!({"jsonrpc":"2.0","id":v.get("id").cloned().unwrap_or(json!(1)),"result":chain_status_json()})
                        .to_string(),
                );
            }
        }
    }

    match (method, target) {
        ("GET", "/status") | ("GET", "/chain/status") => {
            http_ok("application/json", &chain_status_json().to_string())
        }
        ("GET", "/network/status") => {
            http_ok("application/json", &network_status_json().to_string())
        }
        ("GET", "/peer/list") => http_ok("application/json", &json!([]).to_string()),
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

fn chain_status_json() -> serde_json::Value {
    match lifecycle::load_state() {
        Ok(state) => json!({
            "network_id": state.consensus.network_id,
            "height": state.current_height,
            "last_block_hash": state.consensus.last_block_hash_hex,
            "produced_blocks": state.produced_blocks,
        }),
        Err(_) => json!({"network_id":"AOXC-UNSET","height":0_u64,"degraded":true}),
    }
}

fn network_status_json() -> serde_json::Value {
    match effective_settings_for_ops() {
        Ok(settings) => json!({
            "bind_host": settings.network.bind_host,
            "p2p_port": settings.network.p2p_port,
            "rpc_port": settings.network.rpc_port,
            "metrics_port": settings.telemetry.prometheus_port,
        }),
        Err(_) => json!({"degraded":true}),
    }
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

fn block_view_json(target: &str) -> serde_json::Value {
    json!({
        "height": query_value(target, "height").unwrap_or_else(|| "latest".to_string()),
        "hash": lifecycle::load_state().ok().map(|s| s.consensus.last_block_hash_hex).unwrap_or_default(),
    })
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
    json!({
        "account": query_value(target, "account").unwrap_or_else(|| "unknown".to_string()),
        "nonce": 0_u64,
    })
}

fn balance_json(target: &str) -> serde_json::Value {
    json!({
        "account": query_value(target, "account").unwrap_or_else(|| "unknown".to_string()),
        "balance": 0_u64,
    })
}

fn metrics_payload() -> String {
    "# HELP aox_node_up AOXC node process health\n# TYPE aox_node_up gauge\naox_node_up 1\n# HELP aox_rpc_queryable RPC query availability\n# TYPE aox_rpc_queryable gauge\naox_rpc_queryable 1\n"
        .to_string()
}

fn http_ok(content_type: &str, body: &str) -> String {
    format!(
        "HTTP/1.1 200 OK\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    )
}

fn http_not_found(method: &str, target: &str) -> String {
    let body =
        json!({"code":"METHOD_NOT_FOUND","message":format!("unsupported route {method} {target}")})
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
}
