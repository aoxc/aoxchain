use aoxcvm::{
    context::{call::CallContext, deterministic::DeterminismLimits},
    gas::meter::{GasError, GasMeter},
    receipts::{
        outcome::{ExecutionReceipt, ReceiptStatus},
        proof::ReceiptProof,
    },
    state::JournaledState,
    tx::replay::NonceWindow,
};

fn mutate(seed: &mut u64) -> u8 {
    *seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
    (*seed >> 32) as u8
}

#[test]
fn adversarial_receipt_proof_fuzz_closure() {
    let mut seed = 0xA0C2_2026_u64;
    for i in 0..512_u64 {
        let mut state = JournaledState::default();
        let writes = (mutate(&mut seed) % 8 + 1) as usize;
        for j in 0..writes {
            let key = vec![j as u8, mutate(&mut seed)];
            let value = vec![mutate(&mut seed), mutate(&mut seed), mutate(&mut seed)];
            state.put(key, value);
        }

        let status = match mutate(&mut seed) % 3 {
            0 => ReceiptStatus::Success,
            1 => ReceiptStatus::Reverted,
            _ => ReceiptStatus::Failed,
        };

        let receipt = ExecutionReceipt::from_state(status, i, vec![format!("iter-{i}")], &state);
        let proof = ReceiptProof::new(&receipt, 3);
        assert!(proof.verify_receipt(&receipt));

        let mut tampered = receipt.clone();
        tampered.gas_used = tampered.gas_used.saturating_add(1);
        assert!(!proof.verify_receipt(&tampered));
    }
}

#[test]
fn property_nonce_verifier_monotonicity() {
    let mut window = NonceWindow::default();

    for nonce in 1..1000_u64 {
        assert!(window.check_and_update(nonce));
        assert!(!window.check_and_update(nonce));
        assert!(!window.check_and_update(nonce.saturating_sub(1)));
    }
}

#[test]
fn call_model_nested_return_revert_and_storage_rollback() {
    let limits = DeterminismLimits::default();
    let mut state = JournaledState::default();

    let root = CallContext::new(0);
    assert!(root.depth < limits.max_call_depth);

    let cp_root = state.checkpoint();
    state.put(b"root".to_vec(), b"committed".to_vec());

    let child = CallContext::new(root.depth + 1);
    assert!(child.depth < limits.max_call_depth);

    let cp_child = state.checkpoint();
    state.put(b"child".to_vec(), b"ephemeral".to_vec());

    // Simulate child revert propagation.
    state
        .rollback(cp_child)
        .expect("child rollback must succeed");
    assert_eq!(state.get(b"child"), None);

    // Root call still commits.
    state.commit(cp_root).expect("root commit must succeed");
    assert_eq!(state.get(b"root"), Some(&b"committed"[..]));
}

#[test]
fn call_depth_edge_case_is_bounded() {
    let limits = DeterminismLimits::default();
    let near_limit = CallContext::new(limits.max_call_depth - 1);
    assert!(near_limit.depth < limits.max_call_depth);

    let at_limit = CallContext::new(limits.max_call_depth);
    assert!(!(at_limit.depth < limits.max_call_depth));
}

#[test]
fn gas_economics_cost_envelope_and_dos_guard() {
    // Canonical synthetic table for closure-gate stress accounting.
    const COST_ADD: u64 = 3;
    const COST_STORAGE_WRITE: u64 = 200;
    const COST_VERIFY_PQ_SIG: u64 = 5_000;

    assert!(COST_ADD < COST_STORAGE_WRITE);
    assert!(COST_STORAGE_WRITE < COST_VERIFY_PQ_SIG);

    let mut meter = GasMeter::new(10_000);

    for _ in 0..100 {
        meter.charge(COST_ADD).expect("add cost must fit envelope");
    }
    for _ in 0..25 {
        meter
            .charge(COST_STORAGE_WRITE)
            .expect("storage write cost must fit envelope");
    }

    let remaining_before_pq = meter.remaining();
    assert!(remaining_before_pq < COST_VERIFY_PQ_SIG);

    let out = meter.charge(COST_VERIFY_PQ_SIG);
    assert_eq!(out, Err(GasError::OutOfGas));
}
