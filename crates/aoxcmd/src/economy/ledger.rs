use crate::{
    data_home::{read_file, resolve_home, write_file},
    error::{AppError, ErrorCode},
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, path::PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LedgerState {
    pub treasury_balance: u64,
    pub transfers: u64,
    pub delegations: BTreeMap<String, u64>,
    pub updated_at: String,
}

impl Default for LedgerState {
    fn default() -> Self {
        Self::new()
    }
}

impl LedgerState {
    pub fn new() -> Self {
        Self {
            treasury_balance: 1_000_000_000_000,
            transfers: 0,
            delegations: BTreeMap::new(),
            updated_at: Utc::now().to_rfc3339(),
        }
    }
}

pub fn ledger_path() -> Result<PathBuf, AppError> {
    Ok(resolve_home()?.join("ledger").join("ledger.json"))
}

pub fn load() -> Result<LedgerState, AppError> {
    let path = ledger_path()?;
    let raw = read_file(&path).map_err(|_| {
        AppError::new(
            ErrorCode::LedgerInvalid,
            format!("Ledger file is missing at {}", path.display()),
        )
    })?;
    serde_json::from_str(&raw).map_err(|e| {
        AppError::with_source(ErrorCode::LedgerInvalid, "Failed to parse ledger state", e)
    })
}

pub fn persist(ledger: &LedgerState) -> Result<(), AppError> {
    let path = ledger_path()?;
    let content = serde_json::to_string_pretty(ledger).map_err(|e| {
        AppError::with_source(
            ErrorCode::OutputEncodingFailed,
            "Failed to encode ledger state",
            e,
        )
    })?;
    write_file(&path, &content)
}

pub fn init() -> Result<LedgerState, AppError> {
    let ledger = LedgerState::new();
    persist(&ledger)?;
    Ok(ledger)
}

pub fn transfer(to: &str, amount: u64) -> Result<LedgerState, AppError> {
    let mut ledger = load()?;
    if ledger.treasury_balance < amount {
        return Err(AppError::new(
            ErrorCode::LedgerInvalid,
            "Treasury balance is insufficient for the requested transfer",
        ));
    }
    ledger.treasury_balance -= amount;
    ledger.transfers += 1;
    ledger.delegations.entry(to.to_string()).or_insert(0);
    ledger.updated_at = Utc::now().to_rfc3339();
    persist(&ledger)?;
    Ok(ledger)
}

pub fn delegate(validator: &str, amount: u64) -> Result<LedgerState, AppError> {
    let mut ledger = load()?;
    if ledger.treasury_balance < amount {
        return Err(AppError::new(
            ErrorCode::LedgerInvalid,
            "Treasury balance is insufficient for delegation",
        ));
    }
    ledger.treasury_balance -= amount;
    *ledger.delegations.entry(validator.to_string()).or_insert(0) += amount;
    ledger.updated_at = Utc::now().to_rfc3339();
    persist(&ledger)?;
    Ok(ledger)
}

pub fn undelegate(validator: &str, amount: u64) -> Result<LedgerState, AppError> {
    let mut ledger = load()?;
    let entry = ledger.delegations.entry(validator.to_string()).or_insert(0);
    if *entry < amount {
        return Err(AppError::new(
            ErrorCode::LedgerInvalid,
            "Delegation balance is insufficient for undelegation",
        ));
    }
    *entry -= amount;
    ledger.treasury_balance += amount;
    ledger.updated_at = Utc::now().to_rfc3339();
    persist(&ledger)?;
    Ok(ledger)
}
