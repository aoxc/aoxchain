use crate::{
    config::settings::Settings,
    data_home::{read_file, resolve_home, write_file},
    error::{AppError, ErrorCode},
};
use std::path::PathBuf;

/// Returns the canonical AOXC CLI settings path.
///
/// The loader intentionally stores the effective CLI settings document under:
/// `<home>/config/settings.json`
///
/// This location is treated as the operator-facing runtime settings surface
/// and is validated before persistence and after loading.
pub fn settings_path() -> Result<PathBuf, AppError> {
    Ok(resolve_home()?.join("config").join("settings.json"))
}

/// Creates and persists the canonical default AOXC settings document.
///
/// The modern safe default is the `validation` profile, as defined by
/// `Settings::default_for(...)`.
pub fn init_default() -> Result<Settings, AppError> {
    let home = resolve_home()?;
    let settings = Settings::default_for(home.display().to_string());
    persist(&settings)?;
    Ok(settings)
}

/// Creates and persists profile-specific AOXC settings.
///
/// This helper is suitable for environment-aware bootstrap flows where the
/// caller already resolved the intended canonical profile name.
pub fn init_for_profile(profile: &str) -> Result<Settings, AppError> {
    let home = resolve_home()?;
    let settings = Settings::default_for_profile(home.display().to_string(), profile)
        .map_err(|e| AppError::new(ErrorCode::ConfigInvalid, e))?;
    persist(&settings)?;
    Ok(settings)
}

/// Loads and validates the AOXC settings document from disk.
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

/// Loads settings if present, otherwise initializes canonical defaults.
///
/// This helper intentionally only falls back to initialization when the
/// configuration is missing. Invalid configuration remains a hard failure
/// because silently overwriting an operator-edited document would be unsafe.
pub fn load_or_init() -> Result<Settings, AppError> {
    match load() {
        Ok(settings) => Ok(settings),
        Err(error) if error.code() == ErrorCode::ConfigMissing.as_str() => init_default(),
        Err(error) => Err(error),
    }
}

/// Loads settings if present, otherwise initializes a requested canonical profile.
///
/// This is useful for profile-aware bootstrap flows that want missing config
/// creation to align with the target environment rather than the generic
/// validation default.
pub fn load_or_init_for_profile(profile: &str) -> Result<Settings, AppError> {
    match load() {
        Ok(settings) => Ok(settings),
        Err(error) if error.code() == ErrorCode::ConfigMissing.as_str() => {
            init_for_profile(profile)
        }
        Err(error) => Err(error),
    }
}

/// Persists a validated AOXC settings document to disk.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settings_path_points_to_config_settings_json() {
        let path = settings_path().expect("settings path must resolve");
        let rendered = path.display().to_string();

        assert!(rendered.ends_with("config/settings.json"));
    }

    #[test]
    fn init_default_creates_validation_profile_settings() {
        let settings = init_default().expect("default settings initialization must succeed");
        assert_eq!(settings.profile, "validation");
    }

    #[test]
    fn init_for_profile_accepts_validation_alias() {
        let settings = init_for_profile("validator")
            .expect("legacy validator alias initialization must succeed");
        assert_eq!(settings.profile, "validation");
    }

    #[test]
    fn persist_roundtrip_preserves_settings() {
        let original = Settings::default_for("/tmp/aoxc-loader".to_string());

        persist(&original).expect("settings persistence must succeed");
        let loaded = load().expect("settings load must succeed");

        assert_eq!(original, loaded);
    }

    #[test]
    fn load_or_init_for_profile_initializes_requested_profile_when_missing() {
        let settings =
            load_or_init_for_profile("devnet").expect("profile-aware load or init must succeed");

        assert_eq!(settings.profile, "devnet");
    }
}
