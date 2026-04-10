// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use aoxcvm::auth::scheme::SignatureAlgorithm;

use crate::admission::{
    AdmissionContext, IdentityTier, QuantumTransitionStage, evaluate_submit_tx_admission,
};
use crate::error::RpcError;
use crate::middleware::zkp_validator::ZkpValidator;
use crate::types::{TxSubmissionRequest, TxSubmissionResult};

#[derive(Debug, Clone, Default)]
pub struct TxSubmissionService {
    pub zkp_validator: ZkpValidator,
    pub quantum_transition_stage: QuantumTransitionStage,
}

impl TxSubmissionService {
    #[must_use]
    pub fn with_transition_stage(stage: QuantumTransitionStage) -> Self {
        Self {
            zkp_validator: ZkpValidator::default(),
            quantum_transition_stage: stage,
        }
    }

    pub fn submit(&self, request: TxSubmissionRequest) -> Result<TxSubmissionResult, RpcError> {
        if request.actor_id.trim().is_empty() || request.tx_payload.is_empty() {
            return Err(RpcError::InvalidRequest);
        }

        let context = AdmissionContext {
            identity_tier: parse_identity_tier(request.identity_tier.as_deref())?,
            signer_algorithms: parse_signer_algorithms(
                &request.signer_algorithms,
                self.quantum_transition_stage,
            )?,
            remaining_budget_units: request.remaining_budget_units.unwrap_or(0),
        };

        evaluate_submit_tx_admission(&context, self.quantum_transition_stage)?;
        self.zkp_validator.validate(&request.zkp_proof)?;

        Ok(TxSubmissionResult {
            tx_id: format!("tx-{}", hex_fragment(&request.tx_payload)),
            accepted: true,
        })
    }
}

fn parse_identity_tier(raw: Option<&str>) -> Result<IdentityTier, RpcError> {
    match raw
        .unwrap_or("anonymous")
        .trim()
        .to_ascii_lowercase()
        .as_str()
    {
        "anonymous" => Ok(IdentityTier::Anonymous),
        "api_key" | "apikey" => Ok(IdentityTier::ApiKey),
        "signed_client" | "signed" => Ok(IdentityTier::SignedClient),
        "operator" => Ok(IdentityTier::Operator),
        _ => Err(RpcError::AdmissionDenied(
            "unknown identity tier; expected anonymous/api_key/signed_client/operator".to_string(),
        )),
    }
}

fn parse_signer_algorithms(
    raw: &[String],
    stage: QuantumTransitionStage,
) -> Result<Vec<SignatureAlgorithm>, RpcError> {
    if raw.is_empty() {
        return Err(RpcError::AdmissionDenied(
            "signer_algorithms must include at least one signer".to_string(),
        ));
    }

    let parsed: Vec<SignatureAlgorithm> = raw
        .iter()
        .map(|alg| match alg.trim().to_ascii_lowercase().as_str() {
            "ed25519" => Ok(SignatureAlgorithm::Ed25519),
            "ecdsa-p256" | "ecdsa_p256" => Ok(SignatureAlgorithm::EcdsaP256),
            "ml-dsa-65" | "ml_dsa_65" => Ok(SignatureAlgorithm::MlDsa65),
            "ml-dsa-87" | "ml_dsa_87" => Ok(SignatureAlgorithm::MlDsa87),
            _ => Err(RpcError::AdmissionDenied(format!(
                "unsupported signer algorithm: {alg}"
            ))),
        })
        .collect::<Result<Vec<_>, _>>()?;

    let has_classical = parsed.iter().any(|alg| {
        matches!(
            alg,
            SignatureAlgorithm::Ed25519 | SignatureAlgorithm::EcdsaP256
        )
    });
    let has_post_quantum = parsed.iter().any(|alg| {
        matches!(
            alg,
            SignatureAlgorithm::MlDsa65 | SignatureAlgorithm::MlDsa87
        )
    });

    match stage {
        QuantumTransitionStage::ClassicalAllowed => Ok(parsed),
        QuantumTransitionStage::HybridRequired => {
            if has_classical && has_post_quantum {
                Ok(parsed)
            } else {
                Err(RpcError::AdmissionDenied(
                    "submit_tx requires hybrid signer set in hybrid_required stage".to_string(),
                ))
            }
        }
        QuantumTransitionStage::PostQuantumOnly => {
            if has_post_quantum && !has_classical {
                Ok(parsed)
            } else {
                Err(RpcError::AdmissionDenied(
                    "submit_tx requires PQ-only signer set in post_quantum_only stage".to_string(),
                ))
            }
        }
    }
}

fn hex_fragment(payload: &[u8]) -> String {
    payload
        .iter()
        .take(6)
        .map(|byte| format!("{:02x}", byte))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::TxSubmissionService;
    use crate::admission::QuantumTransitionStage;
    use crate::types::TxSubmissionRequest;

    fn valid_request() -> TxSubmissionRequest {
        TxSubmissionRequest {
            actor_id: "actor-1".to_string(),
            tx_payload: vec![1, 2, 3, 4],
            zkp_proof: vec![9; 64],
            identity_tier: Some("signed_client".to_string()),
            signer_algorithms: vec!["ed25519".to_string(), "ml-dsa-65".to_string()],
            remaining_budget_units: Some(100),
        }
    }

    #[test]
    fn submit_enforces_admission_before_zkp() {
        let service = TxSubmissionService::default();
        let mut req = valid_request();
        req.remaining_budget_units = Some(1);

        let err = service.submit(req).expect_err("must fail admission budget");
        assert!(err.to_string().contains("ADMISSION_DENIED"));
    }

    #[test]
    fn submit_accepts_hybrid_tx_context() {
        let service = TxSubmissionService::default();
        let result = service
            .submit(valid_request())
            .expect("valid request accepted");

        assert!(result.accepted);
        assert!(result.tx_id.starts_with("tx-"));
    }

    #[test]
    fn classical_allowed_stage_accepts_classical_only_signer_set() {
        let service =
            TxSubmissionService::with_transition_stage(QuantumTransitionStage::ClassicalAllowed);
        let mut req = valid_request();
        req.signer_algorithms = vec!["ed25519".to_string()];

        let result = service
            .submit(req)
            .expect("classical_allowed should permit classical-only");
        assert!(result.accepted);
    }

    #[test]
    fn post_quantum_only_stage_rejects_classical_signer_set() {
        let service =
            TxSubmissionService::with_transition_stage(QuantumTransitionStage::PostQuantumOnly);
        let mut req = valid_request();
        req.signer_algorithms = vec!["ed25519".to_string(), "ml-dsa-65".to_string()];

        let err = service
            .submit(req)
            .expect_err("post_quantum_only should reject classical signers");
        assert!(err.to_string().contains("PQ-only signer set"));
    }
}
