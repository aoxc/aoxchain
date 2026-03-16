/// Canonical AOXChain port map used by CLI and network defaults.
///
/// Ports are intentionally mnemonic and grouped by subsystem family.
pub const RPC_HTTP_PORT: u16 = 2626;
pub const P2P_PRIMARY_PORT: u16 = 2727;
pub const P2P_GOSSIP_PORT: u16 = 2828;
pub const P2P_DISCOVERY_PORT: u16 = 2929;
pub const RPC_WS_PORT: u16 = 3030;
pub const RPC_GRPC_PORT: u16 = 3131;
pub const METRICS_PORT: u16 = 3232;
pub const ADMIN_API_PORT: u16 = 3333;
pub const PROFILER_PORT: u16 = 3434;
pub const STORAGE_API_PORT: u16 = 3535;
pub const LIVE_SMOKE_TEST_PORT: u16 = 3636;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PortBinding {
    pub name: &'static str,
    pub protocol: &'static str,
    pub bind: &'static str,
    pub port: u16,
    pub purpose: &'static str,
}

pub const PORT_BINDINGS: [PortBinding; 11] = [
    PortBinding {
        name: "rpc_http",
        protocol: "tcp",
        bind: "0.0.0.0",
        port: RPC_HTTP_PORT,
        purpose: "General JSON-RPC HTTP API",
    },
    PortBinding {
        name: "p2p_primary",
        protocol: "tcp/quic",
        bind: "0.0.0.0",
        port: P2P_PRIMARY_PORT,
        purpose: "Primary peer-to-peer transport",
    },
    PortBinding {
        name: "p2p_gossip",
        protocol: "udp",
        bind: "0.0.0.0",
        port: P2P_GOSSIP_PORT,
        purpose: "Gossip fanout traffic",
    },
    PortBinding {
        name: "p2p_discovery",
        protocol: "udp",
        bind: "0.0.0.0",
        port: P2P_DISCOVERY_PORT,
        purpose: "Peer discovery and liveness probes",
    },
    PortBinding {
        name: "rpc_ws",
        protocol: "tcp",
        bind: "0.0.0.0",
        port: RPC_WS_PORT,
        purpose: "Realtime websocket subscriptions",
    },
    PortBinding {
        name: "rpc_grpc",
        protocol: "tcp",
        bind: "0.0.0.0",
        port: RPC_GRPC_PORT,
        purpose: "High-throughput gRPC API",
    },
    PortBinding {
        name: "metrics",
        protocol: "tcp",
        bind: "127.0.0.1",
        port: METRICS_PORT,
        purpose: "Prometheus metrics exporter",
    },
    PortBinding {
        name: "admin_api",
        protocol: "tcp",
        bind: "127.0.0.1",
        port: ADMIN_API_PORT,
        purpose: "Node admin and operational endpoints",
    },
    PortBinding {
        name: "profiler",
        protocol: "tcp",
        bind: "127.0.0.1",
        port: PROFILER_PORT,
        purpose: "pprof/diagnostics endpoints",
    },
    PortBinding {
        name: "storage_api",
        protocol: "tcp",
        bind: "127.0.0.1",
        port: STORAGE_API_PORT,
        purpose: "Storage/index service API",
    },
    PortBinding {
        name: "live_smoke_test",
        protocol: "tcp",
        bind: "127.0.0.1",
        port: LIVE_SMOKE_TEST_PORT,
        purpose: "Deterministic network smoke tests",
    },
];

#[cfg(test)]
mod tests {
    use super::{
        LIVE_SMOKE_TEST_PORT, P2P_GOSSIP_PORT, P2P_PRIMARY_PORT, PORT_BINDINGS, RPC_HTTP_PORT,
    };

    #[test]
    fn canonical_ports_match_expected_values() {
        assert_eq!(RPC_HTTP_PORT, 2626);
        assert_eq!(P2P_PRIMARY_PORT, 2727);
        assert_eq!(P2P_GOSSIP_PORT, 2828);
        assert_eq!(LIVE_SMOKE_TEST_PORT, 3636);
    }

    #[test]
    fn port_binding_names_are_unique() {
        let mut names: Vec<&str> = PORT_BINDINGS.iter().map(|p| p.name).collect();
        let before = names.len();
        names.sort_unstable();
        names.dedup();
        assert_eq!(names.len(), before);
    }
}
