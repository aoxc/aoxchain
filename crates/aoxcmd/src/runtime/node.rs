// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use crate::{error::AppError, node::lifecycle::load_state};

const STATUS_UNINITIALIZED: &str = "uninitialized";
const STATUS_DEGRADED_KEY_STATE: &str = "degraded-key-state";
const STATUS_HEALTHY: &str = "healthy";
const STATUS_BOOTSTRAPPED: &str = "bootstrapped";

/// Returns the canonical AOXC runtime health status.
///
/// Health policy:
/// - A non-initialized node is reported as `uninitialized`.
/// - A node with non-empty but non-active key material state is reported as
///   `degraded-key-state`.
/// - A node that has already advanced beyond height zero is reported as
///   `healthy`.
/// - An initialized node at height zero is reported as `bootstrapped`.
///
/// Operational rationale:
/// - This surface is intentionally lightweight and read-oriented.
/// - The result is intended for CLI health checks and operator-facing status
///   summaries rather than deep diagnostics.
pub fn health_status() -> Result<&'static str, AppError> {
    let state = load_state()?;
    Ok(classify_health(&state))
}

/// Classifies runtime health from a loaded node-state snapshot.
///
/// Classification order matters:
/// 1. initialization failure dominates every other state,
/// 2. degraded key state overrides normal bootstrap/healthy posture,
/// 3. positive chain advancement indicates healthy runtime progress,
/// 4. otherwise the node is merely bootstrapped.
fn classify_health(state: &crate::node::state::NodeState) -> &'static str {
    if !state.initialized {
        STATUS_UNINITIALIZED
    } else if !state.key_material.operational_state.is_empty()
        && state.key_material.operational_state != "active"
    {
        STATUS_DEGRADED_KEY_STATE
    } else if state.current_height > 0 {
        STATUS_HEALTHY
    } else {
        STATUS_BOOTSTRAPPED
    }
}

#[cfg(test)]
mod tests {
    use super::{
        classify_health, STATUS_BOOTSTRAPPED, STATUS_DEGRADED_KEY_STATE, STATUS_HEALTHY,
        STATUS_UNINITIALIZED,
    };
    use crate::node::state::NodeState;

    #[test]
    fn health_is_uninitialized_when_state_is_not_initialized() {
        let mut state = NodeState::bootstrap();
        state.initialized = false;

        assert_eq!(classify_health(&state), STATUS_UNINITIALIZED);
    }

    #[test]
    fn health_is_degraded_when_key_material_state_is_not_active() {
        let mut state = NodeState::bootstrap();
        state.key_material.operational_state = "locked".to_string();

        assert_eq!(classify_health(&state), STATUS_DEGRADED_KEY_STATE);
    }

    #[test]
    fn health_is_healthy_after_block_production_begins() {
        let mut state = NodeState::bootstrap();
        state.current_height = 1;
        state.produced_blocks = 1;

        assert_eq!(classify_health(&state), STATUS_HEALTHY);
    }

    #[test]
    fn health_is_bootstrapped_when_initialized_but_height_is_zero() {
        let state = NodeState::bootstrap();

        assert_eq!(classify_health(&state), STATUS_BOOTSTRAPPED);
    }

    #[test]
    fn degraded_key_state_takes_precedence_over_positive_height() {
        let mut state = NodeState::bootstrap();
        state.current_height = 3;
        state.produced_blocks = 3;
        state.key_material.operational_state = "revoked".to_string();

        assert_eq!(classify_health(&state), STATUS_DEGRADED_KEY_STATE);
    }
}
