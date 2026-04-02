//! State journal access verification for AOXCVM phase-1.

use crate::state::JournaledState;

/// Compact snapshot used by verifier when comparing replay runs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StateSnapshot {
    pub canonical: Vec<u8>,
}

impl StateSnapshot {
    /// Captures canonical state bytes from a journaled state instance.
    pub fn capture(state: &JournaledState) -> Self {
        Self {
            canonical: state.canonical_bytes(),
        }
    }

    /// Checks equality against another snapshot.
    pub fn matches(&self, other: &Self) -> bool {
        self.canonical == other.canonical
    }
}

#[cfg(test)]
mod tests {
    use super::StateSnapshot;
    use crate::state::JournaledState;

    #[test]
    fn snapshot_matches_identical_state() {
        let mut a = JournaledState::default();
        a.put(vec![1], vec![2]);
        let b = a.clone();
        let sa = StateSnapshot::capture(&a);
        let sb = StateSnapshot::capture(&b);
        assert!(sa.matches(&sb));
    }
}
