#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapabilityObject {
    pub capability_id: [u8; 32],
    pub namespace: String,
    pub action: String,
    pub resource: String,
}
