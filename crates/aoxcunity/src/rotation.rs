use crate::error::ConsensusError;
use crate::validator::{Validator, ValidatorId};

/// Deterministic proposer rotation.
///
/// Current policy:
/// - only active proposal-eligible validators participate,
/// - proposer selection is derived from block height modulo active set size.
#[derive(Debug, Clone)]
pub struct ValidatorRotation {
    validators: Vec<Validator>,
}

impl ValidatorRotation {
    pub fn new(validators: Vec<Validator>) -> Result<Self, ConsensusError> {
        if validators.is_empty() {
            return Err(ConsensusError::EmptyValidatorSet);
        }

        Ok(Self { validators })
    }

    pub fn proposer(&self, height: u64) -> Option<ValidatorId> {
        let eligible: Vec<&Validator> = self
            .validators
            .iter()
            .filter(|validator| validator.is_eligible_for_proposal())
            .collect();

        if eligible.is_empty() {
            return None;
        }

        let index = (height as usize) % eligible.len();
        Some(eligible[index].id)
    }

    pub fn validators(&self) -> &[Validator] {
        &self.validators
    }

    pub fn total_voting_power(&self) -> u64 {
        self.validators
            .iter()
            .filter(|validator| validator.is_eligible_for_vote())
            .map(|validator| validator.voting_power)
            .sum()
    }

    pub fn voting_power_of(&self, validator_id: ValidatorId) -> Option<u64> {
        self.validators
            .iter()
            .find(|validator| validator.id == validator_id)
            .map(|validator| validator.voting_power)
    }
}
