use std::collections::BTreeMap;

use chrono::Utc;
use sha2::{Digest, Sha256};

use aoxcontract::{ContractDescriptor, ContractId, ContractManifest, ContractStatus};

use crate::contract::error::ContractRegistryError;
use crate::contract::receipt::{
    ContractActivated, ContractDeprecated, ContractReceipt, ContractRegistered, ContractRevoked,
};
use crate::contract::record::{ManifestDigest, OnChainContractRecord, RegisteredAtHeight};

#[derive(Debug, Default)]
pub struct ContractRegistry {
    records: BTreeMap<ContractId, OnChainContractRecord>,
}

impl ContractRegistry {
    pub fn register_contract(
        &mut self,
        descriptor: ContractDescriptor,
        height: u64,
    ) -> Result<ContractReceipt, ContractRegistryError> {
        let contract_id = descriptor.contract_id.clone();
        if self.records.contains_key(&contract_id) {
            return Err(ContractRegistryError::AlreadyRegistered(contract_id.0));
        }

        let manifest_digest = manifest_digest_hex(&descriptor.manifest);
        let record = OnChainContractRecord {
            contract_id: contract_id.clone(),
            manifest: descriptor.manifest,
            status: ContractStatus::Registered,
            manifest_digest: ManifestDigest(manifest_digest),
            registered_at_height: RegisteredAtHeight(height),
            updated_at: Utc::now(),
        };

        self.records.insert(contract_id.clone(), record);

        Ok(ContractReceipt::Registered(ContractRegistered {
            contract_id,
        }))
    }

    pub fn activate_contract(
        &mut self,
        contract_id: &ContractId,
    ) -> Result<ContractReceipt, ContractRegistryError> {
        let record = self
            .records
            .get_mut(contract_id)
            .ok_or_else(|| ContractRegistryError::NotFound(contract_id.0.clone()))?;

        record.status = ContractStatus::Active;
        record.updated_at = Utc::now();

        Ok(ContractReceipt::Activated(ContractActivated {
            contract_id: contract_id.clone(),
        }))
    }

    pub fn deprecate_contract(
        &mut self,
        contract_id: &ContractId,
    ) -> Result<ContractReceipt, ContractRegistryError> {
        let record = self
            .records
            .get_mut(contract_id)
            .ok_or_else(|| ContractRegistryError::NotFound(contract_id.0.clone()))?;

        record.status = ContractStatus::Deprecated;
        record.updated_at = Utc::now();

        Ok(ContractReceipt::Deprecated(ContractDeprecated {
            contract_id: contract_id.clone(),
        }))
    }

    pub fn revoke_contract(
        &mut self,
        contract_id: &ContractId,
    ) -> Result<ContractReceipt, ContractRegistryError> {
        let record = self
            .records
            .get_mut(contract_id)
            .ok_or_else(|| ContractRegistryError::NotFound(contract_id.0.clone()))?;

        record.status = ContractStatus::Revoked;
        record.updated_at = Utc::now();

        Ok(ContractReceipt::Revoked(ContractRevoked {
            contract_id: contract_id.clone(),
        }))
    }

    pub fn get_contract(&self, contract_id: &ContractId) -> Option<&OnChainContractRecord> {
        self.records.get(contract_id)
    }

    pub fn all_contracts(&self) -> Vec<&OnChainContractRecord> {
        self.records.values().collect()
    }
}

fn manifest_digest_hex(manifest: &ContractManifest) -> String {
    let bytes = manifest.identity_material().unwrap_or_default();
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex::encode(hasher.finalize())
}