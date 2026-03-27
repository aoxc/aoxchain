// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

/// Defines accepted signature combinations for a HyperVM deployment.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignaturePolicy {
    ClassicalOnly,
    Hybrid,
    PostQuantumOnly,
}

/// Hybrid signature container for migration-friendly verification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HybridSignature {
    pub classical: Vec<u8>,
    pub post_quantum: Option<Vec<u8>>,
}

impl HybridSignature {
    pub fn is_valid_for_policy(&self, policy: SignaturePolicy) -> bool {
        match policy {
            SignaturePolicy::ClassicalOnly => !self.classical.is_empty(),
            SignaturePolicy::Hybrid => !self.classical.is_empty() && self.post_quantum.is_some(),
            SignaturePolicy::PostQuantumOnly => self
                .post_quantum
                .as_ref()
                .map(|bytes| !bytes.is_empty())
                .unwrap_or(false),
        }
    }
}

/// Interface for pluggable signer/validator backends.
pub trait HybridSigner: Send + Sync {
    fn sign_hybrid(&self, message: &[u8]) -> HybridSignature;

    fn verify_hybrid(
        &self,
        message: &[u8],
        signature: &HybridSignature,
        policy: SignaturePolicy,
    ) -> bool;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn policy_validation_works() {
        let sig = HybridSignature {
            classical: vec![1, 2, 3],
            post_quantum: Some(vec![9, 8, 7]),
        };

        assert!(sig.is_valid_for_policy(SignaturePolicy::ClassicalOnly));
        assert!(sig.is_valid_for_policy(SignaturePolicy::Hybrid));
        assert!(sig.is_valid_for_policy(SignaturePolicy::PostQuantumOnly));
    }
}
