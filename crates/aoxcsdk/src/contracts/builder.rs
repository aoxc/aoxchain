use thiserror::Error;

use aoxcontract::{
    ArtifactDigest, ArtifactFormat, ArtifactLocationKind, Compatibility, ContractArtifactRef,
    ContractCapability, ContractError, ContractManifest, ContractMetadata, ContractPolicy,
    ContractVersion, Entrypoint, Integrity, SourceTrustLevel, VmTarget,
};

#[derive(Debug, Error)]
pub enum BuilderError {
    #[error(transparent)]
    Contract(#[from] ContractError),
    #[error("missing field: {0}")]
    MissingField(&'static str),
}

#[derive(Debug, Default, Clone)]
pub struct ContractManifestBuilder {
    pub name: Option<String>,
    pub package: Option<String>,
    pub version: Option<String>,
    pub contract_version: Option<String>,
    pub vm_target: Option<VmTarget>,
    pub artifact_digest: Option<ArtifactDigest>,
    pub artifact_location: Option<String>,
    pub metadata: Option<ContractMetadata>,
    pub entrypoints: Vec<Entrypoint>,
    pub allowed_capabilities: Vec<ContractCapability>,
}

impl ContractManifestBuilder {
    pub fn new() -> Self {
        Self::default()
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
    pub fn with_artifact_location(mut self, location: impl Into<String>) -> Self {
        self.artifact_location = Some(location.into());
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

    pub fn build(self) -> Result<ContractManifest, BuilderError> {
        let vm_target = self
            .vm_target
            .ok_or(BuilderError::MissingField("vm_target"))?;
        let digest = self
            .artifact_digest
            .ok_or(BuilderError::MissingField("artifact_digest"))?;
        let artifact = ContractArtifactRef::new(
            digest.clone(),
            4096,
            match vm_target {
                VmTarget::Evm => ArtifactFormat::EvmBytecode,
                _ => ArtifactFormat::WasmModule,
            },
            ArtifactLocationKind::Uri,
            self.artifact_location
                .ok_or(BuilderError::MissingField("artifact_location"))?,
            None,
            Some(match vm_target {
                VmTarget::Evm => "application/octet-stream".to_string(),
                _ => "application/wasm".to_string(),
            }),
            vm_target.clone(),
        )?;
        let policy = ContractPolicy::new(
            vec![vm_target.clone()],
            vec![artifact.artifact_format.clone()],
            10 * 1024 * 1024,
            self.allowed_capabilities,
            vec![],
            true,
            true,
            SourceTrustLevel::ReviewRequired,
        )?;
        let compatibility = Compatibility::new(1, vec![1], vec![], vec![], vec![], false)?;
        let metadata = self
            .metadata
            .ok_or(BuilderError::MissingField("metadata"))?;
        let integrity = Integrity {
            digest: digest.clone(),
            artifact_size: artifact.artifact_size,
            artifact_format: artifact.artifact_format.clone(),
            media_type: artifact.media_type.clone(),
            signature_required: true,
            source_trust_level: SourceTrustLevel::ReviewRequired,
        };
        Ok(ContractManifest::new(
            self.name.ok_or(BuilderError::MissingField("name"))?,
            self.package.ok_or(BuilderError::MissingField("package"))?,
            self.version.ok_or(BuilderError::MissingField("version"))?,
            ContractVersion(
                self.contract_version
                    .ok_or(BuilderError::MissingField("contract_version"))?,
            ),
            vm_target,
            artifact,
            self.entrypoints,
            digest,
            metadata,
            policy,
            compatibility,
            integrity,
            1,
        )?)
    }
}
