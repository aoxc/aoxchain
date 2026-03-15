use crate::{error::AiError, manifest::ModelManifest, model::AiTask};
use std::{collections::HashMap, fs, path::Path};

/// Stores loaded manifests and task bindings for runtime lookup.
#[derive(Debug, Default)]
pub struct ModelRegistry {
    manifests: HashMap<String, ModelManifest>,
    bindings: HashMap<AiTask, String>,
}

impl ModelRegistry {
    /// Creates an empty registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Loads all YAML manifests from a directory.
    pub fn load_dir(mut self, dir: impl AsRef<Path>) -> Result<Self, AiError> {
        let dir_ref = dir.as_ref();

        let entries = fs::read_dir(dir_ref).map_err(|err| AiError::Io {
            path: dir_ref.display().to_string(),
            reason: err.to_string(),
        })?;

        for entry in entries {
            let entry = entry.map_err(|err| AiError::Io {
                path: dir_ref.display().to_string(),
                reason: err.to_string(),
            })?;
            let path = entry.path();

            let is_yaml = path
                .extension()
                .and_then(|s| s.to_str())
                .map(|ext| ext.eq_ignore_ascii_case("yaml") || ext.eq_ignore_ascii_case("yml"))
                .unwrap_or(false);

            if is_yaml {
                let manifest = ModelManifest::from_yaml_file(&path)?;
                self.register(manifest)?;
            }
        }

        Ok(self)
    }

    /// Registers a validated manifest in the registry.
    pub fn register(&mut self, manifest: ModelManifest) -> Result<(), AiError> {
        manifest.validate()?;

        let id = manifest.id().to_owned();

        if self.manifests.contains_key(&id) {
            return Err(AiError::ManifestValidation(format!(
                "duplicate manifest id '{}'",
                id
            )));
        }

        for task in &manifest.spec.bindings.default_for_tasks {
            self.bindings.insert(*task, id.clone());
        }

        self.manifests.insert(id, manifest);
        Ok(())
    }

    /// Returns the manifest bound to the supplied task.
    pub fn resolve_for_task(&self, task: AiTask) -> Result<&ModelManifest, AiError> {
        let model_id = self
            .bindings
            .get(&task)
            .ok_or_else(|| AiError::BindingNotFound(format!("{task:?}")))?;

        self.manifests
            .get(model_id)
            .ok_or_else(|| AiError::ModelNotFound(model_id.clone()))
    }

    /// Returns a manifest by explicit identifier.
    pub fn get(&self, id: &str) -> Result<&ModelManifest, AiError> {
        self.manifests
            .get(id)
            .ok_or_else(|| AiError::ModelNotFound(id.to_owned()))
    }

    /// Binds a task to a manifest identifier without validating existence.
    ///
    /// Prefer `bind_checked` in runtime configuration paths.
    pub fn bind(&mut self, task: AiTask, model_id: impl Into<String>) {
        self.bindings.insert(task, model_id.into());
    }

    /// Binds a task to an existing manifest identifier.
    pub fn bind_checked(
        &mut self,
        task: AiTask,
        model_id: impl Into<String>,
    ) -> Result<(), AiError> {
        let model_id = model_id.into();

        if !self.manifests.contains_key(&model_id) {
            return Err(AiError::ModelNotFound(model_id));
        }

        self.bindings.insert(task, model_id);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{error::AiError, model::AiTask, test_support::base_manifest};
    use std::{
        fs,
        time::{SystemTime, UNIX_EPOCH},
    };

    fn unique_temp_dir() -> std::path::PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time must be after unix epoch")
            .as_nanos();

        let dir = std::env::temp_dir().join(format!("aoxcai_registry_test_{nanos}"));
        fs::create_dir_all(&dir).expect("temporary test directory must be created");
        dir
    }

    #[test]
    fn register_and_get_returns_manifest_by_id() {
        let mut registry = ModelRegistry::new();
        let manifest = base_manifest();
        let id = manifest.metadata.id.clone();

        registry
            .register(manifest)
            .expect("manifest registration must succeed");

        let resolved = registry
            .get(&id)
            .expect("manifest must be retrievable by id");

        assert_eq!(resolved.metadata.id, id);
    }

    #[test]
    fn resolve_for_task_returns_default_bound_manifest() {
        let mut registry = ModelRegistry::new();
        let manifest = base_manifest();

        registry
            .register(manifest)
            .expect("manifest registration must succeed");

        let resolved = registry
            .resolve_for_task(AiTask::ValidatorAdmission)
            .expect("task binding must resolve");

        assert_eq!(resolved.metadata.id, "test-model");
    }

    #[test]
    fn resolve_for_task_returns_binding_not_found_when_unbound() {
        let registry = ModelRegistry::new();

        let err = registry
            .resolve_for_task(AiTask::ArtifactInspection)
            .expect_err("unbound task must fail");

        match err {
            AiError::BindingNotFound(task) => {
                assert!(task.contains("ArtifactInspection"));
            }
            other => panic!("unexpected error: {other}"),
        }
    }

    #[test]
    fn get_returns_model_not_found_for_unknown_id() {
        let registry = ModelRegistry::new();

        let err = registry
            .get("missing-model")
            .expect_err("unknown model id must fail");

        match err {
            AiError::ModelNotFound(id) => assert_eq!(id, "missing-model"),
            other => panic!("unexpected error: {other}"),
        }
    }

    #[test]
    fn bind_overrides_existing_task_binding() {
        let mut registry = ModelRegistry::new();

        let mut manifest_a = base_manifest();
        manifest_a.metadata.id = "model-a".to_owned();

        let mut manifest_b = base_manifest();
        manifest_b.metadata.id = "model-b".to_owned();

        registry
            .register(manifest_a)
            .expect("first manifest registration must succeed");
        registry
            .register(manifest_b)
            .expect("second manifest registration must succeed");

        registry.bind(AiTask::ValidatorAdmission, "model-b");

        let resolved = registry
            .resolve_for_task(AiTask::ValidatorAdmission)
            .expect("task binding must resolve");

        assert_eq!(resolved.metadata.id, "model-b");
    }

    #[test]
    fn bind_can_create_dangling_reference_and_resolve_then_fails_with_model_not_found() {
        let mut registry = ModelRegistry::new();
        registry.bind(AiTask::ValidatorAdmission, "missing-model");

        let err = registry
            .resolve_for_task(AiTask::ValidatorAdmission)
            .expect_err("dangling binding must fail");

        match err {
            AiError::ModelNotFound(id) => assert_eq!(id, "missing-model"),
            other => panic!("unexpected error: {other}"),
        }
    }

    #[test]
    fn load_dir_loads_yaml_manifests_and_ignores_non_yaml_files() {
        let dir = unique_temp_dir();

        let manifest_path = dir.join("model.yaml");
        let ignored_path = dir.join("notes.txt");

        let manifest_yaml = r#"
api_version: "aoxc.ai/v1"
kind: "ModelManifest"

metadata:
  id: "loaded-model"
  name: "Loaded Model"
  version: "1.0.0"
  description: "Loaded from disk for registry tests."
  owner: "AOXC Core"
  tags: []
  created_at: "2026-03-15T00:00:00Z"
  updated_at: "2026-03-15T00:00:00Z"

spec:
  enabled: true

  lifecycle:
    hot_reload: false
    reload_strategy: "none"
    immutable_fields: []

  bindings:
    default_for_tasks:
      - "validator_admission"
    optional_tasks: []
    node_scopes: []
    tenants: []

  backend:
    type: "heuristic"
    driver: "builtin"
    priority: 1
    timeout_ms: 1000
    max_retries: 0
    retry_backoff_ms: 0
    circuit_breaker:
      enabled: false
      failure_threshold: 0
      cooldown_ms: 0
    remote_http: null
    local_candle: null
    local_onnx: null
    heuristic:
      ruleset: "default"
      anomaly_keywords:
        - "revoked"
    ensemble: null

  input:
    format: "structured_json"
    include_narrative: true
    include_signals: true
    include_findings: true
    include_context_metadata: true
    include_evidence_refs: false
    max_signal_count: 64
    max_finding_count: 32
    max_evidence_refs: 0
    truncate_strategy: "weight_desc"
    signal_encoding: "typed"
    subject:
      required_fields:
        - "subject_id"
        - "subject_kind"
        - "task"
        - "mode"
      optional_fields: []
    prompt:
      style: "plain"
      system_instructions: ""
      narrative_template: ""

  output:
    format: "normalized_json"
    required_fields:
      - "label"
      - "risk_bps"
      - "confidence_bps"
      - "rationale"
    optional_fields:
      - "recommended_action"
      - "attributes"
    mapping:
      label_field: "label"
      risk_bps_field: "risk_bps"
      confidence_bps_field: "confidence_bps"
      rationale_field: "rationale"
      recommended_action_field: "recommended_action"
      attributes_field: "attributes"
    validation:
      allowed_labels:
        - "trusted"
        - "review"
        - "suspicious"
        - "malicious"
        - "unknown"
      risk_bps_min: 0
      risk_bps_max: 10000
      confidence_bps_min: 0
      confidence_bps_max: 10000
      on_invalid_output: "reject"

  decision:
    mode: "enforced"
    thresholds:
      allow_max_risk_bps: 2499
      review_max_risk_bps: 6999
      deny_min_risk_bps: 7000
    confidence:
      minimum_confidence_bps: 3500
      low_confidence_action: "review"
    fusion:
      strategy: "weighted"
      weights:
        model_risk_bps: 6000
        deterministic_risk_bps: 4000
      deterministic_overrides:
        deny_on_critical_finding: true
        deny_on_revoked_identity: true
        deny_on_invalid_quorum_proof: true
        review_on_missing_context: true
        review_on_model_timeout: true
    actions:
      trusted: "allow"
      review: "review"
      suspicious: "review"
      malicious: "deny"
      unknown: "review"

  fallback:
    enabled: false
    backend: null
    action_on_backend_error: "review"
    action_on_timeout: "review"
    action_on_schema_error: "review"
    action_on_unreachable_backend: "review"
    action_on_empty_response: "review"

  observability:
    metrics:
      enabled: false
      namespace: "test"
      labels: {}
    tracing:
      enabled: false
      include_request_hash: false
      include_subject_id: false
      include_backend_latency: false
    audit:
      enabled: false
      log_decision_input: false
      log_decision_output: false
      redact_fields: []
      persist_reports: false
      persist_path: ""

  security:
    manifest_signature:
      required: false
      algorithm: "none"
      public_key_path: null
      signature_field: ""
    allowed_endpoints: []
    allow_private_networks: false
    allow_file_backends: false

  compatibility:
    min_node_version: "0.1.0"
    max_node_version: null
    supported_tasks:
      - "validator_admission"
    supported_modes:
      - "advisory"
      - "enforced"

  extensions: {}
"#;

        fs::write(&manifest_path, manifest_yaml).expect("manifest file must be written");
        fs::write(&ignored_path, "this is not yaml").expect("ignored file must be written");

        let registry = ModelRegistry::new()
            .load_dir(&dir)
            .expect("registry must load valid manifest directory");

        let manifest = registry
            .get("loaded-model")
            .expect("loaded manifest must be retrievable");

        assert_eq!(manifest.metadata.id, "loaded-model");

        let resolved = registry
            .resolve_for_task(AiTask::ValidatorAdmission)
            .expect("loaded task binding must resolve");

        assert_eq!(resolved.metadata.id, "loaded-model");

        fs::remove_dir_all(&dir).expect("temporary directory must be removed");
    }

    #[test]
    fn load_dir_returns_io_error_for_missing_directory() {
        let missing_dir = std::env::temp_dir().join("aoxcai_registry_missing_dir_never_created");

        let err = ModelRegistry::new()
            .load_dir(&missing_dir)
            .expect_err("missing directory must fail");

        match err {
            AiError::Io { path, .. } => {
                assert!(path.contains("aoxcai_registry_missing_dir_never_created"));
            }
            other => panic!("unexpected error: {other}"),
        }
    }
}
