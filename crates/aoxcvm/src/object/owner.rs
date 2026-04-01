#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Owner {
    Identity([u8; 32]),
    Shared,
    Governance,
}
