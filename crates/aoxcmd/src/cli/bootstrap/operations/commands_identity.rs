use super::*;

const ACCOUNT_ID_DERIVATION_DOMAIN_V2: &str = "AOXC/ACCOUNT_ID/V2";
const ACCOUNT_ID_CHECKSUM_DOMAIN_V2: &str = "AOXC/ACCOUNT_ID/V2/CHECKSUM";
const DEFAULT_ACCOUNT_ID_PAYLOAD_BYTES: usize = 20;

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
    let secure_account = derive_secure_validator_account_id(
        profile,
        &name,
        &summary,
        account_payload_bytes,
        account_salt.as_deref(),
    );
    let validator_account_id = account_mode.select_primary(
        &secure_account.account_id,
        &validator_account_id_legacy,
    );

    emit_serialized(
        &AddressCreateOutput {
            profile: summary.profile,
            validator_name: name,
            validator_account_id,
            validator_account_id_mode: account_mode.as_str().to_string(),
            validator_account_id_legacy,
            account_id_checksum: secure_account.checksum,
            account_id_derivation_domain: ACCOUNT_ID_DERIVATION_DOMAIN_V2.to_string(),
            account_id_payload_hex_chars: secure_account.payload_hex_chars,
            account_id_salt_applied: account_salt.is_some(),
            bundle_fingerprint: summary.bundle_fingerprint,
            consensus_public_key: summary.consensus_public_key,
            transport_public_key: summary.transport_public_key,
        },
        output_format(args),
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AccountIdMode {
    Secure,
    Legacy,
    Dual,
}

impl AccountIdMode {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Secure => "secure",
            Self::Legacy => "legacy",
            Self::Dual => "dual",
        }
    }

    fn select_primary<'a>(self, secure: &'a str, legacy: &'a str) -> String {
        match self {
            Self::Secure | Self::Dual => secure.to_string(),
            Self::Legacy => legacy.to_string(),
        }
    }
}

fn parse_account_id_mode(args: &[String]) -> Result<AccountIdMode, AppError> {
    let mode = parse_optional_text_arg(args, "--account-id-mode", true)
        .unwrap_or_else(|| "dual".to_string())
        .to_ascii_lowercase();
    match mode.as_str() {
        "secure" | "v2" => Ok(AccountIdMode::Secure),
        "legacy" | "v1" => Ok(AccountIdMode::Legacy),
        "dual" | "compat" => Ok(AccountIdMode::Dual),
        other => Err(AppError::new(
            ErrorCode::UsageInvalidArguments,
            format!(
                "Unsupported account-id mode `{}`; expected secure, legacy, or dual",
                other
            ),
        )),
    }
}

fn parse_account_id_payload_bytes(args: &[String]) -> Result<usize, AppError> {
    let Some(value) = parse_optional_text_arg(args, "--account-id-bytes", false) else {
        return Ok(DEFAULT_ACCOUNT_ID_PAYLOAD_BYTES);
    };
    let parsed = value.parse::<usize>().map_err(|error| {
        AppError::with_source(
            ErrorCode::UsageInvalidArguments,
            "address create --account-id-bytes must be a positive integer",
            error,
        )
    })?;
    if !(16..=32).contains(&parsed) {
        return Err(AppError::new(
            ErrorCode::UsageInvalidArguments,
            "address create --account-id-bytes must be between 16 and 32",
        ));
    }
    Ok(parsed)
}

struct DerivedAccountId {
    account_id: String,
    checksum: String,
    payload_hex_chars: usize,
}

fn derive_secure_validator_account_id(
    profile: EnvironmentProfile,
    validator_name: &str,
    summary: &crate::keys::material::KeyMaterialSummary,
    payload_bytes: usize,
    account_salt: Option<&str>,
) -> DerivedAccountId {
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

    DerivedAccountId {
        account_id: format!("AOXC_{}_{}_{}", profile_tag, payload, checksum),
        checksum,
        payload_hex_chars: payload.len(),
    }
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
        AccountIdMode, DEFAULT_ACCOUNT_ID_PAYLOAD_BYTES, EnvironmentProfile,
        derive_secure_validator_account_id, parse_account_id_mode, parse_account_id_payload_bytes,
    };
    use crate::keys::material::KeyMaterial;

    #[test]
    fn secure_account_id_is_deterministic_for_same_inputs() {
        let material = KeyMaterial::generate("validator-01", "validation", "Test#2026!")
            .expect("key material generation should succeed");
        let summary = material.summary().expect("summary should be available");

        let left = derive_secure_validator_account_id(
            EnvironmentProfile::Validation,
            "Validator 01",
            &summary,
            DEFAULT_ACCOUNT_ID_PAYLOAD_BYTES,
            None,
        );
        let right = derive_secure_validator_account_id(
            EnvironmentProfile::Validation,
            "Validator 01",
            &summary,
            DEFAULT_ACCOUNT_ID_PAYLOAD_BYTES,
            None,
        );

        assert_eq!(left.account_id, right.account_id);
        assert_eq!(left.checksum, right.checksum);
    }

    #[test]
    fn secure_account_id_is_profile_scoped() {
        let material = KeyMaterial::generate("validator-02", "validation", "Test#2026!")
            .expect("key material generation should succeed");
        let summary = material.summary().expect("summary should be available");

        let validation = derive_secure_validator_account_id(
            EnvironmentProfile::Validation,
            "Validator 02",
            &summary,
            DEFAULT_ACCOUNT_ID_PAYLOAD_BYTES,
            None,
        );
        let testnet = derive_secure_validator_account_id(
            EnvironmentProfile::Testnet,
            "Validator 02",
            &summary,
            DEFAULT_ACCOUNT_ID_PAYLOAD_BYTES,
            None,
        );

        assert_ne!(validation.account_id, testnet.account_id);
        assert!(validation.account_id.starts_with("AOXC_VAL_"));
        assert!(testnet.account_id.starts_with("AOXC_TEST_"));
    }

    #[test]
    fn secure_account_id_changes_with_optional_salt() {
        let material = KeyMaterial::generate("validator-03", "validation", "Test#2026!")
            .expect("key material generation should succeed");
        let summary = material.summary().expect("summary should be available");
        let default_id = derive_secure_validator_account_id(
            EnvironmentProfile::Validation,
            "Validator 03",
            &summary,
            DEFAULT_ACCOUNT_ID_PAYLOAD_BYTES,
            None,
        );
        let salted_id = derive_secure_validator_account_id(
            EnvironmentProfile::Validation,
            "Validator 03",
            &summary,
            DEFAULT_ACCOUNT_ID_PAYLOAD_BYTES,
            Some("HIGH-SECURITY-SALT"),
        );
        assert_ne!(default_id.account_id, salted_id.account_id);
    }

    #[test]
    fn account_id_mode_parser_accepts_aliases() {
        let args = vec![
            "address-create".to_string(),
            "--account-id-mode".to_string(),
            "v2".to_string(),
        ];
        assert_eq!(
            parse_account_id_mode(&args).expect("v2 alias should parse"),
            AccountIdMode::Secure
        );
    }

    #[test]
    fn account_id_payload_bytes_parser_enforces_bounds() {
        let args = vec![
            "address-create".to_string(),
            "--account-id-bytes".to_string(),
            "12".to_string(),
        ];
        assert!(parse_account_id_payload_bytes(&args).is_err());
    }
}
