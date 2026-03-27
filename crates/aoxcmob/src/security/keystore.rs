// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use crate::error::MobError;
use crate::types::DeviceProfile;
use ed25519_dalek::SigningKey;
use std::sync::Mutex;

/// Secure storage boundary for mobile device credentials.
///
/// In production mobile applications this trait should be backed by platform
/// security primitives such as Android Keystore or iOS Keychain / Secure Enclave.
pub trait SecureStore: Send + Sync {
    fn store_device(&self, profile: DeviceProfile, signing_key: SigningKey)
    -> Result<(), MobError>;
    fn load_device_profile(&self) -> Result<DeviceProfile, MobError>;
    fn load_signing_key(&self) -> Result<SigningKey, MobError>;
    fn is_provisioned(&self) -> Result<bool, MobError>;
}

#[derive(Debug, Clone)]
struct StoredDevice {
    profile: DeviceProfile,
    signing_key_bytes: [u8; 32],
}

/// In-memory reference implementation used for tests and local development.
#[derive(Debug, Default)]
pub struct InMemorySecureStore {
    inner: Mutex<Option<StoredDevice>>,
}

impl InMemorySecureStore {
    #[must_use]
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(None),
        }
    }
}

impl SecureStore for InMemorySecureStore {
    fn store_device(
        &self,
        profile: DeviceProfile,
        signing_key: SigningKey,
    ) -> Result<(), MobError> {
        let mut guard = self
            .inner
            .lock()
            .map_err(|_| MobError::SecureStorePoisoned)?;
        if guard.is_some() {
            return Err(MobError::DeviceAlreadyProvisioned);
        }
        let stored = StoredDevice {
            profile,
            signing_key_bytes: signing_key.to_bytes(),
        };
        *guard = Some(stored);
        Ok(())
    }

    fn load_device_profile(&self) -> Result<DeviceProfile, MobError> {
        let guard = self
            .inner
            .lock()
            .map_err(|_| MobError::SecureStorePoisoned)?;
        guard
            .as_ref()
            .map(|value| value.profile.clone())
            .ok_or(MobError::DeviceNotProvisioned)
    }

    fn load_signing_key(&self) -> Result<SigningKey, MobError> {
        let guard = self
            .inner
            .lock()
            .map_err(|_| MobError::SecureStorePoisoned)?;
        let bytes = guard
            .as_ref()
            .map(|value| value.signing_key_bytes)
            .ok_or(MobError::DeviceNotProvisioned)?;
        Ok(SigningKey::from_bytes(&bytes))
    }

    fn is_provisioned(&self) -> Result<bool, MobError> {
        let guard = self
            .inner
            .lock()
            .map_err(|_| MobError::SecureStorePoisoned)?;
        Ok(guard.is_some())
    }
}
