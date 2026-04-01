use crate::auth::envelope::AuthEnvelope;
use crate::auth::scheme::AuthScheme;
use crate::errors::AoxcvmError;
use crate::result::Result;

#[derive(Debug, Clone, Copy)]
pub struct AuthVerifierPolicy {
    pub require_pq_scheme: bool,
    pub max_nonce_gap: u64,
}

impl Default for AuthVerifierPolicy {
    fn default() -> Self {
        Self { require_pq_scheme: false, max_nonce_gap: 1_000_000 }
    }
}

pub fn verify_envelope(
    envelope: &AuthEnvelope,
    expected_nonce: u64,
    current_epoch: u64,
    policy: AuthVerifierPolicy,
) -> Result<()> {
    if envelope.expiry_epoch < current_epoch {
        return Err(AoxcvmError::AuthorizationFailed("envelope expired"));
    }

    if envelope.nonce < expected_nonce || envelope.nonce.saturating_sub(expected_nonce) > policy.max_nonce_gap {
        return Err(AoxcvmError::AuthorizationFailed("nonce outside allowed range"));
    }

    if policy.require_pq_scheme && !envelope.scheme.is_post_quantum_ready() {
        return Err(AoxcvmError::AuthorizationFailed("non-PQ scheme rejected by policy"));
    }

    if matches!(envelope.scheme, AuthScheme::Threshold) && envelope.capability_scope.is_empty() {
        return Err(AoxcvmError::AuthorizationFailed("threshold auth requires explicit capability scope"));
    }

    Ok(())
}
