//! Deterministic fingerprint helpers built on quantum-hardened hashes.

use crate::crypto::hash::quantum_hardened_digest;

/// Produces a canonical execution fingerprint encoded as uppercase hex.
pub fn canonical_execution_fingerprint(namespace: &'static [u8], payload: &[u8]) -> String {
    let digest = quantum_hardened_digest(namespace, payload);
    encode_hex_upper(&digest.to_bytes())
}

/// Backward-compatible wrapper kept for migration from pre-audit naming.
#[deprecated(
    since = "0.1.1",
    note = "use canonical_execution_fingerprint for explicit canonical semantics"
)]
pub fn execution_fingerprint(namespace: &'static [u8], payload: &[u8]) -> String {
    canonical_execution_fingerprint(namespace, payload)
}

fn encode_hex_upper(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789ABCDEF";
    let mut out = String::with_capacity(bytes.len() * 2);
    for &byte in bytes {
        out.push(HEX[(byte >> 4) as usize] as char);
        out.push(HEX[(byte & 0x0F) as usize] as char);
    }
    out
}

#[cfg(test)]
mod tests {
    #[allow(deprecated)]
    use super::{canonical_execution_fingerprint, execution_fingerprint};

    #[test]
    fn fingerprint_is_stable_and_uppercase_hex() {
        let fp = canonical_execution_fingerprint(b"receipt", b"payload");
        assert_eq!(fp.len(), 192);
        assert!(fp.chars().all(|c| c.is_ascii_hexdigit()));
        assert_eq!(fp, fp.to_ascii_uppercase());
    }

    #[test]
    fn fingerprint_is_domain_separated() {
        let a = canonical_execution_fingerprint(b"receipt", b"payload");
        let b = canonical_execution_fingerprint(b"state", b"payload");
        assert_ne!(a, b);
    }

    #[allow(deprecated)]
    #[test]
    fn legacy_helper_matches_canonical_output() {
        let legacy = execution_fingerprint(b"receipt", b"payload");
        let canonical = canonical_execution_fingerprint(b"receipt", b"payload");
        assert_eq!(legacy, canonical);
    }
}
