// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use crate::types::ChainStatus;

#[derive(Debug, Clone)]
pub struct QueryService {
    pub chain_id: String,
}

impl Default for QueryService {
    fn default() -> Self {
        Self {
            chain_id: "AOX-MAIN".to_string(),
        }
    }
}

impl QueryService {
    #[must_use]
    pub fn get_chain_status(&self, height: u64, syncing: bool) -> ChainStatus {
        ChainStatus {
            chain_id: self.chain_id.clone(),
            height,
            syncing,
        }
    }
}
