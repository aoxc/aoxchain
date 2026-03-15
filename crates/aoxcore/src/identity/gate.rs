use std::collections::{HashMap, HashSet};

/// Canonical role identifiers used by the AOXC identity layer.
pub const ROLE_VALIDATOR: &str = "VAL";
pub const ROLE_NODE: &str = "NOD";
pub const ROLE_ORACLE: &str = "AOR";
pub const ROLE_GOVERNANCE: &str = "GOV";

/// Canonical module identifiers.
pub const MODULE_CONSENSUS: &str = "consensus";
pub const MODULE_NETWORK: &str = "network";
pub const MODULE_ORACLE: &str = "oracle";
pub const MODULE_GOVERNANCE: &str = "governance";

/// Access-control gate for AOXC modules.
///
/// The gate enforces which actor roles are permitted to interact
/// with which internal modules.
#[derive(Debug, Clone)]
pub struct Gate {
    permissions: HashMap<String, HashSet<String>>,
}

impl Default for Gate {
    fn default() -> Self {
        Self::new()
    }
}

impl Gate {
    /// Constructs a new gate with default AOXC permissions.
    #[must_use]
    pub fn new() -> Self {
        let mut permissions: HashMap<String, HashSet<String>> = HashMap::new();

        permissions.insert(
            ROLE_VALIDATOR.into(),
            HashSet::from([MODULE_CONSENSUS.into()]),
        );

        permissions.insert(ROLE_NODE.into(), HashSet::from([MODULE_NETWORK.into()]));

        permissions.insert(ROLE_ORACLE.into(), HashSet::from([MODULE_ORACLE.into()]));

        permissions.insert(
            ROLE_GOVERNANCE.into(),
            HashSet::from([MODULE_GOVERNANCE.into()]),
        );

        Self { permissions }
    }

    /// Returns true if the specified role is allowed to access the module.
    #[must_use]
    pub fn allow(&self, role: &str, module: &str) -> bool {
        let role = normalize(role);
        let module = normalize(module);

        match self.permissions.get(&role) {
            Some(modules) => modules.contains(&module),
            None => false,
        }
    }

    /// Grants a permission for a role to access a module.
    pub fn grant(&mut self, role: &str, module: &str) {
        let role = normalize(role);
        let module = normalize(module);

        self.permissions.entry(role).or_default().insert(module);
    }

    /// Revokes a permission for a role to access a module.
    pub fn revoke(&mut self, role: &str, module: &str) {
        let role = normalize(role);
        let module = normalize(module);

        if let Some(modules) = self.permissions.get_mut(&role) {
            modules.remove(&module);
        }
    }

    /// Returns all modules accessible by the specified role.
    #[must_use]
    pub fn modules_for_role(&self, role: &str) -> Vec<String> {
        let role = normalize(role);

        match self.permissions.get(&role) {
            Some(modules) => modules.iter().cloned().collect(),
            None => Vec::new(),
        }
    }

    /// Returns all roles known by the gate.
    #[must_use]
    pub fn roles(&self) -> Vec<String> {
        self.permissions.keys().cloned().collect()
    }

    /// Returns true if the role exists in the gate.
    #[must_use]
    pub fn has_role(&self, role: &str) -> bool {
        let role = normalize(role);
        self.permissions.contains_key(&role)
    }

    /// Clears all permissions for a role.
    pub fn clear_role(&mut self, role: &str) {
        let role = normalize(role);
        self.permissions.remove(&role);
    }

    /// Returns the number of registered roles.
    #[must_use]
    pub fn role_count(&self) -> usize {
        self.permissions.len()
    }
}

/// Normalizes role and module identifiers.
fn normalize(value: &str) -> String {
    value
        .trim()
        .to_ascii_uppercase()
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_permissions_work() {
        let gate = Gate::new();

        assert!(gate.allow("VAL", "consensus"));
        assert!(gate.allow("NOD", "network"));
        assert!(gate.allow("AOR", "oracle"));
        assert!(gate.allow("GOV", "governance"));
    }

    #[test]
    fn deny_invalid_role() {
        let gate = Gate::new();
        assert!(!gate.allow("HAX", "consensus"));
    }

    #[test]
    fn grant_and_revoke_work() {
        let mut gate = Gate::new();

        gate.grant("VAL", "network");
        assert!(gate.allow("VAL", "network"));

        gate.revoke("VAL", "network");
        assert!(!gate.allow("VAL", "network"));
    }

    #[test]
    fn role_listing_works() {
        let gate = Gate::new();

        let roles = gate.roles();
        assert!(roles.contains(&"VAL".to_string()));
    }
}
