// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use crate::{
    data_home::{read_file, resolve_home},
    error::{AppError, ErrorCode},
    storage::redb_chain::{append_chain_log, load_ledger_state, persist_ledger_state},
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, path::PathBuf};

const INITIAL_TREASURY_BALANCE: u64 = 1_000_000_000_000;

/// Canonical AOXC ledger state.
///
/// Design intent:
/// - Preserve a compact operator-plane ledger view suitable for local treasury
///   and delegation lifecycle operations.
/// - Maintain deterministic validation at the storage boundary.
/// - Support legacy JSON migration into the canonical redb-backed ledger store.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
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
    /// Returns the canonical default AOXC ledger state.
    pub fn new() -> Self {
        Self {
            treasury_balance: INITIAL_TREASURY_BALANCE,
            transfers: 0,
            delegations: BTreeMap::new(),
            updated_at: Utc::now().to_rfc3339(),
        }
    }

    /// Validates semantic integrity for AOXC ledger state.
    ///
    /// Validation policy:
    /// - `updated_at` must be a valid RFC3339 timestamp.
    /// - Delegation targets must not be blank after trimming.
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

/// Returns the canonical legacy JSON ledger path.
///
/// Canonical legacy path:
/// - `<AOXC_HOME>/ledger/ledger.json`
///
/// Operational note:
/// - The redb-backed storage surface is authoritative.
/// - This JSON path exists for backward compatibility and one-time migration.
pub fn ledger_path() -> Result<PathBuf, AppError> {
    Ok(resolve_home()?.join("ledger").join("ledger.json"))
}

/// Loads ledger state from the canonical redb store and falls back to the
/// legacy JSON ledger only when a valid legacy payload is still present.
///
/// Migration policy:
/// - Canonical redb storage is preferred.
/// - When redb load fails and a legacy JSON ledger exists, the legacy payload
///   is loaded, validated, persisted into redb, and returned.
/// - If legacy JSON is absent, the original redb error is preserved.
/// - If legacy JSON exists but is invalid, the invalid legacy payload is
///   rejected explicitly.
pub fn load() -> Result<LedgerState, AppError> {
    match load_ledger_state() {
        Ok(ledger) => Ok(ledger),
        Err(primary_error) => try_load_legacy_ledger(primary_error),
    }
}

/// Persists validated ledger state into the canonical redb store.
///
/// Persistence policy:
/// - Semantic validation is enforced before write.
/// - Successful persistence appends a best-effort chain-log breadcrumb.
pub fn persist(ledger: &LedgerState) -> Result<(), AppError> {
    ledger
        .validate()
        .map_err(|error| AppError::new(ErrorCode::LedgerInvalid, error))?;

    persist_ledger_state(ledger)?;

    let _ = append_chain_log(
        "ledger",
        "persist_ledger_state",
        &format!(
            "treasury_balance={} transfers={}",
            ledger.treasury_balance, ledger.transfers
        ),
    );

    Ok(())
}

/// Initializes the canonical AOXC ledger with default balances.
pub fn init() -> Result<LedgerState, AppError> {
    let ledger = LedgerState::new();
    persist(&ledger)?;
    Ok(ledger)
}

/// Records a treasury transfer intent.
///
/// Current operational model:
/// - Treasury balance is reduced by `amount`.
/// - A delegation map entry for the target is materialized when absent so the
///   operator surface can observe the target in subsequent local state views.
pub fn transfer(to: &str, amount: u64) -> Result<LedgerState, AppError> {
    let target = normalize_required_subject(to, "transfer target")?;
    let mut ledger = load()?;

    if ledger.treasury_balance < amount {
        return Err(AppError::new(
            ErrorCode::LedgerInvalid,
            "Treasury balance is insufficient for the requested transfer",
        ));
    }

    ledger.treasury_balance -= amount;
    ledger.transfers = ledger.transfers.saturating_add(1);
    ledger.delegations.entry(target).or_insert(0);
    ledger.updated_at = Utc::now().to_rfc3339();

    persist(&ledger)?;
    Ok(ledger)
}

/// Delegates treasury balance to the supplied validator identifier.
pub fn delegate(validator: &str, amount: u64) -> Result<LedgerState, AppError> {
    let validator = normalize_required_subject(validator, "validator")?;
    let mut ledger = load()?;

    if ledger.treasury_balance < amount {
        return Err(AppError::new(
            ErrorCode::LedgerInvalid,
            "Treasury balance is insufficient for delegation",
        ));
    }

    ledger.treasury_balance -= amount;
    *ledger.delegations.entry(validator).or_insert(0) += amount;
    ledger.updated_at = Utc::now().to_rfc3339();

    persist(&ledger)?;
    Ok(ledger)
}

/// Undelegates balance from the supplied validator identifier.
pub fn undelegate(validator: &str, amount: u64) -> Result<LedgerState, AppError> {
    let validator = normalize_required_subject(validator, "validator")?;
    let mut ledger = load()?;

    let entry = ledger.delegations.entry(validator).or_insert(0);
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

/// Attempts to load and migrate the legacy JSON ledger payload.
fn try_load_legacy_ledger(primary_error: AppError) -> Result<LedgerState, AppError> {
    let path = ledger_path()?;
    let raw = match read_file(&path) {
        Ok(raw) => raw,
        Err(_) => return Err(primary_error),
    };

    let ledger: LedgerState = serde_json::from_str(&raw).map_err(|error| {
        AppError::with_source(
            ErrorCode::LedgerInvalid,
            format!("Failed to parse legacy ledger state at {}", path.display()),
            error,
        )
    })?;

    ledger
        .validate()
        .map_err(|error| AppError::new(ErrorCode::LedgerInvalid, error))?;

    persist_ledger_state(&ledger)?;

    let _ = append_chain_log(
        "ledger",
        "migrate_json_to_redb",
        "legacy ledger.json migrated",
    );

    Ok(ledger)
}

/// Normalizes required operator-facing ledger subjects.
fn normalize_required_subject(value: &str, field: &str) -> Result<String, AppError> {
    let normalized = value.split_whitespace().collect::<Vec<_>>().join(" ");
    if normalized.is_empty() {
        return Err(AppError::new(
            ErrorCode::UsageInvalidArguments,
            format!("Ledger {} must not be blank", field),
        ));
    }
    Ok(normalized)
}

#[cfg(test)]
mod tests {
    use super::{LedgerState, delegate, load, persist, transfer, undelegate};
    use crate::{
        error::ErrorCode,
        test_support::{AoxcHomeGuard, TestHome, aoxc_home_test_lock},
    };

    fn with_test_home<T>(label: &str, test: impl FnOnce(&TestHome) -> T) -> T {
        let _lock = aoxc_home_test_lock();
        let home = TestHome::new(label);
        let _guard = AoxcHomeGuard::install(home.path());
        test(&home)
    }

    #[test]
    fn persist_rejects_blank_delegation_targets() {
        with_test_home("ledger-invalid-persist", |_home| {
            let mut ledger = LedgerState::new();
            ledger.delegations.insert(" ".to_string(), 10);

            let error = persist(&ledger).expect_err("blank delegation target should be rejected");
            assert_eq!(error.code(), ErrorCode::LedgerInvalid.as_str());
        });
    }

    #[test]
    fn load_rejects_invalid_timestamp() {
        with_test_home("ledger-invalid-load", |home| {
            let mut ledger = LedgerState::new();
            ledger.updated_at = "not-a-timestamp".to_string();

            let path = home.path().join("ledger").join("ledger.json");
            std::fs::create_dir_all(path.parent().expect("parent must exist"))
                .expect("ledger dir should create");
            std::fs::write(
                &path,
                serde_json::to_string_pretty(&ledger).expect("ledger should encode"),
            )
            .expect("ledger should write");

            let error = load().expect_err("invalid timestamp should be rejected");
            assert_eq!(error.code(), ErrorCode::LedgerInvalid.as_str());
        });
    }

    #[test]
    fn transfer_rejects_blank_target() {
        with_test_home("ledger-transfer-blank-target", |_home| {
            let error = transfer("   ", 10).expect_err("blank transfer target must fail");
            assert_eq!(error.code(), ErrorCode::UsageInvalidArguments.as_str());
        });
    }

    #[test]
    fn delegate_and_undelegate_round_trip_balance() {
        with_test_home("ledger-delegate-undelegate", |_home| {
            let initial = LedgerState::new();
            persist(&initial).expect("initial ledger should persist");

            let delegated = delegate("validator-01", 100).expect("delegation should succeed");
            assert_eq!(delegated.treasury_balance, initial.treasury_balance - 100);
            assert_eq!(delegated.delegations.get("validator-01"), Some(&100));

            let undelegated = undelegate("validator-01", 40).expect("undelegation should succeed");
            assert_eq!(undelegated.treasury_balance, initial.treasury_balance - 60);
            assert_eq!(undelegated.delegations.get("validator-01"), Some(&60));
        });
    }

    #[test]
    fn transfer_increments_transfer_counter() {
        with_test_home("ledger-transfer-counter", |_home| {
            let initial = LedgerState::new();
            persist(&initial).expect("initial ledger should persist");

            let transferred = transfer("ops", 25).expect("transfer should succeed");
            assert_eq!(transferred.transfers, 1);
        });
    }
}
