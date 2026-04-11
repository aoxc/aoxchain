use super::*;

pub fn cmd_faucet_config(args: &[String]) -> Result<(), AppError> {
    let mut state = load_faucet_state()?;
    let now_unix = now_unix_secs()?;
    let mut changed = false;

    if has_flag(args, "--enable") {
        state.enabled = true;
        changed = true;
    }
    if has_flag(args, "--disable") {
        state.enabled = false;
        changed = true;
    }

    if let Some(amount) = arg_value(args, "--max-claim-amount") {
        state.max_claim_amount =
            parse_positive_u64_value(&amount, "--max-claim-amount", "faucet config")?;
        changed = true;
    }

    if let Some(cooldown_secs) = arg_value(args, "--cooldown-secs") {
        state.cooldown_secs =
            parse_positive_u64_value(&cooldown_secs, "--cooldown-secs", "faucet config")?;
        changed = true;
    }

    if let Some(limit) = arg_value(args, "--daily-limit-per-account") {
        state.daily_limit_per_account =
            parse_positive_u64_value(&limit, "--daily-limit-per-account", "faucet config")?;
        changed = true;
    }

    if let Some(limit) = arg_value(args, "--daily-global-limit") {
        state.daily_global_limit =
            parse_positive_u64_value(&limit, "--daily-global-limit", "faucet config")?;
        changed = true;
    }

    if let Some(balance) = arg_value(args, "--min-reserve-balance") {
        state.min_reserve_balance =
            parse_positive_u64_value(&balance, "--min-reserve-balance", "faucet config")?;
        changed = true;
    }

    if let Some(account) = parse_optional_text_arg(args, "--ban-account", true)
        && !state.banned_accounts.contains(&account)
    {
        state.banned_accounts.push(account.clone());
        state.banned_accounts.sort();
        changed = true;
    }

    if let Some(account) = parse_optional_text_arg(args, "--unban-account", true) {
        let initial_len = state.banned_accounts.len();
        state
            .banned_accounts
            .retain(|existing| existing != &account);
        changed = changed || initial_len != state.banned_accounts.len();
    }

    if let Some(account) = parse_optional_text_arg(args, "--allow-account", true)
        && !state.allowlisted_accounts.contains(&account)
    {
        state.allowlisted_accounts.push(account.clone());
        state.allowlisted_accounts.sort();
        changed = true;
    }

    if let Some(account) = parse_optional_text_arg(args, "--disallow-account", true) {
        let initial_len = state.allowlisted_accounts.len();
        state
            .allowlisted_accounts
            .retain(|existing| existing != &account);
        changed = changed || initial_len != state.allowlisted_accounts.len();
    }

    prune_faucet_history(&mut state, now_unix);
    if changed {
        append_faucet_audit(
            &mut state,
            "config-update",
            "operator-cli",
            "Faucet configuration updated via CLI",
            now_unix,
        );
    }
    persist_faucet_state(&state)?;
    emit_serialized(&state, output_format(args))
}

pub fn cmd_faucet_config_show(args: &[String]) -> Result<(), AppError> {
    let state = load_faucet_state()?;
    emit_serialized(&state, output_format(args))
}
