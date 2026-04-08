use serde::{Deserialize, Serialize};

/// Asset category recognized by the registry.
///
/// Security rationale:
/// The asset class drives policy boundaries, governance expectations, risk
/// treatment, and lifecycle assumptions. This field is therefore a primary
/// policy selector.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AssetClass {
    Native,
    Constitutional,
    System,
    Treasury,
    Governance,
    Utility,
    Synthetic,
    Wrapped,
    Experimental,
}

/// Issuance model declared for the asset.
///
/// Security rationale:
/// Supply semantics are protocol-critical. The supply model must remain
/// consistent with mint authority, optional supply caps, and asset class.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SupplyModel {
    FixedGenesis,
    TreasuryAuthorizedEmission,
    GovernanceAuthorizedEmission,
    ProgrammaticEmission,
    WrappedBacked,
}

/// Authorized minter classification.
///
/// Security rationale:
/// Mint authority defines the authorization boundary for supply expansion and
/// therefore must match the declared supply model and class-level policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MintAuthority {
    ProtocolOnly,
    Treasury,
    Governance,
    Bridge,
}

/// Registry lifecycle state.
///
/// Security rationale:
/// Registry status is an explicit state machine, not a free-form label.
/// Invalid or reversible transitions can cause governance confusion and
/// operational risk.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RegistryStatus {
    Proposed,
    Registered,
    Active,
    Frozen,
    Deprecated,
    Revoked,
}

/// Coarse risk signal used for operator visibility and downstream policy.
///
/// Security rationale:
/// Risk grade does not replace authorization or economic review, but it
/// remains useful for policy gating and should be consistent with class-level
/// expectations where defined.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RiskGrade {
    Low,
    Medium,
    High,
    Critical,
}
