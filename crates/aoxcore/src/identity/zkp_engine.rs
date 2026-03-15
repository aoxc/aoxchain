use sha3::{Digest, Sha3_256};

/// Minimal deterministic ZKP envelope used by higher layers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ZkpProof {
    pub circuit_id: String,
    pub proof_bytes: Vec<u8>,
    pub public_inputs_hash: String,
}

/// Produces a deterministic pseudo-proof artifact for testing and integration wiring.
#[must_use]
pub fn generate_proof(circuit_id: &str, witness: &[u8], public_inputs: &[u8]) -> ZkpProof {
    let mut proof_hasher = Sha3_256::new();
    proof_hasher.update(circuit_id.as_bytes());
    proof_hasher.update(witness);
    let proof_bytes = proof_hasher.finalize().to_vec();

    let mut inputs_hasher = Sha3_256::new();
    inputs_hasher.update(public_inputs);
    let public_inputs_hash = hex::encode(inputs_hasher.finalize());

    ZkpProof {
        circuit_id: circuit_id.to_string(),
        proof_bytes,
        public_inputs_hash,
    }
}

pub fn verify_proof(proof: &ZkpProof, expected_public_inputs: &[u8]) -> Result<(), String> {
    if proof.circuit_id.trim().is_empty() {
        return Err("ZKP_EMPTY_CIRCUIT_ID".to_string());
    }

    if proof.proof_bytes.len() < 32 {
        return Err("ZKP_PROOF_TOO_SHORT".to_string());
    }

    let mut hasher = Sha3_256::new();
    hasher.update(expected_public_inputs);
    let expected_hash = hex::encode(hasher.finalize());

    if expected_hash != proof.public_inputs_hash {
        return Err("ZKP_PUBLIC_INPUT_MISMATCH".to_string());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{generate_proof, verify_proof};

    #[test]
    fn proof_roundtrip() {
        let proof = generate_proof("transfer_v1", b"witness", b"public-inputs");
        assert!(verify_proof(&proof, b"public-inputs").is_ok());
    }
}
