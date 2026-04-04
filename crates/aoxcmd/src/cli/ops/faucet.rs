use super::*;

pub fn cmd_faucet_status(args: &[String]) -> Result<(), AppError> {
    #[derive(serde::Serialize)]
    struct FaucetStatus {
        enabled: bool,
        network_kind: String,
        treasury_balance: u64,
        total_distributed_today: u64,
        claims_today: usize,
        account_remaining_allowance: Option<u64>,
        next_eligible_claim_time: Option<u64>,
        faucet: FaucetState,
    }

    let now_unix = now_unix_secs()?;
    let account_id = parse_optional_text_arg(args, "--account-id", false);
    let settings = effective_settings_for_ops()?;
    let mut state = load_faucet_state()?;
    prune_faucet_history(&mut state, now_unix);
    let day_ago = now_unix.saturating_sub(24 * 60 * 60);
    let recent: Vec<&FaucetClaimRecord> = state
        .claims
        .iter()
        .filter(|record| record.claimed_at >= day_ago)
        .collect();
    let total_distributed_today = recent.iter().map(|record| record.amount).sum::<u64>();

    let decision = account_id
        .as_ref()
        .map(|id| evaluate_faucet_claim(&state, id, 1, now_unix, false, None, &settings.profile));
    let ledger_state = ledger::load().unwrap_or_default();

    let response = FaucetStatus {
        enabled: state.enabled,
        network_kind: settings.profile,
        treasury_balance: ledger_state.treasury_balance,
        total_distributed_today,
        claims_today: recent.len(),
        account_remaining_allowance: decision.as_ref().map(|d| d.daily_remaining),
        next_eligible_claim_time: decision.and_then(|d| d.next_eligible_claim_at),
        faucet: state,
    };

    emit_serialized(&response, output_format(args))
}

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

    if let Some(account) = parse_optional_text_arg(args, "--ban-account", true) {
        if !state.banned_accounts.contains(&account) {
            state.banned_accounts.push(account.clone());
            state.banned_accounts.sort();
            changed = true;
        }
    }

    if let Some(account) = parse_optional_text_arg(args, "--unban-account", true) {
        let initial_len = state.banned_accounts.len();
        state
            .banned_accounts
            .retain(|existing| existing != &account);
        changed = changed || initial_len != state.banned_accounts.len();
    }

    if let Some(account) = parse_optional_text_arg(args, "--allow-account", true) {
        if !state.allowlisted_accounts.contains(&account) {
            state.allowlisted_accounts.push(account.clone());
            state.allowlisted_accounts.sort();
            changed = true;
        }
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

pub fn cmd_faucet_claim(args: &[String]) -> Result<(), AppError> {
    #[derive(serde::Serialize)]
    struct FaucetClaimResponse {
        tx_hash: String,
        status: &'static str,
        account_id: String,
        amount: u64,
        claimed_last_24h: u64,
        daily_remaining: u64,
        global_remaining: u64,
        cooldown_remaining_secs: u64,
        next_eligible_claim_at: Option<u64>,
        claims_total: usize,
        automation_hint: &'static str,
        ledger: crate::economy::ledger::LedgerState,
    }

    let account_id =
        parse_required_or_default_text_arg(args, "--account-id", "testnet-user", false)?;
    let force = has_flag(args, "--force");
    let auto_init = has_flag(args, "--auto-init");
    let mut state = load_faucet_state()?;
    let settings = effective_settings_for_ops()?;
    let now_unix = now_unix_secs()?;
    prune_faucet_history(&mut state, now_unix);
    let mut ledger_snapshot = ledger::load().unwrap_or_default();

    if !state.enabled && !force {
        return Err(AppError::new(
            ErrorCode::PolicyGateFailed,
            "Faucet is disabled for this profile; use --force only for controlled tests",
        ));
    }

    let amount = parse_positive_u64_arg(args, "--amount", state.max_claim_amount, "faucet claim")?;

    let decision = evaluate_faucet_claim(
        &state,
        &account_id,
        amount,
        now_unix,
        force,
        Some(ledger_snapshot.treasury_balance),
        &settings.profile,
    );
    if !decision.allowed {
        append_faucet_audit(
            &mut state,
            "claim-denied",
            "operator-cli",
            &decision
                .denied_reason
                .clone()
                .unwrap_or_else(|| "Faucet claim denied".to_string()),
            now_unix,
        );
        persist_faucet_state(&state)?;
        return Err(AppError::new(
            ErrorCode::PolicyGateFailed,
            decision
                .denied_reason
                .unwrap_or_else(|| "Faucet claim was denied".to_string()),
        ));
    }

    let ledger_result = match ledger::delegate(&account_id, amount) {
        Ok(ledger) => ledger,
        Err(error) if auto_init && error.kind() == ErrorCode::FilesystemIoFailed => {
            let _ = ledger::init()?;
            ledger::delegate(&account_id, amount)?
        }
        Err(error) => return Err(error),
    };

    let tx_id = faucet_tx_id(&account_id, amount, now_unix, state.claims.len());
    state.claims.push(FaucetClaimRecord {
        account_id: account_id.clone(),
        amount,
        claimed_at: now_unix,
        tx_hash: tx_id.clone(),
        status: "confirmed".to_string(),
    });
    append_faucet_audit(
        &mut state,
        "claim-approved",
        "operator-cli",
        &format!("account_id={account_id} amount={amount} tx_hash={tx_id}"),
        now_unix,
    );
    persist_faucet_state(&state)?;
    ledger_snapshot = ledger_result.clone();

    let response = FaucetClaimResponse {
        tx_hash: tx_id,
        status: "confirmed",
        account_id,
        amount,
        cooldown_remaining_secs: state.cooldown_secs,
        claimed_last_24h: decision.claimed_last_24h.saturating_add(amount),
        daily_remaining: decision.daily_remaining.saturating_sub(amount),
        global_remaining: decision.global_remaining.saturating_sub(amount),
        next_eligible_claim_at: Some(now_unix.saturating_add(state.cooldown_secs)),
        claims_total: state.claims.len(),
        automation_hint: "Use --format json for CI/CD scripts and --auto-init for first-run ephemeral homes.",
        ledger: ledger_snapshot,
    };

    emit_serialized(&response, output_format(args))
}

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

pub fn cmd_faucet_history(args: &[String]) -> Result<(), AppError> {
    let account_id =
        parse_required_or_default_text_arg(args, "--account-id", "testnet-user", false)?;
    let mut state = load_faucet_state()?;
    prune_faucet_history(&mut state, now_unix_secs()?);
    let claims = state
        .claims
        .into_iter()
        .filter(|claim| claim.account_id == account_id)
        .collect::<Vec<_>>();
    emit_serialized(&claims, output_format(args))
}

pub fn cmd_faucet_balance(args: &[String]) -> Result<(), AppError> {
    #[derive(serde::Serialize)]
    struct FaucetBalance {
        treasury_balance: u64,
        reserve_floor: u64,
        available_for_faucet: u64,
    }

    let state = load_faucet_state()?;
    let ledger = ledger::load().unwrap_or_default();
    let response = FaucetBalance {
        treasury_balance: ledger.treasury_balance,
        reserve_floor: state.min_reserve_balance,
        available_for_faucet: ledger
            .treasury_balance
            .saturating_sub(state.min_reserve_balance),
    };
    emit_serialized(&response, output_format(args))
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

pub fn cmd_faucet_config_show(args: &[String]) -> Result<(), AppError> {
    let state = load_faucet_state()?;
    emit_serialized(&state, output_format(args))
}

pub fn cmd_faucet_audit(args: &[String]) -> Result<(), AppError> {
    let mut state = load_faucet_state()?;
    prune_faucet_history(&mut state, now_unix_secs()?);
    emit_serialized(&state.audit_log, output_format(args))
}
