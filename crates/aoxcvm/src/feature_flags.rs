#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct FeatureFlags {
    pub pq_auth_primary: bool,
    pub authority_metering: bool,
    pub deterministic_host_v2: bool,
}
