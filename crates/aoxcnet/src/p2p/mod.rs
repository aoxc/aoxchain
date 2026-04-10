// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use std::collections::{HashMap, HashSet, VecDeque};
use std::time::{SystemTime, UNIX_EPOCH};

use aoxcunity::messages::ConsensusMessage;
use blake3::Hasher;
use serde::{Deserialize, Serialize};

use crate::config::{NetworkConfig, SecurityMode};
use crate::discovery::{DiscoveryTable, PeerCandidate, RpcSurface};
use crate::error::NetworkError;
use crate::gossip::peer::Peer;
use crate::metrics::{NetworkMetrics, NetworkMetricsSnapshot};
use crate::secure_session::{
    AOXC_Q_RELEASE_LINE, HandshakeIntent, HandshakePolicy, HandshakeRejectReason, PeerClass,
    TransportCryptoProfile,
};

include!("p2p_envelope.rs");
include!("p2p_network.rs");
include!("p2p_tests.rs");
