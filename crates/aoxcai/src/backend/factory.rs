use crate::{
    backend::{heuristic::HeuristicBackendRuntime, remote_http::RemoteHttpBackendRuntime},
    error::AiError,
    manifest::ModelManifest,
    traits::InferenceBackend,
};

/// Constructs concrete inference backend instances from manifest configuration.
///
/// This factory is the sole runtime entry point for backend materialization.
/// It translates the manifest-declared backend type into a concrete backend
/// implementation and enforces configuration presence for the selected driver.
pub struct BackendFactory;

impl BackendFactory {
    /// Builds an inference backend from the supplied manifest.
    ///
    /// The selected backend is derived from `manifest.spec.backend.type`.
    /// Unsupported or partially configured backends are rejected with a
    /// manifest or backend validation error before execution begins.
    pub fn build(manifest: &ModelManifest) -> Result<Box<dyn InferenceBackend>, AiError> {
        match manifest.spec.backend.r#type.as_str() {
            "heuristic" => {
                ensure_heuristic_config_present(manifest)?;
                Ok(Box::new(HeuristicBackendRuntime::new()))
            }
            "remote_http" => Ok(Box::new(RemoteHttpBackendRuntime::new(manifest)?)),
            other => Err(AiError::UnsupportedBackend(other.to_owned())),
        }
    }
}

/// Ensures that heuristic backend configuration is present when the manifest
/// selects the heuristic runtime.
fn ensure_heuristic_config_present(manifest: &ModelManifest) -> Result<(), AiError> {
    if manifest.spec.backend.heuristic.is_none() {
        return Err(AiError::ManifestValidation(
            "heuristic backend requires spec.backend.heuristic".to_owned(),
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::base_manifest;

    #[test]
    fn heuristic_backend_is_constructed() {
        let manifest = base_manifest();
        let backend =
            BackendFactory::build(&manifest).expect("heuristic backend must be constructed");

        assert_eq!(backend.name(), "heuristic");
    }

    #[test]
    fn unsupported_backend_is_rejected() {
        let mut manifest = base_manifest();
        manifest.spec.backend.r#type = "unknown".to_owned();
        manifest.spec.backend.heuristic = None;

        let err = match BackendFactory::build(&manifest) {
            Ok(_) => panic!("expected unsupported backend error"),
            Err(err) => err,
        };

        match err {
            AiError::UnsupportedBackend(value) => assert_eq!(value, "unknown"),
            other => panic!("unexpected error: {other}"),
        }
    }

    #[test]
    fn heuristic_backend_requires_embedded_configuration() {
        let mut manifest = base_manifest();
        manifest.spec.backend.heuristic = None;

        let err = match BackendFactory::build(&manifest) {
            Ok(_) => panic!("expected manifest validation error"),
            Err(err) => err,
        };

        match err {
            AiError::ManifestValidation(message) => {
                assert_eq!(message, "heuristic backend requires spec.backend.heuristic");
            }
            other => panic!("unexpected error: {other}"),
        }
    }
}

#[test]
fn remote_http_backend_is_constructed() {
    let manifest = crate::test_support::remote_http_manifest("http://127.0.0.1:8088/infer");

    let backend =
        BackendFactory::build(&manifest).expect("remote_http backend must be constructed");

    assert_eq!(backend.name(), "remote_http");
}

#[test]
fn remote_http_backend_requires_embedded_configuration() {
    let mut manifest = crate::test_support::base_manifest();
    manifest.spec.backend.r#type = "remote_http".to_owned();
    manifest.spec.backend.heuristic = None;
    manifest.spec.backend.remote_http = None;

    let err = match BackendFactory::build(&manifest) {
        Ok(_) => panic!("expected manifest validation error"),
        Err(err) => err,
    };

    match err {
        AiError::ManifestValidation(message) => {
            assert_eq!(
                message,
                "remote_http backend requires spec.backend.remote_http"
            );
        }
        other => panic!("unexpected error: {other}"),
    }
}
