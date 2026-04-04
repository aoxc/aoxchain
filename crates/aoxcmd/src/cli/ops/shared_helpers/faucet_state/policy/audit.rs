use super::*;

pub(in crate::cli::ops) fn now_unix_secs() -> Result<u64, AppError> {
    let now = Utc::now().timestamp();
    u64::try_from(now).map_err(|_| {
        AppError::new(
            ErrorCode::NodeStateInvalid,
            "System clock produced a negative unix timestamp",
        )
    })
}

pub(in crate::cli::ops) fn faucet_tx_id(
    account_id: &str,
    amount: u64,
    now_unix: u64,
    nonce: usize,
) -> String {
    let mut hasher = Sha256::new();
    hasher.update(account_id.as_bytes());
    hasher.update(amount.to_le_bytes());
    hasher.update(now_unix.to_le_bytes());
    hasher.update(nonce.to_le_bytes());
    format!("faucet-{}", hex::encode(hasher.finalize()))
}

pub(in crate::cli::ops) fn append_faucet_audit(
    state: &mut FaucetState,
    action: &str,
    actor: &str,
    detail: &str,
    now_unix: u64,
) {
    state.audit_log.push(FaucetAuditRecord {
        at_unix: now_unix,
        action: action.to_string(),
        actor: actor.to_string(),
        detail: detail.to_string(),
    });
}
