/// Filecoin/EAM-style compatibility manifest.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FilecoinCompatibilityProfile {
    pub supports_eam_records: bool,
    pub supports_eth_address_mapping: bool,
}

impl Default for FilecoinCompatibilityProfile {
    fn default() -> Self {
        Self {
            supports_eam_records: true,
            supports_eth_address_mapping: true,
        }
    }
}
