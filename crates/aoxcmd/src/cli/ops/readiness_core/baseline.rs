use super::*;

pub(in crate::cli::ops) fn has_release_evidence(dir: &Path) -> bool {
    has_matching_artifact(dir, "release-evidence-", ".md")
        && has_matching_artifact(dir, "build-manifest-", ".json")
        && has_matching_artifact(dir, "compat-matrix-", ".json")
        && has_matching_artifact(dir, "production-audit-", ".json")
        && has_matching_artifact(dir, "sbom-", ".json")
        && (has_matching_artifact(dir, "aoxc-", ".sig")
            || has_matching_artifact(dir, "aoxc-", ".sig.status"))
}

pub(in crate::cli::ops) fn has_release_provenance_bundle(dir: &Path) -> bool {
    has_matching_artifact(dir, "provenance-", ".json")
        && has_matching_artifact(dir, "release-provenance-", ".json")
        && has_matching_artifact(dir, "release-sbom-", ".json")
        && has_matching_artifact(dir, "release-build-manifest-", ".json")
        && has_matching_artifact(dir, "release-signature-status-", ".txt")
}

pub(in crate::cli::ops) fn has_production_closure_artifacts(dir: &Path) -> bool {
    [
        "production-audit.json",
        "runtime-status.json",
        "soak-plan.json",
        "telemetry-snapshot.json",
        "aoxhub-rollout.json",
        "alert-rules.md",
    ]
    .iter()
    .all(|file| dir.join(file).exists())
}

pub(in crate::cli::ops) fn has_security_drill_artifact(dir: &Path) -> bool {
    json_artifact_has_required_strings(
        &dir.join("security-drill.json"),
        "scenarios",
        &["penetration-baseline", "rpc-authz", "session-replay"],
    )
}

pub(in crate::cli::ops) fn has_desktop_wallet_compat_artifact(dir: &Path) -> bool {
    json_artifact_has_required_strings(
        &dir.join("desktop-wallet-compat.json"),
        "surfaces",
        &["desktop-wallet", "aoxhub", "mainnet", "testnet"],
    )
}

pub(in crate::cli::ops) fn json_artifact_has_required_strings(
    path: &Path,
    key: &str,
    required: &[&str],
) -> bool {
    let Ok(raw) = fs::read_to_string(path) else {
        return false;
    };
    let Ok(value) = serde_json::from_str::<Value>(&raw) else {
        return false;
    };
    let Some(values) = value
        .get(key)
        .and_then(|entry| entry.as_array())
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.as_str())
                .collect::<Vec<_>>()
        })
    else {
        return false;
    };

    required
        .iter()
        .all(|needle| values.iter().any(|value| value == needle))
}

pub(in crate::cli::ops) fn has_matching_artifact(dir: &Path, prefix: &str, suffix: &str) -> bool {
    std::fs::read_dir(dir)
        .ok()
        .into_iter()
        .flatten()
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| entry.file_name().into_string().ok())
        .any(|name| name.starts_with(prefix) && name.ends_with(suffix))
}

pub(in crate::cli::ops) fn compare_embedded_network_profiles()
-> Result<ProfileBaselineReport, AppError> {
    let repo_root = locate_repo_root();
    compare_network_profile_pair(
        repo_root.join("configs").join("mainnet.toml"),
        repo_root.join("configs").join("testnet.toml"),
    )
}

pub(in crate::cli::ops) fn compare_aoxhub_network_profiles()
-> Result<ProfileBaselineReport, AppError> {
    let repo_root = locate_repo_root();
    compare_network_profile_pair(
        repo_root.join("configs").join("aoxhub-mainnet.toml"),
        repo_root.join("configs").join("aoxhub-testnet.toml"),
    )
}

pub(in crate::cli::ops) fn compare_network_profile_pair(
    mainnet_path: PathBuf,
    testnet_path: PathBuf,
) -> Result<ProfileBaselineReport, AppError> {
    let mainnet = parse_network_profile(&mainnet_path)?;
    let testnet = parse_network_profile(&testnet_path)?;

    let shared_controls = vec![
        BaselineControl {
            name: "security_mode",
            mainnet: mainnet.security_mode.clone(),
            testnet: testnet.security_mode.clone(),
            passed: !mainnet.security_mode.trim().is_empty()
                && mainnet.security_mode == testnet.security_mode,
            expectation: "Both profiles must enforce the same security mode before promotion",
        },
        BaselineControl {
            name: "peer_seed_count",
            mainnet: mainnet.peers.len().to_string(),
            testnet: testnet.peers.len().to_string(),
            passed: !mainnet.peers.is_empty() && mainnet.peers.len() == testnet.peers.len(),
            expectation: "Both profiles should define the same number of bootstrap peers",
        },
        BaselineControl {
            name: "listen_port_offset",
            mainnet: normalized_port_pair(&mainnet.listen_addr, &mainnet.rpc_addr),
            testnet: normalized_port_pair(&testnet.listen_addr, &testnet.rpc_addr),
            passed: ports_are_shifted_consistently(&mainnet, &testnet),
            expectation: "Testnet should differ only by deterministic port offsets, not by capability shape",
        },
    ];

    let drift = shared_controls
        .iter()
        .filter(|control| !control.passed)
        .map(|control| {
            format!(
                "{} mismatch (mainnet={}, testnet={})",
                control.name, control.mainnet, control.testnet
            )
        })
        .collect::<Vec<_>>();

    Ok(ProfileBaselineReport {
        mainnet_path: mainnet_path.display().to_string(),
        testnet_path: testnet_path.display().to_string(),
        passed: drift.is_empty(),
        shared_controls,
        drift,
    })
}

pub(in crate::cli::ops) fn locate_repo_root() -> PathBuf {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    for candidate in cwd.ancestors() {
        if candidate.join("Cargo.toml").exists() && candidate.join("configs").exists() {
            return candidate.to_path_buf();
        }
    }
    cwd
}

pub(in crate::cli::ops) fn open_checklist_items(path: &Path) -> Vec<String> {
    let raw = match fs::read_to_string(path) {
        Ok(raw) => raw,
        Err(_) => return vec![format!("missing-checklist:{}", path.display())],
    };

    raw.lines()
        .map(str::trim)
        .filter(|line| line.starts_with("- [ ] "))
        .map(|line| line.trim_start_matches("- [ ] ").to_string())
        .collect()
}

pub(in crate::cli::ops) fn parse_network_profile(
    path: &Path,
) -> Result<NetworkProfileConfig, AppError> {
    let raw = fs::read_to_string(path).map_err(|error| {
        AppError::with_source(
            ErrorCode::FilesystemIoFailed,
            format!("Failed to read network profile {}", path.display()),
            error,
        )
    })?;

    let mut config = NetworkProfileConfig::default();
    let mut in_peers = false;

    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        if in_peers {
            if trimmed == "]" {
                in_peers = false;
                continue;
            }
            let peer = trimmed.trim_end_matches(',').trim_matches('"').trim();
            if !peer.is_empty() {
                config.peers.push(peer.to_string());
            }
            continue;
        }

        if let Some((key, value)) = trimmed.split_once('=') {
            let key = key.trim();
            let value = value.trim();
            match key {
                "chain_id" => config.chain_id = unquote(value),
                "listen_addr" => config.listen_addr = unquote(value),
                "rpc_addr" => config.rpc_addr = unquote(value),
                "security_mode" => config.security_mode = unquote(value),
                "peers" if value == "[" => in_peers = true,
                _ => {}
            }
        }
    }

    if config.chain_id.is_empty()
        || config.listen_addr.is_empty()
        || config.rpc_addr.is_empty()
        || config.security_mode.is_empty()
    {
        return Err(AppError::new(
            ErrorCode::ConfigInvalid,
            format!(
                "Network profile {} is missing required fields",
                path.display()
            ),
        ));
    }

    Ok(config)
}

pub(in crate::cli::ops) fn unquote(value: &str) -> String {
    value.trim().trim_matches('"').to_string()
}

pub(in crate::cli::ops) fn normalized_port_pair(listen_addr: &str, rpc_addr: &str) -> String {
    format!(
        "{}/{}",
        extract_port(listen_addr).map_or_else(|| "?".to_string(), |p| p.to_string()),
        extract_port(rpc_addr).map_or_else(|| "?".to_string(), |p| p.to_string())
    )
}

pub(in crate::cli::ops) fn ports_are_shifted_consistently(
    mainnet: &NetworkProfileConfig,
    testnet: &NetworkProfileConfig,
) -> bool {
    let mainnet_listen = extract_port(&mainnet.listen_addr);
    let testnet_listen = extract_port(&testnet.listen_addr);
    let mainnet_rpc = extract_port(&mainnet.rpc_addr);
    let testnet_rpc = extract_port(&testnet.rpc_addr);

    match (mainnet_listen, testnet_listen, mainnet_rpc, testnet_rpc) {
        (Some(ml), Some(tl), Some(mr), Some(tr)) => tl > ml && tr > mr && (tl - ml) == (tr - mr),
        _ => false,
    }
}

pub(in crate::cli::ops) fn extract_port(addr: &str) -> Option<u16> {
    addr.rsplit(':').next()?.parse::<u16>().ok()
}
