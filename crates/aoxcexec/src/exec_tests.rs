#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::{Signer, SigningKey};

    fn sample_context() -> ExecutionContext {
        ExecutionContext {
            block_height: 7,
            timestamp: 1_735_689_600,
            max_gas_per_block: 200_000,
            chain_id: 42,
            replay_domain: "aoxc-mainnet".to_string(),
            max_batch_tx_count: 128,
            max_batch_bytes: 1024 * 1024,
            max_receipt_size: 4096,
            max_total_rejected_payloads_before_abort_threshold: 16,
        }
    }

    fn signing_key_from_seed(seed: u8) -> SigningKey {
        SigningKey::from_bytes(&[seed; 32])
    }

    fn sample_payload(
        tx_hash: [u8; 32],
        signer_seed: u8,
        nonce: u64,
        lane_id: &str,
        gas_limit: Gas,
        size: usize,
    ) -> ExecutionPayload {
        let signing_key = signing_key_from_seed(signer_seed);
        ExecutionPayload {
            version: 1,
            chain_id: 42,
            tx_hash,
            lane_id: lane_id.to_string(),
            sender: signing_key.verifying_key().to_bytes(),
            nonce,
            gas_limit,
            max_fee: gas_limit,
            max_priority_fee: gas_limit / 10,
            expiration_timestamp: 1_735_689_900,
            payload_type: PayloadType::Call,
            access_scope: vec![lane_id.to_string()],
            replay_domain: "aoxc-mainnet".to_string(),
            auth_scheme: AuthScheme::Ed25519,
            signature: vec![0u8; 64],
            data: vec![7u8; size],
        }
        .sign_with_ed25519(&signing_key)
        .expect("signature generation should succeed")
    }

    #[test]
    fn successful_batch_execution_produces_real_commitments_and_results() {
        let orchestrator = DeterministicOrchestrator::default();
        let context = sample_context();
        let payloads = vec![
            sample_payload([1; 32], 11, 0, "native", 50_000, 32),
            sample_payload([2; 32], 22, 0, "evm", 75_000, 64),
        ];

        let outcome = orchestrator
            .execute_batch(&context, &payloads)
            .expect("execution should succeed");

        assert_eq!(outcome.receipts.len(), 2);
        assert_eq!(outcome.results.len(), 2);
        assert!(outcome.receipts.iter().all(|receipt| receipt.success));
        assert_ne!(outcome.state_root, [0u8; 32]);
        assert_ne!(outcome.receipt_root, [0u8; 32]);
        assert_ne!(outcome.transactions_root, [0u8; 32]);
        assert_ne!(outcome.execution_trace_root, [0u8; 32]);
        assert_ne!(outcome.block_execution_root, [0u8; 32]);
    }

    #[test]
    fn duplicate_transactions_reject_the_entire_batch() {
        let orchestrator = DeterministicOrchestrator::default();
        let context = sample_context();
        let payloads = vec![
            sample_payload([9; 32], 1, 0, "native", 50_000, 12),
            sample_payload([9; 32], 2, 0, "evm", 60_000, 16),
        ];

        let result = orchestrator.execute_batch(&context, &payloads);

        assert_eq!(result, Err(ExecutionError::DuplicateTransaction([9; 32])));
    }

    #[test]
    fn duplicate_sender_nonce_rejects_before_execution() {
        let orchestrator = DeterministicOrchestrator::default();
        let context = sample_context();
        let sender = signing_key_from_seed(7).verifying_key().to_bytes();
        let payloads = vec![
            sample_payload([1; 32], 7, 4, "native", 50_000, 12),
            sample_payload([2; 32], 7, 4, "evm", 60_000, 16),
        ];

        let result = orchestrator.execute_batch(&context, &payloads);

        assert_eq!(
            result,
            Err(ExecutionError::DuplicateSenderNonce { sender, nonce: 4 })
        );
    }

    #[test]
    fn invalid_signature_returns_failed_receipt_without_state_mutation() {
        let orchestrator = DeterministicOrchestrator::default();
        let context = sample_context();
        let mut payload = sample_payload([3; 32], 9, 0, "native", 50_000, 12);
        payload.signature = vec![99u8; 64];
        let valid = sample_payload([4; 32], 10, 0, "native", 50_000, 12);

        let outcome = orchestrator
            .execute_batch(&context, &[payload, valid])
            .expect("batch should return receipts");

        assert_eq!(
            outcome
                .receipts
                .iter()
                .filter(|receipt| !receipt.success)
                .count(),
            1
        );
        assert_eq!(
            outcome
                .receipts
                .iter()
                .filter(|receipt| receipt.success)
                .count(),
            1
        );
        assert_eq!(outcome.results.len(), 1);
    }

    #[test]
    fn nonce_gap_yields_canonical_rejection() {
        let orchestrator = DeterministicOrchestrator::default();
        let context = sample_context();
        let payloads = vec![
            sample_payload([1; 32], 8, 0, "native", 50_000, 12),
            sample_payload([2; 32], 8, 2, "native", 50_000, 12),
        ];

        let outcome = orchestrator
            .execute_batch(&context, &payloads)
            .expect("batch should complete");

        assert!(outcome.receipts[0].success);
        assert!(!outcome.receipts[1].success);
        assert_eq!(outcome.summary.nonce_violation_count, 1);
    }

    #[test]
    fn registry_checksum_mismatch_halts_execution() {
        let mut registry = default_lane_registry();
        let native_entries = registry
            .policies
            .get_mut("native")
            .expect("native policy exists");
        native_entries[0].checksum = [0u8; 32];
        let orchestrator = DeterministicOrchestrator::new(
            registry,
            default_lanes(),
            InMemoryStateStore::default(),
        );

        let result = orchestrator.execute_batch(
            &sample_context(),
            &[sample_payload([1; 32], 1, 0, "native", 50_000, 8)],
        );

        assert!(matches!(
            result,
            Err(ExecutionError::ConfigChecksumMismatch { .. })
        ));
    }

    #[test]
    fn serialization_freeze_for_payload_v1_is_stable() {
        let payload = sample_payload([1; 32], 2, 3, "wasm", 90_000, 4);
        let encoded = serde_json::to_string(&payload).expect("serialization should succeed");
        let digest = payload.signing_digest().expect("payload digest");
        let signing_key = signing_key_from_seed(2);
        let expected_signature = signing_key.sign(&digest).to_bytes();
        let sender_json = signing_key
            .verifying_key()
            .to_bytes()
            .iter()
            .map(u8::to_string)
            .collect::<Vec<_>>()
            .join(",");
        let signature_json = expected_signature
            .iter()
            .map(u8::to_string)
            .collect::<Vec<_>>()
            .join(",");
        let expected = format!(
            "{{\"version\":1,\"chain_id\":42,\"tx_hash\":[1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1],\"lane_id\":\"wasm\",\"sender\":[{sender_json}],\"nonce\":3,\"gas_limit\":90000,\"max_fee\":90000,\"max_priority_fee\":9000,\"expiration_timestamp\":1735689900,\"payload_type\":\"Call\",\"access_scope\":[\"wasm\"],\"replay_domain\":\"aoxc-mainnet\",\"auth_scheme\":\"Ed25519\",\"signature\":[{signature_json}],\"data\":[7,7,7,7]}}",
        );
        assert_eq!(encoded, expected);
    }

    #[test]
    fn deterministic_replay_holds_across_payload_sizes() {
        let orchestrator = DeterministicOrchestrator::default();
        let context = sample_context();
        for payload_size in [1usize, 2, 7, 16, 31, 63] {
            let payloads = vec![
                sample_payload([1; 32], 3, 0, "native", 60_000, payload_size),
                sample_payload([2; 32], 4, 0, "evm", 80_000, payload_size),
            ];

            let left = orchestrator
                .execute_batch(&context, &payloads)
                .expect("left outcome");
            let right = orchestrator
                .execute_batch(&context, &payloads)
                .expect("right outcome");

            assert_eq!(left.receipts, right.receipts);
            assert_eq!(left.state_root, right.state_root);
            assert_eq!(left.receipt_root, right.receipt_root);
            assert_eq!(left.block_execution_root, right.block_execution_root);
        }
    }

    #[test]
    fn invalid_tx_never_mutates_state_across_sizes() {
        let orchestrator = DeterministicOrchestrator::default();
        let context = sample_context();
        for size in [1usize, 4, 8, 16, 31] {
            let mut invalid = sample_payload([7; 32], 8, 0, "native", 50_000, size);
            invalid.signature = vec![0u8; 64];
            let outcome = orchestrator
                .execute_batch(&context, &[invalid])
                .expect("outcome");
            assert_eq!(outcome.results.len(), 0);
            assert_eq!(outcome.receipts.len(), 1);
            assert!(!outcome.receipts[0].success);
            assert_eq!(
                outcome.state_root,
                InMemoryStateStore::default().snapshot_root().expect("root")
            );
        }
    }

    #[test]
    fn invalid_context_rejects_zero_max_receipt_size() {
        let mut context = sample_context();
        context.max_receipt_size = 0;
        let orchestrator = DeterministicOrchestrator::default();
        let payloads = vec![sample_payload([1; 32], 2, 0, "native", 50_000, 4)];
        let err = orchestrator
            .execute_batch(&context, &payloads)
            .expect_err("context must be rejected");
        assert_eq!(
            err,
            ExecutionError::InvalidContext("max_receipt_size must be greater than zero")
        );
    }

    #[test]
    fn invalid_scope_item_is_rejected() {
        let orchestrator = DeterministicOrchestrator::default();
        let context = sample_context();
        let mut payload = sample_payload([5; 32], 6, 0, "native", 50_000, 8);
        payload.access_scope = vec!["native".to_string(), "native".to_string()];
        let signing_key = signing_key_from_seed(6);
        payload = payload
            .sign_with_ed25519(&signing_key)
            .expect("signature generation should succeed");

        let outcome = orchestrator
            .execute_batch(&context, &[payload])
            .expect("batch should return receipt");
        assert_eq!(outcome.receipts.len(), 1);
        assert!(!outcome.receipts[0].success);
        assert!(
            outcome.receipts[0]
                .error_message
                .as_deref()
                .unwrap_or_default()
                .contains("access_scope")
        );
    }

    #[test]
    fn invalid_policy_configuration_is_rejected() {
        let bad_policy = LanePolicy {
            lane_id: "native".to_string(),
            enabled: true,
            base_gas: 0,
            gas_per_byte: 1,
            max_payload_bytes: 1024,
            max_gas_per_tx: 1_000_000,
            max_sender_txs_per_block: 10,
        };
        let registry = LaneRegistry::new(vec![LaneRegistryPolicy::new(
            "native-mainnet",
            1,
            1,
            "gov://bootstrap/native/v1",
            bad_policy,
        )]);
        let orchestrator = DeterministicOrchestrator::new(
            registry,
            default_lanes(),
            InMemoryStateStore::default(),
        );
        let context = sample_context();
        let payload = sample_payload([1; 32], 1, 0, "native", 50_000, 8);

        let err = orchestrator
            .execute_batch(&context, &[payload])
            .expect_err("invalid policy should fail");
        assert_eq!(
            err,
            ExecutionError::InvalidPolicy("base_gas must be greater than zero")
        );
    }
}
