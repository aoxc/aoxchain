#[must_use]
pub fn prometheus_metrics_snapshot(total_requests: u64, rejected_requests: u64) -> String {
    format!(
        "# HELP aox_rpc_requests_total Total RPC requests\n\
# TYPE aox_rpc_requests_total counter\n\
aox_rpc_requests_total {}\n\
# HELP aox_rpc_rejected_total Total rejected RPC requests\n\
# TYPE aox_rpc_rejected_total counter\n\
aox_rpc_rejected_total {}\n",
        total_requests, rejected_requests
    )
}
