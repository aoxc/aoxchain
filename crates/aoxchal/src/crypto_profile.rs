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

/// Machine-readable proof composition summary provided by upper layers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ProofSummary {
    pub classical_signatures: u8,
    pub pq_signatures: u8,
}

/// Structured admission denial reasons.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AdmissionFailure {
    InsufficientSignatureCount,
    MissingClassicalSignature,
    MissingPostQuantumSignature,
    ClassicalNotPermitted,
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

/// Evaluate request proof composition against policy and runtime scheme.
pub fn evaluate_admission(
    policy: CryptoPolicy,
    cpu: CpuCapabilities,
    proof: ProofSummary,
) -> Result<CryptoScheme, AdmissionFailure> {
    let total_signatures = proof
        .classical_signatures
        .saturating_add(proof.pq_signatures);

    if total_signatures < policy.min_signatures {
        return Err(AdmissionFailure::InsufficientSignatureCount);
    }

    let runtime_scheme = choose_runtime_scheme(policy, cpu);

    match runtime_scheme {
        CryptoScheme::Classical => {
            if proof.classical_signatures == 0 {
                return Err(AdmissionFailure::MissingClassicalSignature);
            }
        }
        CryptoScheme::Hybrid => {
            if proof.classical_signatures == 0 {
                return Err(AdmissionFailure::MissingClassicalSignature);
            }
            if proof.pq_signatures == 0 {
                return Err(AdmissionFailure::MissingPostQuantumSignature);
            }
        }
        CryptoScheme::PostQuantumPrimary => {
            if proof.pq_signatures == 0 {
                return Err(AdmissionFailure::MissingPostQuantumSignature);
            }
            if proof.classical_signatures > 0 {
                return Err(AdmissionFailure::ClassicalNotPermitted);
            }
        }
    }

    Ok(runtime_scheme)
}

#[cfg(test)]
mod tests {
    use super::{
        AdmissionFailure, CryptoPolicy, CryptoScheme, ProfileStage, ProofSummary,
        choose_runtime_scheme, evaluate_admission,
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

        assert_eq!(
            choose_runtime_scheme(policy, portable),
            CryptoScheme::Hybrid
        );
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
    fn admission_rejects_missing_minimum_signature_count() {
        let err = evaluate_admission(
            CryptoPolicy::hybrid_gate(),
            CpuCapabilities::portable(),
            ProofSummary {
                classical_signatures: 1,
                pq_signatures: 0,
            },
        )
        .expect_err("should reject if count is below min signature threshold");

        assert_eq!(err, AdmissionFailure::InsufficientSignatureCount);
    }

    #[test]
    fn hybrid_runtime_requires_both_classical_and_pq() {
        let err = evaluate_admission(
            CryptoPolicy::hybrid_gate(),
            CpuCapabilities::portable(),
            ProofSummary {
                classical_signatures: 2,
                pq_signatures: 0,
            },
        )
        .expect_err("hybrid runtime requires PQ signer evidence");

        assert_eq!(err, AdmissionFailure::MissingPostQuantumSignature);
    }

    #[test]
    fn pq_primary_rejects_mixed_fallback_proofs() {
        let err = evaluate_admission(
            CryptoPolicy::pq_primary(),
            CpuCapabilities::portable(),
            ProofSummary {
                classical_signatures: 1,
                pq_signatures: 1,
            },
        )
        .expect_err("pq-primary runtime must reject classical signatures");

        assert_eq!(err, AdmissionFailure::ClassicalNotPermitted);
    }
}
