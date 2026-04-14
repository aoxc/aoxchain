//! Virtual machine profile selection and quantum-hardening policy.

/// Known VM runtime families exposed by AOXC-VMachine-QX1.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum VmFlavor {
    /// Baseline deterministic VM profile.
    #[default]
    Deterministic,
    /// Advanced deterministic profile with stronger guardrails.
    AdvancedDeterministic,
    /// Quantum-resistant profile requiring post-quantum controls.
    QuantumResistant,
}

/// Signature policy required for VM admission.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SignaturePolicy {
    /// Require ML-DSA attestations on privileged operations.
    pub require_ml_dsa: bool,
    /// Require SLH-DSA attestations on privileged operations.
    pub require_slh_dsa: bool,
    /// Require a hybrid (classical + PQ) authorization bundle.
    pub require_hybrid_bundle: bool,
}

impl SignaturePolicy {
    /// Classical-only policy for compatibility mode.
    pub const fn classical_compat() -> Self {
        Self {
            require_ml_dsa: false,
            require_slh_dsa: false,
            require_hybrid_bundle: false,
        }
    }

    /// Strongest currently supported post-quantum policy.
    pub const fn quantum_hardened() -> Self {
        Self {
            require_ml_dsa: true,
            require_slh_dsa: true,
            require_hybrid_bundle: true,
        }
    }
}

impl Default for SignaturePolicy {
    fn default() -> Self {
        Self::classical_compat()
    }
}

/// Declarative plan for constructing a VM with explicit security posture.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VmConstructionPlan {
    pub flavor: VmFlavor,
    pub signature_policy: SignaturePolicy,
    /// Enables deterministic anti-replay constraints at VM boundary.
    pub enforce_replay_window: bool,
}

impl VmConstructionPlan {
    /// Baseline deterministic plan.
    pub const fn deterministic() -> Self {
        Self {
            flavor: VmFlavor::Deterministic,
            signature_policy: SignaturePolicy::classical_compat(),
            enforce_replay_window: false,
        }
    }

    /// Advanced VM plan with stronger deterministic controls.
    pub const fn advanced() -> Self {
        Self {
            flavor: VmFlavor::AdvancedDeterministic,
            signature_policy: SignaturePolicy::classical_compat(),
            enforce_replay_window: true,
        }
    }

    /// Quantum-resistant VM plan with post-quantum controls enabled.
    pub const fn quantum_resistant() -> Self {
        Self {
            flavor: VmFlavor::QuantumResistant,
            signature_policy: SignaturePolicy::quantum_hardened(),
            enforce_replay_window: true,
        }
    }

    /// Validates that the selected flavor and policy are coherent.
    pub const fn validate(self) -> Result<Self, VmProfileError> {
        match self.flavor {
            VmFlavor::QuantumResistant => {
                if !self.signature_policy.require_ml_dsa {
                    return Err(VmProfileError::MissingMlDsa);
                }
                if !self.signature_policy.require_slh_dsa {
                    return Err(VmProfileError::MissingSlhDsa);
                }
                if !self.signature_policy.require_hybrid_bundle {
                    return Err(VmProfileError::MissingHybridBundle);
                }
                if !self.enforce_replay_window {
                    return Err(VmProfileError::MissingReplayWindow);
                }
            }
            VmFlavor::AdvancedDeterministic => {
                if !self.enforce_replay_window {
                    return Err(VmProfileError::MissingReplayWindow);
                }
            }
            VmFlavor::Deterministic => {}
        }
        Ok(self)
    }
}

impl Default for VmConstructionPlan {
    fn default() -> Self {
        Self::deterministic()
    }
}

/// Profile validation errors for VM construction plans.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VmProfileError {
    MissingMlDsa,
    MissingSlhDsa,
    MissingHybridBundle,
    MissingReplayWindow,
}

#[cfg(test)]
mod tests {
    use super::{SignaturePolicy, VmConstructionPlan, VmFlavor, VmProfileError};

    #[test]
    fn deterministic_profile_is_valid_by_default() {
        let plan = VmConstructionPlan::default().validate().expect("valid");
        assert_eq!(plan.flavor, VmFlavor::Deterministic);
    }

    #[test]
    fn advanced_profile_requires_replay_window() {
        let invalid = VmConstructionPlan {
            flavor: VmFlavor::AdvancedDeterministic,
            signature_policy: SignaturePolicy::classical_compat(),
            enforce_replay_window: false,
        };
        assert_eq!(invalid.validate(), Err(VmProfileError::MissingReplayWindow));
    }

    #[test]
    fn quantum_profile_requires_all_pq_controls() {
        let invalid = VmConstructionPlan {
            flavor: VmFlavor::QuantumResistant,
            signature_policy: SignaturePolicy {
                require_ml_dsa: true,
                require_slh_dsa: false,
                require_hybrid_bundle: true,
            },
            enforce_replay_window: true,
        };

        assert_eq!(invalid.validate(), Err(VmProfileError::MissingSlhDsa));
        assert!(VmConstructionPlan::quantum_resistant().validate().is_ok());
    }
}
