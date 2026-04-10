// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use crate::cpu_opt::CpuCapabilities;
use serde::{Deserialize, Serialize};

/// Cryptographic verification posture used by upper layers.
///
/// The crate deliberately models posture at policy-level rather than hard-coding
/// one concrete signature suite. This keeps runtime behavior migration-friendly
/// while preserving deterministic admission decisions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CryptoScheme {
    /// Classical-only verification surface.
    Classical,
    /// Hybrid verification: classical + PQ verification required.
    Hybrid,
    /// PQ-primary verification posture.
    PostQuantumPrimary,
}

/// Runtime activation state for cryptographic migration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProfileStage {
    /// System runs classical verification only.
    ClassicalOnly,
    /// System accepts only hybrid transactions to train migration pathways.
    HybridRequired,
    /// System requires PQ-primary verification.
    PostQuantumRequired,
}

/// Deterministic policy object for migration-safe scheme selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CryptoPolicy {
    pub stage: ProfileStage,
    /// Minimum number of independent signatures expected by policy.
    pub min_signatures: u8,
    /// When true, low-capability hosts may stay hybrid to avoid partial outages.
    pub allow_hybrid_fallback: bool,
}

/// Deterministic summary of signatures/proofs found in an incoming transaction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ProofSummary {
    /// Number of valid classical signatures attached to the transaction.
    pub classical_signatures: u8,
    /// Number of valid post-quantum signatures attached to the transaction.
    pub pq_signatures: u8,
}

/// Stable verification requirement set derived from runtime scheme.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct VerificationRequirements {
    pub require_classical: bool,
    pub require_post_quantum: bool,
    pub min_total_signatures: u8,
}

/// Deterministic admission failures for cryptographic policy enforcement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AdmissionError {
    /// Policy is malformed (e.g., 0 required signatures).
    InvalidPolicy,
    /// Not enough signatures to satisfy policy.
    InsufficientTotalSignatures { required: u8, received: u8 },
    /// Classical signature is required but missing.
    MissingClassicalSignature,
    /// Post-quantum signature is required but missing.
    MissingPostQuantumSignature,
}

impl CryptoPolicy {
    /// Conservative bootstrap policy for development and bring-up networks.
    #[must_use]
    pub const fn bootstrap() -> Self {
        Self {
            stage: ProfileStage::ClassicalOnly,
            min_signatures: 1,
            allow_hybrid_fallback: true,
        }
    }

    /// Migration-ready policy with dual verification.
    #[must_use]
    pub const fn hybrid_gate() -> Self {
        Self {
            stage: ProfileStage::HybridRequired,
            min_signatures: 2,
            allow_hybrid_fallback: true,
        }
    }

    /// Production-oriented strict PQ posture.
    #[must_use]
    pub const fn pq_primary() -> Self {
        Self {
            stage: ProfileStage::PostQuantumRequired,
            min_signatures: 2,
            allow_hybrid_fallback: false,
        }
    }

    /// Validates static policy invariants.
    #[must_use]
    pub const fn is_valid(self) -> bool {
        self.min_signatures > 0
    }

    /// Returns true if transition is monotonic and does not downgrade cryptographic posture.
    #[must_use]
    pub const fn can_transition_to(self, next: Self) -> bool {
        let curr_stage_rank = match self.stage {
            ProfileStage::ClassicalOnly => 0_u8,
            ProfileStage::HybridRequired => 1_u8,
            ProfileStage::PostQuantumRequired => 2_u8,
        };
        let next_stage_rank = match next.stage {
            ProfileStage::ClassicalOnly => 0_u8,
            ProfileStage::HybridRequired => 1_u8,
            ProfileStage::PostQuantumRequired => 2_u8,
        };

        next_stage_rank >= curr_stage_rank && next.min_signatures >= self.min_signatures
    }
}

/// Deterministically choose runtime scheme posture from policy and host profile.
///
/// The function intentionally does not inspect wall-clock data or non-deterministic
/// external state. Nodes with equal policy and capabilities always choose the same
/// result.
#[must_use]
pub const fn choose_runtime_scheme(policy: CryptoPolicy, cpu: CpuCapabilities) -> CryptoScheme {
    match policy.stage {
        ProfileStage::ClassicalOnly => CryptoScheme::Classical,
        ProfileStage::HybridRequired => CryptoScheme::Hybrid,
        ProfileStage::PostQuantumRequired => {
            if cpu.supports_wide_parallelism() || !policy.allow_hybrid_fallback {
                CryptoScheme::PostQuantumPrimary
            } else {
                CryptoScheme::Hybrid
            }
        }
    }
}

/// Maps a runtime scheme and policy to explicit signature requirements.
#[must_use]
pub const fn verification_requirements(
    scheme: CryptoScheme,
    policy: CryptoPolicy,
) -> VerificationRequirements {
    match scheme {
        CryptoScheme::Classical => VerificationRequirements {
            require_classical: true,
            require_post_quantum: false,
            min_total_signatures: policy.min_signatures,
        },
        CryptoScheme::Hybrid => VerificationRequirements {
            require_classical: true,
            require_post_quantum: true,
            min_total_signatures: policy.min_signatures,
        },
        CryptoScheme::PostQuantumPrimary => VerificationRequirements {
            require_classical: false,
            require_post_quantum: true,
            min_total_signatures: policy.min_signatures,
        },
    }
}

/// Evaluates whether an incoming proof summary satisfies deterministic policy checks.
pub const fn evaluate_admission(
    policy: CryptoPolicy,
    cpu: CpuCapabilities,
    proof: ProofSummary,
) -> Result<CryptoScheme, AdmissionError> {
    if !policy.is_valid() {
        return Err(AdmissionError::InvalidPolicy);
    }

    let scheme = choose_runtime_scheme(policy, cpu);
    let req = verification_requirements(scheme, policy);
    let total = proof.classical_signatures.saturating_add(proof.pq_signatures);

    if total < req.min_total_signatures {
        return Err(AdmissionError::InsufficientTotalSignatures {
            required: req.min_total_signatures,
            received: total,
        });
    }
    if req.require_classical && proof.classical_signatures == 0 {
        return Err(AdmissionError::MissingClassicalSignature);
    }
    if req.require_post_quantum && proof.pq_signatures == 0 {
        return Err(AdmissionError::MissingPostQuantumSignature);
    }

    Ok(scheme)
}

#[cfg(test)]
mod tests {
    use super::{
        AdmissionError, CryptoPolicy, CryptoScheme, ProfileStage, ProofSummary, evaluate_admission,
        choose_runtime_scheme,
    };
    use crate::cpu_opt::CpuCapabilities;

    #[test]
    fn classical_stage_stays_classical() {
        let policy = CryptoPolicy::bootstrap();
        let cpu = CpuCapabilities::portable();
        assert_eq!(choose_runtime_scheme(policy, cpu), CryptoScheme::Classical);
    }

    #[test]
    fn hybrid_stage_is_stable_across_profiles() {
        let policy = CryptoPolicy::hybrid_gate();
        let portable = CpuCapabilities::portable();
        let avx2 = CpuCapabilities::from_flags(true, true, false, false);

        assert_eq!(choose_runtime_scheme(policy, portable), CryptoScheme::Hybrid);
        assert_eq!(choose_runtime_scheme(policy, avx2), CryptoScheme::Hybrid);
    }

    #[test]
    fn pq_stage_prefers_pq_on_vector_capable_hosts() {
        let policy = CryptoPolicy {
            stage: ProfileStage::PostQuantumRequired,
            min_signatures: 2,
            allow_hybrid_fallback: true,
        };
        let cpu = CpuCapabilities::from_flags(true, true, false, false);
        assert_eq!(
            choose_runtime_scheme(policy, cpu),
            CryptoScheme::PostQuantumPrimary
        );
    }

    #[test]
    fn pq_stage_can_fallback_when_explicitly_allowed() {
        let policy = CryptoPolicy {
            stage: ProfileStage::PostQuantumRequired,
            min_signatures: 2,
            allow_hybrid_fallback: true,
        };
        let cpu = CpuCapabilities::portable();
        assert_eq!(choose_runtime_scheme(policy, cpu), CryptoScheme::Hybrid);
    }

    #[test]
    fn pq_primary_policy_disallows_fallback() {
        let policy = CryptoPolicy::pq_primary();
        let cpu = CpuCapabilities::portable();
        assert_eq!(
            choose_runtime_scheme(policy, cpu),
            CryptoScheme::PostQuantumPrimary
        );
    }

    #[test]
    fn transition_policy_rejects_downgrade() {
        let from = CryptoPolicy::pq_primary();
        let to = CryptoPolicy::bootstrap();
        assert!(!from.can_transition_to(to));
    }

    #[test]
    fn transition_policy_accepts_monotonic_upgrade() {
        let from = CryptoPolicy::bootstrap();
        let to = CryptoPolicy::hybrid_gate();
        assert!(from.can_transition_to(to));
    }

    #[test]
    fn hybrid_admission_requires_both_signature_domains() {
        let policy = CryptoPolicy::hybrid_gate();
        let cpu = CpuCapabilities::portable();
        let proof = ProofSummary {
            classical_signatures: 2,
            pq_signatures: 0,
        };
        assert_eq!(
            evaluate_admission(policy, cpu, proof),
            Err(AdmissionError::MissingPostQuantumSignature)
        );
    }

    #[test]
    fn pq_admission_accepts_pq_only_bundle() {
        let policy = CryptoPolicy::pq_primary();
        let cpu = CpuCapabilities::portable();
        let proof = ProofSummary {
            classical_signatures: 0,
            pq_signatures: 2,
        };
        assert_eq!(
            evaluate_admission(policy, cpu, proof),
            Ok(CryptoScheme::PostQuantumPrimary)
        );
    }

    #[test]
    fn admission_rejects_invalid_zero_signature_policy() {
        let policy = CryptoPolicy {
            stage: ProfileStage::HybridRequired,
            min_signatures: 0,
            allow_hybrid_fallback: true,
        };
        let cpu = CpuCapabilities::portable();
        let proof = ProofSummary::default();
        assert_eq!(
            evaluate_admission(policy, cpu, proof),
            Err(AdmissionError::InvalidPolicy)
        );
    }
}
