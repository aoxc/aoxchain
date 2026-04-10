use super::governance::{
    combine_governance_decisions, evaluate_governance, evaluate_layer_and_quantum_guardrails,
    validate_inputs, validate_scenario_multiplier_bps,
};
use super::math::{quantum_readiness_index_bps, scale_u64_by_bps};
use super::model::{
    EconomicFloorReport, EnergyError, FloorModelInputs, GovernancePolicy, ScenarioFloorProjection,
    ScenarioProjectionSummary, UnitAmount,
};

#[derive(Debug, Default, Clone, Copy)]
pub struct EnergyAnchorEngine;

impl EnergyAnchorEngine {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    pub fn compute(
        &self,
        inputs: &FloorModelInputs,
        governance: &GovernancePolicy,
        previous_approved_per_unit_floor: Option<UnitAmount>,
        emergency_override: bool,
    ) -> Result<EconomicFloorReport, EnergyError> {
        validate_inputs(inputs, governance)?;

        let raw_energy_cost = inputs
            .energy
            .energy_price_per_kwh
            .checked_mul_u64(inputs.energy.kilowatt_hours_per_period)?;
        let energy_overhead_cost = raw_energy_cost.apply_bps(inputs.energy.cooling_overhead_bps)?;
        let total_energy_cost = raw_energy_cost.checked_add(energy_overhead_cost)?;

        let total_operational_cost = inputs
            .operations
            .infrastructure_cost_per_period
            .checked_add(inputs.operations.validator_operations_cost_per_period)?
            .checked_add(inputs.operations.storage_cost_per_period)?
            .checked_add(inputs.operations.bandwidth_cost_per_period)?
            .checked_add(inputs.operations.maintenance_cost_per_period)?;

        let total_layer_cost = inputs
            .layer_costs
            .kernel_layer_cost_per_period
            .checked_add(inputs.layer_costs.consensus_layer_cost_per_period)?
            .checked_add(inputs.layer_costs.execution_layer_cost_per_period)?
            .checked_add(inputs.layer_costs.settlement_layer_cost_per_period)?
            .checked_add(inputs.layer_costs.networking_layer_cost_per_period)?;

        let base_sustainable_cost = total_energy_cost
            .checked_add(total_operational_cost)?
            .checked_add(total_layer_cost)?;
        let continuity_component =
            base_sustainable_cost.apply_bps(inputs.policy.continuity_buffer_bps)?;
        let security_component =
            base_sustainable_cost.apply_bps(inputs.policy.security_reserve_bps)?;
        let quantum_transition_component =
            base_sustainable_cost.apply_bps(inputs.policy.quantum_transition_reserve_bps)?;
        let quantum_assurance_component =
            base_sustainable_cost.apply_bps(inputs.policy.quantum_assurance_bps)?;
        let sustainable_cost = base_sustainable_cost
            .checked_add(continuity_component)?
            .checked_add(security_component)?
            .checked_add(quantum_transition_component)?
            .checked_add(quantum_assurance_component)?;

        let treasury_build_component =
            sustainable_cost.apply_bps(inputs.policy.treasury_build_bps)?;
        let target_margin_component =
            sustainable_cost.apply_bps(inputs.policy.target_margin_bps)?;
        let pre_tax_full_cost = sustainable_cost
            .checked_add(treasury_build_component)?
            .checked_add(target_margin_component)?;

        let tax_component = pre_tax_full_cost.apply_bps(inputs.policy.tax_bps)?;
        let full_network_cost_floor = pre_tax_full_cost.checked_add(tax_component)?;
        let per_unit_floor =
            full_network_cost_floor.checked_div_u64_ceil(inputs.demand.units_per_period)?;
        let quantum_readiness_index_bps = quantum_readiness_index_bps(
            sustainable_cost,
            quantum_transition_component,
            quantum_assurance_component,
            security_component,
        );

        let (period_decision, mut audit_notes) = evaluate_governance(
            per_unit_floor,
            previous_approved_per_unit_floor,
            governance,
            emergency_override,
        );
        let (guardrail_decision, guardrail_notes) = evaluate_layer_and_quantum_guardrails(
            total_layer_cost,
            inputs.layer_costs.kernel_layer_cost_per_period,
            inputs.layer_costs.execution_layer_cost_per_period,
            quantum_readiness_index_bps,
            governance,
            emergency_override,
        );
        audit_notes.extend(guardrail_notes);
        let governance_decision = combine_governance_decisions(period_decision, guardrail_decision);

        Ok(EconomicFloorReport {
            raw_energy_cost,
            energy_overhead_cost,
            total_energy_cost,
            total_operational_cost,
            total_layer_cost,
            kernel_layer_cost: inputs.layer_costs.kernel_layer_cost_per_period,
            consensus_layer_cost: inputs.layer_costs.consensus_layer_cost_per_period,
            execution_layer_cost: inputs.layer_costs.execution_layer_cost_per_period,
            settlement_layer_cost: inputs.layer_costs.settlement_layer_cost_per_period,
            networking_layer_cost: inputs.layer_costs.networking_layer_cost_per_period,
            sustainable_cost,
            continuity_component,
            security_component,
            quantum_transition_component,
            quantum_assurance_component,
            treasury_build_component,
            target_margin_component,
            pre_tax_full_cost,
            tax_component,
            full_network_cost_floor,
            per_unit_floor,
            quantum_readiness_index_bps,
            governance_decision,
            audit_notes,
        })
    }

    pub fn project_multi_scenario(
        &self,
        base_inputs: &FloorModelInputs,
        governance: &GovernancePolicy,
        previous_approved_per_unit_floor: Option<UnitAmount>,
        emergency_override: bool,
        demand_multipliers_bps: &[u32],
    ) -> Result<Vec<ScenarioFloorProjection>, EnergyError> {
        if demand_multipliers_bps.is_empty() {
            return Err(EnergyError::InvalidInput(
                "demand_multipliers_bps must contain at least one scenario".to_owned(),
            ));
        }

        let mut projections = Vec::with_capacity(demand_multipliers_bps.len());
        for multiplier in demand_multipliers_bps {
            validate_scenario_multiplier_bps(*multiplier)?;

            let mut scenario_inputs = base_inputs.clone();
            scenario_inputs.demand.units_per_period =
                scale_u64_by_bps(base_inputs.demand.units_per_period, *multiplier).max(1);

            let report = self.compute(
                &scenario_inputs,
                governance,
                previous_approved_per_unit_floor,
                emergency_override,
            )?;

            projections.push(ScenarioFloorProjection {
                scenario_label: format!("demand-{}bps", multiplier),
                demand_multiplier_bps: *multiplier,
                projected_units_per_period: scenario_inputs.demand.units_per_period,
                report,
            });
        }

        Ok(projections)
    }

    pub fn summarize_projection(
        &self,
        projections: &[ScenarioFloorProjection],
    ) -> Result<ScenarioProjectionSummary, EnergyError> {
        if projections.is_empty() {
            return Err(EnergyError::InvalidInput(
                "projections must contain at least one scenario".to_owned(),
            ));
        }

        let mut min_floor = projections[0].report.per_unit_floor;
        let mut max_floor = projections[0].report.per_unit_floor;
        let mut sum: u128 = 0;

        for projection in projections {
            let floor = projection.report.per_unit_floor;
            if floor < min_floor {
                min_floor = floor;
            }
            if floor > max_floor {
                max_floor = floor;
            }
            sum = sum
                .checked_add(floor.micros())
                .ok_or(EnergyError::ArithmeticOverflow)?;
        }

        Ok(ScenarioProjectionSummary {
            scenario_count: projections.len(),
            min_per_unit_floor: min_floor,
            max_per_unit_floor: max_floor,
            avg_per_unit_floor: UnitAmount::from_micros(sum / projections.len() as u128),
        })
    }
}
