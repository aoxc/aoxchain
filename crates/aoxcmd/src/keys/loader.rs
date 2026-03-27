// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use crate::{
    data_home::{read_file, write_file},
    error::{AppError, ErrorCode},
    keys::{material::KeyMaterial, paths::operator_key_path},
};

pub fn load_operator_key() -> Result<KeyMaterial, AppError> {
    let path = operator_key_path()?;
    let raw = read_file(&path).map_err(|_| {
        AppError::new(
            ErrorCode::KeyMaterialMissing,
            format!("Operator key material is missing at {}", path.display()),
        )
    })?;
    serde_json::from_str(&raw).map_err(|e| {
        AppError::with_source(
            ErrorCode::KeyMaterialInvalid,
            "Failed to parse operator key material",
            e,
        )
    })
}

pub fn persist_operator_key(material: &KeyMaterial) -> Result<(), AppError> {
    let path = operator_key_path()?;
    let content = serde_json::to_string_pretty(material).map_err(|e| {
        AppError::with_source(
            ErrorCode::OutputEncodingFailed,
            "Failed to encode key material",
            e,
        )
    })?;
    write_file(&path, &content)
}

#[cfg(test)]
mod tests {
    use super::{load_operator_key, persist_operator_key};
    use crate::keys::material::KeyMaterial;
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
    fn persist_and_reload_operator_key_round_trips_bundle() {
        let _guard = env_lock().lock().expect("test env mutex must lock");
        let home = unique_test_home("operator-key");
        env::set_var("AOXC_HOME", &home);

        let material = KeyMaterial::generate("validator-01", "testnet", "Test#2026!")
            .expect("key material generation should succeed");
        let fingerprint = material.fingerprint().to_string();

        persist_operator_key(&material).expect("operator key should persist");
        let reloaded = load_operator_key().expect("persisted operator key should reload");

        assert_eq!(reloaded.fingerprint(), fingerprint);
        assert_eq!(reloaded.bundle.keys.len(), material.bundle.keys.len());

        let _ = fs::remove_dir_all(&home);
        env::remove_var("AOXC_HOME");
    }
}
