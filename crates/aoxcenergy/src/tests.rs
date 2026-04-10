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
        layer_costs: LayerCostInputs {
            kernel_layer_cost_per_period: UnitAmount::from_micros(4_000_000),
            consensus_layer_cost_per_period: UnitAmount::from_micros(3_000_000),
            execution_layer_cost_per_period: UnitAmount::from_micros(2_000_000),
            settlement_layer_cost_per_period: UnitAmount::from_micros(1_500_000),
            networking_layer_cost_per_period: UnitAmount::from_micros(1_000_000),
        },
        policy: PolicyInputs {
            continuity_buffer_bps: 1_000,
            security_reserve_bps: 500,
            quantum_transition_reserve_bps: 250,
            quantum_assurance_bps: 250,
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
        max_quantum_reserve_bps: 1_000,
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
fn layer_costs_are_tracked_and_consistent() {
    let engine = EnergyAnchorEngine::new();
    let report = engine
        .compute(&base_inputs(), &base_governance(), None, false)
        .expect("computation must succeed");

    let recomputed = report
        .kernel_layer_cost
        .checked_add(report.consensus_layer_cost)
        .expect("addition must succeed")
        .checked_add(report.execution_layer_cost)
        .expect("addition must succeed")
        .checked_add(report.settlement_layer_cost)
        .expect("addition must succeed")
        .checked_add(report.networking_layer_cost)
        .expect("addition must succeed");

    assert_eq!(recomputed, report.total_layer_cost);
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
fn excessive_quantum_reserve_is_rejected() {
    let engine = EnergyAnchorEngine::new();
    let mut inputs = base_inputs();
    inputs.policy.quantum_transition_reserve_bps = 700;
    inputs.policy.quantum_assurance_bps = 500;

    let err = engine
        .compute(&inputs, &base_governance(), None, false)
        .expect_err("excessive quantum reserve must fail");

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
fn quantum_readiness_index_is_non_zero_for_quantum_budget() {
    let engine = EnergyAnchorEngine::new();
    let report = engine
        .compute(&base_inputs(), &base_governance(), None, false)
        .expect("computation must succeed");

    assert!(report.quantum_readiness_index_bps > 0);
    assert!(report.quantum_readiness_index_bps <= BPS_DENOMINATOR);
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
        .saturating_add(shares.layers)
        .saturating_add(shares.continuity)
        .saturating_add(shares.security)
        .saturating_add(shares.quantum_transition)
        .saturating_add(shares.quantum_assurance)
        .saturating_add(shares.treasury_build)
        .saturating_add(shares.target_margin)
        .saturating_add(shares.tax);

    assert_eq!(sum, BPS_DENOMINATOR);
}

#[test]
fn kernel_layer_ratio_is_reported() {
    let engine = EnergyAnchorEngine::new();
    let report = engine
        .compute(&base_inputs(), &base_governance(), None, false)
        .expect("computation must succeed");

    let ratio = report
        .kernel_layer_ratio_bps()
        .expect("layer ratio must exist for non-zero layer cost");
    assert!(ratio > 0);
}

#[test]
fn project_multi_scenario_returns_deterministic_projections() {
    let engine = EnergyAnchorEngine::new();
    let base = base_inputs();
    let projections = engine
        .project_multi_scenario(
            &base,
            &base_governance(),
            None,
            false,
            &[8_000, 10_000, 12_000],
        )
        .expect("projection must succeed");

    assert_eq!(projections.len(), 3);
    assert_eq!(projections[0].projected_units_per_period, 80);
    assert_eq!(projections[1].projected_units_per_period, 100);
    assert_eq!(projections[2].projected_units_per_period, 120);

    assert!(projections[0].report.per_unit_floor > projections[2].report.per_unit_floor);
}
