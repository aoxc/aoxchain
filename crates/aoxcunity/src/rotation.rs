// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use std::collections::HashSet;

use sha2::{Digest, Sha256};

use crate::error::ConsensusError;
use crate::validator::{Validator, ValidatorId, ValidatorRole};

/// Deterministic proposer rotation.
///
/// Current policy:
/// - only active proposal-eligible validators participate,
/// - proposer selection is weight-aware,
/// - proposer lottery is deterministic per `(height, round, entropy)`.
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
        self.proposer_with_round(height, 0, [0u8; 32])
    }

    pub fn proposer_with_round(
        &self,
        height: u64,
        round: u64,
        entropy: [u8; 32],
    ) -> Option<ValidatorId> {
        let eligible: Vec<&Validator> = self
            .validators
            .iter()
            .filter(|validator| validator.is_eligible_for_proposal())
            .collect();

        if eligible.is_empty() {
            return None;
        }

        let total_weight: u128 = eligible
            .iter()
            .map(|validator| u128::from(validator.effective_voting_power().max(1)))
            .sum();

        if total_weight == 0 {
            return None;
        }

        let lottery = proposer_lottery(height, round, entropy, self.validator_set_hash());
        let cursor = lottery % total_weight;

        let mut running = 0u128;
        for validator in eligible {
            running = running.saturating_add(u128::from(validator.effective_voting_power().max(1)));
            if cursor < running {
                return Some(validator.id);
            }
        }

        None
    }

    pub fn validators(&self) -> &[Validator] {
        &self.validators
    }

    pub fn validators_mut(&mut self) -> &mut [Validator] {
        &mut self.validators
    }

    pub fn total_voting_power(&self) -> u64 {
        self.validators
            .iter()
            .filter(|validator| validator.is_eligible_for_vote())
            .map(|validator| validator.effective_voting_power())
            .sum()
    }

    pub fn voting_power_of(&self, validator_id: ValidatorId) -> Option<u64> {
        self.validators
            .iter()
            .find(|validator| validator.id == validator_id)
            .map(Validator::effective_voting_power)
    }

    pub fn eligible_voting_power_of(&self, validator_id: ValidatorId) -> Option<u64> {
        self.validators
            .iter()
            .find(|validator| validator.id == validator_id && validator.is_eligible_for_vote())
            .map(Validator::effective_voting_power)
    }

    pub fn contains_active_vote_eligible_validator(&self, validator_id: ValidatorId) -> bool {
        self.eligible_voting_power_of(validator_id).is_some()
    }

    pub fn validator(&self, validator_id: ValidatorId) -> Option<&Validator> {
        self.validators
            .iter()
            .find(|validator| validator.id == validator_id)
    }

    pub fn validator_mut(&mut self, validator_id: ValidatorId) -> Option<&mut Validator> {
        self.validators
            .iter_mut()
            .find(|validator| validator.id == validator_id)
    }

    #[must_use]
    pub fn validator_set_hash(&self) -> [u8; 32] {
        let mut validators = self.validators.clone();
        validators.sort_by(|a, b| a.id.cmp(&b.id));

        let mut hasher = Sha256::new();
        hasher.update(b"AOXC_VALIDATOR_SET_V2");
        hasher.update((validators.len() as u64).to_le_bytes());

        for validator in validators {
            hasher.update(validator.id);
            hasher.update(validator.effective_voting_power().to_le_bytes());
            hasher.update([match validator.role {
                ValidatorRole::Validator => 0,
                ValidatorRole::Observer => 1,
                ValidatorRole::Proposer => 2,
            }]);
            hasher.update([u8::from(validator.active)]);
            hasher.update([match validator.lifecycle {
                crate::validator::ValidatorLifecycle::Joining => 0,
                crate::validator::ValidatorLifecycle::Active => 1,
                crate::validator::ValidatorLifecycle::Exiting => 2,
                crate::validator::ValidatorLifecycle::Jailed => 3,
            }]);
        }

        hasher.finalize().into()
    }

    #[must_use]
    pub fn pq_attestation_root(&self) -> [u8; 32] {
        let mut validators = self.validators.clone();
        validators.sort_by(|a, b| a.id.cmp(&b.id));

        let mut hasher = Sha256::new();
        hasher.update(b"AOXC_PQ_ATTESTATION_ROOT_V1");
        hasher.update((validators.len() as u64).to_le_bytes());

        for validator in validators {
            hasher.update(validator.id);
            hasher.update(validator.pq_attestation_commitment);
        }

        hasher.finalize().into()
    }
}

fn proposer_lottery(height: u64, round: u64, entropy: [u8; 32], set_root: [u8; 32]) -> u128 {
    let mut hasher = Sha256::new();
    hasher.update(b"AOXC_PROPOSER_LOTTERY_V1");
    hasher.update(height.to_le_bytes());
    hasher.update(round.to_le_bytes());
    hasher.update(entropy);
    hasher.update(set_root);

    let digest: [u8; 32] = hasher.finalize().into();
    let mut lower = [0u8; 16];
    lower.copy_from_slice(&digest[..16]);
    u128::from_le_bytes(lower)
}

#[cfg(test)]
mod tests {
    use super::ValidatorRotation;
    use crate::validator::{Validator, ValidatorRole};

    #[test]
    fn weighted_proposer_is_deterministic() {
        let mut heavy = Validator::new([1u8; 32], 100, ValidatorRole::Validator);
        heavy.delegate(50);
        let light = Validator::new([2u8; 32], 1, ValidatorRole::Validator);

        let rotation = ValidatorRotation::new(vec![heavy, light]).expect("rotation");

        let a = rotation.proposer_with_round(5, 2, [7u8; 32]);
        let b = rotation.proposer_with_round(5, 2, [7u8; 32]);

        assert_eq!(a, b);
    }

    #[test]
    fn pq_attestation_root_tracks_commitments() {
        let a =
            Validator::new([1u8; 32], 10, ValidatorRole::Validator).with_pq_attestation([8u8; 32]);
        let b =
            Validator::new([2u8; 32], 10, ValidatorRole::Validator).with_pq_attestation([9u8; 32]);
        let c =
            Validator::new([2u8; 32], 10, ValidatorRole::Validator).with_pq_attestation([4u8; 32]);

        let root_a = ValidatorRotation::new(vec![a.clone(), b])
            .expect("rotation")
            .pq_attestation_root();
        let root_b = ValidatorRotation::new(vec![a, c])
            .expect("rotation")
            .pq_attestation_root();

        assert_ne!(root_a, root_b);
    }
}
