// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

pub const DEFAULT_PACEMAKER_BASE_TIMEOUT_MS: u64 = 1_000;
pub const DEFAULT_PACEMAKER_MAX_TIMEOUT_MS: u64 = 60_000;

/// Deterministic pacemaker timeout configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PacemakerConfig {
    pub base_timeout_ms: u64,
    pub max_timeout_ms: u64,
}

impl PacemakerConfig {
    #[must_use]
    pub fn new(base_timeout_ms: u64, max_timeout_ms: u64) -> Self {
        let bounded_base = base_timeout_ms.max(1);
        let bounded_max = max_timeout_ms.max(bounded_base);
        Self {
            base_timeout_ms: bounded_base,
            max_timeout_ms: bounded_max,
        }
    }
}

impl Default for PacemakerConfig {
    fn default() -> Self {
        Self {
            base_timeout_ms: DEFAULT_PACEMAKER_BASE_TIMEOUT_MS,
            max_timeout_ms: DEFAULT_PACEMAKER_MAX_TIMEOUT_MS,
        }
    }
}

/// Consensus round and pacemaker state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RoundState {
    pub round: u64,
    pub timeout_ms: u64,
    pub timeout_count: u32,
    pub last_round_change_reason: RoundChangeReason,
    pub pacemaker: PacemakerConfig,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoundChangeReason {
    NormalProgress,
    Timeout,
    LeaderFailure,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PacemakerStep {
    pub previous_round: u64,
    pub next_round: u64,
    pub timeout_ms: u64,
    pub reason: RoundChangeReason,
}

impl RoundState {
    #[must_use]
    pub fn new() -> Self {
        Self::with_pacemaker(PacemakerConfig::default())
    }

    #[must_use]
    pub fn with_pacemaker(pacemaker: PacemakerConfig) -> Self {
        Self {
            round: 0,
            timeout_ms: pacemaker.base_timeout_ms,
            timeout_count: 0,
            last_round_change_reason: RoundChangeReason::NormalProgress,
            pacemaker,
        }
    }

    pub fn advance(&mut self) {
        self.round = self.round.saturating_add(1);
        self.timeout_count = 0;
        self.timeout_ms = self.pacemaker.base_timeout_ms;
        self.last_round_change_reason = RoundChangeReason::NormalProgress;
    }

    pub fn advance_to(&mut self, round: u64) {
        if round > self.round {
            self.round = round;
            self.timeout_count = 0;
            self.timeout_ms = self.pacemaker.base_timeout_ms;
            self.last_round_change_reason = RoundChangeReason::NormalProgress;
        }
    }

    #[must_use]
    pub fn on_timeout(&mut self) -> PacemakerStep {
        let previous = self.round;
        self.round = self.round.saturating_add(1);
        self.timeout_count = self.timeout_count.saturating_add(1);
        self.timeout_ms = self.timeout_ms.saturating_mul(2).clamp(
            self.pacemaker.base_timeout_ms,
            self.pacemaker.max_timeout_ms,
        );
        self.last_round_change_reason = RoundChangeReason::Timeout;

        PacemakerStep {
            previous_round: previous,
            next_round: self.round,
            timeout_ms: self.timeout_ms,
            reason: RoundChangeReason::Timeout,
        }
    }

    #[must_use]
    pub fn on_leader_failure(&mut self) -> PacemakerStep {
        let mut step = self.on_timeout();
        self.last_round_change_reason = RoundChangeReason::LeaderFailure;
        step.reason = RoundChangeReason::LeaderFailure;
        step
    }
}

impl Default for RoundState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::{PacemakerConfig, RoundChangeReason, RoundState};

    #[test]
    fn advance_to_is_monotonic() {
        let mut state = RoundState::new();
        state.advance_to(5);
        state.advance_to(3);
        assert_eq!(state.round, 5);
    }

    #[test]
    fn timeout_triggers_round_change_and_backoff() {
        let mut state = RoundState::new();
        let step = state.on_timeout();

        assert_eq!(step.previous_round, 0);
        assert_eq!(step.next_round, 1);
        assert_eq!(step.timeout_ms, 2_000);
        assert_eq!(step.reason, RoundChangeReason::Timeout);
    }

    #[test]
    fn custom_pacemaker_configuration_is_applied() {
        let mut state = RoundState::with_pacemaker(PacemakerConfig::new(750, 3_000));
        assert_eq!(state.timeout_ms, 750);

        let first = state.on_timeout();
        let second = state.on_timeout();
        let third = state.on_timeout();

        assert_eq!(first.timeout_ms, 1_500);
        assert_eq!(second.timeout_ms, 3_000);
        assert_eq!(third.timeout_ms, 3_000);
    }

    #[test]
    fn leader_failure_is_explicit_round_change_reason() {
        let mut state = RoundState::new();
        let step = state.on_leader_failure();

        assert_eq!(step.reason, RoundChangeReason::LeaderFailure);
        assert_eq!(
            state.last_round_change_reason,
            RoundChangeReason::LeaderFailure
        );
    }
}
