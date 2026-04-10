// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::{
    ArtifactFormat, ContractCapability, ContractError, PolicyValidationError, SourceTrustLevel,
    Validate, VmTarget,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum QuantumMigrationMode {
    ClassicalOnly,
    #[default]
    HybridDualSign,
    PostQuantumOnly,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QuantumSecurityProfile {
    pub migration_mode: QuantumMigrationMode,
    #[serde(default)]
    pub transition_epoch_start: Option<u64>,
    #[serde(default)]
    pub classical_retirement_epoch: Option<u64>,
    #[serde(default = "default_min_signature_bundles")]
    pub min_signature_bundles: u8,
    #[serde(default)]
    pub pq_signature_schemes: Vec<String>,
}

const fn default_min_signature_bundles() -> u8 {
    1
}

impl Default for QuantumSecurityProfile {
    fn default() -> Self {
        Self {
            migration_mode: QuantumMigrationMode::ClassicalOnly,
            transition_epoch_start: None,
            classical_retirement_epoch: None,
            min_signature_bundles: default_min_signature_bundles(),
            pq_signature_schemes: vec![],
        }
    }
}

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
    #[serde(default, skip_serializing_if = "is_default_quantum_security")]
    pub quantum_security: QuantumSecurityProfile,
}

fn is_default_quantum_security(profile: &QuantumSecurityProfile) -> bool {
    profile == &QuantumSecurityProfile::default()
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
            quantum_security: QuantumSecurityProfile::default(),
        };
        policy.validate()?;
        Ok(policy)
    }

    pub fn with_quantum_security(
        mut self,
        quantum_security: QuantumSecurityProfile,
    ) -> Result<Self, ContractError> {
        self.quantum_security = quantum_security;
        self.validate()?;
        Ok(self)
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

    /// Returns true when post-quantum signatures are mandatory at `chain_epoch`.
    pub fn requires_post_quantum_signatures_at(&self, chain_epoch: u64) -> bool {
        match self.quantum_security.migration_mode {
            QuantumMigrationMode::ClassicalOnly => false,
            QuantumMigrationMode::PostQuantumOnly => self
                .quantum_security
                .transition_epoch_start
                .map(|start| chain_epoch >= start)
                .unwrap_or(true),
            QuantumMigrationMode::HybridDualSign => self
                .quantum_security
                .classical_retirement_epoch
                .map(|retire| chain_epoch >= retire)
                .unwrap_or(false),
        }
    }

    /// Returns required signature bundle count at `chain_epoch`.
    pub fn minimum_required_signature_bundles_at(&self, chain_epoch: u64) -> u8 {
        if self.requires_post_quantum_signatures_at(chain_epoch) {
            return self.quantum_security.min_signature_bundles.max(1);
        }
        match self.quantum_security.migration_mode {
            QuantumMigrationMode::HybridDualSign => self
                .quantum_security
                .transition_epoch_start
                .map(|start| {
                    if chain_epoch >= start {
                        self.quantum_security.min_signature_bundles.max(2)
                    } else {
                        1
                    }
                })
                .unwrap_or(self.quantum_security.min_signature_bundles.max(2)),
            _ => 1,
        }
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
        if !self.signature_required
            && self.quantum_security.migration_mode != QuantumMigrationMode::ClassicalOnly
        {
            return Err(PolicyValidationError::PolicyViolation(
                "quantum migration mode requires signature_required=true".into(),
            )
            .into());
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
        if self.quantum_security.migration_mode == QuantumMigrationMode::PostQuantumOnly
            && self.quantum_security.pq_signature_schemes.is_empty()
        {
            return Err(PolicyValidationError::PolicyViolation(
                "post_quantum_only requires at least one pq signature scheme".into(),
            )
            .into());
        }
        if self.quantum_security.migration_mode == QuantumMigrationMode::ClassicalOnly
            && !self.quantum_security.pq_signature_schemes.is_empty()
        {
            return Err(PolicyValidationError::PolicyViolation(
                "classical_only cannot declare pq signature schemes".into(),
            )
            .into());
        }
        let mut seen_schemes = BTreeSet::new();
        for scheme in &self.quantum_security.pq_signature_schemes {
            let canonical = scheme.trim();
            if canonical.is_empty()
                || !canonical
                    .chars()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || matches!(c, '_' | '-'))
            {
                return Err(PolicyValidationError::PolicyViolation(
                    "pq signature scheme id must be canonical snake-like lowercase".into(),
                )
                .into());
            }
            if !seen_schemes.insert(canonical) {
                return Err(PolicyValidationError::DuplicateCapability(format!(
                    "pq_signature_scheme:{canonical}"
                ))
                .into());
            }
        }
        if self.quantum_security.min_signature_bundles == 0 {
            return Err(PolicyValidationError::PolicyViolation(
                "min_signature_bundles must be at least 1".into(),
            )
            .into());
        }
        if self.quantum_security.migration_mode == QuantumMigrationMode::ClassicalOnly
            && (self.quantum_security.transition_epoch_start.is_some()
                || self.quantum_security.classical_retirement_epoch.is_some())
        {
            return Err(PolicyValidationError::PolicyViolation(
                "classical_only cannot define migration epochs".into(),
            )
            .into());
        }
        if let (Some(start), Some(retire)) = (
            self.quantum_security.transition_epoch_start,
            self.quantum_security.classical_retirement_epoch,
        ) && retire < start
        {
            return Err(PolicyValidationError::PolicyViolation(
                "classical_retirement_epoch cannot be before transition_epoch_start".into(),
            )
            .into());
        }
        if self.quantum_security.migration_mode == QuantumMigrationMode::PostQuantumOnly
            && self.quantum_security.classical_retirement_epoch.is_some()
        {
            return Err(PolicyValidationError::PolicyViolation(
                "post_quantum_only cannot define classical_retirement_epoch".into(),
            )
            .into());
        }

        Ok(())
    }
}
