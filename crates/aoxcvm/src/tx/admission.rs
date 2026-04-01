use crate::constants::VM_CANONICAL_CHAIN_DOMAIN;
use crate::errors::AoxcvmError;
use crate::result::Result;
use crate::tx::envelope::TransactionEnvelope;

pub fn check_admission(tx: &TransactionEnvelope) -> Result<()> {
    if tx.chain_domain != VM_CANONICAL_CHAIN_DOMAIN {
        return Err(AoxcvmError::AdmissionRejected("unexpected chain domain"));
    }
    if tx.max_gas == 0 {
        return Err(AoxcvmError::AdmissionRejected("gas budget cannot be zero"));
    }
    if tx.max_authority == 0 {
        return Err(AoxcvmError::AdmissionRejected("authority budget cannot be zero"));
    }
    if tx.target_package.is_empty() || tx.target_entrypoint.is_empty() {
        return Err(AoxcvmError::AdmissionRejected("package target is missing"));
    }
    Ok(())
}
