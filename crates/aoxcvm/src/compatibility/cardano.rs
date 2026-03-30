// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

/// Cardano-style compatibility manifest.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CardanoCompatibilityProfile {
    pub supports_utxo_create: bool,
    pub supports_utxo_spend: bool,
    pub supports_owner_validation: bool,
}

impl Default for CardanoCompatibilityProfile {
    fn default() -> Self {
        Self {
            supports_utxo_create: true,
            supports_utxo_spend: true,
            supports_owner_validation: true,
        }
    }
}
