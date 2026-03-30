// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContractCapability {
    StorageRead,
    StorageWrite,
    ExternalCall,
    NativeTokenTouch,
    CrossLaneInvoke,
    PrivilegedHook,
    GovernanceBound,
    TreasurySensitive,
}
