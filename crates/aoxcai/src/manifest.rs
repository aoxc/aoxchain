use crate::{
    error::AiError,
    model::{AiMode, AiTask},
};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, fs, path::Path};

/// Represents the root runtime manifest for a loadable AOXC AI model.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModelManifest {
    pub api_version: String,
    pub kind: String,
    pub metadata: Metadata,
    pub spec: Spec,
}

impl ModelManifest {
    /// Loads and validates a manifest from a YAML file.
    pub fn from_yaml_file(path: impl AsRef<Path>) -> Result<Self, AiError> {
        let path_ref = path.as_ref();
        let raw = fs::read_to_string(path_ref).map_err(|err| AiError::Io {
            path: path_ref.display().to_string(),
            reason: err.to_string(),
        })?;

        let manifest: Self =
            serde_yaml::from_str(&raw).map_err(|err| AiError::ManifestParse(err.to_string()))?;

        manifest.validate()?;
        Ok(manifest)
    }

    /// Applies structural validation required by the runtime.
    pub fn validate(&self) -> Result<(), AiError> {
        if self.api_version.trim().is_empty() {
            return Err(AiError::ManifestValidation(
                "api_version must not be empty".to_owned(),
            ));
        }

        if self.kind != "ModelManifest" {
            return Err(AiError::ManifestValidation(format!(
                "unsupported kind '{}'",
                self.kind
            )));
        }

        if self.metadata.id.trim().is_empty() {
            return Err(AiError::ManifestValidation(
                "metadata.id must not be empty".to_owned(),
            ));
        }

        if self.spec.backend.r#type.trim().is_empty() {
            return Err(AiError::ManifestValidation(
                "spec.backend.type must not be empty".to_owned(),
            ));
        }

        if self.spec.output.required_fields.is_empty() {
            return Err(AiError::ManifestValidation(
                "spec.output.required_fields must not be empty".to_owned(),
            ));
        }

        Ok(())
    }

    /// Returns true when this manifest is enabled for runtime use.
    pub fn is_enabled(&self) -> bool {
        self.spec.enabled
    }

    /// Returns the manifest identifier.
    pub fn id(&self) -> &str {
        &self.metadata.id
    }

    /// Returns the configured backend type.
    pub fn backend_type(&self) -> &str {
        &self.spec.backend.r#type
    }

    /// Returns true when the manifest declares the supplied task as default.
    pub fn binds_task(&self, task: AiTask) -> bool {
        self.spec.bindings.default_for_tasks.contains(&task)
    }

    /// Returns the decision mode declared by the manifest.
    pub fn decision_mode(&self) -> AiMode {
        self.spec.decision.mode
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Metadata {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub owner: String,
    pub tags: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Spec {
    pub enabled: bool,
    pub lifecycle: Lifecycle,
    pub bindings: Bindings,
    pub backend: Backend,
    pub input: Input,
    pub output: Output,
    pub decision: Decision,
    pub fallback: Fallback,
    pub observability: Observability,
    pub security: Security,
    pub compatibility: Compatibility,
    pub extensions: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Lifecycle {
    pub hot_reload: bool,
    pub reload_strategy: String,
    pub immutable_fields: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Bindings {
    pub default_for_tasks: Vec<AiTask>,
    pub optional_tasks: Vec<AiTask>,
    pub node_scopes: Vec<String>,
    pub tenants: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Backend {
    #[serde(rename = "type")]
    pub r#type: String,
    pub driver: String,
    pub priority: u32,
    pub timeout_ms: u64,
    pub max_retries: u32,
    pub retry_backoff_ms: u64,
    pub circuit_breaker: CircuitBreaker,
    pub remote_http: Option<RemoteHttpBackend>,
    pub local_candle: Option<LocalCandleBackend>,
    pub local_onnx: Option<LocalOnnxBackend>,
    pub heuristic: Option<HeuristicBackend>,
    pub ensemble: Option<EnsembleBackend>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CircuitBreaker {
    pub enabled: bool,
    pub failure_threshold: u32,
    pub cooldown_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoteHttpBackend {
    pub endpoint: String,
    pub method: String,
    pub headers: BTreeMap<String, String>,
    pub auth: Auth,
    pub tls: Tls,
    pub rate_limit: RateLimit,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Auth {
    pub mode: String,
    pub env_key: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Tls {
    pub enabled: bool,
    pub verify_peer: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RateLimit {
    pub requests_per_minute: u32,
    pub burst: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LocalCandleBackend {
    pub model_path: String,
    pub tokenizer_path: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LocalOnnxBackend {
    pub model_path: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HeuristicBackend {
    pub ruleset: String,
    pub anomaly_keywords: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EnsembleBackend {
    pub members: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Input {
    pub format: String,
    pub include_narrative: bool,
    pub include_signals: bool,
    pub include_findings: bool,
    pub include_context_metadata: bool,
    pub include_evidence_refs: bool,
    pub max_signal_count: usize,
    pub max_finding_count: usize,
    pub max_evidence_refs: usize,
    pub truncate_strategy: String,
    pub signal_encoding: String,
    pub subject: SubjectShape,
    pub prompt: Prompt,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SubjectShape {
    pub required_fields: Vec<String>,
    pub optional_fields: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Prompt {
    pub style: String,
    pub system_instructions: String,
    pub narrative_template: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Output {
    pub format: String,
    pub required_fields: Vec<String>,
    pub optional_fields: Vec<String>,
    pub mapping: OutputMapping,
    pub validation: OutputValidation,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OutputMapping {
    pub label_field: String,
    pub risk_bps_field: String,
    pub confidence_bps_field: String,
    pub rationale_field: String,
    pub recommended_action_field: String,
    pub attributes_field: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OutputValidation {
    pub allowed_labels: Vec<String>,
    pub risk_bps_min: u16,
    pub risk_bps_max: u16,
    pub confidence_bps_min: u16,
    pub confidence_bps_max: u16,
    pub on_invalid_output: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Decision {
    pub mode: AiMode,
    pub thresholds: Thresholds,
    pub confidence: Confidence,
    pub fusion: Fusion,
    pub actions: ActionMap,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Thresholds {
    pub allow_max_risk_bps: u16,
    pub review_max_risk_bps: u16,
    pub deny_min_risk_bps: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Confidence {
    pub minimum_confidence_bps: u16,
    pub low_confidence_action: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Fusion {
    pub strategy: String,
    pub weights: FusionWeights,
    pub deterministic_overrides: DeterministicOverrides,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FusionWeights {
    pub model_risk_bps: u16,
    pub deterministic_risk_bps: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeterministicOverrides {
    pub deny_on_critical_finding: bool,
    pub deny_on_revoked_identity: bool,
    pub deny_on_invalid_quorum_proof: bool,
    pub review_on_missing_context: bool,
    pub review_on_model_timeout: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActionMap {
    pub trusted: String,
    pub review: String,
    pub suspicious: String,
    pub malicious: String,
    pub unknown: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Fallback {
    pub enabled: bool,
    pub backend: Option<String>,
    pub action_on_backend_error: String,
    pub action_on_timeout: String,
    pub action_on_schema_error: String,
    pub action_on_unreachable_backend: String,
    pub action_on_empty_response: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Observability {
    pub metrics: Metrics,
    pub tracing: Tracing,
    pub audit: Audit,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Metrics {
    pub enabled: bool,
    pub namespace: String,
    pub labels: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Tracing {
    pub enabled: bool,
    pub include_request_hash: bool,
    pub include_subject_id: bool,
    pub include_backend_latency: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Audit {
    pub enabled: bool,
    pub log_decision_input: bool,
    pub log_decision_output: bool,
    pub redact_fields: Vec<String>,
    pub persist_reports: bool,
    pub persist_path: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Security {
    pub manifest_signature: ManifestSignature,
    pub allowed_endpoints: Vec<String>,
    pub allow_private_networks: bool,
    pub allow_file_backends: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ManifestSignature {
    pub required: bool,
    pub algorithm: String,
    pub public_key_path: Option<String>,
    pub signature_field: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Compatibility {
    pub min_node_version: String,
    pub max_node_version: Option<String>,
    pub supported_tasks: Vec<AiTask>,
    pub supported_modes: Vec<AiMode>,
}
