// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use serde::{Deserialize, Serialize};

use crate::{ContractError, ManifestValidationError, Validate, VmTarget};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Entrypoint {
    pub name: String,
    pub vm_target: VmTarget,
    pub selector: Option<String>,
    pub required_capabilities: Vec<crate::ContractCapability>,
}

impl Entrypoint {
    pub fn new(
        name: impl Into<String>,
        vm_target: VmTarget,
        selector: Option<String>,
        required_capabilities: Vec<crate::ContractCapability>,
    ) -> Result<Self, ContractError> {
        let entrypoint = Self {
            name: name.into(),
            vm_target,
            selector,
            required_capabilities,
        };
        entrypoint.validate()?;
        Ok(entrypoint)
    }
}

impl Validate for Entrypoint {
    fn validate(&self) -> Result<(), ContractError> {
        let trimmed = self.name.trim();
        if trimmed.is_empty() {
            return Err(ManifestValidationError::InvalidContractName.into());
        }
        if trimmed.len() > 128 {
            return Err(ManifestValidationError::InvalidContractName.into());
        }
        if !trimmed
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | ':' | '-'))
        {
            return Err(ManifestValidationError::InvalidContractName.into());
        }
        Ok(())
    }
}
