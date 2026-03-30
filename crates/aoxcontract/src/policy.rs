// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::{
    ArtifactFormat, ContractCapability, ContractError, PolicyValidationError, SourceTrustLevel,
    Validate, VmTarget,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContractPolicy {
    pub allowed_vm_targets: Vec<VmTarget>,
    pub allowed_artifact_formats: Vec<ArtifactFormat>,
    pub max_artifact_size: u64,
    pub allowed_capabilities: Vec<ContractCapability>,
    pub forbidden_capabilities: Vec<ContractCapability>,
    pub review_required: bool,
    pub signature_required: bool,
    pub source_trust_level: SourceTrustLevel,
}

impl ContractPolicy {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        allowed_vm_targets: Vec<VmTarget>,
        allowed_artifact_formats: Vec<ArtifactFormat>,
        max_artifact_size: u64,
        allowed_capabilities: Vec<ContractCapability>,
        forbidden_capabilities: Vec<ContractCapability>,
        review_required: bool,
        signature_required: bool,
        source_trust_level: SourceTrustLevel,
    ) -> Result<Self, ContractError> {
        let policy = Self {
            allowed_vm_targets,
            allowed_artifact_formats,
            max_artifact_size,
            allowed_capabilities,
            forbidden_capabilities,
            review_required,
            signature_required,
            source_trust_level,
        };
        policy.validate()?;
        Ok(policy)
    }

    pub fn enforces(
        &self,
        vm_target: &VmTarget,
        artifact_format: &ArtifactFormat,
        artifact_size: u64,
    ) -> Result<(), ContractError> {
        if !self.allowed_vm_targets.contains(vm_target) {
            return Err(
                PolicyValidationError::PolicyViolation("vm target not allowed".into()).into(),
            );
        }
        if !self.allowed_artifact_formats.contains(artifact_format) {
            return Err(PolicyValidationError::PolicyViolation(
                "artifact format not allowed".into(),
            )
            .into());
        }
        if artifact_size > self.max_artifact_size {
            return Err(crate::ArtifactValidationError::ArtifactTooLarge.into());
        }
        Ok(())
    }
}

impl Validate for ContractPolicy {
    fn validate(&self) -> Result<(), ContractError> {
        if self.allowed_vm_targets.is_empty() {
            return Err(PolicyValidationError::EmptyAllowedVmTargets.into());
        }
        if self.allowed_artifact_formats.is_empty() {
            return Err(PolicyValidationError::EmptyAllowedArtifactFormats.into());
        }
        if self.max_artifact_size == 0 {
            return Err(PolicyValidationError::InvalidArtifactSizeLimit.into());
        }

        let mut seen = BTreeSet::new();
        for capability in self
            .allowed_capabilities
            .iter()
            .chain(self.forbidden_capabilities.iter())
        {
            let inserted = seen.insert(capability);
            if !inserted {
                return Err(
                    PolicyValidationError::DuplicateCapability(format!("{capability:?}")).into(),
                );
            }
        }

        if self
            .forbidden_capabilities
            .contains(&ContractCapability::TreasurySensitive)
            && self
                .allowed_capabilities
                .contains(&ContractCapability::TreasurySensitive)
        {
            return Err(PolicyValidationError::PolicyViolation(
                "treasury_sensitive cannot be both allowed and forbidden".into(),
            )
            .into());
        }

        Ok(())
    }
}
