#[cfg(test)]
mod tests {
    use super::*;

    fn sample_unsigned() -> Certificate {
        Certificate::new_unsigned(
            "AOXC-0001-MAIN".to_string(),
            "AOXC-VAL-EU-3F7A9C21D4E8B7AA-K9".to_string(),
            "VAL".to_string(),
            "EU".to_string(),
            "A1B2C3D4".to_string(),
            1_700_000_000,
            1_800_000_000,
        )
    }

    #[test]
    fn unsigned_certificate_validates_successfully() {
        let cert = sample_unsigned();
        assert_eq!(cert.validate_unsigned(), Ok(()));
    }

    #[test]
    fn signed_certificate_requires_issuer_and_signature() {
        let cert = sample_unsigned();
        assert_eq!(cert.validate_signed(), Err(CertificateError::EmptyIssuer));
    }

    #[test]
    fn signed_certificate_validates_when_completed() {
        let mut cert = sample_unsigned();
        cert.issuer = "AOXC-ROOT-CA".to_string();
        cert.signature = "A1B2".to_string();

        assert_eq!(cert.validate_signed(), Ok(()));
    }

    #[test]
    fn invalid_validity_window_is_rejected() {
        let cert = Certificate::new_unsigned(
            "AOXC-0001-MAIN".to_string(),
            "AOXC-VAL-EU-3F7A9C21D4E8B7AA-K9".to_string(),
            "VAL".to_string(),
            "EU".to_string(),
            "A1B2C3D4".to_string(),
            100,
            100,
        );

        assert_eq!(
            cert.validate_unsigned(),
            Err(CertificateError::InvalidValidityWindow)
        );
    }

    #[test]
    fn invalid_public_key_hex_is_rejected() {
        let cert = Certificate::new_unsigned(
            "AOXC-0001-MAIN".to_string(),
            "AOXC-VAL-EU-3F7A9C21D4E8B7AA-K9".to_string(),
            "VAL".to_string(),
            "EU".to_string(),
            "ZZ_NOT_HEX".to_string(),
            100,
            200,
        );

        assert_eq!(
            cert.validate_unsigned(),
            Err(CertificateError::InvalidPublicKeyHex)
        );
    }

    #[test]
    fn odd_length_public_key_hex_is_rejected() {
        let cert = Certificate::new_unsigned(
            "AOXC-0001-MAIN".to_string(),
            "AOXC-VAL-EU-3F7A9C21D4E8B7AA-K9".to_string(),
            "VAL".to_string(),
            "EU".to_string(),
            "ABC".to_string(),
            100,
            200,
        );

        assert_eq!(
            cert.validate_unsigned(),
            Err(CertificateError::InvalidPublicKeyHex)
        );
    }

    #[test]
    fn odd_length_signature_hex_is_rejected() {
        let mut cert = sample_unsigned();
        cert.issuer = "AOXC-ROOT-CA".to_string();
        cert.signature = "ABC".to_string();

        assert_eq!(
            cert.validate_signed(),
            Err(CertificateError::InvalidSignatureHex)
        );
    }

    #[test]
    fn surrounding_whitespace_is_rejected() {
        let cert = Certificate::new_unsigned(
            " AOXC-0001-MAIN".to_string(),
            "AOXC-VAL-EU-3F7A9C21D4E8B7AA-K9".to_string(),
            "VAL".to_string(),
            "EU".to_string(),
            "A1B2C3D4".to_string(),
            1_700_000_000,
            1_800_000_000,
        );

        assert_eq!(
            cert.validate_unsigned(),
            Err(CertificateError::InvalidChain)
        );
    }

    #[test]
    fn signing_payload_excludes_signature() {
        let mut cert = sample_unsigned();
        cert.issuer = "AOXC-ROOT-CA".to_string();
        cert.signature = "DEADBEEF".to_string();

        let payload = cert.signing_payload();
        assert_eq!(payload.issuer, "AOXC-ROOT-CA");
        assert_eq!(payload.actor_id, cert.actor_id);
    }

    #[test]
    fn signing_payload_bytes_validate_before_serialization() {
        let mut cert = sample_unsigned();
        cert.issuer = "AOXC-ROOT-CA".to_string();

        let bytes = cert
            .signing_payload_bytes()
            .expect("signing payload serialization must succeed");

        assert!(!bytes.is_empty());
    }

    #[test]
    fn unsigned_view_clears_signature() {
        let mut cert = sample_unsigned();
        cert.signature = "BEEF".to_string();

        let unsigned = cert.unsigned_view();
        assert!(unsigned.signature.is_empty());
    }

    #[test]
    fn decoded_public_key_bytes_are_available() {
        let cert = sample_unsigned();

        let decoded = cert
            .public_key_bytes()
            .expect("public key bytes must decode successfully");

        assert_eq!(decoded, vec![0xA1, 0xB2, 0xC3, 0xD4]);
    }

    #[test]
    fn decoded_signature_bytes_are_available() {
        let mut cert = sample_unsigned();
        cert.issuer = "AOXC-ROOT-CA".to_string();
        cert.signature = "DEADBEEF".to_string();

        let decoded = cert
            .signature_bytes()
            .expect("signature bytes must decode successfully");

        assert_eq!(decoded, vec![0xDE, 0xAD, 0xBE, 0xEF]);
    }

    #[test]
    fn fingerprint_is_stable() {
        let mut cert = sample_unsigned();
        cert.issuer = "AOXC-ROOT-CA".to_string();
        cert.signature = "DEADBEEF".to_string();

        let a = cert.fingerprint().expect("fingerprint must succeed");
        let b = cert.fingerprint().expect("fingerprint must succeed");

        assert_eq!(a, b);
        assert_eq!(a.len(), 16);
    }

    #[test]
    fn validity_helpers_work() {
        let cert = sample_unsigned();

        assert!(cert.is_valid_at(1_750_000_000));
        assert!(cert.is_expired_at(1_800_000_000));
        assert!(cert.is_not_yet_valid_at(1_600_000_000));
        assert_eq!(
            cert.validity_state_at(1_750_000_000),
            CertificateValidityState::Valid
        );
        assert_eq!(
            cert.validity_state_at(1_600_000_000),
            CertificateValidityState::NotYetValid
        );
        assert_eq!(
            cert.validity_state_at(1_800_000_000),
            CertificateValidityState::Expired
        );
    }
}
