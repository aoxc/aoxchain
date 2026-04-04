// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use super::types::*;
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

fn evaluate_consensus_profile_audit(
    genesis: &BootstrapGenesisDocument,
    profile: EnvironmentProfile,
    genesis_path: String,
) -> ConsensusProfileAuditReport {
    let mut passed = Vec::new();
    let mut warnings = Vec::new();
    let mut blockers = Vec::new();

    let consensus_profile = genesis
        .consensus
        .consensus_identity_profile
        .trim()
        .to_ascii_lowercase();

    match consensus_profile.as_str() {
        "hybrid" | "pq-hybrid" => {
            passed.push("consensus-identity-profile-hybrid".to_string());
        }
        "pq" | "pq-only" | "post-quantum" => {
            passed.push("consensus-identity-profile-pq".to_string());
        }
        "classical" => {
            warnings.push(
                "classical consensus profile should only be used for controlled migration windows"
                    .to_string(),
            );
        }
        _ => {
            blockers.push(format!(
                "unsupported consensus identity profile `{}`",
                genesis.consensus.consensus_identity_profile
            ));
        }
    }

    if matches!(
        profile,
        EnvironmentProfile::Mainnet | EnvironmentProfile::Testnet
    ) && consensus_profile == "classical"
    {
        blockers.push(
            "mainnet/testnet profiles must not run with classical-only consensus identity profile"
                .to_string(),
        );
    }
    if matches!(
        profile,
        EnvironmentProfile::Mainnet | EnvironmentProfile::Testnet
    ) {
        let normalized_quorum = genesis
            .consensus
            .validator_quorum_policy
            .trim()
            .to_ascii_lowercase();
        if normalized_quorum.contains("pq") || normalized_quorum.contains("hybrid") {
            passed.push("validator-quorum-policy-pq-hybrid".to_string());
        } else {
            blockers.push(format!(
                "mainnet/testnet profiles must declare a pq/hybrid quorum policy; got `{}`",
                genesis.consensus.validator_quorum_policy
            ));
        }
    }

    let expected_identity = profile.identity();
    if genesis.identity.network_class == expected_identity.network_class {
        passed.push("identity-network-class-alignment".to_string());
    } else {
        blockers.push(format!(
            "identity network_class `{}` does not match expected `{}` for profile `{}`",
            genesis.identity.network_class,
            expected_identity.network_class,
            profile.as_str()
        ));
    }

    if genesis.identity.network_id == expected_identity.network_id {
        passed.push("identity-network-id-alignment".to_string());
    } else {
        blockers.push(format!(
            "identity network_id `{}` does not match expected `{}` for profile `{}`",
            genesis.identity.network_id,
            expected_identity.network_id,
            profile.as_str()
        ));
    }

    if genesis.identity.chain_id == expected_identity.chain_id {
        passed.push("identity-chain-id-alignment".to_string());
    } else {
        blockers.push(format!(
            "identity chain_id `{}` does not match expected `{}` for profile `{}`",
            genesis.identity.chain_id,
            expected_identity.chain_id,
            profile.as_str()
        ));
    }

    if genesis
        .consensus
        .engine
        .trim()
        .eq_ignore_ascii_case("aoxcunity")
    {
        passed.push("consensus-engine".to_string());
    } else {
        blockers.push(format!(
            "unsupported consensus engine `{}`; expected `aoxcunity`",
            genesis.consensus.engine
        ));
    }

    if genesis.consensus.mode.trim().eq_ignore_ascii_case("bft") {
        passed.push("consensus-mode".to_string());
    } else {
        blockers.push(format!(
            "unsupported consensus mode `{}`; expected `bft`",
            genesis.consensus.mode
        ));
    }

    if genesis
        .integrity
        .hash_algorithm
        .trim()
        .eq_ignore_ascii_case("sha256")
    {
        passed.push("integrity-hash-algorithm".to_string());
    } else {
        blockers.push(format!(
            "unsupported integrity hash algorithm `{}`; expected `sha256`",
            genesis.integrity.hash_algorithm
        ));
    }

    let expected_identity = profile.identity();
    if genesis.identity.network_class == expected_identity.network_class {
        passed.push("identity-network-class-alignment".to_string());
    } else {
        blockers.push(format!(
            "identity network_class `{}` does not match expected `{}` for profile `{}`",
            genesis.identity.network_class,
            expected_identity.network_class,
            profile.as_str()
        ));
    }

    if genesis.identity.network_id == expected_identity.network_id {
        passed.push("identity-network-id-alignment".to_string());
    } else {
        blockers.push(format!(
            "identity network_id `{}` does not match expected `{}` for profile `{}`",
            genesis.identity.network_id,
            expected_identity.network_id,
            profile.as_str()
        ));
    }

    if genesis.identity.chain_id == expected_identity.chain_id {
        passed.push("identity-chain-id-alignment".to_string());
    } else {
        blockers.push(format!(
            "identity chain_id `{}` does not match expected `{}` for profile `{}`",
            genesis.identity.chain_id,
            expected_identity.chain_id,
            profile.as_str()
        ));
    }

    if genesis
        .consensus
        .engine
        .trim()
        .eq_ignore_ascii_case("aoxcunity")
    {
        passed.push("consensus-engine".to_string());
    } else {
        blockers.push(format!(
            "unsupported consensus engine `{}`; expected `aoxcunity`",
            genesis.consensus.engine
        ));
    }

    if genesis.consensus.mode.trim().eq_ignore_ascii_case("bft") {
        passed.push("consensus-mode".to_string());
    } else {
        blockers.push(format!(
            "unsupported consensus mode `{}`; expected `bft`",
            genesis.consensus.mode
        ));
    }

    if genesis
        .integrity
        .hash_algorithm
        .trim()
        .eq_ignore_ascii_case("sha256")
    {
        passed.push("integrity-hash-algorithm".to_string());
    } else {
        blockers.push(format!(
            "unsupported integrity hash algorithm `{}`; expected `sha256`",
            genesis.integrity.hash_algorithm
        ));
    }

    if genesis
        .integrity
        .hash_algorithm
        .trim()
        .eq_ignore_ascii_case("sha256")
    {
        passed.push("integrity-hash-algorithm".to_string());
    } else {
        blockers.push(format!(
            "unsupported integrity hash algorithm `{}`; expected `sha256`",
            genesis.integrity.hash_algorithm
        ));
    }

    let block_time_in_envelope =
        genesis.consensus.block_time_ms >= 500 && genesis.consensus.block_time_ms <= 15_000;
    if block_time_in_envelope {
        passed.push("block-time-envelope".to_string());
    } else {
        let message = format!(
            "block_time_ms={} is outside recommended envelope [500, 15000]",
            genesis.consensus.block_time_ms
        );
        if matches!(
            profile,
            EnvironmentProfile::Mainnet | EnvironmentProfile::Testnet
        ) {
            blockers.push(format!(
                "mainnet/testnet profiles require block_time_ms in [500, 15000]: {message}"
            ));
        } else {
            warnings.push(message);
        }
    }

    if genesis.consensus.validator_quorum_policy.trim().is_empty() {
        blockers.push("validator quorum policy must not be empty".to_string());
    } else {
        passed.push("validator-quorum-policy".to_string());
    }

    if genesis.integrity.deterministic_serialization_required {
        passed.push("deterministic-serialization".to_string());
    } else {
        blockers.push("deterministic serialization must be enabled".to_string());
    }

    if genesis.environment.eq_ignore_ascii_case(profile.as_str()) {
        passed.push("profile-environment-alignment".to_string());
    } else {
        let message = format!(
            "requested profile `{}` differs from genesis environment `{}`",
            profile.as_str(),
            genesis.environment
        );
        if matches!(
            profile,
            EnvironmentProfile::Mainnet | EnvironmentProfile::Testnet
        ) {
            blockers.push(format!(
                "mainnet/testnet profiles require strict profile-environment alignment: {message}"
            ));
        } else {
            warnings.push(message);
        }
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

    ConsensusProfileAuditReport {
        genesis_path,
        profile: profile.as_str().to_string(),
        consensus_identity_profile: genesis.consensus.consensus_identity_profile.clone(),
        score,
        verdict,
        passed,
        warnings,
        blockers,
    }
}

pub fn cmd_consensus_profile_audit(args: &[String]) -> Result<(), AppError> {
    let profile_input = arg_value(args, "--profile")
        .or_else(|| load().ok().map(|settings| settings.profile))
        .unwrap_or_else(|| "validation".to_string());
    let profile = EnvironmentProfile::parse(&profile_input)?;

    let path = if let Some(path) = arg_value(args, "--genesis") {
        PathBuf::from(path)
    } else {
        genesis_path()?
    };
    let path_display = path.display().to_string();

    let raw = read_file(&path)?;
    let genesis = serde_json::from_str::<BootstrapGenesisDocument>(&raw).map_err(|error| {
        AppError::with_source(
            ErrorCode::ConfigInvalid,
            format!(
                "Failed to decode AOXC genesis document for consensus profile audit: {}",
                path.display()
            ),
            error,
        )
    })?;

    let report = evaluate_consensus_profile_audit(&genesis, profile, path_display);

    if has_flag(args, "--enforce") && report.verdict != "pass" {
        return Err(AppError::new(
            ErrorCode::PolicyGateFailed,
            format!(
                "Consensus profile audit enforcement failed with verdict {} (score {})",
                report.verdict, report.score
            ),
        ));
    }

    emit_serialized(&report, output_format(args))
}

pub fn consensus_profile_gate_status(
    genesis_override: Option<&Path>,
    profile_override: Option<&str>,
) -> Result<ConsensusProfileGateStatus, String> {
    let profile_input = profile_override
        .map(str::to_string)
        .or_else(|| load().ok().map(|settings| settings.profile))
        .unwrap_or_else(|| "validation".to_string());
    let profile = EnvironmentProfile::parse(&profile_input)
        .map_err(|error| format!("invalid profile `{profile_input}`: {error}"))?;

    let path = if let Some(path) = genesis_override {
        path.to_path_buf()
    } else {
        genesis_path().map_err(|error| format!("failed to resolve genesis path: {error}"))?
    };
    let path_display = path.display().to_string();

    let raw = read_file(&path)
        .map_err(|error| format!("failed to read genesis `{path_display}`: {error}"))?;
    let genesis = serde_json::from_str::<BootstrapGenesisDocument>(&raw)
        .map_err(|error| format!("failed to decode genesis `{path_display}`: {error}"))?;

    let report = evaluate_consensus_profile_audit(&genesis, profile, path_display.clone());
    let strict_warning_free = !matches!(
        profile,
        EnvironmentProfile::Mainnet | EnvironmentProfile::Testnet
    ) || report.warnings.is_empty();
    let passed = report.blockers.is_empty() && strict_warning_free;
    let detail = format!(
        "profile={}, consensus_profile={}, score={}, verdict={}",
        report.profile, report.consensus_identity_profile, report.score, report.verdict
    );

    Ok(ConsensusProfileGateStatus {
        passed,
        detail,
        verdict: report.verdict.to_string(),
        blockers: report.blockers,
        profile: report.profile,
        genesis_path: path_display,
    })
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

mod helpers_args;
mod helpers_core;

use helpers_args::*;
use helpers_core::*;

#[cfg(test)]
include!("helpers_tests.rs");
