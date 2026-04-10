use super::model::{BPS_DENOMINATOR, UnitAmount};

pub const MAX_SCENARIO_MULTIPLIER_BPS: u32 = 50_000;

#[must_use]
pub fn share_bps(part: u128, total: u128) -> u32 {
    if total == 0 {
        return 0;
    }
    let numerator = part.saturating_mul(BPS_DENOMINATOR as u128);
    (numerator / total) as u32
}

#[must_use]
pub fn scale_u64_by_bps(value: u64, bps: u32) -> u64 {
    let weighted = (value as u128).saturating_mul(bps as u128);
    (weighted / BPS_DENOMINATOR as u128) as u64
}

#[must_use]
pub fn quantum_readiness_index_bps(
    sustainable_cost: UnitAmount,
    quantum_transition_component: UnitAmount,
    quantum_assurance_component: UnitAmount,
    security_component: UnitAmount,
) -> u32 {
    if sustainable_cost.is_zero() {
        return 0;
    }

    let quantum_total = quantum_transition_component
        .micros()
        .saturating_add(quantum_assurance_component.micros())
        .saturating_add(security_component.micros());

    share_bps(quantum_total, sustainable_cost.micros()).min(BPS_DENOMINATOR)
}
