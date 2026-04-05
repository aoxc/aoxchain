// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use serde::{Deserialize, Serialize};

use crate::ports::{P2P_DISCOVERY_PORT, P2P_PRIMARY_PORT};

/// Defines the network security posture enforced by the node.
///
/// Each mode represents a formally recognized operational trust boundary.
/// Deployments MUST select the mode that accurately reflects the security
/// guarantees expected from the target environment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SecurityMode {
    /// Permits non-hardened behavior intended exclusively for local
    /// development, isolated simulation, and controlled debugging.
    ///
    /// This mode MUST NOT be enabled in staging or production, as it weakens
    /// transport trust assumptions and materially reduces operational rigor.
    Insecure,

    /// Enforces secure network behavior suitable for standard protected
    /// environments, including authenticated peer establishment and replay
    /// resistance controls.
    MutualAuth,

    /// Enforces the strictest policy profile for production-grade and
    /// audit-sensitive deployments, including tighter timing, stronger
    /// behavioral assumptions, and stricter configuration requirements.
    AuditStrict,
}

/// Enumerates supported external execution-domain families.
///
/// This classification allows the protocol to distinguish native AOXC traffic
/// from foreign execution environments for routing, policy, and attestation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ExternalDomainKind {
    Native,
    Evm,
    Move,
    Utxo,
    Wasm,
}

/// Defines the preferred transport strategy for peer communication.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransportPreference {
    Tcp,
    Quic,
    Hybrid,
}

/// Defines the canonical serial class of the chain.
///
/// This abstraction allows the protocol to distinguish whether the chain uses
/// a purely sovereign native identity model or additionally exposes
/// compatibility-oriented external serial mappings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SerialIdentityClass {
    /// Pure sovereign identity model with no externally mirrored serial space.
    NativeSovereign,

    /// Sovereign identity model with an EVM-facing numeric protocol serial.
    NativeWithEvmProjection,

    /// Sovereign identity model with multiple compatibility-oriented serial
    /// projections for foreign execution ecosystems.
    MultiDomainProjected,
}

/// Defines the canonical serial identity of the AOXC network.
///
/// This structure intentionally separates symbolic identity, institutional
/// serial formatting, numeric protocol serials, and derivation-path identity.
/// The separation is deliberate: a sovereign chain must not collapse all
/// identity concerns into a single borrowed value.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerialIdentityPolicy {
    /// Canonical institutional chain name.
    ///
    /// This is the highest-level human-readable chain identity used across
    /// governance, audit artifacts, operator handbooks, release management,
    /// and institutional documentation.
    pub canonical_chain_name: String,

    /// Canonical operator-facing serial label.
    ///
    /// This value is intended for logs, dashboards, reports, explorer headers,
    /// and user-facing administrative surfaces.
    pub canonical_serial_label: String,

    /// Canonical fixed-width genesis-origin serial string.
    ///
    /// This field represents the institutional origin marker of the chain.
    /// It is symbolic and archival in nature and MUST NOT be treated as a
    /// substitute for numeric protocol validation.
    pub genesis_origin_serial: String,

    /// Canonical numeric protocol serial.
    ///
    /// This value is the authoritative numeric identifier recognized by the
    /// AOXC runtime for machine-level protocol separation.
    pub protocol_serial: u64,

    /// Canonical BIP44 coin type assigned to the AOXC identity space.
    ///
    /// This field defines the derivation-domain identity used by wallet
    /// infrastructure operating under AOXC's own sovereign standards.
    pub bip44_coin_type: u32,

    /// Canonical serial identity model used by the chain.
    pub serial_identity_class: SerialIdentityClass,

    /// Indicates whether the protocol permits external compatibility mappings
    /// in addition to its sovereign native serial system.
    pub allow_external_serial_projection: bool,
}

impl Default for SerialIdentityPolicy {
    fn default() -> Self {
        Self {
            canonical_chain_name: "AOXC-MAINNET".to_string(),
            canonical_serial_label: "AOXC-000001".to_string(),
            genesis_origin_serial: "000000000001".to_string(),
            protocol_serial: 2626,
            bip44_coin_type: 2626,
            serial_identity_class: SerialIdentityClass::NativeWithEvmProjection,
            allow_external_serial_projection: true,
        }
    }
}

impl SerialIdentityPolicy {
    /// Returns the canonical BIP44 account derivation prefix.
    ///
    /// The returned value is deterministic and institutionally anchored to the
    /// configured AOXC coin-type identity.
    #[must_use]
    pub fn derivation_path_prefix(&self) -> String {
        format!("m/44'/{}'/0'/0", self.bip44_coin_type)
    }

    /// Returns `true` when the policy is strictly native and does not allow
    /// outward serial projection.
    #[must_use]
    pub fn is_strictly_native(&self) -> bool {
        matches!(
            self.serial_identity_class,
            SerialIdentityClass::NativeSovereign
        ) && !self.allow_external_serial_projection
    }

    /// Validates the serial identity policy against institutional invariants.
    ///
    /// These checks exist to ensure that symbolic identity, numeric protocol
    /// identity, and derivation identity remain internally coherent.
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.canonical_chain_name.trim().is_empty() {
            return Err("SERIAL_IDENTITY_CHAIN_NAME_EMPTY");
        }

        if self.canonical_serial_label.trim().is_empty() {
            return Err("SERIAL_IDENTITY_LABEL_EMPTY");
        }

        if self.genesis_origin_serial.len() != 12 {
            return Err("SERIAL_IDENTITY_GENESIS_SERIAL_LENGTH_INVALID");
        }

        if !self
            .genesis_origin_serial
            .bytes()
            .all(|byte| byte.is_ascii_digit())
        {
            return Err("SERIAL_IDENTITY_GENESIS_SERIAL_NON_NUMERIC");
        }

        if self.protocol_serial == 0 {
            return Err("SERIAL_IDENTITY_PROTOCOL_SERIAL_INVALID");
        }

        if self.bip44_coin_type == 0 {
            return Err("SERIAL_IDENTITY_BIP44_COIN_TYPE_INVALID");
        }

        if !self.canonical_serial_label.starts_with("AOXC-") {
            return Err("SERIAL_IDENTITY_LABEL_PREFIX_INVALID");
        }

        if matches!(
            self.serial_identity_class,
            SerialIdentityClass::NativeSovereign
        ) && self.allow_external_serial_projection
        {
            return Err("SERIAL_IDENTITY_EXTERNAL_PROJECTION_FORBIDDEN");
        }

        Ok(())
    }
}

/// Defines interoperability controls for foreign execution domains.
///
/// This structure intentionally references `SerialIdentityPolicy` rather than
/// attempting to overload a single string field with all identity concerns.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteropPolicy {
    /// Canonical AOXC serial identity policy bound to this runtime.
    pub serial_identity: SerialIdentityPolicy,

    /// Enables or disables communication with non-native execution domains.
    pub allow_external_domains: bool,

    /// Explicit allowlist of foreign domain families that may be recognized by
    /// the node under controlled interoperability policy.
    pub allowed_domains: Vec<ExternalDomainKind>,

    /// Requires domain-level attestation before foreign-origin messages may be
    /// accepted into the AOXC runtime trust boundary.
    pub require_domain_attestation: bool,
}

impl Default for InteropPolicy {
    fn default() -> Self {
        Self {
            serial_identity: SerialIdentityPolicy::default(),
            allow_external_domains: false,
            allowed_domains: vec![ExternalDomainKind::Native],
            require_domain_attestation: true,
        }
    }
}

impl InteropPolicy {
    /// Returns the canonical local chain identifier used for protocol framing,
    /// peer admission checks, and domain separation.
    #[must_use]
    pub fn canonical_chain_id(&self) -> &str {
        &self.serial_identity.canonical_chain_name
    }

    /// Returns the canonical numeric protocol serial of the local chain.
    #[must_use]
    pub fn canonical_protocol_serial(&self) -> u64 {
        self.serial_identity.protocol_serial
    }

    /// Returns `true` when the supplied domain is admissible under the
    /// configured interoperability policy.
    #[must_use]
    pub fn is_domain_allowed(&self, domain: ExternalDomainKind) -> bool {
        self.allowed_domains.contains(&domain)
    }

    /// Validates the interoperability policy and its bound serial identity.
    pub fn validate(&self) -> Result<(), &'static str> {
        self.serial_identity.validate()?;

        if self.allowed_domains.is_empty() {
            return Err("INTEROP_POLICY_ALLOWED_DOMAINS_EMPTY");
        }

        if !self.allowed_domains.contains(&ExternalDomainKind::Native) {
            return Err("INTEROP_POLICY_NATIVE_DOMAIN_REQUIRED");
        }

        if !self.allow_external_domains && self.allowed_domains != vec![ExternalDomainKind::Native]
        {
            return Err("INTEROP_POLICY_EXTERNAL_DOMAINS_FORBIDDEN");
        }

        if self.serial_identity.is_strictly_native() && self.allow_external_domains {
            return Err("INTEROP_POLICY_NATIVE_SERIAL_CLASS_CONFLICT");
        }

        Ok(())
    }
}

/// Defines controls for the high-scrutiny inspection lane.
///
/// This policy is intended for elevated-risk operations such as bridge-facing
/// flows, sensitive routing paths, and compliance-sensitive message handling.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InspectionLanePolicy {
    /// Enables AI-assisted inspection for high-risk message flows.
    pub enable_ai_inspection_lane: bool,

    /// Requires explicit human intervention or approval before final
    /// acceptance when the inspection lane is engaged.
    pub require_human_override: bool,

    /// Minimum KYC tier required for bridge-related operations processed
    /// through the inspection lane.
    pub minimum_bridge_kyc_tier: u8,
}

impl Default for InspectionLanePolicy {
    fn default() -> Self {
        Self {
            enable_ai_inspection_lane: true,
            require_human_override: true,
            minimum_bridge_kyc_tier: 2,
        }
    }
}

/// Represents the complete network runtime configuration.
///
/// The structure is intentionally designed for deterministic serialization so
/// that TOML, JSON, and comparable config backends can consume it without
/// ambiguity or lossy translation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    /// Local socket address on which the node accepts primary P2P traffic.
    pub listen_addr: String,

    /// Publicly advertised address communicated to remote peers.
    pub public_advertise_addr: String,

    /// Local socket address for peer discovery traffic.
    pub discovery_addr: String,

    /// Maximum number of simultaneously accepted inbound peers.
    pub max_inbound_peers: usize,

    /// Maximum number of simultaneously maintained outbound peers.
    pub max_outbound_peers: usize,

    /// Heartbeat interval in milliseconds.
    pub heartbeat_ms: u64,

    /// Maximum permitted handshake duration in milliseconds.
    pub handshake_timeout_ms: u64,

    /// Maximum idle connection duration in milliseconds.
    pub idle_timeout_ms: u64,

    /// Maximum permitted frame size in bytes.
    pub max_frame_bytes: usize,

    /// Maximum number of messages permitted in a single gossip batch.
    pub max_gossip_batch: usize,

    /// Maximum number of messages permitted in a single synchronization batch.
    pub max_sync_batch: usize,

    /// Replay protection window size.
    pub replay_window_size: usize,

    /// Maximum tolerated peer clock skew in seconds.
    pub allowed_clock_skew_secs: u64,

    /// Peer ban duration in seconds.
    pub peer_ban_secs: u64,

    /// Preferred transport selection for runtime connection establishment.
    pub transport_preference: TransportPreference,

    /// Active node security posture.
    pub security_mode: SecurityMode,

    /// Cross-domain interoperability controls, including canonical serial
    /// identity definitions.
    pub interop: InteropPolicy,

    /// Inspection-lane controls for elevated-risk operations.
    pub inspection: InspectionLanePolicy,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            listen_addr: format!("0.0.0.0:{P2P_PRIMARY_PORT}"),
            public_advertise_addr: format!("127.0.0.1:{P2P_PRIMARY_PORT}"),
            discovery_addr: format!("0.0.0.0:{P2P_DISCOVERY_PORT}"),
            max_inbound_peers: 64,
            max_outbound_peers: 64,
            heartbeat_ms: 1_000,
            handshake_timeout_ms: 5_000,
            idle_timeout_ms: 30_000,
            max_frame_bytes: 256 * 1024,
            max_gossip_batch: 128,
            max_sync_batch: 256,
            replay_window_size: 4_096,
            allowed_clock_skew_secs: 30,
            peer_ban_secs: 900,
            transport_preference: TransportPreference::Hybrid,
            security_mode: SecurityMode::MutualAuth,
            interop: InteropPolicy::default(),
            inspection: InspectionLanePolicy::default(),
        }
    }
}

impl NetworkConfig {
    /// Returns the aggregate peer capacity using saturating arithmetic to
    /// eliminate overflow risk under malformed or extreme input values.
    #[must_use]
    pub fn max_peers_total(&self) -> usize {
        self.max_inbound_peers
            .saturating_add(self.max_outbound_peers)
    }

    /// Returns `true` when the runtime is operating under the strictest
    /// deployment posture.
    #[must_use]
    pub fn is_audit_strict(&self) -> bool {
        matches!(self.security_mode, SecurityMode::AuditStrict)
    }

    /// Returns `true` when the runtime must enforce certificate-backed mutual
    /// authentication on peer transport setup.
    #[must_use]
    pub fn requires_mutual_auth(&self) -> bool {
        !matches!(self.security_mode, SecurityMode::Insecure)
    }

    /// Validates the complete network configuration against minimum safety,
    /// identity-consistency, and operational-correctness requirements.
    ///
    /// The returned error codes are intentionally stable and machine-readable
    /// so they can be consumed by CI validation, diagnostics, and deployment
    /// enforcement workflows.
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.listen_addr.trim().is_empty() {
            return Err("NETWORK_CONFIG_LISTEN_ADDR_EMPTY");
        }

        if self.public_advertise_addr.trim().is_empty() {
            return Err("NETWORK_CONFIG_PUBLIC_ADVERTISE_ADDR_EMPTY");
        }

        if self.discovery_addr.trim().is_empty() {
            return Err("NETWORK_CONFIG_DISCOVERY_ADDR_EMPTY");
        }

        if self.max_inbound_peers == 0 || self.max_outbound_peers == 0 {
            return Err("NETWORK_CONFIG_PEER_LIMIT_ZERO");
        }

        if self.max_frame_bytes < 1024 {
            return Err("NETWORK_CONFIG_FRAME_TOO_SMALL");
        }

        if self.heartbeat_ms == 0 {
            return Err("NETWORK_CONFIG_HEARTBEAT_INVALID");
        }

        if self.handshake_timeout_ms == 0 || self.idle_timeout_ms == 0 {
            return Err("NETWORK_CONFIG_TIMEOUT_INVALID");
        }

        if self.replay_window_size == 0 {
            return Err("NETWORK_CONFIG_REPLAY_WINDOW_INVALID");
        }

        if self.max_gossip_batch == 0 {
            return Err("NETWORK_CONFIG_GOSSIP_BATCH_INVALID");
        }

        if self.max_sync_batch == 0 {
            return Err("NETWORK_CONFIG_SYNC_BATCH_INVALID");
        }

        self.interop.validate()?;

        if self.requires_mutual_auth()
            && matches!(self.transport_preference, TransportPreference::Tcp)
        {
            return Err("NETWORK_CONFIG_MTLS_TRANSPORT_REQUIRED");
        }

        if self.requires_mutual_auth() && !self.interop.require_domain_attestation {
            return Err("NETWORK_CONFIG_DOMAIN_ATTESTATION_REQUIRED");
        }

        if self.is_audit_strict() {
            if self.allowed_clock_skew_secs > 60 {
                return Err("NETWORK_CONFIG_CLOCK_SKEW_UNSAFE");
            }

            if self.peer_ban_secs < 300 {
                return Err("NETWORK_CONFIG_BAN_DURATION_WEAK");
            }

            if self.handshake_timeout_ms > self.idle_timeout_ms {
                return Err("NETWORK_CONFIG_HANDSHAKE_TIMEOUT_EXCEEDS_IDLE_TIMEOUT");
            }
        }

        Ok(())
    }
}
