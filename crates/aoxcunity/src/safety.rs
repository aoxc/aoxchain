use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JustificationRef {
    pub block_hash: [u8; 32],
    pub height: u64,
    pub round: u64,
    pub epoch: u64,
    pub certificate_hash: [u8; 32],
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct LockState {
    locked: Option<JustificationRef>,
}

impl LockState {
    #[must_use]
    pub fn current(&self) -> Option<&JustificationRef> {
        self.locked.as_ref()
    }

    #[must_use]
    pub fn can_advance_to(&self, candidate: &JustificationRef) -> bool {
        self.locked.as_ref().is_none_or(|current| {
            candidate.epoch > current.epoch
                || (candidate.epoch == current.epoch
                    && (candidate.height > current.height
                        || (candidate.height == current.height
                            && candidate.round >= current.round)))
        })
    }

    pub fn advance_to(&mut self, candidate: JustificationRef) -> bool {
        if !self.can_advance_to(&candidate) {
            return false;
        }

        self.locked = Some(candidate);
        true
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SafeToVote {
    Yes,
    No(SafetyViolation),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SafetyViolation {
    LockRegression,
    EpochRegression,
    RoundRegression,
}

#[must_use]
pub fn evaluate_safe_to_vote(lock_state: &LockState, candidate: &JustificationRef) -> SafeToVote {
    match lock_state.current() {
        None => SafeToVote::Yes,
        Some(current) if candidate.epoch < current.epoch => {
            SafeToVote::No(SafetyViolation::EpochRegression)
        }
        Some(current) if candidate.epoch == current.epoch && candidate.height < current.height => {
            SafeToVote::No(SafetyViolation::LockRegression)
        }
        Some(current)
            if candidate.epoch == current.epoch
                && candidate.height == current.height
                && candidate.round < current.round =>
        {
            SafeToVote::No(SafetyViolation::RoundRegression)
        }
        _ => SafeToVote::Yes,
    }
}

#[cfg(test)]
mod tests {
    use super::{JustificationRef, LockState, SafeToVote, SafetyViolation, evaluate_safe_to_vote};

    fn justification(height: u64, round: u64, epoch: u64, tag: u8) -> JustificationRef {
        JustificationRef {
            block_hash: [tag; 32],
            height,
            round,
            epoch,
            certificate_hash: [tag.wrapping_add(1); 32],
        }
    }

    #[test]
    fn lock_state_advances_monotonically() {
        let mut lock = LockState::default();
        assert!(lock.advance_to(justification(10, 2, 4, 1)));
        assert!(!lock.advance_to(justification(9, 9, 4, 2)));
        assert!(lock.advance_to(justification(10, 3, 4, 3)));
    }

    #[test]
    fn safe_to_vote_rejects_epoch_and_round_regressions() {
        let mut lock = LockState::default();
        lock.advance_to(justification(10, 4, 7, 1));

        assert_eq!(
            evaluate_safe_to_vote(&lock, &justification(11, 1, 6, 2)),
            SafeToVote::No(SafetyViolation::EpochRegression)
        );
        assert_eq!(
            evaluate_safe_to_vote(&lock, &justification(10, 3, 7, 2)),
            SafeToVote::No(SafetyViolation::RoundRegression)
        );
    }
}
