// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use aoxcvm::auth::scheme::{AuthProfile, SignatureAlgorithm};

use crate::error::RpcError;

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

/// Policy requirement for a single RPC method.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MethodAdmissionPolicy {
    pub method: &'static str,
    pub min_identity_tier: IdentityTier,
    pub cost_class: MethodCostClass,
    pub required_auth_profile: AuthProfile,
}

impl MethodAdmissionPolicy {
    /// Returns a production-oriented default policy for a method name.
    #[must_use]
    pub fn for_method(method: &str) -> Option<Self> {
        match method {
            "health" | "status" => Some(Self {
                method: "health",
                min_identity_tier: IdentityTier::Anonymous,
                cost_class: MethodCostClass::Low,
                required_auth_profile: AuthProfile::Legacy,
            }),
            "query_state" | "get_block" => Some(Self {
                method: "query_state",
                min_identity_tier: IdentityTier::ApiKey,
                cost_class: MethodCostClass::Medium,
                required_auth_profile: AuthProfile::HybridMandatory,
            }),
            "simulate_tx" | "trace_tx" => Some(Self {
                method: "simulate_tx",
                min_identity_tier: IdentityTier::SignedClient,
                cost_class: MethodCostClass::High,
                required_auth_profile: AuthProfile::HybridMandatory,
            }),
            "submit_tx" => Some(Self {
                method: "submit_tx",
                min_identity_tier: IdentityTier::SignedClient,
                cost_class: MethodCostClass::Critical,
                required_auth_profile: AuthProfile::HybridMandatory,
            }),
            "operator_rotate_keys" | "operator_set_profile" => Some(Self {
                method: "operator_rotate_keys",
                min_identity_tier: IdentityTier::Operator,
                cost_class: MethodCostClass::Critical,
                required_auth_profile: AuthProfile::PostQuantumStrict,
            }),
            _ => None,
        }
    }
}

/// Runtime context supplied to admission evaluation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdmissionContext {
    pub identity_tier: IdentityTier,
    pub signer_algorithms: Vec<SignatureAlgorithm>,
    pub remaining_budget_units: u32,
}

pub fn evaluate_submit_tx_admission(
    context: &AdmissionContext,
    stage: QuantumTransitionStage,
) -> Result<(), RpcError> {
    let auth_profile = match stage {
        QuantumTransitionStage::ClassicalAllowed => AuthProfile::Legacy,
        QuantumTransitionStage::HybridRequired => AuthProfile::HybridMandatory,
        QuantumTransitionStage::PostQuantumOnly => AuthProfile::PostQuantumStrict,
    };
    let policy = MethodAdmissionPolicy {
        method: "submit_tx",
        min_identity_tier: IdentityTier::SignedClient,
        cost_class: MethodCostClass::Critical,
        required_auth_profile: auth_profile,
    };
    evaluate_admission_policy(&policy, context)
}

/// Evaluates method access before expensive RPC execution paths.
pub fn evaluate_method_admission(method: &str, context: &AdmissionContext) -> Result<(), RpcError> {
    let policy = MethodAdmissionPolicy::for_method(method)
        .ok_or_else(|| RpcError::AdmissionDenied("unsupported RPC method".to_string()))?;
    evaluate_admission_policy(&policy, context)
}

fn evaluate_admission_policy(
    policy: &MethodAdmissionPolicy,
    context: &AdmissionContext,
) -> Result<(), RpcError> {
    if context.identity_tier < policy.min_identity_tier {
        return Err(RpcError::AdmissionDenied(
            "identity tier does not satisfy method policy".to_string(),
        ));
    }

    if !policy
        .required_auth_profile
        .signer_set_is_valid(&context.signer_algorithms)
    {
        return Err(RpcError::AdmissionDenied(
            "signer set does not satisfy VM auth profile".to_string(),
        ));
    }

    if context.remaining_budget_units < policy.cost_class.budget_units() {
        return Err(RpcError::AdmissionDenied(
            "request budget exhausted for method cost class".to_string(),
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{AdmissionContext, IdentityTier, evaluate_method_admission};
    use aoxcvm::auth::scheme::SignatureAlgorithm;

    #[test]
    fn submit_tx_accepts_hybrid_signer_set() {
        let context = AdmissionContext {
            identity_tier: IdentityTier::SignedClient,
            signer_algorithms: vec![SignatureAlgorithm::Ed25519, SignatureAlgorithm::MlDsa65],
            remaining_budget_units: 80,
        };

        assert!(evaluate_method_admission("submit_tx", &context).is_ok());
    }

    #[test]
    fn submit_tx_rejects_classic_only_signers() {
        let context = AdmissionContext {
            identity_tier: IdentityTier::SignedClient,
            signer_algorithms: vec![SignatureAlgorithm::Ed25519],
            remaining_budget_units: 80,
        };

        let error = evaluate_method_admission("submit_tx", &context)
            .expect_err("classic-only signer set should be denied");

        assert!(error.to_string().contains("signer set"));
    }

    #[test]
    fn operator_method_requires_operator_tier() {
        let context = AdmissionContext {
            identity_tier: IdentityTier::SignedClient,
            signer_algorithms: vec![SignatureAlgorithm::MlDsa87],
            remaining_budget_units: 80,
        };

        let error = evaluate_method_admission("operator_set_profile", &context)
            .expect_err("non-operator tier should be denied");

        assert!(error.to_string().contains("identity tier"));
    }

    #[test]
    fn expensive_method_rejects_insufficient_budget() {
        let context = AdmissionContext {
            identity_tier: IdentityTier::SignedClient,
            signer_algorithms: vec![SignatureAlgorithm::Ed25519, SignatureAlgorithm::MlDsa65],
            remaining_budget_units: 10,
        };

        let error = evaluate_method_admission("submit_tx", &context)
            .expect_err("insufficient budget should be denied");

        assert!(error.to_string().contains("budget"));
    }
}
