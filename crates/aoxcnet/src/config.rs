/// Security policy profile for p2p transport.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SecurityMode {
    /// No encryption or certificate checks.
    Insecure,
    /// Mutual certificate verification and replay protection.
    MutualAuth,
    /// Mutual auth + strict policy checks for production environments.
    AuditStrict,
}

/// Network configuration parameters.
#[derive(Debug, Clone)]
pub struct NetworkConfig {
    pub listen_addr: String,
    pub public_advertise_addr: String,
    pub max_peers: usize,
    pub heartbeat_ms: u64,
    pub security_mode: SecurityMode,
}

use crate::ports::P2P_PRIMARY_PORT;

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            listen_addr: format!("0.0.0.0:{P2P_PRIMARY_PORT}"),
            public_advertise_addr: format!("127.0.0.1:{P2P_PRIMARY_PORT}"),
            max_peers: 128,
            heartbeat_ms: 1_000,
            security_mode: SecurityMode::MutualAuth,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::NetworkConfig;

    #[test]
    fn default_network_uses_canonical_primary_p2p_port() {
        let config = NetworkConfig::default();
        assert_eq!(config.listen_addr, "0.0.0.0:2727");
        assert_eq!(config.public_advertise_addr, "127.0.0.1:2727");
    }
}
