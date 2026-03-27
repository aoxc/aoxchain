use crate::config::RpcConfig;
use crate::error::RpcError;
use crate::grpc::services::{query_service::QueryService, tx_submission::TxSubmissionService};

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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn startup_checks_fail_for_bad_grpc_bind_addr() {
        let mut config = RpcConfig::default();
        config.grpc_bind_addr = "bad-addr".to_string();
        let server = GrpcServer::new(config);

        let result = server.startup_checks();
        assert!(matches!(result, Err(RpcError::InvalidRequest)));
    }

    #[test]
    fn startup_checks_fail_for_missing_tls_files() {
        let mut config = RpcConfig::default();
        config.genesis_hash = Some(format!("0x{}", "ab".repeat(32)));
        config.tls_cert_path = "".to_string();
        config.tls_key_path = "".to_string();

        let server = GrpcServer::new(config);
        let result = server.startup_checks();

        assert!(matches!(result, Err(RpcError::InternalError)));
    }
}
