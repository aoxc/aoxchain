use super::*;

pub fn cmd_peer_list(args: &[String]) -> Result<(), AppError> {
    let settings = effective_settings_for_ops()?;
    let response = build_network_snapshot(&settings);

    emit_serialized(&response, output_format(args))
}

pub fn cmd_network_status(args: &[String]) -> Result<(), AppError> {
    let settings = effective_settings_for_ops()?;
    let snapshot = build_network_snapshot(&settings);
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
    let response = build_network_snapshot(&settings);

    emit_serialized(&response, output_format(args))
}

#[derive(serde::Serialize)]
struct PeerView {
    peer_id: String,
    address: String,
    direction: &'static str,
    connected_since: String,
    sync_state: &'static str,
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

fn build_network_snapshot(settings: &Settings) -> NetworkSnapshot {
    let now = Utc::now().to_rfc3339();
    let peers = vec![PeerView {
        peer_id: "self".to_string(),
        address: format!(
            "{}:{}",
            settings.network.bind_host, settings.network.p2p_port
        ),
        direction: "inbound+outbound",
        connected_since: now,
        sync_state: "in-sync",
    }];

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
        peers,
    }
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
