#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IdentityObject {
    pub identity_id: [u8; 32],
    pub active_key_id: [u8; 32],
    pub recovery_enabled: bool,
}
