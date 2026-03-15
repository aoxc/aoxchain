use aoxcore::identity::ca::CertificateAuthority;

use super::loader::{KeyBootstrapRequest, KeyLoader, KeyLoaderError};
use super::material::KeyMaterial;
use super::paths::KeyPaths;

/// High-level facade used by `aoxcmd` runtime bootstrap to resolve local keys.
#[derive(Debug, Clone)]
pub struct KeyManager {
    paths: KeyPaths,
    request: KeyBootstrapRequest,
}

impl KeyManager {
    /// Creates a new key manager.
    #[must_use]
    pub fn new(paths: KeyPaths, request: KeyBootstrapRequest) -> Self {
        Self { paths, request }
    }

    /// Returns the canonical key paths managed by this instance.
    #[must_use]
    pub fn paths(&self) -> &KeyPaths {
        &self.paths
    }

    /// Returns the bootstrap request associated with this manager.
    #[must_use]
    pub fn request(&self) -> &KeyBootstrapRequest {
        &self.request
    }

    /// Loads existing key material or creates it if absent.
    pub fn load_or_create(&self, ca: &CertificateAuthority) -> Result<KeyMaterial, KeyLoaderError> {
        KeyLoader::load_or_create(&self.paths, &self.request, ca)
    }
}
