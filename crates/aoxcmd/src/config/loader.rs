use crate::{
    config::settings::Settings,
    data_home::{read_file, resolve_home, write_file},
    error::{AppError, ErrorCode},
};
use std::path::PathBuf;

pub fn settings_path() -> Result<PathBuf, AppError> {
    Ok(resolve_home()?.join("config").join("settings.json"))
}

pub fn init_default() -> Result<Settings, AppError> {
    let home = resolve_home()?;
    let settings = Settings::default_for(home.display().to_string());
    persist(&settings)?;
    Ok(settings)
}

pub fn load() -> Result<Settings, AppError> {
    let path = settings_path()?;
    let raw = read_file(&path).map_err(|_| {
        AppError::new(
            ErrorCode::ConfigMissing,
            format!("Configuration file is missing at {}", path.display()),
        )
    })?;
    let settings: Settings = serde_json::from_str(&raw).map_err(|e| {
        AppError::with_source(
            ErrorCode::ConfigInvalid,
            "Failed to parse configuration file",
            e,
        )
    })?;
    settings
        .validate()
        .map_err(|e| AppError::new(ErrorCode::ConfigInvalid, e))?;
    Ok(settings)
}

pub fn load_or_init() -> Result<Settings, AppError> {
    match load() {
        Ok(settings) => Ok(settings),
        Err(error) if error.code() == ErrorCode::ConfigMissing.as_str() => init_default(),
        Err(error) => Err(error),
    }
}

pub fn persist(settings: &Settings) -> Result<(), AppError> {
    settings
        .validate()
        .map_err(|e| AppError::new(ErrorCode::ConfigInvalid, e))?;
    let path = settings_path()?;
    let content = serde_json::to_string_pretty(settings).map_err(|e| {
        AppError::with_source(
            ErrorCode::OutputEncodingFailed,
            "Failed to encode configuration",
            e,
        )
    })?;
    write_file(&path, &content)
}
