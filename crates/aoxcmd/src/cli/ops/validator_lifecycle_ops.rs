use super::*;

#[derive(Serialize)]
struct ValidatorLifecycleReport {
    action: &'static str,
    status: &'static str,
    validator_id: String,
    profile: String,
    consensus_key: Option<String>,
    network_key: Option<String>,
    stake: Option<u64>,
    commission_bps: Option<u16>,
    name: Option<String>,
    metadata: Option<String>,
    activation_height: Option<u64>,
    activation_epoch: Option<u64>,
    via_governance: bool,
    from: Option<String>,
    notes: Vec<String>,
}

fn parse_required_flag(args: &[String], flag: &str, context: &str) -> Result<String, AppError> {
    parse_optional_text_arg(args, flag, false).ok_or_else(|| {
        AppError::new(
            ErrorCode::UsageInvalidArguments,
            format!("Flag {flag} is required for {context}"),
        )
    })
}

fn parse_optional_u16_flag(args: &[String], flag: &str) -> Result<Option<u16>, AppError> {
    let Some(value) = arg_value(args, flag) else {
        return Ok(None);
    };
    let parsed = value.parse::<u16>().map_err(|error| {
        AppError::with_source(
            ErrorCode::UsageInvalidArguments,
            format!("Invalid numeric value for {flag}"),
            error,
        )
    })?;
    Ok(Some(parsed))
}

pub fn cmd_validator_join(args: &[String]) -> Result<(), AppError> {
    let validator_id = parse_required_flag(args, "--validator-id", "validator join")?;
    let profile = parse_required_or_default_text_arg(args, "--profile", "testnet", true)?;
    let consensus_key = parse_optional_text_arg(args, "--consensus-key", false);
    let network_key = parse_optional_text_arg(args, "--network-key", false);
    let stake = arg_value(args, "--stake")
        .as_deref()
        .map(|value| parse_positive_u64_value(value, "--stake", "validator join"))
        .transpose()?;
    let commission_bps = parse_optional_u16_flag(args, "--commission-bps")?;
    let name = parse_optional_text_arg(args, "--name", false);
    let metadata = parse_optional_text_arg(args, "--metadata", false);
    let activation_height = arg_value(args, "--activation-height")
        .as_deref()
        .map(|value| {
            parse_positive_u64_value(value, "--activation-height", "validator join activation")
        })
        .transpose()?;
    let activation_epoch = arg_value(args, "--activation-epoch")
        .as_deref()
        .map(|value| {
            parse_positive_u64_value(value, "--activation-epoch", "validator join activation")
        })
        .transpose()?;
    let via_governance = has_flag(args, "--via-governance");
    let from = parse_optional_text_arg(args, "--from", false);

    if activation_height.is_some() && activation_epoch.is_some() {
        return Err(AppError::new(
            ErrorCode::UsageInvalidArguments,
            "Use only one of --activation-height or --activation-epoch",
        ));
    }

    if let Some(value) = commission_bps
        && value > 10_000
    {
        return Err(AppError::new(
            ErrorCode::UsageInvalidArguments,
            "Flag --commission-bps must be between 0 and 10000",
        ));
    }

    emit_serialized(
        &ValidatorLifecycleReport {
            action: "validator-join",
            status: "accepted",
            validator_id,
            profile,
            consensus_key,
            network_key,
            stake,
            commission_bps,
            name,
            metadata,
            activation_height,
            activation_epoch,
            via_governance,
            from,
            notes: vec![
                "Validator onboarding request recorded for deterministic workflow".to_string(),
                "Run `aoxc validator activate --validator-id <id> --stake <amount>` for stake-backed activation".to_string(),
            ],
        },
        output_format(args),
    )
}

pub fn cmd_validator_activate(args: &[String]) -> Result<(), AppError> {
    let validator = parse_required_flag(args, "--validator-id", "validator activate")?;
    let amount = parse_positive_u64_arg(args, "--stake", 1000, "validator activate")?;
    let delegated = ledger::delegate(&validator, amount)?;
    emit_serialized(&delegated, output_format(args))
}

pub fn cmd_validator_bond(args: &[String]) -> Result<(), AppError> {
    cmd_validator_activate(args)
}

pub fn cmd_validator_unbond(args: &[String]) -> Result<(), AppError> {
    let validator = parse_required_flag(args, "--validator-id", "validator unbond")?;
    let amount = parse_positive_u64_arg(args, "--stake", 1000, "validator unbond")?;
    let undelegated = ledger::undelegate(&validator, amount)?;
    emit_serialized(&undelegated, output_format(args))
}

pub fn cmd_validator_set_status(args: &[String]) -> Result<(), AppError> {
    let validator_id = parse_required_flag(args, "--validator-id", "validator set-status")?;
    let status = parse_required_flag(args, "--status", "validator set-status")?;
    if !matches!(status.as_str(), "active" | "inactive" | "jailed") {
        return Err(AppError::new(
            ErrorCode::UsageInvalidArguments,
            "Flag --status must be one of: active, inactive, jailed",
        ));
    }

    let mut details = BTreeMap::new();
    details.insert("validator_id".to_string(), validator_id);
    details.insert("status".to_string(), status);
    details.insert("result".to_string(), "accepted".to_string());

    emit_serialized(
        &text_envelope("validator-set-status", "ok", details),
        output_format(args),
    )
}

pub fn cmd_validator_commission_set(args: &[String]) -> Result<(), AppError> {
    let validator_id = parse_required_flag(args, "--validator-id", "validator commission-set")?;
    let commission_bps = parse_optional_u16_flag(args, "--commission-bps")?.ok_or_else(|| {
        AppError::new(
            ErrorCode::UsageInvalidArguments,
            "Flag --commission-bps is required for validator commission-set",
        )
    })?;

    if commission_bps > 10_000 {
        return Err(AppError::new(
            ErrorCode::UsageInvalidArguments,
            "Flag --commission-bps must be between 0 and 10000",
        ));
    }

    let mut details = BTreeMap::new();
    details.insert("validator_id".to_string(), validator_id);
    details.insert("commission_bps".to_string(), commission_bps.to_string());
    details.insert("result".to_string(), "accepted".to_string());

    emit_serialized(
        &text_envelope("validator-commission-set", "ok", details),
        output_format(args),
    )
}
