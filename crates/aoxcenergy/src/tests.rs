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
        max_kernel_layer_share_bps: 4_000,
        max_execution_layer_share_bps: 3_500,
        min_quantum_readiness_bps: 800,
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
fn economic_zone_classification_works() {
    let engine = EnergyAnchorEngine::new();
    let report = engine
        .compute(&base_inputs(), &base_governance(), None, false)
        .expect("computation must succeed");

    // Loss Zone
    let loss_val = UnitAmount::from_micros(report.per_unit_floor.micros().saturating_sub(1));
    assert_eq!(report.classify_realized_value(loss_val, 1_000), EconomicZone::LossZone);

    // Survival Zone
    let survival_val = report.per_unit_floor.checked_add(UnitAmount::from_micros(1)).unwrap();
    assert_eq!(report.classify_realized_value(survival_val, 1_000), EconomicZone::SurvivalZone);
}

#[test]
fn quantum_readiness_logic_checks() {
    let engine = EnergyAnchorEngine::new();
    let report = engine
        .compute(&base_inputs(), &base_governance(), None, false)
        .expect("computation must succeed");

    assert!(report.quantum_readiness_index_bps > 0);
    
    // Test rejection on low readiness
    let mut inputs = base_inputs();
    inputs.policy.quantum_transition_reserve_bps = 0;
    inputs.policy.quantum_assurance_bps = 0;
    
    let report_low = engine
        .compute(&inputs, &base_governance(), None, false)
        .expect("computation must succeed");
    
    assert_eq!(report_low.governance_decision, GovernanceDecision::Rejected);
}

#[test]
fn compute_integrated_full_flow() {
    let engine = EnergyAnchorEngine::new();
    let request = IntegratedFloorRequest {
        inputs: base_inputs(),
        governance: base_governance(),
        previous_approved_per_unit_floor: None,
        emergency_override: false,
        scenario_multipliers_bps: vec![8_000, 10_000, 12_000],
    };

    let integrated = engine
        .compute_integrated(&request)
        .expect("integrated compute must succeed");

    assert_eq!(integrated.summary.scenario_count, 3);
    assert_eq!(integrated.projections.len(), 3);
    assert!(integrated.base_report.is_consistent());
}
