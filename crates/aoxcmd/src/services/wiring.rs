use super::registry::ServiceRegistry;

#[must_use]
pub fn wire_defaults() -> ServiceRegistry {
    let mut registry = ServiceRegistry::new();
    registry.register("node-runtime");
    registry.register("key-manager");
    registry.register("consensus-engine");
    registry
}
