use super::*;

pub fn cmd_tx_get(args: &[String]) -> Result<(), AppError> {
    #[derive(serde::Serialize)]
    struct TxView {
        tx_hash: String,
        known: bool,
        block_height: u64,
        execution_status: String,
        source: String,
    }

    let tx_hash = arg_value(args, "--hash")
        .and_then(|value| normalize_text(&value, false))
        .ok_or_else(|| {
            AppError::new(
                ErrorCode::UsageInvalidArguments,
                "Flag --hash must not be blank",
            )
        })?;
    let state = lifecycle::load_state()?;
    let indexed = load_tx_index_entry(&tx_hash)?;
    let known = indexed.is_some()
        || (state.last_tx != "none" && tx_hash == state.last_tx)
        || (state.last_tx != "none" && tx_hash == tx_hash_hex(&state.last_tx));

    let (block_height, execution_status, source) = if let Some(entry) = indexed {
        (
            entry.block_height,
            entry.execution_status,
            "tx-index".to_string(),
        )
    } else {
        (
            state.current_height,
            if known {
                "applied".to_string()
            } else {
                "unknown".to_string()
            },
            "runtime-last-tx".to_string(),
        )
    };

    let tx = TxView {
        tx_hash,
        known,
        block_height,
        execution_status,
        source,
    };

    emit_serialized(&tx, output_format(args))
}

pub fn cmd_tx_receipt(args: &[String]) -> Result<(), AppError> {
    #[derive(serde::Serialize)]
    struct TxReceiptView {
        tx_hash: String,
        found: bool,
        success: bool,
        gas_used: u64,
        fee_paid: u64,
        events: Vec<String>,
        logs: Vec<String>,
        state_change_summary: String,
    }

    let tx_hash = arg_value(args, "--hash")
        .and_then(|value| normalize_text(&value, false))
        .ok_or_else(|| {
            AppError::new(
                ErrorCode::UsageInvalidArguments,
                "Flag --hash must not be blank",
            )
        })?;
    let state = lifecycle::load_state()?;
    let indexed = load_tx_index_entry(&tx_hash)?;
    let found = indexed.is_some()
        || (state.last_tx != "none" && tx_hash == state.last_tx)
        || (state.last_tx != "none" && tx_hash == tx_hash_hex(&state.last_tx));
    let receipt = TxReceiptView {
        tx_hash,
        found,
        success: found,
        gas_used: indexed.as_ref().map_or(0, |entry| entry.gas_used),
        fee_paid: indexed.as_ref().map_or(0, |entry| entry.fee_paid),
        events: indexed.as_ref().map_or_else(
            || {
                if found {
                    vec!["runtime_tx_applied".to_string()]
                } else {
                    Vec::new()
                }
            },
            |entry| entry.events.clone(),
        ),
        logs: Vec::new(),
        state_change_summary: if let Some(entry) = indexed {
            entry.state_change_summary
        } else if found {
            "local runtime marker updated".to_string()
        } else {
            "receipt not found".to_string()
        },
    };

    emit_serialized(&receipt, output_format(args))
}

pub fn cmd_account_get(args: &[String]) -> Result<(), AppError> {
    #[derive(serde::Serialize)]
    struct AccountView {
        account_id: String,
        known: bool,
        balance: u64,
        nonce: u64,
        source: &'static str,
    }

    let account_id = arg_value(args, "--id")
        .and_then(|value| normalize_text(&value, false))
        .ok_or_else(|| {
            AppError::new(
                ErrorCode::UsageInvalidArguments,
                "Flag --id must not be blank",
            )
        })?;
    let ledger = ledger::load().unwrap_or_default();
    let balance = if account_id == "treasury" {
        ledger.treasury_balance
    } else {
        ledger.delegations.get(&account_id).copied().unwrap_or(0)
    };

    let account = AccountView {
        known: account_id == "treasury" || ledger.delegations.contains_key(&account_id),
        account_id,
        balance,
        nonce: 0,
        source: "local-ledger",
    };

    emit_serialized(&account, output_format(args))
}

pub fn cmd_balance_get(args: &[String]) -> Result<(), AppError> {
    #[derive(serde::Serialize)]
    struct BalanceView {
        account_id: String,
        balance: u64,
        known: bool,
        source: &'static str,
    }

    let account_id = arg_value(args, "--id")
        .and_then(|value| normalize_text(&value, false))
        .ok_or_else(|| {
            AppError::new(
                ErrorCode::UsageInvalidArguments,
                "Flag --id must not be blank",
            )
        })?;
    let ledger = ledger::load().unwrap_or_default();
    let balance = if account_id == "treasury" {
        ledger.treasury_balance
    } else {
        ledger.delegations.get(&account_id).copied().unwrap_or(0)
    };
    let response = BalanceView {
        known: account_id == "treasury" || ledger.delegations.contains_key(&account_id),
        account_id,
        balance,
        source: "local-ledger",
    };

    emit_serialized(&response, output_format(args))
}
