use std::collections::HashSet;

use sha2::{Digest, Sha256};

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

        let mut seen = HashSet::new();
        for validator in &validators {
            if !seen.insert(validator.id) {
                return Err(ConsensusError::DuplicateValidator);
            }
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

    pub fn eligible_voting_power_of(&self, validator_id: ValidatorId) -> Option<u64> {
        self.validators
            .iter()
            .find(|validator| validator.id == validator_id && validator.is_eligible_for_vote())
            .map(|validator| validator.voting_power)
    }

    pub fn contains_active_vote_eligible_validator(&self, validator_id: ValidatorId) -> bool {
        self.eligible_voting_power_of(validator_id).is_some()
    }

    pub fn validator(&self, validator_id: ValidatorId) -> Option<&Validator> {
        self.validators
            .iter()
            .find(|validator| validator.id == validator_id)
    }

    #[must_use]
    pub fn validator_set_hash(&self) -> [u8; 32] {
        let mut validators = self.validators.clone();
        validators.sort_by(|a, b| a.id.cmp(&b.id));

        let mut hasher = Sha256::new();
        hasher.update(b"AOXC_VALIDATOR_SET_V1");
        hasher.update((validators.len() as u64).to_le_bytes());

        for validator in validators {
            hasher.update(validator.id);
            hasher.update(validator.voting_power.to_le_bytes());
            hasher.update([match validator.role {
                crate::validator::ValidatorRole::Validator => 0,
                crate::validator::ValidatorRole::Observer => 1,
                crate::validator::ValidatorRole::Proposer => 2,
            }]);
            hasher.update([u8::from(validator.active)]);
        }

        hasher.finalize().into()
    }
}

#[cfg(test)]
mod tests {
    use crate::validator::{Validator, ValidatorRole};

    use super::ValidatorRotation;

    fn validator(id: u8, power: u64, role: ValidatorRole, active: bool) -> Validator {
        let mut validator = Validator::new([id; 32], power, role);
        validator.active = active;
        validator
    }

    #[test]
    fn validator_set_hash_is_deterministic_for_same_members_different_order() {
        let a = ValidatorRotation::new(vec![
            validator(1, 10, ValidatorRole::Validator, true),
            validator(2, 5, ValidatorRole::Observer, false),
        ])
        .unwrap();
        let b = ValidatorRotation::new(vec![
            validator(2, 5, ValidatorRole::Observer, false),
            validator(1, 10, ValidatorRole::Validator, true),
        ])
        .unwrap();

        assert_eq!(a.validator_set_hash(), b.validator_set_hash());
    }

    #[test]
    fn validator_set_hash_changes_when_power_role_or_activity_changes() {
        let base =
            ValidatorRotation::new(vec![validator(1, 10, ValidatorRole::Validator, true)]).unwrap();
        let different_power =
            ValidatorRotation::new(vec![validator(1, 11, ValidatorRole::Validator, true)]).unwrap();
        let different_role =
            ValidatorRotation::new(vec![validator(1, 10, ValidatorRole::Observer, true)]).unwrap();
        let different_activity =
            ValidatorRotation::new(vec![validator(1, 10, ValidatorRole::Validator, false)])
                .unwrap();

        assert_ne!(
            base.validator_set_hash(),
            different_power.validator_set_hash()
        );
        assert_ne!(
            base.validator_set_hash(),
            different_role.validator_set_hash()
        );
        assert_ne!(
            base.validator_set_hash(),
            different_activity.validator_set_hash()
        );
    }
}
