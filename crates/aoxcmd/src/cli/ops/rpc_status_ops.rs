use super::*;

pub fn cmd_rpc_status(args: &[String]) -> Result<(), AppError> {
    #[derive(serde::Serialize)]
    struct RpcStatus {
        enabled: bool,
        bind_host: String,
        port: u16,
        http_ready: bool,
        jsonrpc_ready: bool,
        required_endpoint_ready: bool,
        uptime_secs: u64,
        listener_active: bool,
        curl_compatible: bool,
        probe_target: String,
        http_base_url: String,
        probe_mode: &'static str,
        required_endpoint_probes: BTreeMap<&'static str, bool>,
        jsonrpc_status_probe: bool,
        rest_endpoints: Vec<&'static str>,
        json_rpc_methods: Vec<&'static str>,
        curl_examples: BTreeMap<&'static str, String>,
    }

    let settings = effective_settings_for_ops()?;
    let state = lifecycle::load_state()?;
    let probe_target = format!(
        "{}:{}",
        settings.network.bind_host, settings.network.rpc_port
    );
    let listener_active = rpc_listener_active(&probe_target);
    let uptime_secs = uptime_secs_from_rfc3339(&state.updated_at);
    let curl_host = if settings.network.bind_host == "0.0.0.0" {
        "127.0.0.1".to_string()
    } else {
        settings.network.bind_host.clone()
    };
    let http_base_url = format!("http://{}:{}", curl_host, settings.network.rpc_port);
    let mut curl_examples = BTreeMap::new();
    curl_examples.insert("health", format!("curl -fsS {http_base_url}/health"));
    curl_examples.insert("status", format!("curl -fsS {http_base_url}/status"));
    curl_examples.insert(
        "latest-block",
        format!("curl -fsS {http_base_url}/block/latest"),
    );
    curl_examples.insert(
        "consensus-status",
        format!("curl -fsS {http_base_url}/consensus/status"),
    );
    curl_examples.insert("vm-status", format!("curl -fsS {http_base_url}/vm/status"));
    curl_examples.insert(
        "json-rpc-status",
        format!(
            "curl -fsS -H 'content-type: application/json' -d '{{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"status\",\"params\":[]}}' {http_base_url}"
        ),
    );
    curl_examples.insert(
        "faucet-status",
        format!("curl -fsS {http_base_url}/faucet/status"),
    );
    curl_examples.insert(
        "faucet-claim",
        format!(
            "curl -fsS -X POST -H 'content-type: application/json' -d '{{\"account_id\":\"devnet-user\",\"amount\":1000}}' {http_base_url}/faucet/claim"
        ),
    );
    let required_paths = [
        "/health",
        "/status",
        "/chain/status",
        "/consensus/status",
        "/vm/status",
    ];
    let mut required_endpoint_probes = BTreeMap::new();
    if listener_active {
        for path in required_paths {
            required_endpoint_probes.insert(
                path,
                rpc_http_get_probe(&curl_host, settings.network.rpc_port, path),
            );
        }
    } else {
        for path in required_paths {
            required_endpoint_probes.insert(path, false);
        }
    }
    let required_endpoint_ready = required_endpoint_probes.values().all(|ready| *ready);
    let jsonrpc_status_probe =
        listener_active && rpc_jsonrpc_status_probe(&curl_host, settings.network.rpc_port);
    let response = RpcStatus {
        enabled: true,
        bind_host: settings.network.bind_host.clone(),
        port: settings.network.rpc_port,
        http_ready: required_endpoint_ready,
        jsonrpc_ready: jsonrpc_status_probe,
        required_endpoint_ready,
        uptime_secs,
        listener_active,
        curl_compatible: required_endpoint_ready && jsonrpc_status_probe,
        probe_target,
        http_base_url,
        probe_mode: if listener_active {
            "tcp+http-active-probe"
        } else {
            "tcp-connect"
        },
        required_endpoint_probes,
        jsonrpc_status_probe,
        rest_endpoints: vec![
            "/health",
            "/status",
            "/metrics",
            "/chain/status",
            "/block/latest",
            "/block/{height}",
            "/tx/{hash}",
            "/tx/{hash}/receipt",
            "/account/{id}",
            "/consensus/status",
            "/network/peers",
            "/vm/status",
            "/state/root",
            "/rpc/status",
            "/faucet/status",
            "/faucet/claim",
            "/faucet/history/{account_id}",
            "/faucet/balance",
            "/faucet/config",
            "/faucet/enable",
            "/faucet/disable",
            "/faucet/ban",
            "/faucet/unban",
            "/faucet/config/update",
        ],
        json_rpc_methods: vec![
            "status",
            "getLatestBlock",
            "getBlockByHeight",
            "getBlockByHash",
            "getTxByHash",
            "getReceiptByHash",
            "getAccount",
            "getBalance",
            "getStateRoot",
            "getConsensusStatus",
            "getNetworkStatus",
            "getPeers",
            "getVmStatus",
        ],
        curl_examples,
    };

    emit_serialized(&response, output_format(args))
}

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
