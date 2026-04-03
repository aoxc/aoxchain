use super::*;

pub(in crate::cli::ops) fn faucet_state_path() -> Result<PathBuf, AppError> {
    Ok(resolve_home()?.join("ledger").join("faucet_state.json"))
}

pub(in crate::cli::ops) fn load_faucet_state() -> Result<FaucetState, AppError> {
    let path = faucet_state_path()?;
    if !path.exists() {
        let state = FaucetState::default();
        persist_faucet_state(&state)?;
        return Ok(state);
    }

    let raw = fs::read_to_string(&path).map_err(|error| {
        AppError::with_source(
            ErrorCode::FilesystemIoFailed,
            format!("Failed to read faucet state from {}", path.display()),
            error,
        )
    })?;

    serde_json::from_str::<FaucetState>(&raw).map_err(|error| {
        AppError::with_source(
            ErrorCode::LedgerInvalid,
            format!("Failed to parse faucet state from {}", path.display()),
            error,
        )
    })
}

pub(in crate::cli::ops) fn persist_faucet_state(state: &FaucetState) -> Result<(), AppError> {
    let path = faucet_state_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| {
            AppError::with_source(
                ErrorCode::FilesystemIoFailed,
                format!(
                    "Failed to create faucet state directory {}",
                    parent.display()
                ),
                error,
            )
        })?;
    }

    let payload = serde_json::to_string_pretty(state).map_err(|error| {
        AppError::with_source(
            ErrorCode::OutputEncodingFailed,
            "Failed to encode faucet state",
            error,
        )
    })?;

    fs::write(&path, payload).map_err(|error| {
        AppError::with_source(
            ErrorCode::FilesystemIoFailed,
            format!("Failed to write faucet state to {}", path.display()),
            error,
        )
    })?;

    Ok(())
}
