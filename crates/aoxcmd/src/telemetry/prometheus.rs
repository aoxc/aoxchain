/// Static metrics snapshot used by `/metrics` style exporters.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MetricsSnapshot {
    pub tps: f64,
    pub peer_count: usize,
    pub error_rate: f64,
}

impl Default for MetricsSnapshot {
    fn default() -> Self {
        Self {
            tps: 0.0,
            peer_count: 0,
            error_rate: 0.0,
        }
    }
}

impl MetricsSnapshot {
    /// Encodes a minimal Prometheus exposition payload for runtime dashboards.
    #[must_use]
    pub fn to_prometheus(self) -> String {
        format!(
            "aox_tps {}\naox_peer_count {}\naox_error_rate {}\n",
            self.tps, self.peer_count, self.error_rate
        )
    }
}

#[cfg(test)]
mod tests {
    use super::MetricsSnapshot;

    #[test]
    fn renders_prometheus_payload() {
        let snapshot = MetricsSnapshot {
            tps: 17.5,
            peer_count: 9,
            error_rate: 0.02,
        };

        let payload = snapshot.to_prometheus();

        assert!(payload.contains("aox_tps 17.5"));
        assert!(payload.contains("aox_peer_count 9"));
        assert!(payload.contains("aox_error_rate 0.02"));
    }
}
