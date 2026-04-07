use super::*;
use std::io::{self, Write};

fn prompt_password_twice(context: &str) -> Result<String, AppError> {
    let mut first = String::new();
    let mut second = String::new();

    print!("Enter password for {context}: ");
    io::stdout().flush().map_err(|error| {
        AppError::with_source(
            ErrorCode::FilesystemIoFailed,
            "Failed to flush stdout for password prompt",
            error,
        )
    })?;
    io::stdin().read_line(&mut first).map_err(|error| {
        AppError::with_source(
            ErrorCode::FilesystemIoFailed,
            "Failed to read password input",
            error,
        )
    })?;

    print!("Confirm password for {context}: ");
    io::stdout().flush().map_err(|error| {
        AppError::with_source(
            ErrorCode::FilesystemIoFailed,
            "Failed to flush stdout for password confirmation prompt",
            error,
        )
    })?;
    io::stdin().read_line(&mut second).map_err(|error| {
        AppError::with_source(
            ErrorCode::FilesystemIoFailed,
            "Failed to read password confirmation input",
            error,
        )
    })?;

    let first = first.trim().to_string();
    let second = second.trim().to_string();
    if first.is_empty() {
        return Err(AppError::new(
            ErrorCode::UsageInvalidArguments,
            "Password must not be empty.",
        ));
    }
    if first != second {
        return Err(AppError::new(
            ErrorCode::UsageInvalidArguments,
            "Password confirmation mismatch.",
        ));
    }
    Ok(first)
}

fn resolve_or_prompt_password(args: &[String], context: &str) -> Result<String, AppError> {
    if let Some(value) = arg_value(args, "--password") {
        let password = value.trim().to_string();
        if password.is_empty() {
            return Err(AppError::new(
                ErrorCode::UsageInvalidArguments,
                "Password must not be empty.",
            ));
        }
        Ok(password)
    } else {
        prompt_password_twice(context)
    }
}

fn node_runtime_summary(
    topology_role: String,
    allocation_preset: &str,
    genesis_accounts_preview: Vec<BootstrapAccountRecord>,
    genesis_accounts_total: usize,
    bootstrap: ProfileBootstrapSummary,
) -> TopologyBootstrapNodeSummary {
    let rpc_url = format!("http://{}:{}", bootstrap.bind_host, bootstrap.rpc_port);
    let metrics_url = format!(
        "http://{}:{}/metrics",
        bootstrap.bind_host, bootstrap.prometheus_port
    );
    let start_command = format!("aoxc node start --home {}", bootstrap.home_dir);
    let query_commands = vec![
        format!("aoxc query network peers --home {}", bootstrap.home_dir),
        format!("aoxc query chain status --home {}", bootstrap.home_dir),
        format!("aoxc api status --home {}", bootstrap.home_dir),
    ];

    TopologyBootstrapNodeSummary {
        topology_role,
        bootstrap,
        rpc_url,
        metrics_url,
        start_command,
        query_commands,
        allocation_preset: allocation_preset.to_string(),
        genesis_accounts_total,
        genesis_accounts_preview,
    }
}

fn upsert_named_account(
    genesis: &mut BootstrapGenesisDocument,
    account_id: String,
    balance: &str,
    role: &str,
) {
    if let Some(existing) = genesis
        .state
        .accounts
        .iter_mut()
        .find(|entry| entry.account_id == account_id)
    {
        existing.balance = balance.to_string();
        existing.role = role.to_string();
    } else {
        genesis.state.accounts.push(BootstrapAccountRecord {
            account_id,
            balance: balance.to_string(),
            role: role.to_string(),
        });
    }
}

fn apply_allocation_preset(
    home_dir: &str,
    allocation_preset: &str,
    topology_role: &str,
    ordinal: usize,
) -> Result<(usize, Vec<BootstrapAccountRecord>), AppError> {
    let _home_override = ScopedHomeOverride::install(Path::new(home_dir));
    let mut genesis = load_genesis()?;

    let stem = topology_role
        .replace('-', "_")
        .to_ascii_uppercase()
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric() || *ch == '_')
        .collect::<String>();
    let operator = format!("AOXC_OP_{stem}_{ordinal:02}");
    let treasury = format!("AOXC_TREASURY_{stem}_{ordinal:02}");
    let governance = format!("AOXC_GOV_{stem}_{ordinal:02}");
    let system = format!("AOXC_SYS_{stem}_{ordinal:02}");
    let user = format!("AOXC_USER_{stem}_{ordinal:02}");
    let pocket = format!("AOXC_POCKET_{stem}_{ordinal:02}");

    match allocation_preset {
        "minimal" => {
            upsert_named_account(&mut genesis, treasury, "500000", "treasury");
            upsert_named_account(&mut genesis, operator, "50000", "validator");
            upsert_named_account(&mut genesis, user, "10000", "user");
        }
        "balanced" => {
            upsert_named_account(&mut genesis, treasury, "5000000", "treasury");
            upsert_named_account(&mut genesis, operator, "1000000", "validator");
            upsert_named_account(&mut genesis, governance, "250000", "governance");
            upsert_named_account(&mut genesis, system, "150000", "system");
            upsert_named_account(&mut genesis, user, "75000", "user");
        }
        "validator-heavy" => {
            upsert_named_account(&mut genesis, treasury, "3000000", "treasury");
            upsert_named_account(&mut genesis, operator, "4000000", "validator");
            upsert_named_account(&mut genesis, governance, "350000", "governance");
            upsert_named_account(&mut genesis, system, "200000", "system");
            upsert_named_account(&mut genesis, pocket, "125000", "pocket");
        }
        _ => {
            return Err(AppError::new(
                ErrorCode::UsageInvalidArguments,
                "Unsupported --allocation-preset. Use minimal, balanced, or validator-heavy.",
            ));
        }
    }

    persist_genesis(&genesis)?;
    sync_optional_accounts_binding(&genesis)?;

    let preview = genesis.state.accounts.iter().take(8).cloned().collect();
    Ok((genesis.state.accounts.len(), preview))
}

pub fn cmd_config_init(args: &[String]) -> Result<(), AppError> {
    let home = resolve_home()?;
    ensure_layout(&home)?;

    let profile_input = arg_value(args, "--profile").unwrap_or_else(|| "validation".to_string());
    let profile = EnvironmentProfile::parse(&profile_input)?;
    let bind_host = parse_optional_text_arg(args, "--bind-host", false);
    let json_logs = has_flag(args, "--json-logs");

    let mut settings =
        if profile == EnvironmentProfile::Validation && bind_host.is_none() && !json_logs {
            init_default()?
        } else {
            build_profile_settings(home.display().to_string(), profile, bind_host)?
        };

    if json_logs {
        settings.logging.json = true;
    }

    settings
        .validate()
        .map_err(|error| AppError::new(ErrorCode::ConfigInvalid, error))?;

    persist(&settings)?;
    emit_serialized(&settings, output_format(args))
}

pub fn cmd_config_validate(args: &[String]) -> Result<(), AppError> {
    let settings = load()?;
    settings
        .validate()
        .map_err(|error| AppError::new(ErrorCode::ConfigInvalid, error))?;

    let mut details = BTreeMap::new();
    details.insert("profile".to_string(), settings.profile);
    details.insert("result".to_string(), "valid".to_string());

    emit_serialized(
        &text_envelope("config-validate", "ok", details),
        output_format(args),
    )
}

pub fn cmd_config_print(args: &[String]) -> Result<(), AppError> {
    let settings = load()?;
    let printable = if has_flag(args, "--redact") {
        settings.redacted()
    } else {
        settings
    };

    emit_serialized(&printable, output_format(args))
}

fn bootstrap_root() -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after unix epoch")
        .as_nanos();

    std::env::temp_dir().join(format!("aoxc-bootstrap-pid{}-{}", process::id(), nanos))
}

pub fn cmd_production_bootstrap(args: &[String]) -> Result<(), AppError> {
    let home = resolve_home()?;
    ensure_layout(&home)?;

    let profile = EnvironmentProfile::parse(
        &arg_value(args, "--profile").unwrap_or_else(|| "mainnet".to_string()),
    )?;
    let name = parse_required_or_default_text_arg(args, "--name", "validator-01")?;
    let password = parse_required_text_arg(args, "--password", false, "production bootstrap")?;
    let bind_host = parse_optional_text_arg(args, "--bind-host", false);

    let mut settings = build_profile_settings(home.display().to_string(), profile, bind_host)?;
    settings.logging.json = true;
    settings
        .validate()
        .map_err(|error| AppError::new(ErrorCode::ConfigInvalid, error))?;
    persist(&settings)?;

    let mut genesis = profile.genesis_document();
    let _material = bootstrap_operator_key(&name, profile.as_str(), &password)?;
    let operator_summary = inspect_operator_key()?;
    upsert_validator_account(
        &mut genesis,
        &operator_summary,
        DEFAULT_VALIDATOR_GENESIS_BALANCE,
    )?;
    materialize_binding_documents(&genesis, &operator_summary, &settings)?;

    let genesis_json = serde_json::to_string_pretty(&genesis).map_err(|error| {
        AppError::with_source(
            ErrorCode::OutputEncodingFailed,
            "Failed to encode production genesis document",
            error,
        )
    })?;
    write_file(&genesis_path()?, &genesis_json)?;

    let operator_fp = operator_fingerprint()?;
    let consensus_pk = consensus_public_key_hex()?;
    let node_state = bootstrap_state()?;

    let summary = ProfileBootstrapSummary {
        profile: profile.as_str().to_string(),
        home_dir: home.display().to_string(),
        bind_host: settings.network.bind_host.clone(),
        p2p_port: settings.network.p2p_port,
        rpc_port: settings.network.rpc_port,
        prometheus_port: settings.telemetry.prometheus_port,
        chain_id: genesis.identity.chain_id,
        network_id: genesis.identity.network_id,
        operator_fingerprint: operator_fp,
        consensus_public_key: consensus_pk,
        node_height: node_state.current_height,
    };

    emit_serialized(&summary, output_format(args))
}

pub fn cmd_dual_profile_bootstrap(args: &[String]) -> Result<(), AppError> {
    let output_dir = arg_value(args, "--output-dir")
        .map(PathBuf::from)
        .unwrap_or_else(|| bootstrap_root().join("dual-profile"));

    let password = parse_required_text_arg(args, "--password", false, "dual profile bootstrap")?;
    let name = parse_required_or_default_text_arg(args, "--name", "validator-01")?;

    let mainnet_dir = output_dir.join("mainnet");
    let testnet_dir = output_dir.join("testnet");

    let mainnet =
        bootstrap_profile_directory(&mainnet_dir, EnvironmentProfile::Mainnet, &name, &password)?;
    let testnet =
        bootstrap_profile_directory(&testnet_dir, EnvironmentProfile::Testnet, &name, &password)?;

    let result = DualProfileBootstrapResult {
        output_dir: output_dir.display().to_string(),
        profiles: vec![mainnet, testnet],
        launch_hint: "Use the generated profile directories with AOXC runtime launch surfaces.",
    };

    emit_serialized(&result, output_format(args))
}

pub fn cmd_topology_bootstrap(args: &[String]) -> Result<(), AppError> {
    let topology_mode = arg_value(args, "--mode").unwrap_or_else(|| "single".to_string());
    let password = resolve_or_prompt_password(args, "topology bootstrap")?;
    let name_prefix = parse_required_or_default_text_arg(args, "--name-prefix", "validator")?;
    let allocation_preset =
        arg_value(args, "--allocation-preset").unwrap_or_else(|| "balanced".to_string());
    let output_dir = arg_value(args, "--output-dir")
        .map(PathBuf::from)
        .unwrap_or_else(|| bootstrap_root().join("topology-bootstrap"));

    let (profile, nodes, launch_hint): (
        EnvironmentProfile,
        Vec<TopologyBootstrapNodeSummary>,
        &'static str,
    ) = match topology_mode.as_str() {
        "single" => {
            let profile = EnvironmentProfile::parse(
                &arg_value(args, "--profile").unwrap_or_else(|| "testnet".to_string()),
            )?;
            let operator_name = format!("{name_prefix}-01");
            let bootstrap = bootstrap_profile_directory(
                &output_dir.join("node-01"),
                profile,
                &operator_name,
                &password,
            )?;
            let (genesis_accounts_total, genesis_accounts_preview) =
                apply_allocation_preset(&bootstrap.home_dir, &allocation_preset, "single-node", 1)?;
            (
                profile,
                vec![node_runtime_summary(
                    "single-node".to_string(),
                    &allocation_preset,
                    genesis_accounts_preview,
                    genesis_accounts_total,
                    bootstrap,
                )],
                "Single-node topology generated. Use `aoxc node start --home <node-home>` to launch.",
            )
        }
        "mainchain-4" => {
            let mut nodes = Vec::with_capacity(4);
            for ordinal in 1..=4u16 {
                let operator_name = format!("{name_prefix}-{:02}", ordinal);
                let topology_role = format!("mainchain-validator-{ordinal}");
                let bootstrap = bootstrap_profile_directory_with_port_offset(
                    &output_dir.join(format!("node-{:02}", ordinal)),
                    EnvironmentProfile::Mainnet,
                    &operator_name,
                    &password,
                    (ordinal - 1) * 10,
                )?;
                let (genesis_accounts_total, genesis_accounts_preview) = apply_allocation_preset(
                    &bootstrap.home_dir,
                    &allocation_preset,
                    &topology_role,
                    ordinal as usize,
                )?;
                nodes.push(node_runtime_summary(
                    topology_role,
                    &allocation_preset,
                    genesis_accounts_preview,
                    genesis_accounts_total,
                    bootstrap,
                ));
            }
            (
                EnvironmentProfile::Mainnet,
                nodes,
                "Four-node mainchain topology generated. Start each node with its dedicated --home directory.",
            )
        }
        _ => {
            return Err(AppError::new(
                ErrorCode::UsageInvalidArguments,
                "Unsupported --mode. Use --mode single or --mode mainchain-4.",
            ));
        }
    };

    let result = TopologyBootstrapResult {
        topology_mode,
        output_dir: output_dir.display().to_string(),
        profile: profile.as_str().to_string(),
        node_count: nodes.len(),
        nodes,
        genesis_consistency: vec![
            "Use the exact generated genesis.json, validators.json, bootnodes.json, and certificate.json for every node in the topology.".to_string(),
            "Do not mix identity bundles across different topology-bootstrap runs.".to_string(),
        ],
        rpc_api_runbook: vec![
            "Run `aoxc network-identity-gate --enforce --env <profile>` before startup.".to_string(),
            "Start each node with its dedicated home directory using the generated start_command value.".to_string(),
            "Validate API/RPC with `aoxc api status`, `aoxc query chain status`, and `aoxc query network peers` for every node.".to_string(),
        ],
        economics_summary: vec![
            format!("allocation_preset={allocation_preset}"),
            "Preset adds deterministic treasury/system/governance/user-style accounts into genesis.".to_string(),
            "Use `aoxc genesis-validate --strict` after bootstrap to enforce deterministic genesis integrity.".to_string(),
        ],
        launch_hint,
    };

    emit_serialized(&result, output_format(args))
}
