pub mod contracts;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChainConfig {
    pub chain_name: String,
    pub network_id: u32,
}

impl Default for ChainConfig {
    fn default() -> Self {
        Self {
            chain_name: "AOXC-MAIN".to_string(),
            network_id: 1,
        }
    }
}
