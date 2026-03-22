use std::collections::BTreeMap;

use aoxcontract::ContractId;
use aoxcore::contract::record::OnChainContractRecord;

#[derive(Debug, Default, Clone)]
pub struct ContractStore {
    by_id: BTreeMap<ContractId, OnChainContractRecord>,
}

impl ContractStore {
    pub fn put(&mut self, record: OnChainContractRecord) {
        self.by_id.insert(record.contract_id.clone(), record);
    }

    pub fn get(&self, contract_id: &ContractId) -> Option<&OnChainContractRecord> {
        self.by_id.get(contract_id)
    }

    pub fn list(&self) -> Vec<&OnChainContractRecord> {
        self.by_id.values().collect()
    }
}
