// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use crate::error::RpcError;

/// mTLS authentication policy for zero-trust ingress.
#[derive(Debug, Clone)]
pub struct MtlsPolicy {
    pub trusted_fingerprints: Vec<String>,
}

impl MtlsPolicy {
    #[must_use]
    pub fn new(trusted_fingerprints: Vec<String>) -> Self {
        Self {
            trusted_fingerprints,
        }
    }

    pub fn validate_client_fingerprint(&self, presented_fingerprint: &str) -> Result<(), RpcError> {
        if self
            .trusted_fingerprints
            .iter()
            .any(|fingerprint| fingerprint == presented_fingerprint)
        {
            return Ok(());
        }

        Err(RpcError::MtlsAuthFailed)
    }
}
