use super::*;

const ACCOUNT_ID_DERIVATION_DOMAIN_V2: &str = "AOXC/ACCOUNT_ID/V2";
const ACCOUNT_ID_CHECKSUM_DOMAIN_V2: &str = "AOXC/ACCOUNT_ID/V2/CHECKSUM";
const DEFAULT_ACCOUNT_ID_PAYLOAD_BYTES: usize = 20;
const MIN_ACCOUNT_ID_PAYLOAD_BYTES: usize = 16;
const MAX_ACCOUNT_ID_PAYLOAD_BYTES: usize = 32;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AccountIdMode {
    Secure,
    Legacy,
    Dual,
}

fn parse_account_id_mode(args: &[String]) -> Result<AccountIdMode, AppError> {
    let mode = arg_value(args, "--account-id-mode").unwrap_or_else(|| "secure".to_string());
    match mode.trim().to_ascii_lowercase().as_str() {
        "secure" => Ok(AccountIdMode::Secure),
        "legacy" => Ok(AccountIdMode::Legacy),
        "dual" => Ok(AccountIdMode::Dual),
        _ => Err(AppError::new(
            ErrorCode::UsageInvalidArguments,
            "address create: --account-id-mode must be one of secure|legacy|dual",
        )),
    }
}

fn parse_account_id_payload_bytes(args: &[String]) -> Result<usize, AppError> {
    let Some(raw) = arg_value(args, "--account-id-bytes") else {
        return Ok(DEFAULT_ACCOUNT_ID_PAYLOAD_BYTES);
    };

    let parsed = raw.parse::<usize>().map_err(|error| {
        AppError::with_source(
            ErrorCode::UsageInvalidArguments,
            "address create: --account-id-bytes must be an integer in [16, 32]",
            error,
        )
    })?;

    if !(MIN_ACCOUNT_ID_PAYLOAD_BYTES..=MAX_ACCOUNT_ID_PAYLOAD_BYTES).contains(&parsed) {
        return Err(AppError::new(
            ErrorCode::UsageInvalidArguments,
            "address create: --account-id-bytes must be in [16, 32]",
        ));
    }

    Ok(parsed)
}

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
    let account_mode = parse_account_id_mode(args)?;
    let account_payload_bytes = parse_account_id_payload_bytes(args)?;
    let account_salt = parse_optional_text_arg(args, "--account-salt", false);

    let material = bootstrap_operator_key(&name, profile.as_str(), &password)?;
    let summary = material.summary()?;
    let validator_account_id_legacy = format!(
        "AOXC_VALIDATOR_{}",
        summary
            .bundle_fingerprint
            .chars()
            .take(16)
            .collect::<String>()
    );
    let (secure_account_id, account_id_checksum) = derive_secure_validator_account_id(
        profile,
        &name,
        &summary,
        account_payload_bytes,
        account_salt.as_deref(),
    );
    let validator_account_id = match account_mode {
        AccountIdMode::Secure | AccountIdMode::Dual => secure_account_id,
        AccountIdMode::Legacy => validator_account_id_legacy.clone(),
    };

    emit_serialized(
        &AddressCreateOutput {
            profile: summary.profile,
            validator_name: name,
            validator_account_id,
            validator_account_id_legacy,
            account_id_checksum,
            account_id_derivation_domain: ACCOUNT_ID_DERIVATION_DOMAIN_V2.to_string(),
            bundle_fingerprint: summary.bundle_fingerprint,
            consensus_public_key: summary.consensus_public_key,
            transport_public_key: summary.transport_public_key,
        },
        output_format(args),
    )
}

fn derive_secure_validator_account_id(
    profile: EnvironmentProfile,
    validator_name: &str,
    summary: &crate::keys::material::KeyMaterialSummary,
    payload_bytes: usize,
    account_salt: Option<&str>,
) -> (String, String) {
    let identity = profile.identity();
    let normalized_name = validator_name
        .trim()
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric())
        .flat_map(|ch| ch.to_uppercase())
        .take(16)
        .collect::<String>();
    let profile_tag = profile_account_tag(profile);

    let mut hasher = Sha256::new();
    hasher.update(ACCOUNT_ID_DERIVATION_DOMAIN_V2.as_bytes());
    hasher.update([0x1F]);
    hasher.update(profile.as_str().as_bytes());
    hasher.update([0x1F]);
    hasher.update(identity.network_id.as_bytes());
    hasher.update([0x1F]);
    hasher.update(identity.chain_id.to_be_bytes());
    hasher.update([0x1F]);
    hasher.update(normalized_name.as_bytes());
    hasher.update([0x1F]);
    hasher.update(summary.bundle_fingerprint.as_bytes());
    hasher.update([0x1F]);
    hasher.update(summary.consensus_public_key.as_bytes());
    hasher.update([0x1F]);
    hasher.update(summary.transport_public_key.as_bytes());
    if let Some(salt) = account_salt {
        hasher.update([0x1F]);
        hasher.update(salt.as_bytes());
    }
    let digest = hasher.finalize();

    let payload = hex::encode_upper(&digest[..payload_bytes]);

    let mut checksum_hasher = Sha256::new();
    checksum_hasher.update(ACCOUNT_ID_CHECKSUM_DOMAIN_V2.as_bytes());
    checksum_hasher.update([0x1F]);
    checksum_hasher.update(payload.as_bytes());
    let checksum = hex::encode_upper(&checksum_hasher.finalize()[..4]);

    (
        format!("AOXC_{}_{}_{}", profile_tag, payload, checksum),
        checksum,
    )
}

const fn profile_account_tag(profile: EnvironmentProfile) -> &'static str {
    match profile {
        EnvironmentProfile::Mainnet => "MAIN",
        EnvironmentProfile::Testnet => "TEST",
        EnvironmentProfile::Validation => "VAL",
        EnvironmentProfile::Devnet => "DEV",
        EnvironmentProfile::Localnet => "LOCAL",
    }
}

#[cfg(test)]
mod tests {
    use super::{
        AccountIdMode, EnvironmentProfile, derive_secure_validator_account_id,
        parse_account_id_mode, parse_account_id_payload_bytes,
    };
    use crate::keys::material::KeyMaterial;

    #[test]
    fn secure_account_id_is_deterministic_for_same_inputs() {
        let material = KeyMaterial::generate("validator-01", "validation", "Test#2026!")
            .expect("key material generation should succeed");
        let summary = material.summary().expect("summary should be available");

        let (left, left_checksum) = derive_secure_validator_account_id(
            EnvironmentProfile::Validation,
            "Validator 01",
            &summary,
            20,
            None,
        );
        let (right, right_checksum) = derive_secure_validator_account_id(
            EnvironmentProfile::Validation,
            "Validator 01",
            &summary,
            20,
            None,
        );

        assert_eq!(left, right);
        assert_eq!(left_checksum, right_checksum);
    }

    #[test]
    fn secure_account_id_is_profile_scoped() {
        let material = KeyMaterial::generate("validator-02", "validation", "Test#2026!")
            .expect("key material generation should succeed");
        let summary = material.summary().expect("summary should be available");

        let (validation_id, _) = derive_secure_validator_account_id(
            EnvironmentProfile::Validation,
            "Validator 02",
            &summary,
            20,
            None,
        );
        let (testnet_id, _) = derive_secure_validator_account_id(
            EnvironmentProfile::Testnet,
            "Validator 02",
            &summary,
            20,
            None,
        );

        assert_ne!(validation_id, testnet_id);
        assert!(validation_id.starts_with("AOXC_VAL_"));
        assert!(testnet_id.starts_with("AOXC_TEST_"));
    }

    #[test]
    fn secure_account_id_supports_payload_length_and_salt() {
        let material = KeyMaterial::generate("validator-03", "validation", "Test#2026!")
            .expect("key material generation should succeed");
        let summary = material.summary().expect("summary should be available");

        let (short_id, _) = derive_secure_validator_account_id(
            EnvironmentProfile::Validation,
            "Validator 03",
            &summary,
            16,
            None,
        );
        let (long_id, _) = derive_secure_validator_account_id(
            EnvironmentProfile::Validation,
            "Validator 03",
            &summary,
            32,
            Some("salt-a"),
        );

        assert_ne!(short_id, long_id);
        assert!(short_id.len() < long_id.len());
    }

    #[test]
    fn parse_account_id_mode_validates_allowed_values() {
        let secure =
            parse_account_id_mode(&["--account-id-mode".to_string(), "secure".to_string()])
                .expect("secure mode should parse");
        let dual = parse_account_id_mode(&["--account-id-mode".to_string(), "dual".to_string()])
            .expect("dual mode should parse");

        assert_eq!(secure, AccountIdMode::Secure);
        assert_eq!(dual, AccountIdMode::Dual);
        assert!(
            parse_account_id_mode(&["--account-id-mode".to_string(), "invalid".to_string()])
                .is_err()
        );
    }

    #[test]
    fn parse_account_id_payload_bytes_enforces_range() {
        assert_eq!(
            parse_account_id_payload_bytes(&[]).expect("default bytes should parse"),
            20
        );
        assert_eq!(
            parse_account_id_payload_bytes(&["--account-id-bytes".to_string(), "32".to_string()])
                .expect("maximum allowed bytes should parse"),
            32
        );
        assert!(
            parse_account_id_payload_bytes(&["--account-id-bytes".to_string(), "12".to_string()])
                .is_err()
        );
    }
}
