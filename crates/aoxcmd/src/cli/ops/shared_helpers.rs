use super::*;

pub(super) fn effective_settings_for_ops() -> Result<Settings, AppError> {
    match load() {
        Ok(settings) => Ok(settings),
        Err(error) if error.code() == ErrorCode::ConfigMissing.as_str() => {
            let home = resolve_home()?;
            Ok(Settings::default_for(home.display().to_string()))
        }
        Err(error) => Err(error),
    }
}

pub(super) fn parse_positive_u64_arg(
    args: &[String],
    flag: &str,
    default: u64,
    context: &str,
) -> Result<u64, AppError> {
    let value = match arg_value(args, flag) {
        Some(value) => normalize_text(&value, false).ok_or_else(|| {
            AppError::new(
                ErrorCode::UsageInvalidArguments,
                format!("Flag {flag} must not be blank for {context}"),
            )
        })?,
        None => default.to_string(),
    };

    let parsed = value.parse::<u64>().map_err(|_| {
        AppError::new(
            ErrorCode::UsageInvalidArguments,
            format!("Invalid numeric value for {flag}"),
        )
    })?;

    if parsed == 0 {
        return Err(AppError::new(
            ErrorCode::UsageInvalidArguments,
            format!("Flag {flag} must be greater than zero"),
        ));
    }

    Ok(parsed)
}

pub(super) fn parse_positive_u64_value(
    value: &str,
    flag: &str,
    context: &str,
) -> Result<u64, AppError> {
    let normalized = normalize_text(value, false).ok_or_else(|| {
        AppError::new(
            ErrorCode::UsageInvalidArguments,
            format!("Flag {flag} must not be blank for {context}"),
        )
    })?;

    let parsed = normalized.parse::<u64>().map_err(|_| {
        AppError::new(
            ErrorCode::UsageInvalidArguments,
            format!("Invalid numeric value for {flag}"),
        )
    })?;

    if parsed == 0 {
        return Err(AppError::new(
            ErrorCode::UsageInvalidArguments,
            format!("Flag {flag} must be greater than zero"),
        ));
    }

    Ok(parsed)
}

pub(super) fn parse_required_or_default_text_arg(
    args: &[String],
    flag: &str,
    default: &str,
    lowercase: bool,
) -> Result<String, AppError> {
    match arg_value(args, flag) {
        Some(value) => normalize_text(&value, lowercase).ok_or_else(|| {
            AppError::new(
                ErrorCode::UsageInvalidArguments,
                format!("Flag {flag} must not be blank"),
            )
        }),
        None => Ok(default.to_string()),
    }
}

pub(super) fn parse_optional_text_arg(
    args: &[String],
    flag: &str,
    lowercase: bool,
) -> Option<String> {
    arg_value(args, flag).and_then(|value| normalize_text(&value, lowercase))
}

pub(super) fn normalize_text(value: &str, lowercase: bool) -> Option<String> {
    let normalized = value.split_whitespace().collect::<Vec<_>>().join(" ");
    if normalized.is_empty() {
        return None;
    }

    if lowercase {
        Some(normalized.to_ascii_lowercase())
    } else {
        Some(normalized)
    }
}

pub(super) fn derive_state_root(state: &crate::node::state::NodeState) -> Result<String, AppError> {
    let encoded = serde_json::to_vec(state).map_err(|error| {
        AppError::with_source(
            ErrorCode::OutputEncodingFailed,
            "Failed to serialize node state for state-root derivation",
            error,
        )
    })?;
    let mut hasher = Sha256::new();
    hasher.update(encoded);
    Ok(hex::encode(hasher.finalize()))
}

pub(super) fn runtime_db_root() -> Result<PathBuf, AppError> {
    Ok(resolve_home()?.join("runtime").join("db"))
}

pub(super) fn open_runtime_store() -> Result<HybridDataStore, AppError> {
    let db_root = runtime_db_root()?;
    HybridDataStore::new(&db_root, IndexBackend::Redb).map_err(|error| {
        AppError::with_source(
            ErrorCode::FilesystemIoFailed,
            format!("Failed to open runtime data store at {}", db_root.display()),
            error,
        )
    })
}

pub(super) fn load_historical_block(
    requested_height: &str,
    requested_hash: Option<&str>,
) -> Result<Option<BlockEnvelope>, AppError> {
    let store = open_runtime_store()?;
    let target_hash = requested_hash.and_then(|value| normalize_text(value, true));

    let requested = normalize_text(requested_height, true).unwrap_or_else(|| "latest".to_string());
    let target_height = if requested == "latest" {
        Some(lifecycle::load_state()?.current_height)
    } else {
        Some(requested.parse::<u64>().map_err(|_| {
            AppError::new(
                ErrorCode::UsageInvalidArguments,
                "Flag --height must be a positive integer or 'latest'",
            )
        })?)
    };

    let candidate = if let Some(hash) = target_hash.as_deref() {
        match store.get_block_by_hash(hash) {
            Ok(block) => Some(block),
            Err(DataError::NotFound) => None,
            Err(error) => {
                return Err(AppError::with_source(
                    ErrorCode::LedgerInvalid,
                    format!("Failed to load historical block by hash {hash}"),
                    error,
                ));
            }
        }
    } else if let Some(height) = target_height {
        match store.get_block_by_height(height) {
            Ok(block) => Some(block),
            Err(DataError::NotFound) => None,
            Err(error) => {
                return Err(AppError::with_source(
                    ErrorCode::LedgerInvalid,
                    format!("Failed to load historical block at height {height}"),
                    error,
                ));
            }
        }
    } else {
        None
    };

    Ok(candidate.filter(|block| {
        target_height
            .map(|height| block.height == height)
            .unwrap_or(true)
            && target_hash
                .as_ref()
                .map(|hash| block.block_hash_hex.eq_ignore_ascii_case(hash))
                .unwrap_or(true)
    }))
}

pub(super) fn historical_tx_hashes(envelope: &BlockEnvelope) -> Vec<String> {
    let payload = match serde_json::from_slice::<Value>(&envelope.payload) {
        Ok(value) => value,
        Err(_) => return Vec::new(),
    };

    payload
        .get("body")
        .and_then(|body| body.get("sections"))
        .and_then(Value::as_array)
        .map(|sections| {
            sections
                .iter()
                .filter_map(|section| section.get("payload").and_then(Value::as_str))
                .map(str::to_string)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
}

pub(super) fn tx_index_path() -> Result<PathBuf, AppError> {
    Ok(resolve_home()?.join("runtime").join(TX_INDEX_FILE))
}

pub(super) fn load_tx_index() -> Result<TxIndex, AppError> {
    let path = tx_index_path()?;
    if !path.exists() {
        return Ok(TxIndex {
            entries: BTreeMap::new(),
        });
    }

    let raw = fs::read_to_string(&path).map_err(|error| {
        AppError::with_source(
            ErrorCode::FilesystemIoFailed,
            format!("Failed to read transaction index from {}", path.display()),
            error,
        )
    })?;

    serde_json::from_str::<TxIndex>(&raw).map_err(|error| {
        AppError::with_source(
            ErrorCode::LedgerInvalid,
            format!("Failed to parse transaction index from {}", path.display()),
            error,
        )
    })
}

pub(super) fn load_tx_index_entry(tx_hash: &str) -> Result<Option<TxIndexEntry>, AppError> {
    let normalized_hash = tx_hash.to_ascii_lowercase();
    let tx_index = load_tx_index()?;
    Ok(tx_index
        .entries
        .get(&normalized_hash)
        .cloned()
        .or_else(|| tx_index.entries.get(tx_hash).cloned()))
}

pub(super) fn tx_hash_hex(tx_payload: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(tx_payload.as_bytes());
    hex::encode(hasher.finalize())
}

pub(super) fn load_state_root_for_height(height: u64) -> Result<Option<(u64, String)>, AppError> {
    let Some(block) = load_historical_block(&height.to_string(), None)? else {
        return Ok(None);
    };

    let payload = serde_json::from_slice::<Value>(&block.payload).ok();
    let indexed_root = payload
        .as_ref()
        .and_then(|value| value.get("state_root"))
        .and_then(Value::as_str)
        .or_else(|| {
            payload
                .as_ref()
                .and_then(|value| value.get("stateRoot"))
                .and_then(Value::as_str)
        })
        .or_else(|| {
            payload
                .as_ref()
                .and_then(|value| value.get("state_root_hex"))
                .and_then(Value::as_str)
        })
        .map(str::to_string)
        .unwrap_or_else(|| {
            let mut hasher = Sha256::new();
            hasher.update(&block.payload);
            hex::encode(hasher.finalize())
        });

    Ok(Some((block.height, indexed_root)))
}

pub(super) fn uptime_secs_from_rfc3339(value: &str) -> u64 {
    chrono::DateTime::parse_from_rfc3339(value)
        .ok()
        .and_then(|time| {
            let elapsed = Utc::now().signed_duration_since(time.with_timezone(&Utc));
            (elapsed.num_seconds() >= 0).then_some(elapsed.num_seconds() as u64)
        })
        .unwrap_or(0)
}

pub(super) fn rpc_listener_active(probe_target: &str) -> bool {
    match probe_target.parse() {
        Ok(addr) => TcpStream::connect_timeout(&addr, Duration::from_millis(250)).is_ok(),
        Err(_) => false,
    }
}

pub(super) fn rpc_http_get_probe(host: &str, port: u16, path: &str) -> bool {
    let request = format!(
        "GET {path} HTTP/1.1\r\nHost: {host}:{port}\r\nConnection: close\r\nAccept: application/json\r\n\r\n"
    );
    rpc_http_status_code(host, port, &request)
        .map(|code| (200..300).contains(&code))
        .unwrap_or(false)
}

pub(super) fn rpc_jsonrpc_status_probe(host: &str, port: u16) -> bool {
    let body = r#"{"jsonrpc":"2.0","id":1,"method":"status","params":[]}"#;
    let request = format!(
        "POST / HTTP/1.1\r\nHost: {host}:{port}\r\nConnection: close\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{body}",
        body.len()
    );
    rpc_http_status_code(host, port, &request)
        .map(|code| (200..300).contains(&code))
        .unwrap_or(false)
}

pub(super) fn rpc_http_status_code(host: &str, port: u16, request: &str) -> Option<u16> {
    let target = format!("{host}:{port}");
    let addr = target.to_socket_addrs().ok()?.next()?;
    let mut stream = TcpStream::connect_timeout(&addr, Duration::from_millis(350)).ok()?;
    if stream
        .set_read_timeout(Some(Duration::from_millis(350)))
        .is_err()
    {
        return None;
    }
    if stream
        .set_write_timeout(Some(Duration::from_millis(350)))
        .is_err()
    {
        return None;
    }
    if stream.write_all(request.as_bytes()).is_err() {
        return None;
    }
    let mut reader = BufReader::new(stream);
    let mut status_line = String::new();
    if reader.read_line(&mut status_line).ok()? == 0 {
        return None;
    }
    let mut parts = status_line.split_whitespace();
    let _http_version = parts.next()?;
    parts.next()?.parse::<u16>().ok()
}

pub(super) fn faucet_state_path() -> Result<PathBuf, AppError> {
    Ok(resolve_home()?.join("ledger").join("faucet_state.json"))
}

pub(super) fn load_faucet_state() -> Result<FaucetState, AppError> {
    let path = faucet_state_path()?;
    if !path.exists() {
        let state = FaucetState::default();
        persist_faucet_state(&state)?;
        return Ok(state);
    }

    let raw = fs::read_to_string(&path).map_err(|error| {
        AppError::with_source(
            ErrorCode::FilesystemIoFailed,
            format!("Failed to read faucet state from {}", path.display()),
            error,
        )
    })?;

    serde_json::from_str::<FaucetState>(&raw).map_err(|error| {
        AppError::with_source(
            ErrorCode::LedgerInvalid,
            format!("Failed to parse faucet state from {}", path.display()),
            error,
        )
    })
}

pub(super) fn persist_faucet_state(state: &FaucetState) -> Result<(), AppError> {
    let path = faucet_state_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| {
            AppError::with_source(
                ErrorCode::FilesystemIoFailed,
                format!(
                    "Failed to create faucet state directory {}",
                    parent.display()
                ),
                error,
            )
        })?;
    }

    let payload = serde_json::to_string_pretty(state).map_err(|error| {
        AppError::with_source(
            ErrorCode::OutputEncodingFailed,
            "Failed to encode faucet state",
            error,
        )
    })?;

    fs::write(&path, payload).map_err(|error| {
        AppError::with_source(
            ErrorCode::FilesystemIoFailed,
            format!("Failed to write faucet state to {}", path.display()),
            error,
        )
    })?;

    Ok(())
}

pub(super) fn evaluate_faucet_claim(
    state: &FaucetState,
    account_id: &str,
    amount: u64,
    now_unix: u64,
    force: bool,
    treasury_balance: Option<u64>,
    network_kind: &str,
) -> FaucetClaimDecision {
    if network_kind == "mainnet" {
        return FaucetClaimDecision {
            allowed: false,
            cooldown_remaining_secs: 0,
            claimed_last_24h: 0,
            daily_remaining: state.daily_limit_per_account,
            global_distributed_last_24h: 0,
            global_remaining: state.daily_global_limit,
            next_eligible_claim_at: None,
            denied_reason: Some("Mainnet profile does not allow faucet claims".to_string()),
        };
    }

    let day_ago = now_unix.saturating_sub(24 * 60 * 60);
    let relevant_claims: Vec<&FaucetClaimRecord> = state
        .claims
        .iter()
        .filter(|claim| claim.account_id == account_id)
        .collect();
    let global_recent_claims: Vec<&FaucetClaimRecord> = state
        .claims
        .iter()
        .filter(|claim| claim.claimed_at >= day_ago)
        .collect();

    let claimed_last_24h = relevant_claims
        .iter()
        .filter(|claim| claim.claimed_at >= day_ago)
        .map(|claim| claim.amount)
        .sum::<u64>();

    let global_distributed_last_24h = global_recent_claims.iter().map(|claim| claim.amount).sum();

    let latest_claim = relevant_claims
        .iter()
        .max_by_key(|claim| claim.claimed_at)
        .copied();

    let cooldown_remaining_secs = latest_claim
        .map(|claim| {
            let unlock_at = claim.claimed_at.saturating_add(state.cooldown_secs);
            unlock_at.saturating_sub(now_unix)
        })
        .unwrap_or(0);

    let daily_remaining = state
        .daily_limit_per_account
        .saturating_sub(claimed_last_24h);
    let global_remaining = state
        .daily_global_limit
        .saturating_sub(global_distributed_last_24h);
    let next_eligible_claim_at = if cooldown_remaining_secs > 0 {
        Some(now_unix.saturating_add(cooldown_remaining_secs))
    } else {
        None
    };

    if force {
        return FaucetClaimDecision {
            allowed: true,
            cooldown_remaining_secs,
            claimed_last_24h,
            daily_remaining,
            global_distributed_last_24h,
            global_remaining,
            next_eligible_claim_at,
            denied_reason: None,
        };
    }

    if state
        .banned_accounts
        .iter()
        .any(|entry| entry == account_id)
    {
        return FaucetClaimDecision {
            allowed: false,
            cooldown_remaining_secs,
            claimed_last_24h,
            daily_remaining,
            global_distributed_last_24h,
            global_remaining,
            next_eligible_claim_at,
            denied_reason: Some("Account is banned from faucet".to_string()),
        };
    }

    if !state.allowlisted_accounts.is_empty()
        && !state
            .allowlisted_accounts
            .iter()
            .any(|entry| entry == account_id)
    {
        return FaucetClaimDecision {
            allowed: false,
            cooldown_remaining_secs,
            claimed_last_24h,
            daily_remaining,
            global_distributed_last_24h,
            global_remaining,
            next_eligible_claim_at,
            denied_reason: Some("Account is not in faucet allowlist".to_string()),
        };
    }

    if amount > state.max_claim_amount {
        return FaucetClaimDecision {
            allowed: false,
            cooldown_remaining_secs,
            claimed_last_24h,
            daily_remaining,
            global_distributed_last_24h,
            global_remaining,
            next_eligible_claim_at,
            denied_reason: Some(format!(
                "Requested amount exceeds max claim amount (max={})",
                state.max_claim_amount
            )),
        };
    }

    if cooldown_remaining_secs > 0 {
        return FaucetClaimDecision {
            allowed: false,
            cooldown_remaining_secs,
            claimed_last_24h,
            daily_remaining,
            global_distributed_last_24h,
            global_remaining,
            next_eligible_claim_at,
            denied_reason: Some(format!(
                "Cooldown is active; try again in {} seconds",
                cooldown_remaining_secs
            )),
        };
    }

    if claimed_last_24h.saturating_add(amount) > state.daily_limit_per_account {
        return FaucetClaimDecision {
            allowed: false,
            cooldown_remaining_secs,
            claimed_last_24h,
            daily_remaining,
            global_distributed_last_24h,
            global_remaining,
            next_eligible_claim_at,
            denied_reason: Some(format!(
                "Daily limit exceeded for account (limit={})",
                state.daily_limit_per_account
            )),
        };
    }

    if global_distributed_last_24h.saturating_add(amount) > state.daily_global_limit {
        return FaucetClaimDecision {
            allowed: false,
            cooldown_remaining_secs,
            claimed_last_24h,
            daily_remaining,
            global_distributed_last_24h,
            global_remaining,
            next_eligible_claim_at,
            denied_reason: Some(format!(
                "Daily global faucet limit exceeded (limit={})",
                state.daily_global_limit
            )),
        };
    }

    if let Some(balance) = treasury_balance
        && balance.saturating_sub(amount) < state.min_reserve_balance
    {
        return FaucetClaimDecision {
            allowed: false,
            cooldown_remaining_secs,
            claimed_last_24h,
            daily_remaining,
            global_distributed_last_24h,
            global_remaining,
            next_eligible_claim_at,
            denied_reason: Some(format!(
                "Reserve floor check failed (min_reserve_balance={})",
                state.min_reserve_balance
            )),
        };
    }

    FaucetClaimDecision {
        allowed: true,
        cooldown_remaining_secs,
        claimed_last_24h,
        daily_remaining,
        global_distributed_last_24h,
        global_remaining,
        next_eligible_claim_at,
        denied_reason: None,
    }
}

pub(super) fn prune_faucet_history(state: &mut FaucetState, now_unix: u64) {
    let retention = ChronoDuration::hours(48).num_seconds().unsigned_abs();
    let oldest = now_unix.saturating_sub(retention);
    state.claims.retain(|claim| claim.claimed_at >= oldest);
    let audit_retention = ChronoDuration::hours(FAUCET_AUDIT_RETENTION_HOURS)
        .num_seconds()
        .unsigned_abs();
    let audit_oldest = now_unix.saturating_sub(audit_retention);
    state
        .audit_log
        .retain(|entry| entry.at_unix >= audit_oldest);
}

pub(super) fn now_unix_secs() -> Result<u64, AppError> {
    let now = Utc::now().timestamp();
    u64::try_from(now).map_err(|_| {
        AppError::new(
            ErrorCode::NodeStateInvalid,
            "System clock produced a negative unix timestamp",
        )
    })
}

pub(super) fn faucet_tx_id(account_id: &str, amount: u64, now_unix: u64, nonce: usize) -> String {
    let mut hasher = Sha256::new();
    hasher.update(account_id.as_bytes());
    hasher.update(amount.to_le_bytes());
    hasher.update(now_unix.to_le_bytes());
    hasher.update(nonce.to_le_bytes());
    format!("faucet-{}", hex::encode(hasher.finalize()))
}

pub(super) fn append_faucet_audit(
    state: &mut FaucetState,
    action: &str,
    actor: &str,
    detail: &str,
    now_unix: u64,
) {
    state.audit_log.push(FaucetAuditRecord {
        at_unix: now_unix,
        action: action.to_string(),
        actor: actor.to_string(),
        detail: detail.to_string(),
    });
}
