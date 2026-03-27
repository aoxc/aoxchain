//! AOXC CLI bootstrap commands.
//!
//! This module provides bootstrap-oriented command handlers for:
//! - deterministic operator key bootstrap,
//! - canonical environment-aware genesis installation,
//! - environment-aware configuration initialization,
//! - dual-profile bootstrap flows used for local operational readiness.
//!
//! The implementation is intentionally aligned with the AOXC
//! single-binary, multi-network operating model. Network identity is derived
//! from canonical environment profiles and environment bundles rather than
//! ad hoc chain-number arguments.

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
    node::{engine, lifecycle::bootstrap_state},
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    env, fs,
    path::{Path, PathBuf},
};

/// Canonical AOXC environment identity description used by bootstrap flows.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct CanonicalIdentity {
    family_id: u32,
    chain_name: String,
    network_class: String,
    network_serial: String,
    chain_id: u64,
    network_id: String,
}

/// Canonical bootstrap genesis document.
///
/// This structure intentionally mirrors the AOXC environment-level genesis
/// schema used under `configs/environments/*/genesis.v1.json` rather than the
/// older `chain_num`-based bootstrap format.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct BootstrapGenesisDocument {
    schema_version: u8,
    genesis_kind: String,
    environment: String,
    family_name: String,
    family_code: String,
    identity: CanonicalIdentity,
    consensus: BootstrapConsensusConfig,
    economics: BootstrapEconomicsConfig,
    state: BootstrapStateConfig,
    bindings: BootstrapBindingsConfig,
    integrity: BootstrapIntegrityConfig,
    metadata: BootstrapMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct BootstrapConsensusConfig {
    engine: String,
    mode: String,
    genesis_epoch: u64,
    block_time_ms: u64,
    validator_quorum_policy: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct BootstrapEconomicsConfig {
    native_symbol: String,
    native_decimals: u8,
    initial_treasury: BootstrapTreasuryConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct BootstrapTreasuryConfig {
    account_id: String,
    amount: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct BootstrapStateConfig {
    accounts: Vec<BootstrapAccountRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct BootstrapAccountRecord {
    account_id: String,
    balance: String,
    role: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct BootstrapBindingsConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    accounts_file: Option<String>,
    validators_file: String,
    bootnodes_file: String,
    certificate_file: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct BootstrapIntegrityConfig {
    hash_algorithm: String,
    deterministic_serialization_required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct BootstrapMetadata {
    description: String,
    status: String,
}

#[derive(Debug, Clone, Serialize)]
struct ProfileBootstrapSummary {
    profile: String,
    home_dir: String,
    bind_host: String,
    p2p_port: u16,
    rpc_port: u16,
    prometheus_port: u16,
    chain_id: u64,
    network_id: String,
    operator_fingerprint: String,
    consensus_public_key: String,
    node_height: u64,
}

#[derive(Debug, Clone, Serialize)]
struct DualProfileBootstrapResult {
    output_dir: String,
    profiles: Vec<ProfileBootstrapSummary>,
    launch_hint: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EnvironmentProfile {
    Mainnet,
    Testnet,
    Validation,
    Devnet,
    Localnet,
}

impl EnvironmentProfile {
    fn parse(value: &str) -> Result<Self, AppError> {
        match value.trim().to_ascii_lowercase().as_str() {
            "mainnet" => Ok(Self::Mainnet),
            "testnet" => Ok(Self::Testnet),
            "validation" => Ok(Self::Validation),
            "validator" => Ok(Self::Validation),
            "devnet" => Ok(Self::Devnet),
            "localnet" => Ok(Self::Localnet),
            other => Err(AppError::new(
                ErrorCode::UsageInvalidArguments,
                format!("Unsupported AOXC profile `{}`", other),
            )),
        }
    }

    const fn as_str(self) -> &'static str {
        match self {
            Self::Mainnet => "mainnet",
            Self::Testnet => "testnet",
            Self::Validation => "validation",
            Self::Devnet => "devnet",
            Self::Localnet => "localnet",
        }
    }

    fn identity(self) -> CanonicalIdentity {
        match self {
            Self::Mainnet => CanonicalIdentity {
                family_id: 2626,
                chain_name: "AOXC AKDENIZ".to_string(),
                network_class: "public_mainnet".to_string(),
                network_serial: "2626-001".to_string(),
                chain_id: 2_626_000_001,
                network_id: "aoxc-mainnet-2626-001".to_string(),
            },
            Self::Testnet => CanonicalIdentity {
                family_id: 2626,
                chain_name: "AOXC PUSULA".to_string(),
                network_class: "public_testnet".to_string(),
                network_serial: "2626-002".to_string(),
                chain_id: 2_626_010_001,
                network_id: "aoxc-testnet-2626-002".to_string(),
            },
            Self::Validation => CanonicalIdentity {
                family_id: 2626,
                chain_name: "AOXC MIZAN".to_string(),
                network_class: "validation".to_string(),
                network_serial: "2626-004".to_string(),
                chain_id: 2_626_030_001,
                network_id: "aoxc-validation-2626-004".to_string(),
            },
            Self::Devnet => CanonicalIdentity {
                family_id: 2626,
                chain_name: "AOXC KIVILCIM".to_string(),
                network_class: "devnet".to_string(),
                network_serial: "2626-003".to_string(),
                chain_id: 2_626_020_001,
                network_id: "aoxc-devnet-2626-003".to_string(),
            },
            Self::Localnet => CanonicalIdentity {
                family_id: 2626,
                chain_name: "AOXC LOCALNET ATLAS".to_string(),
                network_class: "localnet".to_string(),
                network_serial: "2626-900".to_string(),
                chain_id: 2_626_900_001,
                network_id: "aoxc-localnet-2626-900".to_string(),
            },
        }
    }

    fn genesis_document(self) -> BootstrapGenesisDocument {
        let identity = self.identity();

        BootstrapGenesisDocument {
            schema_version: 1,
            genesis_kind: "aoxc-genesis-config".to_string(),
            environment: self.as_str().to_string(),
            family_name: "AOXC".to_string(),
            family_code: "aoxc".to_string(),
            identity,
            consensus: BootstrapConsensusConfig {
                engine: "aoxcunity".to_string(),
                mode: "bft".to_string(),
                genesis_epoch: 0,
                block_time_ms: 3_000,
                validator_quorum_policy: "strict-majority".to_string(),
            },
            economics: BootstrapEconomicsConfig {
                native_symbol: "AOXC".to_string(),
                native_decimals: 18,
                initial_treasury: BootstrapTreasuryConfig {
                    account_id: "AOXC_TREASURY_GENESIS".to_string(),
                    amount: "1000000000".to_string(),
                },
            },
            state: BootstrapStateConfig {
                accounts: vec![BootstrapAccountRecord {
                    account_id: "AOXC_TREASURY_GENESIS".to_string(),
                    balance: "1000000000".to_string(),
                    role: "treasury".to_string(),
                }],
            },
            bindings: BootstrapBindingsConfig {
                accounts_file: if self == Self::Localnet {
                    Some("accounts.json".to_string())
                } else {
                    None
                },
                validators_file: "validators.json".to_string(),
                bootnodes_file: "bootnodes.json".to_string(),
                certificate_file: "certificate.json".to_string(),
            },
            integrity: BootstrapIntegrityConfig {
                hash_algorithm: "sha256".to_string(),
                deterministic_serialization_required: true,
            },
            metadata: BootstrapMetadata {
                description: format!("Canonical AOXC {} genesis configuration.", self.as_str()),
                status: "active".to_string(),
            },
        }
    }
}

fn genesis_path() -> Result<PathBuf, AppError> {
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

    let profile_input = arg_value(args, "--profile").unwrap_or_else(|| "validation".to_string());
    let profile = EnvironmentProfile::parse(&profile_input)?;
    let name = arg_value(args, "--name").unwrap_or_else(|| "validator-01".to_string());
    let password = arg_value(args, "--password").ok_or_else(|| {
        AppError::new(
            ErrorCode::UsageInvalidArguments,
            "Missing required flag --password for key bootstrap",
        )
    })?;

    let material = bootstrap_operator_key(&name, profile.as_str(), &password)?;
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
    let profile_input = arg_value(args, "--profile")
        .or_else(|| load_or_init().ok().map(|settings| settings.profile))
        .unwrap_or_else(|| "validation".to_string());

    let profile = EnvironmentProfile::parse(&profile_input)?;
    let genesis = profile.genesis_document();

    let content = serde_json::to_string_pretty(&genesis).map_err(|e| {
        AppError::with_source(
            ErrorCode::OutputEncodingFailed,
            "Failed to encode AOXC genesis document",
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

    let profile_input = arg_value(args, "--profile").unwrap_or_else(|| "validation".to_string());
    let profile = EnvironmentProfile::parse(&profile_input)?;
    let bind_host = arg_value(args, "--bind-host");
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
        .map_err(|e| AppError::new(ErrorCode::ConfigInvalid, e))?;

    persist(&settings)?;
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

fn bootstrap_root() -> PathBuf {
    env::temp_dir().join("aoxc-bootstrap")
}

pub fn cmd_production_bootstrap(args: &[String]) -> Result<(), AppError> {
    let home = resolve_home()?;
    ensure_layout(&home)?;

    let profile = EnvironmentProfile::parse(
        &arg_value(args, "--profile").unwrap_or_else(|| "mainnet".to_string()),
    )?;
    let name = arg_value(args, "--name").unwrap_or_else(|| "validator-01".to_string());
    let password = arg_value(args, "--password").ok_or_else(|| {
        AppError::new(
            ErrorCode::UsageInvalidArguments,
            "Missing required flag --password for production bootstrap",
        )
    })?;
    let bind_host = arg_value(args, "--bind-host");
    let skip_produce_once = has_flag(args, "--skip-produce-once");
    let produce_once_tx = arg_value(args, "--produce-once-tx")
        .unwrap_or_else(|| "bootstrap-mainnet-anchor".to_string());

    let mut settings = build_profile_settings(home.display().to_string(), profile, bind_host)?;
    settings.logging.json = true;
    settings
        .validate()
        .map_err(|e| AppError::new(ErrorCode::ConfigInvalid, e))?;
    persist(&settings)?;

    let genesis = profile.genesis_document();
    let genesis_json = serde_json::to_string_pretty(&genesis).map_err(|e| {
        AppError::with_source(
            ErrorCode::OutputEncodingFailed,
            "Failed to encode production genesis document",
            e,
        )
    })?;
    write_file(&genesis_path()?, &genesis_json)?;

    let _material = bootstrap_operator_key(&name, profile.as_str(), &password)?;
    let operator_fp = operator_fingerprint()?;
    let consensus_pk = consensus_public_key_hex()?;
    let mut node_state = bootstrap_state()?;
    if !skip_produce_once {
        node_state = engine::produce_once(&produce_once_tx)?;
    }

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

    let password = arg_value(args, "--password").ok_or_else(|| {
        AppError::new(
            ErrorCode::UsageInvalidArguments,
            "Missing required flag --password for dual profile bootstrap",
        )
    })?;

    let mainnet_dir = output_dir.join("mainnet");
    let testnet_dir = output_dir.join("testnet");

    let mainnet =
        bootstrap_profile_directory(&mainnet_dir, EnvironmentProfile::Mainnet, &password)?;
    let testnet =
        bootstrap_profile_directory(&testnet_dir, EnvironmentProfile::Testnet, &password)?;

    let result = DualProfileBootstrapResult {
        output_dir: output_dir.display().to_string(),
        profiles: vec![mainnet, testnet],
        launch_hint: "Use the generated profile directories with AOXC runtime launch surfaces.",
    };

    emit_serialized(&result, output_format(args))
}

fn bootstrap_profile_directory(
    output_dir: &Path,
    profile: EnvironmentProfile,
    password: &str,
) -> Result<ProfileBootstrapSummary, AppError> {
    fs::create_dir_all(output_dir).map_err(|e| {
        AppError::with_source(
            ErrorCode::FilesystemIoFailed,
            format!(
                "Failed to create bootstrap output directory {}",
                output_dir.display()
            ),
            e,
        )
    })?;

    let home_dir = output_dir.join("home");
    fs::create_dir_all(&home_dir).map_err(|e| {
        AppError::with_source(
            ErrorCode::FilesystemIoFailed,
            format!("Failed to create home directory {}", home_dir.display()),
            e,
        )
    })?;

    let settings = build_profile_settings(home_dir.display().to_string(), profile, None)?;
    let config_path = home_dir.join("config.json");
    let settings_json = serde_json::to_string_pretty(&settings).map_err(|e| {
        AppError::with_source(
            ErrorCode::OutputEncodingFailed,
            "Failed to encode bootstrap settings",
            e,
        )
    })?;
    write_file(&config_path, &settings_json)?;

    let genesis = profile.genesis_document();
    let genesis_json = serde_json::to_string_pretty(&genesis).map_err(|e| {
        AppError::with_source(
            ErrorCode::OutputEncodingFailed,
            "Failed to encode bootstrap genesis",
            e,
        )
    })?;
    let identity_dir = home_dir.join("identity");
    fs::create_dir_all(&identity_dir).map_err(|e| {
        AppError::with_source(
            ErrorCode::FilesystemIoFailed,
            format!(
                "Failed to create identity directory {}",
                identity_dir.display()
            ),
            e,
        )
    })?;
    write_file(&identity_dir.join("genesis.json"), &genesis_json)?;

    let material = bootstrap_operator_key("validator-01", profile.as_str(), password)?;
    let operator_fp = operator_fingerprint()?;
    let consensus_pk = consensus_public_key_hex()?;
    let node_state = bootstrap_state()?;

    let summary = ProfileBootstrapSummary {
        profile: profile.as_str().to_string(),
        home_dir: home_dir.display().to_string(),
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

    let material_path = home_dir.join("operator-key-bootstrap.json");
    let material_json = serde_json::to_string_pretty(&material).map_err(|e| {
        AppError::with_source(
            ErrorCode::OutputEncodingFailed,
            "Failed to encode operator bootstrap material",
            e,
        )
    })?;
    write_file(&material_path, &material_json)?;

    Ok(summary)
}

fn build_profile_settings(
    home_dir: String,
    profile: EnvironmentProfile,
    bind_host: Option<String>,
) -> Result<Settings, AppError> {
    let mut settings = Settings::default_for_profile(home_dir, profile.as_str())
        .map_err(|e| AppError::new(ErrorCode::ConfigInvalid, e))?;

    if let Some(bind_host) = bind_host {
        settings.network.bind_host = bind_host;
    }

    settings
        .validate()
        .map_err(|e| AppError::new(ErrorCode::ConfigInvalid, e))?;

    Ok(settings)
}

fn load_genesis() -> Result<BootstrapGenesisDocument, AppError> {
    let raw = read_file(&genesis_path()?)?;
    serde_json::from_str::<BootstrapGenesisDocument>(&raw).map_err(|e| {
        AppError::with_source(
            ErrorCode::OutputEncodingFailed,
            "Failed to decode AOXC genesis document",
            e,
        )
    })
}

fn validate_genesis(genesis: &BootstrapGenesisDocument) -> Result<(), AppError> {
    if genesis.schema_version != 1 {
        return Err(AppError::new(
            ErrorCode::ConfigInvalid,
            "Genesis validation failed: schema_version must be 1",
        ));
    }

    if genesis.genesis_kind.trim() != "aoxc-genesis-config" {
        return Err(AppError::new(
            ErrorCode::ConfigInvalid,
            "Genesis validation failed: genesis_kind mismatch",
        ));
    }

    if genesis.environment.trim().is_empty() {
        return Err(AppError::new(
            ErrorCode::ConfigInvalid,
            "Genesis validation failed: environment must not be empty",
        ));
    }

    if genesis.identity.family_id != 2626 {
        return Err(AppError::new(
            ErrorCode::ConfigInvalid,
            "Genesis validation failed: family_id must equal 2626",
        ));
    }

    if genesis.identity.chain_name.trim().is_empty()
        || genesis.identity.network_class.trim().is_empty()
        || genesis.identity.network_serial.trim().is_empty()
        || genesis.identity.network_id.trim().is_empty()
    {
        return Err(AppError::new(
            ErrorCode::ConfigInvalid,
            "Genesis validation failed: identity fields must not be empty",
        ));
    }

    if genesis.identity.chain_id == 0 {
        return Err(AppError::new(
            ErrorCode::ConfigInvalid,
            "Genesis validation failed: chain_id must be non-zero",
        ));
    }

    if genesis.consensus.block_time_ms == 0 {
        return Err(AppError::new(
            ErrorCode::ConfigInvalid,
            "Genesis validation failed: block_time_ms must be non-zero",
        ));
    }

    if genesis.state.accounts.is_empty() {
        return Err(AppError::new(
            ErrorCode::ConfigInvalid,
            "Genesis validation failed: at least one state account is required",
        ));
    }

    if genesis.bindings.validators_file.trim().is_empty()
        || genesis.bindings.bootnodes_file.trim().is_empty()
        || genesis.bindings.certificate_file.trim().is_empty()
    {
        return Err(AppError::new(
            ErrorCode::ConfigInvalid,
            "Genesis validation failed: binding file references must not be empty",
        ));
    }

    Ok(())
}
