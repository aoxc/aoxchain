use super::*;

pub(in crate::cli::ops) fn evaluate_faucet_claim(
    state: &FaucetState,
    account_id: &str,
    amount: u64,
    now_unix: u64,
    force: bool,
    treasury_balance: Option<u64>,
    network_kind: &str,
) -> FaucetClaimDecision {
    if network_kind == "mainnet" {
        return FaucetClaimDecision {
            allowed: false,
            cooldown_remaining_secs: 0,
            claimed_last_24h: 0,
            daily_remaining: state.daily_limit_per_account,
            global_distributed_last_24h: 0,
            global_remaining: state.daily_global_limit,
            next_eligible_claim_at: None,
            denied_reason: Some("Mainnet profile does not allow faucet claims".to_string()),
        };
    }

    let day_ago = now_unix.saturating_sub(24 * 60 * 60);
    let relevant_claims: Vec<&FaucetClaimRecord> = state
        .claims
        .iter()
        .filter(|claim| claim.account_id == account_id)
        .collect();
    let global_recent_claims: Vec<&FaucetClaimRecord> = state
        .claims
        .iter()
        .filter(|claim| claim.claimed_at >= day_ago)
        .collect();

    let claimed_last_24h = relevant_claims
        .iter()
        .filter(|claim| claim.claimed_at >= day_ago)
        .map(|claim| claim.amount)
        .sum::<u64>();

    let global_distributed_last_24h = global_recent_claims.iter().map(|claim| claim.amount).sum();

    let latest_claim = relevant_claims
        .iter()
        .max_by_key(|claim| claim.claimed_at)
        .copied();

    let cooldown_remaining_secs = latest_claim
        .map(|claim| {
            let unlock_at = claim.claimed_at.saturating_add(state.cooldown_secs);
            unlock_at.saturating_sub(now_unix)
        })
        .unwrap_or(0);

    let daily_remaining = state
        .daily_limit_per_account
        .saturating_sub(claimed_last_24h);
    let global_remaining = state
        .daily_global_limit
        .saturating_sub(global_distributed_last_24h);
    let next_eligible_claim_at = if cooldown_remaining_secs > 0 {
        Some(now_unix.saturating_add(cooldown_remaining_secs))
    } else {
        None
    };

    if force {
        return FaucetClaimDecision {
            allowed: true,
            cooldown_remaining_secs,
            claimed_last_24h,
            daily_remaining,
            global_distributed_last_24h,
            global_remaining,
            next_eligible_claim_at,
            denied_reason: None,
        };
    }

    if state
        .banned_accounts
        .iter()
        .any(|entry| entry == account_id)
    {
        return FaucetClaimDecision {
            allowed: false,
            cooldown_remaining_secs,
            claimed_last_24h,
            daily_remaining,
            global_distributed_last_24h,
            global_remaining,
            next_eligible_claim_at,
            denied_reason: Some("Account is banned from faucet".to_string()),
        };
    }

    if !state.allowlisted_accounts.is_empty()
        && !state
            .allowlisted_accounts
            .iter()
            .any(|entry| entry == account_id)
    {
        return FaucetClaimDecision {
            allowed: false,
            cooldown_remaining_secs,
            claimed_last_24h,
            daily_remaining,
            global_distributed_last_24h,
            global_remaining,
            next_eligible_claim_at,
            denied_reason: Some("Account is not in faucet allowlist".to_string()),
        };
    }

    if amount > state.max_claim_amount {
        return FaucetClaimDecision {
            allowed: false,
            cooldown_remaining_secs,
            claimed_last_24h,
            daily_remaining,
            global_distributed_last_24h,
            global_remaining,
            next_eligible_claim_at,
            denied_reason: Some(format!(
                "Requested amount exceeds max claim amount (max={})",
                state.max_claim_amount
            )),
        };
    }

    if cooldown_remaining_secs > 0 {
        return FaucetClaimDecision {
            allowed: false,
            cooldown_remaining_secs,
            claimed_last_24h,
            daily_remaining,
            global_distributed_last_24h,
            global_remaining,
            next_eligible_claim_at,
            denied_reason: Some(format!(
                "Cooldown is active; try again in {} seconds",
                cooldown_remaining_secs
            )),
        };
    }

    if claimed_last_24h.saturating_add(amount) > state.daily_limit_per_account {
        return FaucetClaimDecision {
            allowed: false,
            cooldown_remaining_secs,
            claimed_last_24h,
            daily_remaining,
            global_distributed_last_24h,
            global_remaining,
            next_eligible_claim_at,
            denied_reason: Some(format!(
                "Daily limit exceeded for account (limit={})",
                state.daily_limit_per_account
            )),
        };
    }

    if global_distributed_last_24h.saturating_add(amount) > state.daily_global_limit {
        return FaucetClaimDecision {
            allowed: false,
            cooldown_remaining_secs,
            claimed_last_24h,
            daily_remaining,
            global_distributed_last_24h,
            global_remaining,
            next_eligible_claim_at,
            denied_reason: Some(format!(
                "Daily global faucet limit exceeded (limit={})",
                state.daily_global_limit
            )),
        };
    }

    if let Some(balance) = treasury_balance
        && balance.saturating_sub(amount) < state.min_reserve_balance
    {
        return FaucetClaimDecision {
            allowed: false,
            cooldown_remaining_secs,
            claimed_last_24h,
            daily_remaining,
            global_distributed_last_24h,
            global_remaining,
            next_eligible_claim_at,
            denied_reason: Some(format!(
                "Reserve floor check failed (min_reserve_balance={})",
                state.min_reserve_balance
            )),
        };
    }

    FaucetClaimDecision {
        allowed: true,
        cooldown_remaining_secs,
        claimed_last_24h,
        daily_remaining,
        global_distributed_last_24h,
        global_remaining,
        next_eligible_claim_at,
        denied_reason: None,
    }
}

pub(in crate::cli::ops) fn prune_faucet_history(state: &mut FaucetState, now_unix: u64) {
    let retention = ChronoDuration::hours(48).num_seconds().unsigned_abs();
    let oldest = now_unix.saturating_sub(retention);
    state.claims.retain(|claim| claim.claimed_at >= oldest);
    let audit_retention = ChronoDuration::hours(FAUCET_AUDIT_RETENTION_HOURS)
        .num_seconds()
        .unsigned_abs();
    let audit_oldest = now_unix.saturating_sub(audit_retention);
    state
        .audit_log
        .retain(|entry| entry.at_unix >= audit_oldest);
}
