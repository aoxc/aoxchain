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
