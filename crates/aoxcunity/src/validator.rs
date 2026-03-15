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

/// Canonical validator record.
///
/// This structure captures the minimum identity required by the consensus
/// layer to determine voting and proposer rotation eligibility.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Validator {
    pub id: ValidatorId,
    pub voting_power: u64,
    pub role: ValidatorRole,
    pub active: bool,
}

impl Validator {
    pub fn new(id: ValidatorId, voting_power: u64, role: ValidatorRole) -> Self {
        Self {
            id,
            voting_power,
            role,
            active: true,
        }
    }

    pub fn is_eligible_for_proposal(&self) -> bool {
        self.active
            && matches!(
                self.role,
                ValidatorRole::Validator | ValidatorRole::Proposer
            )
    }

    pub fn is_eligible_for_vote(&self) -> bool {
        self.active
            && matches!(
                self.role,
                ValidatorRole::Validator | ValidatorRole::Proposer
            )
    }
}
