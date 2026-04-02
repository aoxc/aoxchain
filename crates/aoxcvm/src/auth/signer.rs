//! Signer classification and deterministic signer metadata surfaces.

/// Canonical signer classes used by constitutional auth paths.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum SignerClass {
    /// Chain/system authority keys.
    System,
    /// Constitutional and policy governance keys.
    Governance,
    /// Operational keys with bounded runtime powers.
    Operations,
    /// Standard application/user entry keys.
    Application,
}

impl SignerClass {
    /// Stable wire identifier used in receipts and governance snapshots.
    pub const fn wire_id(self) -> &'static str {
        match self {
            Self::System => "system",
            Self::Governance => "governance",
            Self::Operations => "operations",
            Self::Application => "application",
        }
    }

    /// Whether this signer class is considered privileged.
    pub const fn is_privileged(self) -> bool {
        matches!(self, Self::System | Self::Governance | Self::Operations)
    }
}

#[cfg(test)]
mod tests {
    use super::SignerClass;

    #[test]
    fn wire_ids_are_stable() {
        assert_eq!(SignerClass::System.wire_id(), "system");
        assert_eq!(SignerClass::Governance.wire_id(), "governance");
        assert_eq!(SignerClass::Operations.wire_id(), "operations");
        assert_eq!(SignerClass::Application.wire_id(), "application");
    }

    #[test]
    fn privileged_set_is_bounded() {
        assert!(SignerClass::System.is_privileged());
        assert!(SignerClass::Governance.is_privileged());
        assert!(SignerClass::Operations.is_privileged());
        assert!(!SignerClass::Application.is_privileged());
    }
}
