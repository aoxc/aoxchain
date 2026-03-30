// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

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

/// Maximum accepted canonical module identifier length.
///
/// This bound is intentionally conservative. It is sufficient for operator and
/// internal module naming while preventing unbounded identifiers from entering
/// the access-control surface.
pub const MAX_MODULE_IDENTIFIER_LEN: usize = 64;

/// Canonical role code length.
///
/// Current AOXC policy uses fixed-width 3-character role codes.
pub const ROLE_CODE_LEN: usize = 3;

/// Access-control errors for the AOXC gate surface.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum GateError {
    EmptyRole,
    InvalidRole,
    EmptyModule,
    InvalidModule,
    PermissionDenied,
    PolicyInvalid,
}

impl GateError {
    /// Returns a stable symbolic error code suitable for logs and telemetry.
    #[must_use]
    pub const fn code(&self) -> &'static str {
        match self {
            Self::EmptyRole => "GATE_EMPTY_ROLE",
            Self::InvalidRole => "GATE_INVALID_ROLE",
            Self::EmptyModule => "GATE_EMPTY_MODULE",
            Self::InvalidModule => "GATE_INVALID_MODULE",
            Self::PermissionDenied => "GATE_PERMISSION_DENIED",
            Self::PolicyInvalid => "GATE_POLICY_INVALID",
        }
    }
}

impl fmt::Display for GateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyRole => write!(f, "gate validation failed: role must not be empty"),
            Self::InvalidRole => write!(f, "gate validation failed: role is not canonical"),
            Self::EmptyModule => write!(f, "gate validation failed: module must not be empty"),
            Self::InvalidModule => write!(f, "gate validation failed: module is not canonical"),
            Self::PermissionDenied => write!(f, "gate access denied for role and module"),
            Self::PolicyInvalid => write!(f, "gate policy validation failed"),
        }
    }
}

impl std::error::Error for GateError {}

/// Access-control gate for AOXC modules.
///
/// The gate enforces which actor roles are permitted to interact with which
/// internal modules.
///
/// Design properties:
/// - deterministic iteration order,
/// - strict canonicalization,
/// - no silent stripping of invalid characters,
/// - explicit role alias mapping,
/// - independent policy validation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Gate {
    permissions: BTreeMap<String, BTreeSet<String>>,
}

impl Default for Gate {
    fn default() -> Self {
        Self::new()
    }
}

impl Gate {
    /// Constructs a new gate with canonical AOXC default permissions.
    #[must_use]
    pub fn new() -> Self {
        let mut permissions: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();

        insert_permission_unchecked(&mut permissions, ROLE_VALIDATOR, MODULE_CONSENSUS);
        insert_permission_unchecked(&mut permissions, ROLE_NODE, MODULE_NETWORK);
        insert_permission_unchecked(&mut permissions, ROLE_ORACLE, MODULE_ORACLE);
        insert_permission_unchecked(&mut permissions, ROLE_GOVERNANCE, MODULE_GOVERNANCE);

        let gate = Self { permissions };
        debug_assert!(gate.validate().is_ok());

        gate
    }

    /// Validates the internal gate policy state.
    ///
    /// Validation policy:
    /// - every stored role must be canonical,
    /// - every stored module must be canonical,
    /// - each registered role must retain at least one module permission.
    pub fn validate(&self) -> Result<(), GateError> {
        for (role, modules) in &self.permissions {
            canonicalize_role(role)?;

            if modules.is_empty() {
                return Err(GateError::PolicyInvalid);
            }

            for module in modules {
                canonicalize_module(module)?;
            }
        }

        Ok(())
    }

    /// Returns true if the specified role is allowed to access the module.
    ///
    /// Compatibility behavior:
    /// - invalid input does not panic,
    /// - invalid input resolves to `false`,
    /// - callers that need error detail should use `try_allow`.
    #[must_use]
    pub fn allow(&self, role: &str, module: &str) -> bool {
        self.try_allow(role, module).unwrap_or(false)
    }

    /// Returns whether the specified role is allowed to access the module.
    ///
    /// This is the strict variant of `allow`.
    pub fn try_allow(&self, role: &str, module: &str) -> Result<bool, GateError> {
        let canonical_role = canonicalize_role(role)?;
        let canonical_module = canonicalize_module(module)?;

        Ok(self
            .permissions
            .get(&canonical_role)
            .is_some_and(|modules| modules.contains(&canonical_module)))
    }

    /// Requires that the specified role is permitted to access the module.
    ///
    /// This helper is suitable for call paths where permission denial must be
    /// surfaced explicitly rather than collapsed into a boolean.
    pub fn require_allowed(&self, role: &str, module: &str) -> Result<(), GateError> {
        if self.try_allow(role, module)? {
            Ok(())
        } else {
            Err(GateError::PermissionDenied)
        }
    }

    /// Grants a permission for a role to access a module.
    ///
    /// Validation policy:
    /// - role must canonicalize successfully,
    /// - module must canonicalize successfully.
    pub fn grant(&mut self, role: &str, module: &str) -> Result<(), GateError> {
        let canonical_role = canonicalize_role(role)?;
        let canonical_module = canonicalize_module(module)?;

        self.permissions
            .entry(canonical_role)
            .or_default()
            .insert(canonical_module);

        Ok(())
    }

    /// Revokes a permission for a role to access a module.
    ///
    /// Return value:
    /// - `true` if a permission existed and was removed,
    /// - `false` if the permission was not present.
    ///
    /// Cleanup policy:
    /// - empty role entries are removed after revocation.
    pub fn revoke(&mut self, role: &str, module: &str) -> Result<bool, GateError> {
        let canonical_role = canonicalize_role(role)?;
        let canonical_module = canonicalize_module(module)?;

        let Some(modules) = self.permissions.get_mut(&canonical_role) else {
            return Ok(false);
        };

        let removed = modules.remove(&canonical_module);

        if modules.is_empty() {
            self.permissions.remove(&canonical_role);
        }

        Ok(removed)
    }

    /// Returns all modules accessible by the specified role.
    ///
    /// Compatibility behavior:
    /// - invalid role input resolves to an empty vector,
    /// - callers that need error detail should use `try_modules_for_role`.
    #[must_use]
    pub fn modules_for_role(&self, role: &str) -> Vec<String> {
        self.try_modules_for_role(role).unwrap_or_default()
    }

    /// Returns all modules accessible by the specified role.
    ///
    /// The returned vector is deterministic because the internal set is ordered.
    pub fn try_modules_for_role(&self, role: &str) -> Result<Vec<String>, GateError> {
        let canonical_role = canonicalize_role(role)?;

        Ok(self
            .permissions
            .get(&canonical_role)
            .map(|modules| modules.iter().cloned().collect())
            .unwrap_or_default())
    }

    /// Returns all roles known by the gate.
    ///
    /// The returned vector is deterministic because the internal map is ordered.
    #[must_use]
    pub fn roles(&self) -> Vec<String> {
        self.permissions.keys().cloned().collect()
    }

    /// Returns true if the role exists in the gate.
    ///
    /// Compatibility behavior:
    /// - invalid input resolves to `false`.
    #[must_use]
    pub fn has_role(&self, role: &str) -> bool {
        match canonicalize_role(role) {
            Ok(canonical_role) => self.permissions.contains_key(&canonical_role),
            Err(_) => false,
        }
    }

    /// Clears all permissions for a role.
    ///
    /// Return value:
    /// - `true` if the role existed and was removed,
    /// - `false` if the role did not exist.
    pub fn clear_role(&mut self, role: &str) -> Result<bool, GateError> {
        let canonical_role = canonicalize_role(role)?;
        Ok(self.permissions.remove(&canonical_role).is_some())
    }

    /// Returns the number of registered roles.
    #[must_use]
    pub fn role_count(&self) -> usize {
        self.permissions.len()
    }

    /// Returns the total number of permission bindings currently stored.
    #[must_use]
    pub fn permission_count(&self) -> usize {
        self.permissions.values().map(BTreeSet::len).sum()
    }
}

/// Inserts a known-good canonical permission without re-validating.
///
/// This helper is restricted to internal bootstrap code where the constants are
/// controlled by the crate and already satisfy canonical policy.
fn insert_permission_unchecked(
    permissions: &mut BTreeMap<String, BTreeSet<String>>,
    role: &str,
    module: &str,
) {
    permissions
        .entry(role.to_string())
        .or_default()
        .insert(module.to_ascii_uppercase());
}

/// Canonicalizes a role input into an AOXC role code.
///
/// Policy:
/// - canonical 3-character role codes are accepted,
/// - selected descriptive aliases are mapped explicitly,
/// - invalid characters are rejected rather than silently removed.
fn canonicalize_role(role: &str) -> Result<String, GateError> {
    if role.is_empty() || role.trim().is_empty() {
        return Err(GateError::EmptyRole);
    }

    if role != role.trim() {
        return Err(GateError::InvalidRole);
    }

    let normalized = role.to_ascii_lowercase();

    let canonical = match normalized.as_str() {
        "val" | "validator" => ROLE_VALIDATOR,
        "nod" | "node" => ROLE_NODE,
        "aor" | "oracle" => ROLE_ORACLE,
        "gov" | "governance" => ROLE_GOVERNANCE,
        _ => {
            if role.len() == ROLE_CODE_LEN && role.chars().all(|ch| ch.is_ascii_alphanumeric()) {
                return Ok(role.to_ascii_uppercase());
            }
            return Err(GateError::InvalidRole);
        }
    };

    Ok(canonical.to_string())
}

/// Canonicalizes a module identifier.
///
/// Policy:
/// - module identifiers must not be blank,
/// - surrounding whitespace is rejected rather than normalized,
/// - only ASCII alphanumeric characters plus `_`, `-`, and `.` are accepted,
/// - the returned canonical representation is uppercase.
fn canonicalize_module(module: &str) -> Result<String, GateError> {
    if module.is_empty() || module.trim().is_empty() {
        return Err(GateError::EmptyModule);
    }

    if module != module.trim() {
        return Err(GateError::InvalidModule);
    }

    if module.len() > MAX_MODULE_IDENTIFIER_LEN {
        return Err(GateError::InvalidModule);
    }

    if !module
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '-' || ch == '.')
    {
        return Err(GateError::InvalidModule);
    }

    Ok(module.to_ascii_uppercase())
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
    fn descriptive_role_aliases_are_supported() {
        let gate = Gate::new();

        assert!(gate.allow("validator", "consensus"));
        assert!(gate.allow("node", "network"));
        assert!(gate.allow("oracle", "oracle"));
        assert!(gate.allow("governance", "governance"));
    }

    #[test]
    fn deny_invalid_role() {
        let gate = Gate::new();
        assert!(!gate.allow("HAX", "consensus"));
    }

    #[test]
    fn try_allow_rejects_non_canonical_module_input() {
        let gate = Gate::new();

        let result = gate.try_allow("VAL", "consensus!");

        assert_eq!(result, Err(GateError::InvalidModule));
    }

    #[test]
    fn grant_and_revoke_work() {
        let mut gate = Gate::new();

        gate.grant("VAL", "network").expect("grant must succeed");
        assert!(gate.allow("VAL", "network"));

        let removed = gate.revoke("VAL", "network").expect("revoke must succeed");
        assert!(removed);
        assert!(!gate.allow("VAL", "network"));
    }

    #[test]
    fn revoke_returns_false_when_permission_does_not_exist() {
        let mut gate = Gate::new();

        let removed = gate.revoke("VAL", "oracle").expect("revoke must succeed");
        assert!(!removed);
    }

    #[test]
    fn role_listing_works_and_is_deterministic() {
        let gate = Gate::new();

        let roles = gate.roles();
        assert!(roles.contains(&"VAL".to_string()));
        assert_eq!(roles, vec!["AOR", "GOV", "NOD", "VAL"]);
    }

    #[test]
    fn modules_for_role_are_deterministic() {
        let mut gate = Gate::new();
        gate.grant("VAL", "network").expect("grant must succeed");
        gate.grant("VAL", "oracle").expect("grant must succeed");

        let modules = gate.modules_for_role("VAL");
        assert_eq!(modules, vec!["CONSENSUS", "NETWORK", "ORACLE"]);
    }

    #[test]
    fn clear_role_removes_role_entry() {
        let mut gate = Gate::new();

        let removed = gate.clear_role("VAL").expect("clear_role must succeed");
        assert!(removed);
        assert!(!gate.has_role("VAL"));
    }

    #[test]
    fn invalid_role_input_is_rejected_for_mutation_paths() {
        let mut gate = Gate::new();

        let result = gate.grant(" validator ", "consensus");
        assert_eq!(result, Err(GateError::InvalidRole));
    }

    #[test]
    fn gate_policy_validation_accepts_default_state() {
        let gate = Gate::new();
        assert!(gate.validate().is_ok());
    }

    #[test]
    fn gate_policy_validation_rejects_empty_module_set() {
        let mut permissions = BTreeMap::new();
        permissions.insert("VAL".to_string(), BTreeSet::new());

        let gate = Gate { permissions };

        assert_eq!(gate.validate(), Err(GateError::PolicyInvalid));
    }

    #[test]
    fn permission_count_reports_total_bindings() {
        let mut gate = Gate::new();
        assert_eq!(gate.permission_count(), 4);

        gate.grant("VAL", "network").expect("grant must succeed");
        assert_eq!(gate.permission_count(), 5);
    }

    #[test]
    fn require_allowed_returns_permission_denied_for_unknown_binding() {
        let gate = Gate::new();

        let result = gate.require_allowed("VAL", "network");
        assert_eq!(result, Err(GateError::PermissionDenied));
    }
}
