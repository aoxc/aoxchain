use super::*;

pub fn cmd_faucet_reset(args: &[String]) -> Result<(), AppError> {
    let keep_config = has_flag(args, "--keep-config");
    let now_unix = now_unix_secs()?;
    let state = if keep_config {
        let current = load_faucet_state()?;
        FaucetState {
            enabled: current.enabled,
            max_claim_amount: current.max_claim_amount,
            cooldown_secs: current.cooldown_secs,
            daily_limit_per_account: current.daily_limit_per_account,
            daily_global_limit: current.daily_global_limit,
            min_reserve_balance: current.min_reserve_balance,
            claims: Vec::new(),
            banned_accounts: current.banned_accounts,
            allowlisted_accounts: current.allowlisted_accounts,
            audit_log: current.audit_log,
        }
    } else {
        FaucetState::default()
    };
    let mut state = state;
    append_faucet_audit(
        &mut state,
        "reset",
        "operator-cli",
        "Faucet state reset via CLI",
        now_unix,
    );
    persist_faucet_state(&state)?;
    emit_serialized(&state, output_format(args))
}

pub fn cmd_faucet_enable(args: &[String]) -> Result<(), AppError> {
    let settings = effective_settings_for_ops()?;
    if settings.profile == "mainnet" {
        return Err(AppError::new(
            ErrorCode::PolicyGateFailed,
            "Mainnet profile cannot enable faucet",
        ));
    }
    let mut state = load_faucet_state()?;
    state.enabled = true;
    let now_unix = now_unix_secs()?;
    append_faucet_audit(
        &mut state,
        "enabled",
        "operator-cli",
        "Faucet enabled",
        now_unix,
    );
    persist_faucet_state(&state)?;
    emit_serialized(&state, output_format(args))
}

pub fn cmd_faucet_disable(args: &[String]) -> Result<(), AppError> {
    let mut state = load_faucet_state()?;
    state.enabled = false;
    let now_unix = now_unix_secs()?;
    append_faucet_audit(
        &mut state,
        "disabled",
        "operator-cli",
        "Faucet disabled",
        now_unix,
    );
    persist_faucet_state(&state)?;
    emit_serialized(&state, output_format(args))
}

pub fn cmd_faucet_audit(args: &[String]) -> Result<(), AppError> {
    let mut state = load_faucet_state()?;
    prune_faucet_history(&mut state, now_unix_secs()?);
    emit_serialized(&state.audit_log, output_format(args))
}
