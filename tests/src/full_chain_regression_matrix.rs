// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use aoxcnet::{
    config::NetworkConfig,
    gossip::peer::{NodeCertificate, Peer, PeerRole},
    p2p::P2PNetwork,
};
use aoxcunity::{
    BlockBody, ConsensusMessage, LaneCommitment, LaneCommitmentSection, LaneType, Proposer,
    QuorumThreshold,
};

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
