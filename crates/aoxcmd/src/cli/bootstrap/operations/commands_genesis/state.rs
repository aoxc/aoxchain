use super::*;
use std::path::{Path, PathBuf};

const ALLOWED_GENESIS_ACCOUNT_ROLES: [&str; 12] = [
    "treasury",
    "validator",
    "system",
    "user",
    "governance",
    "forge",
    "quorum",
    "seal",
    "archive",
    "sentinel",
    "relay",
    "pocket",
];

fn is_allowed_genesis_account_role(role: &str) -> bool {
    ALLOWED_GENESIS_ACCOUNT_ROLES.contains(&role)
}

pub fn cmd_genesis_start(args: &[String]) -> Result<(), AppError> {
    let profile_input = arg_value(args, "--profile")
        .or_else(|| load().ok().map(|settings| settings.profile))
        .unwrap_or_else(|| "validation".to_string());
    let profile = EnvironmentProfile::parse(&profile_input)?;

    let mut init_args = vec![
        "--profile".to_string(),
        profile.as_str().to_string(),
        "--format".to_string(),
        "json".to_string(),
    ];
    if has_flag(args, "--home")
        && let Some(home) = arg_value(args, "--home")
    {
        init_args.push("--home".to_string());
        init_args.push(home);
    }

    if !genesis_path()?.exists() && !has_flag(args, "--no-init") {
        cmd_genesis_init(&init_args)?;
    }

    let mut genesis = load_genesis()?;
    if genesis.environment != profile.as_str() {
        return Err(AppError::new(
            ErrorCode::UsageInvalidArguments,
            format!(
                "Profile mismatch: genesis environment '{}' does not match --profile '{}'",
                genesis.environment,
                profile.as_str()
            ),
        ));
    }

    if has_flag(args, "--bootstrap-key-if-missing") {
        let operator_name =
            parse_required_or_default_text_arg(args, "--operator-name", "validator-01")?;
        let password = parse_required_text_arg(
            args,
            "--password",
            false,
            "genesis start with --bootstrap-key-if-missing",
        )?;
        if inspect_operator_key().is_err() {
            bootstrap_operator_key(&operator_name, profile.as_str(), &password)?;
        }
    }

    let override_count = apply_genesis_start_overrides(args, &mut genesis)?;
    persist_genesis(&genesis)?;

    validate_genesis(&genesis)?;
    validate_binding_files(&genesis)?;
    enforce_genesis_start_security(args, &genesis)?;
    if has_flag(args, "--strict") {
        validate_identity_against_repo_policy(&genesis)?;
    }
    if has_flag(args, "--production-gate") {
        let status = role_topology_core7_status(&genesis.environment)?;
        if !status.missing.is_empty() || !status.non_core_active.is_empty() {
            return Err(AppError::new(
                ErrorCode::ConfigInvalid,
                "genesis start production gate failed: use `aoxc genesis production-gate` for details",
            ));
        }
    }

    if has_flag(args, "--dry-run") || has_flag(args, "--skip-node-run") {
        let mut details = BTreeMap::new();
        details.insert("action".to_string(), "genesis-start".to_string());
        details.insert("result".to_string(), "validated".to_string());
        details.insert("profile".to_string(), genesis.environment.clone());
        details.insert(
            "chain_id".to_string(),
            genesis.identity.chain_id.to_string(),
        );
        details.insert(
            "network_id".to_string(),
            genesis.identity.network_id.clone(),
        );
        details.insert("overrides_applied".to_string(), override_count.to_string());
        details.insert("node_run".to_string(), "skipped".to_string());
        return emit_serialized(
            &text_envelope("genesis-start", "ok", details),
            output_format(args),
        );
    }

    let mut node_args = Vec::<String>::new();
    forward_flag_with_value(args, "--rounds", &mut node_args);
    forward_flag_with_value(args, "--tx-prefix", &mut node_args);
    forward_flag_with_value(args, "--interval-secs", &mut node_args);
    forward_flag_with_value(args, "--log-level", &mut node_args);
    forward_boolean_flag(args, "--continuous", &mut node_args);
    forward_boolean_flag(args, "--bounded", &mut node_args);
    forward_boolean_flag(args, "--no-live-log", &mut node_args);
    forward_boolean_flag(args, "--no-rpc-serve", &mut node_args);
    forward_flag_with_value(args, "--known-bootnode", &mut node_args);
    forward_flag_with_value(args, "--known-bootnode-file", &mut node_args);
    forward_flag_with_value(args, "--bootnodes-file", &mut node_args);
    forward_flag_with_value(args, "--bootnodes-sha256", &mut node_args);
    forward_flag_with_value(args, "--certificate-file", &mut node_args);
    forward_flag_with_value(args, "--certificate-sha256", &mut node_args);
    forward_boolean_flag(args, "--strict-bootnode-id", &mut node_args);
    forward_flag_with_value(args, "--format", &mut node_args);

    crate::cli::ops::cmd_node_run(&node_args)
}

pub fn cmd_genesis_init(args: &[String]) -> Result<(), AppError> {
    let profile_input = arg_value(args, "--profile")
        .or_else(|| load().ok().map(|settings| settings.profile))
        .unwrap_or_else(|| "validation".to_string());

    let profile = EnvironmentProfile::parse(&profile_input)?;
    let settings = load()
        .ok()
        .filter(|settings| settings.profile == profile.as_str())
        .unwrap_or(build_profile_settings(
            resolve_home()?.display().to_string(),
            profile,
            None,
        )?);

    let operator = inspect_operator_key().map_err(|_| {
        AppError::new(
            ErrorCode::UsageInvalidArguments,
            "Genesis init requires operator key material. Run `aoxc key-bootstrap --profile <profile> --password <value>` first.",
        )
    })?;

    let mut genesis = profile.genesis_document();
    upsert_validator_account(&mut genesis, &operator, DEFAULT_VALIDATOR_GENESIS_BALANCE)?;
    materialize_binding_documents(&genesis, &operator, &settings)?;

    let content = serde_json::to_string_pretty(&genesis).map_err(|error| {
        AppError::with_source(
            ErrorCode::OutputEncodingFailed,
            "Failed to encode AOXC genesis document",
            error,
        )
    })?;

    write_file(&genesis_path()?, &content)?;
    emit_serialized(&genesis, output_format(args))
}

pub(in crate::cli::bootstrap::operations) fn apply_genesis_start_overrides(
    args: &[String],
    genesis: &mut BootstrapGenesisDocument,
) -> Result<usize, AppError> {
    let mut applied = 0usize;

    if let Some(value) = parse_optional_text_arg(args, "--family-name", false) {
        genesis.family_name = value;
        applied += 1;
    }
    if let Some(value) = parse_optional_text_arg(args, "--family-code", false) {
        genesis.family_code = value;
        applied += 1;
    }
    if let Some(value) = parse_optional_text_arg(args, "--chain-name", false) {
        genesis.identity.chain_name = value;
        applied += 1;
    }
    if let Some(value) = parse_optional_text_arg(args, "--network-class", true) {
        genesis.identity.network_class = value;
        applied += 1;
    }
    if let Some(value) = parse_optional_text_arg(args, "--network-serial", false) {
        genesis.identity.network_serial = value;
        applied += 1;
    }
    if let Some(value) = parse_optional_text_arg(args, "--network-id", false) {
        genesis.identity.network_id = value;
        applied += 1;
    }
    if let Some(value) = parse_optional_text_arg(args, "--validator-quorum-policy", false) {
        genesis.consensus.validator_quorum_policy = value;
        applied += 1;
    }
    if let Some(value) = parse_optional_text_arg(args, "--consensus-identity-profile", true) {
        genesis.consensus.consensus_identity_profile = value;
        applied += 1;
    }
    if let Some(value) = parse_optional_text_arg(args, "--native-symbol", false) {
        genesis.economics.native_symbol = value;
        applied += 1;
    }
    if let Some(value) = parse_optional_text_arg(args, "--treasury-account-id", false) {
        genesis.economics.initial_treasury.account_id = value;
        applied += 1;
    }
    if let Some(value) = parse_optional_text_arg(args, "--treasury-amount", false) {
        if !is_non_zero_decimal_string(&value) {
            return Err(AppError::new(
                ErrorCode::UsageInvalidArguments,
                "Flag --treasury-amount must be a non-zero decimal string",
            ));
        }
        genesis.economics.initial_treasury.amount = value;
        applied += 1;
    }

    applied += apply_u32_override(args, "--family-id", &mut genesis.identity.family_id)?;
    applied += apply_u64_override(args, "--chain-id", &mut genesis.identity.chain_id)?;
    applied += apply_u64_override(
        args,
        "--genesis-epoch",
        &mut genesis.consensus.genesis_epoch,
    )?;
    applied += apply_u64_override(
        args,
        "--block-time-ms",
        &mut genesis.consensus.block_time_ms,
    )?;
    applied += apply_u64_override(
        args,
        "--epoch-length-blocks",
        &mut genesis.consensus.consensus_timing.epoch_length_blocks,
    )?;
    applied += apply_u64_override(
        args,
        "--pacemaker-base-timeout-ms",
        &mut genesis.consensus.consensus_timing.pacemaker_base_timeout_ms,
    )?;
    applied += apply_u64_override(
        args,
        "--pacemaker-max-timeout-ms",
        &mut genesis.consensus.consensus_timing.pacemaker_max_timeout_ms,
    )?;
    applied += apply_u64_override(
        args,
        "--reconfiguration-finality-lag-blocks",
        &mut genesis
            .consensus
            .consensus_timing
            .reconfiguration_finality_lag_blocks,
    )?;
    applied += apply_u8_override(
        args,
        "--native-decimals",
        &mut genesis.economics.native_decimals,
    )?;

    Ok(applied)
}

fn apply_u64_override(args: &[String], flag: &str, target: &mut u64) -> Result<usize, AppError> {
    let Some(raw) = arg_value(args, flag) else {
        return Ok(0);
    };
    let parsed = raw.parse::<u64>().map_err(|_| {
        AppError::new(
            ErrorCode::UsageInvalidArguments,
            format!("Flag {flag} must be an unsigned integer"),
        )
    })?;
    *target = parsed;
    Ok(1)
}

fn apply_u32_override(args: &[String], flag: &str, target: &mut u32) -> Result<usize, AppError> {
    let Some(raw) = arg_value(args, flag) else {
        return Ok(0);
    };
    let parsed = raw.parse::<u32>().map_err(|_| {
        AppError::new(
            ErrorCode::UsageInvalidArguments,
            format!("Flag {flag} must be an unsigned integer"),
        )
    })?;
    *target = parsed;
    Ok(1)
}

fn apply_u8_override(args: &[String], flag: &str, target: &mut u8) -> Result<usize, AppError> {
    let Some(raw) = arg_value(args, flag) else {
        return Ok(0);
    };
    let parsed = raw.parse::<u8>().map_err(|_| {
        AppError::new(
            ErrorCode::UsageInvalidArguments,
            format!("Flag {flag} must be an unsigned integer in u8 range"),
        )
    })?;
    *target = parsed;
    Ok(1)
}

fn forward_flag_with_value(args: &[String], flag: &str, node_args: &mut Vec<String>) {
    if let Some(value) = arg_value(args, flag) {
        node_args.push(flag.to_string());
        node_args.push(value);
    }
}

fn forward_boolean_flag(args: &[String], flag: &str, node_args: &mut Vec<String>) {
    if has_flag(args, flag) {
        node_args.push(flag.to_string());
    }
}

fn enforce_genesis_start_security(
    args: &[String],
    genesis: &BootstrapGenesisDocument,
) -> Result<(), AppError> {
    if has_flag(args, "--enforce-pq-consensus")
        && !genesis
            .consensus
            .consensus_identity_profile
            .to_ascii_lowercase()
            .starts_with("pq")
    {
        return Err(AppError::new(
            ErrorCode::ConfigInvalid,
            "Genesis start security gate failed: --enforce-pq-consensus requires a pq* consensus identity profile",
        ));
    }

    if has_flag(args, "--enforce-block-validation-rules") {
        let timing = &genesis.consensus.consensus_timing;
        let block_time_ok = (500..=15_000).contains(&genesis.consensus.block_time_ms);
        let base_ok = timing.pacemaker_base_timeout_ms > 0
            && timing.pacemaker_base_timeout_ms <= genesis.consensus.block_time_ms;
        let max_ok = timing.pacemaker_max_timeout_ms >= timing.pacemaker_base_timeout_ms
            && timing.pacemaker_max_timeout_ms <= 120_000;
        let finality_ok = timing.reconfiguration_finality_lag_blocks >= 2
            && timing.reconfiguration_finality_lag_blocks <= 256;
        if !(block_time_ok && base_ok && max_ok && finality_ok) {
            return Err(AppError::new(
                ErrorCode::ConfigInvalid,
                format!(
                    "Genesis start security gate failed: timing rules violated (block_time_ms={}, base_timeout_ms={}, max_timeout_ms={}, finality_lag_blocks={})",
                    genesis.consensus.block_time_ms,
                    timing.pacemaker_base_timeout_ms,
                    timing.pacemaker_max_timeout_ms,
                    timing.reconfiguration_finality_lag_blocks
                ),
            ));
        }
    }

    if let Some(expected) = parse_optional_text_arg(args, "--expected-genesis-sha256", true) {
        verify_file_sha256(
            &genesis_path()?,
            &expected,
            "Genesis start security gate failed: genesis hash mismatch",
        )?;
    }

    let genesis_file = genesis_path()?;
    let root = genesis_file.parent().ok_or_else(|| {
        AppError::new(
            ErrorCode::FilesystemIoFailed,
            "Genesis start security gate failed: identity directory not accessible",
        )
    })?;

    if let Some(expected) = parse_optional_text_arg(args, "--expected-validators-sha256", true) {
        verify_file_sha256(
            &root.join(&genesis.bindings.validators_file),
            &expected,
            "Genesis start security gate failed: validators binding hash mismatch",
        )?;
    }
    if let Some(expected) = parse_optional_text_arg(args, "--expected-bootnodes-sha256", true) {
        verify_file_sha256(
            &root.join(&genesis.bindings.bootnodes_file),
            &expected,
            "Genesis start security gate failed: bootnodes binding hash mismatch",
        )?;
    }
    if let Some(expected) = parse_optional_text_arg(args, "--expected-certificate-sha256", true) {
        verify_file_sha256(
            &root.join(&genesis.bindings.certificate_file),
            &expected,
            "Genesis start security gate failed: certificate binding hash mismatch",
        )?;
    }

    Ok(())
}

fn verify_file_sha256(path: &Path, expected_hex: &str, message: &str) -> Result<(), AppError> {
    if expected_hex.len() != 64 || !expected_hex.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(AppError::new(
            ErrorCode::UsageInvalidArguments,
            "Expected sha256 digest must be a 64-byte hexadecimal string",
        ));
    }

    let raw = read_file(path)?;
    let mut hasher = Sha256::new();
    hasher.update(raw.as_bytes());
    let digest = hex::encode(hasher.finalize());
    if digest != expected_hex.to_ascii_lowercase() {
        return Err(AppError::new(
            ErrorCode::ConfigInvalid,
            format!(
                "{message}: path={} expected={expected_hex} computed={digest}",
                path.display()
            ),
        ));
    }

    Ok(())
}

pub fn cmd_genesis_add_account(args: &[String]) -> Result<(), AppError> {
    let account_id = parse_required_text_arg(args, "--account-id", false, "genesis add account")?;
    let balance = parse_required_text_arg(args, "--balance", false, "genesis add account")?;
    let role = parse_required_or_default_text_arg(args, "--role", "user")?;
    let normalized_role = role.to_ascii_lowercase();

    if !is_allowed_genesis_account_role(&normalized_role) {
        return Err(AppError::new(
            ErrorCode::UsageInvalidArguments,
            format!(
                "Unsupported genesis account role '{}'. Allowed roles: {}",
                role,
                ALLOWED_GENESIS_ACCOUNT_ROLES.join(", ")
            ),
        ));
    }

    if !is_decimal_string(&balance) {
        return Err(AppError::new(
            ErrorCode::UsageInvalidArguments,
            "Genesis account balance must be a decimal string",
        ));
    }

    let mut genesis = load_genesis()?;
    if let Some(existing) = genesis
        .state
        .accounts
        .iter_mut()
        .find(|entry| entry.account_id == account_id)
    {
        existing.balance = balance.clone();
        existing.role = normalized_role.clone();
    } else {
        genesis.state.accounts.push(BootstrapAccountRecord {
            account_id: account_id.clone(),
            balance: balance.clone(),
            role: normalized_role.clone(),
        });
    }

    persist_genesis(&genesis)?;
    sync_optional_accounts_binding(&genesis)?;

    let mut details = BTreeMap::new();
    details.insert("account_id".to_string(), account_id);
    details.insert("balance".to_string(), balance);
    details.insert("role".to_string(), normalized_role);
    details.insert(
        "accounts_total".to_string(),
        genesis.state.accounts.len().to_string(),
    );

    emit_serialized(
        &text_envelope("genesis-add-account", "ok", details),
        output_format(args),
    )
}

pub fn cmd_genesis_add_validator(args: &[String]) -> Result<(), AppError> {
    let validator_id =
        parse_required_text_arg(args, "--validator-id", false, "genesis add validator")?;
    let consensus_public_key = parse_required_text_arg(
        args,
        "--consensus-public-key",
        false,
        "genesis add validator",
    )?;
    let network_public_key =
        parse_required_text_arg(args, "--network-public-key", false, "genesis add validator")?;
    let balance =
        parse_required_or_default_text_arg(args, "--balance", DEFAULT_VALIDATOR_GENESIS_BALANCE)?;
    let display_name = parse_required_or_default_text_arg(args, "--display-name", &validator_id)?;
    let consensus_fingerprint = parse_required_or_default_text_arg(
        args,
        "--consensus-fingerprint",
        &derive_short_fingerprint(&consensus_public_key),
    )?;
    let network_fingerprint = parse_required_or_default_text_arg(
        args,
        "--network-fingerprint",
        &derive_short_fingerprint(&network_public_key),
    )?;

    if !is_decimal_string(&balance) {
        return Err(AppError::new(
            ErrorCode::UsageInvalidArguments,
            "Validator balance must be a decimal string",
        ));
    }

    let mut genesis = load_genesis()?;
    let settings = load()
        .ok()
        .filter(|settings| settings.profile == genesis.environment)
        .unwrap_or(build_profile_settings(
            resolve_home()?.display().to_string(),
            EnvironmentProfile::parse(&genesis.environment)?,
            None,
        )?);

    let bootnode_address = parse_required_or_default_text_arg(
        args,
        "--bootnode-address",
        &format!(
            "{}:{}",
            settings.network.bind_host, settings.network.p2p_port
        ),
    )?;

    let validator_account_id = format!(
        "AOXC_VALIDATOR_{}",
        derive_short_fingerprint(&validator_id).to_ascii_uppercase()
    );

    if let Some(existing) = genesis
        .state
        .accounts
        .iter_mut()
        .find(|entry| entry.account_id == validator_account_id)
    {
        existing.balance = balance.clone();
        existing.role = "validator".to_string();
    } else {
        genesis.state.accounts.push(BootstrapAccountRecord {
            account_id: validator_account_id.clone(),
            balance: balance.clone(),
            role: "validator".to_string(),
        });
    }

    persist_genesis(&genesis)?;
    sync_optional_accounts_binding(&genesis)?;

    let mut validators_doc = load_or_default_validators_binding(&genesis)?;
    upsert_validator_binding(
        &mut validators_doc,
        BootstrapValidatorBindingRecord {
            validator_id: validator_id.clone(),
            display_name,
            role: "validator".to_string(),
            consensus_key_algorithm: "ed25519".to_string(),
            consensus_public_key_encoding: "hex".to_string(),
            consensus_public_key: consensus_public_key.clone(),
            consensus_key_fingerprint: consensus_fingerprint,
            network_key_algorithm: "ed25519".to_string(),
            network_public_key_encoding: "hex".to_string(),
            network_public_key: network_public_key.clone(),
            network_key_fingerprint: network_fingerprint,
            weight: 1,
            status: "active".to_string(),
        },
    );
    persist_validators_binding(&genesis, &validators_doc)?;

    let mut bootnodes_doc = load_or_default_bootnodes_binding(&genesis)?;
    upsert_bootnode_binding(
        &mut bootnodes_doc,
        BootstrapBootnodeRecord {
            node_id: format!("bootnode-{validator_id}"),
            display_name: format!("{validator_id} bootnode"),
            transport_key_algorithm: "ed25519".to_string(),
            transport_public_key_encoding: "hex".to_string(),
            transport_public_key: network_public_key,
            transport_key_fingerprint: derive_short_fingerprint(&validator_id),
            address: bootnode_address.clone(),
            transport: "tcp".to_string(),
            status: "active".to_string(),
        },
    );
    persist_bootnodes_binding(&genesis, &bootnodes_doc)?;

    let mut details = BTreeMap::new();
    details.insert("validator_id".to_string(), validator_id);
    details.insert("validator_account_id".to_string(), validator_account_id);
    details.insert("validator_balance".to_string(), balance);
    details.insert("bootnode_address".to_string(), bootnode_address);

    emit_serialized(
        &text_envelope("genesis-add-validator", "ok", details),
        output_format(args),
    )
}

pub fn cmd_genesis_validate(args: &[String]) -> Result<(), AppError> {
    let genesis = load_genesis()?;
    validate_genesis(&genesis)?;
    validate_binding_files(&genesis)?;

    let strict = has_flag(args, "--strict");
    if strict {
        validate_identity_against_repo_policy(&genesis)?;
    }

    let mut details = BTreeMap::new();
    details.insert(
        "chain_id".to_string(),
        genesis.identity.chain_id.to_string(),
    );
    details.insert(
        "network_id".to_string(),
        genesis.identity.network_id.clone(),
    );
    details.insert(
        "accounts".to_string(),
        genesis.state.accounts.len().to_string(),
    );
    details.insert("strict_policy".to_string(), strict.to_string());

    emit_serialized(
        &text_envelope("genesis-validate", "ok", details),
        output_format(args),
    )
}

pub fn cmd_genesis_production_gate(args: &[String]) -> Result<(), AppError> {
    let genesis = load_genesis()?;
    validate_genesis(&genesis)?;
    validate_binding_files(&genesis)?;
    validate_identity_against_repo_policy(&genesis)?;

    let role_report = role_topology_core7_status(&genesis.environment)?;

    if !role_report.missing.is_empty() || !role_report.non_core_active.is_empty() {
        return Err(AppError::new(
            ErrorCode::ConfigInvalid,
            format!(
                "Production gate failed: missing_core7_roles={} non_core_active_roles={}",
                if role_report.missing.is_empty() {
                    "none".to_string()
                } else {
                    role_report.missing.join(",")
                },
                if role_report.non_core_active.is_empty() {
                    "none".to_string()
                } else {
                    role_report.non_core_active.join(",")
                }
            ),
        ));
    }

    let mut details = BTreeMap::new();
    details.insert("environment".to_string(), genesis.environment);
    details.insert(
        "chain_id".to_string(),
        genesis.identity.chain_id.to_string(),
    );
    details.insert("network_id".to_string(), genesis.identity.network_id);
    details.insert("strict_policy".to_string(), "true".to_string());
    details.insert(
        "core7_topology".to_string(),
        "active-and-exclusive".to_string(),
    );
    details.insert("production_gate".to_string(), "pass".to_string());

    emit_serialized(
        &text_envelope("genesis-production-gate", "ok", details),
        output_format(args),
    )
}

struct RoleTopologyCore7Status {
    missing: Vec<String>,
    non_core_active: Vec<String>,
}

fn role_topology_core7_status(environment: &str) -> Result<RoleTopologyCore7Status, AppError> {
    const CORE7_TOPOLOGY_ROLES: [&str; 7] = [
        "core_val",
        "core_prop",
        "core_guard",
        "data_arch",
        "sec_sent",
        "net_relay",
        "serv_rpc",
    ];

    let repo_root = locate_repo_root_for_gate()?;
    let path = repo_root
        .join("configs")
        .join("environments")
        .join(environment)
        .join("topology")
        .join("role-topology.toml");
    let content = read_file(&path)?;

    let mut states = BTreeMap::<String, bool>::new();
    let mut current_role: Option<String> = None;

    for raw in content.lines() {
        let line = raw.trim();

        if line.starts_with("[roles.") && line.ends_with(']') {
            let role = line
                .trim_start_matches("[roles.")
                .trim_end_matches(']')
                .to_string();
            current_role = Some(role.clone());
            states.entry(role).or_insert(false);
            continue;
        }

        if let Some(role) = current_role.as_ref()
            && line.starts_with("enabled")
        {
            states.insert(role.clone(), line.ends_with("true"));
        }
    }

    let missing = CORE7_TOPOLOGY_ROLES
        .iter()
        .filter(|role| !states.get(**role).copied().unwrap_or(false))
        .map(|role| role.to_string())
        .collect::<Vec<_>>();

    let non_core_active = states
        .iter()
        .filter(|(role, enabled)| **enabled && !CORE7_TOPOLOGY_ROLES.contains(&role.as_str()))
        .map(|(role, _)| role.clone())
        .collect::<Vec<_>>();

    Ok(RoleTopologyCore7Status {
        missing,
        non_core_active,
    })
}

fn locate_repo_root_for_gate() -> Result<PathBuf, AppError> {
    let current = std::env::current_dir().map_err(|error| {
        AppError::with_source(
            ErrorCode::FilesystemIoFailed,
            "Genesis production gate failed: cannot resolve current directory",
            error,
        )
    })?;

    for candidate in current.ancestors() {
        if candidate.join("Cargo.toml").exists() && candidate.join("configs").exists() {
            return Ok(candidate.to_path_buf());
        }
    }

    Err(AppError::new(
        ErrorCode::FilesystemIoFailed,
        "Genesis production gate failed: repository root with configs/ not found",
    ))
}

pub fn cmd_genesis_inspect(args: &[String]) -> Result<(), AppError> {
    let genesis = load_genesis()?;
    emit_serialized(&genesis, output_format(args))
}

pub fn cmd_genesis_hash(args: &[String]) -> Result<(), AppError> {
    let raw = read_file(&genesis_path()?)?;
    let mut hasher = Sha256::new();
    hasher.update(raw.as_bytes());
    let digest = hex::encode(hasher.finalize());

    let mut details = BTreeMap::new();
    details.insert("sha256".to_string(), digest);

    emit_serialized(
        &text_envelope("genesis-hash", "ok", details),
        output_format(args),
    )
}

#[cfg(test)]
mod tests {
    use super::{ALLOWED_GENESIS_ACCOUNT_ROLES, is_allowed_genesis_account_role};
    use std::collections::BTreeSet;

    #[test]
    fn genesis_account_role_accepts_core7_and_system_roles() {
        for role in [
            "treasury",
            "validator",
            "system",
            "user",
            "governance",
            "forge",
            "quorum",
            "seal",
            "archive",
            "sentinel",
            "relay",
            "pocket",
        ] {
            assert!(is_allowed_genesis_account_role(role));
        }
    }

    #[test]
    fn genesis_account_role_rejects_unknown_role() {
        assert!(!is_allowed_genesis_account_role("bridge"));
    }

    #[test]
    fn genesis_account_role_list_is_unique_and_stable() {
        assert_eq!(ALLOWED_GENESIS_ACCOUNT_ROLES.len(), 12);

        let unique: BTreeSet<&str> = ALLOWED_GENESIS_ACCOUNT_ROLES.iter().copied().collect();
        assert_eq!(unique.len(), ALLOWED_GENESIS_ACCOUNT_ROLES.len());

        let expected = BTreeSet::from([
            "treasury",
            "validator",
            "system",
            "user",
            "governance",
            "forge",
            "quorum",
            "seal",
            "archive",
            "sentinel",
            "relay",
            "pocket",
        ]);

        assert_eq!(unique, expected);
    }

    #[test]
    fn genesis_account_role_entries_are_lowercase_ascii_safe_tokens() {
        for role in ALLOWED_GENESIS_ACCOUNT_ROLES {
            assert!(
                role.bytes()
                    .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit()),
                "role contains unexpected characters: {role}"
            );
        }
    }

    #[test]
    fn genesis_account_role_rejects_common_bypass_payloads() {
        for payload in [
            "",
            " validator",
            "validator ",
            "validator\n",
            "validator\t",
            "VALIDATOR",
            "Validator",
            "validator\0",
            "validator;DROP TABLE accounts",
            "validator/admin",
            "../validator",
        ] {
            assert!(
                !is_allowed_genesis_account_role(payload),
                "payload should be rejected: {payload:?}"
            );
        }
    }

    #[test]
    fn genesis_account_role_validation_matches_cli_normalization_behavior() {
        for input in ["USER", "TreAsury", "FORGE", "relay"] {
            let normalized = input.to_ascii_lowercase();
            assert!(is_allowed_genesis_account_role(&normalized));
        }

        for input in [" validator ", "system\n", "pocket\t"] {
            let normalized = input.to_ascii_lowercase();
            assert!(!is_allowed_genesis_account_role(&normalized));
        }
    }
}
