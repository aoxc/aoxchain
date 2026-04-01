//! Authentication configuration defaults for AOXCVM nodes.

use crate::auth::scheme::{AuthProfile, SignatureAlgorithm};

/// Configuration knob set for authentication-surface hardening.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AuthConfig {
    /// Verification policy profile.
    pub profile: AuthProfile,
    /// Primary signature algorithm accepted for operator identities.
    pub primary_algorithm: SignatureAlgorithm,
    /// Whether key-rotation transactions must include a post-quantum signer.
    pub require_pq_for_rotation: bool,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            profile: AuthProfile::HybridMandatory,
            primary_algorithm: SignatureAlgorithm::MlDsa65,
            require_pq_for_rotation: true,
        }
    }
}
