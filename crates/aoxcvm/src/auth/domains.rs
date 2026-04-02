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
}

impl AuthDomain {
    /// Stable wire identifier.
    pub const fn wire_id(self) -> &'static str {
        match self {
            Self::Transaction => "tx",
            Self::Governance => "governance",
            Self::Package => "package",
            Self::Upgrade => "upgrade",
        }
    }

    /// Parses a known domain identifier.
    pub fn parse(wire: &str) -> Option<Self> {
        match wire {
            "tx" => Some(Self::Transaction),
            "governance" => Some(Self::Governance),
            "package" => Some(Self::Package),
            "upgrade" => Some(Self::Upgrade),
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
        ] {
            assert_eq!(AuthDomain::parse(domain.wire_id()), Some(domain));
        }
    }
}
