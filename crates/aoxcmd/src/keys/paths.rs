use std::path::{Path, PathBuf};

/// Canonical filesystem paths used by the AOXC key lifecycle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyPaths {
    pub secret_keyfile_path: PathBuf,
    pub certificate_path: PathBuf,
    pub passport_path: PathBuf,
}

impl KeyPaths {
    /// Builds canonical key paths from a base directory and logical key name.
    ///
    /// Example output for `name = "node"`:
    /// - `<base>/node.key`
    /// - `<base>/node.cert.json`
    /// - `<base>/node.passport.json`
    #[must_use]
    pub fn new(base_dir: impl AsRef<Path>, name: &str) -> Self {
        let base_dir = base_dir.as_ref();

        Self {
            secret_keyfile_path: base_dir.join(format!("{name}.key")),
            certificate_path: base_dir.join(format!("{name}.cert.json")),
            passport_path: base_dir.join(format!("{name}.passport.json")),
        }
    }

    /// Returns the parent directory that must exist before persistence.
    #[must_use]
    pub fn base_dir(&self) -> PathBuf {
        self.secret_keyfile_path
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| PathBuf::from("."))
    }
}
