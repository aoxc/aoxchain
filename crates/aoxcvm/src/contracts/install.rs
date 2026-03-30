// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use serde::{Deserialize, Serialize};

use aoxcontract::ContractDescriptor;

use crate::contracts::binding::VmLaneBinding;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LaneInstallSpec {
    pub lane: VmLaneBinding,
    pub artifact_location: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InstallableContract {
    pub descriptor: ContractDescriptor,
    pub install_spec: LaneInstallSpec,
}
