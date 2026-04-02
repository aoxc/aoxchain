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
    /// Whether hybrid profile use is explicitly authorized for a migration window.
    pub allow_hybrid_migration_window: bool,
}

impl AuthConfig {
    /// Returns the profile actually enforced at runtime.
    ///
    /// Even if `profile` is set to `HybridMandatory`, runtime falls back to
    /// `PostQuantumStrict` unless `allow_hybrid_migration_window` is true.
    pub const fn effective_profile(self) -> AuthProfile {
        match (self.profile, self.allow_hybrid_migration_window) {
            (AuthProfile::HybridMandatory, false) => AuthProfile::PostQuantumStrict,
            (profile, _) => profile,
        }
    }
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            profile: AuthProfile::PostQuantumStrict,
            primary_algorithm: SignatureAlgorithm::MlDsa65,
            require_pq_for_rotation: true,
            allow_hybrid_migration_window: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::AuthConfig;
    use crate::auth::scheme::AuthProfile;

    #[test]
    fn hybrid_profile_is_gated_without_migration_window() {
        let cfg = AuthConfig {
            profile: AuthProfile::HybridMandatory,
            ..AuthConfig::default()
        };
        assert_eq!(cfg.effective_profile(), AuthProfile::PostQuantumStrict);
    }

    #[test]
    fn hybrid_profile_can_be_opted_in_for_controlled_migration() {
        let cfg = AuthConfig {
            profile: AuthProfile::HybridMandatory,
            allow_hybrid_migration_window: true,
            ..AuthConfig::default()
        };
        assert_eq!(cfg.effective_profile(), AuthProfile::HybridMandatory);
    }
}
