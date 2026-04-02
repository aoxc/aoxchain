//! Deterministic fingerprint helpers built on quantum-impact-hardened hashes.

use crate::crypto::hash::quantum_unaffected_digest;

/// Produces a canonical execution fingerprint encoded as uppercase hex.
pub fn execution_fingerprint(namespace: &'static [u8], payload: &[u8]) -> String {
    let digest = quantum_unaffected_digest(namespace, payload);
    encode_hex_upper(&digest.to_bytes())
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
    use super::execution_fingerprint;

    #[test]
    fn fingerprint_is_stable_and_uppercase_hex() {
        let fp = execution_fingerprint(b"receipt", b"payload");
        assert_eq!(fp.len(), 192);
        assert!(fp.chars().all(|c| c.is_ascii_hexdigit()));
        assert_eq!(fp, fp.to_ascii_uppercase());
    }

    #[test]
    fn fingerprint_is_domain_separated() {
        let a = execution_fingerprint(b"receipt", b"payload");
        let b = execution_fingerprint(b"state", b"payload");
        assert_ne!(a, b);
    }
}
