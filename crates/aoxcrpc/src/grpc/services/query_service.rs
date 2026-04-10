// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use crate::admission::{AdmissionContext, evaluate_method_admission};
use crate::error::RpcError;
use crate::types::ChainStatus;

#[derive(Debug, Clone)]
pub struct QueryService {
    pub chain_id: String,
}

impl Default for QueryService {
    fn default() -> Self {
        Self {
            chain_id: "AOX-MAIN".to_string(),
        }
    }
}

impl QueryService {
    #[must_use]
    pub fn get_chain_status(&self, height: u64, syncing: bool) -> ChainStatus {
        ChainStatus {
            chain_id: self.chain_id.clone(),
            height,
            syncing,
        }
    }

    pub fn get_chain_status_admitted(
        &self,
        height: u64,
        syncing: bool,
        context: &AdmissionContext,
    ) -> Result<ChainStatus, RpcError> {
        evaluate_method_admission("query_state", context)?;
        Ok(self.get_chain_status(height, syncing))
    }
}

#[cfg(test)]
mod tests {
    use super::QueryService;
    use crate::admission::{AdmissionContext, IdentityTier};
    use aoxchal::cpu_opt::CpuCapabilities;
    use aoxcvm::auth::scheme::SignatureAlgorithm;
    #[test]
    fn admitted_status_requires_api_key_or_higher() {
        let service = QueryService::default();
        let context = AdmissionContext {
            identity_tier: IdentityTier::Anonymous,
            signer_algorithms: vec![SignatureAlgorithm::Ed25519, SignatureAlgorithm::MlDsa65],
            verified_signature_count: 2,
            remaining_budget_units: 10,
            is_operator_authenticated: false,
            cpu_capabilities: CpuCapabilities::portable(),
        };

        let err = service
            .get_chain_status_admitted(100, false, &context)
            .expect_err("anonymous should be denied");
        assert!(matches!(
            err,
            RpcError::AdmissionDenied {
                code: AdmissionFailure::Method(MethodAdmissionFailure::IdentityTierTooLow),
                ..
            }
        ));
    }

    #[test]
    fn admitted_status_accepts_api_key_with_hybrid_signers() {
        let service = QueryService::default();
        let context = AdmissionContext {
            identity_tier: IdentityTier::ApiKey,
            signer_algorithms: vec![SignatureAlgorithm::Ed25519, SignatureAlgorithm::MlDsa65],
            verified_signature_count: 2,
            remaining_budget_units: 10,
            is_operator_authenticated: false,
            cpu_capabilities: CpuCapabilities::portable(),
        };

        let status = service
            .get_chain_status_admitted(42, false, &context)
            .expect("admission should pass");
        assert_eq!(status.height, 42);
    }
}
