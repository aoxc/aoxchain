// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

use crate::{CompatibilityError, ContractError, Validate, VmTarget};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeFamily {
    AoxVm,
    Evm,
    Wasm,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NetworkClass {
    Mainnet,
    Testnet,
    Devnet,
    Airgapped,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Compatibility {
    pub minimum_schema_version: u32,
    pub supported_schema_versions: Vec<u32>,
    pub supported_runtime_families: Vec<RuntimeFamily>,
    pub supported_network_classes: Vec<NetworkClass>,
    pub incompatible_targets: Vec<VmTarget>,
    pub deprecated: bool,
}

impl Compatibility {
    pub fn new(
        minimum_schema_version: u32,
        supported_schema_versions: Vec<u32>,
        supported_runtime_families: Vec<RuntimeFamily>,
        supported_network_classes: Vec<NetworkClass>,
        incompatible_targets: Vec<VmTarget>,
        deprecated: bool,
    ) -> Result<Self, ContractError> {
        let value = Self {
            minimum_schema_version,
            supported_schema_versions,
            supported_runtime_families,
            supported_network_classes,
            incompatible_targets,
            deprecated,
        };
        value.validate()?;
        Ok(value)
    }

    pub fn supports_schema_version(&self, schema_version: u32) -> bool {
        schema_version >= self.minimum_schema_version
            && self.supported_schema_versions.contains(&schema_version)
    }
}

impl Validate for Compatibility {
    fn validate(&self) -> Result<(), ContractError> {
        if self.minimum_schema_version == 0 {
            return Err(CompatibilityError::MissingMinimumSchemaVersion.into());
        }
        if self.supported_schema_versions.is_empty() {
            return Err(CompatibilityError::EmptySupportedSchemaVersions.into());
        }
        if self.supported_runtime_families.is_empty() {
            return Err(CompatibilityError::EmptySupportedRuntimeFamilies.into());
        }
        if self.supported_network_classes.is_empty() {
            return Err(CompatibilityError::EmptySupportedNetworkClasses.into());
        }
        let unique: BTreeSet<u32> = self.supported_schema_versions.iter().copied().collect();
        if unique.len() != self.supported_schema_versions.len() {
            return Err(CompatibilityError::DuplicateSupportedSchemaVersions.into());
        }
        if !self
            .supported_schema_versions
            .contains(&self.minimum_schema_version)
        {
            return Err(CompatibilityError::MinimumSchemaVersionNotSupported.into());
        }
        Ok(())
    }
}
