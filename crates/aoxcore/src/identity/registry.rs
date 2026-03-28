// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt;
use std::time::{SystemTime, UNIX_EPOCH};

/// Maximum accepted actor identifier length.
const MAX_ACTOR_ID_LEN: usize = 128;

/// Maximum accepted role length.
const MAX_ROLE_LEN: usize = 32;

/// Actor registry entry.
///
/// Stores minimal identity metadata required by the node runtime.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ActorEntry {
    pub actor_id: String,
    pub role: String,
    pub registered_at: u64,
}

/// Identity registry.
///
/// Maintains a deterministic mapping between actor identifiers and their roles.
///
/// Design properties:
/// - deterministic iteration order,
/// - strict actor and role validation,
/// - explicit duplicate/conflict handling,
/// - serialization-friendly shape for persistence and synchronization.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Registry {
    actors: BTreeMap<String, ActorEntry>,
}

/// Canonical registry error surface.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum RegistryError {
    EmptyActorId,
    InvalidActorId,
    EmptyRole,
    InvalidRole,
    InvalidRegisteredAt,
    ActorAlreadyRegisteredWithDifferentRole,
    TimeError,
    SerializationFailed(String),
    ParseFailed(String),
}

impl RegistryError {
    /// Returns a stable symbolic error code suitable for logs and telemetry.
    #[must_use]
    pub const fn code(&self) -> &'static str {
        match self {
            Self::EmptyActorId => "REGISTRY_EMPTY_ACTOR_ID",
            Self::InvalidActorId => "REGISTRY_INVALID_ACTOR_ID",
            Self::EmptyRole => "REGISTRY_EMPTY_ROLE",
            Self::InvalidRole => "REGISTRY_INVALID_ROLE",
            Self::InvalidRegisteredAt => "REGISTRY_INVALID_REGISTERED_AT",
            Self::ActorAlreadyRegisteredWithDifferentRole => {
                "REGISTRY_ACTOR_ALREADY_REGISTERED_WITH_DIFFERENT_ROLE"
            }
            Self::TimeError => "REGISTRY_TIME_ERROR",
            Self::SerializationFailed(_) => "REGISTRY_SERIALIZATION_FAILED",
            Self::ParseFailed(_) => "REGISTRY_PARSE_FAILED",
        }
    }
}

impl fmt::Display for RegistryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyActorId => {
                write!(f, "registry operation failed: actor_id must not be empty")
            }
            Self::InvalidActorId => {
                write!(f, "registry operation failed: actor_id is not canonical")
            }
            Self::EmptyRole => {
                write!(f, "registry operation failed: role must not be empty")
            }
            Self::InvalidRole => {
                write!(f, "registry operation failed: role is not canonical")
            }
            Self::InvalidRegisteredAt => {
                write!(f, "registry operation failed: registered_at is invalid")
            }
            Self::ActorAlreadyRegisteredWithDifferentRole => {
                write!(
                    f,
                    "registry operation failed: actor already exists with a different role"
                )
            }
            Self::TimeError => {
                write!(f, "registry operation failed: system time is invalid")
            }
            Self::SerializationFailed(error) => {
                write!(f, "registry serialization failed: {}", error)
            }
            Self::ParseFailed(error) => {
                write!(f, "registry parsing failed: {}", error)
            }
        }
    }
}

impl std::error::Error for RegistryError {}

impl Default for Registry {
    fn default() -> Self {
        Self::new()
    }
}

impl Registry {
    /// Creates a new empty registry.
    #[must_use]
    pub fn new() -> Self {
        Self {
            actors: BTreeMap::new(),
        }
    }

    /// Validates the internal registry state.
    ///
    /// Validation policy:
    /// - every actor_id must be canonical,
    /// - every role must be canonical,
    /// - registered_at must be non-zero,
    /// - map keys must match the embedded actor_id field.
    pub fn validate(&self) -> Result<(), RegistryError> {
        for (key, entry) in &self.actors {
            let canonical_actor_id = canonicalize_actor_id(&entry.actor_id)?;
            let canonical_role = canonicalize_role(&entry.role)?;

            if entry.registered_at == 0 {
                return Err(RegistryError::InvalidRegisteredAt);
            }

            if key != &canonical_actor_id {
                return Err(RegistryError::InvalidActorId);
            }

            if entry.role != canonical_role {
                return Err(RegistryError::InvalidRole);
            }
        }

        Ok(())
    }

    /// Registers an actor using the current system time.
    ///
    /// Return value:
    /// - `Ok(true)` if the actor was inserted,
    /// - `Ok(false)` if the actor already existed with the same role,
    /// - `Err(...)` if validation fails or the existing role conflicts.
    pub fn register(&mut self, actor_id: String, role: String) -> Result<bool, RegistryError> {
        let now = current_time()?;
        self.register_at(actor_id, role, now)
    }

    /// Registers an actor at an explicit timestamp.
    ///
    /// This helper is useful for:
    /// - deterministic tests,
    /// - replay/import flows,
    /// - controlled synchronization.
    pub fn register_at(
        &mut self,
        actor_id: String,
        role: String,
        registered_at: u64,
    ) -> Result<bool, RegistryError> {
        let canonical_actor_id = canonicalize_actor_id(&actor_id)?;
        let canonical_role = canonicalize_role(&role)?;

        if registered_at == 0 {
            return Err(RegistryError::InvalidRegisteredAt);
        }

        if let Some(existing) = self.actors.get(&canonical_actor_id) {
            if existing.role == canonical_role {
                return Ok(false);
            }

            return Err(RegistryError::ActorAlreadyRegisteredWithDifferentRole);
        }

        let entry = ActorEntry {
            actor_id: canonical_actor_id.clone(),
            role: canonical_role,
            registered_at,
        };

        self.actors.insert(canonical_actor_id, entry);
        Ok(true)
    }

    /// Returns the role associated with an actor.
    ///
    /// Compatibility behavior:
    /// - invalid actor identifiers resolve to `None`,
    /// - callers that need strict validation should canonicalize before lookup.
    #[must_use]
    pub fn role(&self, actor_id: &str) -> Option<&str> {
        let canonical_actor_id = canonicalize_actor_id(actor_id).ok()?;
        self.actors
            .get(&canonical_actor_id)
            .map(|entry| entry.role.as_str())
    }

    /// Returns full metadata for an actor.
    ///
    /// Compatibility behavior:
    /// - invalid actor identifiers resolve to `None`.
    #[must_use]
    pub fn get(&self, actor_id: &str) -> Option<&ActorEntry> {
        let canonical_actor_id = canonicalize_actor_id(actor_id).ok()?;
        self.actors.get(&canonical_actor_id)
    }

    /// Returns true if the actor exists.
    #[must_use]
    pub fn exists(&self, actor_id: &str) -> bool {
        self.get(actor_id).is_some()
    }

    /// Returns the number of registered actors.
    #[must_use]
    pub fn len(&self) -> usize {
        self.actors.len()
    }

    /// Returns true if the registry is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.actors.is_empty()
    }

    /// Deterministically exports actor identifiers.
    ///
    /// Useful for hashing or synchronization.
    #[must_use]
    pub fn export_actor_ids(&self) -> Vec<String> {
        self.actors.keys().cloned().collect()
    }

    /// Deterministically exports full actor entries.
    #[must_use]
    pub fn export_entries(&self) -> Vec<ActorEntry> {
        self.actors.values().cloned().collect()
    }

    /// Serializes the registry to pretty JSON.
    pub fn to_json(&self) -> Result<String, RegistryError> {
        self.validate()?;

        serde_json::to_string_pretty(self)
            .map_err(|error| RegistryError::SerializationFailed(error.to_string()))
    }

    /// Restores a registry from JSON and validates it.
    pub fn from_json(data: &str) -> Result<Self, RegistryError> {
        let registry: Self = serde_json::from_str(data)
            .map_err(|error| RegistryError::ParseFailed(error.to_string()))?;

        registry.validate()?;
        Ok(registry)
    }
}

/// Returns the current UNIX timestamp in seconds.
///
/// Unlike the previous lenient behavior, invalid system time is surfaced
/// explicitly so that callers can fail safely rather than silently recording
/// a zero timestamp.
fn current_time() -> Result<u64, RegistryError> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|value| value.as_secs())
        .map_err(|_| RegistryError::TimeError)
}

/// Canonicalizes and validates an AOXC actor identifier.
///
/// Policy:
/// - must not be blank,
/// - surrounding whitespace is rejected rather than normalized,
/// - must be bounded,
/// - must begin with `AOXC-`,
/// - only ASCII alphanumeric characters plus `_`, `-`, and `.` are accepted.
fn canonicalize_actor_id(actor_id: &str) -> Result<String, RegistryError> {
    if actor_id.is_empty() || actor_id.trim().is_empty() {
        return Err(RegistryError::EmptyActorId);
    }

    if actor_id != actor_id.trim() {
        return Err(RegistryError::InvalidActorId);
    }

    if actor_id.len() > MAX_ACTOR_ID_LEN {
        return Err(RegistryError::InvalidActorId);
    }

    if !actor_id.starts_with("AOXC-") {
        return Err(RegistryError::InvalidActorId);
    }

    if !actor_id
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '-' || ch == '.')
    {
        return Err(RegistryError::InvalidActorId);
    }

    Ok(actor_id.to_string())
}

/// Canonicalizes and validates a role identifier.
///
/// Policy:
/// - descriptive aliases are mapped explicitly,
/// - surrounding whitespace is rejected rather than normalized,
/// - canonical output is uppercase,
/// - only ASCII alphanumeric characters plus `_` and `-` are accepted.
fn canonicalize_role(role: &str) -> Result<String, RegistryError> {
    if role.is_empty() || role.trim().is_empty() {
        return Err(RegistryError::EmptyRole);
    }

    if role != role.trim() {
        return Err(RegistryError::InvalidRole);
    }

    if role.len() > MAX_ROLE_LEN {
        return Err(RegistryError::InvalidRole);
    }

    let normalized = role.to_ascii_lowercase();

    let canonical = match normalized.as_str() {
        "validator" | "val" => "VAL",
        "node" | "nod" => "NOD",
        "oracle" | "aor" => "AOR",
        "governance" | "gov" => "GOV",
        "operator" | "opr" => "OPR",
        "observer" | "obs" => "OBS",
        _ => {
            if role
                .chars()
                .all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '-')
            {
                return Ok(role.to_ascii_uppercase());
            }

            return Err(RegistryError::InvalidRole);
        }
    };

    Ok(canonical.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn register_and_lookup_actor() {
        let mut registry = Registry::new();

        let inserted = registry
            .register_at("AOXC-VAL-EU-1234".into(), "validator".into(), 100)
            .unwrap();

        assert!(inserted);
        assert_eq!(registry.role("AOXC-VAL-EU-1234"), Some("VAL"));
    }

    #[test]
    fn duplicate_registration_with_same_role_returns_false() {
        let mut registry = Registry::new();

        let first = registry
            .register_at("AOXC-ACTOR-1".into(), "node".into(), 100)
            .unwrap();
        let second = registry
            .register_at("AOXC-ACTOR-1".into(), "NOD".into(), 101)
            .unwrap();

        assert!(first);
        assert!(!second);
        assert_eq!(registry.len(), 1);
        assert_eq!(registry.get("AOXC-ACTOR-1").unwrap().registered_at, 100);
    }

    #[test]
    fn duplicate_registration_with_different_role_is_rejected() {
        let mut registry = Registry::new();

        registry
            .register_at("AOXC-ACTOR-2".into(), "node".into(), 100)
            .unwrap();

        let error = registry
            .register_at("AOXC-ACTOR-2".into(), "validator".into(), 101)
            .unwrap_err();

        assert_eq!(
            error,
            RegistryError::ActorAlreadyRegisteredWithDifferentRole
        );
    }

    #[test]
    fn deterministic_export() {
        let mut registry = Registry::new();

        registry
            .register_at("AOXC-B".into(), "node".into(), 100)
            .unwrap();
        registry
            .register_at("AOXC-A".into(), "node".into(), 100)
            .unwrap();

        let actors = registry.export_actor_ids();

        assert_eq!(actors, vec!["AOXC-A".to_string(), "AOXC-B".to_string()]);
    }

    #[test]
    fn export_entries_is_deterministic() {
        let mut registry = Registry::new();

        registry
            .register_at("AOXC-B".into(), "node".into(), 100)
            .unwrap();
        registry
            .register_at("AOXC-A".into(), "validator".into(), 200)
            .unwrap();

        let entries = registry.export_entries();

        assert_eq!(entries[0].actor_id, "AOXC-A");
        assert_eq!(entries[0].role, "VAL");
        assert_eq!(entries[1].actor_id, "AOXC-B");
        assert_eq!(entries[1].role, "NOD");
    }

    #[test]
    fn invalid_actor_id_is_rejected() {
        let mut registry = Registry::new();

        let error = registry
            .register_at("bad actor".into(), "node".into(), 100)
            .unwrap_err();

        assert_eq!(error, RegistryError::InvalidActorId);
    }

    #[test]
    fn invalid_role_is_rejected() {
        let mut registry = Registry::new();

        let error = registry
            .register_at("AOXC-ACTOR-3".into(), "node!".into(), 100)
            .unwrap_err();

        assert_eq!(error, RegistryError::InvalidRole);
    }

    #[test]
    fn invalid_timestamp_is_rejected() {
        let mut registry = Registry::new();

        let error = registry
            .register_at("AOXC-ACTOR-4".into(), "node".into(), 0)
            .unwrap_err();

        assert_eq!(error, RegistryError::InvalidRegisteredAt);
    }

    #[test]
    fn lookup_of_invalid_actor_returns_none() {
        let registry = Registry::new();
        assert_eq!(registry.role(" bad "), None);
        assert_eq!(registry.get("bad actor"), None);
        assert!(!registry.exists("bad actor"));
    }

    #[test]
    fn json_roundtrip_preserves_registry() {
        let mut registry = Registry::new();
        registry
            .register_at("AOXC-ACTOR-5".into(), "governance".into(), 100)
            .unwrap();

        let json = registry.to_json().unwrap();
        let restored = Registry::from_json(&json).unwrap();

        assert_eq!(registry, restored);
    }

    #[test]
    fn validate_rejects_corrupted_internal_state() {
        let mut actors = BTreeMap::new();
        actors.insert(
            "AOXC-ACTOR-6".to_string(),
            ActorEntry {
                actor_id: "AOXC-ACTOR-6".to_string(),
                role: " bad ".to_string(),
                registered_at: 100,
            },
        );

        let registry = Registry { actors };

        assert_eq!(registry.validate(), Err(RegistryError::InvalidRole));
    }
}
