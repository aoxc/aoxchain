use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Identifies the high-level AI task requested by the node.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AiTask {
    ValidatorAdmission,
    PeerScreening,
    TransactionScreening,
    ArtifactInspection,
}

/// Defines whether the AI pipeline is advisory or enforced.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AiMode {
    Advisory,
    Enforced,
}

/// Represents a normalized signal supplied to the inference layer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InferenceSignal {
    pub name: String,
    pub value: String,
    pub weight_bps: u16,
    pub source: String,
    pub attributes: BTreeMap<String, String>,
}

impl InferenceSignal {
    /// Constructs a new normalized signal.
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

/// Represents a deterministic finding collected before model inference.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InferenceFinding {
    pub code: String,
    pub message: String,
    pub severity: String,
    pub attributes: BTreeMap<String, String>,
}

impl InferenceFinding {
    /// Constructs a new finding.
    pub fn new(
        code: impl Into<String>,
        message: impl Into<String>,
        severity: impl Into<String>,
    ) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            severity: severity.into(),
            attributes: BTreeMap::new(),
        }
    }
}

/// Captures contextual metadata associated with a subject under evaluation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InferenceContext {
    pub subject_id: String,
    pub subject_kind: String,
    pub metadata: BTreeMap<String, String>,
}

impl InferenceContext {
    /// Constructs a new inference context.
    pub fn new(subject_id: impl Into<String>, subject_kind: impl Into<String>) -> Self {
        Self {
            subject_id: subject_id.into(),
            subject_kind: subject_kind.into(),
            metadata: BTreeMap::new(),
        }
    }
}

/// Represents the canonical request submitted to a backend.
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
    /// Returns a stable subject identifier reference.
    pub fn subject_id(&self) -> &str {
        &self.context.subject_id
    }
}

/// Represents a backend-normalized inference output.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModelOutput {
    pub backend: String,
    pub model_id: String,
    pub label: String,
    pub risk_bps: u16,
    pub confidence_bps: u16,
    pub rationale: String,
    pub recommended_action: Option<String>,
    pub attributes: BTreeMap<String, String>,
}

/// Describes the final action to be enforced or advised by the node.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DecisionAction {
    Allow,
    Review,
    Deny,
}

/// Represents the final fused assessment emitted by the engine.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Assessment {
    pub action: DecisionAction,
    pub effective_risk_bps: u16,
    pub confidence_bps: u16,
    pub rationale: String,
}

/// Represents the full decision report returned by the AI engine.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DecisionReport {
    pub request: InferenceRequest,
    pub model_output: ModelOutput,
    pub assessment: Assessment,
    pub manifest_id: String,
    pub backend_type: String,
}
