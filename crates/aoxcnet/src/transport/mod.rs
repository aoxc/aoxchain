pub mod live_tcp;

/// Supported transport protocols for AOXChain p2p links.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransportKind {
    Tcp,
    Udp,
    Quic,
}

#[derive(Debug, Clone)]
pub struct TransportConfig {
    pub kind: TransportKind,
    pub bind_addr: String,
}

impl Default for TransportConfig {
    fn default() -> Self {
        Self {
            kind: TransportKind::Quic,
            bind_addr: "0.0.0.0:26656".to_string(),
        }
    }
}
