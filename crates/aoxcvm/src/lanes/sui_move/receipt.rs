// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

/// Sui / Move lane receipt extension.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SuiMoveLaneReceipt {
    pub published_package: Option<[u8; 32]>,
    pub mutated_object: Option<[u8; 32]>,
}
