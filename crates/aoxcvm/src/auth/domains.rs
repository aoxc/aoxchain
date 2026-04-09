//! Deterministic authentication domain identifiers.

/// Canonical domain for auth envelopes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AuthDomain {
    /// Transaction submission and execution entry.
    Transaction,
    /// Governance action and constitutional transitions.
    Governance,
    /// Package publication/promotion actions.
    Package,
    /// Upgrade lane actions.
    Upgrade,
    /// Constitutional recovery actions governed by constitutional signers.
    ConstitutionalRecovery,
}

impl AuthDomain {
    /// Stable wire identifier.
    pub const fn wire_id(self) -> &'static str {
        match self {
            Self::Transaction => "tx",
            Self::Governance => "governance",
            Self::Package => "package",
            Self::Upgrade => "upgrade",
            Self::ConstitutionalRecovery => "constitutional-recovery",
        }
    }

    /// Canonical domain-separation tag used for signed payload hashing.
    pub const fn canonical_tag(self) -> &'static str {
        match self {
            Self::Transaction => "AOX/TX/V1",
            Self::Governance => "AOX/GOVERNANCE/V1",
            Self::Package => "AOX/PACKAGE/V1",
            Self::Upgrade => "AOX/UPGRADE/V1",
            Self::ConstitutionalRecovery => "AOX/CONS_RECOVERY/V1",
        }
    }

    /// Parses a known domain identifier.
    pub fn parse(wire: &str) -> Option<Self> {
        match wire {
            "tx" => Some(Self::Transaction),
            "governance" => Some(Self::Governance),
            "package" => Some(Self::Package),
            "upgrade" => Some(Self::Upgrade),
            "constitutional-recovery" => Some(Self::ConstitutionalRecovery),
            "AOX/TX/V1" => Some(Self::Transaction),
            "AOX/GOVERNANCE/V1" => Some(Self::Governance),
            "AOX/PACKAGE/V1" => Some(Self::Package),
            "AOX/UPGRADE/V1" => Some(Self::Upgrade),
            "AOX/CONS_RECOVERY/V1" => Some(Self::ConstitutionalRecovery),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::AuthDomain;

    #[test]
    fn parse_roundtrip() {
        for domain in [
            AuthDomain::Transaction,
            AuthDomain::Governance,
            AuthDomain::Package,
            AuthDomain::Upgrade,
            AuthDomain::ConstitutionalRecovery,
        ] {
            assert_eq!(AuthDomain::parse(domain.wire_id()), Some(domain));
            assert_eq!(AuthDomain::parse(domain.canonical_tag()), Some(domain));
        }
    }
}
