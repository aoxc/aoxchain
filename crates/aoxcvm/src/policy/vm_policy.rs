use crate::feature_flags::FeatureFlags;
use crate::limits::ExecutionLimits;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VmPolicy {
    pub protocol_version: u32,
    pub limits: ExecutionLimits,
    pub features: FeatureFlags,
}
