use crate::{
    error::AiError,
    manifest::ModelManifest,
    model::{FindingSeverity, InferenceRequest, ModelOutput, OutputLabel},
    traits::InferenceBackend,
};
use std::collections::BTreeMap;

/// Built-in deterministic fallback backend.
///
/// The heuristic backend is intentionally transparent and conservative. It is
/// suitable for advisory mode, offline fallback, and policy-hardening paths.
pub struct HeuristicBackendRuntime;

impl HeuristicBackendRuntime {
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl Default for HeuristicBackendRuntime {
    fn default() -> Self {
        Self::new()
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
        let heuristic = manifest.spec.backend.heuristic.as_ref().ok_or_else(|| {
            AiError::ManifestValidation(
                "heuristic backend requires spec.backend.heuristic".to_owned(),
            )
        })?;

        let mut risk_bps: u16 = 0;

        for signal in &request.signals {
            let signal_value = signal.value.to_ascii_lowercase();
            let matched = heuristic
                .anomaly_keywords
                .iter()
                .any(|keyword| signal_value.contains(&keyword.to_ascii_lowercase()));

            if matched {
                risk_bps = risk_bps.saturating_add(signal.weight_bps.min(2_500));
            }
        }

        for finding in &request.findings {
            let increment = match finding.severity {
                FindingSeverity::Critical => 4_000,
                FindingSeverity::High => 2_500,
                FindingSeverity::Warning => 1_000,
                FindingSeverity::Info => 250,
            };
            risk_bps = risk_bps.saturating_add(increment);
        }

        let risk_bps = risk_bps.min(10_000);
        let label = if risk_bps >= 7_000 {
            OutputLabel::Malicious
        } else if risk_bps >= 3_500 {
            OutputLabel::Suspicious
        } else if risk_bps >= 1_500 {
            OutputLabel::Review
        } else {
            OutputLabel::Trusted
        };

        let mut attributes = BTreeMap::new();
        attributes.insert("ruleset".to_owned(), heuristic.ruleset.clone());
        attributes.insert("signal_count".to_owned(), request.signals.len().to_string());
        attributes.insert(
            "finding_count".to_owned(),
            request.findings.len().to_string(),
        );

        Ok(ModelOutput {
            backend: self.name().to_owned(),
            model_id: manifest.metadata.id.clone(),
            label,
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
        model::{FindingSeverity, InferenceFinding, InferenceSignal},
        test_support::{empty_request, heuristic_manifest, request_with},
    };

    #[tokio::test]
    async fn infer_returns_trusted_when_risk_is_absent() {
        let manifest = heuristic_manifest();
        let backend = HeuristicBackendRuntime::new();
        let output = backend
            .infer(&manifest, &empty_request())
            .await
            .expect("heuristic inference must succeed");

        assert_eq!(output.label, OutputLabel::Trusted);
        assert_eq!(output.risk_bps, 0);
    }

    #[tokio::test]
    async fn infer_escalates_when_keywords_match() {
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

        assert_eq!(output.label, OutputLabel::Review);
        assert_eq!(output.risk_bps, 2_000);
    }

    #[tokio::test]
    async fn infer_saturates_risk_at_maximum() {
        let manifest = heuristic_manifest();
        let backend = HeuristicBackendRuntime::new();
        let request = request_with(
            vec![
                InferenceSignal::new("status", "revoked_identity", 10_000, "unit_test"),
                InferenceSignal::new("runtime", "anomaly", 10_000, "unit_test"),
                InferenceSignal::new("runtime", "revoked_again", 10_000, "unit_test"),
            ],
            vec![
                InferenceFinding::new("revoked_identity", "critical", FindingSeverity::Critical),
                InferenceFinding::new(
                    "invalid_quorum_proof",
                    "critical",
                    FindingSeverity::Critical,
                ),
                InferenceFinding::new("runtime_anomaly", "warning", FindingSeverity::Warning),
            ],
        );

        let output = backend
            .infer(&manifest, &request)
            .await
            .expect("heuristic inference must succeed");
        assert_eq!(output.risk_bps, 10_000);
        assert_eq!(output.label, OutputLabel::Malicious);
    }

    #[tokio::test]
    async fn infer_emits_structural_attributes() {
        let manifest = heuristic_manifest();
        let backend = HeuristicBackendRuntime::new();
        let request = request_with(
            vec![InferenceSignal::new("status", "healthy", 250, "unit_test")],
            vec![InferenceFinding::new("info", "info", FindingSeverity::Info)],
        );

        let output = backend
            .infer(&manifest, &request)
            .await
            .expect("heuristic inference must succeed");
        assert_eq!(
            output.attributes.get("ruleset").map(String::as_str),
            Some("default")
        );
        assert_eq!(
            output.attributes.get("signal_count").map(String::as_str),
            Some("1")
        );
        assert_eq!(
            output.attributes.get("finding_count").map(String::as_str),
            Some("1")
        );
    }

    #[tokio::test]
    async fn infer_weights_findings_by_severity() {
        let manifest = heuristic_manifest();
        let backend = HeuristicBackendRuntime::new();
        let request = request_with(
            vec![],
            vec![
                InferenceFinding::new(
                    "revoked_identity",
                    "Critical identity revocation detected.",
                    FindingSeverity::Critical,
                ),
                InferenceFinding::new(
                    "runtime_anomaly",
                    "Runtime anomaly detected.",
                    FindingSeverity::Warning,
                ),
            ],
        );

        let output = backend
            .infer(&manifest, &request)
            .await
            .expect("heuristic inference must succeed");

        assert_eq!(output.risk_bps, 5_000);
        assert_eq!(output.label, OutputLabel::Suspicious);
    }
}
