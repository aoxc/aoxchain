// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use crate::{config::settings::Settings, node::state::NodeState};
use serde::Serialize;

/// Canonical AOXC runtime context exported by operator-facing runtime surfaces.
///
/// Design intent:
/// - Bind the effective runtime settings surface to the currently observed
///   node-state surface.
/// - Preserve a single serializable envelope for diagnostics, status, and
///   runtime inspection commands.
/// - Allow callers to distinguish between "settings are known" and
///   "node state has not yet been initialized or could not be loaded".
///
/// Semantics:
/// - `settings` is always present and represents the effective runtime
///   configuration contract.
/// - `node_state` is optional because early bootstrap or read-only runtime
///   surfaces may legitimately operate before a canonical node-state payload
///   exists.
#[derive(Debug, Clone, Serialize)]
pub struct RuntimeContext {
    pub settings: Settings,
    pub node_state: Option<NodeState>,
}

impl RuntimeContext {
    /// Constructs a runtime context from effective settings and an optional
    /// node-state snapshot.
    pub fn new(settings: Settings, node_state: Option<NodeState>) -> Self {
        Self {
            settings,
            node_state,
        }
    }

    /// Constructs a runtime context for settings-only flows where node state is
    /// not yet available or is intentionally omitted.
    pub fn without_node_state(settings: Settings) -> Self {
        Self {
            settings,
            node_state: None,
        }
    }

    /// Returns `true` when a canonical node-state snapshot is attached.
    pub fn has_node_state(&self) -> bool {
        self.node_state.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::RuntimeContext;
    use crate::{config::settings::Settings, node::state::NodeState};

    #[test]
    fn runtime_context_new_preserves_supplied_state() {
        let settings = Settings::default_for("/tmp/aoxc".to_string());
        let node_state = Some(NodeState::bootstrap());

        let context = RuntimeContext::new(settings.clone(), node_state.clone());

        assert_eq!(context.settings.profile, settings.profile);
        assert_eq!(context.node_state.is_some(), node_state.is_some());
    }

    #[test]
    fn runtime_context_without_node_state_omits_runtime_snapshot() {
        let settings = Settings::default_for("/tmp/aoxc".to_string());

        let context = RuntimeContext::without_node_state(settings);

        assert!(!context.has_node_state());
    }

    #[test]
    fn runtime_context_has_node_state_reports_presence_correctly() {
        let settings = Settings::default_for("/tmp/aoxc".to_string());

        let without_state = RuntimeContext::without_node_state(settings.clone());
        let with_state = RuntimeContext::new(settings, Some(NodeState::bootstrap()));

        assert!(!without_state.has_node_state());
        assert!(with_state.has_node_state());
    }
}
