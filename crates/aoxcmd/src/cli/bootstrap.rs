// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

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
        loader::{init_default, load, persist},
        settings::Settings,
    },
    data_home::{ScopedHomeOverride, ensure_layout, read_file, resolve_home, write_file},
    error::{AppError, ErrorCode},
    keys::manager::{
        bootstrap_operator_key, consensus_public_key_hex, inspect_operator_key,
        operator_fingerprint, rotate_operator_key, verify_operator_key,
    },
    node::lifecycle::bootstrap_state,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
    process,
    time::{SystemTime, UNIX_EPOCH},
};

const DEFAULT_VALIDATOR_GENESIS_BALANCE: &str = "50000000";

fn default_consensus_identity_profile() -> String {
    "hybrid".to_string()
}

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
    #[serde(default = "default_consensus_identity_profile")]
    consensus_identity_profile: String,
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct BootstrapValidatorBindingsDocument {
    schema_version: u8,
    environment: String,
    identity: CanonicalIdentity,
    validators: Vec<BootstrapValidatorBindingRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct BootstrapValidatorBindingRecord {
    validator_id: String,
    display_name: String,
    role: String,
    consensus_key_algorithm: String,
    consensus_public_key_encoding: String,
    consensus_public_key: String,
    consensus_key_fingerprint: String,
    network_key_algorithm: String,
    network_public_key_encoding: String,
    network_public_key: String,
    network_key_fingerprint: String,
    weight: u64,
    status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct BootstrapBootnodesDocument {
    schema_version: u8,
    environment: String,
    identity: CanonicalIdentity,
    bootnodes: Vec<BootstrapBootnodeRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct BootstrapBootnodeRecord {
    node_id: String,
    display_name: String,
    transport_key_algorithm: String,
    transport_public_key_encoding: String,
    transport_public_key: String,
    transport_key_fingerprint: String,
    address: String,
    transport: String,
    status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct BootstrapCertificateDocument {
    schema_version: u8,
    certificate_kind: String,
    environment: String,
    identity: CanonicalIdentity,
    certificate: BootstrapCertificateBody,
    metadata: BootstrapMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct BootstrapCertificateBody {
    status: String,
    issuer: String,
    subject: String,
    certificate_serial: String,
    issued_at: String,
    expires_at: Option<String>,
    fingerprint_sha256: String,
    signature_algorithm: String,
    signature: String,
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

#[derive(Debug, Clone, Serialize)]
struct AddressCreateOutput {
    profile: String,
    validator_name: String,
    validator_account_id: String,
    bundle_fingerprint: String,
    consensus_public_key: String,
    transport_public_key: String,
}

#[derive(Debug, Clone, Serialize)]
struct GenesisTemplateOutput {
    profile: String,
    output_path: String,
    chain_name: String,
    network_id: String,
    validator_quorum_policy: String,
    consensus_identity_profile: String,
    deterministic_serialization_required: bool,
    notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
struct GenesisSecurityAuditReport {
    genesis_path: String,
    profile: String,
    score: u8,
    verdict: &'static str,
    passed: Vec<String>,
    warnings: Vec<String>,
    blockers: Vec<String>,
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
                consensus_identity_profile: default_consensus_identity_profile(),
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

pub fn cmd_genesis_template_advanced(args: &[String]) -> Result<(), AppError> {
    let profile_input = arg_value(args, "--profile").unwrap_or_else(|| "testnet".to_string());
    let profile = EnvironmentProfile::parse(&profile_input)?;

    let mut genesis = profile.genesis_document();
    genesis.metadata.status = "template-advanced".to_string();
    genesis.metadata.description =
        "AOXC advanced genesis template with strict bindings and deterministic integrity controls"
            .to_string();
    genesis.consensus.validator_quorum_policy = "pq-hybrid-threshold-2of3".to_string();
    genesis.consensus.consensus_identity_profile = "post_quantum".to_string();
    genesis.state.accounts.push(BootstrapAccountRecord {
        account_id: "AOXC_GOVERNANCE_TREASURY".to_string(),
        balance: "250000000".to_string(),
        role: "governance".to_string(),
    });

    let output_path = arg_value(args, "--out")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            resolve_home()
                .unwrap_or_else(|_| PathBuf::from("."))
                .join("support")
                .join(format!(
                    "genesis.{}.advanced.example.json",
                    profile.as_str()
                ))
        });

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).map_err(|error| {
            AppError::with_source(
                ErrorCode::FilesystemIoFailed,
                format!(
                    "Failed to create genesis template directory {}",
                    parent.display()
                ),
                error,
            )
        })?;
    }

    write_json_pretty(
        &output_path,
        &genesis,
        "Failed to encode advanced AOXC genesis template",
    )?;

    emit_serialized(
        &GenesisTemplateOutput {
            profile: profile.as_str().to_string(),
            output_path: output_path.display().to_string(),
            chain_name: genesis.identity.chain_name,
            network_id: genesis.identity.network_id,
            validator_quorum_policy: genesis.consensus.validator_quorum_policy,
            consensus_identity_profile: genesis.consensus.consensus_identity_profile,
            deterministic_serialization_required: genesis
                .integrity
                .deterministic_serialization_required,
            notes: vec![
                "Customize validator set, bootnodes, and certificate bindings before use"
                    .to_string(),
                "Run `aoxc genesis-security-audit --profile <profile> --enforce` before promotion"
                    .to_string(),
            ],
        },
        output_format(args),
    )
}

pub fn cmd_genesis_security_audit(args: &[String]) -> Result<(), AppError> {
    let profile_input = arg_value(args, "--profile")
        .or_else(|| load().ok().map(|settings| settings.profile))
        .unwrap_or_else(|| "validation".to_string());
    let profile = EnvironmentProfile::parse(&profile_input)?;

    let path = arg_value(args, "--genesis")
        .map(PathBuf::from)
        .unwrap_or(genesis_path()?);

    let raw = read_file(&path)?;
    let genesis = serde_json::from_str::<BootstrapGenesisDocument>(&raw).map_err(|error| {
        AppError::with_source(
            ErrorCode::ConfigInvalid,
            format!(
                "Failed to decode AOXC genesis document for security audit: {}",
                path.display()
            ),
            error,
        )
    })?;

    let mut passed = Vec::new();
    let mut warnings = Vec::new();
    let mut blockers = Vec::new();

    if genesis.integrity.deterministic_serialization_required {
        passed.push("deterministic-serialization".to_string());
    } else {
        blockers.push("deterministic serialization must be enabled".to_string());
    }

    if !genesis.consensus.validator_quorum_policy.trim().is_empty() {
        passed.push("validator-quorum-policy".to_string());
    } else {
        blockers.push("validator quorum policy must not be empty".to_string());
    }

    let identity_profile = genesis
        .consensus
        .consensus_identity_profile
        .trim()
        .to_ascii_lowercase();
    match identity_profile.as_str() {
        "classical" | "hybrid" | "post_quantum" => {
            passed.push("consensus-identity-profile".to_string());
            if matches!(identity_profile.as_str(), "hybrid" | "post_quantum")
                && !genesis
                    .consensus
                    .validator_quorum_policy
                    .to_ascii_lowercase()
                    .contains("pq")
            {
                warnings.push(
                    "hybrid/post_quantum profile should use a PQ-aware quorum policy label"
                        .to_string(),
                );
            }
        }
        _ => blockers.push(
            "consensus_identity_profile must be one of classical/hybrid/post_quantum".to_string(),
        ),
    }

    let validator_accounts = genesis
        .state
        .accounts
        .iter()
        .filter(|entry| entry.role.eq_ignore_ascii_case("validator"))
        .count();

    if validator_accounts >= 3 {
        passed.push("validator-account-threshold".to_string());
    } else {
        warnings.push(format!(
            "validator account count is {}; testnet recommendation is >= 3",
            validator_accounts
        ));
    }

    if genesis.bindings.validators_file.trim().is_empty()
        || genesis.bindings.bootnodes_file.trim().is_empty()
        || genesis.bindings.certificate_file.trim().is_empty()
    {
        blockers.push("binding file references must be populated".to_string());
    } else {
        passed.push("binding-references".to_string());
    }

    if genesis.environment.eq_ignore_ascii_case(profile.as_str()) {
        passed.push("profile-environment-alignment".to_string());
    } else {
        warnings.push(format!(
            "requested profile `{}` differs from genesis environment `{}`",
            profile.as_str(),
            genesis.environment
        ));
    }

    let score = (passed.len() as u8)
        .saturating_mul(20)
        .saturating_sub((warnings.len() as u8).saturating_mul(5));
    let verdict = if blockers.is_empty() {
        if warnings.is_empty() {
            "pass"
        } else {
            "candidate-with-warnings"
        }
    } else {
        "fail"
    };

    let report = GenesisSecurityAuditReport {
        genesis_path: path.display().to_string(),
        profile: profile.as_str().to_string(),
        score,
        verdict,
        passed,
        warnings,
        blockers,
    };

    if has_flag(args, "--enforce") && report.verdict != "pass" {
        return Err(AppError::new(
            ErrorCode::PolicyGateFailed,
            format!(
                "Genesis security audit enforcement failed with verdict {} (score {})",
                report.verdict, report.score
            ),
        ));
    }

    emit_serialized(&report, output_format(args))
}

pub fn cmd_consensus_profile_audit(args: &[String]) -> Result<(), AppError> {
    let mut mapped = args.to_vec();
    if has_flag(args, "--strict") && !mapped.iter().any(|arg| arg == "--enforce") {
        mapped.push("--enforce".to_string());
    }
    cmd_genesis_security_audit(&mapped)
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

pub fn cmd_genesis_add_account(args: &[String]) -> Result<(), AppError> {
    let account_id = parse_required_text_arg(args, "--account-id", false, "genesis add account")?;
    let balance = parse_required_text_arg(args, "--balance", false, "genesis add account")?;
    let role = parse_required_or_default_text_arg(args, "--role", "user")?;

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
        existing.role = role.clone();
    } else {
        genesis.state.accounts.push(BootstrapAccountRecord {
            account_id: account_id.clone(),
            balance: balance.clone(),
            role: role.clone(),
        });
    }

    persist_genesis(&genesis)?;
    sync_optional_accounts_binding(&genesis)?;

    let mut details = BTreeMap::new();
    details.insert("account_id".to_string(), account_id);
    details.insert("balance".to_string(), balance);
    details.insert("role".to_string(), role);
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

fn bootstrap_profile_directory(
    output_dir: &Path,
    profile: EnvironmentProfile,
    operator_name: &str,
    password: &str,
) -> Result<ProfileBootstrapSummary, AppError> {
    fs::create_dir_all(output_dir).map_err(|error| {
        AppError::with_source(
            ErrorCode::FilesystemIoFailed,
            format!(
                "Failed to create bootstrap output directory {}",
                output_dir.display()
            ),
            error,
        )
    })?;

    let home_dir = output_dir.join("home");
    ensure_layout(&home_dir)?;

    let _home_override = ScopedHomeOverride::install(&home_dir);

    let settings = build_profile_settings(home_dir.display().to_string(), profile, None)?;
    persist(&settings)?;

    let mut genesis = profile.genesis_document();
    let material = bootstrap_operator_key(operator_name, profile.as_str(), password)?;
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
            "Failed to encode bootstrap genesis",
            error,
        )
    })?;
    write_file(&genesis_path()?, &genesis_json)?;

    let operator_fp = operator_fingerprint()?;
    let consensus_pk = consensus_public_key_hex()?;
    let node_state = bootstrap_state()?;

    let material_path = home_dir.join("support").join("operator-key-bootstrap.json");
    let material_json = serde_json::to_string_pretty(&material).map_err(|error| {
        AppError::with_source(
            ErrorCode::OutputEncodingFailed,
            "Failed to encode operator bootstrap material",
            error,
        )
    })?;
    write_file(&material_path, &material_json)?;

    Ok(ProfileBootstrapSummary {
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
    })
}

fn build_profile_settings(
    home_dir: String,
    profile: EnvironmentProfile,
    bind_host: Option<String>,
) -> Result<Settings, AppError> {
    let mut settings = Settings::default_for_profile(home_dir, profile.as_str())
        .map_err(|error| AppError::new(ErrorCode::ConfigInvalid, error))?;

    if let Some(bind_host) = bind_host {
        settings.network.bind_host = bind_host;
    }

    settings
        .validate()
        .map_err(|error| AppError::new(ErrorCode::ConfigInvalid, error))?;

    Ok(settings)
}

fn upsert_validator_account(
    genesis: &mut BootstrapGenesisDocument,
    operator: &crate::keys::material::KeyMaterialSummary,
    balance: &str,
) -> Result<(), AppError> {
    if !is_non_zero_decimal_string(balance) {
        return Err(AppError::new(
            ErrorCode::UsageInvalidArguments,
            "Validator genesis balance must be a non-zero decimal string",
        ));
    }

    let account_id = format!(
        "AOXC_VALIDATOR_{}",
        operator
            .bundle_fingerprint
            .chars()
            .take(16)
            .collect::<String>()
    );

    if let Some(existing) = genesis
        .state
        .accounts
        .iter_mut()
        .find(|record| record.account_id == account_id)
    {
        existing.role = "validator".to_string();
        existing.balance = balance.to_string();
    } else {
        genesis.state.accounts.push(BootstrapAccountRecord {
            account_id,
            balance: balance.to_string(),
            role: "validator".to_string(),
        });
    }

    Ok(())
}

fn materialize_binding_documents(
    genesis: &BootstrapGenesisDocument,
    operator: &crate::keys::material::KeyMaterialSummary,
    settings: &Settings,
) -> Result<(), AppError> {
    let identity_dir = genesis_path()?
        .parent()
        .ok_or_else(|| {
            AppError::new(
                ErrorCode::FilesystemIoFailed,
                "Failed to resolve identity directory for genesis bindings",
            )
        })?
        .to_path_buf();

    let short_fp = operator
        .bundle_fingerprint
        .chars()
        .take(12)
        .collect::<String>()
        .to_ascii_lowercase();
    let validator_id = format!("aoxc-val-{}-{short_fp}", genesis.environment);
    let bootnode_id = format!("aoxc-boot-{}-{short_fp}", genesis.environment);
    let display_name = format!(
        "AOXC {} validator {}",
        genesis.environment.to_ascii_uppercase(),
        &short_fp
    );

    let validators_doc = BootstrapValidatorBindingsDocument {
        schema_version: 2,
        environment: genesis.environment.clone(),
        identity: genesis.identity.clone(),
        validators: vec![BootstrapValidatorBindingRecord {
            validator_id: validator_id.clone(),
            display_name: display_name.clone(),
            role: "validator".to_string(),
            consensus_key_algorithm: "ed25519".to_string(),
            consensus_public_key_encoding: "hex".to_string(),
            consensus_public_key: operator.consensus_public_key.clone(),
            consensus_key_fingerprint: operator.consensus_key_fingerprint.clone(),
            network_key_algorithm: "ed25519".to_string(),
            network_public_key_encoding: "hex".to_string(),
            network_public_key: operator.transport_public_key.clone(),
            network_key_fingerprint: operator.transport_key_fingerprint.clone(),
            weight: 1,
            status: "active".to_string(),
        }],
    };
    write_json_pretty(
        &identity_dir.join(&genesis.bindings.validators_file),
        &validators_doc,
        "Failed to encode validators binding document",
    )?;

    let bootnodes_doc = BootstrapBootnodesDocument {
        schema_version: 2,
        environment: genesis.environment.clone(),
        identity: genesis.identity.clone(),
        bootnodes: vec![BootstrapBootnodeRecord {
            node_id: bootnode_id,
            display_name: format!("AOXC {} bootnode {}", genesis.environment, short_fp),
            transport_key_algorithm: "ed25519".to_string(),
            transport_public_key_encoding: "hex".to_string(),
            transport_public_key: operator.transport_public_key.clone(),
            transport_key_fingerprint: operator.transport_key_fingerprint.clone(),
            address: format!(
                "{}:{}",
                settings.network.bind_host, settings.network.p2p_port
            ),
            transport: "tcp".to_string(),
            status: "active".to_string(),
        }],
    };
    write_json_pretty(
        &identity_dir.join(&genesis.bindings.bootnodes_file),
        &bootnodes_doc,
        "Failed to encode bootnodes binding document",
    )?;

    let issued_at = chrono::Utc::now().to_rfc3339();
    let fingerprint_seed = format!(
        "{}:{}:{}:{}",
        genesis.identity.network_id, validator_id, operator.bundle_fingerprint, issued_at
    );
    let cert_fingerprint = hex::encode(Sha256::digest(fingerprint_seed.as_bytes()));
    let certificate_doc = BootstrapCertificateDocument {
        schema_version: 1,
        certificate_kind: "aoxc-environment-certificate".to_string(),
        environment: genesis.environment.clone(),
        identity: genesis.identity.clone(),
        certificate: BootstrapCertificateBody {
            status: "active".to_string(),
            issuer: "AOXC Bootstrap Authority".to_string(),
            subject: format!("{} Environment Bundle", genesis.identity.chain_name),
            certificate_serial: format!(
                "AOXC-CERT-{}-{}",
                genesis.environment.to_ascii_uppercase(),
                genesis.identity.network_serial
            ),
            issued_at,
            expires_at: None,
            fingerprint_sha256: cert_fingerprint,
            signature_algorithm: "ed25519".to_string(),
            signature: operator.consensus_key_fingerprint.clone(),
        },
        metadata: BootstrapMetadata {
            description: format!(
                "Generated AOXC bootstrap certificate for {}",
                genesis.environment
            ),
            status: "active".to_string(),
        },
    };
    write_json_pretty(
        &identity_dir.join(&genesis.bindings.certificate_file),
        &certificate_doc,
        "Failed to encode certificate binding document",
    )?;

    if let Some(accounts_file) = &genesis.bindings.accounts_file {
        #[derive(Serialize)]
        struct AccountsDoc<'a> {
            schema_version: u8,
            environment: &'a str,
            identity: &'a CanonicalIdentity,
            accounts: &'a [BootstrapAccountRecord],
        }

        let accounts_doc = AccountsDoc {
            schema_version: 1,
            environment: &genesis.environment,
            identity: &genesis.identity,
            accounts: &genesis.state.accounts,
        };

        write_json_pretty(
            &identity_dir.join(accounts_file),
            &accounts_doc,
            "Failed to encode accounts binding document",
        )?;
    }

    Ok(())
}

fn write_json_pretty<T: Serialize>(
    path: &Path,
    payload: &T,
    context: &str,
) -> Result<(), AppError> {
    let encoded = serde_json::to_string_pretty(payload)
        .map_err(|error| AppError::with_source(ErrorCode::OutputEncodingFailed, context, error))?;
    write_file(path, &encoded)
}

fn load_genesis() -> Result<BootstrapGenesisDocument, AppError> {
    let raw = read_file(&genesis_path()?)?;
    serde_json::from_str::<BootstrapGenesisDocument>(&raw).map_err(|error| {
        AppError::with_source(
            ErrorCode::ConfigInvalid,
            "Failed to decode AOXC genesis document",
            error,
        )
    })
}

fn persist_genesis(genesis: &BootstrapGenesisDocument) -> Result<(), AppError> {
    write_json_pretty(
        &genesis_path()?,
        genesis,
        "Failed to encode AOXC genesis document",
    )
}

fn identity_dir_from_genesis(genesis: &BootstrapGenesisDocument) -> Result<PathBuf, AppError> {
    let path = genesis_path()?;
    let root = path.parent().ok_or_else(|| {
        AppError::new(
            ErrorCode::FilesystemIoFailed,
            "Failed to resolve identity directory for genesis bindings",
        )
    })?;

    if genesis.bindings.validators_file.trim().is_empty()
        || genesis.bindings.bootnodes_file.trim().is_empty()
        || genesis.bindings.certificate_file.trim().is_empty()
    {
        return Err(AppError::new(
            ErrorCode::ConfigInvalid,
            "Genesis binding file references must not be empty",
        ));
    }

    Ok(root.to_path_buf())
}

fn sync_optional_accounts_binding(genesis: &BootstrapGenesisDocument) -> Result<(), AppError> {
    let Some(accounts_file) = genesis.bindings.accounts_file.as_deref() else {
        return Ok(());
    };

    #[derive(Serialize)]
    struct AccountsDoc<'a> {
        schema_version: u8,
        environment: &'a str,
        identity: &'a CanonicalIdentity,
        accounts: &'a [BootstrapAccountRecord],
    }

    let doc = AccountsDoc {
        schema_version: 1,
        environment: &genesis.environment,
        identity: &genesis.identity,
        accounts: &genesis.state.accounts,
    };

    let path = identity_dir_from_genesis(genesis)?.join(accounts_file);
    write_json_pretty(&path, &doc, "Failed to encode accounts binding document")
}

fn derive_short_fingerprint(value: &str) -> String {
    let digest = Sha256::digest(value.trim().as_bytes());
    hex::encode(digest)[..16].to_string()
}

fn load_or_default_validators_binding(
    genesis: &BootstrapGenesisDocument,
) -> Result<BootstrapValidatorBindingsDocument, AppError> {
    let path = identity_dir_from_genesis(genesis)?.join(&genesis.bindings.validators_file);
    if !path.exists() {
        return Ok(BootstrapValidatorBindingsDocument {
            schema_version: 2,
            environment: genesis.environment.clone(),
            identity: genesis.identity.clone(),
            validators: Vec::new(),
        });
    }

    let raw = read_file(&path)?;
    serde_json::from_str::<BootstrapValidatorBindingsDocument>(&raw).map_err(|error| {
        AppError::with_source(
            ErrorCode::ConfigInvalid,
            format!(
                "Failed to decode validators binding document: {}",
                path.display()
            ),
            error,
        )
    })
}

fn upsert_validator_binding(
    doc: &mut BootstrapValidatorBindingsDocument,
    record: BootstrapValidatorBindingRecord,
) {
    if let Some(existing) = doc
        .validators
        .iter_mut()
        .find(|existing| existing.validator_id == record.validator_id)
    {
        *existing = record;
    } else {
        doc.validators.push(record);
    }
}

fn persist_validators_binding(
    genesis: &BootstrapGenesisDocument,
    doc: &BootstrapValidatorBindingsDocument,
) -> Result<(), AppError> {
    let path = identity_dir_from_genesis(genesis)?.join(&genesis.bindings.validators_file);
    write_json_pretty(&path, doc, "Failed to encode validators binding document")
}

fn load_or_default_bootnodes_binding(
    genesis: &BootstrapGenesisDocument,
) -> Result<BootstrapBootnodesDocument, AppError> {
    let path = identity_dir_from_genesis(genesis)?.join(&genesis.bindings.bootnodes_file);
    if !path.exists() {
        return Ok(BootstrapBootnodesDocument {
            schema_version: 2,
            environment: genesis.environment.clone(),
            identity: genesis.identity.clone(),
            bootnodes: Vec::new(),
        });
    }

    let raw = read_file(&path)?;
    serde_json::from_str::<BootstrapBootnodesDocument>(&raw).map_err(|error| {
        AppError::with_source(
            ErrorCode::ConfigInvalid,
            format!(
                "Failed to decode bootnodes binding document: {}",
                path.display()
            ),
            error,
        )
    })
}

fn upsert_bootnode_binding(doc: &mut BootstrapBootnodesDocument, record: BootstrapBootnodeRecord) {
    if let Some(existing) = doc
        .bootnodes
        .iter_mut()
        .find(|existing| existing.node_id == record.node_id)
    {
        *existing = record;
    } else {
        doc.bootnodes.push(record);
    }
}

fn persist_bootnodes_binding(
    genesis: &BootstrapGenesisDocument,
    doc: &BootstrapBootnodesDocument,
) -> Result<(), AppError> {
    let path = identity_dir_from_genesis(genesis)?.join(&genesis.bindings.bootnodes_file);
    write_json_pretty(&path, doc, "Failed to encode bootnodes binding document")
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

    if genesis.family_name.trim().is_empty() || genesis.family_code.trim().is_empty() {
        return Err(AppError::new(
            ErrorCode::ConfigInvalid,
            "Genesis validation failed: family identity fields must not be empty",
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

    if genesis.environment.trim() != genesis.identity.network_class.trim()
        && !matches!(
            (
                genesis.environment.trim(),
                genesis.identity.network_class.trim()
            ),
            ("mainnet", "public_mainnet") | ("testnet", "public_testnet")
        )
    {
        return Err(AppError::new(
            ErrorCode::ConfigInvalid,
            "Genesis validation failed: environment and network_class are inconsistent",
        ));
    }

    if genesis.consensus.block_time_ms == 0 {
        return Err(AppError::new(
            ErrorCode::ConfigInvalid,
            "Genesis validation failed: block_time_ms must be non-zero",
        ));
    }

    if genesis.economics.native_symbol.trim().is_empty() {
        return Err(AppError::new(
            ErrorCode::ConfigInvalid,
            "Genesis validation failed: native_symbol must not be empty",
        ));
    }

    if !is_non_zero_decimal_string(&genesis.economics.initial_treasury.amount) {
        return Err(AppError::new(
            ErrorCode::ConfigInvalid,
            "Genesis validation failed: treasury amount must be a non-zero decimal string",
        ));
    }

    if genesis.state.accounts.is_empty() {
        return Err(AppError::new(
            ErrorCode::ConfigInvalid,
            "Genesis validation failed: at least one state account is required",
        ));
    }

    let mut seen_accounts = BTreeSet::new();
    for account in &genesis.state.accounts {
        if account.account_id.trim().is_empty()
            || account.role.trim().is_empty()
            || !is_decimal_string(&account.balance)
        {
            return Err(AppError::new(
                ErrorCode::ConfigInvalid,
                "Genesis validation failed: account fields are invalid",
            ));
        }

        if !seen_accounts.insert(account.account_id.clone()) {
            return Err(AppError::new(
                ErrorCode::ConfigInvalid,
                "Genesis validation failed: duplicate account_id detected",
            ));
        }
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

    if genesis.integrity.hash_algorithm.trim() != "sha256" {
        return Err(AppError::new(
            ErrorCode::ConfigInvalid,
            "Genesis validation failed: hash_algorithm must equal sha256",
        ));
    }

    if !genesis.integrity.deterministic_serialization_required {
        return Err(AppError::new(
            ErrorCode::ConfigInvalid,
            "Genesis validation failed: deterministic serialization must be required",
        ));
    }

    Ok(())
}

#[derive(Debug, Deserialize)]
struct ValidatorSetDocument {
    validators: Vec<ValidatorRecord>,
}

#[derive(Debug, Deserialize)]
struct ValidatorRecord {
    validator_id: String,
    consensus_public_key: String,
    status: String,
}

fn validate_binding_files(genesis: &BootstrapGenesisDocument) -> Result<(), AppError> {
    let genesis_file = genesis_path()?;
    let root = genesis_file.parent().ok_or_else(|| {
        AppError::new(
            ErrorCode::FilesystemIoFailed,
            "Genesis validation failed: identity directory is not accessible",
        )
    })?;

    let validators_path = root.join(&genesis.bindings.validators_file);
    let validators_raw = read_file(&validators_path)?;
    let validators_doc: ValidatorSetDocument =
        serde_json::from_str(&validators_raw).map_err(|error| {
            AppError::with_source(
                ErrorCode::ConfigInvalid,
                format!(
                    "Genesis validation failed: validators binding is not valid JSON: {}",
                    validators_path.display()
                ),
                error,
            )
        })?;

    if validators_doc.validators.is_empty() {
        return Err(AppError::new(
            ErrorCode::ConfigInvalid,
            format!(
                "Genesis validation failed: validators file must contain at least one validator: {}",
                validators_path.display()
            ),
        ));
    }

    for validator in &validators_doc.validators {
        if validator.validator_id.trim().is_empty() {
            return Err(AppError::new(
                ErrorCode::ConfigInvalid,
                "Genesis validation failed: validator_id must not be empty",
            ));
        }

        if validator.consensus_public_key.trim().is_empty()
            || validator
                .consensus_public_key
                .to_ascii_lowercase()
                .contains("pending_real_value")
        {
            return Err(AppError::new(
                ErrorCode::ConfigInvalid,
                format!(
                    "Genesis validation failed: validator consensus key is empty or placeholder for {}",
                    validator.validator_id
                ),
            ));
        }

        if validator.status.trim() != "active" {
            return Err(AppError::new(
                ErrorCode::ConfigInvalid,
                format!(
                    "Genesis validation failed: validator {} is not active",
                    validator.validator_id
                ),
            ));
        }
    }

    let bootnodes_path = root.join(&genesis.bindings.bootnodes_file);
    let bootnodes_raw = read_file(&bootnodes_path)?;
    let bootnodes_json: Value = serde_json::from_str(&bootnodes_raw).map_err(|error| {
        AppError::with_source(
            ErrorCode::ConfigInvalid,
            format!(
                "Genesis validation failed: bootnodes binding is not valid JSON: {}",
                bootnodes_path.display()
            ),
            error,
        )
    })?;

    if bootnodes_json
        .get("bootnodes")
        .and_then(Value::as_array)
        .is_none_or(|entries| entries.is_empty())
    {
        return Err(AppError::new(
            ErrorCode::ConfigInvalid,
            format!(
                "Genesis validation failed: bootnodes file must contain at least one bootnode: {}",
                bootnodes_path.display()
            ),
        ));
    }

    let certificate_path = root.join(&genesis.bindings.certificate_file);
    let certificate_raw = read_file(&certificate_path)?;
    let certificate_json: Value = serde_json::from_str(&certificate_raw).map_err(|error| {
        AppError::with_source(
            ErrorCode::ConfigInvalid,
            format!(
                "Genesis validation failed: certificate binding is not valid JSON: {}",
                certificate_path.display()
            ),
            error,
        )
    })?;

    if certificate_json
        .as_object()
        .is_none_or(|object| object.is_empty())
    {
        return Err(AppError::new(
            ErrorCode::ConfigInvalid,
            format!(
                "Genesis validation failed: certificate file is empty: {}",
                certificate_path.display()
            ),
        ));
    }

    if let Some(accounts_file) = &genesis.bindings.accounts_file {
        let accounts_path = root.join(accounts_file);
        let accounts_raw = read_file(&accounts_path)?;
        let accounts_json: Value = serde_json::from_str(&accounts_raw).map_err(|error| {
            AppError::with_source(
                ErrorCode::ConfigInvalid,
                format!(
                    "Genesis validation failed: accounts binding is not valid JSON: {}",
                    accounts_path.display()
                ),
                error,
            )
        })?;

        if accounts_json
            .get("accounts")
            .and_then(Value::as_array)
            .is_none_or(|entries| entries.is_empty())
        {
            return Err(AppError::new(
                ErrorCode::ConfigInvalid,
                format!(
                    "Genesis validation failed: accounts file must contain at least one account: {}",
                    accounts_path.display()
                ),
            ));
        }
    }

    Ok(())
}

fn parse_required_text_arg(
    args: &[String],
    flag: &str,
    lowercase: bool,
    context: &str,
) -> Result<String, AppError> {
    let value = arg_value(args, flag).ok_or_else(|| {
        AppError::new(
            ErrorCode::UsageInvalidArguments,
            format!("Missing required flag {flag} for {context}"),
        )
    })?;

    normalize_text(&value, lowercase).ok_or_else(|| {
        AppError::new(
            ErrorCode::UsageInvalidArguments,
            format!("Flag {flag} must not be blank"),
        )
    })
}

fn parse_required_or_default_text_arg(
    args: &[String],
    flag: &str,
    default: &str,
) -> Result<String, AppError> {
    match arg_value(args, flag) {
        Some(value) => normalize_text(&value, false).ok_or_else(|| {
            AppError::new(
                ErrorCode::UsageInvalidArguments,
                format!("Flag {flag} must not be blank"),
            )
        }),
        None => Ok(default.to_string()),
    }
}

fn parse_optional_text_arg(args: &[String], flag: &str, lowercase: bool) -> Option<String> {
    arg_value(args, flag).and_then(|value| normalize_text(&value, lowercase))
}

fn normalize_text(value: &str, lowercase: bool) -> Option<String> {
    let normalized = value.split_whitespace().collect::<Vec<_>>().join(" ");
    if normalized.is_empty() {
        return None;
    }

    if lowercase {
        Some(normalized.to_ascii_lowercase())
    } else {
        Some(normalized)
    }
}

fn is_decimal_string(value: &str) -> bool {
    let trimmed = value.trim();
    !trimmed.is_empty() && trimmed.chars().all(|ch| ch.is_ascii_digit())
}

fn is_non_zero_decimal_string(value: &str) -> bool {
    is_decimal_string(value) && value.trim().chars().any(|ch| ch != '0')
}

#[cfg(test)]
mod tests {
    use super::{
        BootstrapBootnodeRecord, BootstrapBootnodesDocument, BootstrapValidatorBindingRecord,
        BootstrapValidatorBindingsDocument, CanonicalIdentity, derive_short_fingerprint,
        upsert_bootnode_binding, upsert_validator_binding,
    };

    fn canonical_identity() -> CanonicalIdentity {
        CanonicalIdentity {
            family_id: 2626,
            chain_name: "AOXC TEST".to_string(),
            network_class: "validation".to_string(),
            network_serial: "2626-004".to_string(),
            chain_id: 2_626_030_001,
            network_id: "aoxc-validation-2626-004".to_string(),
        }
    }

    #[test]
    fn derive_short_fingerprint_returns_16_hex_characters() {
        let value = derive_short_fingerprint("validator-01");
        assert_eq!(value.len(), 16);
        assert!(value.chars().all(|ch| ch.is_ascii_hexdigit()));
    }

    #[test]
    fn upsert_validator_binding_replaces_existing_record() {
        let mut doc = BootstrapValidatorBindingsDocument {
            schema_version: 2,
            environment: "validation".to_string(),
            identity: canonical_identity(),
            validators: vec![BootstrapValidatorBindingRecord {
                validator_id: "val-01".to_string(),
                display_name: "Validator 01".to_string(),
                role: "validator".to_string(),
                consensus_key_algorithm: "ed25519".to_string(),
                consensus_public_key_encoding: "hex".to_string(),
                consensus_public_key: "abc".to_string(),
                consensus_key_fingerprint: "fp1".to_string(),
                network_key_algorithm: "ed25519".to_string(),
                network_public_key_encoding: "hex".to_string(),
                network_public_key: "def".to_string(),
                network_key_fingerprint: "fp2".to_string(),
                weight: 1,
                status: "active".to_string(),
            }],
        };

        upsert_validator_binding(
            &mut doc,
            BootstrapValidatorBindingRecord {
                validator_id: "val-01".to_string(),
                display_name: "Validator One".to_string(),
                role: "validator".to_string(),
                consensus_key_algorithm: "ed25519".to_string(),
                consensus_public_key_encoding: "hex".to_string(),
                consensus_public_key: "new-consensus".to_string(),
                consensus_key_fingerprint: "new-fp1".to_string(),
                network_key_algorithm: "ed25519".to_string(),
                network_public_key_encoding: "hex".to_string(),
                network_public_key: "new-network".to_string(),
                network_key_fingerprint: "new-fp2".to_string(),
                weight: 2,
                status: "active".to_string(),
            },
        );

        assert_eq!(doc.validators.len(), 1);
        assert_eq!(doc.validators[0].display_name, "Validator One");
        assert_eq!(doc.validators[0].consensus_public_key, "new-consensus");
        assert_eq!(doc.validators[0].weight, 2);
    }

    #[test]
    fn upsert_bootnode_binding_replaces_existing_record() {
        let mut doc = BootstrapBootnodesDocument {
            schema_version: 2,
            environment: "validation".to_string(),
            identity: canonical_identity(),
            bootnodes: vec![BootstrapBootnodeRecord {
                node_id: "boot-01".to_string(),
                display_name: "Boot 01".to_string(),
                transport_key_algorithm: "ed25519".to_string(),
                transport_public_key_encoding: "hex".to_string(),
                transport_public_key: "pk1".to_string(),
                transport_key_fingerprint: "fp1".to_string(),
                address: "127.0.0.1:39001".to_string(),
                transport: "tcp".to_string(),
                status: "active".to_string(),
            }],
        };

        upsert_bootnode_binding(
            &mut doc,
            BootstrapBootnodeRecord {
                node_id: "boot-01".to_string(),
                display_name: "Boot Updated".to_string(),
                transport_key_algorithm: "ed25519".to_string(),
                transport_public_key_encoding: "hex".to_string(),
                transport_public_key: "pk2".to_string(),
                transport_key_fingerprint: "fp2".to_string(),
                address: "10.0.0.1:39001".to_string(),
                transport: "tcp".to_string(),
                status: "active".to_string(),
            },
        );

        assert_eq!(doc.bootnodes.len(), 1);
        assert_eq!(doc.bootnodes[0].display_name, "Boot Updated");
        assert_eq!(doc.bootnodes[0].address, "10.0.0.1:39001");
    }
}
