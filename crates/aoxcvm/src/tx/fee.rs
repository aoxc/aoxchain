//! Fee-budget model and helper operations.

use serde::{Deserialize, Serialize};

/// Upper bound a sender agrees to pay.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct FeeBudget {
    pub gas_limit: u64,
    pub gas_price: u64,
}

impl FeeBudget {
    pub const fn new(gas_limit: u64, gas_price: u64) -> Self {
        Self {
            gas_limit,
            gas_price,
        }
    }

    pub fn max_cost(self) -> Option<u128> {
        (self.gas_limit as u128).checked_mul(self.gas_price as u128)
    }

    pub fn price_per_unit(self) -> u64 {
        self.gas_price
    }
}
