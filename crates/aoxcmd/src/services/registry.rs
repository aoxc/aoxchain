// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use serde::Serialize;

const SERVICE_CONFIG: ServiceDescriptor =
    ServiceDescriptor::new("config", "Operator configuration resolution and validation");
const SERVICE_KEYS: ServiceDescriptor =
    ServiceDescriptor::new("keys", "Identity material bootstrap and verification");
const SERVICE_NODE: ServiceDescriptor =
    ServiceDescriptor::new("node", "Local runtime state and block production lifecycle");
const SERVICE_ECONOMY: ServiceDescriptor =
    ServiceDescriptor::new("economy", "Treasury and delegation state management");
const SERVICE_TELEMETRY: ServiceDescriptor =
    ServiceDescriptor::new("telemetry", "Metrics and operator diagnostics capture");

/// Canonical AOXC service descriptor published by operator-facing registry
/// surfaces.
///
/// Design intent:
/// - Preserve a stable machine-readable service catalog for runtime inspection,
///   build manifests, and operator diagnostics.
/// - Keep service identity compact and explicit.
/// - Avoid dynamic or environment-dependent naming in the default registry so
///   downstream tooling can rely on a deterministic service contract.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ServiceDescriptor {
    pub name: &'static str,
    pub purpose: &'static str,
}

impl ServiceDescriptor {
    /// Constructs a canonical service descriptor from explicit logical labels.
    pub const fn new(name: &'static str, purpose: &'static str) -> Self {
        Self { name, purpose }
    }

    /// Returns `true` when both descriptor fields are present and non-empty.
    ///
    /// This helper is primarily intended for defensive integrity checks in test
    /// and manifest-generation surfaces.
    pub fn is_complete(&self) -> bool {
        !self.name.is_empty() && !self.purpose.is_empty()
    }
}

/// Returns the canonical AOXC default service registry.
///
/// Registry policy:
/// - Service names are deterministic and stable.
/// - The default registry enumerates the major operator-plane surfaces that are
///   expected to exist in every AOXC command build.
/// - Ordering is intentional and should remain stable unless the public service
///   contract changes.
pub fn default_registry() -> Vec<ServiceDescriptor> {
    vec![
        SERVICE_CONFIG,
        SERVICE_KEYS,
        SERVICE_NODE,
        SERVICE_ECONOMY,
        SERVICE_TELEMETRY,
    ]
}

/// Returns `true` when the supplied service registry is structurally valid.
///
/// Validation policy:
/// - Every entry must be complete.
/// - Service names must be unique.
pub fn registry_is_valid(registry: &[ServiceDescriptor]) -> bool {
    registry.iter().all(ServiceDescriptor::is_complete) && has_unique_names(registry)
}

fn has_unique_names(registry: &[ServiceDescriptor]) -> bool {
    let mut names = std::collections::BTreeSet::new();
    registry.iter().all(|service| names.insert(service.name))
}

#[cfg(test)]
mod tests {
    use super::{
        default_registry, registry_is_valid, ServiceDescriptor, SERVICE_CONFIG, SERVICE_ECONOMY,
        SERVICE_KEYS, SERVICE_NODE, SERVICE_TELEMETRY,
    };

    #[test]
    fn default_registry_returns_canonical_operator_plane_services() {
        let registry = default_registry();

        assert_eq!(
            registry,
            vec![
                SERVICE_CONFIG,
                SERVICE_KEYS,
                SERVICE_NODE,
                SERVICE_ECONOMY,
                SERVICE_TELEMETRY,
            ]
        );
    }

    #[test]
    fn service_descriptor_new_preserves_supplied_labels() {
        let descriptor = ServiceDescriptor::new("runtime", "Runtime state inspection");

        assert_eq!(descriptor.name, "runtime");
        assert_eq!(descriptor.purpose, "Runtime state inspection");
    }

    #[test]
    fn service_descriptor_reports_completeness_for_non_empty_fields() {
        let descriptor = ServiceDescriptor::new("config", "Configuration");

        assert!(descriptor.is_complete());
    }

    #[test]
    fn registry_validation_accepts_default_registry() {
        let registry = default_registry();

        assert!(registry_is_valid(&registry));
    }

    #[test]
    fn registry_validation_rejects_duplicate_names() {
        let registry = vec![
            ServiceDescriptor::new("config", "Configuration"),
            ServiceDescriptor::new("config", "Duplicate configuration entry"),
        ];

        assert!(!registry_is_valid(&registry));
    }

    #[test]
    fn registry_validation_rejects_blank_fields() {
        let registry = vec![ServiceDescriptor::new("", "Missing name")];

        assert!(!registry_is_valid(&registry));
    }
}
