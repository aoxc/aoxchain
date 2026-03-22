use crate::{
    error::AiError,
    manifest::{FusionStrategy, ModelManifest, Thresholds},
    model::{
        Assessment, DecisionAction, FindingSeverity, InferenceFinding, InferenceRequest,
        ModelOutput, OutputLabel,
    },
    traits::DecisionPolicy,
};

/// Default deterministic fusion policy for AOXC AI decisions.
#[derive(Debug, Default)]
pub struct FusionPolicy;

impl FusionPolicy {
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl DecisionPolicy for FusionPolicy {
    fn name(&self) -> &'static str {
        "fusion_policy_v1"
    }

    async fn decide(
        &self,
        manifest: &ModelManifest,
        _request: &InferenceRequest,
        output: &ModelOutput,
        findings: &[InferenceFinding],
    ) -> Result<Assessment, AiError> {
        match manifest.spec.decision.fusion.strategy {
            FusionStrategy::Weighted => decide_weighted(manifest, output, findings),
        }
    }
}

fn decide_weighted(
    manifest: &ModelManifest,
    output: &ModelOutput,
    findings: &[InferenceFinding],
) -> Result<Assessment, AiError> {
    let thresholds = &manifest.spec.decision.thresholds;
    let confidence = &manifest.spec.decision.confidence;
    let weights = &manifest.spec.decision.fusion.weights;
    let overrides = &manifest.spec.decision.fusion.deterministic_overrides;

    let deterministic_risk_bps = deterministic_risk_from_findings(findings);
    let numerator = (output.risk_bps as u32 * weights.model_risk_bps as u32)
        + (deterministic_risk_bps as u32 * weights.deterministic_risk_bps as u32);

    let denominator =
        (weights.model_risk_bps as u32 + weights.deterministic_risk_bps as u32).max(1);

    let mut effective_risk_bps = (numerator / denominator).min(10_000) as u16;

    if should_force_deny(findings, overrides) {
        effective_risk_bps = effective_risk_bps.max(thresholds.deny_min_risk_bps);
    }

    let mut action = if effective_risk_bps >= thresholds.deny_min_risk_bps {
        DecisionAction::Deny
    } else {
        map_label_to_action(manifest, output.label)
    };

    action = tighten_action_by_risk(action, thresholds, effective_risk_bps);

    if action != DecisionAction::Deny && output.confidence_bps < confidence.minimum_confidence_bps {
        action = confidence.low_confidence_action.into();
    }

    if should_force_review(findings, overrides) && action == DecisionAction::Allow {
        action = DecisionAction::Review;
    }

    Ok(Assessment {
        action,
        effective_risk_bps,
        confidence_bps: output.confidence_bps,
        rationale: format!(
            "Fusion policy applied manifest='{}' model_label='{:?}' model_risk_bps={} deterministic_risk_bps={}.",
            manifest.metadata.id, output.label, output.risk_bps, deterministic_risk_bps
        ),
    })
}

fn deterministic_risk_from_findings(findings: &[InferenceFinding]) -> u16 {
    let mut risk: u16 = 0;

    for finding in findings {
        let increment = match finding.severity {
            FindingSeverity::Critical => 9_000,
            FindingSeverity::High => 5_000,
            FindingSeverity::Warning => 2_000,
            FindingSeverity::Info => 500,
        };
        risk = risk.saturating_add(increment);
    }

    risk.min(10_000)
}

fn should_force_deny(
    findings: &[InferenceFinding],
    overrides: &crate::manifest::DeterministicOverrides,
) -> bool {
    if overrides.deny_on_critical_finding
        && findings
            .iter()
            .any(|finding| finding.severity == FindingSeverity::Critical)
    {
        return true;
    }

    if overrides.deny_on_revoked_identity
        && findings
            .iter()
            .any(|finding| finding.code == "revoked_identity")
    {
        return true;
    }

    if overrides.deny_on_invalid_quorum_proof
        && findings
            .iter()
            .any(|finding| finding.code == "invalid_quorum_proof")
    {
        return true;
    }

    false
}

fn should_force_review(
    findings: &[InferenceFinding],
    overrides: &crate::manifest::DeterministicOverrides,
) -> bool {
    if overrides.review_on_missing_context
        && findings
            .iter()
            .any(|finding| finding.code == "missing_context")
    {
        return true;
    }

    if overrides.review_on_model_timeout
        && findings
            .iter()
            .any(|finding| finding.code == "model_timeout")
    {
        return true;
    }

    false
}

fn tighten_action_by_risk(
    action: DecisionAction,
    thresholds: &Thresholds,
    effective_risk_bps: u16,
) -> DecisionAction {
    if effective_risk_bps >= thresholds.deny_min_risk_bps {
        return DecisionAction::Deny;
    }

    if effective_risk_bps > thresholds.allow_max_risk_bps && action == DecisionAction::Allow {
        return DecisionAction::Review;
    }

    action
}

fn map_label_to_action(manifest: &ModelManifest, label: OutputLabel) -> DecisionAction {
    let actions = &manifest.spec.decision.actions;

    match label {
        OutputLabel::Trusted => actions.trusted.into(),
        OutputLabel::Review => actions.review.into(),
        OutputLabel::Suspicious => actions.suspicious.into(),
        OutputLabel::Malicious => actions.malicious.into(),
        OutputLabel::Unknown => actions.unknown.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        model::{InferenceFinding, OutputLabel},
        test_support::{base_manifest, empty_request, model_output},
    };

    #[tokio::test]
    async fn decide_allows_low_risk_trusted_output() {
        let manifest = base_manifest();
        let policy = FusionPolicy::new();
        let output = model_output(OutputLabel::Trusted, 1_000, 8_000);

        let assessment = policy
            .decide(&manifest, &empty_request(), &output, &[])
            .await
            .expect("decision must succeed");

        assert_eq!(assessment.action, DecisionAction::Allow);
        assert_eq!(assessment.effective_risk_bps, 600);
    }

    #[tokio::test]
    async fn decide_reviews_when_confidence_is_below_minimum() {
        let manifest = base_manifest();
        let policy = FusionPolicy::new();
        let output = model_output(OutputLabel::Trusted, 1_000, 1_000);

        let assessment = policy
            .decide(&manifest, &empty_request(), &output, &[])
            .await
            .expect("decision must succeed");

        assert_eq!(assessment.action, DecisionAction::Review);
    }

    #[tokio::test]
    async fn decide_forces_review_on_missing_context_finding() {
        let manifest = base_manifest();
        let policy = FusionPolicy::new();
        let output = model_output(OutputLabel::Trusted, 0, 8_000);
        let findings = vec![InferenceFinding::new(
            "missing_context",
            "Context is incomplete.",
            FindingSeverity::Info,
        )];

        let assessment = policy
            .decide(&manifest, &empty_request(), &output, &findings)
            .await
            .expect("decision must succeed");

        assert_eq!(assessment.action, DecisionAction::Review);
    }

    #[tokio::test]
    async fn decide_forces_deny_on_invalid_quorum_override() {
        let manifest = base_manifest();
        let policy = FusionPolicy::new();
        let output = model_output(OutputLabel::Trusted, 0, 8_000);
        let findings = vec![InferenceFinding::new(
            "invalid_quorum_proof",
            "Quorum proof is invalid.",
            FindingSeverity::Info,
        )];

        let assessment = policy
            .decide(&manifest, &empty_request(), &output, &findings)
            .await
            .expect("decision must succeed");

        assert_eq!(assessment.action, DecisionAction::Deny);
        assert!(
            assessment.effective_risk_bps >= manifest.spec.decision.thresholds.deny_min_risk_bps
        );
    }

    #[tokio::test]
    async fn decide_forces_deny_on_critical_finding() {
        let manifest = base_manifest();
        let policy = FusionPolicy::new();
        let output = model_output(OutputLabel::Trusted, 1_000, 8_000);
        let findings = vec![InferenceFinding::new(
            "runtime_anomaly",
            "Critical anomaly detected.",
            FindingSeverity::Critical,
        )];

        let assessment = policy
            .decide(&manifest, &empty_request(), &output, &findings)
            .await
            .expect("decision must succeed");

        assert_eq!(assessment.action, DecisionAction::Deny);
    }
}
