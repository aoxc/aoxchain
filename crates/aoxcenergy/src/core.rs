// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt;

/// Canonical basis-point denominator.
///
/// `10_000 bps = 100.00%`
pub const BPS_DENOMINATOR: u32 = 10_000;
const MAX_SCENARIO_MULTIPLIER_BPS: u32 = 50_000;

/// Represents a non-negative fixed-point monetary amount in micro-units.
///
/// This type intentionally avoids floating-point arithmetic in the canonical
/// economic floor calculation path.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
pub struct UnitAmount {
    micros: u128,
}

impl UnitAmount {
    /// Constructs a new amount from micro-units.
    #[must_use]
    pub const fn from_micros(micros: u128) -> Self {
        Self { micros }
    }

    /// Returns the raw micro-unit representation.
    #[must_use]
    pub const fn micros(self) -> u128 {
        self.micros
    }

    /// Returns true when the amount is zero.
    #[must_use]
    pub const fn is_zero(self) -> bool {
        self.micros == 0
    }

    /// Checked addition.
    pub fn checked_add(self, other: Self) -> Result<Self, EnergyError> {
        let micros = self
            .micros
            .checked_add(other.micros)
            .ok_or(EnergyError::ArithmeticOverflow)?;
        Ok(Self { micros })
    }

    /// Checked multiplication by an integer scalar.
    pub fn checked_mul_u64(self, rhs: u64) -> Result<Self, EnergyError> {
        let micros = self
            .micros
            .checked_mul(rhs as u128)
            .ok_or(EnergyError::ArithmeticOverflow)?;
        Ok(Self { micros })
    }

    /// Applies a basis-point ratio to the amount.
    pub fn apply_bps(self, bps: u32) -> Result<Self, EnergyError> {
        if bps > BPS_DENOMINATOR {
            return Err(EnergyError::InvalidInput(format!(
                "bps '{}' exceeds denominator '{}'",
                bps, BPS_DENOMINATOR
            )));
        }

        let weighted = self
            .micros
            .checked_mul(bps as u128)
            .ok_or(EnergyError::ArithmeticOverflow)?;
        Ok(Self {
            micros: weighted / (BPS_DENOMINATOR as u128),
        })
    }

    /// Divides the amount by a strictly positive integer, rounding up.
    pub fn checked_div_u64_ceil(self, rhs: u64) -> Result<Self, EnergyError> {
        if rhs == 0 {
            return Err(EnergyError::InvalidInput(
                "division by zero is not permitted".to_owned(),
            ));
        }

        let divisor = rhs as u128;
        let micros = self.micros.div_ceil(divisor);
        Ok(Self { micros })
    }
}

/// Errors emitted by the AOXC energy and economic floor engine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EnergyError {
    InvalidInput(String),
    ArithmeticOverflow,
}

impl fmt::Display for EnergyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidInput(message) => write!(f, "invalid input: {message}"),
            Self::ArithmeticOverflow => write!(f, "arithmetic overflow"),
        }
    }
}

impl Error for EnergyError {}

/// Economic zone classification relative to the computed per-unit floor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EconomicZone {
    LossZone,
    SurvivalZone,
    TreasuryBuildZone,
}

/// Governance decision emitted after evaluating the proposed floor update.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GovernanceDecision {
    Approved,
    RequiresReview,
    Rejected,
}

/// Inputs representing direct energy expenditure.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EnergyInputs {
    /// Price per kilowatt-hour, expressed in micro-units.
    pub energy_price_per_kwh: UnitAmount,

    /// Expected energy consumption for the evaluation period.
    pub kilowatt_hours_per_period: u64,

    /// Additional cooling and power inefficiency overhead.
    pub cooling_overhead_bps: u32,
}

/// Inputs representing non-energy operational expenditure.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OperationsInputs {
    pub infrastructure_cost_per_period: UnitAmount,
    pub validator_operations_cost_per_period: UnitAmount,
    pub storage_cost_per_period: UnitAmount,
    pub bandwidth_cost_per_period: UnitAmount,
    pub maintenance_cost_per_period: UnitAmount,
}

/// Inputs representing explicit costs across AOXChain runtime layers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LayerCostInputs {
    pub kernel_layer_cost_per_period: UnitAmount,
    pub consensus_layer_cost_per_period: UnitAmount,
    pub execution_layer_cost_per_period: UnitAmount,
    pub settlement_layer_cost_per_period: UnitAmount,
    pub networking_layer_cost_per_period: UnitAmount,
}

/// Inputs representing policy-controlled economic components.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PolicyInputs {
    /// Continuity reserve for ensuring uninterrupted operation.
    pub continuity_buffer_bps: u32,

    /// Security reserve for adverse or hostile operating conditions.
    pub security_reserve_bps: u32,

    /// Additional reserve for post-quantum migration and rollout hardening.
    pub quantum_transition_reserve_bps: u32,

    /// Ongoing assurance reserve for post-quantum validation, audits, and
    /// verification operations.
    pub quantum_assurance_bps: u32,

    /// Treasury formation component for durable reserve creation.
    pub treasury_build_bps: u32,

    /// Target operating margin above sustainable cost.
    pub target_margin_bps: u32,

    /// Aggregate tax burden applied to the pre-tax full economic floor.
    pub tax_bps: u32,
}

/// Demand / output model inputs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DemandInputs {
    /// Expected number of economic units supported by the period.
    pub units_per_period: u64,
}

/// Governance constraints controlling what kind of floor changes are accepted.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GovernancePolicy {
    pub max_tax_bps: u32,
    pub max_treasury_build_bps: u32,
    pub max_quantum_reserve_bps: u32,
    pub max_period_floor_increase_bps: u32,
    pub max_kernel_layer_share_bps: u32,
    pub max_execution_layer_share_bps: u32,
    pub min_quantum_readiness_bps: u32,
    pub allow_emergency_override: bool,
}

/// Full canonical input bundle.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FloorModelInputs {
    pub energy: EnergyInputs,
    pub operations: OperationsInputs,
    pub layer_costs: LayerCostInputs,
    pub policy: PolicyInputs,
    pub demand: DemandInputs,
}

/// Detailed report produced by the engine.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EconomicFloorReport {
    pub raw_energy_cost: UnitAmount,
    pub energy_overhead_cost: UnitAmount,
    pub total_energy_cost: UnitAmount,
    pub total_operational_cost: UnitAmount,
    pub total_layer_cost: UnitAmount,
    pub kernel_layer_cost: UnitAmount,
    pub consensus_layer_cost: UnitAmount,
    pub execution_layer_cost: UnitAmount,
    pub settlement_layer_cost: UnitAmount,
    pub networking_layer_cost: UnitAmount,
    pub sustainable_cost: UnitAmount,
    pub continuity_component: UnitAmount,
    pub security_component: UnitAmount,
    pub quantum_transition_component: UnitAmount,
    pub quantum_assurance_component: UnitAmount,
    pub treasury_build_component: UnitAmount,
    pub target_margin_component: UnitAmount,
    pub pre_tax_full_cost: UnitAmount,
    pub tax_component: UnitAmount,
    pub full_network_cost_floor: UnitAmount,
    pub per_unit_floor: UnitAmount,
    pub quantum_readiness_index_bps: u32,
    pub governance_decision: GovernanceDecision,
    pub audit_notes: Vec<String>,
}

/// Component share breakdown in basis points (`10_000 = 100%`) relative to the
/// full network floor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CostShareBps {
    pub energy: u32,
    pub operations: u32,
    pub layers: u32,
    pub continuity: u32,
    pub security: u32,
    pub quantum_transition: u32,
    pub quantum_assurance: u32,
    pub treasury_build: u32,
    pub target_margin: u32,
    pub tax: u32,
}

/// Scenario projection output for demand sensitivity analysis.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScenarioFloorProjection {
    pub scenario_label: String,
    pub demand_multiplier_bps: u32,
    pub projected_units_per_period: u64,
    pub report: EconomicFloorReport,
}

/// Aggregate summary across deterministic demand scenarios.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScenarioProjectionSummary {
    pub scenario_count: usize,
    pub min_per_unit_floor: UnitAmount,
    pub max_per_unit_floor: UnitAmount,
    pub avg_per_unit_floor: UnitAmount,
}

impl EconomicFloorReport {
    /// Classifies a realized unit value against the computed floor.
    #[must_use]
    pub fn classify_realized_value(
        &self,
        realized_per_unit_value: UnitAmount,
        treasury_build_threshold_bps: u32,
    ) -> EconomicZone {
        if realized_per_unit_value < self.per_unit_floor {
            return EconomicZone::LossZone;
        }

        let markup = self
            .per_unit_floor
            .apply_bps(treasury_build_threshold_bps)
            .unwrap_or_default();

        let treasury_build_threshold = self
            .per_unit_floor
            .checked_add(markup)
            .unwrap_or(self.per_unit_floor);

        if realized_per_unit_value >= treasury_build_threshold {
            EconomicZone::TreasuryBuildZone
        } else {
            EconomicZone::SurvivalZone
        }
    }

    /// Verifies canonical arithmetic identities across computed report fields.
    #[must_use]
    pub fn is_consistent(&self) -> bool {
        let total_energy = self
            .raw_energy_cost
            .checked_add(self.energy_overhead_cost)
            .ok();
        let base_sustainable = self
            .total_energy_cost
            .checked_add(self.total_operational_cost)
            .ok()
            .and_then(|value| value.checked_add(self.total_layer_cost).ok());
        let sustainable = base_sustainable.and_then(|value| {
            value
                .checked_add(self.continuity_component)
                .ok()
                .and_then(|v| v.checked_add(self.security_component).ok())
                .and_then(|v| v.checked_add(self.quantum_transition_component).ok())
                .and_then(|v| v.checked_add(self.quantum_assurance_component).ok())
        });
        let pre_tax = self
            .sustainable_cost
            .checked_add(self.treasury_build_component)
            .ok()
            .and_then(|value| value.checked_add(self.target_margin_component).ok());
        let full = self.pre_tax_full_cost.checked_add(self.tax_component).ok();

        total_energy == Some(self.total_energy_cost)
            && sustainable == Some(self.sustainable_cost)
            && pre_tax == Some(self.pre_tax_full_cost)
            && full == Some(self.full_network_cost_floor)
    }

    /// Returns normalized component shares in basis points relative to
    /// `full_network_cost_floor`. The shares always sum to `10_000` when the
    /// floor is non-zero.
    #[must_use]
    pub fn cost_share_bps(&self) -> Option<CostShareBps> {
        if self.full_network_cost_floor.is_zero() {
            return None;
        }

        let total = self.full_network_cost_floor.micros();
        let energy = share_bps(self.total_energy_cost.micros(), total);
        let operations = share_bps(self.total_operational_cost.micros(), total);
        let layers = share_bps(self.total_layer_cost.micros(), total);
        let continuity = share_bps(self.continuity_component.micros(), total);
        let security = share_bps(self.security_component.micros(), total);
        let quantum_transition = share_bps(self.quantum_transition_component.micros(), total);
        let quantum_assurance = share_bps(self.quantum_assurance_component.micros(), total);
        let treasury_build = share_bps(self.treasury_build_component.micros(), total);
        let target_margin = share_bps(self.target_margin_component.micros(), total);

        let allocated = energy
            .saturating_add(operations)
            .saturating_add(layers)
            .saturating_add(continuity)
            .saturating_add(security)
            .saturating_add(quantum_transition)
            .saturating_add(quantum_assurance)
            .saturating_add(treasury_build)
            .saturating_add(target_margin);
        let tax = BPS_DENOMINATOR.saturating_sub(allocated);

        Some(CostShareBps {
            energy,
            operations,
            layers,
            continuity,
            security,
            quantum_transition,
            quantum_assurance,
            treasury_build,
            target_margin,
            tax,
        })
    }

    /// Returns kernel-layer ratio within the complete layer-cost bucket.
    #[must_use]
    pub fn kernel_layer_ratio_bps(&self) -> Option<u32> {
        if self.total_layer_cost.is_zero() {
            return None;
        }

        Some(share_bps(
            self.kernel_layer_cost.micros(),
            self.total_layer_cost.micros(),
        ))
    }
}

fn share_bps(part: u128, total: u128) -> u32 {
    if total == 0 {
        return 0;
    }

    let numerator = part.saturating_mul(BPS_DENOMINATOR as u128);
    (numerator / total) as u32
}

fn scale_u64_by_bps(value: u64, bps: u32) -> u64 {
    let weighted = (value as u128).saturating_mul(bps as u128);
    (weighted / BPS_DENOMINATOR as u128) as u64
}

fn quantum_readiness_index_bps(
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

/// AOXC energy and economic floor engine.
#[derive(Debug, Default, Clone, Copy)]
pub struct EnergyAnchorEngine;

impl EnergyAnchorEngine {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Computes the full network economic floor and applies governance review
    /// rules to the resulting per-unit floor.
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

    /// Runs deterministic demand projections using basis-point multipliers.
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

    /// Returns aggregate floor summary across demand scenarios.
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

        let avg = UnitAmount::from_micros(sum / projections.len() as u128);
        Ok(ScenarioProjectionSummary {
            scenario_count: projections.len(),
            min_per_unit_floor: min_floor,
            max_per_unit_floor: max_floor,
            avg_per_unit_floor: avg,
        })
    }
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

fn validate_scenario_multiplier_bps(value: u32) -> Result<(), EnergyError> {
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

fn validate_inputs(
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

fn combine_governance_decisions(
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

fn evaluate_layer_and_quantum_guardrails(
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

fn evaluate_governance(
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
