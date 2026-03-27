use serde::{Deserialize, Serialize};

/// Mutable network metrics for diagnostics, alerting, and audit reporting.
#[derive(Debug, Clone, Default)]
pub struct NetworkMetrics {
    pub accepted_peers: u64,
    pub rejected_peers: u64,
    pub active_sessions: u64,
    pub failed_handshakes: u64,
    pub replay_detections: u64,
    pub banned_peers: u64,
    pub frames_in: u64,
    pub frames_out: u64,
    pub bytes_in: u64,
    pub bytes_out: u64,
    pub gossip_messages: u64,
    pub sync_requests: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NetworkMetricsSnapshot {
    pub accepted_peers: u64,
    pub rejected_peers: u64,
    pub active_sessions: u64,
    pub failed_handshakes: u64,
    pub replay_detections: u64,
    pub banned_peers: u64,
    pub frames_in: u64,
    pub frames_out: u64,
    pub bytes_in: u64,
    pub bytes_out: u64,
    pub gossip_messages: u64,
    pub sync_requests: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NetworkHealthReport {
    pub readiness_score: u8,
    pub classification: &'static str,
    pub acceptance_rate_bps: u16,
    pub handshake_failure_rate_bps: u16,
    pub replay_rate_bps: u16,
    pub pressure_state: &'static str,
    pub recommendations: Vec<String>,
}

impl NetworkMetrics {
    #[must_use]
    pub fn snapshot(&self) -> NetworkMetricsSnapshot {
        NetworkMetricsSnapshot {
            accepted_peers: self.accepted_peers,
            rejected_peers: self.rejected_peers,
            active_sessions: self.active_sessions,
            failed_handshakes: self.failed_handshakes,
            replay_detections: self.replay_detections,
            banned_peers: self.banned_peers,
            frames_in: self.frames_in,
            frames_out: self.frames_out,
            bytes_in: self.bytes_in,
            bytes_out: self.bytes_out,
            gossip_messages: self.gossip_messages,
            sync_requests: self.sync_requests,
        }
    }

    #[must_use]
    pub fn accepted_peer_rate_bps(&self) -> u16 {
        ratio_bps(
            self.accepted_peers,
            self.accepted_peers.saturating_add(self.rejected_peers),
        )
    }

    #[must_use]
    pub fn handshake_failure_rate_bps(&self) -> u16 {
        ratio_bps(
            self.failed_handshakes,
            self.failed_handshakes.saturating_add(self.accepted_peers),
        )
    }

    #[must_use]
    pub fn replay_rate_bps(&self) -> u16 {
        ratio_bps(self.replay_detections, self.frames_in.max(1))
    }

    #[must_use]
    pub fn pressure_state(&self) -> &'static str {
        if self.active_sessions == 0 {
            return "idle";
        }

        let frame_pressure = self.frames_in.saturating_sub(self.frames_out);
        if frame_pressure > 1_000 || self.failed_handshakes > self.accepted_peers {
            "high"
        } else if frame_pressure > 200 || self.replay_detections > 0 {
            "elevated"
        } else {
            "normal"
        }
    }

    #[must_use]
    pub fn readiness_report(&self) -> NetworkHealthReport {
        let acceptance_rate_bps = self.accepted_peer_rate_bps();
        let handshake_failure_rate_bps = self.handshake_failure_rate_bps();
        let replay_rate_bps = self.replay_rate_bps();
        let pressure_state = self.pressure_state();

        let mut score: i32 = 100;
        score -= i32::from(10_000_u16.saturating_sub(acceptance_rate_bps)) / 200;
        score -= i32::from(handshake_failure_rate_bps) / 120;
        score -= i32::from(replay_rate_bps) / 80;

        if pressure_state == "elevated" {
            score -= 10;
        } else if pressure_state == "high" {
            score -= 25;
        }

        let readiness_score = score.clamp(0, 100) as u8;
        let classification = if readiness_score >= 85 {
            "stable"
        } else if readiness_score >= 60 {
            "guarded"
        } else {
            "critical"
        };

        let mut recommendations = Vec::new();
        if acceptance_rate_bps < 8_500 {
            recommendations.push(
                "Peer admission rejection ratio is high; review allowlists, certificate chain and anti-sybil rules"
                    .to_string(),
            );
        }
        if handshake_failure_rate_bps > 1_200 {
            recommendations.push(
                "Handshake failures exceed normal envelope; inspect TLS/mTLS and protocol version compatibility"
                    .to_string(),
            );
        }
        if replay_rate_bps > 80 {
            recommendations.push(
                "Replay detection rate is elevated; tighten nonce window and investigate duplicate broadcasters"
                    .to_string(),
            );
        }
        if pressure_state == "high" {
            recommendations.push(
                "Transport backpressure is high; scale networking workers or reduce gossip fanout"
                    .to_string(),
            );
        }

        NetworkHealthReport {
            readiness_score,
            classification,
            acceptance_rate_bps,
            handshake_failure_rate_bps,
            replay_rate_bps,
            pressure_state,
            recommendations,
        }
    }
}

fn ratio_bps(numerator: u64, denominator: u64) -> u16 {
    if denominator == 0 {
        return 0;
    }
    ((numerator.saturating_mul(10_000)) / denominator).min(10_000) as u16
}

#[cfg(test)]
mod tests {
    use super::NetworkMetrics;

    #[test]
    fn readiness_report_is_stable_for_healthy_traffic() {
        let metrics = NetworkMetrics {
            accepted_peers: 950,
            rejected_peers: 30,
            active_sessions: 120,
            failed_handshakes: 12,
            replay_detections: 1,
            frames_in: 5000,
            frames_out: 4980,
            ..NetworkMetrics::default()
        };

        let report = metrics.readiness_report();
        assert!(report.readiness_score >= 85);
        assert_eq!(report.classification, "stable");
        assert!(report.recommendations.is_empty());
    }

    #[test]
    fn readiness_report_turns_critical_under_transport_stress() {
        let metrics = NetworkMetrics {
            accepted_peers: 100,
            rejected_peers: 80,
            active_sessions: 210,
            failed_handshakes: 220,
            replay_detections: 90,
            frames_in: 1200,
            frames_out: 50,
            ..NetworkMetrics::default()
        };

        let report = metrics.readiness_report();
        assert!(report.readiness_score < 60);
        assert_eq!(report.classification, "critical");
        assert_eq!(report.pressure_state, "high");
        assert!(report.recommendations.len() >= 3);
    }

    #[test]
    fn basis_point_ratios_are_bounded() {
        let metrics = NetworkMetrics {
            accepted_peers: 5,
            rejected_peers: 0,
            failed_handshakes: 2,
            frames_in: 0,
            ..NetworkMetrics::default()
        };

        assert_eq!(metrics.accepted_peer_rate_bps(), 10_000);
        assert_eq!(metrics.handshake_failure_rate_bps(), 2_857);
        assert_eq!(metrics.replay_rate_bps(), 0);
    }
}
