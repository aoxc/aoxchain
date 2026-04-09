//! Shared test fixtures for auth modules.
//!
//! Centralizing these values prevents drift between test modules when
//! algorithm support or envelope validation constraints evolve.

use crate::auth::scheme::SignatureAlgorithm;

/// Returns an in-range deterministic signature size for fixture generation.
pub(crate) const fn fixture_signature_len(algorithm: SignatureAlgorithm) -> usize {
    match algorithm {
        SignatureAlgorithm::Ed25519 => 64,
        SignatureAlgorithm::EcdsaP256 => 64,
        SignatureAlgorithm::MlDsa65 => 3309,
        SignatureAlgorithm::MlDsa87 => 3500,
        SignatureAlgorithm::SlhDsa128s => 3000,
    }
}
