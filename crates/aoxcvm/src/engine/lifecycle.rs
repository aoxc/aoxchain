use crate::auth::verifier::AuthVerifierPolicy;
use crate::engine::executor::execute;
use crate::engine::finalization::FinalizationOutcome;
use crate::policy::vm_policy::VmPolicy;
use crate::result::Result;
use crate::state::overlay::StateOverlay;
use crate::tx::envelope::TransactionEnvelope;
use crate::tx::validation::validate_transaction;

pub fn run_transaction(
    tx: &TransactionEnvelope,
    expected_nonce: u64,
    current_epoch: u64,
    auth_policy: AuthVerifierPolicy,
    vm_policy: &VmPolicy,
) -> Result<FinalizationOutcome> {
    validate_transaction(tx, expected_nonce, current_epoch, auth_policy)?;
    let mut overlay = StateOverlay::default();
    let _machine = execute(tx, vm_policy, &mut overlay)?;

    Ok(FinalizationOutcome {
        success: true,
        diff: overlay.pending,
    })
}
