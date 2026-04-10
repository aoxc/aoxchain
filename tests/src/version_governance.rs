// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use std::fs;
use std::path::Path;

#[test]
fn workspace_version_and_version_policy_remain_synchronized() {
    let cargo_toml = fs::read_to_string("../Cargo.toml").expect("Cargo.toml must be readable");
    let policy_toml =
        fs::read_to_string("../configs/version-policy.toml").expect("version policy must exist");

    let workspace_version = read_required_value(&cargo_toml, "version");
    let policy_version = read_required_value(&policy_toml, "current");

    assert_eq!(
        workspace_version, policy_version,
        "workspace release version and policy version must stay synchronized"
    );
}

#[test]
fn version_policy_declares_required_tracks_for_controlled_rollout() {
    let policy_toml =
        fs::read_to_string("../configs/version-policy.toml").expect("version policy must exist");

    for required in [
        "strategy = \"global-workspace-version-with-component-schema-tracks\"",
        "release_channel = \"controlled\"",
        "protocol_schema = 1",
        "manifest_schema = 1",
        "certificate_schema = 1",
        "native_token_policy_schema = 1",
    ] {
        assert!(
            policy_toml.contains(required),
            "version policy missing required key/value: {required}"
        );
    }
}

#[test]
fn workspace_version_uses_semver_core_shape() {
    let cargo_toml = fs::read_to_string("../Cargo.toml").expect("Cargo.toml must be readable");
    let workspace_version = read_required_value(&cargo_toml, "version");

    let core = workspace_version
        .split('-')
        .next()
        .expect("version core must exist");
    let mut parts = core.split('.');
    let major = parts.next().expect("major segment");
    let minor = parts.next().expect("minor segment");
    let patch = parts.next().expect("patch segment");
    assert!(
        parts.next().is_none(),
        "version must have exactly three numeric core segments"
    );
    assert!(
        major.chars().all(|c| c.is_ascii_digit())
            && minor.chars().all(|c| c.is_ascii_digit())
            && patch.chars().all(|c| c.is_ascii_digit()),
        "version core must be MAJOR.MINOR.PATCH with numeric segments"
    );
}

#[test]
fn versioning_gate_script_exists() {
    assert!(
        Path::new("../scripts/validation/versioning_gate.sh").exists(),
        "version governance shell gate must exist"
    );
}

fn read_required_value(contents: &str, key: &str) -> String {
    contents
        .lines()
        .find_map(|line| {
            let trimmed = line.trim();
            if !trimmed.starts_with(&format!("{key} = \"")) {
                return None;
            }
            trimmed
                .split_once('=')
                .map(|(_, value)| value.trim().trim_matches('"').to_string())
        })
        .expect("required key must exist")
}
