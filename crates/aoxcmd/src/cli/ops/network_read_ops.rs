use super::*;
use sha2::{Digest, Sha256};

pub fn cmd_peer_list(args: &[String]) -> Result<(), AppError> {
    let settings = effective_settings_for_ops()?;
    let options = DiscoveryOptions::parse(args)?;
    let response = build_network_snapshot(&settings, &options);

    emit_serialized(&response, output_format(args))
}

pub fn cmd_network_status(args: &[String]) -> Result<(), AppError> {
    let settings = effective_settings_for_ops()?;
    let options = DiscoveryOptions::parse(args)?;
    let snapshot = build_network_snapshot(&settings, &options);
    let status = NetworkStatus {
        mode: snapshot.mode,
        bind_host: snapshot.bind_host,
        p2p_port: snapshot.p2p_port,
        rpc_port: snapshot.rpc_port,
        peer_count: snapshot.peer_count,
        listener_active: snapshot.listener_active,
        sync_state: snapshot.sync_state,
    };

    emit_serialized(&status, output_format(args))
}

pub fn cmd_network_full(args: &[String]) -> Result<(), AppError> {
    let settings = effective_settings_for_ops()?;
    let options = DiscoveryOptions::parse(args)?;
    let response = build_network_snapshot(&settings, &options);

    emit_serialized(&response, output_format(args))
}

#[derive(serde::Serialize)]
struct PeerView {
    peer_id: String,
    address: String,
    direction: &'static str,
    connected_since: String,
    sync_state: &'static str,
    genesis_match: bool,
    rpc_http: Option<String>,
    rpc_jsonrpc: Option<String>,
    quantum_ready: Option<bool>,
}

#[derive(serde::Serialize)]
struct NetworkSnapshot {
    mode: &'static str,
    bind_host: String,
    p2p_port: u16,
    rpc_port: u16,
    peer_count: usize,
    listener_active: bool,
    sync_state: &'static str,
    enforce_official_peers: bool,
    discovery_enabled: bool,
    genesis_fingerprint: String,
    bootstrap_limit: usize,
    quantum_only: bool,
    peers: Vec<PeerView>,
}

#[derive(serde::Serialize)]
struct NetworkStatus {
    mode: &'static str,
    bind_host: String,
    p2p_port: u16,
    rpc_port: u16,
    peer_count: usize,
    listener_active: bool,
    sync_state: &'static str,
}

#[derive(Debug, Clone)]
struct DiscoveryOptions {
    discovery_enabled: bool,
    bootstrap_limit: usize,
    quantum_only: bool,
    include_rpc: bool,
    genesis_fingerprint_override: Option<String>,
}

impl DiscoveryOptions {
    fn parse(args: &[String]) -> Result<Self, AppError> {
        let discovery_enabled = !has_flag(args, "--no-auto-discovery");
        let quantum_only = has_flag(args, "--quantum-only");
        let include_rpc = has_flag(args, "--include-rpc");
        let bootstrap_limit =
            parse_positive_u64_arg(args, "--bootstrap-limit", 8, "network")?.clamp(1, 128) as usize;
        let genesis_fingerprint_override = arg_value(args, "--genesis-fingerprint")
            .and_then(|value| normalize_text(&value, false));

        Ok(Self {
            discovery_enabled,
            bootstrap_limit,
            quantum_only,
            include_rpc,
            genesis_fingerprint_override,
        })
    }
}

fn build_network_snapshot(settings: &Settings, options: &DiscoveryOptions) -> NetworkSnapshot {
    let now = Utc::now().to_rfc3339();
    let local_genesis_fp = options
        .genesis_fingerprint_override
        .clone()
        .unwrap_or_else(|| derive_settings_genesis_fingerprint(settings));
    let mut peers = vec![PeerView {
        peer_id: "self".to_string(),
        address: format!(
            "{}:{}",
            settings.network.bind_host, settings.network.p2p_port
        ),
        direction: "inbound+outbound",
        connected_since: now,
        sync_state: "in-sync",
        genesis_match: true,
        rpc_http: options.include_rpc.then(|| {
            format!(
                "http://{}:{}",
                settings.network.bind_host, settings.network.rpc_port
            )
        }),
        rpc_jsonrpc: options.include_rpc.then(|| {
            format!(
                "http://{}:{}/jsonrpc",
                settings.network.bind_host, settings.network.rpc_port
            )
        }),
        quantum_ready: options.include_rpc.then_some(true),
    }];

    if options.discovery_enabled {
        let mut discovered = load_profile_bootnodes(settings, &local_genesis_fp, options);
        peers.append(&mut discovered);
    }

    let probe_target = format!(
        "{}:{}",
        settings.network.bind_host, settings.network.rpc_port
    );
    let listener_active = rpc_listener_active(&probe_target);
    NetworkSnapshot {
        mode: "single-node",
        bind_host: settings.network.bind_host.clone(),
        p2p_port: settings.network.p2p_port,
        rpc_port: settings.network.rpc_port,
        peer_count: peers.len(),
        listener_active,
        sync_state: "in-sync",
        enforce_official_peers: settings.network.enforce_official_peers,
        discovery_enabled: options.discovery_enabled,
        genesis_fingerprint: local_genesis_fp,
        bootstrap_limit: options.bootstrap_limit,
        quantum_only: options.quantum_only,
        peers,
    }
}

fn load_profile_bootnodes(
    settings: &Settings,
    local_genesis_fp: &str,
    options: &DiscoveryOptions,
) -> Vec<PeerView> {
    let path = Path::new("configs")
        .join("environments")
        .join(&settings.profile)
        .join("bootnodes.json");
    let Ok(raw) = fs::read_to_string(path) else {
        return Vec::new();
    };
    let Ok(json) = serde_json::from_str::<serde_json::Value>(&raw) else {
        return Vec::new();
    };
    let Some(items) = json.get("bootnodes").and_then(serde_json::Value::as_array) else {
        return Vec::new();
    };

    items
        .iter()
        .filter_map(|item| {
            let node_id = item.get("node_id")?.as_str()?.to_string();
            let address = item.get("address")?.as_str()?.to_string();
            let fingerprint = item
                .get("transport_key_fingerprint")
                .and_then(serde_json::Value::as_str)
                .unwrap_or_default()
                .to_string();
            let genesis_match = !fingerprint.is_empty()
                && fingerprint.starts_with(&local_genesis_fp[..8.min(local_genesis_fp.len())]);
            if !genesis_match {
                return None;
            }

            let quantum_ready = fingerprint.len() >= 64;
            if options.quantum_only && !quantum_ready {
                return None;
            }

            Some(PeerView {
                peer_id: node_id.clone(),
                address,
                direction: "outbound",
                connected_since: Utc::now().to_rfc3339(),
                sync_state: "bootstrap",
                genesis_match,
                rpc_http: options
                    .include_rpc
                    .then(|| format!("http://{}.rpc.local:{}", node_id, settings.network.rpc_port)),
                rpc_jsonrpc: options.include_rpc.then(|| {
                    format!(
                        "http://{}.rpc.local:{}/jsonrpc",
                        node_id, settings.network.rpc_port
                    )
                }),
                quantum_ready: options.include_rpc.then_some(quantum_ready),
            })
        })
        .take(options.bootstrap_limit)
        .collect()
}

fn derive_settings_genesis_fingerprint(settings: &Settings) -> String {
    let mut hasher = Sha256::new();
    hasher.update(settings.profile.as_bytes());
    hasher.update(settings.network.bind_host.as_bytes());
    hasher.update(settings.network.p2p_port.to_le_bytes());
    hasher.update(settings.network.rpc_port.to_le_bytes());
    hex::encode(hasher.finalize())
}

pub fn cmd_state_root(args: &[String]) -> Result<(), AppError> {
    #[derive(serde::Serialize)]
    struct StateRoot {
        state_root: String,
        height: u64,
        updated_at: String,
        source: String,
    }

    let state = lifecycle::load_state()?;
    let requested_height = arg_value(args, "--height")
        .and_then(|value| normalize_text(&value, false))
        .and_then(|value| value.parse::<u64>().ok());
    let indexed = if let Some(height) = requested_height {
        load_state_root_for_height(height)?
    } else {
        None
    };
    let source = if indexed.is_some() {
        "state-root-index".to_string()
    } else {
        "runtime-snapshot".to_string()
    };
    let response = StateRoot {
        state_root: indexed
            .as_ref()
            .map(|(_, root)| root.clone())
            .unwrap_or(derive_state_root(&state)?),
        height: indexed
            .map(|(height, _)| height)
            .unwrap_or(state.current_height),
        updated_at: state.updated_at,
        source,
    };

    emit_serialized(&response, output_format(args))
}
