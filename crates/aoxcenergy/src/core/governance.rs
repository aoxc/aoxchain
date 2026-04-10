use super::math::{MAX_SCENARIO_MULTIPLIER_BPS, share_bps};
use super::model::{
    BPS_DENOMINATOR, EnergyError, FloorModelInputs, GovernanceDecision, GovernancePolicy,
    UnitAmount,
};

pub fn validate_scenario_multiplier_bps(value: u32) -> Result<(), EnergyError> {
    if value == 0 {
        return Err(EnergyError::InvalidInput(
            "scenario.demand_multiplier_bps must be greater than zero".to_owned(),
        ));
    }
    if value > MAX_SCENARIO_MULTIPLIER_BPS {
        return Err(EnergyError::InvalidInput(format!(
            "scenario.demand_multiplier_bps exceeds maximum value '{}'",
            MAX_SCENARIO_MULTIPLIER_BPS
        )));
    }
    Ok(())
}

pub fn validate_inputs(
    inputs: &FloorModelInputs,
    governance: &GovernancePolicy,
) -> Result<(), EnergyError> {
    if inputs.energy.kilowatt_hours_per_period == 0 {
        return Err(EnergyError::InvalidInput(
            "kilowatt_hours_per_period must be greater than zero".to_owned(),
        ));
    }

    if inputs.demand.units_per_period == 0 {
        return Err(EnergyError::InvalidInput(
            "units_per_period must be greater than zero".to_owned(),
        ));
    }

    validate_bps(
        "energy.cooling_overhead_bps",
        inputs.energy.cooling_overhead_bps,
    )?;
    validate_bps(
        "policy.continuity_buffer_bps",
        inputs.policy.continuity_buffer_bps,
    )?;
    validate_bps(
        "policy.security_reserve_bps",
        inputs.policy.security_reserve_bps,
    )?;
    validate_bps(
        "policy.quantum_transition_reserve_bps",
        inputs.policy.quantum_transition_reserve_bps,
    )?;
    validate_bps(
        "policy.quantum_assurance_bps",
        inputs.policy.quantum_assurance_bps,
    )?;
    validate_bps(
        "policy.treasury_build_bps",
        inputs.policy.treasury_build_bps,
    )?;
    validate_bps("policy.target_margin_bps", inputs.policy.target_margin_bps)?;
    validate_bps("policy.tax_bps", inputs.policy.tax_bps)?;

    validate_bps("governance.max_tax_bps", governance.max_tax_bps)?;
    validate_bps(
        "governance.max_treasury_build_bps",
        governance.max_treasury_build_bps,
    )?;
    validate_bps(
        "governance.max_quantum_reserve_bps",
        governance.max_quantum_reserve_bps,
    )?;
    validate_bps(
        "governance.max_period_floor_increase_bps",
        governance.max_period_floor_increase_bps,
    )?;
    validate_bps(
        "governance.max_kernel_layer_share_bps",
        governance.max_kernel_layer_share_bps,
    )?;
    validate_bps(
        "governance.max_execution_layer_share_bps",
        governance.max_execution_layer_share_bps,
    )?;
    validate_bps(
        "governance.min_quantum_readiness_bps",
        governance.min_quantum_readiness_bps,
    )?;

    if inputs.policy.tax_bps > governance.max_tax_bps {
        return Err(EnergyError::InvalidInput(format!(
            "tax_bps '{}' exceeds governance maximum '{}'",
            inputs.policy.tax_bps, governance.max_tax_bps
        )));
    }

    if inputs.policy.treasury_build_bps > governance.max_treasury_build_bps {
        return Err(EnergyError::InvalidInput(format!(
            "treasury_build_bps '{}' exceeds governance maximum '{}'",
            inputs.policy.treasury_build_bps, governance.max_treasury_build_bps
        )));
    }

    let total_quantum_bps = inputs
        .policy
        .quantum_transition_reserve_bps
        .saturating_add(inputs.policy.quantum_assurance_bps);
    if total_quantum_bps > governance.max_quantum_reserve_bps {
        return Err(EnergyError::InvalidInput(format!(
            "total quantum reserve bps '{}' exceeds governance maximum '{}'",
            total_quantum_bps, governance.max_quantum_reserve_bps
        )));
    }

    Ok(())
}

pub fn combine_governance_decisions(
    left: GovernanceDecision,
    right: GovernanceDecision,
) -> GovernanceDecision {
    match (left, right) {
        (GovernanceDecision::Rejected, _) | (_, GovernanceDecision::Rejected) => {
            GovernanceDecision::Rejected
        }
        (GovernanceDecision::RequiresReview, _) | (_, GovernanceDecision::RequiresReview) => {
            GovernanceDecision::RequiresReview
        }
        _ => GovernanceDecision::Approved,
    }
}

pub fn evaluate_layer_and_quantum_guardrails(
    total_layer_cost: UnitAmount,
    kernel_layer_cost: UnitAmount,
    execution_layer_cost: UnitAmount,
    quantum_readiness_index_bps: u32,
    governance: &GovernancePolicy,
    emergency_override: bool,
) -> (GovernanceDecision, Vec<String>) {
    let mut notes = Vec::new();
    let mut decision = GovernanceDecision::Approved;

    if !total_layer_cost.is_zero() {
        let kernel_share = share_bps(kernel_layer_cost.micros(), total_layer_cost.micros());
        notes.push(format!(
            "kernel layer share check: observed={} max={}",
            kernel_share, governance.max_kernel_layer_share_bps
        ));
        if kernel_share > governance.max_kernel_layer_share_bps {
            decision = GovernanceDecision::RequiresReview;
            notes.push("kernel layer share exceeds configured maximum".to_owned());
        }

        let execution_share = share_bps(execution_layer_cost.micros(), total_layer_cost.micros());
        notes.push(format!(
            "execution layer share check: observed={} max={}",
            execution_share, governance.max_execution_layer_share_bps
        ));
        if execution_share > governance.max_execution_layer_share_bps {
            decision = GovernanceDecision::RequiresReview;
            notes.push("execution layer share exceeds configured maximum".to_owned());
        }
    }

    notes.push(format!(
        "quantum readiness check: observed={} min={}",
        quantum_readiness_index_bps, governance.min_quantum_readiness_bps
    ));
    if quantum_readiness_index_bps < governance.min_quantum_readiness_bps {
        if emergency_override && governance.allow_emergency_override {
            decision = combine_governance_decisions(decision, GovernanceDecision::RequiresReview);
            notes.push(
                "quantum readiness is below minimum but emergency override allows review path"
                    .to_owned(),
            );
        } else {
            return (
                GovernanceDecision::Rejected,
                vec!["quantum readiness is below governance minimum and is rejected".to_owned()],
            );
        }
    }

    (decision, notes)
}

pub fn evaluate_governance(
    new_floor: UnitAmount,
    previous_approved_per_unit_floor: Option<UnitAmount>,
    governance: &GovernancePolicy,
    emergency_override: bool,
) -> (GovernanceDecision, Vec<String>) {
    let mut notes = Vec::new();

    if let Some(previous) = previous_approved_per_unit_floor {
        if previous.micros() > 0 && new_floor > previous {
            let increase = new_floor.micros().saturating_sub(previous.micros());
            let increase_bps = (increase * (BPS_DENOMINATOR as u128)) / previous.micros();

            notes.push(format!(
                "period floor increase detected: previous={} new={} increase_bps={}",
                previous.micros(),
                new_floor.micros(),
                increase_bps
            ));

            if increase_bps > governance.max_period_floor_increase_bps as u128 {
                if emergency_override && governance.allow_emergency_override {
                    notes.push(
                        "increase exceeds governance threshold but emergency override is active"
                            .to_owned(),
                    );
                    return (GovernanceDecision::RequiresReview, notes);
                }

                notes.push(
                    "increase exceeds governance threshold and is rejected without override"
                        .to_owned(),
                );
                return (GovernanceDecision::Rejected, notes);
            }
        } else {
            notes.push("no upward governance-sensitive floor jump detected".to_owned());
        }
    }

    notes.push("governance checks passed".to_owned());
    (GovernanceDecision::Approved, notes)
}

fn validate_bps(field: &str, value: u32) -> Result<(), EnergyError> {
    if value > BPS_DENOMINATOR {
        return Err(EnergyError::InvalidInput(format!(
            "{field} exceeds maximum basis-point value '{}'",
            BPS_DENOMINATOR
        )));
    }
    Ok(())
}
