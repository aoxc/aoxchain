// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use thiserror::Error;

use aoxcontract::{
    ArtifactDigest, ArtifactFormat, ArtifactLocationKind, Compatibility, ContractArtifactRef,
    ContractCapability, ContractDescriptor, ContractError, ContractManifest, ContractMetadata,
    ContractPolicy, ContractVersion, Entrypoint, Integrity, NetworkClass, RuntimeFamily,
    SourceTrustLevel, VmTarget,
};

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
        }
    }
}

impl ContractManifestBuilder {
    /// Returns a builder with canonical AOXC defaults.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns a builder preconfigured for WASM artifacts.
    pub fn wasm() -> Self {
        Self::default().with_vm_target(VmTarget::Wasm)
    }

    /// Returns a builder preconfigured for EVM artifacts.
    pub fn evm() -> Self {
        Self::default().with_vm_target(VmTarget::Evm)
    }

    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn with_package(mut self, package: impl Into<String>) -> Self {
        self.package = Some(package.into());
        self
    }

    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        self.version = Some(version.into());
        self
    }

    pub fn with_contract_version(mut self, version: impl Into<String>) -> Self {
        self.contract_version = Some(version.into());
        self
    }

    pub fn with_vm_target(mut self, vm_target: VmTarget) -> Self {
        self.vm_target = Some(vm_target);
        self
    }

    pub fn with_artifact_digest(mut self, digest: ArtifactDigest) -> Self {
        self.artifact_digest = Some(digest);
        self
    }

    pub fn with_artifact_size(mut self, artifact_size: u64) -> Self {
        self.artifact_size = artifact_size;
        self
    }

    pub fn with_artifact_location(mut self, location: impl Into<String>) -> Self {
        self.artifact_location = Some(location.into());
        self
    }

    pub fn with_artifact_location_kind(mut self, kind: ArtifactLocationKind) -> Self {
        self.artifact_location_kind = kind;
        self
    }

    pub fn with_compression(mut self, compression: impl Into<String>) -> Self {
        self.compression = Some(compression.into());
        self
    }

    pub fn with_metadata(mut self, metadata: ContractMetadata) -> Self {
        self.metadata = Some(metadata);
        self
    }

    pub fn add_entrypoint(mut self, entrypoint: Entrypoint) -> Self {
        self.entrypoints.push(entrypoint);
        self
    }

    pub fn allow_capability(mut self, capability: ContractCapability) -> Self {
        self.allowed_capabilities.push(capability);
        self
    }

    pub fn forbid_capability(mut self, capability: ContractCapability) -> Self {
        self.forbidden_capabilities.push(capability);
        self
    }

    pub fn with_max_artifact_size(mut self, max_artifact_size: u64) -> Self {
        self.max_artifact_size = max_artifact_size;
        self
    }

    pub fn with_review_required(mut self, review_required: bool) -> Self {
        self.review_required = review_required;
        self
    }

    pub fn with_signature_required(mut self, signature_required: bool) -> Self {
        self.signature_required = signature_required;
        self
    }

    pub fn with_source_trust_level(mut self, source_trust_level: SourceTrustLevel) -> Self {
        self.source_trust_level = source_trust_level;
        self
    }

    pub fn with_schema_version(mut self, schema_version: u32) -> Self {
        self.schema_version = schema_version;
        self
    }

    pub fn with_minimum_schema_version(mut self, minimum_schema_version: u32) -> Self {
        self.minimum_schema_version = minimum_schema_version;
        self
    }

    pub fn with_supported_schema_versions(mut self, versions: Vec<u32>) -> Self {
        self.supported_schema_versions = versions;
        self
    }

    pub fn with_supported_runtime_families(mut self, families: Vec<RuntimeFamily>) -> Self {
        self.supported_runtime_families = families;
        self
    }

    pub fn with_supported_network_classes(mut self, classes: Vec<NetworkClass>) -> Self {
        self.supported_network_classes = classes;
        self
    }

    /// Builds a validated contract manifest.
    ///
    /// Validation discipline:
    /// - Mandatory builder inputs are extracted in deterministic order.
    /// - Runtime-family and network-class compatibility defaults are filled only
    ///   when the caller did not provide explicit values.
    /// - Downstream semantic validation remains delegated to `aoxcontract`.
    pub fn build(self) -> Result<ContractManifest, BuilderError> {
        let name = required(self.name, "name")?;
        let package = required(self.package, "package")?;
        let version = required(self.version, "version")?;
        let contract_version =
            ContractVersion(required(self.contract_version, "contract_version")?);
        let vm_target = required(self.vm_target, "vm_target")?;
        let digest = required(self.artifact_digest, "artifact_digest")?;
        let artifact_location = required(self.artifact_location, "artifact_location")?;

        let artifact_format = artifact_format_for_vm(&vm_target);
        let artifact_media_type = default_media_type_for_format(&artifact_format).to_string();

        let supported_runtime_families =
            normalize_supported_runtime_families(self.supported_runtime_families, &vm_target);

        let supported_network_classes =
            normalize_supported_network_classes(self.supported_network_classes);

        let artifact = ContractArtifactRef::new(
            digest.clone(),
            self.artifact_size,
            artifact_format.clone(),
            self.artifact_location_kind,
            artifact_location,
            self.compression,
            Some(artifact_media_type),
            vm_target.clone(),
        )?;

        let policy = ContractPolicy::new(
            vec![vm_target.clone()],
            vec![artifact.artifact_format.clone()],
            self.max_artifact_size,
            self.allowed_capabilities,
            self.forbidden_capabilities,
            self.review_required,
            self.signature_required,
            self.source_trust_level.clone(),
        )?;

        let compatibility = Compatibility::new(
            self.minimum_schema_version,
            self.supported_schema_versions,
            supported_runtime_families,
            supported_network_classes,
            vec![],
            false,
        )?;

        let metadata = self
            .metadata
            .unwrap_or_else(|| default_metadata_for_name(&name));

        let integrity = Integrity {
            digest: digest.clone(),
            artifact_size: artifact.artifact_size,
            artifact_format: artifact.artifact_format.clone(),
            media_type: artifact.media_type.clone(),
            signature_required: self.signature_required,
            source_trust_level: self.source_trust_level,
        };

        Ok(ContractManifest::new(
            name,
            package,
            version,
            contract_version,
            vm_target,
            artifact,
            self.entrypoints,
            digest,
            metadata,
            policy,
            compatibility,
            integrity,
            self.schema_version,
        )?)
    }

    /// Builds a validated descriptor by wrapping the built manifest.
    pub fn build_descriptor(self) -> Result<ContractDescriptor, BuilderError> {
        let manifest = self.build()?;
        Ok(ContractDescriptor::new(manifest)?)
    }
}

/// Returns a required builder field or a deterministic `MissingField` error.
fn required<T>(value: Option<T>, field: &'static str) -> Result<T, BuilderError> {
    value.ok_or(BuilderError::MissingField(field))
}

fn normalize_supported_runtime_families(
    families: Vec<RuntimeFamily>,
    vm_target: &VmTarget,
) -> Vec<RuntimeFamily> {
    if families.is_empty() {
        vec![runtime_family_for_vm(vm_target)]
    } else {
        families
    }
}

fn normalize_supported_network_classes(classes: Vec<NetworkClass>) -> Vec<NetworkClass> {
    if classes.is_empty() {
        default_supported_network_classes()
    } else {
        classes
    }
}

fn default_media_type_for_format(format: &ArtifactFormat) -> &'static str {
    match format {
        ArtifactFormat::EvmBytecode => "application/octet-stream",
        ArtifactFormat::WasmModule => "application/wasm",
        ArtifactFormat::Archive => "application/vnd.aox.archive",
        ArtifactFormat::ManifestLinked => "application/json",
    }
}

fn default_supported_network_classes() -> Vec<NetworkClass> {
    vec![
        NetworkClass::Mainnet,
        NetworkClass::Testnet,
        NetworkClass::Devnet,
    ]
}

fn artifact_format_for_vm(vm_target: &VmTarget) -> ArtifactFormat {
    match vm_target {
        VmTarget::Evm => ArtifactFormat::EvmBytecode,
        _ => ArtifactFormat::WasmModule,
    }
}

fn runtime_family_for_vm(vm_target: &VmTarget) -> RuntimeFamily {
    match vm_target {
        VmTarget::Evm => RuntimeFamily::Evm,
        VmTarget::Wasm => RuntimeFamily::Wasm,
        VmTarget::SuiLike | VmTarget::Custom(_) => RuntimeFamily::AoxVm,
    }
}

fn default_metadata_for_name(name: &str) -> ContractMetadata {
    ContractMetadata {
        display_name: name.to_string(),
        description: Some("Generated by AOXC SDK builder".to_string()),
        author: None,
        organization: None,
        source_reference: None,
        tags: vec!["sdk".to_string()],
        created_at: None,
        updated_at: None,
        audit_reference: None,
        notes: None,
    }
}

#[cfg(test)]
mod tests {
    use super::{BuilderError, ContractManifestBuilder};
    use aoxcontract::{
        ArtifactDigest, ArtifactDigestAlgorithm, ContractCapability, Entrypoint, NetworkClass,
        RuntimeFamily, VmTarget,
    };

    fn digest(seed: &str) -> ArtifactDigest {
        ArtifactDigest {
            algorithm: ArtifactDigestAlgorithm::Sha256,
            value: seed.to_string(),
        }
    }

    #[test]
    fn build_wasm_manifest_without_explicit_metadata() {
        let manifest = ContractManifestBuilder::wasm()
            .with_name("hello")
            .with_package("aox.hello")
            .with_artifact_digest(digest(
                "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            ))
            .with_artifact_location("ipfs://hello/contract.wasm")
            .add_entrypoint(
                Entrypoint::new("execute", VmTarget::Wasm, None, vec![])
                    .expect("entrypoint should build"),
            )
            .allow_capability(ContractCapability::StorageRead)
            .build()
            .expect("manifest should be built");

        assert_eq!(manifest.name, "hello");
        assert_eq!(manifest.metadata.display_name, "hello");
        assert_eq!(manifest.schema_version, 1);
        assert_eq!(
            manifest.compatibility.supported_runtime_families,
            vec![RuntimeFamily::Wasm]
        );
        assert_eq!(
            manifest.compatibility.supported_network_classes,
            vec![
                NetworkClass::Mainnet,
                NetworkClass::Testnet,
                NetworkClass::Devnet
            ]
        );
    }

    #[test]
    fn build_descriptor_works() {
        let descriptor = ContractManifestBuilder::evm()
            .with_name("evm_contract")
            .with_package("aox.evm")
            .with_artifact_digest(digest(
                "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
            ))
            .with_artifact_location("ipfs://evm/contract.bin")
            .add_entrypoint(
                Entrypoint::new("execute", VmTarget::Evm, None, vec![])
                    .expect("entrypoint should build"),
            )
            .build_descriptor()
            .expect("descriptor should be built");

        assert_eq!(descriptor.manifest.vm_target, VmTarget::Evm);
    }

    #[test]
    fn missing_required_field_returns_package_error_when_package_is_absent() {
        let err = ContractManifestBuilder::new()
            .with_name("incomplete")
            .build()
            .expect_err("builder should fail");

        assert!(matches!(err, BuilderError::MissingField("package")));
    }

    #[test]
    fn missing_required_field_returns_vm_target_error_when_prior_fields_exist() {
        let err = ContractManifestBuilder::new()
            .with_name("incomplete")
            .with_package("aox.incomplete")
            .with_version("1.0.0")
            .with_contract_version("1.0.0")
            .build()
            .expect_err("builder should fail");

        assert!(matches!(err, BuilderError::MissingField("vm_target")));
    }

    #[test]
    fn missing_required_field_returns_artifact_location_error_after_vm_and_digest_are_set() {
        let err = ContractManifestBuilder::new()
            .with_name("incomplete")
            .with_package("aox.incomplete")
            .with_version("1.0.0")
            .with_contract_version("1.0.0")
            .with_vm_target(VmTarget::Wasm)
            .with_artifact_digest(digest(
                "eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee",
            ))
            .build()
            .expect_err("builder should fail");

        assert!(matches!(
            err,
            BuilderError::MissingField("artifact_location")
        ));
    }

    #[test]
    fn explicit_compatibility_overrides_are_preserved() {
        let manifest = ContractManifestBuilder::new()
            .with_name("custom-compat")
            .with_package("aox.compat")
            .with_version("1.0.0")
            .with_contract_version("1.0.0")
            .with_vm_target(VmTarget::Custom("kernel-x".to_string()))
            .with_artifact_digest(digest(
                "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc",
            ))
            .with_artifact_location("ipfs://custom/contract.bin")
            .with_supported_runtime_families(vec![RuntimeFamily::AoxVm, RuntimeFamily::Wasm])
            .with_supported_network_classes(vec![NetworkClass::Airgapped])
            .add_entrypoint(
                Entrypoint::new(
                    "execute",
                    VmTarget::Custom("kernel-x".to_string()),
                    None,
                    vec![],
                )
                .expect("entrypoint should build"),
            )
            .build()
            .expect("manifest should build");

        assert_eq!(
            manifest.compatibility.supported_runtime_families,
            vec![RuntimeFamily::AoxVm, RuntimeFamily::Wasm]
        );
        assert_eq!(
            manifest.compatibility.supported_network_classes,
            vec![NetworkClass::Airgapped]
        );
    }

    #[test]
    fn empty_compatibility_overrides_fall_back_to_defaults() {
        let manifest = ContractManifestBuilder::new()
            .with_name("default-compat")
            .with_package("aox.default")
            .with_version("1.0.0")
            .with_contract_version("1.0.0")
            .with_vm_target(VmTarget::Wasm)
            .with_artifact_digest(digest(
                "dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd",
            ))
            .with_artifact_location("ipfs://default/contract.wasm")
            .with_supported_runtime_families(vec![])
            .with_supported_network_classes(vec![])
            .add_entrypoint(
                Entrypoint::new("execute", VmTarget::Wasm, None, vec![])
                    .expect("entrypoint should build"),
            )
            .build()
            .expect("manifest should build");

        assert_eq!(
            manifest.compatibility.supported_runtime_families,
            vec![RuntimeFamily::Wasm]
        );
        assert_eq!(
            manifest.compatibility.supported_network_classes,
            vec![
                NetworkClass::Mainnet,
                NetworkClass::Testnet,
                NetworkClass::Devnet
            ]
        );
    }
}
