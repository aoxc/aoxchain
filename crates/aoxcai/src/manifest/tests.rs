use super::*;
use crate::{error::AiError, model::AiTask, test_support::base_manifest};
use std::{
    collections::BTreeMap,
    env, fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

fn write_temp_manifest(content: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock must be valid")
        .as_nanos();
    let path = env::temp_dir().join(format!("aoxcai-manifest-{unique}.yaml"));
    fs::write(&path, content).expect("temp manifest write must succeed");
    path
}

#[test]
fn base_manifest_is_valid_and_binds_default_task() {
    let manifest = base_manifest();
    manifest.validate().expect("base manifest must validate");
    assert!(manifest.is_enabled());
    assert_eq!(manifest.id(), "test-model");
    assert!(manifest.binds_task(AiTask::ValidatorAdmission));
    assert!(!manifest.binds_task(AiTask::TransactionScreening));
}

#[test]
fn validation_rejects_invalid_threshold_orderings_across_matrix() {
    let mut manifest = base_manifest();
    let ordered_values = [0_u16, 1, 2_499, 7_000, 10_000];

    for allow in ordered_values {
        for review in ordered_values {
            for deny in ordered_values {
                manifest.spec.decision.thresholds.allow_max_risk_bps = allow;
                manifest.spec.decision.thresholds.review_max_risk_bps = review;
                manifest.spec.decision.thresholds.deny_min_risk_bps = deny;

                let ok = allow <= review && review < deny;
                let result = manifest.validate();
                if ok {
                    assert!(
                        result.is_ok(),
                        "expected valid thresholds: {allow}, {review}, {deny}"
                    );
                } else {
                    assert!(
                        result.is_err(),
                        "expected invalid thresholds: {allow}, {review}, {deny}"
                    );
                }
            }
        }
    }
}

#[test]
fn validation_rejects_remote_http_bearer_env_without_env_key() {
    let mut manifest = base_manifest();
    manifest.spec.backend.r#type = BackendType::RemoteHttp;
    manifest.spec.backend.heuristic = None;
    manifest.spec.backend.remote_http = Some(RemoteHttpBackend {
        endpoint: "https://inference.aoxc.local/infer".to_owned(),
        method: HttpMethod::Post,
        headers: BTreeMap::new(),
        auth: Auth {
            mode: AuthMode::BearerEnv,
            env_key: String::new(),
        },
        tls: Tls {
            enabled: true,
            verify_peer: true,
        },
        rate_limit: RateLimit {
            requests_per_minute: 60,
            burst: 10,
        },
    });

    match manifest
        .validate()
        .expect_err("manifest must fail validation")
    {
        AiError::ManifestValidation(message) => {
            assert!(
                message.contains("env_key"),
                "error must describe env_key constraint"
            )
        }
        other => panic!("unexpected error variant: {other}"),
    }
}

#[test]
fn from_yaml_file_returns_io_and_parse_errors() {
    let missing = ModelManifest::from_yaml_file("/tmp/does-not-exist-manifest.yaml")
        .expect_err("missing file must fail with Io error");
    assert!(matches!(missing, AiError::Io { .. }));

    let invalid_yaml_path = write_temp_manifest("api_version: [invalid");
    let parse_err = ModelManifest::from_yaml_file(&invalid_yaml_path)
        .expect_err("invalid yaml must fail parsing");
    assert!(matches!(parse_err, AiError::ManifestParse(_)));
    fs::remove_file(invalid_yaml_path).expect("temp manifest should be removable");
}
