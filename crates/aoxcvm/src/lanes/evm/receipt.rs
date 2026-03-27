// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

/// EVM-specific receipt extension model.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvmLaneReceipt {
    pub contract_address: Option<[u8; 20]>,
    pub reverted: bool,
}
