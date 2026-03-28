// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use std::{
    error::Error as StdError,
    fmt::{Display, Formatter},
    io::ErrorKind,
};

/// Canonical AOXC application error code taxonomy.
///
/// Design intent:
/// - Provide stable machine-readable identifiers for operator-facing failures.
/// - Preserve deterministic exit-code grouping for shell automation and CI.
/// - Separate semantic error identity from human-readable message text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCode {
    UsageUnknownCommand,
    UsageInvalidArguments,
    HomeResolutionFailed,
    FilesystemIoFailed,
    ConfigMissing,
    ConfigInvalid,
    KeyMaterialMissing,
    KeyMaterialInvalid,
    GenesisInvalid,
    LedgerInvalid,
    NodeStateInvalid,
    PolicyGateFailed,
    NetworkProbeFailed,
    AuditFailure,
    OutputEncodingFailed,
}

impl ErrorCode {
    /// Returns the stable AOXC string code associated with this error class.
    ///
    /// Stability contract:
    /// - Values returned by this function are part of the operator-facing
    ///   contract and must not be changed without an explicit compatibility
    ///   migration decision.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::UsageUnknownCommand => "AOXC-USG-001",
            Self::UsageInvalidArguments => "AOXC-USG-002",
            Self::HomeResolutionFailed => "AOXC-HOM-001",
            Self::FilesystemIoFailed => "AOXC-FS-001",
            Self::ConfigMissing => "AOXC-CFG-001",
            Self::ConfigInvalid => "AOXC-CFG-002",
            Self::KeyMaterialMissing => "AOXC-KEY-001",
            Self::KeyMaterialInvalid => "AOXC-KEY-002",
            Self::GenesisInvalid => "AOXC-GEN-001",
            Self::LedgerInvalid => "AOXC-LED-001",
            Self::NodeStateInvalid => "AOXC-NOD-001",
            Self::PolicyGateFailed => "AOXC-POL-001",
            Self::NetworkProbeFailed => "AOXC-NET-001",
            Self::AuditFailure => "AOXC-AUD-001",
            Self::OutputEncodingFailed => "AOXC-OUT-001",
        }
    }

    /// Returns the canonical process exit code for this error class.
    ///
    /// Exit-code policy:
    /// - Related operator failure families intentionally share the same process
    ///   exit code so shell automation can react at the correct abstraction
    ///   level without parsing human-readable text.
    pub const fn exit_code(self) -> i32 {
        match self {
            Self::UsageUnknownCommand | Self::UsageInvalidArguments => 2,
            Self::HomeResolutionFailed | Self::FilesystemIoFailed => 3,
            Self::ConfigMissing | Self::ConfigInvalid => 4,
            Self::KeyMaterialMissing | Self::KeyMaterialInvalid => 5,
            Self::GenesisInvalid => 6,
            Self::LedgerInvalid | Self::NodeStateInvalid => 7,
            Self::PolicyGateFailed | Self::NetworkProbeFailed | Self::AuditFailure => 8,
            Self::OutputEncodingFailed => 9,
        }
    }
}

/// Canonical AOXC application error envelope.
///
/// Design intent:
/// - Preserve a stable machine-readable error code.
/// - Preserve a human-readable operator message.
/// - Optionally retain an underlying source error for diagnostics while keeping
///   display output compact and deterministic.
#[derive(Debug)]
pub struct AppError {
    code: ErrorCode,
    message: String,
    source: Option<Box<dyn StdError + Send + Sync + 'static>>,
}

impl AppError {
    /// Constructs an application error without an underlying source error.
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            source: None,
        }
    }

    /// Constructs an application error that preserves an underlying source
    /// error for diagnostics and chained error reporting.
    pub fn with_source(
        code: ErrorCode,
        message: impl Into<String>,
        source: impl StdError + Send + Sync + 'static,
    ) -> Self {
        Self {
            code,
            message: message.into(),
            source: Some(Box::new(source)),
        }
    }

    /// Returns the stable AOXC string code.
    pub const fn code(&self) -> &'static str {
        self.code.as_str()
    }

    /// Returns the canonical process exit code associated with this error.
    pub const fn exit_code(&self) -> i32 {
        self.code.exit_code()
    }

    /// Returns the strongly typed error code.
    pub const fn kind(&self) -> ErrorCode {
        self.code
    }

    /// Returns the human-readable operator message.
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Returns whether this error wraps an underlying `std::io::Error` with the
    /// provided `ErrorKind`.
    pub fn has_io_error_kind(&self, kind: ErrorKind) -> bool {
        self.source
            .as_deref()
            .and_then(|error| error.downcast_ref::<std::io::Error>())
            .is_some_and(|io_error| io_error.kind() == kind)
    }
}

impl Display for AppError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self.source {
            Some(source) => write!(f, "[{}] {}: {}", self.code.as_str(), self.message, source),
            None => write!(f, "[{}] {}", self.code.as_str(), self.message),
        }
    }
}

impl StdError for AppError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        self.source
            .as_deref()
            .map(|error| error as &(dyn StdError + 'static))
    }
}

#[cfg(test)]
mod tests {
    use super::{AppError, ErrorCode};

    #[test]
    fn config_missing_and_config_invalid_codes_match_canonical_contract() {
        assert_eq!(ErrorCode::ConfigMissing.as_str(), "AOXC-CFG-001");
        assert_eq!(ErrorCode::ConfigInvalid.as_str(), "AOXC-CFG-002");
    }

    #[test]
    fn exit_codes_group_related_error_families() {
        assert_eq!(ErrorCode::UsageUnknownCommand.exit_code(), 2);
        assert_eq!(ErrorCode::FilesystemIoFailed.exit_code(), 3);
        assert_eq!(ErrorCode::ConfigMissing.exit_code(), 4);
        assert_eq!(ErrorCode::ConfigInvalid.exit_code(), 4);
        assert_eq!(ErrorCode::KeyMaterialInvalid.exit_code(), 5);
        assert_eq!(ErrorCode::GenesisInvalid.exit_code(), 6);
        assert_eq!(ErrorCode::LedgerInvalid.exit_code(), 7);
        assert_eq!(ErrorCode::PolicyGateFailed.exit_code(), 8);
        assert_eq!(ErrorCode::OutputEncodingFailed.exit_code(), 9);
    }

    #[test]
    fn app_error_display_without_source_is_stable() {
        let error = AppError::new(ErrorCode::ConfigMissing, "Configuration file is missing");

        assert_eq!(
            format!("{error}"),
            "[AOXC-CFG-001] Configuration file is missing"
        );
    }

    #[test]
    fn app_error_display_with_source_includes_source_message() {
        let error = AppError::with_source(
            ErrorCode::FilesystemIoFailed,
            "Failed to read file",
            std::io::Error::other("permission denied"),
        );

        let rendered = format!("{error}");
        assert!(rendered.contains("[AOXC-FS-001]"));
        assert!(rendered.contains("Failed to read file"));
        assert!(rendered.contains("permission denied"));
    }

    #[test]
    fn app_error_exposes_code_kind_message_and_exit_code() {
        let error = AppError::new(ErrorCode::LedgerInvalid, "Ledger validation failed");

        assert_eq!(error.kind(), ErrorCode::LedgerInvalid);
        assert_eq!(error.code(), "AOXC-LED-001");
        assert_eq!(error.message(), "Ledger validation failed");
        assert_eq!(error.exit_code(), 7);
    }
}
