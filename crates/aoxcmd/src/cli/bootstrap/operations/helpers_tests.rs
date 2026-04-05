#[cfg(test)]
mod tests {
    use super::super::{
        BootstrapBootnodeRecord, BootstrapBootnodesDocument, BootstrapValidatorBindingRecord,
        BootstrapValidatorBindingsDocument, CanonicalIdentity, EnvironmentProfile,
        consensus_profile_gate_status, derive_short_fingerprint, evaluate_consensus_profile_audit,
        upsert_bootnode_binding, upsert_validator_binding, validate_genesis,
    };
    use std::{
        env, fs,
        time::{SystemTime, UNIX_EPOCH},
    };

    fn canonical_identity() -> CanonicalIdentity {
        CanonicalIdentity {
            family_id: 2626,
            chain_name: "AOXC TEST".to_string(),
            network_class: "validation".to_string(),
            network_serial: "2626-004".to_string(),
            chain_id: 2_626_030_001,
            network_id: "aoxc-validation-2626-004".to_string(),
        }
    }

    #[test]
    fn derive_short_fingerprint_returns_16_hex_characters() {
        let value = derive_short_fingerprint("validator-01");
        assert_eq!(value.len(), 16);
        assert!(value.chars().all(|ch| ch.is_ascii_hexdigit()));
    }

    #[test]
    fn upsert_validator_binding_replaces_existing_record() {
        let mut doc = BootstrapValidatorBindingsDocument {
            schema_version: 2,
            environment: "validation".to_string(),
            identity: canonical_identity(),
            validators: vec![BootstrapValidatorBindingRecord {
                validator_id: "val-01".to_string(),
                display_name: "Validator 01".to_string(),
                role: "validator".to_string(),
                consensus_key_algorithm: "ed25519".to_string(),
                consensus_public_key_encoding: "hex".to_string(),
                consensus_public_key: "abc".to_string(),
                consensus_key_fingerprint: "fp1".to_string(),
                network_key_algorithm: "ed25519".to_string(),
                network_public_key_encoding: "hex".to_string(),
                network_public_key: "def".to_string(),
                network_key_fingerprint: "fp2".to_string(),
                weight: 1,
                status: "active".to_string(),
            }],
        };

        upsert_validator_binding(
            &mut doc,
            BootstrapValidatorBindingRecord {
                validator_id: "val-01".to_string(),
                display_name: "Validator One".to_string(),
                role: "validator".to_string(),
                consensus_key_algorithm: "ed25519".to_string(),
                consensus_public_key_encoding: "hex".to_string(),
                consensus_public_key: "new-consensus".to_string(),
                consensus_key_fingerprint: "new-fp1".to_string(),
                network_key_algorithm: "ed25519".to_string(),
                network_public_key_encoding: "hex".to_string(),
                network_public_key: "new-network".to_string(),
                network_key_fingerprint: "new-fp2".to_string(),
                weight: 2,
                status: "active".to_string(),
            },
        );

        assert_eq!(doc.validators.len(), 1);
        assert_eq!(doc.validators[0].display_name, "Validator One");
        assert_eq!(doc.validators[0].consensus_public_key, "new-consensus");
        assert_eq!(doc.validators[0].weight, 2);
    }

    #[test]
    fn upsert_bootnode_binding_replaces_existing_record() {
        let mut doc = BootstrapBootnodesDocument {
            schema_version: 2,
            environment: "validation".to_string(),
            identity: canonical_identity(),
            bootnodes: vec![BootstrapBootnodeRecord {
                node_id: "boot-01".to_string(),
                display_name: "Boot 01".to_string(),
                transport_key_algorithm: "ed25519".to_string(),
                transport_public_key_encoding: "hex".to_string(),
                transport_public_key: "pk1".to_string(),
                transport_key_fingerprint: "fp1".to_string(),
                address: "127.0.0.1:39001".to_string(),
                transport: "tcp".to_string(),
                status: "active".to_string(),
            }],
        };

        upsert_bootnode_binding(
            &mut doc,
            BootstrapBootnodeRecord {
                node_id: "boot-01".to_string(),
                display_name: "Boot Updated".to_string(),
                transport_key_algorithm: "ed25519".to_string(),
                transport_public_key_encoding: "hex".to_string(),
                transport_public_key: "pk2".to_string(),
                transport_key_fingerprint: "fp2".to_string(),
                address: "10.0.0.1:39001".to_string(),
                transport: "tcp".to_string(),
                status: "active".to_string(),
            },
        );

        assert_eq!(doc.bootnodes.len(), 1);
        assert_eq!(doc.bootnodes[0].display_name, "Boot Updated");
        assert_eq!(doc.bootnodes[0].address, "10.0.0.1:39001");
    }

    #[test]
    fn consensus_profile_audit_blocks_classical_mainnet() {
        let mut genesis = EnvironmentProfile::Mainnet.genesis_document();
        genesis.consensus.consensus_identity_profile = "classical".to_string();

        let report = evaluate_consensus_profile_audit(
            &genesis,
            EnvironmentProfile::Mainnet,
            "memory://mainnet".to_string(),
        );

        assert_eq!(report.verdict, "fail");
        assert!(
            report
                .blockers
                .iter()
                .any(|item| item.contains("must not run with classical-only"))
        );
    }

    #[test]
    fn consensus_profile_audit_passes_hybrid_validation_profile() {
        let genesis = EnvironmentProfile::Validation.genesis_document();

        let report = evaluate_consensus_profile_audit(
            &genesis,
            EnvironmentProfile::Validation,
            "memory://validation".to_string(),
        );

        assert_eq!(report.verdict, "pass");
        assert!(report.blockers.is_empty());
    }

    #[test]
    fn consensus_profile_audit_blocks_identity_mismatch() {
        let mut genesis = EnvironmentProfile::Mainnet.genesis_document();
        genesis.identity.network_id = "aoxc-mainnet-invalid".to_string();

        let report = evaluate_consensus_profile_audit(
            &genesis,
            EnvironmentProfile::Mainnet,
            "memory://mainnet".to_string(),
        );

        assert_eq!(report.verdict, "fail");
        assert!(
            report
                .blockers
                .iter()
                .any(|item| item.contains("identity network_id"))
        );
    }

    #[test]
    fn consensus_profile_audit_blocks_invalid_consensus_engine() {
        let mut genesis = EnvironmentProfile::Testnet.genesis_document();
        genesis.consensus.engine = "other-engine".to_string();

        let report = evaluate_consensus_profile_audit(
            &genesis,
            EnvironmentProfile::Testnet,
            "memory://testnet".to_string(),
        );

        assert_eq!(report.verdict, "fail");
        assert!(
            report
                .blockers
                .iter()
                .any(|item| item.contains("unsupported consensus engine"))
        );
    }

    #[test]
    fn consensus_profile_gate_status_reports_pass_for_hybrid_testnet() {
        let genesis = EnvironmentProfile::Testnet.genesis_document();
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time should be monotonic")
            .as_nanos();
        let path = env::temp_dir().join(format!("aoxc-bootstrap-gate-{unique}.json"));
        fs::write(
            &path,
            serde_json::to_string(&genesis).expect("genesis should encode"),
        )
        .expect("genesis file should write");

        let status = consensus_profile_gate_status(Some(&path), Some("testnet"))
            .expect("gate status should evaluate");

        assert!(status.passed);
        assert_eq!(status.verdict, "pass");
        assert!(status.detail.contains("consensus_profile=hybrid"));

        let _ = fs::remove_file(path);
    }

    #[test]
    fn validate_genesis_rejects_unsupported_account_role() {
        let mut genesis = EnvironmentProfile::Validation.genesis_document();
        genesis.state.accounts[0].role = "bridge".to_string();

        let error = validate_genesis(&genesis).expect_err("unsupported role must fail");
        assert!(
            error.to_string().contains("unsupported account role"),
            "error should describe unsupported account role"
        );
    }

    #[test]
    fn validate_genesis_accepts_core7_account_roles() {
        let mut genesis = EnvironmentProfile::Validation.genesis_document();
        genesis
            .state
            .accounts
            .push(super::super::BootstrapAccountRecord {
                account_id: "AOXC_CORE7_FORGE".to_string(),
                balance: "1000".to_string(),
                role: "forge".to_string(),
            });

        validate_genesis(&genesis).expect("core7 roles should be accepted");
    }
}
