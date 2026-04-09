#[cfg(test)]
mod tests {
    use super::*;

    fn bytes32(v: u8) -> [u8; 32] {
        [v; 32]
    }

    fn valid_task() -> Task {
        Task::new(
            bytes32(1),
            Capability::UserSigned,
            TargetOutpost::EthMainnetGateway,
            vec![1, 2, 3],
        )
        .expect("valid task must construct successfully")
    }

    #[test]
    fn active_block_constructs_successfully() {
        let block = Block::new_active_with_timestamp(
            1,
            100,
            bytes32(10),
            bytes32(20),
            bytes32(30),
            vec![valid_task()],
        )
        .expect("active block must construct successfully");

        assert!(block.is_active());
        assert_eq!(block.task_count(), 1);
    }

    #[test]
    fn heartbeat_block_constructs_successfully() {
        let block = Block::new_heartbeat_with_timestamp(2, 200, bytes32(11), bytes32(31))
            .expect("heartbeat block must construct successfully");

        assert!(block.is_heartbeat());
        assert_eq!(block.task_count(), 0);
        assert_eq!(block.header.state_root, ZERO_STATE_ROOT);
    }

    #[test]
    fn epoch_prune_block_constructs_successfully() {
        let block =
            Block::new_epoch_prune_with_timestamp(3, 300, bytes32(12), bytes32(22), bytes32(32))
                .expect("epoch-prune block must construct successfully");

        assert!(block.is_epoch_prune());
        assert_eq!(block.task_count(), 0);
    }

    #[test]
    fn active_block_without_tasks_is_rejected() {
        let result =
            Block::new_active_with_timestamp(1, 100, bytes32(10), bytes32(20), bytes32(30), vec![]);

        assert_eq!(result, Err(BlockError::ActiveBlockRequiresTasks));
    }

    #[test]
    fn empty_task_payload_is_rejected() {
        let result = Task::new(
            bytes32(1),
            Capability::AiAttested,
            TargetOutpost::AovmNative,
            Vec::new(),
        );

        assert_eq!(result, Err(BlockError::EmptyTaskPayload));
    }

    #[test]
    fn heartbeat_with_non_zero_state_root_is_rejected() {
        let block = Block {
            header: BlockHeader {
                height: 5,
                timestamp: 123,
                prev_hash: bytes32(10),
                state_root: bytes32(99),
                producer: bytes32(30),
                quantum_signature_scheme: crate::protocol::quantum::SignatureScheme::MlDsa65,
                quantum_header_proof: vec![0x01],
                block_type: BlockType::Heartbeat,
            },
            tasks: Vec::new(),
        };

        assert_eq!(
            block.validate(),
            Err(BlockError::HeartbeatBlockMustUseZeroStateRoot)
        );
    }

    #[test]
    fn zero_timestamp_is_rejected() {
        let block = Block {
            header: BlockHeader {
                height: 1,
                timestamp: 0,
                prev_hash: bytes32(10),
                state_root: bytes32(20),
                producer: bytes32(30),
                quantum_signature_scheme: crate::protocol::quantum::SignatureScheme::MlDsa65,
                quantum_header_proof: vec![0x01],
                block_type: BlockType::Heartbeat,
            },
            tasks: Vec::new(),
        };

        assert_eq!(block.validate(), Err(BlockError::InvalidTimestamp));
    }

    #[test]
    fn zero_producer_is_rejected() {
        let block = Block {
            header: BlockHeader {
                height: 1,
                timestamp: 1,
                prev_hash: bytes32(10),
                state_root: ZERO_STATE_ROOT,
                producer: ZERO_HASH,
                quantum_signature_scheme: crate::protocol::quantum::SignatureScheme::MlDsa65,
                quantum_header_proof: vec![0x01],
                block_type: BlockType::Heartbeat,
            },
            tasks: Vec::new(),
        };

        assert_eq!(block.validate(), Err(BlockError::InvalidProducer));
    }

    #[test]
    fn empty_quantum_header_proof_is_rejected() {
        let block = Block {
            header: BlockHeader {
                height: 1,
                timestamp: 1,
                prev_hash: bytes32(10),
                state_root: ZERO_STATE_ROOT,
                producer: bytes32(30),
                quantum_signature_scheme: crate::protocol::quantum::SignatureScheme::MlDsa65,
                quantum_header_proof: Vec::new(),
                block_type: BlockType::Heartbeat,
            },
            tasks: Vec::new(),
        };

        assert_eq!(block.validate(), Err(BlockError::InvalidProducer));
    }

    #[test]
    fn empty_quantum_header_proof_is_rejected() {
        let block = Block {
            header: BlockHeader {
                height: 1,
                timestamp: 1,
                prev_hash: bytes32(10),
                state_root: ZERO_STATE_ROOT,
                producer: bytes32(30),
                quantum_signature_scheme: crate::protocol::quantum::SignatureScheme::MlDsa65,
                quantum_header_proof: Vec::new(),
                block_type: BlockType::Heartbeat,
            },
            tasks: Vec::new(),
        };

        assert_eq!(block.validate(), Err(BlockError::InvalidTaskRoot));
    }

    #[test]
    fn non_strict_quantum_signature_scheme_is_rejected() {
        let block = Block {
            header: BlockHeader {
                height: 1,
                timestamp: 1,
                prev_hash: bytes32(10),
                state_root: ZERO_STATE_ROOT,
                producer: bytes32(30),
                quantum_signature_scheme: crate::protocol::quantum::SignatureScheme::Dilithium3,
                quantum_header_proof: vec![0x01],
                block_type: BlockType::Heartbeat,
            },
            tasks: Vec::new(),
        };

        assert_eq!(block.validate(), Err(BlockError::InvalidStateRoot));
    }

    #[test]
    fn block_validate_with_key_bundle_accepts_matching_consensus_producer() {
        use crate::identity::{
            key_bundle::{CryptoProfile, NodeKeyBundleV1, NodeKeyRole},
            key_engine::{KeyEngine, MASTER_SEED_LEN},
            keyfile::encrypt_key_to_envelope,
        };

        let engine = KeyEngine::from_seed([0x66; MASTER_SEED_LEN]);
        let envelope =
            encrypt_key_to_envelope(engine.master_seed(), "Test#2026!").expect("must encrypt");
        let bundle = NodeKeyBundleV1::generate(
            "validator-01",
            "validator",
            "2026-01-01T00:00:00Z".to_string(),
            CryptoProfile::HybridEd25519Dilithium3,
            &engine,
            envelope,
        )
        .expect("bundle generation must succeed");

        let producer = bundle
            .public_key_bytes_for_role(NodeKeyRole::Consensus)
            .expect("consensus key must decode");

        let task = Task::new(
            bytes32(1),
            Capability::UserSigned,
            TargetOutpost::AovmNative,
            vec![1, 2, 3],
        )
        .expect("task must build");

        let block = Block::new_active(1, ZERO_HASH, bytes32(9), producer, vec![task])
            .expect("block must build");

        assert!(block.validate_with_key_bundle(&bundle).is_ok());
    }

    #[test]
    fn parent_link_validation_accepts_valid_chain_link() {
        let genesis = Block::new_active_with_timestamp(
            1,
            100,
            bytes32(0),
            bytes32(10),
            bytes32(20),
            vec![valid_task()],
        )
        .expect("genesis-like block must construct successfully");

        let child = Block::new_active_with_timestamp(
            2,
            101,
            genesis.header_hash(),
            bytes32(11),
            bytes32(21),
            vec![valid_task()],
        )
        .expect("child block must construct successfully");

        assert_eq!(child.validate_parent_link(&genesis), Ok(()));
    }

    #[test]
    fn parent_link_validation_rejects_wrong_previous_hash() {
        let parent = Block::new_active_with_timestamp(
            1,
            100,
            bytes32(0),
            bytes32(10),
            bytes32(20),
            vec![valid_task()],
        )
        .expect("parent block must construct successfully");

        let child = Block::new_active_with_timestamp(
            2,
            101,
            bytes32(77),
            bytes32(11),
            bytes32(21),
            vec![valid_task()],
        )
        .expect("child block must construct successfully");

        assert_eq!(
            child.validate_parent_link(&parent),
            Err(BlockError::InvalidPreviousHash)
        );
    }

    #[test]
    fn duplicate_task_id_detection_works() {
        let task = valid_task();

        let result = Block::new_active_with_timestamp(
            1,
            100,
            bytes32(10),
            bytes32(20),
            bytes32(30),
            vec![task.clone(), task],
        );

        assert_eq!(result, Err(BlockError::DuplicateTaskId));
    }

    #[test]
    fn active_block_with_excessive_total_payload_is_rejected() {
        let payload = vec![7u8; MAX_TASK_PAYLOAD_BYTES];
        let mut tasks = Vec::new();

        for i in 0..65u8 {
            tasks.push(
                Task::new(
                    bytes32(i),
                    Capability::UserSigned,
                    TargetOutpost::AovmNative,
                    payload.clone(),
                )
                .expect("task must construct"),
            );
        }

        let result =
            Block::new_active_with_timestamp(1, 100, bytes32(10), bytes32(20), bytes32(30), tasks);

        assert_eq!(
            result,
            Err(BlockError::TotalPayloadTooLarge {
                size: 65 * MAX_TASK_PAYLOAD_BYTES,
                max: MAX_BLOCK_PAYLOAD_BYTES,
            })
        );
    }

    #[test]
    fn try_task_root_succeeds_for_valid_active_block() {
        let block = Block::new_active_with_timestamp(
            1,
            100,
            bytes32(10),
            bytes32(20),
            bytes32(30),
            vec![valid_task()],
        )
        .expect("active block must construct successfully");

        let root = block
            .try_task_root()
            .expect("task root must be computable for a valid block");

        assert_ne!(root, ZERO_HASH);
    }
}
