use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use aoxcontract::{ContractId, ContractManifest, ContractStatus};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ManifestDigest(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RegisteredAtHeight(pub u64);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OnChainContractRecord {
    pub contract_id: ContractId,
    pub manifest: ContractManifest,
    pub status: ContractStatus,
    pub manifest_digest: ManifestDigest,
    pub registered_at_height: RegisteredAtHeight,
    pub updated_at: DateTime<Utc>,
}
