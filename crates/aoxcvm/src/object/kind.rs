#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObjectKind {
    Identity,
    Asset,
    Capability,
    Contract,
    Package,
    Vault,
    Governance,
}
