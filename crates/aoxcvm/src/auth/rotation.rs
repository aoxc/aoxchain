//! Key-rotation policy helpers for hybrid and post-quantum AOXCVM deployments.

use crate::auth::scheme::SignatureAlgorithm;

/// Rotation plan from an old active signer set into a new set.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RotationPlan {
    pub previous: Vec<SignatureAlgorithm>,
    pub next: Vec<SignatureAlgorithm>,
}

impl RotationPlan {
    /// Returns true when the plan preserves at least one post-quantum key in both generations,
    /// preventing temporary regressions to classical-only authentication.
    pub fn preserves_quantum_continuity(&self) -> bool {
        let previous_has_pq = self.previous.iter().any(|algorithm| algorithm.is_post_quantum());
        let next_has_pq = self.next.iter().any(|algorithm| algorithm.is_post_quantum());
        previous_has_pq && next_has_pq
    }

    /// Returns true when there is at least one shared algorithm between both generations.
    /// This is useful for gradual migrations where verifier trust roots overlap.
    pub fn has_overlap(&self) -> bool {
        self.previous
            .iter()
            .any(|left| self.next.iter().any(|right| left == right))
    }
}

#[cfg(test)]
mod tests {
    use crate::auth::scheme::SignatureAlgorithm;

    use super::RotationPlan;

    #[test]
    fn continuity_requires_pq_in_both_sets() {
        let plan = RotationPlan {
            previous: vec![SignatureAlgorithm::Ed25519, SignatureAlgorithm::MlDsa65],
            next: vec![SignatureAlgorithm::EcdsaP256],
        };
        assert!(!plan.preserves_quantum_continuity());
    }

    #[test]
    fn overlap_detects_shared_algorithms() {
        let plan = RotationPlan {
            previous: vec![SignatureAlgorithm::MlDsa65],
            next: vec![SignatureAlgorithm::MlDsa65, SignatureAlgorithm::MlDsa87],
        };
        assert!(plan.has_overlap());
        assert!(plan.preserves_quantum_continuity());
    }
}
