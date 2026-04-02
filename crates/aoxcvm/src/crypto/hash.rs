//! Hash primitives for deterministic and quantum-hardened commitments.

use sha3::{Digest, Sha3_512};

/// Canonical output size for the SHA3-512 leg.
pub const SHA3_512_BYTES: usize = 64;
/// Canonical output size for the BLAKE3 leg.
pub const BLAKE3_BYTES: usize = 32;

/// Dual-hash digest intended for long-lived commitments where Grover-impact
/// margin and algorithm agility are both required.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuantumHardenedDigest {
    /// SHA3-512 digest (primary long-term preimage margin).
    pub sha3_512: [u8; SHA3_512_BYTES],
    /// BLAKE3 digest (independent implementation/assumption hedge).
    pub blake3_256: [u8; BLAKE3_BYTES],
}

impl QuantumHardenedDigest {
    /// Returns canonical byte layout: SHA3-512 || BLAKE3-256.
    pub fn to_bytes(&self) -> [u8; SHA3_512_BYTES + BLAKE3_BYTES] {
        let mut out = [0_u8; SHA3_512_BYTES + BLAKE3_BYTES];
        out[..SHA3_512_BYTES].copy_from_slice(&self.sha3_512);
        out[SHA3_512_BYTES..].copy_from_slice(&self.blake3_256);
        out
    }
}

/// Computes a domain-separated dual digest for high-assurance commitment
/// surfaces.
pub fn quantum_hardened_digest(domain: &'static [u8], payload: &[u8]) -> QuantumHardenedDigest {
    let domain_len = (domain.len() as u32).to_be_bytes();

    let mut sha3 = Sha3_512::new();
    sha3.update(b"AOXCVM/QHASH/V1");
    sha3.update(domain_len);
    sha3.update(domain);
    sha3.update(payload);
    let sha3_out = sha3.finalize();

    let mut blake3_hasher = blake3::Hasher::new();
    blake3_hasher.update(b"AOXCVM/QHASH/V1");
    blake3_hasher.update(&domain_len);
    blake3_hasher.update(domain);
    blake3_hasher.update(payload);
    let blake3_out = blake3_hasher.finalize();

    let mut sha3_512 = [0_u8; SHA3_512_BYTES];
    sha3_512.copy_from_slice(&sha3_out);

    let mut blake3_256 = [0_u8; BLAKE3_BYTES];
    blake3_256.copy_from_slice(blake3_out.as_bytes());

    QuantumHardenedDigest {
        sha3_512,
        blake3_256,
    }
}

/// Backward-compatible wrapper kept for migration from pre-audit naming.
#[deprecated(
    since = "0.1.1",
    note = "use quantum_hardened_digest; avoid absolute security wording in public APIs"
)]
pub fn quantum_unaffected_digest(domain: &'static [u8], payload: &[u8]) -> QuantumHardenedDigest {
    quantum_hardened_digest(domain, payload)
}

#[cfg(test)]
mod tests {
    use super::quantum_hardened_digest;

    #[test]
    fn digest_is_deterministic() {
        let a = quantum_hardened_digest(b"receipt", b"payload-1");
        let b = quantum_hardened_digest(b"receipt", b"payload-1");
        assert_eq!(a, b);
        assert_eq!(a.to_bytes().len(), 96);
    }

    #[test]
    fn digest_is_domain_separated() {
        let receipt = quantum_hardened_digest(b"receipt", b"payload-1");
        let state = quantum_hardened_digest(b"state", b"payload-1");
        assert_ne!(receipt, state);
    }

    #[allow(deprecated)]
    #[test]
    fn legacy_helper_matches_canonical_output() {
        let legacy = quantum_unaffected_digest(b"receipt", b"payload-1");
        let canonical = quantum_hardened_digest(b"receipt", b"payload-1");
        assert_eq!(legacy, canonical);
    }
}
