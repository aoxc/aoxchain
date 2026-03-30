// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use serde::{Deserialize, Serialize};

pub type ValidatorId = [u8; 32];

/// Validator role classification.
///
/// The role model is intentionally small. More specialized operational
/// responsibilities should live in upper layers rather than bloating the
/// consensus core role taxonomy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ValidatorRole {
    Validator,
    Observer,
    Proposer,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ValidatorLifecycle {
    Joining,
    Active,
    Exiting,
    Jailed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SlashFault {
    Equivocation,
    Liveness,
}

/// Canonical validator record.
///
/// This structure captures validator identity, stake state, and lifecycle gates
/// required by consensus and pacemaker logic.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Validator {
    pub id: ValidatorId,
    pub voting_power: u64,
    pub role: ValidatorRole,
    pub active: bool,
    pub lifecycle: ValidatorLifecycle,
    pub self_bonded: u64,
    pub delegated: u64,
    pub unbonding: u64,
    pub slashed_total: u64,
    pub liveness_misses: u32,
    pub jailed_until_round: Option<u64>,
    pub pq_attestation_commitment: [u8; 32],
}

impl Validator {
    pub fn new(id: ValidatorId, voting_power: u64, role: ValidatorRole) -> Self {
        Self {
            id,
            voting_power,
            role,
            active: true,
            lifecycle: ValidatorLifecycle::Active,
            self_bonded: voting_power,
            delegated: 0,
            unbonding: 0,
            slashed_total: 0,
            liveness_misses: 0,
            jailed_until_round: None,
            pq_attestation_commitment: [0u8; 32],
        }
    }

    #[must_use]
    pub fn with_pq_attestation(mut self, commitment: [u8; 32]) -> Self {
        self.pq_attestation_commitment = commitment;
        self
    }

    pub fn set_pq_attestation_commitment(&mut self, commitment: [u8; 32]) {
        self.pq_attestation_commitment = commitment;
    }

    #[must_use]
    pub fn effective_voting_power(&self) -> u64 {
        self.voting_power.saturating_add(self.delegated)
    }

    pub fn bond(&mut self, amount: u64) {
        self.self_bonded = self.self_bonded.saturating_add(amount);
        self.voting_power = self.voting_power.saturating_add(amount);
    }

    pub fn unbond(&mut self, amount: u64) -> u64 {
        let to_unbond = amount.min(self.self_bonded);
        self.self_bonded = self.self_bonded.saturating_sub(to_unbond);
        self.voting_power = self.voting_power.saturating_sub(to_unbond);
        self.unbonding = self.unbonding.saturating_add(to_unbond);
        to_unbond
    }

    pub fn delegate(&mut self, amount: u64) {
        self.delegated = self.delegated.saturating_add(amount);
    }

    pub fn undelegate(&mut self, amount: u64) -> u64 {
        let amount = amount.min(self.delegated);
        self.delegated = self.delegated.saturating_sub(amount);
        amount
    }

    pub fn activate(&mut self) {
        self.lifecycle = ValidatorLifecycle::Active;
        self.active = true;
    }

    pub fn begin_exit(&mut self) {
        self.lifecycle = ValidatorLifecycle::Exiting;
        self.active = false;
    }

    pub fn jail(&mut self, until_round: u64) {
        self.lifecycle = ValidatorLifecycle::Jailed;
        self.active = false;
        self.jailed_until_round = Some(until_round);
    }

    pub fn try_unjail(&mut self, current_round: u64) -> bool {
        let Some(until_round) = self.jailed_until_round else {
            return false;
        };

        if current_round < until_round {
            return false;
        }

        self.jailed_until_round = None;
        self.lifecycle = ValidatorLifecycle::Active;
        self.active = true;
        self.liveness_misses = 0;
        true
    }

    pub fn register_liveness_miss(&mut self, jail_after_misses: u32, jail_until_round: u64) {
        self.liveness_misses = self.liveness_misses.saturating_add(1);
        if self.liveness_misses >= jail_after_misses {
            self.jail(jail_until_round);
        }
    }

    pub fn slash(&mut self, numerator: u64, denominator: u64, fault: SlashFault) -> u64 {
        if denominator == 0 {
            return 0;
        }

        let base = self.effective_voting_power();
        let slashed = base.saturating_mul(numerator) / denominator;

        let slash_from_bonded = slashed.min(self.voting_power);
        self.voting_power = self.voting_power.saturating_sub(slash_from_bonded);
        self.self_bonded = self.self_bonded.saturating_sub(slash_from_bonded);

        let remainder = slashed.saturating_sub(slash_from_bonded);
        self.delegated = self.delegated.saturating_sub(remainder.min(self.delegated));
        self.slashed_total = self.slashed_total.saturating_add(slashed);

        if matches!(fault, SlashFault::Equivocation) {
            self.jail(u64::MAX);
        }

        slashed
    }

    pub fn is_eligible_for_proposal(&self) -> bool {
        self.active
            && matches!(self.lifecycle, ValidatorLifecycle::Active)
            && matches!(
                self.role,
                ValidatorRole::Validator | ValidatorRole::Proposer
            )
    }

    /// Returns whether the validator may contribute vote authority.
    ///
    /// This function is the authoritative policy gate for consensus vote
    /// eligibility. Callers must not infer vote authority through role or
    /// activity checks outside this function.
    pub fn is_eligible_for_vote(&self) -> bool {
        self.active
            && matches!(self.lifecycle, ValidatorLifecycle::Active)
            && self.effective_voting_power() > 0
            && matches!(
                self.role,
                ValidatorRole::Validator | ValidatorRole::Proposer
            )
    }
}

#[cfg(test)]
mod tests {
    use super::{SlashFault, Validator, ValidatorLifecycle, ValidatorRole};

    #[test]
    fn delegate_changes_effective_voting_power() {
        let mut validator = Validator::new([1u8; 32], 10, ValidatorRole::Validator);
        validator.delegate(15);

        assert_eq!(validator.effective_voting_power(), 25);
    }

    #[test]
    fn slash_for_equivocation_jails_validator() {
        let mut validator = Validator::new([1u8; 32], 100, ValidatorRole::Validator);
        let slashed = validator.slash(5, 100, SlashFault::Equivocation);

        assert_eq!(slashed, 5);
        assert_eq!(validator.lifecycle, ValidatorLifecycle::Jailed);
        assert!(!validator.active);
    }

    #[test]
    fn unjail_requires_round_threshold() {
        let mut validator = Validator::new([1u8; 32], 10, ValidatorRole::Validator);
        validator.jail(10);

        assert!(!validator.try_unjail(9));
        assert!(validator.try_unjail(10));
        assert!(validator.is_eligible_for_vote());
    }
}
