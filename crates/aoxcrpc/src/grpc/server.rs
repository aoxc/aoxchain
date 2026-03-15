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
        if self.config.grpc_bind_addr.trim().is_empty() {
            return Err(RpcError::InvalidRequest);
        }

        if self.config.tls_cert_path.trim().is_empty() || self.config.tls_key_path.trim().is_empty()
        {
            return Err(RpcError::InternalError);
        }

        Ok(())
    }
}
