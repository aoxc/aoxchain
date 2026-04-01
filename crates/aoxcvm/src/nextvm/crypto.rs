use crate::nextvm::error::NextVmError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CryptoProfile {
    ClassicalOnly,
    HybridPqPreferred,
    HybridPqRequired,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignatureEnvelope {
    pub classical_sig: Vec<u8>,
    pub pq_sig: Option<Vec<u8>>,
}

impl CryptoProfile {
    pub fn validate_envelope(&self, envelope: &SignatureEnvelope) -> Result<(), NextVmError> {
        let classical_ok = !envelope.classical_sig.is_empty();
        let pq_ok = envelope
            .pq_sig
            .as_ref()
            .map(|sig| !sig.is_empty())
            .unwrap_or(false);

        match self {
            CryptoProfile::ClassicalOnly if classical_ok => Ok(()),
            CryptoProfile::HybridPqPreferred if classical_ok => Ok(()),
            CryptoProfile::HybridPqRequired if classical_ok && pq_ok => Ok(()),
            _ => Err(NextVmError::InvalidSignatureEnvelope),
        }
    }
}
