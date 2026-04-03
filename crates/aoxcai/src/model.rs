// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// High-level node task categories supported by the AI runtime.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AiTask {
    ValidatorAdmission,
    PeerScreening,
    TransactionScreening,
    ArtifactInspection,
}

/// Execution mode for the AI pipeline.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AiMode {
    Advisory,
    Enforced,
}

/// Deterministic severity contract for pre-model findings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FindingSeverity {
    Info,
    Warning,
    High,
    Critical,
}

/// Normalized model output label.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OutputLabel {
    Trusted,
    Review,
    Suspicious,
    Malicious,
    Unknown,
}

/// Canonical action mapping used across policy and fallback paths.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionName {
    Allow,
    Review,
    Deny,
}

/// Final action emitted by the engine.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DecisionAction {
    Allow,
    Review,
    Deny,
}

impl From<ActionName> for DecisionAction {
    fn from(value: ActionName) -> Self {
        match value {
            ActionName::Allow => Self::Allow,
            ActionName::Review => Self::Review,
            ActionName::Deny => Self::Deny,
        }
    }
}

/// Normalized signal collected before model inference.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InferenceSignal {
    pub name: String,
    pub value: String,
    pub weight_bps: u16,
    pub source: String,
    pub attributes: BTreeMap<String, String>,
}

impl InferenceSignal {
    #[must_use]
    pub fn new(
        name: impl Into<String>,
        value: impl Into<String>,
        weight_bps: u16,
        source: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            value: value.into(),
            weight_bps,
            source: source.into(),
            attributes: BTreeMap::new(),
        }
    }
}

/// Deterministic finding produced before model inference.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InferenceFinding {
    pub code: String,
    pub message: String,
    pub severity: FindingSeverity,
    pub attributes: BTreeMap<String, String>,
}

impl InferenceFinding {
    #[must_use]
    pub fn new(
        code: impl Into<String>,
        message: impl Into<String>,
        severity: FindingSeverity,
    ) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            severity,
            attributes: BTreeMap::new(),
        }
    }
}

/// Context associated with the evaluated subject.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InferenceContext {
    pub subject_id: String,
    pub subject_kind: String,
    pub metadata: BTreeMap<String, String>,
}

impl InferenceContext {
    #[must_use]
    pub fn new(subject_id: impl Into<String>, subject_kind: impl Into<String>) -> Self {
        Self {
            subject_id: subject_id.into(),
            subject_kind: subject_kind.into(),
            metadata: BTreeMap::new(),
        }
    }
}

/// Canonical backend request.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InferenceRequest {
    pub task: AiTask,
    pub mode: AiMode,
    pub context: InferenceContext,
    pub signals: Vec<InferenceSignal>,
    pub findings: Vec<InferenceFinding>,
    pub narrative: Option<String>,
}

impl InferenceRequest {
    #[must_use]
    pub fn subject_id(&self) -> &str {
        &self.context.subject_id
    }
}

/// Canonical backend output.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModelOutput {
    pub backend: String,
    pub model_id: String,
    pub label: OutputLabel,
    pub risk_bps: u16,
    pub confidence_bps: u16,
    pub rationale: String,
    pub recommended_action: Option<ActionName>,
    pub attributes: BTreeMap<String, String>,
}

/// Final fused assessment.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Assessment {
    pub action: DecisionAction,
    pub effective_risk_bps: u16,
    pub confidence_bps: u16,
    pub rationale: String,
}

/// Full engine decision report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DecisionReport {
    pub request: InferenceRequest,
    pub model_output: ModelOutput,
    pub assessment: Assessment,
    pub manifest_id: String,
    pub backend_type: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decision_action_from_action_name_preserves_semantics() {
        assert_eq!(
            DecisionAction::from(ActionName::Allow),
            DecisionAction::Allow
        );
        assert_eq!(
            DecisionAction::from(ActionName::Review),
            DecisionAction::Review
        );
        assert_eq!(DecisionAction::from(ActionName::Deny), DecisionAction::Deny);
    }

    #[test]
    fn constructor_helpers_initialize_empty_maps() {
        let signal = InferenceSignal::new("latency", "120", 900, "net");
        assert_eq!(signal.name, "latency");
        assert!(signal.attributes.is_empty());

        let finding =
            InferenceFinding::new("policy.missing", "missing policy", FindingSeverity::High);
        assert_eq!(finding.code, "policy.missing");
        assert!(finding.attributes.is_empty());

        let context = InferenceContext::new("node-7", "validator");
        assert_eq!(context.subject_id, "node-7");
        assert!(context.metadata.is_empty());
    }

    #[test]
    fn subject_id_accessor_returns_context_subject_id() {
        let request = InferenceRequest {
            task: AiTask::PeerScreening,
            mode: AiMode::Advisory,
            context: InferenceContext::new("peer-42", "peer"),
            signals: vec![],
            findings: vec![],
            narrative: None,
        };

        assert_eq!(request.subject_id(), "peer-42");
    }
}
