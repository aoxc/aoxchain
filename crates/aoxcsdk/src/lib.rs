// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

//! AOXC developer SDK primitives.
//!
//! This crate currently focuses on typed contract-manifest composition helpers
//! that keep client-side integration deterministic and validation-first.

pub mod contracts;

/// Semantic version of this SDK crate, pulled from Cargo metadata.
pub const SDK_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Returns the SDK version string.
pub fn sdk_version() -> &'static str {
    SDK_VERSION
}

/// Small prelude for common SDK imports.
pub mod prelude {
    pub use crate::contracts::{BuilderError, ContractManifestBuilder};
    pub use crate::sdk_version;
}

#[cfg(test)]
mod tests {
    use super::{SDK_VERSION, sdk_version};

    #[test]
    fn version_matches_constant() {
        assert_eq!(sdk_version(), SDK_VERSION);
    }

    #[test]
    fn version_is_not_empty() {
        assert!(!sdk_version().trim().is_empty());
    }
}
