use super::*;
use sha2::{Digest, Sha256};
use std::{collections::BTreeSet, net::IpAddr, path::PathBuf};

pub fn cmd_peer_list(args: &[String]) -> Result<(), AppError> {
    let settings = effective_settings_for_ops()?;
    let options = DiscoveryOptions::parse(args)?;
    let response = build_network_snapshot(&settings, &options)?;

    emit_serialized(&response, output_format(args))
}

pub fn cmd_network_status(args: &[String]) -> Result<(), AppError> {
    let settings = effective_settings_for_ops()?;
    let options = DiscoveryOptions::parse(args)?;
    let snapshot = build_network_snapshot(&settings, &options)?;
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
    let response = build_network_snapshot(&settings, &options)?;

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
    known_bootnodes: BTreeSet<String>,
    known_bootnode_file: Option<PathBuf>,
    bootnodes_file_override: Option<PathBuf>,
    bootnodes_sha256: Option<String>,
    certificate_file_override: Option<PathBuf>,
    certificate_sha256: Option<String>,
    strict_bootnode_id: bool,
    strict_security: bool,
    require_official_peers: bool,
    deny_private_peers: bool,
    require_bootnodes_sha256: bool,
    min_peer_count: usize,
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
        let known_bootnodes = collect_flag_values(args, "--known-bootnode")
            .into_iter()
            .filter_map(|value| normalize_text(&value, false))
            .collect::<BTreeSet<_>>();
        let known_bootnode_file = arg_value(args, "--known-bootnode-file").map(PathBuf::from);
        let bootnodes_file_override = arg_value(args, "--bootnodes-file").map(PathBuf::from);
        let bootnodes_sha256 =
            arg_value(args, "--bootnodes-sha256").and_then(|value| normalize_text(&value, true));
        let certificate_file_override = arg_value(args, "--certificate-file").map(PathBuf::from);
        let certificate_sha256 =
            arg_value(args, "--certificate-sha256").and_then(|value| normalize_text(&value, true));
        let strict_bootnode_id = has_flag(args, "--strict-bootnode-id");
        let strict_security = has_flag(args, "--strict-security");
        let require_official_peers = strict_security || has_flag(args, "--require-official-peers");
        let deny_private_peers = strict_security || has_flag(args, "--deny-private-peers");
        let require_bootnodes_sha256 =
            strict_security || has_flag(args, "--require-bootnodes-sha256");
        let min_peer_count =
            parse_positive_u64_arg(args, "--min-peer-count", 1, "network")?.clamp(1, 256) as usize;

        Ok(Self {
            discovery_enabled,
            bootstrap_limit,
            quantum_only,
            include_rpc,
            genesis_fingerprint_override,
            known_bootnodes,
            known_bootnode_file,
            bootnodes_file_override,
            bootnodes_sha256,
            certificate_file_override,
            certificate_sha256,
            strict_bootnode_id,
            strict_security,
            require_official_peers,
            deny_private_peers,
            require_bootnodes_sha256,
            min_peer_count,
        })
    }
}

fn build_network_snapshot(
    settings: &Settings,
    options: &DiscoveryOptions,
) -> Result<NetworkSnapshot, AppError> {
    if options.require_official_peers && !settings.network.enforce_official_peers {
        return Err(AppError::new(
            ErrorCode::UsageInvalidArguments,
            "Official peer enforcement must be enabled for this network query",
        ));
    }

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
        let mut discovered = load_profile_bootnodes(settings, &local_genesis_fp, options)?;
        peers.append(&mut discovered);
    }

    if peers.len() < options.min_peer_count {
        return Err(AppError::new(
            ErrorCode::ConfigInvalid,
            format!(
                "Insufficient peer inventory: expected at least {} peer(s), discovered {}",
                options.min_peer_count,
                peers.len()
            ),
        ));
    }

    let probe_target = format!(
        "{}:{}",
        settings.network.bind_host, settings.network.rpc_port
    );
    let listener_active = rpc_listener_active(&probe_target);
    Ok(NetworkSnapshot {
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
    })
}

fn load_profile_bootnodes(
    settings: &Settings,
    local_genesis_fp: &str,
    options: &DiscoveryOptions,
) -> Result<Vec<PeerView>, AppError> {
    let path = options.bootnodes_file_override.clone().unwrap_or_else(|| {
        Path::new("configs")
            .join("environments")
            .join(&settings.profile)
            .join("bootnodes.json")
    });
    let raw = fs::read_to_string(&path).map_err(|error| {
        AppError::with_source(
            ErrorCode::FilesystemIoFailed,
            format!("Failed to read bootnodes file: {}", path.display()),
            error,
        )
    })?;
    if options.require_bootnodes_sha256 && options.bootnodes_sha256.is_none() {
        return Err(AppError::new(
            ErrorCode::UsageInvalidArguments,
            "Bootnodes integrity hash is required: pass --bootnodes-sha256",
        ));
    }

    if let Some(expected) = options.bootnodes_sha256.as_ref() {
        verify_sha256_hex(&raw, expected, "bootnodes file")?;
    }
    if let Some(certificate_path) = options.certificate_file_override.as_ref() {
        let certificate_raw = fs::read_to_string(certificate_path).map_err(|error| {
            AppError::with_source(
                ErrorCode::FilesystemIoFailed,
                format!(
                    "Failed to read certificate file: {}",
                    certificate_path.display()
                ),
                error,
            )
        })?;
        if let Some(expected) = options.certificate_sha256.as_ref() {
            verify_sha256_hex(&certificate_raw, expected, "certificate file")?;
        }
    }
    let json = serde_json::from_str::<serde_json::Value>(&raw).map_err(|error| {
        AppError::with_source(
            ErrorCode::ConfigInvalid,
            format!("Invalid bootnodes JSON: {}", path.display()),
            error,
        )
    })?;
    let items = json
        .get("bootnodes")
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| {
            AppError::new(
                ErrorCode::ConfigInvalid,
                format!("Bootnodes list is missing in {}", path.display()),
            )
        })?;

    let mut known_bootnodes = options.known_bootnodes.clone();
    if let Some(path) = options.known_bootnode_file.as_ref() {
        let extra = fs::read_to_string(path).map_err(|error| {
            AppError::with_source(
                ErrorCode::FilesystemIoFailed,
                format!("Failed to read known bootnode file: {}", path.display()),
                error,
            )
        })?;
        for line in extra.lines().map(str::trim).filter(|line| !line.is_empty()) {
            known_bootnodes.insert(line.to_string());
        }
    }

    Ok(items
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
            if options.strict_security && !is_strong_bootnode_id(&node_id) {
                return None;
            }
            if !known_bootnodes.is_empty() && !known_bootnodes.contains(&node_id) {
                return None;
            }
            if options.strict_bootnode_id
                && !node_id
                    .bytes()
                    .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
            {
                return None;
            }

            if options.deny_private_peers && is_private_or_loopback_peer(&address) {
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
        .collect())
}

fn derive_settings_genesis_fingerprint(settings: &Settings) -> String {
    let mut hasher = Sha256::new();
    hasher.update(settings.profile.as_bytes());
    hasher.update(settings.network.bind_host.as_bytes());
    hasher.update(settings.network.p2p_port.to_le_bytes());
    hasher.update(settings.network.rpc_port.to_le_bytes());
    hex::encode(hasher.finalize())
}

fn verify_sha256_hex(raw: &str, expected: &str, context: &str) -> Result<(), AppError> {
    if expected.len() != 64 || !expected.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(AppError::new(
            ErrorCode::UsageInvalidArguments,
            format!("Invalid {context} sha256 value: expected 64 hex characters"),
        ));
    }
    let digest = hex::encode(Sha256::digest(raw.as_bytes()));
    if digest != expected.to_ascii_lowercase() {
        return Err(AppError::new(
            ErrorCode::ConfigInvalid,
            format!("{context} hash mismatch: expected={expected} computed={digest}"),
        ));
    }
    Ok(())
}

fn collect_flag_values(args: &[String], flag: &str) -> Vec<String> {
    args.windows(2)
        .filter(|window| window[0] == flag)
        .map(|window| window[1].clone())
        .collect()
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

fn is_strong_bootnode_id(node_id: &str) -> bool {
    node_id.len() >= 16
        && node_id.bytes().any(|byte| byte.is_ascii_lowercase())
        && node_id.bytes().any(|byte| byte.is_ascii_digit())
        && node_id
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
}

fn is_private_or_loopback_peer(address: &str) -> bool {
    let host = address
        .rsplit_once(':')
        .map(|(host, _)| host.trim_matches(|c| c == '[' || c == ']'))
        .unwrap_or(address);

    if host.eq_ignore_ascii_case("localhost") {
        return true;
    }

    match host.parse::<IpAddr>() {
        Ok(IpAddr::V4(ip)) => ip.is_private() || ip.is_loopback() || ip.is_link_local(),
        Ok(IpAddr::V6(ip)) => {
            ip.is_loopback() || ip.is_unique_local() || ip.is_unicast_link_local()
        }
        Err(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_enables_strict_security_bundle() {
        let args = vec!["--strict-security".to_string()];
        let options = DiscoveryOptions::parse(&args).expect("strict parse should succeed");
        assert!(options.strict_security);
        assert!(options.require_official_peers);
        assert!(options.deny_private_peers);
        assert!(options.require_bootnodes_sha256);
    }

    #[test]
    fn private_or_loopback_peer_detection_handles_common_hosts() {
        assert!(is_private_or_loopback_peer("127.0.0.1:26656"));
        assert!(is_private_or_loopback_peer("10.0.0.15:26656"));
        assert!(is_private_or_loopback_peer("localhost:26656"));
        assert!(!is_private_or_loopback_peer("8.8.8.8:26656"));
    }

    #[test]
    fn strict_bootnode_rule_requires_entropy() {
        assert!(is_strong_bootnode_id("peeralpha9beta123"));
        assert!(!is_strong_bootnode_id("short-id1"));
        assert!(!is_strong_bootnode_id("onlylettersbootnode"));
    }
}
