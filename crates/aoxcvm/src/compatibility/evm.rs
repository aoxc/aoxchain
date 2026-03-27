// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

/// EVM compatibility manifest.
///
/// The manifest is declarative so conformance tests can target it later.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvmCompatibilityProfile {
    pub supports_json_rpc: bool,
    pub supports_contract_deploy: bool,
    pub supports_contract_call: bool,
    pub supports_receipts: bool,
}

impl Default for EvmCompatibilityProfile {
    fn default() -> Self {
        Self {
            supports_json_rpc: true,
            supports_contract_deploy: true,
            supports_contract_call: true,
            supports_receipts: true,
        }
    }
}
