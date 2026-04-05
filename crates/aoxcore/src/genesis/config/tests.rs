#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_public_mainnet_chain_id_correctly() {
        let chain_id = build_chain_id(AOXC_FAMILY_ID, NetworkClass::PublicMainnet, 1).unwrap();
        assert_eq!(chain_id, 2626000001);
    }

    #[test]
    fn builds_public_testnet_chain_id_correctly() {
        let chain_id = build_chain_id(AOXC_FAMILY_ID, NetworkClass::PublicTestnet, 1).unwrap();
        assert_eq!(chain_id, 2626010001);
    }

    #[test]
    fn builds_network_serial_correctly() {
        assert_eq!(build_network_serial(2626, 1), "2626-001");
        assert_eq!(build_network_serial(2626, 12), "2626-012");
    }

    #[test]
    fn builds_network_id_correctly() {
        let network_id = build_network_id(NetworkClass::PublicMainnet, "2626-001");
        assert_eq!(network_id, "aoxc-mainnet-2626-001");
    }

    #[test]
    fn validates_chain_identity_successfully() {
        let identity = ChainIdentity::new(
            AOXC_FAMILY_ID,
            NetworkClass::PublicMainnet,
            1,
            1,
            "AOXC Mihver",
        )
        .unwrap();

        assert_eq!(identity.chain_id, 2626000001);
        assert_eq!(identity.network_serial, "2626-001");
        assert_eq!(identity.network_id, "aoxc-mainnet-2626-001");
    }

    #[test]
    fn genesis_fingerprint_is_deterministic() {
        let identity = ChainIdentity::new(
            AOXC_FAMILY_ID,
            NetworkClass::PublicMainnet,
            1,
            1,
            "AOXC Mihver",
        )
        .unwrap();

        let cfg = GenesisConfig::new(
            identity,
            3000,
            vec![Validator { id: "val-1".into() }],
            vec![GenesisAccount {
                address: "aox1test".into(),
                balance: 1_000_000,
            }],
            10_000_000,
            SettlementLink {
                endpoint: "settlement://root".into(),
            },
            AOXCANDSeal {
                seal_id: "seal-001".into(),
            },
        )
        .unwrap();

        let fp1 = cfg.fingerprint().unwrap();
        let fp2 = cfg.fingerprint().unwrap();

        assert_eq!(fp1, fp2);
    }

    #[test]
    fn rejects_duplicate_validator_ids() {
        let identity =
            ChainIdentity::new(AOXC_FAMILY_ID, NetworkClass::Devnet, 3, 1, "AOXC Kivilcim")
                .unwrap();

        let err = GenesisConfig::new(
            identity,
            3000,
            vec![
                Validator { id: "val-1".into() },
                Validator { id: "val-1".into() },
            ],
            vec![GenesisAccount {
                address: "acct-1".into(),
                balance: 10,
            }],
            10,
            SettlementLink {
                endpoint: "aoxc://settlement/root".into(),
            },
            AOXCANDSeal {
                seal_id: "seal-1".into(),
            },
        )
        .unwrap_err();

        assert!(matches!(
            err,
            GenesisConfigError::DuplicateValidatorId { .. }
        ));
    }

    #[test]
    fn mainnet_quantum_policy_is_strict() {
        let policy = QuantumPolicy::for_network_class(NetworkClass::PublicMainnet);
        assert_eq!(policy.handshake_kem, "ML-KEM-1024");
        assert!(policy.min_signature_threshold >= 3);
        assert!(policy.pq_signature_schemes.iter().any(|v| v == "ML-DSA-87"));
    }

    #[test]
    fn testnet_and_devnet_receive_distinct_quantum_profiles() {
        let testnet = QuantumPolicy::for_network_class(NetworkClass::PublicTestnet);
        let devnet = QuantumPolicy::for_network_class(NetworkClass::Devnet);

        assert_ne!(testnet.handshake_kem, devnet.handshake_kem);
        assert!(testnet.rotation_epoch_blocks > devnet.rotation_epoch_blocks);
    }

    #[test]
    fn default_timestamp_is_deterministic() {
        assert_eq!(default_genesis_timestamp(), DEFAULT_GENESIS_TIMESTAMP_UNIX);
    }

    #[test]
    fn node_policy_defaults_cover_all_seven_roles() {
        let policy = NodePolicy::for_network_class(NetworkClass::PublicTestnet);
        assert_eq!(policy.role_policies.len(), 7);
        assert!(
            policy
                .role_policies
                .iter()
                .any(|role| role.role == NodeRole::Seal && role.quantum_seal_required)
        );
        assert!(
            policy
                .role_policies
                .iter()
                .all(|role| role.multisig_threshold >= 2)
        );
    }

    #[test]
    fn rejects_node_policy_missing_required_role() {
        let identity =
            ChainIdentity::new(AOXC_FAMILY_ID, NetworkClass::Devnet, 3, 1, "AOXC Kivilcim")
                .unwrap();

        let mut cfg = GenesisConfig::new(
            identity,
            3000,
            vec![Validator { id: "val-1".into() }],
            vec![GenesisAccount {
                address: "acct-1".into(),
                balance: 10,
            }],
            10,
            SettlementLink {
                endpoint: "aoxc://settlement/root".into(),
            },
            AOXCANDSeal {
                seal_id: "seal-1".into(),
            },
        )
        .unwrap();

        cfg.node_policy
            .role_policies
            .retain(|policy| policy.role != NodeRole::Seal);

        let err = cfg.validate().unwrap_err();
        assert!(matches!(
            err,
            GenesisConfigError::InvalidNodePolicy | GenesisConfigError::MissingNodeRolePolicy { .. }
        ));
    }

    #[test]
    fn node_policy_multisig_quorum_check_is_kernel_enforced() {
        let policy = NodePolicy::for_network_class(NetworkClass::PublicMainnet);

        let accepted = policy
            .is_multisig_quorum_satisfied(NodeRole::Quorum, 4, 7)
            .unwrap();
        let rejected = policy
            .is_multisig_quorum_satisfied(NodeRole::Quorum, 3, 7)
            .unwrap();

        assert!(accepted);
        assert!(!rejected);
    }

    #[test]
    fn node_policy_accepts_extensible_seal_layers() {
        let mut policy = NodePolicy::for_network_class(NetworkClass::PublicTestnet);
        policy.seal_layers.push(SealLayerPolicy {
            layer_id: "seal-v2".into(),
            commitment_hash: "SHA3-256".into(),
            activation_epoch: 90,
            quantum_hardened: true,
        });

        assert!(policy
            .validate_for_network_class(NetworkClass::PublicTestnet)
            .is_ok());
    }

    #[test]
    fn node_policy_rejects_invalid_multisig_signer_sets() {
        let policy = NodePolicy::for_network_class(NetworkClass::PublicMainnet);
        let eligible_signers: HashSet<String> = [
            "quorum-1".to_string(),
            "quorum-2".to_string(),
            "quorum-3".to_string(),
            "quorum-4".to_string(),
            "quorum-5".to_string(),
            "quorum-6".to_string(),
            "quorum-7".to_string(),
        ]
        .into_iter()
        .collect();

        let err = policy
            .validate_multisig_signers(
                NodeRole::Quorum,
                &[
                    "quorum-1".into(),
                    "quorum-2".into(),
                    "quorum-3".into(),
                    "outsider".into(),
                ],
                &eligible_signers,
            )
            .unwrap_err();

        assert!(matches!(err, GenesisConfigError::InvalidMultisigSigner { .. }));
    }

    #[test]
    fn node_policy_rejects_non_quantum_genesis_seal_layers() {
        let mut policy = NodePolicy::for_network_class(NetworkClass::Devnet);
        policy.seal_layers = vec![SealLayerPolicy {
            layer_id: "seal-v1".into(),
            commitment_hash: "BLAKE3".into(),
            activation_epoch: 0,
            quantum_hardened: false,
        }];

        let err = policy
            .validate_for_network_class(NetworkClass::Devnet)
            .unwrap_err();
        assert!(matches!(err, GenesisConfigError::WeakNodeRolePolicy { .. }));
    }
}
