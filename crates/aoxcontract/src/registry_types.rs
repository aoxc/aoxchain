// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use serde::{Deserialize, Serialize};

use crate::{ContractDescriptor, ContractError, ContractId};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContractStatus {
    Draft,
    Registered,
    Active,
    Deprecated,
    Revoked,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContractActivationMode {
    Manual,
    Governance,
    Genesis,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ContractRecordVersion(pub u32);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RegisteredContract {
    pub contract_id: ContractId,
    pub descriptor: ContractDescriptor,
    pub status: ContractStatus,
    pub activation_mode: ContractActivationMode,
    pub record_version: ContractRecordVersion,
}

impl RegisteredContract {
    pub fn new(
        descriptor: ContractDescriptor,
        status: ContractStatus,
        activation_mode: ContractActivationMode,
        record_version: ContractRecordVersion,
    ) -> Result<Self, ContractError> {
        let contract_id = descriptor.contract_id.clone();
        Ok(Self {
            contract_id,
            descriptor,
            status,
            activation_mode,
            record_version,
        })
    }
}
