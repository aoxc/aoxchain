use aoxcvm::auth::{rotation::RotationPlan, scheme::SignatureAlgorithm};

#[test]
fn rotation_plan_detects_quantum_downgrade() {
    let plan = RotationPlan {
        previous: vec![SignatureAlgorithm::MlDsa65],
        next: vec![SignatureAlgorithm::Ed25519],
    };

    assert!(!plan.preserves_quantum_continuity());
    assert!(!plan.has_overlap());
}

#[test]
fn rotation_plan_supports_gradual_pq_upgrade() {
    let plan = RotationPlan {
        previous: vec![SignatureAlgorithm::Ed25519, SignatureAlgorithm::MlDsa65],
        next: vec![SignatureAlgorithm::MlDsa65, SignatureAlgorithm::MlDsa87],
    };

    assert!(plan.preserves_quantum_continuity());
    assert!(plan.has_overlap());
}
