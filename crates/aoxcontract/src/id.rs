use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{ContractError, IdentityDerivationError, canonical};

pub const CONTRACT_ID_DOMAIN_SEPARATOR: &str = "AOXC-CONTRACT-ID-V1";

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ContractId(pub String);

impl ContractId {
    pub fn derive(manifest: &crate::ContractManifest) -> Result<Self, ContractError> {
        let projection = canonical::identity_projection(manifest)?;
        let bytes = serde_json::to_vec(&projection)
            .map_err(|_| IdentityDerivationError::DerivationFailed)?;

        let mut hasher = Sha256::new();
        hasher.update(CONTRACT_ID_DOMAIN_SEPARATOR.as_bytes());
        hasher.update([0x1f]);
        hasher.update(bytes);
        let digest = hasher.finalize();
        Ok(Self(hex::encode(digest)))
    }
}
