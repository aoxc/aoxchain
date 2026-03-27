// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

pub mod live_tcp;

use serde::{Deserialize, Serialize};

use crate::ports::P2P_PRIMARY_PORT;

/// Supported transport protocols for AOXChain peer links.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransportKind {
    Tcp,
    Udp,
    Quic,
}

#[derive(Debug, Clone)]
pub struct TransportConfig {
    pub kind: TransportKind,
    pub bind_addr: String,
    pub require_mutual_auth: bool,
    pub max_frame_bytes: usize,
}

impl Default for TransportConfig {
    fn default() -> Self {
        Self {
            kind: TransportKind::Quic,
            bind_addr: format!("0.0.0.0:{P2P_PRIMARY_PORT}"),
            require_mutual_auth: true,
            max_frame_bytes: 256 * 1024,
        }
    }
}
