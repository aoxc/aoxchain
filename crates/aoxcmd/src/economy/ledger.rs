// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

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

    pub fn validate(&self) -> Result<(), String> {
        chrono::DateTime::parse_from_rfc3339(&self.updated_at)
            .map_err(|_| "updated_at must be a valid RFC3339 timestamp".to_string())?;
        if self
            .delegations
            .keys()
            .any(|validator| validator.trim().is_empty())
        {
            return Err("delegations cannot contain blank validator ids".to_string());
        }
        Ok(())
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
    let ledger: LedgerState = serde_json::from_str(&raw).map_err(|e| {
        AppError::with_source(ErrorCode::LedgerInvalid, "Failed to parse ledger state", e)
    })?;
    ledger
        .validate()
        .map_err(|e| AppError::new(ErrorCode::LedgerInvalid, e))?;
    Ok(ledger)
}

pub fn persist(ledger: &LedgerState) -> Result<(), AppError> {
    ledger
        .validate()
        .map_err(|e| AppError::new(ErrorCode::LedgerInvalid, e))?;
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

#[cfg(test)]
mod tests {
    use super::{load, persist, LedgerState};
    use crate::error::ErrorCode;
    use std::{
        env, fs,
        path::PathBuf,
        sync::{Mutex, OnceLock},
        time::{SystemTime, UNIX_EPOCH},
    };

    fn env_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    fn unique_test_home(label: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after unix epoch")
            .as_nanos();
        env::temp_dir().join(format!("aoxcmd-{label}-{nanos}"))
    }

    #[test]
    fn persist_rejects_blank_delegation_targets() {
        let _guard = env_lock().lock().expect("test env mutex must lock");
        let home = unique_test_home("ledger-invalid-persist");
        env::set_var("AOXC_HOME", &home);

        let mut ledger = LedgerState::new();
        ledger.delegations.insert(" ".to_string(), 10);

        let error = persist(&ledger).expect_err("blank delegation target should be rejected");
        assert_eq!(error.code(), ErrorCode::LedgerInvalid.as_str());

        let _ = fs::remove_dir_all(&home);
        env::remove_var("AOXC_HOME");
    }

    #[test]
    fn load_rejects_invalid_timestamp() {
        let _guard = env_lock().lock().expect("test env mutex must lock");
        let home = unique_test_home("ledger-invalid-load");
        env::set_var("AOXC_HOME", &home);

        let mut ledger = LedgerState::new();
        ledger.updated_at = "not-a-timestamp".to_string();
        let path = home.join("ledger").join("ledger.json");
        std::fs::create_dir_all(path.parent().expect("parent must exist"))
            .expect("ledger dir should create");
        std::fs::write(
            &path,
            serde_json::to_string_pretty(&ledger).expect("ledger should encode"),
        )
        .expect("ledger should write");

        let error = load().expect_err("invalid timestamp should be rejected");
        assert_eq!(error.code(), ErrorCode::LedgerInvalid.as_str());

        let _ = fs::remove_dir_all(&home);
        env::remove_var("AOXC_HOME");
    }
}
