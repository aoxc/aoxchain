use super::*;

pub(super) fn genesis_path() -> Result<PathBuf, AppError> {
    Ok(resolve_home()?.join("identity").join("genesis.json"))
}

pub fn genesis_ready() -> bool {
    match genesis_path() {
        Ok(path) => path.exists(),
        Err(_) => false,
    }
}

pub fn cmd_testnet_fixture_init(args: &[String]) -> Result<(), AppError> {
    let home = resolve_home()?;
    ensure_layout(&home)?;
    let fixture_dir = home.join("support").join("deterministic-testnet");
    std::fs::create_dir_all(&fixture_dir).map_err(|error| {
        AppError::with_source(
            ErrorCode::FilesystemIoFailed,
            format!(
                "Failed to create fixture directory {}",
                fixture_dir.display()
            ),
            error,
        )
    })?;

    for member in TESTNET_FIXTURE_MEMBERS {
        let file = fixture_dir.join(format!("{}.txt", member.0));
        let payload = format!(
            "name={}\nlabel={}\np2p_port={}\nrpc_port={}\nvalidator_index={}\nseed={}\nrelease={}\n",
            member.0, member.1, member.2, member.3, member.4, member.5, AOXC_RELEASE_NAME
        );
        write_file(&file, &payload)?;
    }

    let mut details = BTreeMap::new();
    details.insert("fixture_dir".to_string(), fixture_dir.display().to_string());
    details.insert(
        "members".to_string(),
        TESTNET_FIXTURE_MEMBERS.len().to_string(),
    );

    emit_serialized(
        &text_envelope("testnet-fixture-init", "ok", details),
        output_format(args),
    )
}

pub fn cmd_key_bootstrap(args: &[String]) -> Result<(), AppError> {
    let home = resolve_home()?;
    ensure_layout(&home)?;

    if inspect_operator_key().is_ok() && !has_flag(args, "--force") {
        return Err(AppError::new(
            ErrorCode::UsageInvalidArguments,
            "Operator key material already exists. Use `aoxc key-rotate --profile <profile> --password <value>` for controlled rotation, or pass --force to overwrite during non-production bootstrap.",
        ));
    }

    let profile_input = arg_value(args, "--profile").unwrap_or_else(|| "validation".to_string());
    let profile = EnvironmentProfile::parse(&profile_input)?;
    let name = parse_required_or_default_text_arg(args, "--name", "validator-01")?;
    let password = parse_required_text_arg(args, "--password", false, "key bootstrap")?;

    let material = bootstrap_operator_key(&name, profile.as_str(), &password)?;
    emit_serialized(&material, output_format(args))
}

pub fn cmd_key_rotate(args: &[String]) -> Result<(), AppError> {
    let home = resolve_home()?;
    ensure_layout(&home)?;

    let profile_input = arg_value(args, "--profile")
        .or_else(|| load().ok().map(|settings| settings.profile))
        .unwrap_or_else(|| "validation".to_string());
    let profile = EnvironmentProfile::parse(&profile_input)?;
    let name = parse_required_or_default_text_arg(args, "--name", "validator-rotated")?;
    let password = parse_required_text_arg(args, "--password", false, "key rotate")?;

    let summary = rotate_operator_key(&name, profile.as_str(), &password)?;
    emit_serialized(&summary, output_format(args))
}

pub fn cmd_keys_show_fingerprint(args: &[String]) -> Result<(), AppError> {
    let fp = operator_fingerprint()?;
    let mut details = BTreeMap::new();
    details.insert("fingerprint".to_string(), fp);

    emit_serialized(
        &text_envelope("keys-show-fingerprint", "ok", details),
        output_format(args),
    )
}

pub fn cmd_keys_inspect(args: &[String]) -> Result<(), AppError> {
    let summary = inspect_operator_key()?;
    emit_serialized(&summary, output_format(args))
}

pub fn cmd_keys_verify(args: &[String]) -> Result<(), AppError> {
    let password = arg_value(args, "--password");
    verify_operator_key(password.as_deref())?;

    let mut details = BTreeMap::new();
    details.insert("result".to_string(), "verified".to_string());
    details.insert(
        "decrypt_check".to_string(),
        if password.is_some() {
            "passed".to_string()
        } else {
            "skipped-no-password".to_string()
        },
    );

    emit_serialized(
        &text_envelope("keys-verify", "ok", details),
        output_format(args),
    )
}

pub fn cmd_address_create(args: &[String]) -> Result<(), AppError> {
    let profile_input = arg_value(args, "--profile").unwrap_or_else(|| "validation".to_string());
    let profile = EnvironmentProfile::parse(&profile_input)?;
    let name = parse_required_or_default_text_arg(args, "--name", "validator-01")?;
    let password = parse_required_text_arg(args, "--password", false, "address create")?;

    let material = bootstrap_operator_key(&name, profile.as_str(), &password)?;
    let summary = material.summary()?;
    let validator_account_id = format!(
        "AOXC_VALIDATOR_{}",
        summary
            .bundle_fingerprint
            .chars()
            .take(16)
            .collect::<String>()
    );

    emit_serialized(
        &AddressCreateOutput {
            profile: summary.profile,
            validator_name: name,
            validator_account_id,
            bundle_fingerprint: summary.bundle_fingerprint,
            consensus_public_key: summary.consensus_public_key,
            transport_public_key: summary.transport_public_key,
        },
        output_format(args),
    )
}
