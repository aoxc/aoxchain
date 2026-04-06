use super::*;

pub fn cmd_query_full(args: &[String]) -> Result<(), AppError> {
    #[derive(serde::Serialize)]
    struct QueryFullResponse {
        release_line: &'static str,
        generated_at: String,
        request: QueryFullRequest,
        chain: QueryChainSurface,
        consensus: QueryConsensusSurface,
        vm: QueryVmSurface,
        network: QueryNetworkSurface,
        rpc: QueryRpcSurface,
        account: QueryAccountSurface,
        tx: QueryTxSurface,
    }

    #[derive(serde::Serialize)]
    struct QueryFullRequest {
        account_id: String,
        tx_hash: String,
    }

    #[derive(serde::Serialize)]
    struct QueryChainSurface {
        network_id: u32,
        current_height: u64,
        latest_block_hash: String,
        latest_parent_hash: String,
        latest_timestamp_unix: u64,
        produced_blocks: u64,
        running: bool,
    }

    #[derive(serde::Serialize)]
    struct QueryConsensusSurface {
        last_round: u64,
        last_message_kind: String,
        latest_proposer: String,
        validator_set_hash: String,
        finalized_height: u64,
        quorum_status: &'static str,
    }

    #[derive(serde::Serialize)]
    struct QueryVmSurface {
        enabled: bool,
        latest_height: u64,
        latest_tx_marker: String,
        last_execution_status: &'static str,
        state_root: String,
    }

    #[derive(serde::Serialize)]
    struct QueryNetworkSurface {
        bind_host: String,
        p2p_port: u16,
        rpc_port: u16,
        probe_target: String,
        listener_active: bool,
    }

    #[derive(serde::Serialize)]
    struct QueryRpcSurface {
        http_base_url: String,
        required_endpoint_ready: bool,
        jsonrpc_status_probe: bool,
        required_endpoint_probes: BTreeMap<&'static str, bool>,
    }

    #[derive(serde::Serialize)]
    struct QueryAccountSurface {
        account_id: String,
        known: bool,
        balance: u64,
        source: &'static str,
    }

    #[derive(serde::Serialize)]
    struct QueryTxSurface {
        tx_hash: String,
        known: bool,
        block_height: u64,
        execution_status: &'static str,
    }

    let settings = effective_settings_for_ops()?;
    let state =
        lifecycle::load_state().unwrap_or_else(|_| crate::node::state::NodeState::bootstrap());
    let ledger = ledger::load().unwrap_or_default();
    let state_root = derive_state_root(&state)?;

    let account_id = parse_required_or_default_text_arg(args, "--account-id", "treasury", false)?;
    let tx_hash = parse_required_or_default_text_arg(args, "--tx-hash", &state.last_tx, false)?;

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
    let mut required_endpoint_probes = BTreeMap::new();
    for path in [
        "/health",
        "/status",
        "/chain/status",
        "/consensus/status",
        "/vm/status",
    ] {
        required_endpoint_probes.insert(
            path,
            listener_active && rpc_http_get_probe(&curl_host, settings.network.rpc_port, path),
        );
    }

    let (account_known, account_balance) = if account_id == "treasury" {
        (true, ledger.treasury_balance)
    } else {
        (
            ledger.delegations.contains_key(&account_id),
            ledger.delegations.get(&account_id).copied().unwrap_or(0),
        )
    };

    let indexed_tx = load_tx_index_entry(&tx_hash)?;
    let tx_known = indexed_tx.is_some()
        || (state.last_tx != "none" && tx_hash == state.last_tx)
        || (state.last_tx != "none" && tx_hash == tx_hash_hex(&state.last_tx));

    let response = QueryFullResponse {
        release_line: AOXC_Q_RELEASE_LINE,
        generated_at: Utc::now().to_rfc3339(),
        request: QueryFullRequest {
            account_id: account_id.clone(),
            tx_hash: tx_hash.clone(),
        },
        chain: QueryChainSurface {
            network_id: state.consensus.network_id,
            current_height: state.current_height,
            latest_block_hash: state.consensus.last_block_hash_hex.clone(),
            latest_parent_hash: state.consensus.last_parent_hash_hex.clone(),
            latest_timestamp_unix: state.consensus.last_timestamp_unix,
            produced_blocks: state.produced_blocks,
            running: state.running,
        },
        consensus: QueryConsensusSurface {
            last_round: state.consensus.last_round,
            last_message_kind: state.consensus.last_message_kind.clone(),
            latest_proposer: state.consensus.last_proposer_hex.clone(),
            validator_set_hash: state.key_material.bundle_fingerprint.clone(),
            finalized_height: state.current_height,
            quorum_status: if state.running {
                "single-node-ok"
            } else {
                "idle"
            },
        },
        vm: QueryVmSurface {
            enabled: true,
            latest_height: state.current_height,
            latest_tx_marker: state.last_tx.clone(),
            last_execution_status: if state.last_tx != "none" {
                "ok"
            } else {
                "idle"
            },
            state_root,
        },
        network: QueryNetworkSurface {
            bind_host: settings.network.bind_host.clone(),
            p2p_port: settings.network.p2p_port,
            rpc_port: settings.network.rpc_port,
            probe_target,
            listener_active,
        },
        rpc: QueryRpcSurface {
            http_base_url: format!("http://{}:{}", curl_host, settings.network.rpc_port),
            required_endpoint_ready: required_endpoint_probes.values().all(|is_ready| *is_ready),
            jsonrpc_status_probe: listener_active
                && rpc_jsonrpc_status_probe(&curl_host, settings.network.rpc_port),
            required_endpoint_probes,
        },
        account: QueryAccountSurface {
            account_id,
            known: account_known,
            balance: account_balance,
            source: "local-ledger",
        },
        tx: QueryTxSurface {
            tx_hash,
            known: tx_known,
            block_height: indexed_tx
                .as_ref()
                .map_or(state.current_height, |entry| entry.block_height),
            execution_status: if indexed_tx.is_some() || tx_known {
                "applied"
            } else {
                "unknown"
            },
        },
    };

    emit_serialized(&response, output_format(args))
}
