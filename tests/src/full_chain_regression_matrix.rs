// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use aoxcnet::{
    config::NetworkConfig,
    gossip::peer::{NodeCertificate, Peer, PeerRole},
    p2p::P2PNetwork,
};
use aoxcore::native_token::{NativeTokenError, NativeTokenLedger, NativeTokenNetwork};
use aoxcunity::{
    BlockBody, ConsensusMessage, LaneCommitment, LaneCommitmentSection, LaneType, Proposer,
    QuorumThreshold, ConsensusState, Validator, ValidatorRole, ValidatorRotation, Vote, VoteKind,
};
use ed25519_dalek::SigningKey;

#[test]
fn quorum_threshold_rejects_invalid_ratios() {
    assert!(QuorumThreshold::new(0, 3).is_err());
    assert!(QuorumThreshold::new(3, 0).is_err());
    assert!(QuorumThreshold::new(4, 3).is_err());
}

#[test]
fn quorum_threshold_enforces_edge_boundaries() {
    let threshold = QuorumThreshold::two_thirds();
    assert!(threshold.is_reached(2, 3));
    assert!(!threshold.is_reached(1, 3));

    // exact boundary: 6/9 reaches 2/3, 5/9 does not.
    assert!(threshold.is_reached(6, 9));
    assert!(!threshold.is_reached(5, 9));
}

#[test]
fn certificate_structure_rejects_empty_and_invalid_windows() {
    let mut cert = certificate();
    cert.subject = "   ".to_string();
    assert!(cert.validate_structure().is_err());

    let mut cert = certificate();
    cert.issuer = "".to_string();
    assert!(cert.validate_structure().is_err());

    let mut cert = certificate();
    cert.serial = "".to_string();
    assert!(cert.validate_structure().is_err());

    let mut cert = certificate();
    cert.valid_until_unix = cert.valid_from_unix.saturating_sub(1);
    assert!(cert.validate_structure().is_err());
}

#[test]
fn certificate_fingerprint_changes_when_identity_fields_change() {
    let cert_a = certificate();
    let mut cert_b = certificate();
    cert_b.serial = "serial-2".to_string();

    assert_ne!(cert_a.fingerprint(), cert_b.fingerprint());
}

#[test]
fn network_rejects_broadcast_without_authenticated_session() {
    let mut network = P2PNetwork::new(NetworkConfig::default());
    network.register_peer(peer()).expect("peer should register");

    let result = network.broadcast_secure(
        "node-1",
        ConsensusMessage::BlockProposal {
            block: sample_block(1),
        },
    );

    assert!(result.is_err(), "broadcast without session must fail");
}

#[test]
fn network_session_broadcast_produces_monotonic_nonce() {
    let mut network = P2PNetwork::new(NetworkConfig::default());
    network.register_peer(peer()).expect("peer should register");
    network
        .establish_session("node-1")
        .expect("session should establish");

    let envelope_a = network
        .broadcast_secure(
            "node-1",
            ConsensusMessage::BlockProposal {
                block: sample_block(1),
            },
        )
        .expect("first broadcast should succeed");

    let envelope_b = network
        .broadcast_secure(
            "node-1",
            ConsensusMessage::BlockProposal {
                block: sample_block(2),
            },
        )
        .expect("second broadcast should succeed");

    assert_eq!(envelope_a.session_id, envelope_b.session_id);
    assert!(envelope_b.nonce > envelope_a.nonce);
}

#[test]
fn finalization_uses_stake_weight_not_validator_headcount() {
    let validators = [
        ([11u8; 32], 80u64),
        ([12u8; 32], 10u64),
        ([13u8; 32], 10u64),
    ]
    .into_iter()
    .map(|(secret, power)| {
        let signer = SigningKey::from_bytes(&secret);
        Validator::new(
            signer.verifying_key().to_bytes(),
            power,
            ValidatorRole::Validator,
        )
    })
    .collect::<Vec<_>>();

    let low_power_a = validators[1].id;
    let low_power_b = validators[2].id;
    let high_power = validators[0].id;

    let rotation = ValidatorRotation::new(validators).expect("validator rotation should build");
    let mut consensus = ConsensusState::new(rotation, QuorumThreshold::two_thirds());

    let genesis = sample_block(1);
    consensus
        .admit_block(genesis.clone())
        .expect("genesis should be accepted");

    let candidate = Proposer::new(2626, high_power)
        .propose(
            genesis.hash,
            2,
            0,
            2,
            1_800_000_002,
            BlockBody {
                sections: vec![aoxcunity::BlockSection::LaneCommitment(
                    LaneCommitmentSection {
                        lanes: vec![LaneCommitment {
                            lane_id: 2,
                            lane_type: LaneType::Native,
                            tx_count: 1,
                            input_root: [2u8; 32],
                            output_root: [3u8; 32],
                            receipt_root: [4u8; 32],
                            state_commitment: [5u8; 32],
                            proof_commitment: [6u8; 32],
                        }],
                    },
                )],
            },
        )
        .expect("candidate block should build");
    consensus
        .admit_block(candidate.clone())
        .expect("candidate block should be accepted");

    for voter in [low_power_a, low_power_b] {
        consensus
            .add_vote(Vote {
                voter,
                block_hash: candidate.hash,
                height: candidate.header.height,
                round: candidate.header.round,
                kind: VoteKind::Commit,
            })
            .expect("low-power vote should be accepted");
    }

    assert_eq!(
        consensus.finalizable_round(candidate.hash),
        None,
        "two out of three validators are insufficient when stake weight is below threshold"
    );

    consensus
        .add_vote(Vote {
            voter: high_power,
            block_hash: candidate.hash,
            height: candidate.header.height,
            round: candidate.header.round,
            kind: VoteKind::Commit,
        })
        .expect("high-power vote should be accepted");

    assert_eq!(
        consensus.finalizable_round(candidate.hash),
        Some(candidate.header.round),
        "adding sufficient stake weight should make the round finalizable"
    );
}

#[test]
fn native_token_quantum_transfer_rejects_replay_and_nonce_regression() {
    let mut ledger = NativeTokenLedger::new_for_network(NativeTokenNetwork::Testnet)
        .expect("testnet native token ledger should initialize");

    let treasury = [0xAA; 32];
    let operator = [0xBB; 32];

    ledger
        .mint(treasury, 1_000)
        .expect("treasury mint should succeed");

    ledger
        .transfer_quantum(treasury, operator, 250, 1, b"proof-001")
        .expect("first quantum transfer should succeed");

    let replay_error = ledger
        .transfer_quantum(treasury, operator, 250, 1, b"proof-001")
        .expect_err("identical quantum transfer should be rejected as replay");
    assert_eq!(replay_error, NativeTokenError::ReplayDetected);

    let nonce_regression = ledger
        .transfer_quantum(treasury, operator, 100, 0, b"proof-002")
        .expect_err("lower nonce than latest accepted value must be rejected");
    assert_eq!(nonce_regression, NativeTokenError::NonceRegression);

    assert_eq!(ledger.balance_of(&treasury), 750);
    assert_eq!(ledger.balance_of(&operator), 250);
    assert_eq!(ledger.latest_nonce_of(&treasury), Some(1));
}

fn sample_block(seed: u8) -> aoxcunity::Block {
    Proposer::new(2626, [seed; 32])
        .propose(
            [0u8; 32],
            u64::from(seed),
            0,
            u64::from(seed),
            1_800_000_000 + u64::from(seed),
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
            },
        )
        .expect("block should build")
}

fn certificate() -> NodeCertificate {
    NodeCertificate {
        subject: "node-1".to_string(),
        issuer: "AOXC-ROOT".to_string(),
        valid_from_unix: 1,
        valid_until_unix: u64::MAX,
        serial: "serial-1".to_string(),
        domain_attestation_hash: "attestation-1".to_string(),
    }
}

fn peer() -> Peer {
    Peer::new(
        "node-1",
        "10.0.0.1:2727",
        "AOXC-MAINNET",
        aoxcnet::config::ExternalDomainKind::Native,
        PeerRole::Validator,
        3,
        true,
        certificate(),
    )
}
