// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use aoxcnet::{
    config::NetworkConfig,
    gossip::peer::{NodeCertificate, Peer, PeerRole},
    p2p::P2PNetwork,
};
use aoxcore::{
    block::{Capability, TargetOutpost},
    identity::{
        actor_id::{generate_actor_id, parse_actor_id, validate_actor_id},
        hd_path::{HdPath, MAX_HD_INDEX},
    },
    transaction::{
        MAX_TRANSACTION_PAYLOAD_BYTES, Transaction, TransactionError, hash_transaction,
        hash_transaction_intent,
    },
};
use aoxcunity::{
    BlockBody, ConsensusMessage, LaneCommitment, LaneCommitmentSection, LaneType, Proposer,
    QuorumThreshold,
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
fn quorum_threshold_handles_zero_total_power_safely() {
    let threshold = QuorumThreshold::two_thirds();
    assert!(!threshold.is_reached(10, 0));
}

#[test]
fn certificate_validity_is_inclusive_on_window_edges() {
    let cert = certificate();
    assert!(cert.is_valid_at(cert.valid_from_unix));
    assert!(cert.is_valid_at(cert.valid_until_unix));
}

#[test]
fn peer_identity_shape_rejects_missing_fields() {
    let mut p = peer();
    assert!(p.has_valid_identity_shape());

    p.id = "".to_string();
    assert!(!p.has_valid_identity_shape());

    p.id = "node-1".to_string();
    p.address = " ".to_string();
    assert!(!p.has_valid_identity_shape());
}

#[test]
fn receive_returns_none_when_inbound_queue_is_empty() {
    let mut network = P2PNetwork::new(NetworkConfig::default());
    assert!(network.receive().is_none());
}

#[test]
fn receive_returns_payload_after_valid_broadcast() {
    let mut network = P2PNetwork::new(NetworkConfig::default());
    network.register_peer(peer()).expect("peer should register");
    network
        .establish_session("node-1")
        .expect("session should establish");

    network
        .broadcast_secure(
            "node-1",
            ConsensusMessage::BlockProposal {
                block: sample_block(3),
            },
        )
        .expect("broadcast should succeed");

    let received = network.receive();
    assert!(received.is_some());
}

#[test]
fn proposer_output_hash_changes_for_distinct_inputs() {
    let block_a = sample_block(11);
    let block_b = sample_block(12);
    assert_ne!(block_a.hash, block_b.hash);
}

#[test]
fn transaction_payload_size_boundary_is_enforced() {
    let signer = SigningKey::from_bytes(&[9u8; 32]);
    let sender = signer.verifying_key().to_bytes();

    let max_ok = Transaction::new(
        sender,
        1,
        Capability::UserSigned,
        TargetOutpost::EthMainnetGateway,
        vec![0xAA; MAX_TRANSACTION_PAYLOAD_BYTES],
        [7u8; 64],
    );
    assert!(max_ok.is_ok());

    let max_fail = Transaction::new(
        sender,
        2,
        Capability::UserSigned,
        TargetOutpost::EthMainnetGateway,
        vec![0xAA; MAX_TRANSACTION_PAYLOAD_BYTES + 1],
        [8u8; 64],
    )
    .expect_err("oversized payload must fail");

    assert!(matches!(
        max_fail,
        TransactionError::PayloadTooLarge {
            size,
            max: MAX_TRANSACTION_PAYLOAD_BYTES,
        } if size == MAX_TRANSACTION_PAYLOAD_BYTES + 1
    ));
}

#[test]
fn transaction_intent_hash_is_signature_agnostic() {
    let signer = SigningKey::from_bytes(&[3u8; 32]);
    let sender = signer.verifying_key().to_bytes();

    let tx_a = Transaction::new(
        sender,
        41,
        Capability::UserSigned,
        TargetOutpost::EthMainnetGateway,
        b"intent-payload".to_vec(),
        [1u8; 64],
    )
    .expect("transaction should build");

    let tx_b = Transaction::new(
        sender,
        41,
        Capability::UserSigned,
        TargetOutpost::EthMainnetGateway,
        b"intent-payload".to_vec(),
        [2u8; 64],
    )
    .expect("transaction should build");

    assert_eq!(
        hash_transaction_intent(&tx_a),
        hash_transaction_intent(&tx_b)
    );
    assert_ne!(hash_transaction(&tx_a), hash_transaction(&tx_b));
}

#[test]
fn actor_id_generation_and_parsing_roundtrip() {
    let signer = SigningKey::from_bytes(&[1u8; 32]);
    let actor_id = generate_actor_id(&signer.verifying_key().to_bytes(), "validator", "eu")
        .expect("actor-id should generate");

    assert_eq!(validate_actor_id(&actor_id), Ok(()));
    let parsed = parse_actor_id(&actor_id).expect("actor-id should parse");
    assert_eq!(parsed.prefix, "AOXC");
}

#[test]
fn hd_path_roundtrip_and_overflow_rejection() {
    let path = HdPath::new(44, 2626, 1, MAX_HD_INDEX).expect("path should build");
    let serialized = path.to_string();
    let reparsed: HdPath = serialized.parse().expect("path should parse");
    assert_eq!(path, reparsed);

    let overflow = format!("m/44/2626/1/2/3/{}", MAX_HD_INDEX + 1);
    assert!(overflow.parse::<HdPath>().is_err());
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
