// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

/// WASM lane receipt extension.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WasmLaneReceipt {
    pub uploaded_code: Option<[u8; 32]>,
    pub instantiated_contract: Option<[u8; 32]>,
}
