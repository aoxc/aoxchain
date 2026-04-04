#[cfg(test)]
mod tests {
    use super::*;
    use crate::identity::{
        key_engine::{KeyEngine, MASTER_SEED_LEN},
        keyfile::encrypt_key_to_envelope,
    };

    fn make_bundle(
        seed_byte: u8,
        node_name: &str,
        profile: &str,
        crypto_profile: CryptoProfile,
    ) -> NodeKeyBundleV1 {
        let engine = KeyEngine::from_seed([seed_byte; MASTER_SEED_LEN]);
        let envelope =
            encrypt_key_to_envelope(engine.master_seed(), "Test#2026!").expect("must encrypt");

        NodeKeyBundleV1::generate(
            node_name,
            profile,
            "2026-01-01T00:00:00Z".to_string(),
            crypto_profile,
            &engine,
            envelope,
        )
        .expect("bundle generation must succeed")
    }

    #[test]
    fn generated_bundle_contains_all_required_roles() {
        let bundle = make_bundle(
            0x33,
            "validator-01",
            "testnet",
            CryptoProfile::HybridEd25519Dilithium3,
        );

        assert_eq!(bundle.keys.len(), 6);
        assert!(bundle.validate().is_ok());
    }

    #[test]
    fn bundle_roundtrip_preserves_bundle_fingerprint() {
        let bundle = make_bundle(
            0x44,
            "validator-02",
            "mainnet",
            CryptoProfile::ClassicEd25519,
        );

        let json = bundle.to_json().expect("json encoding must succeed");
        let decoded = NodeKeyBundleV1::from_json(&json).expect("json decoding must succeed");

        assert_eq!(bundle.bundle_fingerprint, decoded.bundle_fingerprint);
    }

    #[test]
    fn generated_public_keys_have_ed25519_length() {
        let bundle = make_bundle(
            0x55,
            "validator-03",
            "validation",
            CryptoProfile::ClassicEd25519,
        );

        for record in &bundle.keys {
            let decoded = hex::decode(&record.public_key).expect("public key must be valid hex");
            assert_eq!(decoded.len(), AOXC_ED25519_PUBLIC_KEY_LEN);
            assert_eq!(record.public_key_encoding, AOXC_PUBLIC_KEY_ENCODING);
            assert_eq!(record.algorithm, "ed25519");
        }
    }

    #[test]
    fn different_roles_produce_distinct_public_keys() {
        let bundle = make_bundle(
            0x66,
            "validator-04",
            "devnet",
            CryptoProfile::ClassicEd25519,
        );

        let identity = bundle
            .keys
            .iter()
            .find(|record| record.role == NodeKeyRole::Identity)
            .expect("identity role must exist");

        let consensus = bundle
            .keys
            .iter()
            .find(|record| record.role == NodeKeyRole::Consensus)
            .expect("consensus role must exist");

        assert_ne!(identity.public_key, consensus.public_key);
    }

    #[test]
    fn validator_profile_is_accepted_as_validation_alias() {
        let bundle = make_bundle(
            0x77,
            "validator-05",
            "validator",
            CryptoProfile::ClassicEd25519,
        );

        assert_eq!(bundle.profile, "validation");
        assert_eq!(bundle.keys.len(), 6);
        assert!(bundle.validate().is_ok());
    }

    #[test]
    fn unsupported_profile_is_rejected() {
        let engine = KeyEngine::from_seed([0x88; MASTER_SEED_LEN]);
        let envelope =
            encrypt_key_to_envelope(engine.master_seed(), "Test#2026!").expect("must encrypt");

        let result = NodeKeyBundleV1::generate(
            "validator-06",
            "unknown-profile",
            "2026-01-01T00:00:00Z".to_string(),
            CryptoProfile::ClassicEd25519,
            &engine,
            envelope,
        );

        assert!(matches!(
            result,
            Err(NodeKeyBundleError::UnsupportedProfile(_))
        ));
    }

    #[test]
    fn same_seed_same_profile_same_role_material_is_stable_even_when_bundle_fingerprint_varies() {
        let bundle_a = make_bundle(
            0x99,
            "validator-07",
            "mainnet",
            CryptoProfile::ClassicEd25519,
        );
        let bundle_b = make_bundle(
            0x99,
            "validator-07",
            "mainnet",
            CryptoProfile::ClassicEd25519,
        );

        assert_eq!(bundle_a.keys, bundle_b.keys);
        assert_ne!(bundle_a.bundle_fingerprint, bundle_b.bundle_fingerprint);
    }

    #[test]
    fn different_profiles_produce_distinct_consensus_public_keys() {
        let mainnet_bundle = make_bundle(
            0xAA,
            "validator-08",
            "mainnet",
            CryptoProfile::ClassicEd25519,
        );
        let testnet_bundle = make_bundle(
            0xAA,
            "validator-08",
            "testnet",
            CryptoProfile::ClassicEd25519,
        );

        let mainnet_consensus = mainnet_bundle
            .keys
            .iter()
            .find(|record| record.role == NodeKeyRole::Consensus)
            .expect("mainnet consensus role must exist");

        let testnet_consensus = testnet_bundle
            .keys
            .iter()
            .find(|record| record.role == NodeKeyRole::Consensus)
            .expect("testnet consensus role must exist");

        assert_ne!(mainnet_consensus.public_key, testnet_consensus.public_key);
    }

    #[test]
    fn public_key_bytes_are_available_for_every_required_role() {
        let bundle = make_bundle(
            0xBB,
            "validator-09",
            "localnet",
            CryptoProfile::ClassicEd25519,
        );

        for role in NodeKeyRole::all() {
            let bytes = bundle
                .public_key_bytes_for_role(role)
                .expect("public key bytes must exist for each role");
            assert_eq!(bytes.len(), AOXC_ED25519_PUBLIC_KEY_LEN);
        }
    }

    #[test]
    fn validate_rejects_duplicate_role_records() {
        let mut bundle = make_bundle(
            0xCC,
            "validator-10",
            "validation",
            CryptoProfile::ClassicEd25519,
        );

        let duplicate = bundle
            .keys
            .iter()
            .find(|record| record.role == NodeKeyRole::Consensus)
            .expect("consensus role must exist")
            .clone();

        bundle.keys.push(duplicate);

        let result = bundle.validate();
        assert!(matches!(
            result,
            Err(NodeKeyBundleError::DuplicateRole(NodeKeyRole::Consensus))
        ));
    }

    #[test]
    fn validate_rejects_tampered_hd_path() {
        let mut bundle = make_bundle(
            0xDD,
            "validator-11",
            "testnet",
            CryptoProfile::ClassicEd25519,
        );

        let record = bundle
            .keys
            .iter_mut()
            .find(|record| record.role == NodeKeyRole::Consensus)
            .expect("consensus role must exist");

        record.hd_path = "m/44/2626/1/2/1/0".to_string();

        let result = bundle.validate();
        assert!(matches!(
            result,
            Err(NodeKeyBundleError::HdPathMismatch { .. })
        ));
    }

    #[test]
    fn validate_rejects_invalid_public_key_encoding() {
        let mut bundle = make_bundle(
            0xEE,
            "validator-12",
            "mainnet",
            CryptoProfile::ClassicEd25519,
        );

        let record = bundle
            .keys
            .iter_mut()
            .find(|record| record.role == NodeKeyRole::Identity)
            .expect("identity role must exist");

        record.public_key_encoding = "base64".to_string();

        let result = bundle.validate();
        assert!(matches!(
            result,
            Err(NodeKeyBundleError::InvalidPublicKeyEncoding(
                NodeKeyRole::Identity
            ))
        ));
    }

    #[test]
    fn validate_rejects_tampered_record_fingerprint() {
        let mut bundle = make_bundle(
            0x11,
            "validator-13",
            "mainnet",
            CryptoProfile::ClassicEd25519,
        );

        let record = bundle
            .keys
            .iter_mut()
            .find(|record| record.role == NodeKeyRole::Identity)
            .expect("identity role must exist");

        record.fingerprint = "AAAAAAAAAAAAAAAA".to_string();

        let result = bundle.validate();
        assert!(matches!(
            result,
            Err(NodeKeyBundleError::FingerprintMismatch(
                NodeKeyRole::Identity
            ))
        ));
    }

    #[test]
    fn validate_rejects_bundle_fingerprint_mismatch() {
        let mut bundle = make_bundle(
            0x22,
            "validator-14",
            "mainnet",
            CryptoProfile::ClassicEd25519,
        );

        bundle.bundle_fingerprint = "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA".to_string();

        let result = bundle.validate();
        assert_eq!(result, Err(NodeKeyBundleError::BundleFingerprintMismatch));
    }
}
