// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use aoxcnet::config::{ExternalDomainKind, NetworkConfig};
use aoxcnet::gossip::peer::{NodeCertificate, Peer, PeerRole};
use aoxcnet::p2p::P2PNetwork;
use aoxcnet::resilience::{ChaosProfile, ResilienceHarness};
use aoxcunity::messages::ConsensusMessage;
use aoxcunity::vote::{AuthenticatedVote, Vote, VoteAuthenticationContext, VoteKind};

const TEST_NODE_ID: &str = "node-1";
const TEST_NODE_ADDRESS: &str = "10.0.0.1:2727";
const TEST_NETWORK_NAME: &str = "AOXC-MAINNET";
const TEST_ROOT_ISSUER: &str = "AOXC-ROOT";
const TEST_SERIAL: &str = "serial-1";
const TEST_DOMAIN_ATTESTATION_HASH: &str = "attestation-1";

const TEST_NETWORK_ID: u32 = 2626;
const TEST_EPOCH: u64 = 1;
const TEST_SIGNATURE_SCHEME: u16 = 1;
const TEST_MAX_INFLIGHT_FRAMES: usize = 64;
const TEST_MESSAGE_COUNT: u64 = 12;

const TEST_VOTER: [u8; 32] = [7u8; 32];
const TEST_VALIDATOR_SET_ROOT: [u8; 32] = [9u8; 32];
const TEST_PQ_ATTESTATION_ROOT: [u8; 32] = [11u8; 32];
const TEST_SIGNATURE: [u8; 96] = [1u8; 96];

fn make_peer() -> Peer {
    let certificate = NodeCertificate {
        subject: TEST_NODE_ID.to_string(),
        issuer: TEST_ROOT_ISSUER.to_string(),
        valid_from_unix: 1,
        valid_until_unix: u64::MAX,
        serial: TEST_SERIAL.to_string(),
        domain_attestation_hash: TEST_DOMAIN_ATTESTATION_HASH.to_string(),
    };

    Peer::new(
        TEST_NODE_ID,
        TEST_NODE_ADDRESS,
        TEST_NETWORK_NAME,
        ExternalDomainKind::Native,
        PeerRole::Validator,
        3,
        true,
        certificate,
    )
}

fn make_vote_authentication_context() -> VoteAuthenticationContext {
    VoteAuthenticationContext {
        network_id: TEST_NETWORK_ID,
        epoch: TEST_EPOCH,
        validator_set_root: TEST_VALIDATOR_SET_ROOT,
        pq_attestation_root: TEST_PQ_ATTESTATION_ROOT,
        signature_scheme: TEST_SIGNATURE_SCHEME,
    }
}

fn make_vote(height: u64) -> ConsensusMessage {
    ConsensusMessage::Vote(AuthenticatedVote {
        vote: Vote {
            voter: TEST_VOTER,
            block_hash: [height as u8; 32],
            height,
            round: 1,
            kind: VoteKind::Commit,
        },
        context: make_vote_authentication_context(),
        signature: TEST_SIGNATURE.to_vec(),
    })
}

fn build_network() -> P2PNetwork {
    let mut network = P2PNetwork::new(NetworkConfig::default());

    network
        .register_peer(make_peer())
        .expect("test invariant violated: peer registration must succeed");

    network
        .establish_session(TEST_NODE_ID)
        .expect("test invariant violated: session establishment must succeed");

    network
}

#[test]
fn seeded_fuzz_matrix_preserves_repeatability_and_bounds() {
    let seeds = [1_u64, 2, 3, 17, 42, 99, 1234, 9_999];

    for seed in seeds {
        let mut network = build_network();
        let profile = ChaosProfile {
            seed,
            drop_every_nth: (seed as usize % 4) + 2,
            duplicate_every_nth: (seed as usize % 3) + 2,
            reorder_window: ((seed as usize % 3) + 2).min(4),
            max_jitter_ms: seed % 11,
            max_inflight_frames: TEST_MAX_INFLIGHT_FRAMES,
        };

        let mut first_harness =
            ResilienceHarness::new(profile.clone()).expect("valid profile must initialize");
        let mut second_harness =
            ResilienceHarness::new(profile).expect("valid profile must initialize");

        for height in 1..=TEST_MESSAGE_COUNT {
            let envelope = network
                .broadcast_secure(TEST_NODE_ID, make_vote(height))
                .expect("secure broadcast must succeed");

            first_harness
                .enqueue(envelope.clone())
                .expect("first harness must accept envelope");

            second_harness
                .enqueue(envelope)
                .expect("second harness must accept envelope");
        }

        let first_report = first_harness.flush();
        let second_report = second_harness.flush();

        assert_eq!(
            first_report, second_report,
            "seeded execution must remain deterministic for identical profile and input"
        );
        assert!(first_report.peak_inflight_frames <= TEST_MAX_INFLIGHT_FRAMES);
        assert_eq!(
            first_report.accepted_frames + first_report.dropped_frames,
            TEST_MESSAGE_COUNT as usize
        );
        assert!(first_report.delivered.len() >= first_report.accepted_frames);
    }
}
