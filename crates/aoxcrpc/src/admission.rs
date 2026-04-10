// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use aoxchal::cpu_opt::CpuCapabilities;
use aoxchal::crypto_profile::{
    CryptoPolicy, ProfileStage, ProofSummary, evaluate_admission as evaluate_crypto_admission,
};
use aoxcvm::auth::scheme::{AuthProfile, SignatureAlgorithm};

use crate::error::{AdmissionFailure, CryptoAdmissionFailure, MethodAdmissionFailure, RpcError};

/// Coarse identity tiers used by RPC admission and budgeting.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum IdentityTier {
    Anonymous,
    ApiKey,
    SignedClient,
    Operator,
}

/// Deterministic cost class attached to an RPC method.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MethodCostClass {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum QuantumTransitionStage {
    ClassicalAllowed,
    #[default]
    HybridRequired,
    PostQuantumOnly,
}

impl MethodCostClass {
    #[must_use]
    pub const fn budget_units(self) -> u32 {
        match self {
            Self::Low => 1,
            Self::Medium => 5,
            Self::High => 20,
            Self::Critical => 50,
        }
    }
}

/// Policy requirement for a canonical RPC method.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MethodAdmissionPolicy {
    pub canonical_name: &'static str,
    pub min_identity_tier: IdentityTier,
    pub cost_class: MethodCostClass,
    pub required_auth_profile: AuthProfile,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MethodAlias {
    pub exposed_name: &'static str,
    pub canonical_name: &'static str,
}

pub const METHOD_POLICIES: &[MethodAdmissionPolicy] = &[
    MethodAdmissionPolicy {
        canonical_name: "health",
        min_identity_tier: IdentityTier::Anonymous,
        cost_class: MethodCostClass::Low,
        required_auth_profile: AuthProfile::Legacy,
    },
    MethodAdmissionPolicy {
        canonical_name: "query_state",
        min_identity_tier: IdentityTier::ApiKey,
        cost_class: MethodCostClass::Medium,
        required_auth_profile: AuthProfile::HybridMandatory,
    },
    MethodAdmissionPolicy {
        canonical_name: "simulate_tx",
        min_identity_tier: IdentityTier::SignedClient,
        cost_class: MethodCostClass::High,
        required_auth_profile: AuthProfile::HybridMandatory,
    },
    MethodAdmissionPolicy {
        canonical_name: "submit_tx",
        min_identity_tier: IdentityTier::SignedClient,
        cost_class: MethodCostClass::Critical,
        required_auth_profile: AuthProfile::HybridMandatory,
    },
    MethodAdmissionPolicy {
        canonical_name: "operator_rotate_keys",
        min_identity_tier: IdentityTier::Operator,
        cost_class: MethodCostClass::Critical,
        required_auth_profile: AuthProfile::PostQuantumStrict,
    },
];

pub const METHOD_ALIASES: &[MethodAlias] = &[
    MethodAlias {
        exposed_name: "health",
        canonical_name: "health",
    },
    MethodAlias {
        exposed_name: "status",
        canonical_name: "health",
    },
    MethodAlias {
        exposed_name: "query_state",
        canonical_name: "query_state",
    },
    MethodAlias {
        exposed_name: "get_block",
        canonical_name: "query_state",
    },
    MethodAlias {
        exposed_name: "simulate_tx",
        canonical_name: "simulate_tx",
    },
    MethodAlias {
        exposed_name: "trace_tx",
        canonical_name: "simulate_tx",
    },
    MethodAlias {
        exposed_name: "submit_tx",
        canonical_name: "submit_tx",
    },
    MethodAlias {
        exposed_name: "operator_rotate_keys",
        canonical_name: "operator_rotate_keys",
    },
    MethodAlias {
        exposed_name: "operator_set_profile",
        canonical_name: "operator_rotate_keys",
    },
];

#[must_use]
pub fn policy_for_method(method: &str) -> Option<&'static MethodAdmissionPolicy> {
    let canonical_name = METHOD_ALIASES
        .iter()
        .find(|alias| alias.exposed_name == method)
        .map(|alias| alias.canonical_name)?;

    METHOD_POLICIES
        .iter()
        .find(|policy| policy.canonical_name == canonical_name)
}

/// Runtime context supplied to admission evaluation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdmissionContext {
    pub identity_tier: IdentityTier,
    pub signer_algorithms: Vec<SignatureAlgorithm>,
    pub verified_signature_count: u8,
    pub remaining_budget_units: u32,
    pub is_operator_authenticated: bool,
    pub cpu_capabilities: CpuCapabilities,
}

pub fn evaluate_submit_tx_admission(
    context: &AdmissionContext,
    stage: QuantumTransitionStage,
) -> Result<(), RpcError> {
    evaluate_method_admission_with_stage("submit_tx", context, stage)
}

pub fn evaluate_method_admission(method: &str, context: &AdmissionContext) -> Result<(), RpcError> {
    evaluate_method_admission_with_stage(method, context, QuantumTransitionStage::HybridRequired)
}

pub fn evaluate_method_admission_with_stage(
    method: &str,
    context: &AdmissionContext,
    stage: QuantumTransitionStage,
) -> Result<(), RpcError> {
    let policy = policy_for_method(method).ok_or(RpcError::AdmissionDenied {
        code: AdmissionFailure::Method(MethodAdmissionFailure::UnsupportedMethod),
        message: "unsupported RPC method",
    })?;

    let enforce_auth_profile = policy.canonical_name != "submit_tx";
    evaluate_admission_policy(policy, context, enforce_auth_profile)?;

    let crypto_policy = crypto_policy_from_stage(stage);
    let proof_summary = proof_summary_from_context(context)?;

    evaluate_crypto_admission(crypto_policy, context.cpu_capabilities, proof_summary).map_err(
        |failure| RpcError::AdmissionDenied {
            code: AdmissionFailure::Crypto(CryptoAdmissionFailure::from(failure)),
            message: "request proof does not satisfy active cryptographic policy",
        },
    )?;

    Ok(())
}

fn crypto_policy_from_stage(stage: QuantumTransitionStage) -> CryptoPolicy {
    match stage {
        QuantumTransitionStage::ClassicalAllowed => CryptoPolicy {
            stage: ProfileStage::ClassicalOnly,
            min_signatures: 1,
            allow_hybrid_fallback: true,
        },
        QuantumTransitionStage::HybridRequired => CryptoPolicy::hybrid_gate(),
        QuantumTransitionStage::PostQuantumOnly => CryptoPolicy::pq_primary(),
    }
}

fn proof_summary_from_context(context: &AdmissionContext) -> Result<ProofSummary, RpcError> {
    if usize::from(context.verified_signature_count) != context.signer_algorithms.len() {
        return Err(RpcError::AdmissionDenied {
            code: AdmissionFailure::Method(MethodAdmissionFailure::InvalidSignerSet),
            message: "verified signature count does not match signer algorithm declarations",
        });
    }

    let mut summary = ProofSummary::default();
    for algorithm in &context.signer_algorithms {
        if algorithm.is_post_quantum() {
            summary.pq_signatures = summary.pq_signatures.saturating_add(1);
        } else {
            summary.classical_signatures = summary.classical_signatures.saturating_add(1);
        }
    }

    Ok(summary)
}

fn evaluate_admission_policy(
    policy: &MethodAdmissionPolicy,
    context: &AdmissionContext,
    enforce_auth_profile: bool,
) -> Result<(), RpcError> {
    if context.identity_tier < policy.min_identity_tier {
        return Err(RpcError::AdmissionDenied {
            code: AdmissionFailure::Method(MethodAdmissionFailure::IdentityTierTooLow),
            message: "identity tier does not satisfy method policy",
        });
    }

    if policy.min_identity_tier == IdentityTier::Operator && !context.is_operator_authenticated {
        return Err(RpcError::AdmissionDenied {
            code: AdmissionFailure::Method(MethodAdmissionFailure::IdentityTierTooLow),
            message: "operator method requires operator-authenticated caller",
        });
    }

    if enforce_auth_profile
        && !policy
            .required_auth_profile
            .signer_set_is_valid(&context.signer_algorithms)
    {
        return Err(RpcError::AdmissionDenied {
            code: AdmissionFailure::Method(MethodAdmissionFailure::InvalidSignerSet),
            message: "signer set does not satisfy VM auth profile",
        });
    }

    if context.remaining_budget_units < policy.cost_class.budget_units() {
        return Err(RpcError::AdmissionDenied {
            code: AdmissionFailure::Method(MethodAdmissionFailure::BudgetExhausted),
            message: "request budget exhausted for method cost class",
        });
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        AdmissionContext, IdentityTier, MethodAdmissionFailure, QuantumTransitionStage,
        evaluate_method_admission, evaluate_method_admission_with_stage,
        evaluate_submit_tx_admission, policy_for_method,
    };
    use crate::error::{AdmissionFailure, CryptoAdmissionFailure, RpcError};
    use aoxchal::cpu_opt::CpuCapabilities;
    use aoxcvm::auth::scheme::SignatureAlgorithm;

    #[test]
    fn method_alias_resolves_to_policy() {
        let health = policy_for_method("health").expect("health policy must exist");
        let status = policy_for_method("status").expect("status alias must exist");
        assert_eq!(health, status);
        assert_eq!(health.canonical_name, "health");
    }

    #[test]
    fn submit_tx_accepts_hybrid_signer_set() {
        let context = AdmissionContext {
            identity_tier: IdentityTier::SignedClient,
            signer_algorithms: vec![SignatureAlgorithm::Ed25519, SignatureAlgorithm::MlDsa65],
            verified_signature_count: 2,
            remaining_budget_units: 80,
            is_operator_authenticated: false,
            cpu_capabilities: CpuCapabilities::portable(),
        };

        assert!(evaluate_method_admission("submit_tx", &context).is_ok());
    }

    #[test]
    fn submit_tx_rejects_classic_only_signers() {
        let context = AdmissionContext {
            identity_tier: IdentityTier::SignedClient,
            signer_algorithms: vec![SignatureAlgorithm::Ed25519],
            verified_signature_count: 1,
            remaining_budget_units: 80,
            is_operator_authenticated: false,
            cpu_capabilities: CpuCapabilities::portable(),
        };

        let error = evaluate_method_admission("submit_tx", &context)
            .expect_err("classic-only signer set should be denied");

        assert!(matches!(
            error,
            RpcError::AdmissionDenied {
                code: AdmissionFailure::Crypto(CryptoAdmissionFailure::InsufficientSignatureCount),
                ..
            }
        ));
    }

    #[test]
    fn operator_method_requires_operator_tier() {
        let context = AdmissionContext {
            identity_tier: IdentityTier::SignedClient,
            signer_algorithms: vec![SignatureAlgorithm::MlDsa87],
            verified_signature_count: 1,
            remaining_budget_units: 80,
            is_operator_authenticated: false,
            cpu_capabilities: CpuCapabilities::portable(),
        };

        let error = evaluate_method_admission("operator_set_profile", &context)
            .expect_err("non-operator tier should be denied");

        assert!(matches!(
            error,
            RpcError::AdmissionDenied {
                code: AdmissionFailure::Method(MethodAdmissionFailure::IdentityTierTooLow),
                ..
            }
        ));
    }

    #[test]
    fn expensive_method_rejects_insufficient_budget() {
        let context = AdmissionContext {
            identity_tier: IdentityTier::SignedClient,
            signer_algorithms: vec![SignatureAlgorithm::Ed25519, SignatureAlgorithm::MlDsa65],
            verified_signature_count: 2,
            remaining_budget_units: 10,
            is_operator_authenticated: false,
            cpu_capabilities: CpuCapabilities::portable(),
        };

        let error = evaluate_method_admission("submit_tx", &context)
            .expect_err("insufficient budget should be denied");

        assert!(matches!(
            error,
            RpcError::AdmissionDenied {
                code: AdmissionFailure::Method(MethodAdmissionFailure::BudgetExhausted),
                ..
            }
        ));
    }

    #[test]
    fn submit_tx_hybrid_stage_rejects_classical_only_signers() {
        let context = AdmissionContext {
            identity_tier: IdentityTier::SignedClient,
            signer_algorithms: vec![SignatureAlgorithm::Ed25519],
            verified_signature_count: 1,
            remaining_budget_units: 80,
            is_operator_authenticated: false,
            cpu_capabilities: CpuCapabilities::portable(),
        };

        let error = evaluate_submit_tx_admission(&context, QuantumTransitionStage::HybridRequired)
            .expect_err("hybrid stage must reject classical-only proofs");

        assert!(matches!(
            error,
            RpcError::AdmissionDenied {
                code: AdmissionFailure::Crypto(CryptoAdmissionFailure::InsufficientSignatureCount),
                ..
            }
        ));
    }

    #[test]
    fn submit_tx_post_quantum_stage_accepts_pq_signers() {
        let context = AdmissionContext {
            identity_tier: IdentityTier::SignedClient,
            signer_algorithms: vec![SignatureAlgorithm::MlDsa65, SignatureAlgorithm::MlDsa87],
            verified_signature_count: 2,
            remaining_budget_units: 80,
            is_operator_authenticated: false,
            cpu_capabilities: CpuCapabilities::portable(),
        };

        assert!(
            evaluate_submit_tx_admission(&context, QuantumTransitionStage::PostQuantumOnly).is_ok()
        );
    }

    #[test]
    fn verified_signature_count_mismatch_is_rejected() {
        let context = AdmissionContext {
            identity_tier: IdentityTier::SignedClient,
            signer_algorithms: vec![SignatureAlgorithm::Ed25519, SignatureAlgorithm::MlDsa65],
            verified_signature_count: 1,
            remaining_budget_units: 80,
            is_operator_authenticated: false,
            cpu_capabilities: CpuCapabilities::portable(),
        };

        let error = evaluate_method_admission_with_stage(
            "submit_tx",
            &context,
            QuantumTransitionStage::ClassicalAllowed,
        )
        .expect_err("mismatch should be denied");

        assert!(matches!(
            error,
            RpcError::AdmissionDenied {
                code: AdmissionFailure::Method(MethodAdmissionFailure::InvalidSignerSet),
                ..
            }
        ));
    }
}
