use serde::{Deserialize, Serialize};

use aoxcontract::ContractId;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContractReceipt {
    Registered(ContractRegistered),
    Activated(ContractActivated),
    Deprecated(ContractDeprecated),
    Revoked(ContractRevoked),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContractRegistered {
    pub contract_id: ContractId,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContractActivated {
    pub contract_id: ContractId,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContractDeprecated {
    pub contract_id: ContractId,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContractRevoked {
    pub contract_id: ContractId,
}
