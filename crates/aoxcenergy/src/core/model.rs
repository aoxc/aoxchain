use super::math::share_bps;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt;

/// Canonical basis-point denominator.
///
/// `10_000 bps = 100.00%`
pub const BPS_DENOMINATOR: u32 = 10_000;

/// Represents a non-negative fixed-point monetary amount in micro-units.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
pub struct UnitAmount {
    micros: u128,
}

impl UnitAmount {
    #[must_use]
    pub const fn from_micros(micros: u128) -> Self {
        Self { micros }
    }

    #[must_use]
    pub const fn micros(self) -> u128 {
        self.micros
    }

    #[must_use]
    pub const fn is_zero(self) -> bool {
        self.micros == 0
    }

    pub fn checked_add(self, other: Self) -> Result<Self, EnergyError> {
        let micros = self
            .micros
            .checked_add(other.micros)
            .ok_or(EnergyError::ArithmeticOverflow)?;
        Ok(Self { micros })
    }

    pub fn checked_mul_u64(self, rhs: u64) -> Result<Self, EnergyError> {
        let micros = self
            .micros
            .checked_mul(rhs as u128)
            .ok_or(EnergyError::ArithmeticOverflow)?;
        Ok(Self { micros })
    }

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EconomicZone {
    LossZone,
    SurvivalZone,
    TreasuryBuildZone,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GovernanceDecision {
    Approved,
    RequiresReview,
    Rejected,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EnergyInputs {
    pub energy_price_per_kwh: UnitAmount,
    pub kilowatt_hours_per_period: u64,
    pub cooling_overhead_bps: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OperationsInputs {
    pub infrastructure_cost_per_period: UnitAmount,
    pub validator_operations_cost_per_period: UnitAmount,
    pub storage_cost_per_period: UnitAmount,
    pub bandwidth_cost_per_period: UnitAmount,
    pub maintenance_cost_per_period: UnitAmount,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LayerCostInputs {
    pub kernel_layer_cost_per_period: UnitAmount,
    pub consensus_layer_cost_per_period: UnitAmount,
    pub execution_layer_cost_per_period: UnitAmount,
    pub settlement_layer_cost_per_period: UnitAmount,
    pub networking_layer_cost_per_period: UnitAmount,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PolicyInputs {
    pub continuity_buffer_bps: u32,
    pub security_reserve_bps: u32,
    pub quantum_transition_reserve_bps: u32,
    pub quantum_assurance_bps: u32,
    pub treasury_build_bps: u32,
    pub target_margin_bps: u32,
    pub tax_bps: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DemandInputs {
    pub units_per_period: u64,
}

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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FloorModelInputs {
    pub energy: EnergyInputs,
    pub operations: OperationsInputs,
    pub layer_costs: LayerCostInputs,
    pub policy: PolicyInputs,
    pub demand: DemandInputs,
}

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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScenarioFloorProjection {
    pub scenario_label: String,
    pub demand_multiplier_bps: u32,
    pub projected_units_per_period: u64,
    pub report: EconomicFloorReport,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScenarioProjectionSummary {
    pub scenario_count: usize,
    pub min_per_unit_floor: UnitAmount,
    pub max_per_unit_floor: UnitAmount,
    pub avg_per_unit_floor: UnitAmount,
}

/// Full integrated request surface for single-call engine orchestration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IntegratedFloorRequest {
    pub inputs: FloorModelInputs,
    pub governance: GovernancePolicy,
    pub previous_approved_per_unit_floor: Option<UnitAmount>,
    pub emergency_override: bool,
    pub scenario_multipliers_bps: Vec<u32>,
}

/// Integrated deterministic output containing base and scenario analysis.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IntegratedFloorOutput {
    pub base_report: EconomicFloorReport,
    pub projections: Vec<ScenarioFloorProjection>,
    pub summary: ScenarioProjectionSummary,
}

impl EconomicFloorReport {
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
            tax: BPS_DENOMINATOR.saturating_sub(allocated),
        })
    }

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
