use crate::state::diff::StateDiff;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FinalizationOutcome {
    pub success: bool,
    pub diff: StateDiff,
}
