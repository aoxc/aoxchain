// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use aoxconfig::contracts::ContractsConfig;
use aoxcontract::{ContractDescriptor, ContractError, PolicyValidationError};

pub fn validate_runtime_acceptance(
    descriptor: &ContractDescriptor,
    config: &ContractsConfig,
) -> Result<(), ContractError> {
    if !config
        .artifact_policy
        .allowed_vm_targets
        .contains(&descriptor.manifest.vm_target)
    {
        return Err(PolicyValidationError::PolicyViolation(
            "vm target blocked by runtime config".into(),
        )
        .into());
    }
    Ok(())
}
