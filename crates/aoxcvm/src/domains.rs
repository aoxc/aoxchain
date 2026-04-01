/// Canonical execution domains used for replay separation and policy scoping.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Domain {
    L1Transaction,
    Session,
    Governance,
    PackagePublish,
}
