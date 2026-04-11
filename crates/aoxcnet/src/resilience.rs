// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use std::collections::VecDeque;

use rand::rngs::StdRng;
use rand::{RngExt, SeedableRng};
use serde::{Deserialize, Serialize};

use crate::error::NetworkError;
use crate::p2p::ProtocolEnvelope;

/// Bounded, deterministic chaos profile for network resilience simulation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChaosProfile {
    /// Deterministic RNG seed used for repeatable simulation.
    pub seed: u64,
    /// Drop every Nth frame. Zero disables the rule.
    pub drop_every_nth: usize,
    /// Duplicate every Nth frame. Zero disables the rule.
    pub duplicate_every_nth: usize,
    /// Reorder window in number of frames. Zero disables reordering.
    pub reorder_window: usize,
    /// Artificial delay injected into delivered frames, in milliseconds.
    pub max_jitter_ms: u64,
    /// Maximum frames permitted to remain buffered before backpressure rejects.
    pub max_inflight_frames: usize,
}

impl Default for ChaosProfile {
    fn default() -> Self {
        Self {
            seed: 0xA0C2_2626,
            drop_every_nth: 0,
            duplicate_every_nth: 0,
            reorder_window: 0,
            max_jitter_ms: 0,
            max_inflight_frames: 256,
        }
    }
}

impl ChaosProfile {
    pub fn validate(&self) -> Result<(), NetworkError> {
        if self.max_inflight_frames == 0 {
            return Err(NetworkError::TransportUnavailable(
                "chaos profile max_inflight_frames must be non-zero".to_string(),
            ));
        }

        if self.reorder_window > self.max_inflight_frames {
            return Err(NetworkError::TransportUnavailable(
                "chaos profile reorder window exceeds inflight capacity".to_string(),
            ));
        }

        Ok(())
    }
}

/// Delivery event emitted by the resilience harness.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeliveryEvent {
    pub source_peer_id: String,
    pub session_id: String,
    pub nonce: u64,
    pub delivered_at_ms: u64,
    pub duplicated: bool,
    pub payload_hash_hex: String,
}

/// Summary of deterministic chaos simulation.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResilienceReport {
    pub accepted_frames: usize,
    pub dropped_frames: usize,
    pub duplicated_frames: usize,
    pub reordered_frames: usize,
    pub rejected_frames: usize,
    pub peak_inflight_frames: usize,
    pub total_simulated_delay_ms: u64,
    pub delivered: Vec<DeliveryEvent>,
}

/// Deterministic in-memory transport harness capable of simulating bounded
/// drop/duplicate/reorder/jitter conditions.
#[derive(Debug)]
pub struct ResilienceHarness {
    profile: ChaosProfile,
    buffered: VecDeque<ProtocolEnvelope>,
    report: ResilienceReport,
    emitted_frames: usize,
    logical_clock_ms: u64,
    rng: StdRng,
}

impl ResilienceHarness {
    pub fn new(profile: ChaosProfile) -> Result<Self, NetworkError> {
        profile.validate()?;
        let rng = StdRng::seed_from_u64(profile.seed);
        Ok(Self {
            profile,
            buffered: VecDeque::new(),
            report: ResilienceReport::default(),
            emitted_frames: 0,
            logical_clock_ms: 0,
            rng,
        })
    }

    pub fn enqueue(&mut self, envelope: ProtocolEnvelope) -> Result<(), NetworkError> {
        self.emitted_frames = self.emitted_frames.saturating_add(1);

        if self.profile.drop_every_nth != 0
            && self
                .emitted_frames
                .is_multiple_of(self.profile.drop_every_nth)
        {
            self.report.dropped_frames = self.report.dropped_frames.saturating_add(1);
            return Ok(());
        }

        if self.buffered.len() >= self.profile.max_inflight_frames {
            self.report.rejected_frames = self.report.rejected_frames.saturating_add(1);
            return Err(NetworkError::TransportUnavailable(
                "deterministic transport backpressure exceeded inflight capacity".to_string(),
            ));
        }

        self.buffered.push_back(envelope.clone());
        self.report.accepted_frames = self.report.accepted_frames.saturating_add(1);
        self.report.peak_inflight_frames =
            self.report.peak_inflight_frames.max(self.buffered.len());

        if self.profile.duplicate_every_nth != 0
            && self
                .emitted_frames
                .is_multiple_of(self.profile.duplicate_every_nth)
        {
            if self.buffered.len() >= self.profile.max_inflight_frames {
                self.report.rejected_frames = self.report.rejected_frames.saturating_add(1);
                return Err(NetworkError::TransportUnavailable(
                    "deterministic transport duplicate exceeded inflight capacity".to_string(),
                ));
            }

            self.buffered.push_back(envelope);
            self.report.duplicated_frames = self.report.duplicated_frames.saturating_add(1);
            self.report.peak_inflight_frames =
                self.report.peak_inflight_frames.max(self.buffered.len());
        }

        Ok(())
    }

    #[must_use]
    pub fn flush(&mut self) -> ResilienceReport {
        let mut out = Vec::with_capacity(self.buffered.len());
        while !self.buffered.is_empty() {
            let next = if self.profile.reorder_window > 1
                && self.buffered.len() >= self.profile.reorder_window
            {
                self.report.reordered_frames = self.report.reordered_frames.saturating_add(1);
                let idx = self.rng.random_range(0..self.profile.reorder_window);
                self.buffered.remove(idx).expect("indexed buffered frame")
            } else {
                self.buffered.pop_front().expect("front buffered frame")
            };

            let jitter = if self.profile.max_jitter_ms == 0 {
                0
            } else {
                self.rng.random_range(0..=self.profile.max_jitter_ms)
            };
            self.logical_clock_ms = self.logical_clock_ms.saturating_add(jitter);
            self.report.total_simulated_delay_ms =
                self.report.total_simulated_delay_ms.saturating_add(jitter);

            let duplicated = out.iter().any(|event: &DeliveryEvent| {
                event.session_id == next.session_id && event.nonce == next.nonce
            });

            out.push(DeliveryEvent {
                source_peer_id: next.peer_id.clone(),
                session_id: next.session_id.clone(),
                nonce: next.nonce,
                delivered_at_ms: self.logical_clock_ms,
                duplicated,
                payload_hash_hex: next.payload_hash_hex.clone(),
            });
        }

        self.report.delivered = out;
        self.report.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::{ChaosProfile, ResilienceHarness};
    use crate::config::NetworkConfig;
    use crate::gossip::peer::{NodeCertificate, Peer, PeerRole};
    use crate::p2p::P2PNetwork;
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
            crate::config::ExternalDomainKind::Native,
            PeerRole::Validator,
            3,
            true,
            certificate,
        )
    }

    fn vote(height: u64) -> ConsensusMessage {
        ConsensusMessage::Vote(AuthenticatedVote {
            vote: Vote {
                voter: [1u8; 32],
                block_hash: [height as u8; 32],
                height,
                round: 0,
                kind: VoteKind::Prepare,
            },
            context: VoteAuthenticationContext {
                network_id: 2626,
                epoch: 0,
                validator_set_root: [3u8; 32],
                pq_attestation_root: [5u8; 32],
                signature_scheme: 2,
            },
            signature: vec![4u8; 64],
            pq_public_key: Some(vec![7u8; 32]),
            pq_signature: Some(vec![8u8; 32]),
        })
    }

    #[test]
    fn resilience_harness_applies_drop_duplicate_and_reorder_rules() {
        let mut network = P2PNetwork::new(NetworkConfig::default());
        network.register_peer(peer()).expect("peer registers");
        network
            .establish_session("node-1")
            .expect("session established");

        let mut harness = ResilienceHarness::new(ChaosProfile {
            seed: 7,
            drop_every_nth: 3,
            duplicate_every_nth: 2,
            reorder_window: 2,
            max_jitter_ms: 5,
            max_inflight_frames: 16,
        })
        .expect("valid profile");

        for height in 1..=5 {
            let envelope = network
                .broadcast_secure("node-1", vote(height))
                .expect("broadcast works");
            harness.enqueue(envelope).expect("buffer accepts frame");
        }

        let report = harness.flush();
        assert_eq!(report.accepted_frames, 4);
        assert_eq!(report.dropped_frames, 1);
        assert_eq!(report.duplicated_frames, 2);
        assert!(report.reordered_frames >= 1);
        assert_eq!(report.delivered.len(), 6);
        assert!(report.delivered.iter().any(|event| event.duplicated));
    }

    #[test]
    fn resilience_harness_rejects_unbounded_backpressure() {
        let mut network = P2PNetwork::new(NetworkConfig::default());
        network.register_peer(peer()).expect("peer registers");
        network
            .establish_session("node-1")
            .expect("session established");

        let mut harness = ResilienceHarness::new(ChaosProfile {
            max_inflight_frames: 1,
            ..ChaosProfile::default()
        })
        .expect("valid profile");

        let first = network
            .broadcast_secure("node-1", vote(1))
            .expect("broadcast works");
        harness.enqueue(first).expect("first frame fits");

        let second = network
            .broadcast_secure("node-1", vote(2))
            .expect("broadcast works");
        let error = harness
            .enqueue(second)
            .expect_err("second frame should exceed inflight capacity");
        assert_eq!(error.code(), "AOXCNET_TRANSPORT_UNAVAILABLE");
    }

    #[test]
    fn resilience_harness_is_deterministic_for_same_seed() {
        let mut network = P2PNetwork::new(NetworkConfig::default());
        network.register_peer(peer()).expect("peer registers");
        network
            .establish_session("node-1")
            .expect("session established");

        let profile = ChaosProfile {
            seed: 99,
            drop_every_nth: 0,
            duplicate_every_nth: 2,
            reorder_window: 3,
            max_jitter_ms: 10,
            max_inflight_frames: 32,
        };

        let mut left = ResilienceHarness::new(profile.clone()).expect("valid profile");
        let mut right = ResilienceHarness::new(profile).expect("valid profile");

        for height in 1..=6 {
            let envelope = network
                .broadcast_secure("node-1", vote(height))
                .expect("broadcast works");
            left.enqueue(envelope.clone()).expect("left enqueue");
            right.enqueue(envelope).expect("right enqueue");
        }

        assert_eq!(left.flush(), right.flush());
    }
}
