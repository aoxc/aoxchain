/// Consensus round state.
///
/// This structure remains intentionally compact. Additional pacemaker logic
/// can be layered later without destabilizing the core state model.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RoundState {
    pub round: u64,
}

impl RoundState {
    #[must_use]
    pub fn new() -> Self {
        Self { round: 0 }
    }

    pub fn advance(&mut self) {
        self.round = self.round.saturating_add(1);
    }

    pub fn advance_to(&mut self, round: u64) {
        self.round = self.round.max(round);
    }
}

impl Default for RoundState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::RoundState;

    #[test]
    fn advance_to_is_monotonic() {
        let mut state = RoundState::new();
        state.advance_to(5);
        state.advance_to(3);
        assert_eq!(state.round, 5);
    }

    #[test]
    fn advance_to_equal_round_is_noop() {
        let mut state = RoundState::new();
        state.advance_to(4);
        state.advance_to(4);
        assert_eq!(state.round, 4);
    }
}
