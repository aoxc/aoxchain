// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

/// Sui-style compatibility manifest.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SuiCompatibilityProfile {
    pub supports_package_publish: bool,
    pub supports_object_creation: bool,
    pub supports_object_ownership: bool,
}

impl Default for SuiCompatibilityProfile {
    fn default() -> Self {
        Self {
            supports_package_publish: true,
            supports_object_creation: true,
            supports_object_ownership: true,
        }
    }
}
