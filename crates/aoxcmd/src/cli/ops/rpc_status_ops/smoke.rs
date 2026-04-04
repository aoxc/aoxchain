use super::*;

pub fn cmd_rpc_curl_smoke(args: &[String]) -> Result<(), AppError> {
    #[derive(serde::Serialize)]
    struct ProbeResult {
        target: String,
        ok: bool,
    }

    #[derive(serde::Serialize)]
    struct RpcCurlSmokeReport {
        listener_active: bool,
        probe_target: String,
        http_base_url: String,
        probes: Vec<ProbeResult>,
        all_passed: bool,
    }

    let settings = effective_settings_for_ops()?;
    let probe_target = format!(
        "{}:{}",
        settings.network.bind_host, settings.network.rpc_port
    );
    let listener_active = rpc_listener_active(&probe_target);
    let curl_host = if settings.network.bind_host == "0.0.0.0" {
        "127.0.0.1".to_string()
    } else {
        settings.network.bind_host.clone()
    };
    let http_base_url = format!("http://{}:{}", curl_host, settings.network.rpc_port);

    let mut probes = vec![
        ProbeResult {
            target: format!("GET {http_base_url}/health"),
            ok: listener_active
                && rpc_http_get_probe(&curl_host, settings.network.rpc_port, "/health"),
        },
        ProbeResult {
            target: format!("GET {http_base_url}/status"),
            ok: listener_active
                && rpc_http_get_probe(&curl_host, settings.network.rpc_port, "/status"),
        },
        ProbeResult {
            target: format!("GET {http_base_url}/chain/status"),
            ok: listener_active
                && rpc_http_get_probe(&curl_host, settings.network.rpc_port, "/chain/status"),
        },
        ProbeResult {
            target: format!("GET {http_base_url}/consensus/status"),
            ok: listener_active
                && rpc_http_get_probe(&curl_host, settings.network.rpc_port, "/consensus/status"),
        },
        ProbeResult {
            target: format!("GET {http_base_url}/vm/status"),
            ok: listener_active
                && rpc_http_get_probe(&curl_host, settings.network.rpc_port, "/vm/status"),
        },
    ];

    probes.push(ProbeResult {
        target: format!(
            "POST {http_base_url} {{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"status\",\"params\":[]}}"
        ),
        ok: listener_active && rpc_jsonrpc_status_probe(&curl_host, settings.network.rpc_port),
    });

    let report = RpcCurlSmokeReport {
        listener_active,
        probe_target,
        http_base_url,
        all_passed: probes.iter().all(|probe| probe.ok),
        probes,
    };
    emit_serialized(&report, output_format(args))
}
