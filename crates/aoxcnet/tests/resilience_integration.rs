// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use aoxcnet::config::{ExternalDomainKind, NetworkConfig};
use aoxcnet::gossip::peer::{NodeCertificate, Peer, PeerRole};
use aoxcnet::p2p::P2PNetwork;
use aoxcnet::resilience::{ChaosProfile, ResilienceHarness};
use aoxcunity::messages::ConsensusMessage;
use aoxcunity::vote::{AuthenticatedVote, Vote, VoteAuthenticationContext, VoteKind};

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
        ExternalDomainKind::Native,
        PeerRole::Validator,
        3,
        true,
        certificate,
    )
}

fn vote(height: u64) -> ConsensusMessage {
    ConsensusMessage::Vote(AuthenticatedVote {
        vote: Vote {
            voter: [7u8; 32],
            block_hash: [height as u8; 32],
            height,
            round: 1,
            kind: VoteKind::Commit,
        },
        context: VoteAuthenticationContext {
            network_id: 2626,
            epoch: 1,
            validator_set_root: [9u8; 32],
            signature_scheme: 1,
        },
        signature: vec![1u8; 96],
    })
}

#[test]
fn seeded_fuzz_matrix_preserves_repeatability_and_bounds() {
    let seeds = [1_u64, 2, 3, 17, 42, 99, 1234, 9_999];

    for seed in seeds {
        let mut network = P2PNetwork::new(NetworkConfig::default());
        network.register_peer(peer()).expect("peer registers");
        network
            .establish_session("node-1")
            .expect("session established");

        let profile = ChaosProfile {
            seed,
            drop_every_nth: (seed as usize % 4) + 2,
            duplicate_every_nth: (seed as usize % 3) + 2,
            reorder_window: ((seed as usize % 3) + 2).min(4),
            max_jitter_ms: seed % 11,
            max_inflight_frames: 64,
        };

        let mut first = ResilienceHarness::new(profile.clone()).expect("valid profile");
        let mut second = ResilienceHarness::new(profile).expect("valid profile");

        for height in 1..=12 {
            let envelope = network
                .broadcast_secure("node-1", vote(height))
                .expect("broadcast works");
            first.enqueue(envelope.clone()).expect("first enqueue");
            second.enqueue(envelope).expect("second enqueue");
        }

        let left = first.flush();
        let right = second.flush();
        assert_eq!(left, right, "same seed must produce same report");
        assert!(left.peak_inflight_frames <= 64);
        assert_eq!(left.accepted_frames + left.dropped_frames, 12);
        assert!(left.delivered.len() >= left.accepted_frames);
    }
}
