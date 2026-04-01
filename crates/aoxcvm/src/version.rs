#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VmVersion {
    pub major: u16,
    pub minor: u16,
    pub patch: u16,
}

impl VmVersion {
    pub const V1: Self = Self { major: 1, minor: 0, patch: 0 };
}
