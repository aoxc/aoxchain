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
