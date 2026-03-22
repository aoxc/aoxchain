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
