// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use serde::{Deserialize, Serialize};

use crate::{ContractError, ContractId, ContractManifest};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContractDescriptor {
    pub contract_id: ContractId,
    pub manifest: ContractManifest,
    pub display_name: String,
}

impl ContractDescriptor {
    pub fn new(manifest: ContractManifest) -> Result<Self, ContractError> {
        let contract_id = ContractId::derive(&manifest)?;
        let display_name = manifest.metadata.display_name.clone();
        Ok(Self {
            contract_id,
            manifest,
            display_name,
        })
    }
}
