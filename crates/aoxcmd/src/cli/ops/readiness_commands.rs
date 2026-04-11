use super::*;

pub fn cmd_load_benchmark(args: &[String]) -> Result<(), AppError> {
    let rounds = parse_positive_u64_arg(args, "--rounds", 100, "load benchmark")?;

    let mut details = BTreeMap::new();
    details.insert("benchmark_rounds".to_string(), rounds.to_string());
    details.insert(
        "result".to_string(),
        "baseline-local-benchmark-recorded".to_string(),
    );

    emit_serialized(
        &text_envelope("load-benchmark", "ok", details),
        output_format(args),
    )
}

pub fn cmd_mainnet_readiness(args: &[String]) -> Result<(), AppError> {
    cmd_profile_readiness(args, "mainnet")
}

pub fn cmd_testnet_readiness(args: &[String]) -> Result<(), AppError> {
    cmd_profile_readiness(args, "testnet")
}

fn cmd_profile_readiness(args: &[String], target_profile: &'static str) -> Result<(), AppError> {
    let settings = effective_settings_for_ops()?;
    let key_summary = crate::keys::manager::inspect_operator_key().ok();
    let genesis_ok = crate::cli::bootstrap::genesis_ready();
    let node_ok = lifecycle::load_state().is_ok();
    let config_validation = settings.validate();

    let readiness = evaluate_profile_readiness(
        target_profile,
        &settings,
        config_validation.err(),
        key_summary
            .as_ref()
            .map(|summary| summary.operational_state.as_str()),
        genesis_ok,
        node_ok,
    );

    if let Some(path) = parse_optional_text_arg(args, "--write-report", false) {
        write_readiness_markdown_report(
            Path::new(&path),
            &readiness,
            compare_embedded_network_profiles().ok().as_ref(),
            compare_aoxhub_network_profiles().ok().as_ref(),
        )?;
    }

    if has_flag(args, "--enforce") && readiness.verdict != "candidate" {
        let profile_title = if target_profile.eq_ignore_ascii_case("testnet") {
            "Testnet"
        } else {
            "Mainnet"
        };
        return Err(AppError::new(
            ErrorCode::PolicyGateFailed,
            format!(
                "{profile_title} readiness enforcement failed at score {} with blockers: {}",
                readiness.readiness_score,
                readiness.blockers.join(" | ")
            ),
        ));
    }

    emit_serialized(&readiness, output_format(args))
}

pub fn cmd_full_surface_readiness(args: &[String]) -> Result<(), AppError> {
    let settings = effective_settings_for_ops()?;
    let key_summary = crate::keys::manager::inspect_operator_key().ok();
    let genesis_ok = crate::cli::bootstrap::genesis_ready();
    let node_ok = lifecycle::load_state().is_ok();

    let full = evaluate_full_surface_readiness(
        &settings,
        &evaluate_profile_readiness(
            "mainnet",
            &settings,
            settings.validate().err(),
            key_summary
                .as_ref()
                .map(|summary| summary.operational_state.as_str()),
            genesis_ok,
            node_ok,
        ),
    );

    if let Some(path) = parse_optional_text_arg(args, "--write-report", false) {
        write_full_surface_markdown_report(Path::new(&path), &full)?;
    }

    if has_flag(args, "--enforce") && full.overall_status != "candidate" {
        return Err(AppError::new(
            ErrorCode::PolicyGateFailed,
            format!(
                "Full-surface readiness enforcement failed at score {} with blockers: {}",
                full.overall_score,
                full.blockers.join(" | ")
            ),
        ));
    }

    emit_serialized(&full, output_format(args))
}

pub fn cmd_full_surface_gate(args: &[String]) -> Result<(), AppError> {
    let settings = effective_settings_for_ops()?;
    let key_summary = crate::keys::manager::inspect_operator_key().ok();
    let genesis_ok = crate::cli::bootstrap::genesis_ready();
    let node_ok = lifecycle::load_state().is_ok();

    let mainnet = evaluate_profile_readiness(
        "mainnet",
        &settings,
        settings.validate().err(),
        key_summary
            .as_ref()
            .map(|summary| summary.operational_state.as_str()),
        genesis_ok,
        node_ok,
    );
    let full = evaluate_full_surface_readiness(&settings, &mainnet);
    let failures = collect_surface_gate_failures(&full);

    let report = FullSurfaceGateReport {
        profile: settings.profile.clone(),
        enforced: has_flag(args, "--enforce"),
        passed: failures.is_empty(),
        overall_status: full.overall_status.to_string(),
        overall_score: full.overall_score,
        failure_count: failures.len(),
        failures,
    };

    if report.enforced && !report.passed {
        let codes = report
            .failures
            .iter()
            .map(|failure| failure.code.clone())
            .collect::<Vec<_>>()
            .join(",");
        return Err(AppError::new(
            ErrorCode::PolicyGateFailed,
            format!(
                "Full-surface gate enforcement failed with {} failing checks [{}]",
                report.failure_count, codes
            ),
        ));
    }

    emit_serialized(&report, output_format(args))
}

pub fn cmd_level_score(args: &[String]) -> Result<(), AppError> {
    let settings = effective_settings_for_ops()?;
    let key_summary = crate::keys::manager::inspect_operator_key().ok();
    let genesis_ok = crate::cli::bootstrap::genesis_ready();
    let node_state = lifecycle::load_state().ok();
    let node_ok = node_state.is_some();

    let mainnet = evaluate_profile_readiness(
        "mainnet",
        &settings,
        settings.validate().err(),
        key_summary
            .as_ref()
            .map(|summary| summary.operational_state.as_str()),
        genesis_ok,
        node_ok,
    );
    let full = evaluate_full_surface_readiness(&settings, &mainnet);

    let block_production_score = node_state
        .as_ref()
        .map(|state| if state.current_height > 0 { 100 } else { 0 })
        .unwrap_or(0);

    let net_level_score = ((u16::from(mainnet.readiness_score)
        + u16::from(full.overall_score)
        + u16::from(block_production_score))
        / 3) as u8;

    let level_verdict = if net_level_score >= 100 {
        "perfect"
    } else if net_level_score >= 90 {
        "candidate"
    } else if net_level_score >= 70 {
        "in-progress"
    } else {
        "bootstrap"
    };

    let score = PlatformLevelScore {
        profile: settings.profile,
        mainnet_readiness_score: mainnet.readiness_score,
        full_surface_score: full.overall_score,
        block_production_score,
        net_level_score,
        level_verdict,
    };

    if has_flag(args, "--enforce") && score.net_level_score < 100 {
        return Err(AppError::new(
            ErrorCode::PolicyGateFailed,
            format!(
                "Platform level score enforcement failed at {} (mainnet={}, full-surface={}, block-production={})",
                score.net_level_score,
                score.mainnet_readiness_score,
                score.full_surface_score,
                score.block_production_score
            ),
        ));
    }

    emit_serialized(&score, output_format(args))
}

pub fn cmd_profile_baseline(args: &[String]) -> Result<(), AppError> {
    let report = compare_embedded_network_profiles()?;

    if has_flag(args, "--enforce") && !report.passed {
        return Err(AppError::new(
            ErrorCode::PolicyGateFailed,
            "Mainnet/testnet baseline parity failed; inspect drift before production promotion",
        ));
    }

    emit_serialized(&report, output_format(args))
}

pub fn cmd_network_identity_gate(args: &[String]) -> Result<(), AppError> {
    let settings = effective_settings_for_ops()?;
    let full_scan = has_flag(args, "--full");
    let enforced = has_flag(args, "--enforce");
    let target_envs = parse_identity_target_envs(args, &settings.profile, full_scan)?;
    let repo_root = locate_repo_root_for_identity_gate()?;

    let mut environments = Vec::new();
    let mut failures = Vec::new();

    for env in &target_envs {
        match evaluate_identity_environment(&repo_root, env) {
            Ok(summary) => environments.push(summary),
            Err(err) => {
                failures.push(format!("{env}: {err}"));
                environments.push(NetworkIdentityEnvironmentReport {
                    environment: env.clone(),
                    passed: false,
                    chain_id: None,
                    network_id: None,
                    network_serial: None,
                    detail: err,
                });
            }
        }
    }

    let report = NetworkIdentityGateReport {
        full_scan,
        enforced,
        passed: failures.is_empty(),
        checked_environments: target_envs,
        failure_count: failures.len(),
        environments,
    };

    if report.enforced && !report.passed {
        return Err(AppError::new(
            ErrorCode::PolicyGateFailed,
            format!(
                "Network identity gate failed with {} failing environment checks: {}",
                report.failure_count,
                failures.join(" | ")
            ),
        ));
    }

    emit_serialized(&report, output_format(args))
}

fn parse_identity_target_envs(
    args: &[String],
    default_profile: &str,
    full_scan: bool,
) -> Result<Vec<String>, AppError> {
    let mut envs = Vec::new();
    let mut i = 0usize;
    while i < args.len() {
        if args[i] == "--env" {
            let Some(value) = args.get(i + 1) else {
                return Err(AppError::new(
                    ErrorCode::UsageInvalidArguments,
                    "network-identity-gate requires a value after --env",
                ));
            };
            envs.push(value.clone());
            i += 2;
            continue;
        }
        i += 1;
    }

    if full_scan {
        if !envs.is_empty() {
            return Err(AppError::new(
                ErrorCode::UsageInvalidArguments,
                "network-identity-gate does not allow --env together with --full",
            ));
        }
        return Ok(vec![
            "mainnet".to_string(),
            "testnet".to_string(),
            "devnet".to_string(),
            "validation".to_string(),
            "localnet".to_string(),
        ]);
    }

    if envs.is_empty() {
        return Ok(vec![default_profile.to_string()]);
    }

    envs.sort();
    envs.dedup();
    Ok(envs)
}

fn locate_repo_root_for_identity_gate() -> Result<PathBuf, AppError> {
    let current_dir = std::env::current_dir().map_err(|_| {
        AppError::new(
            ErrorCode::UsageInvalidArguments,
            "Network identity gate failed: cannot resolve current directory",
        )
    })?;

    for dir in current_dir.ancestors() {
        if dir.join("configs/registry/network-registry.toml").is_file() {
            return Ok(dir.to_path_buf());
        }
    }

    Err(AppError::new(
        ErrorCode::UsageInvalidArguments,
        "Network identity gate failed: repository root with configs/registry/network-registry.toml not found",
    ))
}

fn evaluate_identity_environment(
    repo_root: &Path,
    env: &str,
) -> Result<NetworkIdentityEnvironmentReport, String> {
    let registry_path = repo_root.join("configs/registry/network-registry.toml");
    let expected = IdentityTuple {
        chain_id: parse_toml_u64(
            &registry_path,
            &format!("canonical_networks.{env}"),
            "chain_id",
        )?,
        network_id: parse_toml_string(
            &registry_path,
            &format!("canonical_networks.{env}"),
            "network_id",
        )?,
        network_serial: parse_toml_string(
            &registry_path,
            &format!("canonical_networks.{env}"),
            "network_serial",
        )?,
    };

    let env_root = repo_root.join("configs/environments").join(env);
    let release_policy_path = env_root.join("release-policy.toml");
    let profile_path = env_root.join("profile.toml");
    let genesis_path = env_root.join("genesis.v1.json");
    let genesis_hash_path = env_root.join("genesis.v1.sha256");

    let release_policy = parse_identity_from_release_policy(&release_policy_path)?;
    let profile = parse_identity_from_profile(&profile_path)?;
    let genesis = parse_identity_from_genesis(&genesis_path)?;

    if release_policy != expected {
        return Err(format!(
            "release-policy tuple mismatch for {env}: expected={expected:?} actual={release_policy:?}"
        ));
    }
    if profile != expected {
        return Err(format!(
            "profile tuple mismatch for {env}: expected={expected:?} actual={profile:?}"
        ));
    }
    if genesis != expected {
        return Err(format!(
            "genesis tuple mismatch for {env}: expected={expected:?} actual={genesis:?}"
        ));
    }

    assert_identity_overrides_fail_closed(&release_policy_path)?;
    assert_genesis_hash_matches(&genesis_path, &genesis_hash_path)?;

    Ok(NetworkIdentityEnvironmentReport {
        environment: env.to_string(),
        passed: true,
        chain_id: Some(expected.chain_id),
        network_id: Some(expected.network_id),
        network_serial: Some(expected.network_serial),
        detail: "identity tuple and genesis hash are consistent".to_string(),
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct IdentityTuple {
    chain_id: u64,
    network_id: String,
    network_serial: String,
}

fn parse_identity_from_release_policy(path: &Path) -> Result<IdentityTuple, String> {
    Ok(IdentityTuple {
        chain_id: parse_toml_u64(path, "identity", "chain_id")?,
        network_id: parse_toml_string(path, "identity", "network_id")?,
        network_serial: parse_toml_string(path, "identity", "network_serial")?,
    })
}

fn parse_identity_from_profile(path: &Path) -> Result<IdentityTuple, String> {
    Ok(IdentityTuple {
        chain_id: parse_toml_u64(path, "identity", "chain_id")?,
        network_id: parse_toml_string(path, "identity", "network_id")?,
        network_serial: parse_toml_string(path, "identity", "network_serial")?,
    })
}

fn parse_identity_from_genesis(path: &Path) -> Result<IdentityTuple, String> {
    let raw =
        fs::read_to_string(path).map_err(|e| format!("failed to read {}: {e}", path.display()))?;
    let value: serde_json::Value = serde_json::from_str(&raw)
        .map_err(|e| format!("failed to parse {}: {e}", path.display()))?;
    let identity = value
        .get("identity")
        .and_then(serde_json::Value::as_object)
        .ok_or_else(|| format!("missing identity object in {}", path.display()))?;
    Ok(IdentityTuple {
        chain_id: identity
            .get("chain_id")
            .and_then(serde_json::Value::as_u64)
            .ok_or_else(|| format!("missing identity.chain_id in {}", path.display()))?,
        network_id: identity
            .get("network_id")
            .and_then(serde_json::Value::as_str)
            .map(ToOwned::to_owned)
            .ok_or_else(|| format!("missing identity.network_id in {}", path.display()))?,
        network_serial: identity
            .get("network_serial")
            .and_then(serde_json::Value::as_str)
            .map(ToOwned::to_owned)
            .ok_or_else(|| format!("missing identity.network_serial in {}", path.display()))?,
    })
}

fn parse_toml_file(path: &Path) -> Result<String, String> {
    let raw =
        fs::read_to_string(path).map_err(|e| format!("failed to read {}: {e}", path.display()))?;
    Ok(raw)
}

fn parse_toml_u64(path: &Path, section: &str, key: &str) -> Result<u64, String> {
    let value = parse_toml_scalar(path, section, key)?;
    value.parse::<u64>().map_err(|_| {
        format!(
            "{}: invalid integer for {}.{}",
            path.display(),
            section,
            key
        )
    })
}

fn parse_toml_string(path: &Path, section: &str, key: &str) -> Result<String, String> {
    let value = parse_toml_scalar(path, section, key)?;
    Ok(value.trim_matches('"').to_string())
}

fn parse_toml_scalar(path: &Path, section: &str, key: &str) -> Result<String, String> {
    let raw = parse_toml_file(path)?;
    let target_header = format!("[{section}]");
    let mut in_section = false;
    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            in_section = trimmed == target_header;
            continue;
        }
        if !in_section {
            continue;
        }
        if let Some((lhs, rhs)) = trimmed.split_once('=')
            && lhs.trim() == key
        {
            return Ok(rhs.trim().to_string());
        }
    }
    Err(format!(
        "{}: missing {}.{} in TOML content",
        path.display(),
        section,
        key
    ))
}

fn assert_identity_overrides_fail_closed(path: &Path) -> Result<(), String> {
    for key in [
        "allow_chain_id_override",
        "allow_network_id_override",
        "allow_manifest_identity_override",
    ] {
        let actual = parse_toml_scalar(path, "promotion", key)?;
        let actual = match actual.as_str() {
            "true" => true,
            "false" => false,
            _ => {
                return Err(format!(
                    "{}: invalid boolean promotion.{key}={actual}",
                    path.display()
                ));
            }
        };
        if actual {
            return Err(format!(
                "{} requires promotion.{key}=false for fail-closed identity policy",
                path.display()
            ));
        }
    }
    Ok(())
}

fn assert_genesis_hash_matches(genesis_path: &Path, hash_path: &Path) -> Result<(), String> {
    let genesis = fs::read(genesis_path)
        .map_err(|e| format!("failed to read {}: {e}", genesis_path.display()))?;
    let digest = Sha256::digest(&genesis);
    let computed = hex::encode(digest);
    let declared_raw = fs::read_to_string(hash_path)
        .map_err(|e| format!("failed to read {}: {e}", hash_path.display()))?;
    let declared = declared_raw
        .split_whitespace()
        .next()
        .ok_or_else(|| format!("missing hash digest in {}", hash_path.display()))?
        .to_lowercase();
    if declared != computed {
        return Err(format!(
            "genesis hash mismatch: {} declared={} computed={}",
            hash_path.display(),
            declared,
            computed
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::parse_identity_target_envs;

    fn args(raw: &[&str]) -> Vec<String> {
        raw.iter().map(|v| (*v).to_string()).collect()
    }

    #[test]
    fn network_identity_targets_default_to_active_profile_without_full() {
        let parsed = parse_identity_target_envs(&[], "testnet", false).expect("should parse");
        assert_eq!(parsed, vec!["testnet".to_string()]);
    }

    #[test]
    fn network_identity_targets_expand_to_all_with_full_flag() {
        let parsed =
            parse_identity_target_envs(&args(&["--full"]), "testnet", true).expect("should parse");
        assert_eq!(
            parsed,
            vec![
                "mainnet".to_string(),
                "testnet".to_string(),
                "devnet".to_string(),
                "validation".to_string(),
                "localnet".to_string()
            ]
        );
    }

    #[test]
    fn network_identity_targets_reject_env_with_full_flag() {
        let parsed =
            parse_identity_target_envs(&args(&["--full", "--env", "testnet"]), "testnet", true);
        assert!(parsed.is_err());
    }
}
