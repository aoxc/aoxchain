// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use aoxcnet::p2p::{ProtocolEnvelope, SessionTicket};
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
fn protocol_envelope_constructor_rejects_invalid_session_and_domain_inputs() {
    let payload = ConsensusMessage::BlockProposal {
        block: sample_block(9),
    };

    let ticket = SessionTicket {
        peer_id: "node-9".to_string(),
        cert_fingerprint: "fp-9".to_string(),
        established_at_unix: 1_800_000_000,
        replay_window_nonce: 1,
        session_id: "session-9".to_string(),
        expires_at_unix: 1_800_000_100,
    };

    assert!(
        ProtocolEnvelope::new("", 2626, &ticket, payload.clone(), 1_800_000_010).is_err(),
        "empty chain identifier must be rejected"
    );
    assert!(
        ProtocolEnvelope::new("AOXC-MAINNET", 0, &ticket, payload.clone(), 1_800_000_010)
            .is_err(),
        "zero protocol serial must be rejected"
    );
    assert!(
        ProtocolEnvelope::new("AOXC-MAINNET", 2626, &ticket, payload, 1_800_000_101).is_err(),
        "issued timestamp after ticket expiry must be rejected"
    );
}

#[test]
fn protocol_envelope_fuzz_mutations_fail_closed_under_integrity_checks() {
    let mut rng = StdRng::seed_from_u64(0xA0C2_2026_0002);
    let canonical_chain_id = "AOXC-MAINNET";
    let canonical_protocol_serial = 2626_u64;

    for idx in 0..1_000_u64 {
        let ticket = SessionTicket {
            peer_id: format!("node-{idx}"),
            cert_fingerprint: format!("fp-{idx}"),
            established_at_unix: 1_800_000_000 + idx,
            replay_window_nonce: idx,
            session_id: format!("session-{idx}"),
            expires_at_unix: 1_900_000_000 + idx,
        };

        let mut envelope = ProtocolEnvelope::new(
            canonical_chain_id,
            canonical_protocol_serial,
            &ticket,
            ConsensusMessage::BlockProposal {
                block: sample_block(((idx % 251) as u8).saturating_add(1)),
            },
            ticket.established_at_unix + 1,
        )
        .expect("baseline envelope fixture must build");
        assert!(
            envelope
                .verify_against(canonical_chain_id, canonical_protocol_serial)
                .is_ok()
        );

        match rng.next_u32() % 6 {
            0 => envelope.protocol_version = envelope.protocol_version.saturating_add(1),
            1 => envelope.chain_id.push_str("-mut"),
            2 => envelope.protocol_serial = envelope.protocol_serial.saturating_add(1),
            3 => envelope.payload_hash_hex = "aa".repeat(32),
            4 => envelope.frame_hash_hex = "bb".repeat(32),
            _ => envelope.issued_at_unix = envelope.expires_at_unix.saturating_add(1),
        }

        assert!(
            envelope
                .verify_against(canonical_chain_id, canonical_protocol_serial)
                .is_err(),
            "mutated envelope should fail canonical verification"
        );
    }
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
