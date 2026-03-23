use std::collections::{BTreeMap, BTreeSet, VecDeque};

use rand::rngs::StdRng;
use rand::{RngExt, SeedableRng};
use serde::{Deserialize, Serialize};

use crate::error::NetworkError;
use crate::p2p::{P2PNetwork, ProtocolEnvelope};

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
    /// Optional contiguous partition window start, expressed as the 1-indexed
    /// emitted frame ordinal. `None` disables the rule.
    pub partition_start_frame: Option<usize>,
    /// Number of sequential emitted frames dropped while the partition window
    /// remains active.
    pub partition_frame_len: usize,
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
            partition_start_frame: None,
            partition_frame_len: 0,
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

        if self.partition_start_frame.is_some() && self.partition_frame_len == 0 {
            return Err(NetworkError::TransportUnavailable(
                "partition_frame_len must be non-zero when partition_start_frame is configured"
                    .to_string(),
            ));
        }

        Ok(())
    }

    #[must_use]
    fn applies_partition_drop(&self, emitted_frame: usize) -> bool {
        let Some(start) = self.partition_start_frame else {
            return false;
        };
        let end = start.saturating_add(self.partition_frame_len);
        (start..end).contains(&emitted_frame)
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
    pub partition_dropped_frames: usize,
    pub peak_inflight_frames: usize,
    pub total_simulated_delay_ms: u64,
    pub delivered: Vec<DeliveryEvent>,
}

impl ResilienceReport {
    #[must_use]
    pub fn delivery_rate(&self) -> f64 {
        let attempted = self.accepted_frames + self.dropped_frames + self.rejected_frames;
        if attempted == 0 {
            return 1.0;
        }
        self.delivered.len() as f64 / attempted as f64
    }

    #[must_use]
    pub fn duplicate_rate(&self) -> f64 {
        if self.delivered.is_empty() {
            return 0.0;
        }
        self.delivered
            .iter()
            .filter(|event| event.duplicated)
            .count() as f64
            / self.delivered.len() as f64
    }

    #[must_use]
    pub fn average_jitter_ms(&self) -> u64 {
        if self.delivered.is_empty() {
            return 0;
        }
        self.total_simulated_delay_ms / self.delivered.len() as u64
    }

    #[must_use]
    pub fn max_consecutive_nonce_gap(&self) -> u64 {
        let mut grouped = BTreeMap::<&str, Vec<u64>>::new();
        for event in &self.delivered {
            grouped
                .entry(event.session_id.as_str())
                .or_default()
                .push(event.nonce);
        }

        grouped
            .values_mut()
            .map(|nonces| {
                nonces.sort_unstable();
                nonces
                    .windows(2)
                    .map(|window| window[1].saturating_sub(window[0]))
                    .max()
                    .unwrap_or(0)
            })
            .max()
            .unwrap_or(0)
    }
}

/// Deterministic peer health classification derived from resilience evidence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RemediationAction {
    Healthy,
    Observe,
    Quarantine,
    Ban,
}

/// Stable peer-specific resilience scorecard.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PeerScoreCard {
    pub peer_id: String,
    pub delivered_frames: usize,
    pub dropped_frames: usize,
    pub duplicate_frames: usize,
    pub rejection_frames: usize,
    pub score: u32,
    pub action: RemediationAction,
}

/// Roll-up assessment suitable for audit notes and release gating.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResilienceAssessment {
    pub delivery_rate: f64,
    pub duplicate_rate: f64,
    pub average_jitter_ms: u64,
    pub max_consecutive_nonce_gap: u64,
    pub peer_scorecards: Vec<PeerScoreCard>,
    pub recommended_action: RemediationAction,
}

impl ResilienceAssessment {
    #[must_use]
    pub fn from_report(report: &ResilienceReport) -> Self {
        let mut delivered_by_peer = BTreeMap::<String, usize>::new();
        let mut duplicates_by_peer = BTreeMap::<String, usize>::new();

        for event in &report.delivered {
            *delivered_by_peer
                .entry(event.source_peer_id.clone())
                .or_default() += 1;
            if event.duplicated {
                *duplicates_by_peer
                    .entry(event.source_peer_id.clone())
                    .or_default() += 1;
            }
        }

        let peers: BTreeSet<String> = delivered_by_peer
            .keys()
            .chain(duplicates_by_peer.keys())
            .cloned()
            .collect();

        let mut peer_scorecards = Vec::with_capacity(peers.len());
        for peer_id in peers {
            let delivered_frames = *delivered_by_peer.get(&peer_id).unwrap_or(&0);
            let duplicate_frames = *duplicates_by_peer.get(&peer_id).unwrap_or(&0);
            let dropped_frames = report.dropped_frames + report.partition_dropped_frames;
            let rejection_frames = report.rejected_frames;

            let mut score = 100u32;
            score = score.saturating_sub((duplicate_frames as u32).saturating_mul(12));
            score = score.saturating_sub((dropped_frames as u32).saturating_mul(8));
            score = score.saturating_sub((rejection_frames as u32).saturating_mul(15));
            score = score.saturating_sub(report.average_jitter_ms().min(30) as u32);

            let action = if score >= 85 {
                RemediationAction::Healthy
            } else if score >= 65 {
                RemediationAction::Observe
            } else if score >= 40 {
                RemediationAction::Quarantine
            } else {
                RemediationAction::Ban
            };

            peer_scorecards.push(PeerScoreCard {
                peer_id,
                delivered_frames,
                dropped_frames,
                duplicate_frames,
                rejection_frames,
                score,
                action,
            });
        }

        let recommended_action = peer_scorecards
            .iter()
            .map(|scorecard| scorecard.action)
            .max_by_key(|action| remediation_rank(*action))
            .unwrap_or(RemediationAction::Healthy);

        Self {
            delivery_rate: report.delivery_rate(),
            duplicate_rate: report.duplicate_rate(),
            average_jitter_ms: report.average_jitter_ms(),
            max_consecutive_nonce_gap: report.max_consecutive_nonce_gap(),
            peer_scorecards,
            recommended_action,
        }
    }

    pub fn enforce(&self, network: &mut P2PNetwork) {
        for scorecard in &self.peer_scorecards {
            match scorecard.action {
                RemediationAction::Healthy | RemediationAction::Observe => {}
                RemediationAction::Quarantine | RemediationAction::Ban => {
                    network.ban_peer(&scorecard.peer_id);
                }
            }
        }
    }
}

/// Named resilience scenario used by deterministic campaign execution.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScenarioDefinition {
    pub name: String,
    pub profile: ChaosProfile,
}

/// Aggregate evidence from executing multiple deterministic resilience scenarios.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CampaignReport {
    pub scenarios: Vec<ScenarioOutcome>,
    pub aggregate_delivery_rate: f64,
    pub aggregate_duplicate_rate: f64,
    pub strongest_action: RemediationAction,
}

/// Individual scenario outcome with assessment evidence.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScenarioOutcome {
    pub name: String,
    pub report: ResilienceReport,
    pub assessment: ResilienceAssessment,
}

/// Executes repeatable chaos campaigns over a static frame set.
#[derive(Debug, Clone)]
pub struct AdversarialCampaign {
    scenarios: Vec<ScenarioDefinition>,
}

impl AdversarialCampaign {
    #[must_use]
    pub fn new(scenarios: Vec<ScenarioDefinition>) -> Self {
        Self { scenarios }
    }

    pub fn execute(&self, frames: &[ProtocolEnvelope]) -> Result<CampaignReport, NetworkError> {
        let mut outcomes = Vec::with_capacity(self.scenarios.len());

        for scenario in &self.scenarios {
            let mut harness = ResilienceHarness::new(scenario.profile.clone())?;
            for frame in frames {
                harness.enqueue(frame.clone())?;
            }

            let report = harness.flush();
            let assessment = ResilienceAssessment::from_report(&report);
            outcomes.push(ScenarioOutcome {
                name: scenario.name.clone(),
                report,
                assessment,
            });
        }

        let scenario_count = outcomes.len().max(1) as f64;
        let aggregate_delivery_rate = outcomes
            .iter()
            .map(|outcome| outcome.report.delivery_rate())
            .sum::<f64>()
            / scenario_count;
        let aggregate_duplicate_rate = outcomes
            .iter()
            .map(|outcome| outcome.report.duplicate_rate())
            .sum::<f64>()
            / scenario_count;
        let strongest_action = outcomes
            .iter()
            .map(|outcome| outcome.assessment.recommended_action)
            .max_by_key(|action| remediation_rank(*action))
            .unwrap_or(RemediationAction::Healthy);

        Ok(CampaignReport {
            scenarios: outcomes,
            aggregate_delivery_rate,
            aggregate_duplicate_rate,
            strongest_action,
        })
    }
}

/// Deterministic in-memory transport harness capable of simulating bounded
/// drop/duplicate/reorder/jitter/partition conditions.
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

        if self.profile.applies_partition_drop(self.emitted_frames) {
            self.report.partition_dropped_frames =
                self.report.partition_dropped_frames.saturating_add(1);
            return Ok(());
        }

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

const fn remediation_rank(action: RemediationAction) -> u8 {
    match action {
        RemediationAction::Healthy => 0,
        RemediationAction::Observe => 1,
        RemediationAction::Quarantine => 2,
        RemediationAction::Ban => 3,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        AdversarialCampaign, ChaosProfile, RemediationAction, ResilienceAssessment,
        ResilienceHarness, ScenarioDefinition,
    };
    use crate::config::NetworkConfig;
    use crate::gossip::peer::{NodeCertificate, Peer, PeerRole};
    use crate::p2p::P2PNetwork;
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
                signature_scheme: 1,
            },
            signature: vec![4u8; 64],
        })
    }

    #[test]
    fn resilience_harness_applies_drop_duplicate_reorder_and_partition_rules() {
        let mut network = P2PNetwork::new(NetworkConfig::default());
        network
            .register_peer(peer("node-1"))
            .expect("peer registers");
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
            partition_start_frame: Some(4),
            partition_frame_len: 1,
        })
        .expect("valid profile");

        for height in 1..=5 {
            let envelope = network
                .broadcast_secure("node-1", vote(height))
                .expect("broadcast works");
            harness.enqueue(envelope).expect("buffer accepts frame");
        }

        let report = harness.flush();
        assert_eq!(report.accepted_frames, 3);
        assert_eq!(report.dropped_frames, 1);
        assert_eq!(report.partition_dropped_frames, 1);
        assert_eq!(report.duplicated_frames, 1);
        assert!(report.reordered_frames >= 1);
        assert_eq!(report.delivered.len(), 4);
        assert!(report.delivered.iter().any(|event| event.duplicated));
    }

    #[test]
    fn resilience_harness_rejects_unbounded_backpressure() {
        let mut network = P2PNetwork::new(NetworkConfig::default());
        network
            .register_peer(peer("node-1"))
            .expect("peer registers");
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
        network
            .register_peer(peer("node-1"))
            .expect("peer registers");
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
            partition_start_frame: Some(4),
            partition_frame_len: 2,
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

    #[test]
    fn assessment_recommends_ban_and_enforces_it() {
        let mut network = P2PNetwork::new(NetworkConfig::default());
        network
            .register_peer(peer("node-1"))
            .expect("peer registers");
        network
            .establish_session("node-1")
            .expect("session established");

        let mut harness = ResilienceHarness::new(ChaosProfile {
            seed: 5,
            drop_every_nth: 2,
            duplicate_every_nth: 1,
            reorder_window: 2,
            max_jitter_ms: 25,
            max_inflight_frames: 32,
            partition_start_frame: Some(5),
            partition_frame_len: 3,
        })
        .expect("valid profile");

        for height in 1..=8 {
            let envelope = network
                .broadcast_secure("node-1", vote(height))
                .expect("broadcast works");
            harness.enqueue(envelope).expect("enqueue works");
        }

        let report = harness.flush();
        let assessment = ResilienceAssessment::from_report(&report);
        assert_eq!(assessment.recommended_action, RemediationAction::Ban);

        assessment.enforce(&mut network);
        let error = network
            .broadcast_secure("node-1", vote(99))
            .expect_err("banned peer must be blocked");
        assert_eq!(error.code(), "AOXCNET_PEER_BANNED");
    }

    #[test]
    fn adversarial_campaign_aggregates_scenarios() {
        let mut network = P2PNetwork::new(NetworkConfig::default());
        network
            .register_peer(peer("node-1"))
            .expect("peer registers");
        network
            .establish_session("node-1")
            .expect("session established");

        let frames = (1..=6)
            .map(|height| {
                network
                    .broadcast_secure("node-1", vote(height))
                    .expect("broadcast works")
            })
            .collect::<Vec<_>>();

        let campaign = AdversarialCampaign::new(vec![
            ScenarioDefinition {
                name: "baseline".to_string(),
                profile: ChaosProfile {
                    max_inflight_frames: 16,
                    ..ChaosProfile::default()
                },
            },
            ScenarioDefinition {
                name: "partitioned".to_string(),
                profile: ChaosProfile {
                    seed: 9,
                    duplicate_every_nth: 2,
                    reorder_window: 2,
                    max_jitter_ms: 5,
                    max_inflight_frames: 16,
                    partition_start_frame: Some(3),
                    partition_frame_len: 2,
                    ..ChaosProfile::default()
                },
            },
        ]);

        let report = campaign.execute(&frames).expect("campaign runs");
        assert_eq!(report.scenarios.len(), 2);
        assert!(report.aggregate_delivery_rate > 0.5);
        assert!(matches!(
            report.strongest_action,
            RemediationAction::Healthy
                | RemediationAction::Observe
                | RemediationAction::Quarantine
                | RemediationAction::Ban
        ));
    }
}
