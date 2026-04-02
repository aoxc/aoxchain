//! Package publication and promotion law runtime checks.

use crate::{
    errors::{AoxcvmError, AoxcvmResult},
    package::dependencies::DependencyGraph,
    policy::{
        execution::{CapabilityContext, RuntimeCapability, RuntimeCapabilityGate},
        governance::{GovernanceAction, GovernanceAuthority},
    },
    vm::admission::ActiveAuthProfile,
};

/// Publication stage in package lifecycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackageLifecycleStage {
    Draft,
    Published,
    Promoted,
    Deprecated,
}

/// Immutable publication request surface.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackagePublicationRequest {
    pub package_id: String,
    pub trust_domain: String,
    pub required_auth_profile: Option<String>,
    pub dependencies: DependencyGraph,
    pub stage: PackageLifecycleStage,
}

/// Runtime package law enforcer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PackagePublicationLaw {
    gate: RuntimeCapabilityGate,
}

impl PackagePublicationLaw {
    /// Admission and runtime checks for package publication/promotion.
    pub fn authorize(
        self,
        authority: GovernanceAuthority,
        request: &PackagePublicationRequest,
        active_profile: Option<&ActiveAuthProfile>,
    ) -> AoxcvmResult<()> {
        authority.authorize(GovernanceAction::MutateRegistry)?;

        if request.trust_domain.trim().is_empty() {
            return Err(AoxcvmError::PolicyViolation(
                "package trust domain must be specified",
            ));
        }

        if request.dependencies.has_cycle() {
            return Err(AoxcvmError::PolicyViolation(
                "package dependency graph contains cycle",
            ));
        }

        if matches!(request.stage, PackageLifecycleStage::Promoted)
            && !matches!(
                authority.lane,
                crate::policy::governance::GovernanceLane::Constitutional
            )
        {
            return Err(AoxcvmError::GovernanceLaneViolation(
                "package promotion requires constitutional lane",
            ));
        }

        if let Some(required_profile) = request.required_auth_profile.as_deref() {
            let active = active_profile.ok_or(AoxcvmError::PolicyViolation(
                "package publication requires active auth profile",
            ))?;
            if active.profile_name != required_profile {
                return Err(AoxcvmError::PolicyViolation(
                    "package publication auth profile mismatch",
                ));
            }
        }

        self.gate.enforce(
            RuntimeCapability::RegistryMutation,
            CapabilityContext {
                signer_class: authority.signer_class,
                lane: authority.lane,
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        auth::{registry::AuthProfileId, signer::SignerClass},
        package::{
            dependencies::DependencyGraph,
            publish::{PackageLifecycleStage, PackagePublicationLaw, PackagePublicationRequest},
        },
        policy::governance::{GovernanceAuthority, GovernanceLane},
        vm::admission::ActiveAuthProfile,
    };

    fn authority() -> GovernanceAuthority {
        GovernanceAuthority {
            signer_class: SignerClass::Governance,
            lane: GovernanceLane::Constitutional,
        }
    }

    fn profile() -> ActiveAuthProfile {
        ActiveAuthProfile {
            profile_id: AuthProfileId::new(3),
            profile_version: 1,
            profile_name: "package-v1".to_string(),
            signer_class: SignerClass::Governance,
        }
    }

    #[test]
    fn package_publication_requires_acyclic_graph() {
        let law = PackagePublicationLaw::default();
        let mut graph = DependencyGraph::default();
        graph.add_dependency("pkg.a".to_string(), "pkg.b".to_string());
        graph.add_dependency("pkg.b".to_string(), "pkg.a".to_string());

        let request = PackagePublicationRequest {
            package_id: "pkg.a".to_string(),
            trust_domain: "core".to_string(),
            required_auth_profile: Some("package-v1".to_string()),
            dependencies: graph,
            stage: PackageLifecycleStage::Published,
        };

        assert!(
            law.authorize(authority(), &request, Some(&profile()))
                .is_err()
        );
    }

    #[test]
    fn package_promotion_requires_constitutional_lane() {
        let law = PackagePublicationLaw::default();
        let authority = GovernanceAuthority {
            signer_class: SignerClass::Operations,
            lane: GovernanceLane::Operations,
        };

        let request = PackagePublicationRequest {
            package_id: "pkg.a".to_string(),
            trust_domain: "core".to_string(),
            required_auth_profile: None,
            dependencies: DependencyGraph::default(),
            stage: PackageLifecycleStage::Promoted,
        };

        assert!(law.authorize(authority, &request, None).is_err());
    }
}
