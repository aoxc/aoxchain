use crate::auth::envelope::AuthEnvelope;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransactionEnvelope {
    pub chain_domain: String,
    pub tx_hash: [u8; 32],
    pub auth: AuthEnvelope,
    pub max_gas: u64,
    pub max_authority: u32,
    pub target_package: String,
    pub target_entrypoint: String,
}
