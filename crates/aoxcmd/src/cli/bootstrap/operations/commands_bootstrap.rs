use super::*;

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
    let password = parse_required_text_arg(args, "--password", false, "topology bootstrap")?;
    let name_prefix = parse_required_or_default_text_arg(args, "--name-prefix", "validator")?;
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
            (
                profile,
                vec![TopologyBootstrapNodeSummary {
                    topology_role: "single-node".to_string(),
                    bootstrap,
                }],
                "Single-node topology generated. Use `aoxc node start --home <node-home>` to launch.",
            )
        }
        "mainchain-4" => {
            let mut nodes = Vec::with_capacity(4);
            for ordinal in 1..=4u16 {
                let operator_name = format!("{name_prefix}-{:02}", ordinal);
                let bootstrap = bootstrap_profile_directory_with_port_offset(
                    &output_dir.join(format!("node-{:02}", ordinal)),
                    EnvironmentProfile::Mainnet,
                    &operator_name,
                    &password,
                    (ordinal - 1) * 10,
                )?;
                nodes.push(TopologyBootstrapNodeSummary {
                    topology_role: format!("mainchain-validator-{ordinal}"),
                    bootstrap,
                });
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
        launch_hint,
    };

    emit_serialized(&result, output_format(args))
}
