use crate::error::RpcError;

#[derive(Debug, Clone)]
pub struct ZkpValidator {
    pub min_proof_len: usize,
}

impl Default for ZkpValidator {
    fn default() -> Self {
        Self { min_proof_len: 64 }
    }
}

impl ZkpValidator {
    pub fn validate(&self, proof: &[u8]) -> Result<(), RpcError> {
        if proof.len() < self.min_proof_len {
            return Err(RpcError::ZkpValidationFailed("PROOF_TOO_SHORT".to_string()));
        }

        if proof.iter().all(|byte| *byte == 0) {
            return Err(RpcError::ZkpValidationFailed("PROOF_ALL_ZERO".to_string()));
        }

        Ok(())
    }
}
