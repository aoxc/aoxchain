use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

/// Actor registry entry.
///
/// Stores minimal identity metadata required by the node runtime.
#[derive(Debug, Clone)]
pub struct ActorEntry {
    pub actor_id: String,
    pub role: String,
    pub registered_at: u64,
}

/// Identity registry.
///
/// Maintains a mapping between actor identifiers and their roles.
pub struct Registry {
    actors: HashMap<String, ActorEntry>,
}

impl Default for Registry {
    fn default() -> Self {
        Self::new()
    }
}

impl Registry {
    /// Creates a new empty registry.
    pub fn new() -> Self {
        Self {
            actors: HashMap::new(),
        }
    }

    /// Registers an actor.
    ///
    /// If the actor already exists, the call is ignored.
    pub fn register(&mut self, actor_id: String, role: String) {
        if self.actors.contains_key(&actor_id) {
            return;
        }

        let entry = ActorEntry {
            actor_id: actor_id.clone(),
            role,
            registered_at: current_time(),
        };

        self.actors.insert(actor_id, entry);
    }

    /// Returns the role associated with an actor.
    pub fn role(&self, actor_id: &str) -> Option<&str> {
        self.actors.get(actor_id).map(|entry| entry.role.as_str())
    }

    /// Returns full metadata for an actor.
    pub fn get(&self, actor_id: &str) -> Option<&ActorEntry> {
        self.actors.get(actor_id)
    }

    /// Returns true if the actor exists.
    pub fn exists(&self, actor_id: &str) -> bool {
        self.actors.contains_key(actor_id)
    }

    /// Returns the number of registered actors.
    pub fn len(&self) -> usize {
        self.actors.len()
    }

    /// Returns true if the registry is empty.
    pub fn is_empty(&self) -> bool {
        self.actors.is_empty()
    }

    /// Deterministically exports actor identifiers.
    ///
    /// Useful for hashing or synchronization.
    pub fn export_actor_ids(&self) -> Vec<String> {
        let mut list: Vec<String> = self.actors.keys().cloned().collect();
        list.sort();
        list
    }
}

/// Returns the current UNIX timestamp in seconds.
///
/// If system time is invalid, zero is returned.
fn current_time() -> u64 {
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(v) => v.as_secs(),
        Err(_) => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn register_and_lookup_actor() {
        let mut registry = Registry::new();

        registry.register("AOXC-VAL-EU-1234".into(), "validator".into());

        assert_eq!(registry.role("AOXC-VAL-EU-1234"), Some("validator"));
    }

    #[test]
    fn duplicate_registration_is_ignored() {
        let mut registry = Registry::new();

        registry.register("actor1".into(), "node".into());
        registry.register("actor1".into(), "node".into());

        assert_eq!(registry.len(), 1);
    }

    #[test]
    fn deterministic_export() {
        let mut registry = Registry::new();

        registry.register("b".into(), "node".into());
        registry.register("a".into(), "node".into());

        let actors = registry.export_actor_ids();

        assert_eq!(actors, vec!["a".to_string(), "b".to_string()]);
    }
}
