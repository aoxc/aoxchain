// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use crate::{
    backend::{heuristic::HeuristicBackendRuntime, remote_http::RemoteHttpBackendRuntime},
    error::AiError,
    manifest::{BackendType, ModelManifest},
    traits::InferenceBackend,
};

/// Constructs concrete backend instances from validated manifest configuration.
pub struct BackendFactory;

impl BackendFactory {
    pub fn build(manifest: &ModelManifest) -> Result<Box<dyn InferenceBackend>, AiError> {
        match manifest.spec.backend.r#type {
            BackendType::Heuristic => Ok(Box::new(HeuristicBackendRuntime::new())),
            BackendType::RemoteHttp => Ok(Box::new(RemoteHttpBackendRuntime::new(manifest)?)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::{base_manifest, remote_http_manifest};

    #[test]
    fn heuristic_backend_is_constructed() {
        let manifest = base_manifest();
        let backend = BackendFactory::build(&manifest).expect("heuristic backend must be built");
        assert_eq!(backend.name(), "heuristic");
    }

    #[test]
    fn remote_http_backend_is_constructed() {
        let manifest = remote_http_manifest("https://inference.aoxc.local/infer");
        let backend = BackendFactory::build(&manifest).expect("remote backend must be built");
        assert_eq!(backend.name(), "remote_http");
    }
}
