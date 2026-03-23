use crate::{
    cli::{AOXC_RELEASE_NAME, TESTNET_FIXTURE_MEMBERS},
    cli_support::{arg_value, emit_serialized, has_flag, output_format, text_envelope},
    config::{
        loader::{init_default, load, load_or_init, persist},
        settings::Settings,
    },
    data_home::{ensure_layout, read_file, resolve_home, write_file},
    error::{AppError, ErrorCode},
    keys::manager::{
        bootstrap_operator_key, consensus_public_key_hex, inspect_operator_key,
        operator_fingerprint, verify_operator_key,
    },
    node::lifecycle::bootstrap_state,
};
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
    let profile = load_or_init()
        .map(|settings| settings.profile)
        .unwrap_or_else(|_| "validator".to_string());
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
    let mut genesis = write_genesis_document(&profile, Some(chain_num))?;
    genesis.block_time_secs = block_time_secs;
    genesis.treasury = treasury;
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
    let home = resolve_home()?;
    ensure_layout(&home)?;
    let profile = arg_value(args, "--profile").unwrap_or_else(|| "validator".to_string());
    let bind_host = arg_value(args, "--bind-host");
    let json_logs = has_flag(args, "--json-logs");
    let settings = if profile == "validator" && bind_host.is_none() && !json_logs {
        init_default()?
    } else {
        let settings = build_profile_settings(home.display().to_string(), &profile, bind_host)?;
        persist(&settings)?;
        settings
    };
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

pub fn cmd_production_bootstrap(args: &[String]) -> Result<(), AppError> {
    let home = resolve_home()?;
    ensure_layout(&home)?;

    let profile = arg_value(args, "--profile").unwrap_or_else(|| "mainnet".to_string());
    let name = arg_value(args, "--name").unwrap_or_else(|| "validator-01".to_string());
    let password = arg_value(args, "--password").ok_or_else(|| {
        AppError::new(
            ErrorCode::UsageInvalidArguments,
            "Missing required flag --password for production bootstrap",
        )
    })?;

    let bind_host = arg_value(args, "--bind-host")
        .unwrap_or_else(|| default_bind_host_for_profile(&profile).to_string());
    let settings = build_profile_settings(
        home.display().to_string(),
        &profile,
        Some(bind_host.clone()),
    )?;
    persist(&settings)?;

    let material = bootstrap_operator_key(&name, &profile, &password)?;
    let genesis = write_genesis_document(&profile, None)?;
    let state = bootstrap_state()?;

    #[derive(Serialize)]
    struct ProductionBootstrapResult {
        profile: String,
        bind_host: String,
        operator_fingerprint: String,
        consensus_public_key: String,
        chain_num: u64,
        node_height: u64,
        readiness_hint: &'static str,
    }

    let result = ProductionBootstrapResult {
        profile,
        bind_host,
        operator_fingerprint: material.fingerprint().to_string(),
        consensus_public_key: genesis
            .validators
            .first()
            .map(|validator| validator.public_key.clone())
            .unwrap_or_default(),
        chain_num: genesis.chain_num,
        node_height: state.current_height,
        readiness_hint: "run `aoxc mainnet-readiness --format json` after bootstrap to inspect remaining blockers",
    };
    emit_serialized(&result, output_format(args))
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

fn build_profile_settings(
    home_dir: String,
    profile: &str,
    bind_host_override: Option<String>,
) -> Result<Settings, AppError> {
    let normalized_profile = profile.trim().to_ascii_lowercase();
    let mut settings = Settings::default_for(home_dir);
    settings.profile = normalized_profile.clone();

    match normalized_profile.as_str() {
        "mainnet" => {
            settings.logging.json = true;
            settings.logging.level = "info".to_string();
            settings.network.bind_host = default_bind_host_for_profile("mainnet").to_string();
        }
        "testnet" => {
            settings.logging.json = true;
            settings.logging.level = "info".to_string();
            settings.network.bind_host = default_bind_host_for_profile("testnet").to_string();
        }
        "validator" => {}
        other => {
            return Err(AppError::new(
                ErrorCode::ConfigInvalid,
                format!("Unsupported profile for config bootstrap: {other}"),
            ));
        }
    }

    if let Some(bind_host) = bind_host_override {
        settings.network.bind_host = bind_host;
    }

    settings
        .validate()
        .map_err(|error| AppError::new(ErrorCode::ConfigInvalid, error))?;
    Ok(settings)
}

fn default_bind_host_for_profile(profile: &str) -> &'static str {
    match profile.trim().to_ascii_lowercase().as_str() {
        "mainnet" | "testnet" => "0.0.0.0",
        _ => "127.0.0.1",
    }
}

fn write_genesis_document(
    profile: &str,
    chain_num_override: Option<u64>,
) -> Result<GenesisDocument, AppError> {
    let chain_num = chain_num_override.unwrap_or_else(|| default_chain_num_for_profile(profile));
    let block_time_secs = if profile.eq_ignore_ascii_case("mainnet") {
        6
    } else {
        4
    };
    let treasury = 1_000_000_000_000u64;
    let validator_key = consensus_public_key_hex().unwrap_or_else(|_| "unbootstrapped".to_string());
    let network_name = match profile.trim().to_ascii_lowercase().as_str() {
        "mainnet" => "AOXC Mainnet Genesis".to_string(),
        "testnet" => "AOXC Testnet Genesis".to_string(),
        _ => "AOXC Local Genesis".to_string(),
    };

    let genesis = GenesisDocument {
        network_name,
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
    Ok(genesis)
}

fn default_chain_num_for_profile(profile: &str) -> u64 {
    match profile.trim().to_ascii_lowercase().as_str() {
        "mainnet" => 1,
        "testnet" => 1001,
        _ => 9001,
    }
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

#[cfg(test)]
mod tests {
    use super::{
        build_profile_settings, cmd_production_bootstrap, default_bind_host_for_profile,
        load_genesis,
    };
    use crate::{cli_support::OutputFormat, test_support::TestHome};

    #[test]
    fn build_profile_settings_hardens_mainnet_defaults() {
        let settings = build_profile_settings("/tmp/aoxc".to_string(), "mainnet", None)
            .expect("mainnet settings should build");

        assert_eq!(settings.profile, "mainnet");
        assert_eq!(settings.network.bind_host, "0.0.0.0");
        assert!(settings.logging.json);
    }

    #[test]
    fn build_profile_settings_rejects_unknown_profile() {
        let error = build_profile_settings("/tmp/aoxc".to_string(), "staging", None)
            .expect_err("unknown profile must be rejected");

        assert_eq!(
            error.code(),
            crate::error::ErrorCode::ConfigInvalid.as_str()
        );
    }

    #[test]
    fn production_bootstrap_materializes_mainnet_ready_foundations() {
        let _home = TestHome::new("production-bootstrap");
        let args = vec![
            "--password".to_string(),
            "Prod#2026!".to_string(),
            "--format".to_string(),
            match OutputFormat::Json {
                OutputFormat::Json => "json".to_string(),
                _ => unreachable!(),
            },
        ];

        cmd_production_bootstrap(&args).expect("production bootstrap should succeed");

        let genesis = load_genesis().expect("bootstrap should write valid genesis");
        assert_eq!(genesis.chain_num, 1);
        assert_eq!(default_bind_host_for_profile("mainnet"), "0.0.0.0");
    }
}
