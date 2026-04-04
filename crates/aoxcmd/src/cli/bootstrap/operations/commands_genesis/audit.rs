use super::*;

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

pub(in super::super) fn evaluate_consensus_profile_audit(
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
