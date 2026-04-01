use crate::auth::verifier::{verify_envelope, AuthVerifierPolicy};
use crate::result::Result;
use crate::tx::admission::check_admission;
use crate::tx::envelope::TransactionEnvelope;

pub fn validate_transaction(
    tx: &TransactionEnvelope,
    expected_nonce: u64,
    current_epoch: u64,
    policy: AuthVerifierPolicy,
) -> Result<()> {
    check_admission(tx)?;
    verify_envelope(&tx.auth, expected_nonce, current_epoch, policy)?;
    Ok(())
}
