use dioxus::prelude::*;

use crate::layouts::AdminLayout;
use crate::views::{
    ConsensusMap, Explorer, Home, LaneMonitor, Nodes, NotFoundPage, Staking, Wallet, ZkpAudit,
};

/// Defines the routing topology for the AOXCHUB control plane.
///
/// Architectural Rationale:
/// - All primary operational surfaces are grouped under a shared administrative
///   layout in order to preserve navigational consistency and visual continuity.
/// - Each route variant maps directly to a single view component, which keeps
///   the routing graph explicit, predictable, and straightforward to audit.
/// - A terminal catch-all route is retained to ensure unmatched paths degrade
///   into a deterministic not-found experience rather than undefined behavior.
///
/// Implementation Note:
/// - The route variant name intentionally matches the underlying component name
///   for the terminal fallback page. This avoids macro-level symbol resolution
///   ambiguity in the Dioxus `Routable` derive flow.
#[derive(Routable, Clone, PartialEq, Debug)]
#[rustfmt::skip]
pub enum Route {
    #[layout(AdminLayout)]
        /// Primary operational landing surface.
        ///
        /// Exposes high-level chain visibility, validator posture, and execution
        /// lane telemetry through the main overview dashboard.
        #[route("/")]
        Home {},

        /// Execution lane monitoring surface.
        ///
        /// Presents runtime-specific throughput, load characteristics, and
        /// checkpoint continuity across supported lanes.
        #[route("/lane-monitor")]
        LaneMonitor {},

        /// Consensus topology visibility surface.
        ///
        /// Provides regional placement, validator distribution, and latency
        /// observation for consensus participants.
        #[route("/consensus-map")]
        ConsensusMap {},

        /// Zero-knowledge audit surface.
        ///
        /// Centralizes proof-verification visibility and integrity-oriented
        /// operational review workflows.
        #[route("/zkp-audit")]
        ZkpAudit {},

        /// Wallet and treasury operations surface.
        ///
        /// Provides visibility into operational custody and treasury-oriented
        /// fund management views.
        #[route("/wallet")]
        Wallet {},

        /// Explorer operations surface.
        ///
        /// Provides chain-level search entry points and latest finalized
        /// summaries for operators who need block and transaction visibility.
        #[route("/explorer")]
        Explorer {},

        /// Staking and validator economics surface.
        ///
        /// Exposes delegation, validator weight, and reward posture metrics
        /// derived from the authoritative in-memory chain state.
        #[route("/staking")]
        Staking {},

        /// Validator inventory surface.
        ///
        /// Enables node-level inspection of current validator status, regional
        /// placement, and latency posture.
        #[route("/nodes")]
        Nodes {},
    #[end_layout]

    /// Catch-all route for unmatched paths.
    ///
    /// Any unresolved route is forwarded to the dedicated not-found page while
    /// preserving the captured path segments for diagnostics and user feedback.
    #[route("/:..segments")]
    NotFoundPage { segments: Vec<String> },
}
