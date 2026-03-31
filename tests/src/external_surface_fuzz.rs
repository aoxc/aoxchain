// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use aoxcnet::{
    config::NetworkConfig,
    gossip::peer::{NodeCertificate, Peer, PeerRole},
    p2p::{P2PNetwork, ProtocolEnvelope, SessionTicket},
};
use aoxcore::{
    block::{Capability, TargetOutpost},
    transaction::{MAX_TRANSACTION_PAYLOAD_BYTES, Transaction},
};
use aoxcunity::{
    BlockBody, ConsensusMessage, LaneCommitment, LaneCommitmentSection, LaneType, Proposer,
};
use ed25519_dalek::{SigningKey, VerifyingKey};
use rand::{Rng, SeedableRng, rngs::StdRng};

#[test]
fn external_transaction_surface_fuzz_corpus_enforces_fail_closed_rules() {
    let mut rng = StdRng::seed_from_u64(0xA0C2_2026_0001);

    for _ in 0..2_500 {
        let mut sender = [0u8; 32];
        rng.fill_bytes(&mut sender);

        let nonce = rng.next_u64();

        let mut signature = [0u8; 64];
        rng.fill_bytes(&mut signature);

        let payload_len = (rng.next_u32() as usize) % (MAX_TRANSACTION_PAYLOAD_BYTES + 9);
        let payload = vec![0xAB; payload_len];

        let sender_valid = sender != [0u8; 32] && VerifyingKey::from_bytes(&sender).is_ok();
        let signature_valid = signature != [0u8; 64];
        let payload_valid = (1..=MAX_TRANSACTION_PAYLOAD_BYTES).contains(&payload_len);

        let result = Transaction::new(
            sender,
            nonce,
            Capability::UserSigned,
            TargetOutpost::EthMainnetGateway,
            payload,
            signature,
        );

        if sender_valid && signature_valid && payload_valid {
            assert!(
                result.is_ok(),
                "transaction should pass when sender/signature/payload satisfy structural policy"
            );
        } else {
            assert!(
                result.is_err(),
                "transaction should fail when any structural policy guard is violated"
            );
        }
    }

    let signer = SigningKey::from_bytes(&[31u8; 32]);
    let tx = Transaction::new(
        signer.verifying_key().to_bytes(),
        1,
        Capability::UserSigned,
        TargetOutpost::EthMainnetGateway,
        vec![0xAA],
        [9u8; 64],
    )
    .expect("known-valid fixture must pass transaction validation");
    assert_eq!(tx.payload_len(), 1);
}

#[test]
fn external_protocol_envelope_tamper_corpus_is_detected() {
    let ticket = SessionTicket {
        peer_id: "node-1".to_string(),
        cert_fingerprint: "fp-1".to_string(),
        established_at_unix: 1_800_000_000,
        replay_window_nonce: 9,
        session_id: "session-1".to_string(),
        expires_at_unix: 1_900_000_000,
    };

    let payload = ConsensusMessage::BlockProposal {
        block: sample_block(1),
    };

    let envelope = ProtocolEnvelope::new("AOXC-MAINNET", 2626, &ticket, payload, 1_800_000_010)
        .expect("fixture envelope must build");
    assert!(envelope.verify_against("AOXC-MAINNET", 2626).is_ok());

    let mut mismatched_chain = envelope.clone();
    mismatched_chain.chain_id = "AOXC-TESTNET".to_string();
    assert!(
        mismatched_chain
            .verify_against("AOXC-MAINNET", 2626)
            .is_err()
    );

    let mut mismatched_serial = envelope.clone();
    mismatched_serial.protocol_serial = 9999;
    assert!(
        mismatched_serial
            .verify_against("AOXC-MAINNET", 2626)
            .is_err()
    );

    let mut tampered_payload_hash = envelope.clone();
    tampered_payload_hash.payload_hash_hex = "00".repeat(32);
    assert!(
        tampered_payload_hash
            .verify_against("AOXC-MAINNET", 2626)
            .is_err()
    );

    let mut tampered_frame_hash = envelope.clone();
    tampered_frame_hash.frame_hash_hex = "11".repeat(32);
    assert!(
        tampered_frame_hash
            .verify_against("AOXC-MAINNET", 2626)
            .is_err()
    );

    let mut invalid_window = envelope;
    invalid_window.issued_at_unix = invalid_window.expires_at_unix.saturating_add(1);
    assert!(invalid_window.verify_against("AOXC-MAINNET", 2626).is_err());
}

#[test]
fn external_transaction_boundary_matrix_rejects_malformed_edges() {
    let signer = SigningKey::from_bytes(&[21u8; 32]);
    let sender = signer.verifying_key().to_bytes();

    let valid = Transaction::new(
        sender,
        7,
        Capability::UserSigned,
        TargetOutpost::EthMainnetGateway,
        vec![0x01; MAX_TRANSACTION_PAYLOAD_BYTES],
        [1u8; 64],
    );
    assert!(
        valid.is_ok(),
        "max-size payload boundary must remain accepted"
    );

    let oversized = Transaction::new(
        sender,
        8,
        Capability::UserSigned,
        TargetOutpost::EthMainnetGateway,
        vec![0x01; MAX_TRANSACTION_PAYLOAD_BYTES + 1],
        [1u8; 64],
    );
    assert!(oversized.is_err(), "oversized payload must fail closed");

    let zero_sender = Transaction::new(
        [0u8; 32],
        9,
        Capability::UserSigned,
        TargetOutpost::EthMainnetGateway,
        vec![0x01],
        [1u8; 64],
    );
    assert!(zero_sender.is_err(), "zero sender must fail closed");

    let zero_signature = Transaction::new(
        sender,
        10,
        Capability::UserSigned,
        TargetOutpost::EthMainnetGateway,
        vec![0x01],
        [0u8; 64],
    );
    assert!(zero_signature.is_err(), "zero signature must fail closed");

    let empty_payload = Transaction::new(
        sender,
        11,
        Capability::UserSigned,
        TargetOutpost::EthMainnetGateway,
        vec![],
        [1u8; 64],
    );
    assert!(empty_payload.is_err(), "empty payload must fail closed");
}

#[test]
fn external_protocol_envelope_randomized_tamper_matrix_detects_corruption() {
    let mut rng = StdRng::seed_from_u64(0xA0C2_2026_0002);
    let ticket = SessionTicket {
        peer_id: "node-rand".to_string(),
        cert_fingerprint: "fp-rand".to_string(),
        established_at_unix: 1_800_000_000,
        replay_window_nonce: 50,
        session_id: "session-rand".to_string(),
        expires_at_unix: 1_900_000_000,
    };

    for idx in 0..512u16 {
        let payload = ConsensusMessage::BlockProposal {
            block: sample_block(((idx % 251) as u8) + 1),
        };
        let mut envelope =
            ProtocolEnvelope::new("AOXC-MAINNET", 2626, &ticket, payload, 1_800_000_100)
                .expect("baseline envelope should build");

        match rng.next_u32() % 6 {
            0 => envelope.protocol_version = envelope.protocol_version.saturating_add(1),
            1 => envelope.chain_id = "AOXC-DEVNET".to_string(),
            2 => envelope.protocol_serial = 7777,
            3 => envelope.payload_hash_hex = "ff".repeat(32),
            4 => envelope.frame_hash_hex = "aa".repeat(32),
            _ => envelope.issued_at_unix = envelope.expires_at_unix.saturating_add(100),
        }

        assert!(
            envelope.verify_against("AOXC-MAINNET", 2626).is_err(),
            "tampered envelope variant #{idx} must be rejected"
        );
    }
}

#[test]
fn external_network_peer_and_session_surface_rejects_adversarial_paths() {
    let mut network = P2PNetwork::new(NetworkConfig::default());
    let peer = valid_peer("node-1", "AOXC-MAINNET");

    network
        .register_peer(peer.clone())
        .expect("valid baseline peer should register");

    assert!(
        network.register_peer(peer.clone()).is_err(),
        "duplicate peer registration must be rejected"
    );

    assert!(
        network.establish_session("unknown-peer").is_err(),
        "unknown peer session establishment must fail"
    );

    network
        .establish_session("node-1")
        .expect("known peer session should establish");

    let no_session_broadcast = network.broadcast_secure(
        "node-2",
        ConsensusMessage::BlockProposal {
            block: sample_block(3),
        },
    );
    assert!(
        no_session_broadcast.is_err(),
        "broadcast without authenticated session must fail"
    );

    network.ban_peer("node-1");
    let banned_broadcast = network.broadcast_secure(
        "node-1",
        ConsensusMessage::BlockProposal {
            block: sample_block(4),
        },
    );
    assert!(banned_broadcast.is_err(), "banned peer must be denied");

    let mut invalid_certificate_peer = valid_peer("node-3", "AOXC-MAINNET");
    invalid_certificate_peer.certificate.valid_until_unix = 0;
    assert!(
        network.register_peer(invalid_certificate_peer).is_err(),
        "peer with malformed certificate validity window must fail admission"
    );
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
        .expect("block fixture should build")
}

fn valid_peer(id: &str, chain_id: &str) -> Peer {
    Peer::new(
        id,
        "10.10.0.1:2727",
        chain_id,
        aoxcnet::config::ExternalDomainKind::Native,
        PeerRole::Validator,
        3,
        true,
        NodeCertificate {
            subject: id.to_string(),
            issuer: "AOXC-ROOT".to_string(),
            valid_from_unix: 1,
            valid_until_unix: u64::MAX,
            serial: format!("serial-{id}"),
            domain_attestation_hash: "attestation-1".to_string(),
        },
    )
}
