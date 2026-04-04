// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use crate::config::RpcConfig;
use crate::error::RpcError;
use crate::grpc::services::{query_service::QueryService, tx_submission::TxSubmissionService};
use crate::types::{ChainStatus, TxSubmissionRequest, TxSubmissionResult};

/// gRPC server entry point.
#[derive(Debug, Clone)]
pub struct GrpcServer {
    pub config: RpcConfig,
    pub query_service: QueryService,
    pub tx_submission_service: TxSubmissionService,
}

impl GrpcServer {
    #[must_use]
    pub fn new(config: RpcConfig) -> Self {
        Self {
            config,
            query_service: QueryService::default(),
            tx_submission_service: TxSubmissionService::default(),
        }
    }

    pub fn startup_checks(&self) -> Result<(), RpcError> {
        if self.config.tls_cert_path.trim().is_empty() || self.config.tls_key_path.trim().is_empty()
        {
            return Err(RpcError::InternalError);
        }

        let validation = self.config.validate();

        if validation
            .errors
            .iter()
            .any(|error| error.contains("grpc_bind_addr") || error.contains("chain_id"))
        {
            return Err(RpcError::InvalidRequest);
        }

        if validation
            .errors
            .iter()
            .any(|error| error.contains("tls") || error.contains("mTLS"))
        {
            return Err(RpcError::InternalError);
        }

        Ok(())
    }

    #[must_use]
    pub fn method_catalog(&self) -> Vec<&'static str> {
        vec!["query.GetChainStatus", "tx.Submit"]
    }

    pub fn query_chain_status(&self, height: u64, syncing: bool) -> Result<ChainStatus, RpcError> {
        self.startup_checks()?;
        Ok(self.query_service.get_chain_status(height, syncing))
    }

    pub fn submit_tx(&self, request: TxSubmissionRequest) -> Result<TxSubmissionResult, RpcError> {
        self.startup_checks()?;
        self.tx_submission_service.submit(request)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn startup_checks_fail_for_bad_grpc_bind_addr() {
        let config = RpcConfig {
            grpc_bind_addr: "bad-addr".to_string(),
            ..RpcConfig::default()
        };
        let server = GrpcServer::new(config);

        let result = server.startup_checks();
        assert!(matches!(result, Err(RpcError::InvalidRequest)));
    }

    #[test]
    fn startup_checks_fail_for_missing_tls_files() {
        let config = RpcConfig {
            genesis_hash: Some(format!("0x{}", "ab".repeat(32))),
            tls_cert_path: "".to_string(),
            tls_key_path: "".to_string(),
            ..RpcConfig::default()
        };

        let server = GrpcServer::new(config);
        let result = server.startup_checks();

        assert!(matches!(result, Err(RpcError::InternalError)));
    }

    #[test]
    fn method_catalog_contains_core_surface() {
        let config = RpcConfig {
            genesis_hash: Some(format!("0x{}", "ab".repeat(32))),
            tls_cert_path: "Cargo.toml".to_string(),
            tls_key_path: "README.md".to_string(),
            mtls_ca_cert_path: Some("Cargo.toml".to_string()),
            ..RpcConfig::default()
        };

        let server = GrpcServer::new(config);
        let methods = server.method_catalog();
        assert!(methods.contains(&"query.GetChainStatus"));
        assert!(methods.contains(&"tx.Submit"));
    }
}
