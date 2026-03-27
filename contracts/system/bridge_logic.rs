// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

#[derive(Debug, Clone)]
pub struct BridgeTransfer {
    pub source_lane: String,
    pub destination_lane: String,
    pub asset: String,
    pub amount: u128,
}

impl BridgeTransfer {
    pub fn validate(&self) -> bool {
        !self.source_lane.is_empty()
            && !self.destination_lane.is_empty()
            && self.amount > 0
            && self.source_lane != self.destination_lane
    }
}
