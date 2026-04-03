use super::*;

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
