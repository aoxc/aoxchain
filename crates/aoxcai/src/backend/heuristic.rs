use crate::{
    error::AiError,
    manifest::ModelManifest,
    model::{InferenceRequest, ModelOutput},
    traits::InferenceBackend,
};
use std::collections::BTreeMap;

/// Built-in deterministic fallback backend.
///
/// This backend is intentionally simple and transparent. It exists to provide
/// stable behavior when no external model is present or when the operator
/// explicitly selects a heuristic runtime.
pub struct HeuristicBackendRuntime;

impl HeuristicBackendRuntime {
    /// Creates a new heuristic backend instance.
    pub fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl InferenceBackend for HeuristicBackendRuntime {
    fn name(&self) -> &'static str {
        "heuristic"
    }

    async fn infer(
        &self,
        manifest: &ModelManifest,
        request: &InferenceRequest,
    ) -> Result<ModelOutput, AiError> {
        let heuristic = manifest
            .spec
            .backend
            .heuristic
            .as_ref()
            .ok_or_else(|| AiError::ManifestValidation(
                "heuristic backend requires spec.backend.heuristic".to_owned(),
            ))?;

        let keywords = &heuristic.anomaly_keywords;
        let mut risk_bps: u16 = 0;

        for signal in &request.signals {
            let value = signal.value.to_ascii_lowercase();
            if keywords.iter().any(|keyword| value.contains(&keyword.to_ascii_lowercase())) {
                risk_bps = risk_bps.saturating_add(signal.weight_bps.min(2_500));
            }
        }

        for finding in &request.findings {
            match finding.severity.as_str() {
                "critical" => risk_bps = risk_bps.saturating_add(4_000),
                "high" => risk_bps = risk_bps.saturating_add(2_500),
                "warning" => risk_bps = risk_bps.saturating_add(1_000),
                _ => risk_bps = risk_bps.saturating_add(250),
            }
        }

        if risk_bps > 10_000 {
            risk_bps = 10_000;
        }

        let label = if risk_bps >= 7_000 {
            "malicious"
        } else if risk_bps >= 3_500 {
            "suspicious"
        } else if risk_bps >= 1_500 {
            "review"
        } else {
            "trusted"
        };

        let mut attributes = BTreeMap::new();
        attributes.insert("ruleset".to_owned(), heuristic.ruleset.clone());
        attributes.insert(
            "signal_count".to_owned(),
            request.signals.len().to_string(),
        );
        attributes.insert(
            "finding_count".to_owned(),
            request.findings.len().to_string(),
        );

        Ok(ModelOutput {
            backend: self.name().to_owned(),
            model_id: manifest.metadata.id.clone(),
            label: label.to_owned(),
            risk_bps,
            confidence_bps: 7_500,
            rationale: format!(
                "Deterministic heuristic assessment completed using ruleset='{}'.",
                heuristic.ruleset
            ),
            recommended_action: None,
            attributes,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        error::AiError,
        model::{InferenceFinding, InferenceSignal},
        test_support::{empty_request, heuristic_manifest, request_with},
    };

    #[tokio::test]
    async fn infer_returns_trusted_when_no_risk_input_is_present() {
        let manifest = heuristic_manifest();
        let backend = HeuristicBackendRuntime::new();
        let request = empty_request();

        let output = backend
            .infer(&manifest, &request)
            .await
            .expect("heuristic inference must succeed");

        assert_eq!(output.backend, "heuristic");
        assert_eq!(output.model_id, manifest.metadata.id);
        assert_eq!(output.label, "trusted");
        assert_eq!(output.risk_bps, 0);
        assert_eq!(output.confidence_bps, 7_500);
        assert_eq!(
            output.attributes.get("signal_count"),
            Some(&"0".to_owned())
        );
        assert_eq!(
            output.attributes.get("finding_count"),
            Some(&"0".to_owned())
        );
    }

    #[tokio::test]
    async fn infer_increases_risk_when_signal_matches_anomaly_keyword() {
        let manifest = heuristic_manifest();
        let backend = HeuristicBackendRuntime::new();
        let request = request_with(
            vec![InferenceSignal::new(
                "status",
                "revoked_identity",
                2_000,
                "unit_test",
            )],
            vec![],
        );

        let output = backend
            .infer(&manifest, &request)
            .await
            .expect("heuristic inference must succeed");

        assert_eq!(output.label, "review");
        assert_eq!(output.risk_bps, 2_000);
    }

    #[tokio::test]
    async fn infer_applies_finding_severity_weights() {
        let manifest = heuristic_manifest();
        let backend = HeuristicBackendRuntime::new();
        let request = request_with(
            vec![],
            vec![
                InferenceFinding::new(
                    "revoked_identity",
                    "Critical identity revocation detected.",
                    "critical",
                ),
                InferenceFinding::new(
                    "runtime_anomaly",
                    "Runtime anomaly detected.",
                    "warning",
                ),
            ],
        );

        let output = backend
            .infer(&manifest, &request)
            .await
            .expect("heuristic inference must succeed");

        assert_eq!(output.risk_bps, 5_000);
        assert_eq!(output.label, "suspicious");
    }

    #[tokio::test]
    async fn infer_caps_risk_at_upper_bound() {
        let manifest = heuristic_manifest();
        let backend = HeuristicBackendRuntime::new();
        let request = request_with(
            vec![
                InferenceSignal::new("signal_a", "revoked", 9_000, "unit_test"),
                InferenceSignal::new("signal_b", "revoked", 9_000, "unit_test"),
                InferenceSignal::new("signal_c", "anomaly", 9_000, "unit_test"),
                InferenceSignal::new("signal_d", "anomaly", 9_000, "unit_test"),
            ],
            vec![
                InferenceFinding::new("critical_1", "Critical condition.", "critical"),
                InferenceFinding::new("critical_2", "Critical condition.", "critical"),
                InferenceFinding::new("critical_3", "Critical condition.", "critical"),
            ],
        );

        let output = backend
            .infer(&manifest, &request)
            .await
            .expect("heuristic inference must succeed");

        assert_eq!(output.risk_bps, 10_000);
        assert_eq!(output.label, "malicious");
    }

    #[tokio::test]
    async fn infer_rejects_missing_heuristic_configuration() {
        let mut manifest = heuristic_manifest();
        manifest.spec.backend.heuristic = None;

        let backend = HeuristicBackendRuntime::new();
        let request = empty_request();

        let err = backend
            .infer(&manifest, &request)
            .await
            .expect_err("missing heuristic config must fail");

        match err {
            AiError::ManifestValidation(message) => {
                assert_eq!(
                    message,
                    "heuristic backend requires spec.backend.heuristic"
                );
            }
            other => panic!("unexpected error: {other}"),
        }
    }
}
