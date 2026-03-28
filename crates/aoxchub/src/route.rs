use dioxus::prelude::*;

use crate::layouts::AdminLayout;
use crate::views::{
    Consensus, ExecutionLanes, Explorer, GovernanceControl, NodesInfrastructure, NotFoundPage,
    Overview, SettingsSecurity, TelemetryAudit, ValidatorsStaking, WalletTreasury,
};

#[derive(Routable, Clone, PartialEq, Debug)]
#[rustfmt::skip]
pub enum Route {
    #[layout(AdminLayout)]
        #[route("/")]
        Overview {},

        #[route("/consensus")]
        Consensus {},

        #[route("/validators-staking")]
        ValidatorsStaking {},

        #[route("/execution-lanes")]
        ExecutionLanes {},

        #[route("/explorer")]
        Explorer {},

        #[route("/wallet-treasury")]
        WalletTreasury {},

        #[route("/nodes-infrastructure")]
        NodesInfrastructure {},

        #[route("/telemetry-audit")]
        TelemetryAudit {},

        #[route("/governance-control")]
        GovernanceControl {},

        #[route("/settings-security")]
        SettingsSecurity {},
    #[end_layout]

    #[route("/:..segments")]
    NotFoundPage { segments: Vec<String> },
}
