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
