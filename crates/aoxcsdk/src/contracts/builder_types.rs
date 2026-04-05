/// Builder-level failure surface for AOXC contract manifest construction.
///
/// Error policy:
/// - Structural and semantic contract-domain failures are propagated from
///   `aoxcontract` as transparent `Contract` errors.
/// - Missing required builder inputs are normalized into deterministic
///   `MissingField` errors so callers can fail early with stable diagnostics.
#[derive(Debug, Error)]
pub enum BuilderError {
    #[error(transparent)]
    Contract(#[from] ContractError),

    #[error("missing field: {0}")]
    MissingField(&'static str),
}

/// Fluent builder for AOXC contract manifests.
///
/// Design objectives:
/// - Provide deterministic defaults for non-critical fields.
/// - Preserve explicit caller intent for compatibility and policy overrides.
/// - Fail closed when required construction inputs are absent.
#[derive(Debug, Clone)]
pub struct ContractManifestBuilder {
    pub name: Option<String>,
    pub package: Option<String>,
    pub version: Option<String>,
    pub contract_version: Option<String>,
    pub vm_target: Option<VmTarget>,
    pub artifact_digest: Option<ArtifactDigest>,
    pub artifact_size: u64,
    pub artifact_location: Option<String>,
    pub artifact_location_kind: ArtifactLocationKind,
    pub compression: Option<String>,
    pub metadata: Option<ContractMetadata>,
    pub entrypoints: Vec<Entrypoint>,
    pub allowed_capabilities: Vec<ContractCapability>,
    pub forbidden_capabilities: Vec<ContractCapability>,
    pub max_artifact_size: u64,
    pub review_required: bool,
    pub signature_required: bool,
    pub source_trust_level: SourceTrustLevel,
    pub schema_version: u32,
    pub minimum_schema_version: u32,
    pub supported_schema_versions: Vec<u32>,
    pub supported_runtime_families: Vec<RuntimeFamily>,
    pub supported_network_classes: Vec<NetworkClass>,
    pub execution_profile: Option<ExecutionProfile>,
    pub contract_class: Option<ContractClass>,
    pub capability_profile: Option<CapabilityProfile>,
    pub policy_profile: Option<PolicyProfile>,
}

impl Default for ContractManifestBuilder {
    fn default() -> Self {
        Self {
            name: None,
            package: None,
            version: Some("1.0.0".to_string()),
            contract_version: Some("1.0.0".to_string()),
            vm_target: None,
            artifact_digest: None,
            artifact_size: 4096,
            artifact_location: None,
            artifact_location_kind: ArtifactLocationKind::Uri,
            compression: None,
            metadata: None,
            entrypoints: Vec::new(),
            allowed_capabilities: Vec::new(),
            forbidden_capabilities: Vec::new(),
            max_artifact_size: 10 * 1024 * 1024,
            review_required: true,
            signature_required: true,
            source_trust_level: SourceTrustLevel::ReviewRequired,
            schema_version: 1,
            minimum_schema_version: 1,
            supported_schema_versions: vec![1],
            supported_runtime_families: Vec::new(),
            supported_network_classes: default_supported_network_classes(),
            execution_profile: None,
            contract_class: None,
            capability_profile: None,
            policy_profile: None,
        }
    }
}
