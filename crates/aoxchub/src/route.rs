use dioxus::prelude::*;

use crate::layouts::AdminLayout;
use crate::views::{ConsensusMap, Home, LaneMonitor, Nodes, Wallet, ZkpAudit};

#[derive(Routable, Clone, PartialEq, Debug)]
#[rustfmt::skip]
pub enum Route {
    #[layout(AdminLayout)]
        #[route("/")]
        Home {},
        #[route("/lane-monitor")]
        LaneMonitor {},
        #[route("/consensus-map")]
        ConsensusMap {},
        #[route("/zkp-audit")]
        ZkpAudit {},
        #[route("/wallet")]
        Wallet {},
        #[route("/nodes")]
        Nodes {},
    #[end_layout]
    #[route("/:..segments")]
    NotFound { segments: Vec<String> },
}
