use crate::{
    cli::{AOXC_RELEASE_NAME, TESTNET_FIXTURE_MEMBERS},
    cli_support::{arg_value, emit_serialized, has_flag, output_format, text_envelope},
    config::loader::{init_default, load, load_or_init},
    data_home::{ensure_layout, read_file, resolve_home, write_file},
    error::{AppError, ErrorCode},
    keys::manager::{
        bootstrap_operator_key, consensus_public_key_hex, export_operator_identity,
        inspect_operator_key, operator_fingerprint, rotate_operator_key, set_operator_key_state,
        verify_operator_key,
    },
};
use aoxcore::identity::key_bundle::NodeKeyOperationalState;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, path::PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct GenesisDocument {
    network_name: String,
    chain_num: u64,
    block_time_secs: u64,
    treasury: u64,
    created_at: String,
    identity_root: String,
    validators: Vec<GenesisValidator>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct GenesisValidator {
    name: String,
    public_key: String,
}

fn genesis_path() -> Result<PathBuf, AppError> {
    Ok(resolve_home()?.join("identity").join("genesis.json"))
}

pub fn cmd_testnet_fixture_init(args: &[String]) -> Result<(), AppError> {
    let home = resolve_home()?;
    ensure_layout(&home)?;
    let fixture_dir = home.join("support").join("deterministic-testnet");
    std::fs::create_dir_all(&fixture_dir).map_err(|e| {
        AppError::with_source(
            ErrorCode::FilesystemIoFailed,
            format!(
                "Failed to create fixture directory {}",
                fixture_dir.display()
            ),
            e,
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
    let profile = arg_value(args, "--profile").unwrap_or_else(|| "validator".to_string());
    let name = arg_value(args, "--name").unwrap_or_else(|| "validator-01".to_string());
    let password = arg_value(args, "--password").ok_or_else(|| {
        AppError::new(
            ErrorCode::UsageInvalidArguments,
            "Missing required flag --password for key bootstrap",
        )
    })?;
    let material = bootstrap_operator_key(&name, &profile, &password)?;
    emit_serialized(&material, output_format(args))
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

pub fn cmd_keys_export_identity(args: &[String]) -> Result<(), AppError> {
    let chain = arg_value(args, "--chain").unwrap_or_else(|| "AOXC-LOCAL".to_string());
    let actor_id = arg_value(args, "--actor-id").unwrap_or_else(|| "AOXC-VAL-LOCAL-01".to_string());
    let zone = arg_value(args, "--zone").unwrap_or_else(|| "local".to_string());
    let issued_at = arg_value(args, "--issued-at")
        .unwrap_or_else(|| "0".to_string())
        .parse::<u64>()
        .map_err(|_| {
            AppError::new(
                ErrorCode::UsageInvalidArguments,
                "Invalid --issued-at value",
            )
        })?;
    let expires_at = arg_value(args, "--expires-at")
        .unwrap_or_else(|| "4102444800".to_string())
        .parse::<u64>()
        .map_err(|_| {
            AppError::new(
                ErrorCode::UsageInvalidArguments,
                "Invalid --expires-at value",
            )
        })?;
    let artifacts = export_operator_identity(&chain, &actor_id, &zone, issued_at, expires_at)?;

    if let Some(output_dir) = arg_value(args, "--output-dir") {
        let dir = std::path::PathBuf::from(output_dir);
        std::fs::create_dir_all(&dir).map_err(|error| {
            AppError::with_source(
                ErrorCode::FilesystemIoFailed,
                format!(
                    "Failed to create identity export directory {}",
                    dir.display()
                ),
                error,
            )
        })?;
        let cert_path = dir.join("node.cert.json");
        let passport_path = dir.join("node.passport.json");
        let cert_text = serde_json::to_string_pretty(&artifacts.certificate).map_err(|error| {
            AppError::with_source(
                ErrorCode::OutputEncodingFailed,
                "Failed to encode exported certificate JSON",
                error,
            )
        })?;
        let passport_text = serde_json::to_string_pretty(&artifacts.passport).map_err(|error| {
            AppError::with_source(
                ErrorCode::OutputEncodingFailed,
                "Failed to encode exported passport JSON",
                error,
            )
        })?;
        write_file(&cert_path, &cert_text)?;
        write_file(&passport_path, &passport_text)?;
    }

    emit_serialized(&artifacts, output_format(args))
}

pub fn cmd_keys_enter_recovery_mode(args: &[String]) -> Result<(), AppError> {
    let summary = set_operator_key_state(NodeKeyOperationalState::RecoveryOnly)?;
    emit_serialized(&summary, output_format(args))
}

pub fn cmd_keys_mark_compromised(args: &[String]) -> Result<(), AppError> {
    let summary = set_operator_key_state(NodeKeyOperationalState::Compromised)?;
    emit_serialized(&summary, output_format(args))
}

pub fn cmd_keys_rotate(args: &[String]) -> Result<(), AppError> {
    let profile = arg_value(args, "--profile").unwrap_or_else(|| "validator".to_string());
    let name = arg_value(args, "--name").unwrap_or_else(|| "validator-01".to_string());
    let password = arg_value(args, "--password").ok_or_else(|| {
        AppError::new(
            ErrorCode::UsageInvalidArguments,
            "Missing required flag --password for key rotation",
        )
    })?;
    let summary = rotate_operator_key(&name, &profile, &password)?;
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

pub fn cmd_genesis_init(args: &[String]) -> Result<(), AppError> {
    let chain_num = arg_value(args, "--chain-num")
        .unwrap_or_else(|| "1001".to_string())
        .parse::<u64>()
        .map_err(|_| {
            AppError::new(
                ErrorCode::UsageInvalidArguments,
                "Invalid --chain-num value",
            )
        })?;
    let block_time_secs = arg_value(args, "--block-time")
        .unwrap_or_else(|| "6".to_string())
        .parse::<u64>()
        .map_err(|_| {
            AppError::new(
                ErrorCode::UsageInvalidArguments,
                "Invalid --block-time value",
            )
        })?;
    let treasury = arg_value(args, "--treasury")
        .unwrap_or_else(|| "1000000000000".to_string())
        .parse::<u64>()
        .map_err(|_| AppError::new(ErrorCode::UsageInvalidArguments, "Invalid --treasury value"))?;
    let validator_key = consensus_public_key_hex().unwrap_or_else(|_| "unbootstrapped".to_string());

    let genesis = GenesisDocument {
        network_name: "AOXC Local Genesis".to_string(),
        chain_num,
        block_time_secs,
        treasury,
        created_at: Utc::now().to_rfc3339(),
        identity_root: format!("aoxc-root-{chain_num}"),
        validators: vec![GenesisValidator {
            name: "local-operator".to_string(),
            public_key: validator_key,
        }],
    };
    let content = serde_json::to_string_pretty(&genesis).map_err(|e| {
        AppError::with_source(
            ErrorCode::OutputEncodingFailed,
            "Failed to encode genesis document",
            e,
        )
    })?;
    write_file(&genesis_path()?, &content)?;
    emit_serialized(&genesis, output_format(args))
}

pub fn cmd_genesis_validate(args: &[String]) -> Result<(), AppError> {
    let genesis = load_genesis()?;
    validate_genesis(&genesis)?;
    let mut details = BTreeMap::new();
    details.insert("chain_num".to_string(), genesis.chain_num.to_string());
    details.insert(
        "validators".to_string(),
        genesis.validators.len().to_string(),
    );
    emit_serialized(
        &text_envelope("genesis-validate", "ok", details),
        output_format(args),
    )
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

pub fn cmd_config_init(args: &[String]) -> Result<(), AppError> {
    let settings = init_default()?;
    emit_serialized(&settings, output_format(args))
}

pub fn cmd_config_validate(args: &[String]) -> Result<(), AppError> {
    let settings = load_or_init()?;
    settings
        .validate()
        .map_err(|e| AppError::new(ErrorCode::ConfigInvalid, e))?;
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

pub(crate) fn genesis_ready() -> bool {
    load_genesis().is_ok()
}

fn load_genesis() -> Result<GenesisDocument, AppError> {
    let raw = read_file(&genesis_path()?).map_err(|_| {
        AppError::new(
            ErrorCode::GenesisInvalid,
            "Genesis document is missing. Execute genesis-init before validation.",
        )
    })?;
    let genesis: GenesisDocument = serde_json::from_str(&raw).map_err(|e| {
        AppError::with_source(
            ErrorCode::GenesisInvalid,
            "Failed to parse genesis document",
            e,
        )
    })?;
    validate_genesis(&genesis)?;
    Ok(genesis)
}

fn validate_genesis(genesis: &GenesisDocument) -> Result<(), AppError> {
    if genesis.chain_num == 0 || genesis.block_time_secs == 0 || genesis.treasury == 0 {
        return Err(AppError::new(
            ErrorCode::GenesisInvalid,
            "Genesis document failed non-zero value validation",
        ));
    }
    if genesis.validators.is_empty() {
        return Err(AppError::new(
            ErrorCode::GenesisInvalid,
            "Genesis document must contain at least one validator",
        ));
    }
    Ok(())
}
