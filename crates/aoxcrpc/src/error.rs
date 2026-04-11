// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use aoxchal::crypto_profile::AdmissionFailure as ChalAdmissionFailure;
use thiserror::Error;

use crate::types::RpcErrorResponse;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MethodAdmissionFailure {
    IdentityTierTooLow,
    UnsupportedMethod,
    InvalidSignerSet,
    BudgetExhausted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CryptoAdmissionFailure {
    InsufficientSignatureCount,
    MissingClassicalSignature,
    MissingPostQuantumSignature,
    ClassicalNotPermitted,
}

impl From<ChalAdmissionFailure> for CryptoAdmissionFailure {
    fn from(value: ChalAdmissionFailure) -> Self {
        match value {
            ChalAdmissionFailure::InsufficientSignatureCount => Self::InsufficientSignatureCount,
            ChalAdmissionFailure::MissingClassicalSignature => Self::MissingClassicalSignature,
            ChalAdmissionFailure::MissingPostQuantumSignature => Self::MissingPostQuantumSignature,
            ChalAdmissionFailure::ClassicalNotPermitted => Self::ClassicalNotPermitted,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdmissionFailure {
    Method(MethodAdmissionFailure),
    Crypto(CryptoAdmissionFailure),
}

/// RPC subsystem errors.
#[derive(Debug, Error)]
pub enum RpcError {
    #[error("INVALID_REQUEST")]
    InvalidRequest,
    #[error("METHOD_NOT_FOUND")]
    MethodNotFound,
    #[error("RATE_LIMIT_EXCEEDED: retry_after_ms={retry_after_ms}")]
    RateLimitExceeded { retry_after_ms: u64 },
    #[error("MTLS_AUTH_FAILED")]
    MtlsAuthFailed,
    #[error("API_KEY_AUTH_FAILED")]
    ApiKeyAuthFailed,
    #[error("ZKP_VALIDATION_FAILED: {0}")]
    ZkpValidationFailed(String),
    #[error("INTERNAL_ERROR")]
    InternalError,
    #[error("ADMISSION_DENIED: {message} ({code:?})")]
    AdmissionDenied {
        code: AdmissionFailure,
        message: &'static str,
    },
    #[error("PAYLOAD_TOO_LARGE")]
    PayloadTooLarge,
}

impl RpcError {
    #[must_use]
    pub fn code(&self) -> &'static str {
        match self {
            Self::InvalidRequest => "INVALID_REQUEST",
            Self::MethodNotFound => "METHOD_NOT_FOUND",
            Self::RateLimitExceeded { .. } => "RATE_LIMIT_EXCEEDED",
            Self::MtlsAuthFailed => "MTLS_AUTH_FAILED",
            Self::ApiKeyAuthFailed => "API_KEY_AUTH_FAILED",
            Self::ZkpValidationFailed(_) => "ZKP_VALIDATION_FAILED",
            Self::InternalError => "INTERNAL_ERROR",
            Self::AdmissionDenied { .. } => "ADMISSION_DENIED",
            Self::PayloadTooLarge => "PAYLOAD_TOO_LARGE",
        }
    }

    #[must_use]
    pub fn retry_after_ms(&self) -> Option<u64> {
        match self {
            Self::RateLimitExceeded { retry_after_ms } => Some(*retry_after_ms),
            _ => None,
        }
    }

    #[must_use]
    pub fn user_hint(&self) -> Option<&'static str> {
        match self {
            Self::InvalidRequest => Some("Fix request schema and required fields."),
            Self::MethodNotFound => Some("Use a supported RPC method and API version."),
            Self::RateLimitExceeded { .. } => {
                Some("Apply retry_after_ms with exponential backoff and jitter.")
            }
            Self::MtlsAuthFailed => Some("Verify client certificate chain and mTLS setup."),
            Self::ApiKeyAuthFailed => Some("Provide a valid API key in the Authorization header."),
            Self::ZkpValidationFailed(_) => Some("Regenerate and submit a valid ZKP proof."),
            Self::InternalError => None,
            Self::AdmissionDenied { .. } => Some(
                "Reduce method cost, upgrade identity tier, or satisfy required signer policy.",
            ),
            Self::PayloadTooLarge => {
                Some("Reduce payload size or split request content into smaller units.")
            }
        }
    }

    #[must_use]
    pub fn to_response(&self, request_id: Option<String>) -> RpcErrorResponse {
        RpcErrorResponse {
            code: self.code(),
            message: self.to_string(),
            retry_after_ms: self.retry_after_ms(),
            request_id,
            user_hint: self.user_hint().map(str::to_string),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rate_limit_error_contains_retry_after_in_response() {
        let err = RpcError::RateLimitExceeded {
            retry_after_ms: 250,
        };
        let response = err.to_response(Some("req-42".to_string()));

        assert_eq!(response.code, "RATE_LIMIT_EXCEEDED");
        assert_eq!(response.retry_after_ms, Some(250));
        assert_eq!(response.request_id.as_deref(), Some("req-42"));
        assert!(
            response
                .user_hint
                .as_deref()
                .is_some_and(|hint| hint.contains("retry_after_ms"))
        );
    }

    #[test]
    fn admission_error_exposes_structured_reason_code() {
        let response = RpcError::AdmissionDenied {
            code: AdmissionFailure::Method(MethodAdmissionFailure::InvalidSignerSet),
            message: "signer set does not satisfy VM auth profile",
        }
        .to_response(Some("req-admission".to_string()));

        assert_eq!(response.code, "ADMISSION_DENIED");
        assert!(response.message.contains("InvalidSignerSet"));
    }

    #[test]
    fn internal_error_has_no_user_hint_or_retry_after() {
        let response = RpcError::InternalError.to_response(Some("req-internal".to_string()));

        assert_eq!(response.code, "INTERNAL_ERROR");
        assert_eq!(response.retry_after_ms, None);
        assert_eq!(response.user_hint, None);
        assert_eq!(response.request_id.as_deref(), Some("req-internal"));
    }

    #[test]
    fn zkp_error_contains_actionable_user_hint() {
        let response = RpcError::ZkpValidationFailed("bad proof".to_string()).to_response(None);

        assert_eq!(response.code, "ZKP_VALIDATION_FAILED");
        assert!(
            response
                .user_hint
                .as_deref()
                .is_some_and(|hint| hint.contains("valid ZKP proof"))
        );
        assert!(response.message.contains("bad proof"));
    }

    #[test]
    fn payload_too_large_has_actionable_hint() {
        let response = RpcError::PayloadTooLarge.to_response(Some("req-big".to_string()));

        assert_eq!(response.code, "PAYLOAD_TOO_LARGE");
        assert_eq!(response.request_id.as_deref(), Some("req-big"));
        assert!(
            response
                .user_hint
                .as_deref()
                .is_some_and(|hint| hint.contains("Reduce payload size"))
        );
    }
}
