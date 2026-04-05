#[cfg(test)]
mod tests {
    use super::*;
    use core::mem::size_of;

    fn addr(byte: u8) -> Address {
        [byte; 32]
    }

    #[test]
    fn policy_profiles_validate_successfully() {
        assert!(
            NativeTokenPolicy::for_network(NativeTokenNetwork::Mainnet)
                .validate()
                .is_ok()
        );
        assert!(
            NativeTokenPolicy::for_network(NativeTokenNetwork::Testnet)
                .validate()
                .is_ok()
        );
        assert!(
            NativeTokenPolicy::for_network(NativeTokenNetwork::Devnet)
                .validate()
                .is_ok()
        );
    }

    #[test]
    fn new_ledger_rejects_invalid_policy() {
        let policy = NativeTokenPolicy {
            version: 99,
            ..NativeTokenPolicy::default()
        };

        let err = NativeTokenLedger::new(policy).unwrap_err();
        assert_eq!(err, NativeTokenError::InvalidPolicy);
    }

    #[test]
    fn mint_updates_supply_and_balance() {
        let mut ledger = NativeTokenLedger::new_for_network(NativeTokenNetwork::Mainnet).unwrap();

        ledger.mint(addr(1), 100).unwrap();

        assert_eq!(ledger.total_supply, 100);
        assert_eq!(ledger.balance_of(&addr(1)), 100);
        assert_eq!(ledger.policy.symbol, NATIVE_TOKEN_SYMBOL);
    }

    #[test]
    fn transfer_moves_balance_without_changing_supply() {
        let mut ledger = NativeTokenLedger::new_for_network(NativeTokenNetwork::Mainnet).unwrap();
        ledger.mint(addr(1), 100).unwrap();

        ledger.transfer(addr(1), addr(2), 30).unwrap();

        assert_eq!(ledger.total_supply, 100);
        assert_eq!(ledger.balance_of(&addr(1)), 70);
        assert_eq!(ledger.balance_of(&addr(2)), 30);
    }

    #[test]
    fn transfer_fails_when_balance_is_insufficient() {
        let mut ledger = NativeTokenLedger::new_for_network(NativeTokenNetwork::Mainnet).unwrap();
        ledger.mint(addr(1), 10).unwrap();

        let err = ledger.transfer(addr(1), addr(2), 11).unwrap_err();
        assert_eq!(err, NativeTokenError::InsufficientBalance);
    }

    #[test]
    fn receipts_emit_expected_events_and_codes() {
        let ledger = NativeTokenLedger::new_for_network(NativeTokenNetwork::Mainnet).unwrap();

        let mint_receipt = ledger
            .mint_receipt([7; HASH_SIZE], addr(9), 42, 21)
            .unwrap();
        assert!(mint_receipt.success);
        assert_eq!(mint_receipt.events.len(), 1);
        assert_eq!(mint_receipt.events[0].event_type, EVENT_NATIVE_MINT);
        assert_eq!(mint_receipt.events[0].data.len(), 80);

        let error_receipt = ledger
            .error_receipt([8; HASH_SIZE], 17, NativeTokenError::InsufficientBalance)
            .unwrap();
        assert!(!error_receipt.success);
        assert_eq!(
            error_receipt.error_code,
            Some(ERROR_CODE_INSUFFICIENT_BALANCE)
        );
    }

    #[test]
    fn mint_is_rejected_when_supply_model_disables_mint() {
        let mut ledger = NativeTokenLedger::new(NativeTokenPolicy {
            supply_model: SupplyModel::FixedGenesis,
            ..NativeTokenPolicy::default()
        })
        .unwrap();

        let err = ledger.mint(addr(1), 10).unwrap_err();
        assert_eq!(err, NativeTokenError::MintDisabledPolicy);
    }

    #[test]
    fn network_profiles_are_distinct_and_quantum_domains_do_not_overlap() {
        let mainnet = NativeTokenPolicy::for_network(NativeTokenNetwork::Mainnet);
        let testnet = NativeTokenPolicy::for_network(NativeTokenNetwork::Testnet);
        let devnet = NativeTokenPolicy::for_network(NativeTokenNetwork::Devnet);

        assert_eq!(mainnet.decimals, 18);
        assert_eq!(testnet.decimals, 18);
        assert_eq!(devnet.decimals, 18);
        assert_ne!(
            mainnet.quantum_policy.anti_replay_domain,
            testnet.quantum_policy.anti_replay_domain
        );
        assert_ne!(
            testnet.quantum_policy.anti_replay_domain,
            devnet.quantum_policy.anti_replay_domain
        );
    }

    #[test]
    fn quantum_transfer_rejects_empty_proof_tag() {
        let mut ledger = NativeTokenLedger::new_for_network(NativeTokenNetwork::Mainnet).unwrap();
        ledger.mint(addr(1), 1_000).unwrap();

        let error = ledger
            .transfer_quantum(addr(1), addr(2), 100, 1, b"")
            .unwrap_err();

        assert_eq!(error, NativeTokenError::InvalidProofTag);
    }

    #[test]
    fn quantum_transfer_rejects_replay_and_nonce_regression() {
        let mut ledger = NativeTokenLedger::new_for_network(NativeTokenNetwork::Mainnet).unwrap();
        ledger.mint(addr(1), 1_000).unwrap();

        ledger
            .transfer_quantum(addr(1), addr(2), 100, 1, b"sig-proof")
            .unwrap();

        let replay_err = ledger
            .transfer_quantum(addr(1), addr(2), 100, 1, b"sig-proof")
            .unwrap_err();
        assert_eq!(replay_err, NativeTokenError::ReplayDetected);

        let regression_err = ledger
            .transfer_quantum(addr(1), addr(2), 100, 0, b"other-proof")
            .unwrap_err();
        assert_eq!(regression_err, NativeTokenError::NonceRegression);
    }

    #[test]
    fn quantum_transfer_rejects_duplicate_commitment_even_if_nonce_path_is_bypassed() {
        let mut ledger = NativeTokenLedger::new_for_network(NativeTokenNetwork::Mainnet).unwrap();
        ledger.mint(addr(1), 1_000).unwrap();

        let digest = ledger.quantum_transfer_digest(addr(1), addr(2), 100, 9, b"proof");
        ledger.consumed_quantum_commitments.insert(digest.digest);

        let error = ledger
            .transfer_quantum(addr(1), addr(2), 100, 9, b"proof")
            .unwrap_err();

        assert_eq!(error, NativeTokenError::ReplayDetected);
    }

    #[test]
    fn quantum_transfer_updates_nonce_and_commitment_store() {
        let mut ledger = NativeTokenLedger::new_for_network(NativeTokenNetwork::Mainnet).unwrap();
        ledger.mint(addr(1), 500).unwrap();

        let digest = ledger.quantum_transfer_digest(addr(1), addr(2), 50, 3, b"proof");

        ledger
            .transfer_quantum(addr(1), addr(2), 50, 3, b"proof")
            .unwrap();

        assert_eq!(ledger.latest_nonce_of(&addr(1)), Some(3));
        assert!(ledger.has_consumed_quantum_commitment(&digest.digest));
    }

    #[test]
    fn quantum_transfer_event_encoding_contains_expected_layout() {
        let from = addr(1);
        let to = addr(2);
        let amount = 77u128;
        let nonce = 9u64;
        let payload = encode_quantum_transfer_event_v1(
            "AOXC/NATIVE_TOKEN/TESTNET/V1",
            from,
            to,
            amount,
            nonce,
            b"proof",
        );

        let expected_len = 1
            + size_of::<Address>()
            + size_of::<Address>()
            + size_of::<u128>()
            + size_of::<u64>()
            + HASH_SIZE;
        assert_eq!(payload[0], NATIVE_TOKEN_QUANTUM_EVENT_VERSION);
        assert_eq!(payload.len(), expected_len);
        assert_eq!(&payload[1..33], &from);
        assert_eq!(&payload[33..65], &to);
        assert_eq!(&payload[65..81], &amount.to_le_bytes());
        assert_eq!(&payload[81..89], &nonce.to_le_bytes());
    }

    #[test]
    fn computed_quantum_transfer_digest_is_deterministic() {
        let a = compute_quantum_transfer_digest(
            "AOXC/NATIVE_TOKEN/MAINNET/V1",
            addr(1),
            addr(2),
            10,
            7,
            b"proof",
        );
        let b = compute_quantum_transfer_digest(
            "AOXC/NATIVE_TOKEN/MAINNET/V1",
            addr(1),
            addr(2),
            10,
            7,
            b"proof",
        );

        assert_eq!(a, b);
    }

    #[test]
    fn quantum_receipt_is_constructed_successfully() {
        let ledger = NativeTokenLedger::new_for_network(NativeTokenNetwork::Mainnet).unwrap();

        let receipt = ledger
            .transfer_quantum_receipt([9; HASH_SIZE], addr(1), addr(2), 55, 4, b"proof", 88)
            .unwrap();

        assert!(receipt.success);
        assert_eq!(receipt.events.len(), 1);
        assert_eq!(
            receipt.events[0].event_type,
            EVENT_NATIVE_TRANSFER_QUANTUM_V1
        );
        assert_eq!(
            receipt.events[0].data.len(),
            1 + size_of::<Address>()
                + size_of::<Address>()
                + size_of::<u128>()
                + size_of::<u64>()
                + HASH_SIZE
        );
    }
}
