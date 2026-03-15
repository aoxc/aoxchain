use crate::{
    error::AiError,
    manifest::ModelManifest,
    model::{Assessment, DecisionAction, InferenceFinding, InferenceRequest, ModelOutput},
    traits::DecisionPolicy,
};

/// Default fusion policy used to combine model output with deterministic state.
///
/// This policy is manifest-driven and therefore stable across backend changes.
#[derive(Debug, Default)]
pub struct FusionPolicy;

impl FusionPolicy {
    /// Creates a new fusion policy.
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

        let mut action = derive_action_from_risk(thresholds, effective_risk_bps);

        if action != DecisionAction::Deny {
            action = map_label_to_action(manifest, &output.label);
            action = tighten_action_by_risk(action, thresholds, effective_risk_bps);
        }

        if action != DecisionAction::Deny
            && output.confidence_bps < confidence.minimum_confidence_bps
        {
            action = map_action_name(&confidence.low_confidence_action);
        }

        if should_force_review(findings, overrides) && action == DecisionAction::Allow {
            action = DecisionAction::Review;
        }

        Ok(Assessment {
            action,
            effective_risk_bps,
            confidence_bps: output.confidence_bps,
            rationale: format!(
                "Fusion policy applied manifest='{}' model_label='{}' model_risk_bps={} deterministic_risk_bps={}.",
                manifest.metadata.id, output.label, output.risk_bps, deterministic_risk_bps
            ),
        })
    }
}

fn deterministic_risk_from_findings(findings: &[InferenceFinding]) -> u16 {
    let mut risk: u16 = 0;

    for finding in findings {
        let increment = match finding.severity.as_str() {
            "critical" => 9_000,
            "high" => 5_000,
            "warning" => 2_000,
            _ => 500,
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
            .any(|finding| finding.severity == "critical")
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

fn derive_action_from_risk(
    thresholds: &crate::manifest::Thresholds,
    effective_risk_bps: u16,
) -> DecisionAction {
    if effective_risk_bps >= thresholds.deny_min_risk_bps {
        DecisionAction::Deny
    } else if effective_risk_bps > thresholds.allow_max_risk_bps {
        DecisionAction::Review
    } else {
        DecisionAction::Allow
    }
}

fn tighten_action_by_risk(
    action: DecisionAction,
    thresholds: &crate::manifest::Thresholds,
    effective_risk_bps: u16,
) -> DecisionAction {
    match derive_action_from_risk(thresholds, effective_risk_bps) {
        DecisionAction::Deny => DecisionAction::Deny,
        DecisionAction::Review => {
            if action == DecisionAction::Allow {
                DecisionAction::Review
            } else {
                action
            }
        }
        DecisionAction::Allow => action,
    }
}

fn map_label_to_action(manifest: &ModelManifest, label: &str) -> DecisionAction {
    let actions = &manifest.spec.decision.actions;

    let action_name = match label {
        "trusted" => &actions.trusted,
        "review" => &actions.review,
        "suspicious" => &actions.suspicious,
        "malicious" => &actions.malicious,
        _ => &actions.unknown,
    };

    map_action_name(action_name)
}

fn map_action_name(name: &str) -> DecisionAction {
    match name {
        "allow" => DecisionAction::Allow,
        "deny" => DecisionAction::Deny,
        _ => DecisionAction::Review,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        model::{InferenceFinding, ModelOutput},
        test_support::empty_request,
    };

    fn base_output(label: &str, risk_bps: u16, confidence_bps: u16) -> ModelOutput {
        ModelOutput {
            backend: "heuristic".to_owned(),
            model_id: "test-model".to_owned(),
            label: label.to_owned(),
            risk_bps,
            confidence_bps,
            rationale: "Unit test model output.".to_owned(),
            recommended_action: None,
            attributes: Default::default(),
        }
    }

    #[tokio::test]
    async fn decide_allows_low_risk_trusted_output() {
        let manifest = crate::test_support::base_manifest();
        let policy = FusionPolicy::new();
        let request = empty_request();
        let output = base_output("trusted", 1_000, 8_000);

        let assessment = policy
            .decide(&manifest, &request, &output, &[])
            .await
            .expect("policy decision must succeed");

        assert_eq!(assessment.action, DecisionAction::Allow);
        assert_eq!(assessment.effective_risk_bps, 600);
        assert_eq!(assessment.confidence_bps, 8_000);
        assert!(assessment.rationale.contains("test-model"));
        assert!(assessment.rationale.contains("trusted"));
    }

    #[tokio::test]
    async fn decide_escalates_to_review_when_weighted_risk_exceeds_allow_threshold() {
        let manifest = crate::test_support::base_manifest();
        let policy = FusionPolicy::new();
        let request = empty_request();
        let output = base_output("trusted", 5_000, 8_000);

        let assessment = policy
            .decide(&manifest, &request, &output, &[])
            .await
            .expect("policy decision must succeed");

        assert_eq!(assessment.action, DecisionAction::Review);
        assert_eq!(assessment.effective_risk_bps, 3_000);
    }

    #[tokio::test]
    async fn decide_reviews_when_weighted_risk_remains_below_deny_threshold() {
        let manifest = crate::test_support::base_manifest();
        let policy = FusionPolicy::new();
        let request = empty_request();
        let output = base_output("trusted", 9_000, 8_000);

        let assessment = policy
            .decide(&manifest, &request, &output, &[])
            .await
            .expect("policy decision must succeed");

        assert_eq!(assessment.action, DecisionAction::Review);
        assert_eq!(assessment.effective_risk_bps, 5_400);
    }

    #[tokio::test]
    async fn decide_forces_deny_on_critical_finding_override() {
        let manifest = crate::test_support::base_manifest();
        let policy = FusionPolicy::new();
        let request = empty_request();
        let output = base_output("trusted", 1_000, 8_000);
        let findings = vec![InferenceFinding::new(
            "runtime_anomaly",
            "Critical anomaly detected.",
            "critical",
        )];

        let assessment = policy
            .decide(&manifest, &request, &output, &findings)
            .await
            .expect("policy decision must succeed");

        assert_eq!(assessment.action, DecisionAction::Deny);
        assert!(
            assessment.effective_risk_bps >= manifest.spec.decision.thresholds.deny_min_risk_bps
        );
    }

    #[tokio::test]
    async fn decide_applies_low_confidence_override() {
        let manifest = crate::test_support::base_manifest();
        let policy = FusionPolicy::new();
        let request = empty_request();
        let output = base_output("trusted", 1_000, 1_000);

        let assessment = policy
            .decide(&manifest, &request, &output, &[])
            .await
            .expect("policy decision must succeed");

        assert_eq!(assessment.action, DecisionAction::Review);
        assert_eq!(assessment.confidence_bps, 1_000);
    }

    #[tokio::test]
    async fn decide_maps_unknown_label_to_unknown_action_path() {
        let manifest = crate::test_support::base_manifest();
        let policy = FusionPolicy::new();
        let request = empty_request();
        let output = base_output("some_unrecognized_label", 1_000, 8_000);

        let assessment = policy
            .decide(&manifest, &request, &output, &[])
            .await
            .expect("policy decision must succeed");

        assert_eq!(assessment.action, DecisionAction::Review);
    }

    #[test]
    fn deterministic_risk_from_findings_caps_at_upper_bound() {
        let findings = vec![
            InferenceFinding::new("f1", "Critical 1", "critical"),
            InferenceFinding::new("f2", "Critical 2", "critical"),
            InferenceFinding::new("f3", "Critical 3", "critical"),
        ];

        let risk = deterministic_risk_from_findings(&findings);

        assert_eq!(risk, 10_000);
    }

    #[test]
    fn map_action_name_maps_allow_and_deny_and_defaults_to_review() {
        assert_eq!(map_action_name("allow"), DecisionAction::Allow);
        assert_eq!(map_action_name("deny"), DecisionAction::Deny);
        assert_eq!(map_action_name("review"), DecisionAction::Review);
        assert_eq!(map_action_name("unexpected"), DecisionAction::Review);
    }

    #[test]
    fn map_label_to_action_uses_manifest_action_mapping() {
        let manifest = crate::test_support::base_manifest();

        assert_eq!(
            map_label_to_action(&manifest, "trusted"),
            DecisionAction::Allow
        );
        assert_eq!(
            map_label_to_action(&manifest, "review"),
            DecisionAction::Review
        );
        assert_eq!(
            map_label_to_action(&manifest, "suspicious"),
            DecisionAction::Review
        );
        assert_eq!(
            map_label_to_action(&manifest, "malicious"),
            DecisionAction::Deny
        );
        assert_eq!(
            map_label_to_action(&manifest, "unknown_label"),
            DecisionAction::Review
        );
    }
}
