// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use aoxcnet::{
    config::NetworkConfig,
    gossip::peer::{NodeCertificate, Peer, PeerRole},
    p2p::P2PNetwork,
    resilience::{ChaosProfile, ResilienceHarness},
};
use aoxcore::{
    block::{Capability, TargetOutpost},
    identity::{
        actor_id::{ActorIdError, generate_actor_id, parse_actor_id, validate_actor_id},
        hd_path::{HdPath, HdPathError, MAX_HD_INDEX},
    },
    transaction::{
        MAX_TRANSACTION_PAYLOAD_BYTES, Transaction, TransactionError, calculate_transaction_root,
        hash_transaction, hash_transaction_intent,
    },
};
use aoxcunity::{
    BlockBody, BlockSection, ConsensusError, ConsensusState, ExternalNetwork, ExternalProofRecord,
    ExternalProofSection, ExternalProofType, LaneCommitment, LaneCommitmentSection, LaneType,
    Proposer, QuorumCertificate, QuorumThreshold, Validator, ValidatorRole, ValidatorRotation,
    Vote, VoteKind,
};
use ed25519_dalek::SigningKey;
use rand::{Rng, SeedableRng, rngs::StdRng};

#[test]
fn transaction_root_and_hashes_remain_stable_under_signature_rotation() {
    let signer = SigningKey::from_bytes(&[7u8; 32]);
    let sender = signer.verifying_key().to_bytes();

    let tx_a = Transaction::new(
        sender,
        41,
        Capability::UserSigned,
        TargetOutpost::EthMainnetGateway,
        b"bridge-intent".to_vec(),
        [9u8; 64],
    )
    .expect("transaction should validate");
    let tx_b = Transaction::new(
        sender,
        41,
        Capability::UserSigned,
        TargetOutpost::EthMainnetGateway,
        b"bridge-intent".to_vec(),
        [3u8; 64],
    )
    .expect("transaction should validate");

    assert_eq!(
        hash_transaction_intent(&tx_a),
        hash_transaction_intent(&tx_b),
        "intent hashes must remain signature-independent"
    );
    assert_ne!(
        hash_transaction(&tx_a),
        hash_transaction(&tx_b),
        "sealed transaction hashes must change when signatures rotate"
    );
    assert_ne!(
        calculate_transaction_root(std::slice::from_ref(&tx_a)),
        calculate_transaction_root(std::slice::from_ref(&tx_b)),
        "collection root must reflect signature-bearing leaf hashes"
    );
}

#[test]
fn transaction_validation_rejects_oversized_payloads() {
    let signer = SigningKey::from_bytes(&[8u8; 32]);
    let error = Transaction::new(
        signer.verifying_key().to_bytes(),
        1,
        Capability::UserSigned,
        TargetOutpost::EthMainnetGateway,
        vec![0xAB; MAX_TRANSACTION_PAYLOAD_BYTES + 1],
        [1u8; 64],
    )
    .expect_err("oversized payload must be rejected");

    assert!(matches!(
        error,
        TransactionError::PayloadTooLarge {
            size,
            max: MAX_TRANSACTION_PAYLOAD_BYTES,
        } if size == MAX_TRANSACTION_PAYLOAD_BYTES + 1
    ));
    assert_eq!(error.code(), "TX_PAYLOAD_TOO_LARGE");
}

#[test]
fn finalized_consensus_rejects_votes_from_conflicting_branch() {
    let validators = [[1u8; 32], [2u8; 32], [3u8; 32]]
        .into_iter()
        .map(|secret| {
            let key = SigningKey::from_bytes(&secret);
            Validator::new(key.verifying_key().to_bytes(), 1, ValidatorRole::Validator)
        })
        .collect::<Vec<_>>();
    let voter = validators[0].id;
    let proposer = validators[1].id;
    let rotation = ValidatorRotation::new(validators).expect("validator rotation");
    let mut consensus = ConsensusState::new(rotation, QuorumThreshold::two_thirds());

    let genesis = build_block([0u8; 32], 1, 1, proposer);
    consensus
        .admit_block(genesis.clone())
        .expect("genesis block should admit");

    let canonical = build_block(genesis.hash, 2, 2, proposer);
    consensus
        .admit_block(canonical.clone())
        .expect("canonical child should admit");

    let certificate = QuorumCertificate::new(
        canonical.hash,
        canonical.header.height,
        2,
        vec![voter],
        1,
        1,
        2626,
        1,
    );
    assert!(consensus.fork_choice.mark_finalized(
        canonical.hash,
        aoxcunity::BlockSeal {
            block_hash: canonical.hash,
            finalized_round: 2,
            attestation_root: certificate.certificate_hash,
            certificate,
        }
    ));

    let stale_vote = Vote {
        voter,
        block_hash: genesis.hash,
        height: genesis.header.height,
        round: genesis.header.round,
        kind: VoteKind::Commit,
    };

    let error = consensus
        .add_vote(stale_vote)
        .expect_err("vote on conflicting branch must be rejected after finality");
    assert!(matches!(error, ConsensusError::StaleVote));
}

#[test]
fn block_production_is_deterministic_for_permuted_body_sections() {
    let proposer = Proposer::new(2626, [7u8; 32]);

    let lane = BlockSection::LaneCommitment(LaneCommitmentSection {
        lanes: vec![LaneCommitment {
            lane_id: 7,
            lane_type: LaneType::Native,
            tx_count: 2,
            input_root: [1u8; 32],
            output_root: [2u8; 32],
            receipt_root: [3u8; 32],
            state_commitment: [4u8; 32],
            proof_commitment: [5u8; 32],
        }],
    });
    let proof = BlockSection::ExternalProof(ExternalProofSection {
        proofs: vec![ExternalProofRecord {
            source_network: ExternalNetwork::Bitcoin,
            proof_type: ExternalProofType::Finality,
            subject_hash: [6u8; 32],
            proof_commitment: [8u8; 32],
            finalized_at: 1_800_000_010,
        }],
    });

    let a = proposer
        .propose(
            [0u8; 32],
            1,
            0,
            1,
            1_800_000_000,
            BlockBody {
                sections: vec![lane.clone(), proof.clone()],
            },
        )
        .expect("first block should build");
    let b = proposer
        .propose(
            [0u8; 32],
            1,
            0,
            1,
            1_800_000_000,
            BlockBody {
                sections: vec![proof, lane],
            },
        )
        .expect("second block should build");

    assert_eq!(a.hash, b.hash);
    assert_eq!(a.header.body_root, b.header.body_root);
}

#[test]
fn fork_choice_accepts_equal_height_siblings_with_deterministic_tiebreak() {
    let validators = [[1u8; 32], [2u8; 32], [3u8; 32]]
        .into_iter()
        .map(|secret| {
            let key = SigningKey::from_bytes(&secret);
            Validator::new(key.verifying_key().to_bytes(), 1, ValidatorRole::Validator)
        })
        .collect::<Vec<_>>();
    let rotation = ValidatorRotation::new(validators).expect("validator rotation");
    let mut consensus = ConsensusState::new(rotation, QuorumThreshold::two_thirds());

    let genesis = build_block([0u8; 32], 1, 1, [9u8; 32]);
    consensus
        .admit_block(genesis.clone())
        .expect("genesis should admit");

    let sibling_a = build_block(genesis.hash, 2, 2, [9u8; 32]);
    let sibling_b = build_block(genesis.hash, 2, 3, [9u8; 32]);
    consensus
        .admit_block(sibling_a.clone())
        .expect("first sibling should admit");
    consensus
        .admit_block(sibling_b.clone())
        .expect("second sibling should admit");

    assert_eq!(
        consensus.fork_choice.get_head(),
        Some(sibling_a.hash.max(sibling_b.hash))
    );
}

#[test]
fn resilience_harness_preserves_integrity_metadata_under_chaos() {
    let mut network = P2PNetwork::new(NetworkConfig::default());
    network.register_peer(peer()).expect("peer registers");
    network
        .establish_session("node-1")
        .expect("session established");

    let mut harness = ResilienceHarness::new(ChaosProfile {
        seed: 42,
        drop_every_nth: 0,
        duplicate_every_nth: 2,
        reorder_window: 3,
        max_jitter_ms: 9,
        max_inflight_frames: 16,
    })
    .expect("valid profile");

    let mut expected_hashes = Vec::new();
    for round in 1..=4 {
        let envelope = network
            .broadcast_secure(
                "node-1",
                aoxcunity::ConsensusMessage::BlockProposal {
                    block: build_block([round as u8; 32], round + 1, round, [5u8; 32]),
                },
            )
            .expect("broadcast should work");
        expected_hashes.push((
            envelope.session_id.clone(),
            envelope.nonce,
            envelope.payload_hash_hex.clone(),
        ));
        harness.enqueue(envelope).expect("frame should enqueue");
    }

    let report = harness.flush();
    assert_eq!(report.accepted_frames, 4);
    assert_eq!(report.duplicated_frames, 2);
    assert_eq!(report.delivered.len(), 6);
    assert!(report.reordered_frames >= 1);

    for (session_id, nonce, payload_hash_hex) in expected_hashes {
        let delivered = report
            .delivered
            .iter()
            .filter(|event| event.session_id == session_id && event.nonce == nonce)
            .collect::<Vec<_>>();
        assert!(
            !delivered.is_empty(),
            "every accepted frame should be delivered at least once"
        );
        assert!(
            delivered
                .iter()
                .all(|event| event.payload_hash_hex == payload_hash_hex),
            "chaos simulation must not mutate payload integrity metadata"
        );
    }
}

#[test]
fn identity_surface_stays_stable_under_randomized_roundtrip_pressure() {
    let mut rng = StdRng::seed_from_u64(0xA0C1_1D3A_u64);

    for _ in 0..1_500 {
        let seed = ((rng.next_u32() as u64) << 32) | (rng.next_u32() as u64);
        let mut secret = [0u8; 32];
        secret[..8].copy_from_slice(&seed.to_le_bytes());
        if secret.iter().all(|b| *b == 0) {
            secret[0] = 1;
        }
        let signer = SigningKey::from_bytes(&secret);
        let pubkey = signer.verifying_key().to_bytes();

        let role = if (rng.next_u32() & 1) == 0 {
            "validator"
        } else {
            "observer"
        };
        let zone = if (rng.next_u32() & 1) == 0 {
            "eu"
        } else {
            "na"
        };

        let actor_id = generate_actor_id(&pubkey, role, zone).expect("actor-id should generate");
        assert_eq!(validate_actor_id(&actor_id), Ok(()));

        let parsed = parse_actor_id(&actor_id).expect("actor-id should parse");
        assert_eq!(parsed.prefix, "AOXC");

        let chain = rng.next_u32() & MAX_HD_INDEX;
        let role_idx = rng.next_u32() & MAX_HD_INDEX;
        let zone_idx = rng.next_u32() & MAX_HD_INDEX;
        let index = rng.next_u32() & MAX_HD_INDEX;

        let hd = HdPath::new(chain, role_idx, zone_idx, index).expect("hd path should build");
        let serialized = hd.to_string();
        let reparsed: HdPath = serialized.parse().expect("hd path should parse");
        assert_eq!(reparsed, hd);
    }
}

#[test]
fn identity_surface_rejects_large_malformed_input_corpus() {
    let malformed_actor_ids = [
        "",
        " ",
        "AOXC-VA-EU-ABCDEF-K9",
        "AOXC-VAL-E-ABCDEF-K9",
        "AOXC-VAL-EU-ABC DEF-K9",
        "AOXC-VAL-EU-ABCDEF🙂-K9",
        "NOPE-VAL-EU-AAAAAAAAAAAAAAAAAAAAAAAA-K9",
        "AOXC-VAL-EU-AAAAAAAAAAAAAAAAAAAAAAAA-IO",
    ];

    for candidate in malformed_actor_ids {
        assert!(
            validate_actor_id(candidate).is_err(),
            "expected malformed actor id to fail: {candidate:?}"
        );
    }

    let malformed_hd_paths = [
        "",
        "m/44/2626/1/2/3",
        "root/44/2626/1/2/3/4",
        "m/44/2626/1/2/3/-1",
        "m/44/9999/1/2/3/4",
        "m/43/2626/1/2/3/4",
        " m/44/2626/1/2/3/4 ",
    ];

    for candidate in malformed_hd_paths {
        assert!(
            candidate.parse::<HdPath>().is_err(),
            "expected malformed hd path to fail: {candidate:?}"
        );
    }

    let overflow = format!("m/44/2626/1/2/3/{}", MAX_HD_INDEX + 1);
    assert_eq!(overflow.parse::<HdPath>(), Err(HdPathError::IndexOverflow));

    let non_empty_error = validate_actor_id("AOXC-VAL-EU-ABCD").expect_err("must fail");
    assert!(!matches!(non_empty_error, ActorIdError::EmptyActorId));
}

fn build_block(
    parent_hash: [u8; 32],
    height: u64,
    round: u64,
    proposer: [u8; 32],
) -> aoxcunity::Block {
    let parent = if height == 1 { [0u8; 32] } else { parent_hash };
    Proposer::new(2626, proposer)
        .propose(
            parent,
            height,
            0,
            round,
            1_800_000_000 + round,
            sample_body(height as u8),
        )
        .expect("block should build")
}

fn sample_body(seed: u8) -> BlockBody {
    BlockBody {
        sections: vec![aoxcunity::BlockSection::LaneCommitment(
            LaneCommitmentSection {
                lanes: vec![LaneCommitment {
                    lane_id: u32::from(seed),
                    lane_type: LaneType::Native,
                    tx_count: 1,
                    input_root: [seed; 32],
                    output_root: [seed.wrapping_add(1); 32],
                    receipt_root: [seed.wrapping_add(2); 32],
                    state_commitment: [seed.wrapping_add(3); 32],
                    proof_commitment: [seed.wrapping_add(4); 32],
                }],
            },
        )],
    }
}

fn peer() -> Peer {
    let certificate = NodeCertificate {
        subject: "node-1".to_string(),
        issuer: "AOXC-ROOT".to_string(),
        valid_from_unix: 1,
        valid_until_unix: u64::MAX,
        serial: "serial-1".to_string(),
        domain_attestation_hash: "attestation-1".to_string(),
    };

    Peer::new(
        "node-1",
        "10.0.0.1:2727",
        "AOXC-MAINNET",
        aoxcnet::config::ExternalDomainKind::Native,
        PeerRole::Validator,
        3,
        true,
        certificate,
    )
}
