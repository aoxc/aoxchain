use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt;

/// Canonical basis-point denominator.
///
/// `10_000 bps = 100.00%`
pub const BPS_DENOMINATOR: u32 = 10_000;

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

/// Inputs representing policy-controlled economic components.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PolicyInputs {
    /// Continuity reserve for ensuring uninterrupted operation.
    pub continuity_buffer_bps: u32,

    /// Security reserve for adverse or hostile operating conditions.
    pub security_reserve_bps: u32,

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
    ///
    /// This may represent transferable units, billing units, fee-bearing units,
    /// or another governance-defined economic denominator.
    pub units_per_period: u64,
}

/// Governance constraints controlling what kind of floor changes are accepted.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GovernancePolicy {
    pub max_tax_bps: u32,
    pub max_treasury_build_bps: u32,
    pub max_period_floor_increase_bps: u32,
    pub allow_emergency_override: bool,
}

/// Full canonical input bundle.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FloorModelInputs {
    pub energy: EnergyInputs,
    pub operations: OperationsInputs,
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
    pub sustainable_cost: UnitAmount,
    pub continuity_component: UnitAmount,
    pub security_component: UnitAmount,
    pub treasury_build_component: UnitAmount,
    pub target_margin_component: UnitAmount,
    pub pre_tax_full_cost: UnitAmount,
    pub tax_component: UnitAmount,
    pub full_network_cost_floor: UnitAmount,
    pub per_unit_floor: UnitAmount,
    pub governance_decision: GovernanceDecision,
    pub audit_notes: Vec<String>,
}

/// Component share breakdown in basis points (`10_000 = 100%`) relative to the
/// full network floor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CostShareBps {
    pub energy: u32,
    pub operations: u32,
    pub continuity: u32,
    pub security: u32,
    pub treasury_build: u32,
    pub target_margin: u32,
    pub tax: u32,
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

        // HATA DÜZELTİLDİ: BPS 10.000 sınırını aşmamak için marjı (markup) hesaplayıp topluyoruz.
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
            .ok();
        let sustainable = base_sustainable.and_then(|value| {
            value
                .checked_add(self.continuity_component)
                .ok()
                .and_then(|v| v.checked_add(self.security_component).ok())
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
        let continuity = share_bps(self.continuity_component.micros(), total);
        let security = share_bps(self.security_component.micros(), total);
        let treasury_build = share_bps(self.treasury_build_component.micros(), total);
        let target_margin = share_bps(self.target_margin_component.micros(), total);

        let allocated = energy
            .saturating_add(operations)
            .saturating_add(continuity)
            .saturating_add(security)
            .saturating_add(treasury_build)
            .saturating_add(target_margin);
        let tax = BPS_DENOMINATOR.saturating_sub(allocated);

        Some(CostShareBps {
            energy,
            operations,
            continuity,
            security,
            treasury_build,
            target_margin,
            tax,
        })
    }
}

fn share_bps(part: u128, total: u128) -> u32 {
    if total == 0 {
        return 0;
    }

    let numerator = part.saturating_mul(BPS_DENOMINATOR as u128);
    (numerator / total) as u32
}

/// AOXC energy and economic floor engine.
///
/// This engine does not attempt to predict or command market price. It computes
/// a deterministic full economic floor representing the minimum sustainable and
/// treasury-aware cost basis of the network.
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

        let base_sustainable_cost = total_energy_cost.checked_add(total_operational_cost)?;
        let continuity_component =
            base_sustainable_cost.apply_bps(inputs.policy.continuity_buffer_bps)?;
        let security_component =
            base_sustainable_cost.apply_bps(inputs.policy.security_reserve_bps)?;
        let sustainable_cost = base_sustainable_cost
            .checked_add(continuity_component)?
            .checked_add(security_component)?;

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

        let (governance_decision, audit_notes) = evaluate_governance(
            per_unit_floor,
            previous_approved_per_unit_floor,
            governance,
            emergency_override,
        );

        Ok(EconomicFloorReport {
            raw_energy_cost,
            energy_overhead_cost,
            total_energy_cost,
            total_operational_cost,
            sustainable_cost,
            continuity_component,
            security_component,
            treasury_build_component,
            target_margin_component,
            pre_tax_full_cost,
            tax_component,
            full_network_cost_floor,
            per_unit_floor,
            governance_decision,
            audit_notes,
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
        "governance.max_period_floor_increase_bps",
        governance.max_period_floor_increase_bps,
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

    Ok(())
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

#[cfg(test)]
mod tests {
    use super::*;

    fn base_inputs() -> FloorModelInputs {
        FloorModelInputs {
            energy: EnergyInputs {
                energy_price_per_kwh: UnitAmount::from_micros(1_000_000),
                kilowatt_hours_per_period: 100,
                cooling_overhead_bps: 1_000,
            },
            operations: OperationsInputs {
                infrastructure_cost_per_period: UnitAmount::from_micros(20_000_000),
                validator_operations_cost_per_period: UnitAmount::from_micros(10_000_000),
                storage_cost_per_period: UnitAmount::from_micros(5_000_000),
                bandwidth_cost_per_period: UnitAmount::from_micros(3_000_000),
                maintenance_cost_per_period: UnitAmount::from_micros(2_000_000),
            },
            policy: PolicyInputs {
                continuity_buffer_bps: 1_000,
                security_reserve_bps: 500,
                treasury_build_bps: 1_500,
                target_margin_bps: 1_000,
                tax_bps: 1_800,
            },
            demand: DemandInputs {
                units_per_period: 100,
            },
        }
    }

    fn base_governance() -> GovernancePolicy {
        GovernancePolicy {
            max_tax_bps: 2_500,
            max_treasury_build_bps: 2_500,
            max_period_floor_increase_bps: 1_000,
            allow_emergency_override: true,
        }
    }

    #[test]
    fn compute_produces_non_zero_full_floor() {
        let engine = EnergyAnchorEngine::new();
        let report = engine
            .compute(&base_inputs(), &base_governance(), None, false)
            .expect("computation must succeed");

        assert!(report.full_network_cost_floor.micros() > 0);
        assert!(report.per_unit_floor.micros() > 0);
        assert_eq!(report.governance_decision, GovernanceDecision::Approved);
    }

    #[test]
    fn tax_and_treasury_components_are_included() {
        let engine = EnergyAnchorEngine::new();
        let report = engine
            .compute(&base_inputs(), &base_governance(), None, false)
            .expect("computation must succeed");

        assert!(report.treasury_build_component.micros() > 0);
        assert!(report.tax_component.micros() > 0);
        assert!(report.full_network_cost_floor > report.sustainable_cost);
    }

    #[test]
    fn excessive_tax_burden_is_rejected() {
        let engine = EnergyAnchorEngine::new();
        let mut inputs = base_inputs();
        inputs.policy.tax_bps = 9_000;

        let err = engine
            .compute(&inputs, &base_governance(), None, false)
            .expect_err("excessive tax must fail");

        assert!(matches!(err, EnergyError::InvalidInput(_)));
    }

    #[test]
    fn excessive_treasury_build_ratio_is_rejected() {
        let engine = EnergyAnchorEngine::new();
        let mut inputs = base_inputs();
        inputs.policy.treasury_build_bps = 9_000;

        let err = engine
            .compute(&inputs, &base_governance(), None, false)
            .expect_err("excessive treasury ratio must fail");

        assert!(matches!(err, EnergyError::InvalidInput(_)));
    }

    #[test]
    fn zero_throughput_is_rejected() {
        let engine = EnergyAnchorEngine::new();
        let mut inputs = base_inputs();
        inputs.demand.units_per_period = 0;

        let err = engine
            .compute(&inputs, &base_governance(), None, false)
            .expect_err("zero throughput must fail");

        assert!(matches!(err, EnergyError::InvalidInput(_)));
    }

    #[test]
    fn large_period_jump_requires_rejection_without_override() {
        let engine = EnergyAnchorEngine::new();
        let inputs = base_inputs();
        let governance = base_governance();

        let report = engine
            .compute(
                &inputs,
                &governance,
                Some(UnitAmount::from_micros(500_000)),
                false,
            )
            .expect("computation must succeed");

        assert_eq!(report.governance_decision, GovernanceDecision::Rejected);
    }

    #[test]
    fn emergency_override_downgrades_rejection_to_review() {
        let engine = EnergyAnchorEngine::new();
        let inputs = base_inputs();
        let governance = base_governance();

        let report = engine
            .compute(
                &inputs,
                &governance,
                Some(UnitAmount::from_micros(500_000)),
                true,
            )
            .expect("computation must succeed");

        assert_eq!(
            report.governance_decision,
            GovernanceDecision::RequiresReview
        );
    }

    #[test]
    fn realized_value_below_floor_is_loss_zone() {
        let engine = EnergyAnchorEngine::new();
        let report = engine
            .compute(&base_inputs(), &base_governance(), None, false)
            .expect("computation must succeed");

        let realized = UnitAmount::from_micros(report.per_unit_floor.micros().saturating_sub(1));
        let zone = report.classify_realized_value(realized, 1_000);

        assert_eq!(zone, EconomicZone::LossZone);
    }

    #[test]
    fn realized_value_above_floor_but_below_treasury_band_is_survival_zone() {
        let engine = EnergyAnchorEngine::new();
        let report = engine
            .compute(&base_inputs(), &base_governance(), None, false)
            .expect("computation must succeed");

        let realized = report
            .per_unit_floor
            .checked_add(UnitAmount::from_micros(1))
            .expect("addition must succeed");

        let zone = report.classify_realized_value(realized, 1_000);

        assert_eq!(zone, EconomicZone::SurvivalZone);
    }

    #[test]
    fn realized_value_well_above_floor_is_treasury_build_zone() {
        let engine = EnergyAnchorEngine::new();
        let report = engine
            .compute(&base_inputs(), &base_governance(), None, false)
            .expect("computation must succeed");

        // HATA DÜZELTİLDİ: apply_bps(12_000) yerine marjı alıp üstüne ekliyoruz.
        let margin = report
            .per_unit_floor
            .apply_bps(2_000)
            .expect("bps application must succeed");

        let realized = report
            .per_unit_floor
            .checked_add(margin)
            .expect("addition must succeed");

        let zone = report.classify_realized_value(realized, 1_000);

        assert_eq!(zone, EconomicZone::TreasuryBuildZone);
    }

    #[test]
    fn report_consistency_checks_pass_on_compute() {
        let engine = EnergyAnchorEngine::new();
        let report = engine
            .compute(&base_inputs(), &base_governance(), None, false)
            .expect("computation must succeed");

        assert!(report.is_consistent());
    }

    #[test]
    fn cost_share_bps_sums_to_full_denominator() {
        let engine = EnergyAnchorEngine::new();
        let report = engine
            .compute(&base_inputs(), &base_governance(), None, false)
            .expect("computation must succeed");

        let shares = report
            .cost_share_bps()
            .expect("non-zero full floor must produce shares");

        let sum = shares
            .energy
            .saturating_add(shares.operations)
            .saturating_add(shares.continuity)
            .saturating_add(shares.security)
            .saturating_add(shares.treasury_build)
            .saturating_add(shares.target_margin)
            .saturating_add(shares.tax);

        assert_eq!(sum, BPS_DENOMINATOR);
    }
}
