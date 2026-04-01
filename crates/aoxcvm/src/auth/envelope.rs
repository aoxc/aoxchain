use crate::auth::scheme::AuthScheme;
use crate::domains::Domain;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthEnvelope {
    pub scheme: AuthScheme,
    pub signer_key_id: [u8; 32],
    pub domain: Domain,
    pub nonce: u64,
    pub expiry_epoch: u64,
    pub payload_digest: [u8; 32],
    pub capability_scope: Vec<String>,
}
