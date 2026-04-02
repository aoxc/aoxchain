// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use serde::{Deserialize, Serialize};

use crate::{
    ArtifactDigest, Compatibility, ContractArtifactRef, ContractError, ContractMetadata,
    ContractPolicy, Entrypoint, ExecutionProfile, ManifestValidationError, Validate,
};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ContractVersion(pub String);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VmTarget {
    Evm,
    Wasm,
    SuiLike,
    Custom(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContractManifest {
    pub name: String,
    pub package: String,
    pub version: String,
    pub contract_version: ContractVersion,
    pub vm_target: VmTarget,
    pub artifact: ContractArtifactRef,
    pub entrypoints: Vec<Entrypoint>,
    pub digest: ArtifactDigest,
    pub metadata: ContractMetadata,
    pub policy: ContractPolicy,
    pub compatibility: Compatibility,
    pub execution_profile: ExecutionProfile,
    pub integrity: crate::Integrity,
    pub schema_version: u32,
}

impl ContractManifest {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        name: impl Into<String>,
        package: impl Into<String>,
        version: impl Into<String>,
        contract_version: ContractVersion,
        vm_target: VmTarget,
        artifact: ContractArtifactRef,
        entrypoints: Vec<Entrypoint>,
        digest: ArtifactDigest,
        metadata: ContractMetadata,
        policy: ContractPolicy,
        compatibility: Compatibility,
        integrity: crate::Integrity,
        schema_version: u32,
    ) -> Result<Self, ContractError> {
        // Phase-2 execution profile derivation must happen before `vm_target`
        // is moved into the manifest instance. This preserves ownership clarity
        // and avoids unnecessary cloning while keeping manifest/profile VM target
        // coupling deterministic and explicit.
        let execution_profile = ExecutionProfile::phase2_default(&vm_target);

        let manifest = Self {
            name: name.into(),
            package: package.into(),
            version: version.into(),
            contract_version,
            vm_target,
            artifact,
            entrypoints,
            digest,
            metadata,
            policy,
            compatibility,
            execution_profile,
            integrity,
            schema_version,
        };

        manifest.validate()?;
        Ok(manifest)
    }

    pub fn identity_material(&self) -> Result<Vec<u8>, ContractError> {
        crate::canonical::canonical_manifest_bytes(self)
    }
}

fn is_valid_symbolic_name(value: &str) -> bool {
    !value.trim().is_empty()
        && value.len() <= 128
        && value
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.'))
}

fn is_semantic_like_version(value: &str) -> bool {
    let mut parts = value.split('.');
    let major = parts.next();
    let minor = parts.next();
    let patch = parts.next();

    major.zip(minor).zip(patch).is_some()
        && parts.next().is_none()
        && value
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-'))
}

impl Validate for ContractManifest {
    fn validate(&self) -> Result<(), ContractError> {
        if !is_valid_symbolic_name(&self.name) {
            return Err(if self.name.trim().is_empty() {
                ManifestValidationError::EmptyContractName
            } else {
                ManifestValidationError::InvalidContractName
            }
            .into());
        }

        if self.package.trim().is_empty() {
            return Err(ManifestValidationError::EmptyPackage.into());
        }

        if !is_semantic_like_version(&self.version)
            || !is_semantic_like_version(&self.contract_version.0)
        {
            return Err(ManifestValidationError::InvalidVersionFormat.into());
        }

        if self.schema_version == 0 {
            return Err(ManifestValidationError::MissingSchemaVersion.into());
        }

        self.artifact.validate()?;
        self.policy.validate()?;
        self.compatibility.validate()?;
        self.integrity.validate()?;

        if self.vm_target != self.artifact.declared_vm_target {
            return Err(ManifestValidationError::VmTargetMismatch.into());
        }

        if self.execution_profile.vm_target != self.vm_target {
            return Err(ManifestValidationError::ExecutionProfileVmTargetMismatch.into());
        }

        if self.digest != self.artifact.artifact_digest || self.digest != self.integrity.digest {
            return Err(ManifestValidationError::DigestAlgorithmMismatch.into());
        }

        if !self
            .compatibility
            .supports_schema_version(self.schema_version)
        {
            return Err(crate::CompatibilityError::CompatibilityMismatch.into());
        }

        if self.entrypoints.is_empty() {
            return Err(ManifestValidationError::EmptyEntrypoints.into());
        }

        self.policy.enforces(
            &self.vm_target,
            &self.artifact.artifact_format,
            self.artifact.artifact_size,
        )?;

        if self.integrity.artifact_size != self.artifact.artifact_size
            || self.integrity.artifact_format != self.artifact.artifact_format
            || self.integrity.media_type != self.artifact.media_type
        {
            return Err(ManifestValidationError::IntegrityMetadataMismatch.into());
        }

        let mut names = std::collections::BTreeSet::new();
        for entrypoint in &self.entrypoints {
            entrypoint.validate()?;

            if entrypoint.vm_target != self.vm_target {
                return Err(ManifestValidationError::VmTargetMismatch.into());
            }

            if !names.insert(entrypoint.name.clone()) {
                return Err(
                    ManifestValidationError::DuplicateEntrypoint(entrypoint.name.clone()).into(),
                );
            }
        }

        Ok(())
    }
}
