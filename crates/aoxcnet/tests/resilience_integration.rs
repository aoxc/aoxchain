use aoxcnet::config::{ExternalDomainKind, NetworkConfig};
use aoxcnet::gossip::peer::{NodeCertificate, Peer, PeerRole};
use aoxcnet::p2p::P2PNetwork;
use aoxcnet::resilience::{
    AdversarialCampaign, ChaosProfile, RemediationAction, ResilienceAssessment, ResilienceHarness,
    ScenarioDefinition,
};
use aoxcunity::messages::ConsensusMessage;
use aoxcunity::vote::{AuthenticatedVote, Vote, VoteAuthenticationContext, VoteKind};

fn peer(id: &str) -> Peer {
    let certificate = NodeCertificate {
        subject: id.to_string(),
        issuer: "AOXC-ROOT".to_string(),
        valid_from_unix: 1,
        valid_until_unix: u64::MAX,
        serial: format!("serial-{id}"),
        domain_attestation_hash: format!("attestation-{id}"),
    };

    Peer::new(
        id,
        "10.0.0.1:2727",
        "AOXC-MAINNET",
        ExternalDomainKind::Native,
        PeerRole::Validator,
        3,
        true,
        certificate,
    )
}

fn vote(height: u64, voter: u8) -> ConsensusMessage {
    ConsensusMessage::Vote(AuthenticatedVote {
        vote: Vote {
            voter: [voter; 32],
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
fn seeded_fuzz_matrix_preserves_repeatability_bounds_and_campaign_assessment() {
    let seeds = [1_u64, 2, 3, 17, 42, 99, 1234, 9_999];

    for seed in seeds {
        let mut network = P2PNetwork::new(NetworkConfig::default());
        for node in ["node-1", "node-2"] {
            network.register_peer(peer(node)).expect("peer registers");
            network
                .establish_session(node)
                .expect("session established");
        }

        let profile = ChaosProfile {
            seed,
            drop_every_nth: (seed as usize % 4) + 2,
            duplicate_every_nth: (seed as usize % 3) + 2,
            reorder_window: ((seed as usize % 3) + 2).min(4),
            max_jitter_ms: seed % 11,
            max_inflight_frames: 64,
            partition_start_frame: Some(((seed as usize % 5) + 3).min(8)),
            partition_frame_len: 1 + (seed as usize % 2),
        };

        let mut first = ResilienceHarness::new(profile.clone()).expect("valid profile");
        let mut second = ResilienceHarness::new(profile.clone()).expect("valid profile");
        let mut frames = Vec::new();

        for height in 1..=12 {
            let peer_id = if height % 2 == 0 { "node-2" } else { "node-1" };
            let envelope = network
                .broadcast_secure(peer_id, vote(height, height as u8))
                .expect("broadcast works");
            first.enqueue(envelope.clone()).expect("first enqueue");
            second.enqueue(envelope.clone()).expect("second enqueue");
            frames.push(envelope);
        }

        let left = first.flush();
        let right = second.flush();
        assert_eq!(left, right, "same seed must produce same report");
        assert!(left.peak_inflight_frames <= 64);
        assert_eq!(
            left.accepted_frames + left.dropped_frames + left.partition_dropped_frames,
            12
        );
        assert!(left.delivered.len() >= left.accepted_frames);

        let assessment = ResilienceAssessment::from_report(&left);
        assert!(assessment.delivery_rate >= 0.0);
        assert!(assessment.duplicate_rate >= 0.0);
        assert!(!assessment.peer_scorecards.is_empty());

        let campaign = AdversarialCampaign::new(vec![
            ScenarioDefinition {
                name: format!("seed-{seed}-baseline"),
                profile: ChaosProfile {
                    max_inflight_frames: 64,
                    ..ChaosProfile::default()
                },
            },
            ScenarioDefinition {
                name: format!("seed-{seed}-chaos"),
                profile,
            },
        ]);

        let campaign_report = campaign.execute(&frames).expect("campaign executes");
        assert_eq!(campaign_report.scenarios.len(), 2);
        assert!(campaign_report.aggregate_delivery_rate > 0.0);
        assert!(matches!(
            campaign_report.strongest_action,
            RemediationAction::Healthy
                | RemediationAction::Observe
                | RemediationAction::Quarantine
                | RemediationAction::Ban
        ));
    }
}
