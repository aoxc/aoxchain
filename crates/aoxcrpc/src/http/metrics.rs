#[must_use]
pub fn prometheus_metrics_snapshot(
    total_requests: u64,
    rejected_requests: u64,
    rate_limited_requests: u64,
    active_rate_limiter_keys: u64,
    health_readiness_score: u8,
) -> String {
    format!(
        "# HELP aox_rpc_requests_total Total RPC requests\n\
# TYPE aox_rpc_requests_total counter\n\
aox_rpc_requests_total {}\n\
# HELP aox_rpc_rejected_total Total rejected RPC requests\n\
# TYPE aox_rpc_rejected_total counter\n\
aox_rpc_rejected_total {}\n\
# HELP aox_rpc_rate_limited_total Total requests rejected by rate limiting\n\
# TYPE aox_rpc_rate_limited_total counter\n\
aox_rpc_rate_limited_total {}\n\
# HELP aox_rpc_rate_limiter_active_keys Active keys tracked by in-memory rate limiter\n\
# TYPE aox_rpc_rate_limiter_active_keys gauge\n\
aox_rpc_rate_limiter_active_keys {}\n\
# HELP aox_rpc_health_readiness_score Current readiness score of RPC node (0-100)\n\
# TYPE aox_rpc_health_readiness_score gauge\n\
aox_rpc_health_readiness_score {}\n",
        total_requests,
        rejected_requests,
        rate_limited_requests,
        active_rate_limiter_keys,
        health_readiness_score
aox_rpc_rate_limiter_active_keys {}\n",
        total_requests, rejected_requests, rate_limited_requests, active_rate_limiter_keys
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snapshot_contains_new_security_metrics() {
        let snapshot = prometheus_metrics_snapshot(120, 5, 3, 42, 85);

        assert!(snapshot.contains("aox_rpc_rate_limited_total 3"));
        assert!(snapshot.contains("aox_rpc_rate_limiter_active_keys 42"));
        assert!(snapshot.contains("aox_rpc_health_readiness_score 85"));
        let snapshot = prometheus_metrics_snapshot(120, 5, 3, 42);

        assert!(snapshot.contains("aox_rpc_rate_limited_total 3"));
        assert!(snapshot.contains("aox_rpc_rate_limiter_active_keys 42"));
    }
}
